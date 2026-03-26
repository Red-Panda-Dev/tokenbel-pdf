//! Data cleaning structures.

use serde::{Deserialize, Serialize};

/// A PDF document with pages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfDocument {
    /// Document path.
    pub path: String,
    /// Document pages.
    pub pages: Vec<Page>,
}

/// A page in a PDF document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    /// Page number (1-based).
    pub number: u32,
    /// Page content.
    pub content: String,
}

/// Extracted financial data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedData {
    /// Company name (if detected).
    pub company_name: Option<String>,
    /// Financial records.
    pub financial_data: Vec<FinancialRecord>,
}

/// A financial record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialRecord {
    /// Period identifier.
    pub period: String,
    /// Revenue value.
    pub revenue: Option<f64>,
    /// Profit value.
    pub profit: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_document_serialization() {
        let doc = PdfDocument {
            path: "/test.pdf".to_string(),
            pages: vec![Page {
                number: 1,
                content: "Test content".to_string(),
            }],
        };
        let json = serde_json::to_string(&doc).unwrap();
        assert!(json.contains("/test.pdf"));
    }

    #[test]
    fn test_page_serialization() {
        let page = Page {
            number: 1,
            content: "Content".to_string(),
        };
        let json = serde_json::to_string(&page).unwrap();
        assert!(json.contains("Content"));
    }

    #[test]
    fn test_financial_record_optional_fields() {
        let record = FinancialRecord {
            period: "Q1 2024".to_string(),
            revenue: Some(1000.0),
            profit: None,
        };
        assert_eq!(record.period, "Q1 2024");
        assert_eq!(record.revenue, Some(1000.0));
        assert_eq!(record.profit, None);
    }

    #[test]
    fn test_extracted_data() {
        let data = ExtractedData {
            company_name: Some("Test LLC".to_string()),
            financial_data: vec![],
        };
        assert_eq!(data.company_name, Some("Test LLC".to_string()));
    }
}
