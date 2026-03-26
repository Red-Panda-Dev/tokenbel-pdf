//! Table normalization functions for financial reports.

pub const NBSP: char = '\u{00A0}';

pub fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}
