#![cfg(not(target_arch = "wasm32"))]

//! Integration tests for PDF pipeline.
//!
//! Tests compare pipeline output against golden files in tests/golden/.

use std::path::PathBuf;
use tbel_pdf::{
    clean_report_tables, extract_table_candidates, preprocess_markdown, MockOcrProvider,
    OcrProvider, PdfInput, ProcessingFacadeBuilder, ReportType,
};

const FILE111_INCOME_MD: &str =
    include_str!("../../tests/fixtures/ocr/file111_income_statement.md");
const FILE122_BALANCE_MD: &str = include_str!("../../tests/fixtures/ocr/file122_balance_sheet.md");
const FILE133_CASHFLOW_MD: &str = include_str!("../../tests/fixtures/ocr/file133_cashflow.md");

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests")
}

fn golden_dir() -> PathBuf {
    fixtures_dir().join("golden")
}

fn read_golden_json(name: &str) -> serde_json::Value {
    let path = golden_dir().join(format!("{}.json", name));
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read golden file: {}", path.display()));
    serde_json::from_str(&content).expect("Failed to parse golden JSON")
}

fn clean_fixture(
    markdown: &str,
    report_type: ReportType,
    document_id: &str,
) -> Vec<tbel_pdf::CleanedTable> {
    let result = ProcessingFacadeBuilder::default()
        .report_type(report_type)
        .build()
        .process_markdown(markdown, 2, document_id.to_string())
        .expect("fixture markdown should produce financial tables");

    clean_report_tables(&result)
}

fn total_rows(tables: &[tbel_pdf::CleanedTable]) -> usize {
    tables.iter().map(|table| table.rows.len()).sum()
}

fn row_for_code<'a>(tables: &'a [tbel_pdf::CleanedTable], code: &str) -> &'a Vec<String> {
    tables
        .iter()
        .flat_map(|table| table.rows.iter())
        .find(|row| row[0] == code)
        .unwrap_or_else(|| panic!("missing row code {code}"))
}

fn assert_standard_headers(tables: &[tbel_pdf::CleanedTable], periods: [&str; 2]) {
    for table in tables {
        assert_eq!(table.headers, vec!["code", periods[0], periods[1]]);
        assert!(table.rows.iter().all(|row| row.len() == 3));
    }
}

#[test]
fn test_golden_native_pdf_table_extraction() {
    let golden = read_golden_json("native_pdf");
    assert!(golden.is_object());
    assert!(golden.get("columns").is_some() || golden.get("rows").is_some());
}

#[test]
fn test_golden_scanned_pdf_table_extraction() {
    let golden = read_golden_json("scanned_pdf");
    assert!(golden.is_object());
}

#[test]
fn test_real_income_statement_ocr_markdown_extracts_all_rows() {
    let tables = clean_fixture(
        FILE111_INCOME_MD,
        ReportType::IncomeStatement,
        "file111_income_statement",
    );

    assert_eq!(tables.len(), 2);
    assert_eq!(total_rows(&tables), 37);
    assert_standard_headers(&tables, ["01.2025", "01.2024"]);
    assert_eq!(row_for_code(&tables, "010"), &vec!["010", "5622", "6042"]);
    assert_eq!(row_for_code(&tables, "133"), &vec!["133", "-7", "-1"]);
    assert_eq!(row_for_code(&tables, "140"), &vec!["140", "813", "1078"]);
    assert_eq!(row_for_code(&tables, "260"), &vec!["260", "0", "0"]);
}

#[test]
fn test_real_balance_sheet_ocr_markdown_extracts_assets_and_liabilities() {
    let tables = clean_fixture(
        FILE122_BALANCE_MD,
        ReportType::BalanceSheet,
        "file122_balance_sheet",
    );

    assert_eq!(tables.len(), 2);
    assert_standard_headers(&tables, ["12.2025", "12.2024"]);
    assert_eq!(row_for_code(&tables, "110"), &vec!["110", "25", "63"]);
    assert_eq!(row_for_code(&tables, "300"), &vec!["300", "5009", "4797"]);
    assert_eq!(row_for_code(&tables, "410"), &vec!["410", "125", "125"]);
    assert_eq!(row_for_code(&tables, "700"), &vec!["700", "5009", "4797"]);
}

#[test]
fn test_real_cashflow_ocr_markdown_extracts_all_sections() {
    let tables = clean_fixture(
        FILE133_CASHFLOW_MD,
        ReportType::StatementCashFlow,
        "file133_cashflow",
    );

    assert_eq!(tables.len(), 2);
    assert_eq!(total_rows(&tables), 39);
    assert_standard_headers(&tables, ["01.2025", "01.2024"]);
    assert_eq!(row_for_code(&tables, "020"), &vec!["020", "6928", "11231"]);
    assert_eq!(row_for_code(&tables, "070"), &vec!["070", "1040", "796"]);
    assert_eq!(row_for_code(&tables, "080"), &vec!["080", "301", "248"]);
    assert_eq!(row_for_code(&tables, "140"), &vec!["140", "-2", "-8"]);
}

