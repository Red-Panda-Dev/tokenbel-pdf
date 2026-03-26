//! Table extraction from HTML/Markdown fragments.
//!
//! Provides functions to parse OCR-generated HTML fragments and extract
//! candidate financial tables.

use scraper::{Html, Selector};

use crate::models::{ReportTable, TableCell};

/// Financial table header patterns.
const FINANCIAL_HEADER_PATTERNS: &[&str] = &[
    "Код строки",
    "Наименование показателей",
    "Активы",
    "Собственный капитал и обязательства",
];

/// Extracts table candidates from HTML content.
///
/// Parses HTML with scraper, finds tables with financial headers,
/// and extracts cells into ReportTable structs.
///
/// # Arguments
///
/// * `html` - HTML content from OCR output
///
/// # Returns
///
/// Vector of ReportTable structs representing candidate tables
pub fn extract_table_candidates(html: &str) -> Vec<ReportTable> {
    let document = Html::parse_document(html);
    let table_selector = Selector::parse("table").unwrap();
    let row_selector = Selector::parse("tr").unwrap();
    let cell_selector = Selector::parse("td, th").unwrap();

    let mut candidates = Vec::new();

    for (table_idx, table_elem) in document.select(&table_selector).enumerate() {
        let rows: Vec<_> = table_elem.select(&row_selector).collect();

        if rows.is_empty() {
            continue;
        }

        // Extract headers from first row
        let headers: Vec<String> = rows[0]
            .select(&cell_selector)
            .map(|cell| cell.text().collect::<String>().trim().to_string())
            .collect();

        // Extract data rows
        let mut table = ReportTable::new(headers, table_idx);

        for (row_idx, row) in rows.iter().skip(1).enumerate() {
            let cells: Vec<TableCell> = row
                .select(&cell_selector)
                .enumerate()
                .map(|(col_idx, cell)| {
                    let content = cell.text().collect::<String>().trim().to_string();
                    TableCell::new(content, row_idx, col_idx)
                })
                .collect();

            if !cells.is_empty() {
                table.rows.push(cells);
            }
        }

        candidates.push(table);
    }

    candidates
}

/// Extracts table candidates from Markdown content.
///
/// Converts markdown tables to HTML and then uses HTML extraction.
///
/// # Arguments
///
/// * `markdown` - Markdown table content
///
/// # Returns
///
/// Vector of ReportTable structs representing candidate tables
pub fn extract_table_candidates_from_markdown(markdown: &str) -> Vec<ReportTable> {
    let html = markdown_to_html(markdown);
    extract_table_candidates(&html)
}

/// Converts a simple markdown table to HTML for parsing.
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

        // Skip separator lines like |---|---|
        if trimmed.contains("---") {
            continue;
        }

        html.push_str("<tr>");

        // Parse cells from |cell1|cell2|cell3|
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

/// Checks if a table is a valid financial report candidate.
///
pub fn is_valid_financial_table(table: &ReportTable) -> bool {
    // Check column count
    if table.column_count() < 3 {
        return false;
    }

    if table.row_count() < 3 {
        return false;
    }

    if has_financial_header(&table.headers) {
        return true;
    }

    count_code_like_rows(table) >= 3
}

fn has_financial_header(headers: &[String]) -> bool {
    let header_text = headers.join(" ");
    FINANCIAL_HEADER_PATTERNS
        .iter()
        .any(|pattern| header_text.contains(pattern))
}

fn count_code_like_rows(table: &ReportTable) -> usize {
    table
        .rows
        .iter()
        .filter(|row| {
            row.iter()
                .take(2)
                .any(|cell| is_code_like(cell.content.as_str()))
        })
        .count()
}

