//! Adapters module - re-exports from parent for backwards compatibility.

pub use crate::date::{DateNormalizer, RuleBasedDateNormalizer, StubDateNormalizer};
pub use crate::ocr::{MistralOcrProvider, MockOcrProvider, OcrProvider, StubOcrProvider};
pub use crate::pdf::PdfReader;
pub use crate::table_extraction::extract_table_candidates;
