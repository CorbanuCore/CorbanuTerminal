use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    codex_home: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    codex_wallet_daemon::run_wallet_daemon(Args::parse().codex_home).await
}
