//! Business-level cleaning for extracted report tables.

use crate::date::DateNormalizer;
use crate::date::RuleBasedDateNormalizer;
use crate::processing::ProcessingResult;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CleanedTable {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

struct PreparedTable {
    date_headers: Vec<String>,
    header: Vec<String>,
    rows: Vec<Vec<String>>,
}

fn is_blank(value: &str) -> bool {
    value.replace('\u{00a0}', " ").trim().is_empty()
}

fn cell_at(row: &[String], col_index: usize) -> String {
    row.get(col_index).cloned().unwrap_or_default()
}

fn contains_year(value: &str) -> bool {
    value.as_bytes().windows(4).any(|window| {
        window[0] == b'2'
            && window[1] == b'0'
            && window[2].is_ascii_digit()
            && window[3].is_ascii_digit()
    })
}

fn find_header_row(rows: &[Vec<String>]) -> usize {
    rows.iter()
        .position(|row| {
            !is_blank(&cell_at(row, 0))
                && contains_year(&cell_at(row, 2))
                && contains_year(&cell_at(row, 3))
        })
        .or_else(|| {
            rows.iter()
                .position(|row| contains_year(&cell_at(row, 2)) && contains_year(&cell_at(row, 3)))
        })
        .or_else(|| {
            rows.iter().position(|row| {
                !is_blank(&cell_at(row, 1))
                    && contains_year(&cell_at(row, 3))
                    && contains_year(&cell_at(row, 4))
            })
        })
        .unwrap_or(0)
}

fn remove_blank_columns(rows: &[Vec<String>]) -> Vec<Vec<String>> {
    let max_cols = rows.iter().map(Vec::len).max().unwrap_or(0);
    let keep_cols: Vec<usize> = (0..max_cols)
        .filter(|col_index| rows.iter().any(|row| !is_blank(&cell_at(row, *col_index))))
        .collect();

    rows.iter()
        .map(|row| {
            keep_cols
                .iter()
                .map(|col_index| cell_at(row, *col_index))
                .collect()
        })
        .collect()
}

fn code_digits(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if !trimmed.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    if matches!(trimmed.len(), 2 | 3) {
        Some(trimmed.to_string())
    } else {
        None
    }
}

fn is_code_column(rows: &[Vec<String>], col_index: usize) -> bool {
    rows.iter()
        .skip(1)
        .filter(|row| code_digits(&cell_at(row, col_index)).is_some())
        .take(3)
        .count()
        >= 3
}

fn align_code_column(mut rows: Vec<Vec<String>>) -> Vec<Vec<String>> {
    while rows.first().map_or(0, Vec::len) > 2 && !is_code_column(&rows, 0) {
        if is_code_column(&rows, 1) {
            rows = rows
                .into_iter()
                .map(|row| row.into_iter().skip(1).collect())
                .collect();
            break;
        }

        rows = rows
            .into_iter()
            .map(|row| row.into_iter().skip(1).collect())
            .collect();
    }

    rows
}

pub fn normalize_date_header(header: &str, fallback_index: usize) -> String {
    let lower = header.to_lowercase();
    let month = [
        ("январ", "01"),
        ("феврал", "02"),
        ("март", "03"),
        ("апрел", "04"),
        ("ма", "05"),
        ("июн", "06"),
        ("июл", "07"),
        ("август", "08"),
        ("сентябр", "09"),
        ("октябр", "10"),
        ("ноябр", "11"),
        ("декабр", "12"),
    ]
    .iter()
    .find_map(|(needle, value)| lower.contains(needle).then_some(*value));

    let year = lower.as_bytes().windows(4).find_map(|window| {
        (window[0] == b'2'
            && window[1] == b'0'
            && window[2].is_ascii_digit()
            && window[3].is_ascii_digit())
        .then(|| std::str::from_utf8(window).ok())
        .flatten()
    });

    match (month, year) {
        (Some(month), Some(year)) => format!("{month}.{year}"),
        _ => format!("date_{fallback_index}"),
    }
}

pub fn parse_belarusian_integer(value: &str) -> i64 {
    let trimmed = value.replace('\u{00a0}', " ").trim().to_string();
    if trimmed.is_empty() || trimmed == "-" {
        return 0;
    }

    let is_negative = trimmed.starts_with('(') && trimmed.ends_with(')');
    let unwrapped = trimmed
        .trim_start_matches('(')
        .trim_end_matches(')')
        .replace(' ', "");
    let integer_part = unwrapped.split(',').next().unwrap_or_default();

    let Ok(parsed) = integer_part.parse::<i64>() else {
        return 0;
    };

    if is_negative {
        -parsed
    } else {
        parsed
    }
}

fn prepare_table_for_cleaning(table: &crate::models::ReportTable) -> Option<PreparedTable> {
    let original_date_headers: Vec<String> = table
        .headers
        .iter()
        .filter(|h| contains_year(h))
        .cloned()
        .collect();

    let mut matrix = Vec::with_capacity(table.rows.len() + 1);
    matrix.push(table.headers.clone());
    matrix.extend(table.rows.iter().map(|row| {
        row.iter()
            .map(|cell| cell.content.clone())
            .collect::<Vec<String>>()
    }));

    if matrix.is_empty() {
        return None;
    }

    let header_row = find_header_row(&matrix);
    let rows = remove_blank_columns(&matrix[header_row..]);
    if rows.is_empty() || rows[0].len() < 3 {
        return None;
    }

    let rows: Vec<Vec<String>> = rows
        .into_iter()
        .map(|row| row.into_iter().skip(1).collect())
        .collect();
    let rows = align_code_column(rows);

    let header = rows.first()?.clone();
    if header.len() < 2 {
        return None;
    }

    Some(PreparedTable {
        date_headers: original_date_headers,
        header,
        rows,
    })
}

fn build_cleaned_rows(rows: &[Vec<String>], headers_len: usize) -> Vec<Vec<String>> {
    let data_row_start = rows
        .iter()
        .position(|row| code_digits(&cell_at(row, 0)).is_some())
        .unwrap_or(0)
        .max(1);

    rows.iter()
        .skip(data_row_start)
        .filter_map(|row| {
            let code = code_digits(&cell_at(row, 0))?;
            let mut cleaned = Vec::with_capacity(headers_len);
            cleaned.push(code);
            cleaned.extend(
                (1..headers_len).map(|col_index| {
                    parse_belarusian_integer(&cell_at(row, col_index)).to_string()
                }),
            );
            Some(cleaned)
        })
        .collect()
}

pub fn clean_report_tables(result: &ProcessingResult) -> Vec<CleanedTable> {
    result
        .tables
        .iter()
        .filter_map(|table| {
            let prepared = prepare_table_for_cleaning(table)?;
            let value_count = prepared.header.len() - 1;
            let num_dates = prepared.date_headers.len();

            let headers: Vec<String> = if num_dates > 0 && num_dates <= value_count {
                std::iter::once("code".to_string())
                    .chain(
                        prepared
                            .date_headers
                            .iter()
                            .enumerate()
                            .map(|(idx, h)| normalize_date_header(h, idx + 1)),
                    )
                    .collect()
            } else {
                std::iter::once("code".to_string())
                    .chain(
                        prepared
                            .header
                            .iter()
                            .skip(1)
                            .enumerate()
                            .map(|(idx, h)| normalize_date_header(h, idx + 1)),
                    )
                    .collect()
            };

            let cleaned_rows = build_cleaned_rows(&prepared.rows, headers.len());

            (!cleaned_rows.is_empty()).then_some(CleanedTable {
                headers,
                rows: cleaned_rows,
            })
        })
        .collect()
}

pub async fn clean_report_tables_with_normalizer<N>(
    result: &ProcessingResult,
    date_normalizer: &N,
) -> Vec<CleanedTable>
where
    N: DateNormalizer,
{
    let mut all_cleaned = Vec::new();

    for table in &result.tables {
        let Some(prepared) = prepare_table_for_cleaning(table) else {
            continue;
        };

        let value_count = prepared.header.len() - 1;
        let num_dates = prepared.date_headers.len();

        let raw_date_headers: Vec<String> = if num_dates > 0 && num_dates <= value_count {
            prepared.date_headers.clone()
        } else {
            prepared
                .header
                .iter()
                .skip(1)
                .filter(|h| contains_year(h))
                .cloned()
                .collect()
        };

        let mut normalized_headers = Vec::with_capacity(raw_date_headers.len());
        for (idx, raw_header) in raw_date_headers.iter().enumerate() {
            match date_normalizer.normalize_header(raw_header).await {
                Ok(normalized) if RuleBasedDateNormalizer::is_valid_mm_yyyy(&normalized) => {
                    tracing::debug!(
                        header = %raw_header,
                        normalized = %normalized,
                        "Date header normalized via normalizer"
                    );
                    normalized_headers.push(normalized);
                }
                Ok(fallback) => {
                    tracing::warn!(
                        header = %raw_header,
                        fallback = %fallback,
                        "Normalizer returned non-MM.YYYY, using date_{} fallback",
                        idx + 1
                    );
                    normalized_headers.push(format!("date_{}", idx + 1));
                }
                Err(err) => {
                    tracing::warn!(
                        header = %raw_header,
                        error = %err,
                        "Normalizer failed, using date_{} fallback",
                        idx + 1
                    );
                    normalized_headers.push(format!("date_{}", idx + 1));
                }
            }
        }

        let headers: Vec<String> = std::iter::once("code".to_string())
            .chain(normalized_headers)
            .collect();

        let cleaned_rows = build_cleaned_rows(&prepared.rows, headers.len());

        if !cleaned_rows.is_empty() {
            all_cleaned.push(CleanedTable {
                headers,
                rows: cleaned_rows,
            });
        }
    }

    all_cleaned
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ReportTable, ReportType, TableCell};

    fn sample_balance_result() -> ProcessingResult {
        let mut table = ReportTable::new(
            vec![
                "".to_string(),
                "Активы".to_string(),
                "Код строки".to_string(),
                "На 31 декабря 2025 года".to_string(),
                "На 31 декабря 2024 года".to_string(),
            ],
            0,
        );

        for (row_index, values) in [
            vec!["", "1", "2", "3", "4"],
            vec!["I. ДОЛГОСРОЧНЫЕ АКТИВЫ", "", "", "", ""],
            vec!["Основные средства", "", "110", "25", "63"],
            vec!["Нематериальные активы", "", "120", "4", "5"],
            vec!["Доходные вложения", "", "130", "-", "2"],
            vec!["Материалы", "", "211", "1 986,99", "(60)"],
            vec!["Invalid", "", "1000", "1", "2"],
            vec!["Text", "", "abc", "1", "2"],
        ]
        .into_iter()
        .enumerate()
        {
            table.rows.push(
                values
                    .into_iter()
                    .enumerate()
                    .map(|(col_index, value)| {
                        TableCell::new(value.to_string(), row_index, col_index)
                    })
                    .collect(),
            );
        }

        ProcessingResult {
            document_id: "file122".to_string(),
            report_type: ReportType::BalanceSheet,
            tables: vec![table],
            page_count: 2,
        }
    }

    #[test]
    fn test_parse_belarusian_integer() {
        assert_eq!(parse_belarusian_integer("1 986,99"), 1986);
        assert_eq!(parse_belarusian_integer("1\u{00a0}986,00"), 1986);
        assert_eq!(parse_belarusian_integer("(60)"), -60);
        assert_eq!(parse_belarusian_integer("-"), 0);
        assert_eq!(parse_belarusian_integer("not numeric"), 0);
    }

    #[test]
    fn test_normalize_date_header() {
        assert_eq!(
            normalize_date_header("На 31 декабря 2025 года", 1),
            "12.2025"
        );
        assert_eq!(
            normalize_date_header("На 31 декабря 2024 года", 2),
            "12.2024"
        );
        assert_eq!(normalize_date_header("unknown", 3), "date_3");
    }

    #[test]
    fn test_clean_report_tables_outputs_three_columns() {
        let result = sample_balance_result();
        let cleaned = clean_report_tables(&result);

        assert_eq!(cleaned.len(), 1);
        assert_eq!(cleaned[0].headers, vec!["code", "12.2025", "12.2024"]);
        assert!(cleaned[0].rows.iter().all(|row| row.len() == 3));
        assert_eq!(cleaned[0].rows[0], vec!["110", "25", "63"]);
        assert_eq!(cleaned[0].rows[3], vec!["211", "1986", "-60"]);
        assert!(!cleaned[0].rows.iter().any(|row| row[0] == "1000"));
        assert!(!cleaned[0].rows.iter().any(|row| row[0] == "abc"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_async_clean_with_stub_normalizer() {
        let stub = crate::date::StubDateNormalizer::new()
            .with_mapping("На 31 декабря 2025 года", "12.2025")
            .with_mapping("На 31 декабря 2024 года", "12.2024");

        let result = sample_balance_result();
        let cleaned = clean_report_tables_with_normalizer(&result, &stub).await;

        assert_eq!(cleaned.len(), 1);
        assert_eq!(cleaned[0].headers, vec!["code", "12.2025", "12.2024"]);
        assert!(cleaned[0].rows.iter().all(|row| row.len() == 3));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_async_clean_year_only_headers() {
        let stub = crate::date::StubDateNormalizer::new()
            .with_mapping("За 2025 г.", "12.2025")
            .with_mapping("За 2024 г.", "12.2024");

        let mut table = ReportTable::new(
            vec![
                "".to_string(),
                "Показатель".to_string(),
                "Код строки".to_string(),
                "За 2025 г.".to_string(),
                "За 2024 г.".to_string(),
            ],
            0,
        );

        for (row_index, values) in [
            vec!["", "1", "2", "3", "4"],
            vec!["Section", "", "", "", ""],
            vec!["Основные средства", "", "110", "25", "63"],
            vec!["Нематериальные активы", "", "120", "4", "5"],
            vec!["Доходные вложения", "", "130", "-", "2"],
        ]
        .into_iter()
        .enumerate()
        {
            table.rows.push(
                values
                    .into_iter()
                    .enumerate()
                    .map(|(col_index, value)| {
                        TableCell::new(value.to_string(), row_index, col_index)
                    })
                    .collect(),
            );
        }

        let result = ProcessingResult {
            document_id: "test_year_only".to_string(),
            report_type: ReportType::StatementCashFlow,
            tables: vec![table],
            page_count: 1,
        };

        let cleaned = clean_report_tables_with_normalizer(&result, &stub).await;

        assert_eq!(cleaned.len(), 1);
        assert_eq!(cleaned[0].headers, vec!["code", "12.2025", "12.2024"]);
        assert_eq!(cleaned[0].rows.len(), 3);
        assert_eq!(cleaned[0].rows[0], vec!["110", "25", "63"]);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_async_clean_fallback_on_invalid_normalizer_output() {
        let stub = crate::date::StubDateNormalizer::new()
            .with_mapping("На 31 декабря 2025 года", "invalid")
            .with_mapping("На 31 декабря 2024 года", "12.2024");

        let result = sample_balance_result();
        let cleaned = clean_report_tables_with_normalizer(&result, &stub).await;

        assert_eq!(cleaned.len(), 1);
        assert_eq!(cleaned[0].headers, vec!["code", "date_1", "12.2024"]);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_async_clean_fallback_on_unmapped_header() {
        let stub = crate::date::StubDateNormalizer::new();

        let result = sample_balance_result();
        let cleaned = clean_report_tables_with_normalizer(&result, &stub).await;

        assert_eq!(cleaned.len(), 1);
        assert_eq!(cleaned[0].headers[0], "code");
        assert!(cleaned[0].headers[1].starts_with("date_"));
        assert!(cleaned[0].headers[2].starts_with("date_"));
    }
}
