//! PDF input types for the pipeline.

use std::path::PathBuf;

/// Input source for PDF processing.
#[derive(Debug, Clone)]
pub enum PdfInput {
    /// Load from file path.
    Path {
        /// File system path.
        path: PathBuf,
        /// Optional document identifier.
        document_id: Option<String>,
    },
    /// Load from raw bytes.
    Bytes {
        /// Raw PDF bytes.
        bytes: Vec<u8>,
        /// Optional document identifier.
        document_id: Option<String>,
        /// Optional filename.
        filename: Option<String>,
    },
    /// Load from URL.
    Url {
        /// Document URL.
        document_url: String,
        /// Optional document identifier.
        document_id: Option<String>,
        /// Optional filename.
        filename: Option<String>,
    },
}

impl PdfInput {
    /// Get the document identifier.
    #[must_use]
    pub fn document_id(&self) -> String {
        match self {
            Self::Path { path, document_id } => document_id.clone().unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            }),
            Self::Bytes { document_id, .. } => {
                document_id.clone().unwrap_or_else(|| "unknown".to_string())
            }
            Self::Url {
                document_id,
                document_url,
                ..
            } => document_id.clone().unwrap_or_else(|| {
                document_url
                    .rsplit('/')
                    .next()
                    .unwrap_or("unknown")
                    .to_string()
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_input_path_document_id() {
        let input = PdfInput::Path {
            path: PathBuf::from("/test/document.pdf"),
            document_id: Some("doc-123".to_string()),
        };
        assert_eq!(input.document_id(), "doc-123");
    }

    #[test]
    fn test_pdf_input_path_document_id_from_filename() {
        let input = PdfInput::Path {
            path: PathBuf::from("/test/myfile.pdf"),
            document_id: None,
        };
        assert_eq!(input.document_id(), "myfile");
    }

    #[test]
    fn test_pdf_input_bytes_document_id() {
        let input = PdfInput::Bytes {
            bytes: vec![1, 2, 3],
            document_id: Some("bytes-doc".to_string()),
            filename: None,
        };
        assert_eq!(input.document_id(), "bytes-doc");
    }

    #[test]
    fn test_pdf_input_url_document_id() {
        let input = PdfInput::Url {
            document_url: "https://example.com/report.pdf".to_string(),
            document_id: Some("url-doc".to_string()),
            filename: None,
        };
        assert_eq!(input.document_id(), "url-doc");
    }

    #[test]
    fn test_pdf_input_url_document_id_from_url() {
        let input = PdfInput::Url {
            document_url: "https://example.com/myreport.pdf".to_string(),
            document_id: None,
            filename: None,
        };
        assert_eq!(input.document_id(), "myreport.pdf");
    }
}
