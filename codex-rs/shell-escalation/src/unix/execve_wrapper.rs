//! Entrypoints for execve interception helper binaries.

use clap::Parser;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Parser)]
pub struct ExecveWrapperCli {
    file: String,

    #[arg(trailing_var_arg = true)]
    argv: Vec<String>,
}

#[tokio::main]
pub async fn main_execve_wrapper() -> anyhow::Result<()> {
    codex_log_guard::guard(
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .with_writer(codex_log_guard::RedactingMakeWriter::new(std::io::stderr))
            .with_ansi(false)
            .finish(),
    )
    .init();

    let ExecveWrapperCli { file, argv } = ExecveWrapperCli::parse();
    let exit_code = crate::run_shell_escalation_execve_wrapper(file, argv).await?;
    std::process::exit(exit_code);
}
