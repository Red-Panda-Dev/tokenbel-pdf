//! Cleaned report model.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A data column in a cleaned report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataColumn {
    /// Column header (normalized date or label).
    pub header: String,
    /// Column values.
    pub values: Vec<String>,
}

impl DataColumn {
    /// Create a new data column.
    #[must_use]
    pub fn new(header: String, values: Vec<String>) -> Self {
        Self { header, values }
    }
}

/// A cleaned and normalized financial report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanedReport {
    /// Report type.
    pub report_type: String,
    /// Data columns.
    pub columns: Vec<DataColumn>,
    /// Source document path.
    pub source_path: Option<PathBuf>,
}

impl CleanedReport {
    /// Create a new cleaned report.
    #[must_use]
    pub fn new(report_type: String, columns: Vec<DataColumn>) -> Self {
        Self {
            report_type,
            columns,
            source_path: None,
        }
    }

    /// Get the number of columns.
    #[must_use]
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Get the number of rows (based on first column).
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.columns.first().map_or(0, |col| col.values.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_column_new() {
        let col = DataColumn::new(
            "01.2024".to_string(),
            vec!["100".to_string(), "200".to_string()],
        );
        assert_eq!(col.header, "01.2024");
        assert_eq!(col.values.len(), 2);
    }

    #[test]
    fn test_cleaned_report_new() {
        let report = CleanedReport::new(
            "balance_sheet".to_string(),
            vec![
                DataColumn::new("code".to_string(), vec!["010".to_string()]),
                DataColumn::new("01.2024".to_string(), vec!["1000".to_string()]),
            ],
        );
        assert_eq!(report.report_type, "balance_sheet");
        assert_eq!(report.column_count(), 2);
        assert_eq!(report.row_count(), 1);
    }

    #[test]
    fn test_cleaned_report_empty() {
        let report = CleanedReport::new("balance_sheet".to_string(), vec![]);
        assert_eq!(report.column_count(), 0);
        assert_eq!(report.row_count(), 0);
    }
}
