//! Pipeline command implementation.

use clap::Args;
use rust_xlsxwriter::{Workbook, XlsxError};
use std::path::PathBuf;

use crate::contract::{ExitCode, SuccessContract};
use crate::date::RuleBasedDateNormalizer;
use crate::error::PipelineError;
use crate::models::{PdfInput, ReportType};
use crate::ocr::{MistralOcrProvider, OcrProvider};
use crate::processing::{ProcessingFacadeBuilder, ProcessingResult};
use crate::report_cleaning::{clean_report_tables_with_normalizer, CleanedTable};

const EXCEL_SHEET_NAME_MAX_CHARS: usize = 31;

/// Pipeline command arguments.
#[derive(Args, Debug)]
pub struct PipelineArgs {
    /// Input PDF URL or path.
    #[arg(short, long)]
    pub input_url: String,

    /// Report type (inferred from filename if not provided).
    #[arg(short, long)]
    pub report_type: Option<String>,

    /// Output JSON file path.
    #[arg(long)]
    pub output_json: Option<PathBuf>,

    /// Output XLSX file path.
    #[arg(long)]
    pub output_xlsx: Option<PathBuf>,

    /// Emit intermediate stage artifacts for debugging.
    #[arg(long)]
    pub emit_stage_artifacts: bool,

    /// Write JSON contract to file.
    #[arg(long)]
    pub emit_contract: Option<PathBuf>,
}

fn excel_safe_sheet_name(report_type: &ReportType) -> String {
    report_type
        .russian_name()
        .chars()
        .take(EXCEL_SHEET_NAME_MAX_CHARS)
        .collect()
}

fn flattened_xlsx_rows(tables: &[CleanedTable]) -> Vec<Vec<String>> {
    let Some(first_table) = tables.first() else {
        return Vec::new();
    };

    let mut rows =
        Vec::with_capacity(1 + tables.iter().map(|table| table.rows.len()).sum::<usize>());
    rows.push(first_table.headers.clone());
    rows.extend(tables.iter().flat_map(|table| table.rows.iter().cloned()));
    rows
}

fn tables_to_xlsx(
    tables: &[CleanedTable],
    report_type: &ReportType,
    output_path: &std::path::Path,
) -> Result<(), PipelineError> {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();

    let sheet_name = excel_safe_sheet_name(report_type);
    sheet
        .set_name(&sheet_name)
        .map_err(|err: XlsxError| PipelineError::ExportError(err.to_string()))?;

    for (row_index, row) in flattened_xlsx_rows(tables).iter().enumerate() {
        for (col_index, value) in row.iter().enumerate() {
            sheet
                .write_string(row_index as u32, col_index as u16, value)
                .map_err(|err: XlsxError| PipelineError::ExportError(err.to_string()))?;
        }
    }

    workbook
        .save(output_path)
        .map_err(|err: XlsxError| PipelineError::ExportError(err.to_string()))?;

    Ok(())
}

fn source_filename(input_url: &str) -> String {
    input_url
        .split(['?', '#'])
        .next()
        .unwrap_or(input_url)
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("document.pdf")
        .to_string()
}

fn safe_output_stem(input_url: &str) -> String {
    let filename = source_filename(input_url);
    let stem = filename
        .rsplit_once('.')
        .map_or(filename.as_str(), |(stem, _)| stem);
    let sanitized: String = stem
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect();

    let sanitized = sanitized.trim_matches('_');
    if sanitized.is_empty() {
        "document".to_string()
    } else {
        sanitized.to_string()
    }
}

fn resolve_report_type(args: &PipelineArgs) -> Result<ReportType, PipelineError> {
    args.report_type
        .as_ref()
        .and_then(|s| s.parse::<ReportType>().ok())
        .or_else(|| ReportType::try_from_filename(&args.input_url))
        .ok_or_else(|| PipelineError::ParseError("Could not determine report type. Use --report-type with one of: balance_sheet, income_statement, statement_cash_flow, statement_equity_changes".to_string()))
}

fn output_xlsx_path(args: &PipelineArgs, output_stem: &str) -> PathBuf {
    args.output_xlsx
        .clone()
        .unwrap_or_else(|| PathBuf::from(format!("{}_output.xlsx", output_stem)))
}

fn stage_artifact_dir(output_stem: &str) -> PathBuf {
    PathBuf::from(format!("{}_artifacts", output_stem))
}

