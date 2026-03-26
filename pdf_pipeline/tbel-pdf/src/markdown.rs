//! Markdown preprocessing utilities for OCR output.
//!
//! Provides functions to clean LaTeX expressions and preprocess markdown
//! text from OCR output for financial table extraction.

use regex::Regex;

/// Cleans LaTeX math expressions from markdown text.
///
/// Removes LaTeX markup from table cells, extracting only readable text content.
/// Example: "$\\begin{gathered} \\text { На } 30 \\text { сентября } \\end{centered}$"
/// becomes "На 30 сентября"
///
/// # Arguments
///
/// * `text` - Markdown text potentially containing LaTeX expressions
///
/// # Returns
///
/// Cleaned markdown text with LaTeX markup removed
pub fn clean_latex_from_markdown(text: &str) -> String {
    // Pattern to match LaTeX math expressions between $ delimiters
    let latex_pattern = Regex::new(r"\$([^$]+)\$").unwrap();

    latex_pattern
        .replace_all(text, |caps: &regex::Captures| {
            let latex_content = &caps[1];
            clean_single_latex_expression(latex_content)
        })
        .to_string()
}

/// Cleans a single LaTeX expression, extracting readable text.
fn clean_single_latex_expression(latex_content: &str) -> String {
    let mut cleaned = latex_content.to_string();

    // Remove \begin{...} and \end{...} commands
    let begin_pattern = Regex::new(r"\\begin\{[^}]+\}").unwrap();
    let end_pattern = Regex::new(r"\\end\{[^}]+\}").unwrap();
    cleaned = begin_pattern.replace_all(&cleaned, "").to_string();
    cleaned = end_pattern.replace_all(&cleaned, "").to_string();

    // Extract text from \text{...} commands
    let text_pattern = Regex::new(r"\\text\s*\{\s*([^}]+)\s*\}").unwrap();
    cleaned = text_pattern.replace_all(&cleaned, "$1").to_string();

    // Remove remaining LaTeX commands (backslash followed by letters)
    let cmd_pattern = Regex::new(r"\\[a-zA-Z]+").unwrap();
    cleaned = cmd_pattern.replace_all(&cleaned, "").to_string();

    // Handle escaped characters like \\
    let escaped_pattern = Regex::new(r"\\(.)").unwrap();
    cleaned = escaped_pattern.replace_all(&cleaned, "$1").to_string();

    // Remove extra braces
    let brace_pattern = Regex::new(r"[{}]").unwrap();
    cleaned = brace_pattern.replace_all(&cleaned, "").to_string();

    // Clean up extra whitespace
    let ws_pattern = Regex::new(r"\s+").unwrap();
    cleaned = ws_pattern.replace_all(&cleaned, " ").to_string();

    cleaned.trim().to_string()
}

/// Checks if a line is a financial table header row.
///
/// Supports multiple header patterns:
/// - Income/Cash Flow: "Наименование показателей" + "Код строки"
/// - Balance Sheet Assets: "Активы" + "Код строки"
/// - Balance Sheet Liabilities: "Собственный капитал и обязательства" + "Код строки"
fn is_financial_table_header(line: &str) -> bool {
    let line_stripped = line.trim();
    if !line_stripped.starts_with('|') {
        return false;
    }

    if !line_stripped.contains("Код строки") {
        return false;
    }

    line_stripped.contains("Наименование показателей")
        || line_stripped.contains("Активы")
        || line_stripped.contains("Собственный капитал и обязательства")
}

/// Counts the number of columns in a markdown table row.
fn count_columns(line: &str) -> usize {
    line.trim().matches('|').count() - 1
}

