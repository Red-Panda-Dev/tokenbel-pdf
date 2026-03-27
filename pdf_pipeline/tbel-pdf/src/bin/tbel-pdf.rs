//! CLI binary entry point.

#[cfg(feature = "cli")]
use clap::Parser;

#[cfg(feature = "cli")]
fn main() -> anyhow::Result<()> {
    use tbel_pdf::commands::App;

    tracing_subscriber::fmt::init();

    let args = App::parse();
    let exit_code =
        tokio::runtime::Runtime::new()?.block_on(async { args.execute().await.unwrap_or(1i32) });

    std::process::exit(exit_code);
}

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("CLI feature not enabled. Recompile with --features cli");
    std::process::exit(1);
}
