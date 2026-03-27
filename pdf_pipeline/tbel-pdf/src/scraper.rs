use regex::Regex;

use crate::models::{CodeValue, ReportTable};

pub fn extract_company_name(content: &str) -> Option<String> {
    let re = Regex::new(r"(?i)(?:ООО|ЗАО| ОАО)\s+[\w\s]+").ok()?;
    re.find(content).map(|m| m.as_str().trim().to_string())
}

pub fn extract_financial_data(content: &str) -> Vec<CodeValue> {
    let re = Regex::new(r"(\d{4})\s*год.*?(\d[\d\s]*)[,\.]?(\d*)").unwrap();

    re.captures_iter(content)
        .filter_map(|cap| {
            Some(CodeValue {
                code: cap.get(1)?.as_str().to_string(),
                name: cap.get(2)?.as_str().replace(' ', "").parse().ok(),
                row_index: 0,
            })
        })
        .collect()
}

pub fn parse_document(content: String) -> ReportTable {
    let _company_name = extract_company_name(&content);
    let _code_values = extract_financial_data(&content);

    ReportTable::new(vec!["code".to_string(), "data".to_string()], 0)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn test_extract_company_name_ooo() {
        let content = "Отчет ООО Технологии Плюс за 2024 год";
        let result = extract_company_name(content);
        assert!(result.is_some());
        assert!(result.unwrap().contains("ООО"));
    }

    #[test]
    fn test_extract_company_name_zao() {
        let content = "Баланс ЗАО Инвест Капитал";
        let result = extract_company_name(content);
        assert!(result.is_some());
        assert!(result.unwrap().contains("ЗАО"));
    }

    #[test]
    fn test_extract_company_name_no_match() {
        let content = "Обычный текст без названия компании";
        let result = extract_company_name(content);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_financial_data() {
        let content = "2024 год 1 500 000 рублей";
        let result = extract_financial_data(content);
        assert!(!result.is_empty());
        assert_eq!(result[0].code, "2024");
    }

    #[test]
    fn test_parse_document_returns_table() {
        let content = "Отчет ООО Тест за 2024 год".to_string();
        let table = parse_document(content);
        assert_eq!(table.headers.len(), 2);
        assert_eq!(table.table_index, 0);
    }
}