fn write_stage_artifacts(
    artifact_dir: &std::path::Path,
    ocr_markdown: &str,
    result: &ProcessingResult,
    cleaned_tables: &[CleanedTable],
    xlsx_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(artifact_dir)?;

    let md_path = artifact_dir.join("ocr_output.md");
    std::fs::write(&md_path, ocr_markdown)?;
    tracing::info!(path = %md_path.display(), "Wrote OCR markdown artifact");

    let tables_json_path = artifact_dir.join("tables.json");
    let tables_json = serde_json::to_string_pretty(&result.tables)?;
    std::fs::write(&tables_json_path, tables_json)?;
    tracing::info!(path = %tables_json_path.display(), "Wrote tables JSON artifact");

    let cleaned_tables_json_path = artifact_dir.join("cleaned_tables.json");
    let cleaned_tables_json = serde_json::to_string_pretty(cleaned_tables)?;
    std::fs::write(&cleaned_tables_json_path, cleaned_tables_json)?;
    tracing::info!(path = %cleaned_tables_json_path.display(), "Wrote cleaned tables JSON artifact");

    let meta_path = artifact_dir.join("meta.json");
    let meta = serde_json::json!({
        "document_id": result.document_id,
        "report_type": result.report_type.to_string(),
        "page_count": result.page_count,
        "table_count": result.tables.len(),
        "cleaned_table_count": cleaned_tables.len(),
        "xlsx_output": xlsx_path.to_string_lossy().to_string(),
    });
    std::fs::write(&meta_path, serde_json::to_string_pretty(&meta)?)?;
    tracing::info!(path = %meta_path.display(), "Wrote metadata artifact");

    Ok(())
}

