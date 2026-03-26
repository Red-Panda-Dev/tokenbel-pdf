//! Code-value pair for financial data.

use serde::{Deserialize, Serialize};

/// A code-value pair from a financial report row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeValue {
    /// Financial code (e.g., "010", "110").
    pub code: String,
    /// Optional name/description.
    pub name: Option<String>,
    /// Row index in the table.
    pub row_index: usize,
}

impl CodeValue {
    /// Create a new code-value pair.
    #[must_use]
    pub fn new(code: String, name: Option<String>, row_index: usize) -> Self {
        Self {
            code,
            name,
            row_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_value_new() {
        let cv = CodeValue::new("010".to_string(), Some("Assets".to_string()), 0);
        assert_eq!(cv.code, "010");
        assert_eq!(cv.name, Some("Assets".to_string()));
        assert_eq!(cv.row_index, 0);
    }

    #[test]
    fn test_code_value_without_name() {
        let cv = CodeValue::new("110".to_string(), None, 5);
        assert_eq!(cv.code, "110");
        assert_eq!(cv.name, None);
    }
}
