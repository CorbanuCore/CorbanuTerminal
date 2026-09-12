//! Release-only probe: exercise the same automatic daemon startup as the TUI.
use std::io::Write;
use std::path::PathBuf;

use codex_wallet_daemon::DaemonStatus;
use codex_wallet_daemon::WalletDaemonClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let home = PathBuf::from(std::env::args_os().nth(1).expect("disposable wallet home"));
    let result: anyhow::Result<()> = async {
        anyhow::ensure!(!home.exists(), "probe requires a fresh home");
        let client = WalletDaemonClient::new(home);
        let expected = DaemonStatus {
            wallet_exists: false,
            address: None,
            network: None,
            locked: true,
            busy: false,
            expires_in_seconds: None,
        };
        anyhow::ensure!(
            client.status().await? == expected,
            "unexpected fresh status"
        );
        // A second request must reuse the daemon successfully.
        anyhow::ensure!(
            client.status().await? == expected,
            "unexpected repeat status"
        );
        Ok(())
    }
    .await;
    match &result {
        Ok(()) => println!("wallet-client-package-ok"),
        Err(error) => println!("wallet-client-package-error: {error:#}"),
    }
    std::io::stdout().flush()?;
    // The supervisor owns this process tree and terminates it after the result.
    // Stay alive so Windows taskkill /T can also collect the spawned daemon.
    let mut done = String::new();
    std::io::stdin().read_line(&mut done)?;
    result
}
