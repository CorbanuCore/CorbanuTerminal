use clap::Parser;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug, Parser)]
struct Cli {
    /// Transport endpoint: `stdio`, `stdio://`, or `ws://IP:PORT`.
    #[arg(
        long,
        value_name = "URL",
        default_value = codex_code_mode_host::DEFAULT_LISTEN_URL
    )]
    listen: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    codex_log_guard::guard(
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_writer(codex_log_guard::RedactingMakeWriter::new(std::io::stderr))
            .with_ansi(false)
            .finish(),
    )
    .init();

    codex_code_mode_host::run_main(&Cli::parse().listen).await
}