fn is_code_like(value: &str) -> bool {
    let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
    matches!(digits.len(), 2 | 3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_table_candidates_simple() {
        let html = r#"<table>
            <tr><th>Код строки</th><th>Наименование</th><th>2024</th></tr>
            <tr><td>010</td><td>Test</td><td>100</td></tr>
            <tr><td>020</td><td>Test2</td><td>200</td></tr>
        </table>"#;

        let candidates = extract_table_candidates(html);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].headers.len(), 3);
        assert_eq!(candidates[0].row_count(), 2);
    }

    #[test]
    fn test_extract_table_candidates_no_financial_header() {
        let html = r#"<table>
            <tr><th>Col1</th><th>Col2</th></tr>
            <tr><td>A</td><td>B</td></tr>
        </table>"#;

        let candidates = extract_table_candidates(html);
        assert_eq!(candidates.len(), 1);
        assert!(!is_valid_financial_table(&candidates[0]));
    }

    #[test]
    fn test_extract_table_candidates_multiple() {
        let html = r#"<table>
            <tr><th>Код строки</th><th>Name</th><th>2024</th></tr>
            <tr><td>010</td><td>Test</td><td>100</td></tr>
        </table>
        <table>
            <tr><th>Активы</th><th>Code</th><th>2023</th></tr>
            <tr><td>Asset1</td><td>100</td><td>500</td></tr>
        </table>"#;

        let candidates = extract_table_candidates(html);
        assert_eq!(candidates.len(), 2);
    }

    #[test]
    fn test_markdown_to_html() {
        let md = "| Header1 | Header2 |\n| --- | --- |\n| A | B |";
        let html = markdown_to_html(md);
        assert!(html.contains("<table>"));
        assert!(html.contains("<tr>"));
        assert!(html.contains("<td>"));
    }

    #[test]
    fn test_extract_table_candidates_from_markdown() {
        let md = r#"| Код строки | Наименование | 2024 |
| --- | --- | --- |
| 010 | Test | 100 |"#;

        let candidates = extract_table_candidates_from_markdown(md);
        assert_eq!(candidates.len(), 1);
    }

    #[test]
    fn test_extract_table_candidates_from_markdown_multiple_tables() {
        let md = r#"| Organization | Value |
| --- | --- |
| Test Co | X |

| Активы | Код строки | 2025 | 2024 |
| --- | --- | --- | --- |
| 1 | 2 | 3 | 4 |
| Основные средства | 110 | 10 | 9 |
| Нематериальные активы | 120 | 11 | 8 |
| Вложения в долгосрочные активы | 140 | 12 | 7 |
| Долгосрочная дебиторская задолженность | 170 | 13 | 6 |
| ИТОГО по разделу I | 190 | 14 | 5 |
| Запасы | 210 | 15 | 4 |
| Налог на добавленную стоимость | 220 | 16 | 3 |
| Денежные средства | 270 | 17 | 2 |
        | ИТОГО по разделу II | 290 | 18 | 1 |
        | БАЛАНС | 300 | 19 | 0 |"#;

        let candidates = extract_table_candidates_from_markdown(md);
        assert_eq!(candidates.len(), 2);
        let valid_count = candidates
            .iter()
            .filter(|t| is_valid_financial_table(t))
            .count();
        assert_eq!(valid_count, 1);
    }

    #[test]
    fn test_extract_table_candidates_from_markdown_keeps_empty_cells_for_continuation_table() {
        let md = r#"| Движение денежных средств по финансовой деятельности |   |   |   |
| --- | --- | --- | --- |
| Поступило денежных средств - всего | 080 | 26 192 | 17 944 |
| кредиты и займы | 081 | 26 192 | 17 944 |
| прочие поступления | 084 | - | - |"#;

        let candidates = extract_table_candidates_from_markdown(md);
        assert_eq!(candidates.len(), 1);

        let table = &candidates[0];
        assert_eq!(table.column_count(), 4);
        assert!(is_valid_financial_table(table));
    }

    #[test]
    fn test_is_valid_financial_table() {
        let mut table = ReportTable::new(
            vec![
                "Код строки".to_string(),
                "Наименование".to_string(),
                "2024".to_string(),
            ],
            0,
        );

        // Add 10 rows
        for i in 0..10 {
            table.rows.push(vec![
                TableCell::new(format!("{:03}", 10 + i), i, 0),
                TableCell::new("Value".to_string(), i, 1),
                TableCell::new("100".to_string(), i, 2),
            ]);
        }

        assert!(is_valid_financial_table(&table));
    }

    #[test]
    fn test_is_valid_financial_table_too_few_rows() {
        let table = ReportTable::new(
            vec![
                "Код строки".to_string(),
                "Наименование".to_string(),
                "2024".to_string(),
            ],
            0,
        );

        assert!(!is_valid_financial_table(&table));
    }

    #[test]
    fn test_is_valid_financial_table_continuation_rows_without_financial_header() {
        let mut table = ReportTable::new(
            vec![
                "col1".to_string(),
                "col2".to_string(),
                "col3".to_string(),
                "col4".to_string(),
            ],
            1,
        );

        let codes = [
            "180", "190", "200", "210", "220", "230", "240", "250", "260",
        ];
        for (row_idx, code) in codes.iter().enumerate() {
            table.rows.push(vec![
                TableCell::new(format!("Line {code}"), row_idx, 0),
                TableCell::new((*code).to_string(), row_idx, 1),
                TableCell::new("10".to_string(), row_idx, 2),
                TableCell::new("9".to_string(), row_idx, 3),
            ]);
        }

        assert!(is_valid_financial_table(&table));
    }

    #[test]
    fn test_is_valid_financial_table_too_few_columns() {
        let table = ReportTable::new(
            vec!["Код строки".to_string(), "Наименование".to_string()],
            0,
        );

        assert!(!is_valid_financial_table(&table));
    }
}