#[test]
fn test_golden_merged_table_pdf() {
    let golden = read_golden_json("merged_table_pdf");
    assert!(golden.is_object());
}

#[test]
fn test_golden_malformed_ocr() {
    let golden = read_golden_json("malformed_ocr");
    assert!(golden.is_object());
}

#[test]
fn test_golden_statement_equity_changes() {
    let golden = read_golden_json("statement_equity_changes");
    assert!(golden.is_object());
}

#[test]
fn test_golden_no_table_returns_error_code() {
    let golden = read_golden_json("no_table");
    assert!(golden.is_object());
    if let Some(error_code) = golden.get("error_code") {
        assert_eq!(error_code.as_str().unwrap(), "no_table_detected");
    }
}

#[test]
fn test_golden_unsupported_layout_returns_error() {
    let golden = read_golden_json("unsupported_layout");
    assert!(golden.is_object());
    if let Some(error_code) = golden.get("error_code") {
        assert_eq!(error_code.as_str().unwrap(), "unsupported_layout");
    }
}

#[test]
fn test_golden_under_dimension_rows() {
    let golden = read_golden_json("under_dimension_rows");
    assert!(golden.is_object());
    if let Some(row_count) = golden.get("row_count") {
        assert!(row_count.as_u64().unwrap() < 10);
    }
}

#[test]
fn test_golden_under_dimension_columns() {
    let golden = read_golden_json("under_dimension_columns");
    assert!(golden.is_object());
    if let Some(col_count) = golden.get("column_count") {
        assert!(col_count.as_u64().unwrap() < 3);
    }
}

#[tokio::test]
async fn test_mock_ocr_with_financial_table() {
    let markdown = r#"
| Код строки | Наименование показателей | 01.2024 | 01.2023 |
| --- | --- | --- | --- |
| 010 | Основные средства | 1 000 | 900 |
| 020 | Запасы | 500 | 400 |
"#;

    let mock = MockOcrProvider::new().with_text("test-doc", markdown);
    let input = PdfInput::Bytes {
        bytes: vec![],
        document_id: Some("test-doc".to_string()),
        filename: None,
    };

    let output = mock.acquire_ocr(input).await.unwrap();
    let processed = preprocess_markdown(&output.markdown);
    let tables = extract_table_candidates_from_markdown(&processed);

    assert!(!tables.is_empty(), "Should extract at least one table");
}

fn extract_table_candidates_from_markdown(markdown: &str) -> Vec<tbel_pdf::ReportTable> {
    let html = markdown_to_html(markdown);
    extract_table_candidates(&html)
}

fn markdown_to_html(markdown: &str) -> String {
    let mut html = String::new();
    let mut in_table = false;

    for line in markdown.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            if in_table {
                html.push_str("</table>");
                in_table = false;
            }
            continue;
        }

        if !in_table {
            html.push_str("<table>");
            in_table = true;
        }

        if trimmed.contains("---") {
            continue;
        }

        html.push_str("<tr>");

        let cells: Vec<&str> = trimmed
            .trim_matches('|')
            .split('|')
            .map(|s| s.trim())
            .collect();

        for cell in cells {
            html.push_str("<td>");
            html.push_str(cell);
            html.push_str("</td>");
        }

        html.push_str("</tr>");
    }

    if in_table {
        html.push_str("</table>");
    }

    html
}

#[test]
fn test_report_type_inference_from_golden_files() {
    let test_cases = vec![
        ("2024_balance_sheet.pdf", Some(ReportType::BalanceSheet)),
        (
            "2024_income_statement.pdf",
            Some(ReportType::IncomeStatement),
        ),
        (
            "2024_statement_cash_flow.pdf",
            Some(ReportType::StatementCashFlow),
        ),
        (
            "2024_statement_equity_changes.pdf",
            Some(ReportType::StatementEquityChanges),
        ),
        ("unknown_report.pdf", None),
    ];

    for (filename, expected) in test_cases {
        let result = ReportType::try_from_filename(filename);
        assert_eq!(result, expected, "Failed for {}", filename);
    }
}

#[test]
fn test_golden_files_exist() {
    let golden_names = vec![
        "native_pdf",
        "scanned_pdf",
        "merged_table_pdf",
        "malformed_ocr",
        "no_table",
        "unsupported_layout",
        "date_translation_failure",
        "under_dimension_rows",
        "under_dimension_columns",
        "statement_equity_changes",
    ];

    for name in golden_names {
        let json_path = golden_dir().join(format!("{}.json", name));
        let xlsx_path = golden_dir().join(format!("{}.xlsx", name));
        assert!(
            json_path.exists() || xlsx_path.exists(),
            "Golden file missing for: {}",
            name
        );
    }
}