/// Preprocesses markdown text in a single pass.
///
/// Combines LaTeX cleaning, company info separation, continuation line joining,
/// and merged table splitting into one efficient pass.
///
/// # Arguments
///
/// * `md` - Raw markdown text from OCR
///
/// # Returns
///
/// Preprocessed markdown text ready for table extraction
pub fn preprocess_markdown(md: &str) -> String {
    // Step 1: Clean LaTeX expressions
    let md = clean_latex_from_markdown(md);

    // Step 2-4: Process in single pass
    let lines: Vec<&str> = md.split('\n').collect();
    let mut fixed_lines: Vec<String> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let line_stripped = line.trim();

        // Handle table rows
        if line_stripped.starts_with('|') {
            // Check if this is a financial table header that needs separation
            if is_financial_table_header(line) {
                // Find previous non-empty line
                let mut prev_idx = i.saturating_sub(1);
                while prev_idx > 0 && lines[prev_idx].trim().is_empty() {
                    prev_idx = prev_idx.saturating_sub(1);
                }

                if prev_idx > 0 {
                    let prev_line = lines[prev_idx].trim();
                    // Check if previous line is a 2-column table row
                    if prev_line.starts_with('|') {
                        let prev_columns = count_columns(prev_line);
                        if prev_columns == 2 {
                            // Insert blank line to separate tables
                            fixed_lines.push(String::new());
                        }
                    }
                }
            }

            // Collect this line and all continuation lines
            let mut collected = line.trim_end().to_string();
            let mut j = i + 1;

            while j < lines.len() {
                let next_line = lines[j];
                let next_stripped = next_line.trim();

                if next_stripped.is_empty() {
                    break;
                }
                if next_stripped.starts_with('|') {
                    break;
                }
                if next_stripped.starts_with('#') {
                    break;
                }

                // Continuation line - add it
                collected.push(' ');
                collected.push_str(next_stripped);
                j += 1;
            }

            // Check for merged tables (double pipe pattern)
            if collected.contains("||") {
                let parts: Vec<&str> = collected.split("||").collect();
                if parts.len() == 2 {
                    let first_row = format!("{} |", parts[0].trim_end());
                    let second_row = format!("|{}", parts[1].trim_start());
                    fixed_lines.push(first_row);
                    fixed_lines.push(String::new()); // Blank line to separate
                    fixed_lines.push(second_row);
                } else {
                    fixed_lines.push(collected);
                }
            } else {
                fixed_lines.push(collected);
            }

            i = j;
            continue;
        }

        // Non-table line - add as-is
        fixed_lines.push(line.to_string());
        i += 1;
    }

    fixed_lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_latex_simple() {
        let input = "Some text $hello$ more text";
        let result = clean_latex_from_markdown(input);
        assert_eq!(result, "Some text hello more text");
    }

    #[test]
    fn test_clean_latex_text_command() {
        let input = "$\\text{На} 30 \\text{сентября}$";
        let result = clean_latex_from_markdown(input);
        assert_eq!(result, "На 30 сентября");
    }

    #[test]
    fn test_clean_latex_begin_end() {
        let input = "$\\begin{gathered} \\text { На } 30 \\end{gathered}$";
        let result = clean_latex_from_markdown(input);
        assert_eq!(result, "На 30");
    }

    #[test]
    fn test_clean_latex_multiple() {
        let input = "$first$ and $second$ and $third$";
        let result = clean_latex_from_markdown(input);
        assert_eq!(result, "first and second and third");
    }

    #[test]
    fn test_clean_latex_no_latex() {
        let input = "Normal text without latex";
        let result = clean_latex_from_markdown(input);
        assert_eq!(result, "Normal text without latex");
    }

    #[test]
    fn test_clean_latex_complex() {
        let input = "$\\begin{gathered} \\text { На } 30 \\text { сентября } \\end{gathered}$";
        let result = clean_latex_from_markdown(input);
        assert_eq!(result, "На 30 сентября");
    }

    #[test]
    fn test_preprocess_markdown_join_continuation() {
        let input = "| 010 | Value\ncontinued here | 100 | 200 |";
        let result = preprocess_markdown(input);
        assert!(result.contains("Value continued here"));
    }

    #[test]
    fn test_preprocess_markdown_split_merged_table() {
        let input = "| 010 | Name || 100 | 200 |";
        let result = preprocess_markdown(input);
        assert!(result.contains("| 010 | Name |"));
        assert!(result.contains("|100 |"));
    }

    #[test]
    fn test_preprocess_markdown_2col_to_4col_separation() {
        let input = "| col1 | col2 |\n\n| Код строки | Наименование | 2024 | 2023 |";
        let result = preprocess_markdown(input);
        // Should have blank line between 2-col and 4-col tables
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.iter().any(|l| l.is_empty()));
    }

    #[test]
    fn test_is_financial_table_header() {
        assert!(is_financial_table_header(
            "| Наименование показателей | Код строки | 2024 | 2023 |"
        ));
        assert!(is_financial_table_header(
            "| Активы | Код строки | 2024 | 2023 |"
        ));
        assert!(is_financial_table_header(
            "| Собственный капитал и обязательства | Код строки | 2024 | 2023 |"
        ));
        assert!(!is_financial_table_header("| Just | some | headers |"));
    }

    #[test]
    fn test_count_columns() {
        assert_eq!(count_columns("| a | b | c |"), 3);
        assert_eq!(count_columns("| 010 | Name |"), 2);
        assert_eq!(count_columns("| a |"), 1);
    }
}
