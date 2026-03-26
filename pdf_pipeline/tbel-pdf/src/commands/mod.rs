//! CLI commands module.

#[cfg(feature = "cli")]
pub mod pipeline;

#[cfg(feature = "cli")]
use clap::{Parser, Subcommand};

#[cfg(feature = "cli")]
#[derive(Parser, Debug)]
#[command(name = "tbel-pdf")]
#[command(about = "TBel PDF processing CLI")]
#[command(subcommand_required = true)]
pub struct App {
    #[command(subcommand)]
    pub command: Commands,
}

#[cfg(feature = "cli")]
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Process a PDF document.
    Pipeline(pipeline::PipelineArgs),
}

#[cfg(feature = "cli")]
impl App {
    /// Execute the CLI command.
    pub async fn execute(self) -> Result<i32, Box<dyn std::error::Error>> {
        match self.command {
            Commands::Pipeline(args) => pipeline::execute(args).await,
        }
    }
}