/// Execute the pipeline command.
pub async fn execute(args: PipelineArgs) -> Result<i32, Box<dyn std::error::Error>> {
    let report_type = resolve_report_type(&args)?;

    tracing::info!(
        input_url = %args.input_url,
        report_type = %report_type,
        "Starting pipeline"
    );

    let api_key = std::env::var("MISTRAL_API_KEY").map_err(|_| {
        PipelineError::ProviderError("MISTRAL_API_KEY environment variable is not set".to_string())
    })?;

    let output_stem = safe_output_stem(&args.input_url);
    let filename = source_filename(&args.input_url);
    let input = PdfInput::from_url(&args.input_url, output_stem.clone(), Some(filename));
    let document_id = input.document_id();
    let xlsx_path = output_xlsx_path(&args, &output_stem);

    let ocr_provider = MistralOcrProvider::with_model(api_key.clone(), "mistral-ocr-latest");

    tracing::info!(document_id = %document_id, "Step 1/4: Running OCR");
    let ocr_output = ocr_provider
        .acquire_ocr(input)
        .await
        .map_err(|e| PipelineError::ProviderError(e.to_string()))?;

    tracing::info!(
        document_id = %document_id,
        pages = ocr_output.page_count,
        markdown_len = ocr_output.markdown.len(),
        "Step 1/4: OCR complete"
    );

    let facade = ProcessingFacadeBuilder::default()
        .report_type(report_type)
        .build();

    tracing::info!(document_id = %document_id, "Step 2/4: Extracting tables from markdown");
    let result = facade
        .process_markdown(
            &ocr_output.markdown,
            ocr_output.page_count,
            document_id.clone(),
        )
        .map_err(|e| {
            tracing::error!(error = %e, "Table extraction failed");
            e
        })?;

    tracing::info!(
        document_id = %document_id,
        tables = result.tables.len(),
        "Step 2/4: Table extraction complete"
    );

    let date_normalizer = RuleBasedDateNormalizer::with_model(api_key, "mistral-large-latest");

    tracing::info!(document_id = %document_id, "Step 3/4: Normalizing date headers");
    let cleaned_tables = clean_report_tables_with_normalizer(&result, &date_normalizer).await;
    if cleaned_tables.is_empty() {
        return Err(PipelineError::NoFinancialTablesFound.into());
    }

    tracing::info!(
        document_id = %document_id,
        path = %xlsx_path.display(),
        "Step 4/4: Generating XLSX"
    );
    tables_to_xlsx(&cleaned_tables, &result.report_type, &xlsx_path)?;
    tracing::info!(path = %xlsx_path.display(), "Step 4/4: XLSX saved");

    if args.emit_stage_artifacts {
        let artifact_dir = stage_artifact_dir(&output_stem);
        tracing::info!(dir = %artifact_dir.display(), "Writing stage artifacts");
        write_stage_artifacts(
            &artifact_dir,
            &ocr_output.markdown,
            &result,
            &cleaned_tables,
            &xlsx_path,
        )?;
    }

    let total_rows: usize = cleaned_tables.iter().map(|t| t.rows.len()).sum();
    let total_cols = cleaned_tables.first().map(|t| t.headers.len()).unwrap_or(0);

    let contract = SuccessContract {
        output_json: args
            .output_json
            .clone()
            .unwrap_or_else(|| PathBuf::from("output.json")),
        output_xlsx: Some(xlsx_path),
        document_id: result.document_id,
        report_type: result.report_type.to_string(),
        row_count: total_rows,
        column_count: total_cols,
    };

    let contract_json = serde_json::to_string_pretty(&contract)?;

    if let Some(path) = args.emit_contract {
        std::fs::write(&path, &contract_json)?;
        tracing::info!(path = %path.display(), "Wrote contract");
    }

    println!("{}", contract_json);

    Ok(ExitCode::Success as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_report_type_from_flag() {
        let args = PipelineArgs {
            input_url: "https://example.com/file.pdf".to_string(),
            report_type: Some("balance_sheet".to_string()),
            output_json: None,
            output_xlsx: None,
            emit_stage_artifacts: false,
            emit_contract: None,
        };
        assert_eq!(
            resolve_report_type(&args).unwrap(),
            ReportType::BalanceSheet
        );
    }

    #[test]
    fn test_resolve_report_type_from_filename() {
        let args = PipelineArgs {
            input_url: "https://example.com/2024_balance_sheet_report.pdf".to_string(),
            report_type: None,
            output_json: None,
            output_xlsx: None,
            emit_stage_artifacts: false,
            emit_contract: None,
        };
        assert_eq!(
            resolve_report_type(&args).unwrap(),
            ReportType::BalanceSheet
        );
    }

    #[test]
    fn test_resolve_report_type_failure() {
        let args = PipelineArgs {
            input_url: "https://example.com/unknown.pdf".to_string(),
            report_type: None,
            output_json: None,
            output_xlsx: None,
            emit_stage_artifacts: false,
            emit_contract: None,
        };
        assert!(resolve_report_type(&args).is_err());
    }

    #[test]
    fn test_output_xlsx_path_explicit() {
        let args = PipelineArgs {
            input_url: "https://example.com/test.pdf".to_string(),
            report_type: None,
            output_json: None,
            output_xlsx: Some(PathBuf::from("/tmp/custom.xlsx")),
            emit_stage_artifacts: false,
            emit_contract: None,
        };
        assert_eq!(
            output_xlsx_path(&args, "test.pdf"),
            PathBuf::from("/tmp/custom.xlsx")
        );
    }

    #[test]
    fn test_output_xlsx_path_default() {
        let args = PipelineArgs {
            input_url: "https://example.com/test.pdf".to_string(),
            report_type: None,
            output_json: None,
            output_xlsx: None,
            emit_stage_artifacts: false,
            emit_contract: None,
        };
        assert_eq!(
            output_xlsx_path(&args, "mydoc"),
            PathBuf::from("mydoc_output.xlsx")
        );
    }

    #[test]
    fn test_excel_safe_sheet_name() {
        let name = excel_safe_sheet_name(&ReportType::BalanceSheet);
        assert_eq!(name, "Баланс");
        assert!(name.len() <= EXCEL_SHEET_NAME_MAX_CHARS);
    }

    #[test]
    fn test_source_filename_from_url() {
        assert_eq!(
            source_filename("https://cetco-resurs.by/file122.pdf?download=1"),
            "file122.pdf"
        );
    }

    #[test]
    fn test_safe_output_stem_from_url() {
        assert_eq!(
            safe_output_stem("https://cetco-resurs.by/file122.pdf"),
            "file122"
        );
        assert_eq!(safe_output_stem("https://example.com/a b.pdf"), "a_b");
    }

    #[test]
    fn test_stage_artifact_dir_uses_safe_stem() {
        assert_eq!(
            stage_artifact_dir("file122"),
            PathBuf::from("file122_artifacts")
        );
    }

    #[test]
    fn test_flattened_xlsx_rows_writes_header_once() {
        let tables = vec![
            CleanedTable {
                headers: vec![
                    "code".to_string(),
                    "12.2025".to_string(),
                    "12.2024".to_string(),
                ],
                rows: vec![
                    vec!["290".to_string(), "4777".to_string(), "4321".to_string()],
                    vec!["300".to_string(), "5009".to_string(), "4797".to_string()],
                ],
            },
            CleanedTable {
                headers: vec![
                    "code".to_string(),
                    "12.2025".to_string(),
                    "12.2024".to_string(),
                ],
                rows: vec![vec![
                    "410".to_string(),
                    "125".to_string(),
                    "125".to_string(),
                ]],
            },
        ];

        assert_eq!(
            flattened_xlsx_rows(&tables),
            vec![
                vec!["code", "12.2025", "12.2024"],
                vec!["290", "4777", "4321"],
                vec!["300", "5009", "4797"],
                vec!["410", "125", "125"],
            ]
        );
    }
}
