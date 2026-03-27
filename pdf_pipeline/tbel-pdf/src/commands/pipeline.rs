//! Pipeline command implementation.

use clap::Args;
use std::path::PathBuf;

use crate::contract::{ExitCode, SuccessContract};
use crate::error::PipelineError;
use crate::models::ReportType;

/// Pipeline command arguments.
#[derive(Args, Debug)]
pub struct PipelineArgs {
    /// Input PDF URL or path.
    #[arg(short, long)]
    pub input_url: String,

    /// Report type (inferred from filename if not provided).
    #[arg(short, long)]
    pub report_type: Option<String>,

    /// Output JSON file path.
    #[arg(long)]
    pub output_json: Option<PathBuf>,

    /// Output XLSX file path.
    #[arg(long)]
    pub output_xlsx: Option<PathBuf>,

    /// Emit intermediate stage artifacts for debugging.
    #[arg(long)]
    pub emit_stage_artifacts: bool,

    /// Write JSON contract to file.
    #[arg(long)]
    pub emit_contract: Option<PathBuf>,
}

/// Execute the pipeline command.
pub async fn execute(args: PipelineArgs) -> Result<i32, Box<dyn std::error::Error>> {
    let report_type = args
        .report_type
        .as_ref()
        .and_then(|s| s.parse::<ReportType>().ok())
        .or_else(|| ReportType::try_from_filename(&args.input_url))
        .ok_or_else(|| PipelineError::ParseError("Could not determine report type".to_string()))?;

    tracing::info!(
        input_url = %args.input_url,
        report_type = %report_type,
        "Starting pipeline"
    );

    let output_json = args
        .output_json
        .unwrap_or_else(|| PathBuf::from("output.json"));

    let contract = SuccessContract {
        output_json: output_json.clone(),
        output_xlsx: args.output_xlsx,
        document_id: "doc-123".to_string(),
        report_type: report_type.to_string(),
        row_count: 0,
        column_count: 0,
    };

    if let Some(path) = args.emit_contract {
        let json = serde_json::to_string_pretty(&contract)?;
        std::fs::write(&path, json)?;
        tracing::info!(path = %path.display(), "Wrote contract");
    }

    println!("{}", serde_json::to_string_pretty(&contract)?);

    Ok(ExitCode::Success as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_args_default() {
        let args = PipelineArgs {
            input_url: "https://example.com/test.pdf".to_string(),
            report_type: None,
            output_json: None,
            output_xlsx: None,
            emit_stage_artifacts: false,
            emit_contract: None,
        };
        assert_eq!(args.input_url, "https://example.com/test.pdf");
        assert!(!args.emit_stage_artifacts);
    }
}
