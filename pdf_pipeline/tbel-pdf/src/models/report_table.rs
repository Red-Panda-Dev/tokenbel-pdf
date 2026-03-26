//! Report table model for financial data extraction.

use serde::{Deserialize, Serialize};

/// A cell in a financial report table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableCell {
    /// Cell content.
    pub content: String,
    /// Row index (0-based).
    pub row_index: usize,
    /// Column index (0-based).
    pub col_index: usize,
}

impl TableCell {
    /// Create a new table cell.
    #[must_use]
    pub fn new(content: String, row_index: usize, col_index: usize) -> Self {
        Self {
            content,
            row_index,
            col_index,
        }
    }
}

/// A financial report table extracted from PDF.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTable {
    /// Column headers.
    pub headers: Vec<String>,
    /// Data rows (each row is a vector of cells).
    pub rows: Vec<Vec<TableCell>>,
    /// Table index in the document.
    pub table_index: usize,
}

impl ReportTable {
    /// Create a new report table with headers.
    #[must_use]
    pub fn new(headers: Vec<String>, table_index: usize) -> Self {
        Self {
            headers,
            rows: Vec::new(),
            table_index,
        }
    }

    /// Get the number of columns.
    #[must_use]
    pub fn column_count(&self) -> usize {
        self.headers.len()
    }

    /// Get the number of data rows.
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_cell_new() {
        let cell = TableCell::new("test".to_string(), 0, 1);
        assert_eq!(cell.content, "test");
        assert_eq!(cell.row_index, 0);
        assert_eq!(cell.col_index, 1);
    }

    #[test]
    fn test_report_table_new() {
        let table = ReportTable::new(vec!["A".to_string(), "B".to_string()], 0);
        assert_eq!(table.column_count(), 2);
        assert_eq!(table.row_count(), 0);
    }

    #[test]
    fn test_report_table_row_count() {
        let mut table = ReportTable::new(vec!["A".to_string(), "B".to_string()], 0);
        table.rows.push(vec![
            TableCell::new("1".to_string(), 0, 0),
            TableCell::new("2".to_string(), 0, 1),
        ]);
        table.rows.push(vec![
            TableCell::new("3".to_string(), 1, 0),
            TableCell::new("4".to_string(), 1, 1),
        ]);
        assert_eq!(table.row_count(), 2);
    }
}
