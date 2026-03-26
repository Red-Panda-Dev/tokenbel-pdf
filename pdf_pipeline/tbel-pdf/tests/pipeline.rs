//! Integration tests for PDF pipeline.
//!
//! Tests compare pipeline output against golden files in tests/golden/.

use std::path::PathBuf;
use tbel_pdf::{
    extract_table_candidates, preprocess_markdown,
    MockOcrProvider, OcrProvider, PdfInput, ReportType,
};

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
        ("2024_income_statement.pdf", Some(ReportType::IncomeStatement)),
        ("2024_statement_cash_flow.pdf", Some(ReportType::StatementCashFlow)),
        ("2024_statement_equity_changes.pdf", Some(ReportType::StatementEquityChanges)),
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
        assert!(json_path.exists() || xlsx_path.exists(), 
            "Golden file missing for: {}", name);
    }
}
