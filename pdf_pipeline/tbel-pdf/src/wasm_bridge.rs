use js_sys::{Object, Reflect, Uint8Array};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use crate::error::PipelineError;
use crate::markdown::{clean_latex_from_markdown, preprocess_markdown};
use crate::models::{PdfInput, ReportTable, ReportType};
use crate::ocr::{MistralOcrProvider, OcrProvider, OcrProviderConfig};
use crate::processing::{ProcessingFacadeBuilder, ProcessingResult};
use crate::table_extraction::{extract_table_candidates_from_markdown, is_valid_financial_table};

mod xlsx_export {
    use rust_xlsxwriter::{Workbook, XlsxError};

    use crate::models::{ReportTable, ReportType};

    const EXCEL_SHEET_NAME_MAX_CHARS: usize = 31;

    fn excel_safe_sheet_name(report_type: &ReportType) -> String {
        report_type
            .russian_name()
            .chars()
            .take(EXCEL_SHEET_NAME_MAX_CHARS)
            .collect()
    }

    pub fn tables_to_xlsx(
        tables: &[ReportTable],
        report_type: &ReportType,
    ) -> Result<Vec<u8>, String> {
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();

        let sheet_name = excel_safe_sheet_name(report_type);

        sheet
            .set_name(&sheet_name)
            .map_err(|err: XlsxError| err.to_string())?;

        let mut current_row = 0u32;
        for (table_index, table) in tables.iter().enumerate() {
            if table_index > 0 {
                current_row += 1;
            }

            for (col_index, header) in table.headers.iter().enumerate() {
                sheet
                    .write_string(current_row, col_index as u16, header)
                    .map_err(|err: XlsxError| err.to_string())?;
            }
            current_row += 1;

            for row in &table.rows {
                for (col_index, cell) in row.iter().enumerate() {
                    sheet
                        .write_string(current_row, col_index as u16, &cell.content)
                        .map_err(|err: XlsxError| err.to_string())?;
                }
                current_row += 1;
            }
        }

        workbook
            .save_to_buffer()
            .map_err(|err: XlsxError| err.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JsErrorCode {
    NoFinancialTablesFound,
    UnsupportedLayout,
    InvalidHeader,
    DimensionValidationFailed,
    ProviderError,
    ParseError,
    ExportError,
    InvalidInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsError {
    pub code: JsErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsSuccess {
    pub document_id: String,
    pub report_type: String,
    pub tables: Vec<JsReportTable>,
    pub page_count: usize,
    pub xlsx: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsReportTable {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub table_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MarkdownRequest {
    markdown: String,
    report_type: String,
    #[serde(default)]
    document_id: Option<String>,
    #[serde(default)]
    page_count: Option<u32>,
    #[serde(default)]
    include_xlsx: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PdfRequest {
    #[serde(default)]
    bytes: Option<Vec<u8>>,
    #[serde(default)]
    url: Option<String>,
    report_type: String,
    ocr: JsOcrConfig,
    #[serde(default)]
    document_id: Option<String>,
    #[serde(default)]
    filename: Option<String>,
    #[serde(default)]
    include_xlsx: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct JsOcrConfig {
    api_key: String,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    document_url_override: Option<String>,
}

impl From<&PipelineError> for JsError {
    fn from(err: &PipelineError) -> Self {
        let (code, message) = match err {
            PipelineError::NoFinancialTablesFound => {
                (JsErrorCode::NoFinancialTablesFound, err.to_string())
            }
            PipelineError::UnsupportedLayout(message) => {
                (JsErrorCode::UnsupportedLayout, message.clone())
            }
            PipelineError::InvalidHeader(message) => (JsErrorCode::InvalidHeader, message.clone()),
            PipelineError::DimensionValidationFailed(message) => {
                (JsErrorCode::DimensionValidationFailed, message.clone())
            }
            PipelineError::ProviderError(message) => (JsErrorCode::ProviderError, message.clone()),
            PipelineError::ParseError(message) => (JsErrorCode::ParseError, message.clone()),
            PipelineError::ExportError(message) => (JsErrorCode::ExportError, message.clone()),
            PipelineError::IoError {
                operation,
                path,
                message,
            } => (
                JsErrorCode::ProviderError,
                format!("IO error during {operation} on '{path}': {message}"),
            ),
        };

        Self { code, message }
    }
}

impl From<&ReportTable> for JsReportTable {
    fn from(table: &ReportTable) -> Self {
        Self {
            headers: table.headers.clone(),
            rows: table
                .rows
                .iter()
                .map(|row| row.iter().map(|cell| cell.content.clone()).collect())
                .collect(),
            table_index: table.table_index,
        }
    }
}

fn js_error(code: JsErrorCode, message: impl Into<String>) -> JsError {
    JsError {
        code,
        message: message.into(),
    }
}

fn js_error_value(code: JsErrorCode, message: impl Into<String>) -> JsValue {
    serde_wasm_bindgen::to_value(&js_error(code, message)).unwrap_or(JsValue::NULL)
}

fn parse_report_type(report_type: &str) -> Result<ReportType, JsError> {
    report_type.parse::<ReportType>().map_err(|message| {
        js_error(
            JsErrorCode::InvalidInput,
            format!("Invalid report type '{report_type}': {message}"),
        )
    })
}

fn success_from_result(result: ProcessingResult, include_xlsx: bool) -> Result<JsSuccess, JsError> {
    let xlsx = if include_xlsx {
        Some(
            xlsx_export::tables_to_xlsx(&result.tables, &result.report_type)
                .map_err(|message| js_error(JsErrorCode::ExportError, message))?,
        )
    } else {
        None
    };

    Ok(JsSuccess {
        document_id: result.document_id,
        report_type: result.report_type.to_string(),
        tables: result.tables.iter().map(JsReportTable::from).collect(),
        page_count: result.page_count,
        xlsx,
    })
}

fn success_to_js_value(success: &JsSuccess) -> Result<JsValue, JsValue> {
    let value = serde_wasm_bindgen::to_value(success).map_err(|err| {
        js_error_value(
            JsErrorCode::ParseError,
            format!("Failed to serialize success payload: {err}"),
        )
    })?;

    if let Some(bytes) = &success.xlsx {
        let object: Object = value.clone().unchecked_into();
        Reflect::set(
            &object,
            &JsValue::from_str("xlsx"),
            &Uint8Array::from(bytes.as_slice()),
        )
        .map_err(|_| {
            js_error_value(
                JsErrorCode::ParseError,
                "Failed to attach XLSX bytes to success payload.",
            )
        })?;
    }

    Ok(value)
}

fn build_pdf_input(request: &PdfRequest) -> Result<PdfInput, JsError> {
    match (&request.bytes, &request.url) {
        (Some(_), Some(_)) => Err(js_error(
            JsErrorCode::InvalidInput,
            "Provide either 'bytes' or 'url', not both.",
        )),
        (None, None) => Err(js_error(
            JsErrorCode::InvalidInput,
            "Either 'bytes' or 'url' is required.",
        )),
        (Some(bytes), None) => {
            if bytes.is_empty() {
                return Err(js_error(
                    JsErrorCode::InvalidInput,
                    "'bytes' must not be empty.",
                ));
            }

            Ok(PdfInput::Bytes {
                bytes: bytes.clone(),
                document_id: request.document_id.clone(),
                filename: request.filename.clone(),
            })
        }
        (None, Some(url)) => {
            if url.trim().is_empty() {
                return Err(js_error(
                    JsErrorCode::InvalidInput,
                    "'url' must not be empty.",
                ));
            }

            Ok(PdfInput::Url {
                document_url: url.clone(),
                document_id: request.document_id.clone(),
                filename: request.filename.clone(),
            })
        }
    }
}

fn decode_request<T>(value: JsValue) -> Result<T, JsValue>
where
    T: serde::de::DeserializeOwned,
{
    serde_wasm_bindgen::from_value(value).map_err(|err| {
        js_error_value(
            JsErrorCode::InvalidInput,
            format!("Invalid request payload: {err}"),
        )
    })
}

fn process_markdown_request(request: MarkdownRequest) -> Result<JsSuccess, JsError> {
    let report_type = parse_report_type(&request.report_type)?;
    let facade = ProcessingFacadeBuilder::default()
        .report_type(report_type)
        .build();

    let document_id = request.document_id.unwrap_or_else(|| "unknown".to_string());
    let page_count = request.page_count.unwrap_or(1) as usize;

    let result = facade
        .process_markdown(&request.markdown, page_count, document_id)
        .map_err(|err| JsError::from(&err))?;

    success_from_result(result, request.include_xlsx.unwrap_or(false))
}

async fn process_pdf_request_with_provider(
    request: PdfRequest,
    ocr_provider: &dyn OcrProvider,
) -> Result<JsSuccess, JsError> {
    let report_type = parse_report_type(&request.report_type)?;
    let facade = ProcessingFacadeBuilder::default()
        .report_type(report_type)
        .build();
    let input = build_pdf_input(&request)?;

    let result = facade
        .process(input, ocr_provider)
        .await
        .map_err(|err| JsError::from(&err))?;

    success_from_result(result, request.include_xlsx.unwrap_or(false))
}

fn mistral_provider_config(config: &JsOcrConfig) -> Result<OcrProviderConfig, JsError> {
    if config.api_key.trim().is_empty() {
        return Err(js_error(
            JsErrorCode::InvalidInput,
            "ocr.api_key is required.",
        ));
    }

    Ok(OcrProviderConfig {
        api_key: config.api_key.clone(),
        model: config.model.clone().unwrap_or_default(),
        document_url_override: config.document_url_override.clone(),
    })
}

#[wasm_bindgen]
pub fn process_markdown(options: JsValue) -> js_sys::Promise {
    let request = match decode_request::<MarkdownRequest>(options) {
        Ok(request) => request,
        Err(err) => return js_sys::Promise::reject(&err),
    };

    future_to_promise(async move {
        match process_markdown_request(request) {
            Ok(success) => success_to_js_value(&success),
            Err(err) => Err(serde_wasm_bindgen::to_value(&err).unwrap_or(JsValue::NULL)),
        }
    })
}

#[wasm_bindgen]
pub fn process_pdf(options: JsValue) -> js_sys::Promise {
    let request = match decode_request::<PdfRequest>(options) {
        Ok(request) => request,
        Err(err) => return js_sys::Promise::reject(&err),
    };

    let provider_config = match mistral_provider_config(&request.ocr) {
        Ok(config) => config,
        Err(err) => {
            return js_sys::Promise::reject(
                &serde_wasm_bindgen::to_value(&err).unwrap_or(JsValue::NULL),
            );
        }
    };

    future_to_promise(async move {
        let provider = MistralOcrProvider::with_config(provider_config);
        match process_pdf_request_with_provider(request, &provider).await {
            Ok(success) => success_to_js_value(&success),
            Err(err) => Err(serde_wasm_bindgen::to_value(&err).unwrap_or(JsValue::NULL)),
        }
    })
}

#[wasm_bindgen]
pub fn validate_markdown(markdown: &str) -> u32 {
    let cleaned = clean_latex_from_markdown(markdown);
    let preprocessed = preprocess_markdown(&cleaned);
    let candidates = extract_table_candidates_from_markdown(&preprocessed);

    candidates
        .iter()
        .filter(|table| is_valid_financial_table(table))
        .count() as u32
}

#[wasm_bindgen]
pub fn get_supported_report_types() -> js_sys::Array {
    let array = js_sys::Array::new();
    for value in [
        "balance_sheet",
        "income_statement",
        "statement_cash_flow",
        "statement_equity_changes",
    ] {
        array.push(&JsValue::from_str(value));
    }
    array
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocr::MockOcrProvider;
    use wasm_bindgen_futures::JsFuture;
    use wasm_bindgen_test::wasm_bindgen_test;

    const TEST_MARKDOWN: &str = r#"| Код строки | Наименование показателей | 2024 | 2023 |
| --- | --- | --- | --- |
| 010 | Основные средства | 1 000 | 900 |
| 020 | Нематериальные активы | 500 | 400 |
| 030 | Вложения в долгосрочные активы | 300 | 200 |
| 040 | Долгосрочная дебиторская задолженность | 200 | 150 |
| 050 | ИТОГО по разделу I | 2 000 | 1 650 |
| 060 | Запасы | 800 | 700 |
| 070 | Налог на добавленную стоимость | 100 | 80 |
| 080 | Денежные средства | 600 | 500 |
| 090 | ИТОГО по разделу II | 1 500 | 1 280 |
| 100 | БАЛАНС | 3 500 | 2 930 |"#;

    fn markdown_request_with_type(include_xlsx: bool, report_type: &str) -> JsValue {
        serde_wasm_bindgen::to_value(&MarkdownRequest {
            markdown: TEST_MARKDOWN.to_string(),
            report_type: report_type.to_string(),
            document_id: Some("wasm-markdown-doc".to_string()),
            page_count: Some(1),
            include_xlsx: Some(include_xlsx),
        })
        .unwrap()
    }

    fn markdown_request(include_xlsx: bool) -> JsValue {
        markdown_request_with_type(include_xlsx, "balance_sheet")
    }

    fn pdf_request() -> PdfRequest {
        PdfRequest {
            bytes: Some(vec![0x25, 0x50, 0x44, 0x46]),
            url: None,
            report_type: "balance_sheet".to_string(),
            ocr: JsOcrConfig {
                api_key: "test-key".to_string(),
                model: None,
                document_url_override: None,
            },
            document_id: Some("mock-doc".to_string()),
            filename: Some("report.pdf".to_string()),
            include_xlsx: Some(true),
        }
    }

    #[wasm_bindgen_test]
    fn test_validate_markdown_finds_valid_table() {
        assert_eq!(validate_markdown(TEST_MARKDOWN), 1);
    }

    #[wasm_bindgen_test]
    async fn test_process_markdown_returns_success() {
        let value = JsFuture::from(process_markdown(markdown_request(false)))
            .await
            .unwrap();
        let success: JsSuccess = serde_wasm_bindgen::from_value(value).unwrap();

        assert_eq!(success.document_id, "wasm-markdown-doc");
        assert_eq!(success.report_type, "balance_sheet");
        assert_eq!(success.tables.len(), 1);
        assert!(success.xlsx.is_none());
    }

    #[wasm_bindgen_test]
    async fn test_process_markdown_can_include_xlsx() {
        let value = JsFuture::from(process_markdown(markdown_request(true)))
            .await
            .unwrap();
        let success: JsSuccess = serde_wasm_bindgen::from_value(value).unwrap();

        let xlsx = success.xlsx.expect("xlsx bytes should exist");
        assert!(!xlsx.is_empty());
        assert_eq!(&xlsx[..4], b"PK\x03\x04");
    }

    #[wasm_bindgen_test]
    async fn test_process_markdown_cash_flow_can_include_xlsx() {
        let value = JsFuture::from(process_markdown(markdown_request_with_type(
            true,
            "statement_cash_flow",
        )))
        .await
        .unwrap();
        let success: JsSuccess = serde_wasm_bindgen::from_value(value).unwrap();

        let xlsx = success.xlsx.expect("xlsx bytes should exist");
        assert!(!xlsx.is_empty());
        assert_eq!(&xlsx[..4], b"PK\x03\x04");
    }

    #[wasm_bindgen_test]
    async fn test_process_markdown_rejects_invalid_report_type() {
        let request = serde_wasm_bindgen::to_value(&MarkdownRequest {
            markdown: TEST_MARKDOWN.to_string(),
            report_type: "not_a_report".to_string(),
            document_id: None,
            page_count: None,
            include_xlsx: None,
        })
        .unwrap();

        let value = JsFuture::from(process_markdown(request)).await.unwrap_err();
        let error: JsError = serde_wasm_bindgen::from_value(value).unwrap();
        assert_eq!(error.code, JsErrorCode::InvalidInput);
    }

    #[wasm_bindgen_test]
    async fn test_process_pdf_with_mock_provider_supports_ocr_path() {
        let request = pdf_request();
        let provider = MockOcrProvider::new().with_text("mock-doc", TEST_MARKDOWN);

        let success = process_pdf_request_with_provider(request, &provider)
            .await
            .unwrap();

        assert_eq!(success.document_id, "mock-doc");
        assert_eq!(success.tables.len(), 1);
        assert!(success.xlsx.is_some());
    }

    #[wasm_bindgen_test]
    async fn test_process_pdf_rejects_missing_input() {
        let request = serde_wasm_bindgen::to_value(&serde_json::json!({
            "report_type": "balance_sheet",
            "ocr": { "api_key": "test-key" }
        }))
        .unwrap();

        let value = JsFuture::from(process_pdf(request)).await.unwrap_err();
        let error: JsError = serde_wasm_bindgen::from_value(value).unwrap();
        assert_eq!(error.code, JsErrorCode::InvalidInput);
    }

    #[wasm_bindgen_test]
    fn test_get_supported_report_types_returns_array() {
        assert_eq!(get_supported_report_types().length(), 4);
    }
}
