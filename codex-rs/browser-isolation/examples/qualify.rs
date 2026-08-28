//! Explicit real-backend smoke probe. Missing infrastructure is an error, never
//! a skipped/pass result. Run only on an authorized qualification host.
use codex_browser_isolation::BrowserError;
use codex_browser_isolation::BrowserRuntime;
use codex_browser_isolation::EnginePreference;
use codex_browser_isolation::LiveBrowserAuthority;
use codex_network_proxy::NetworkProxyConfig;
use codex_network_proxy::NetworkProxyState;
use codex_network_proxy::RemoteNetworkProxyConfig;
use codex_network_proxy::RemoteNetworkProxyLaunchConfig;
use codex_security_policy::AuthorityEpoch;
use codex_security_policy::SecurityControlHealth;
use codex_security_policy::SecurityLevel;
use tokio_util::sync::CancellationToken;

struct Authority(AuthorityEpoch);
impl LiveBrowserAuthority for Authority {
    fn current(&self) -> Result<(SecurityLevel, AuthorityEpoch), BrowserError> {
        Ok((SecurityLevel::Moderate, self.0))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let authority = Authority(AuthorityEpoch::new(*uuid::Uuid::new_v4().as_bytes(), 0, 0)?);
    let cancel = CancellationToken::new();
    let runtime = BrowserRuntime::prepare(EnginePreference::Discover, &authority, &cancel).await?;
    let config = NetworkProxyConfig {
        enabled: true,
        domains: Some(serde_json::from_value(
            serde_json::json!({"example.com":"allow"}),
        )?),
        ..NetworkProxyConfig::default()
    };
    let policy =
        NetworkProxyState::from_remote_launch_config(RemoteNetworkProxyLaunchConfig::new(
            RemoteNetworkProxyConfig::from_effective_config(&config)?,
        ))?;
    if runtime.health(&authority, &cancel).await != (SecurityControlHealth::Enforcing {}) {
        return Err(BrowserError::HealthCheckFailed.into());
    }
    let page = runtime
        .acquire("https://example.com/", &policy, &authority, &cancel)
        .await?;
    if !page
        .untrusted_html
        .as_ref()
        .is_some_and(|bytes| String::from_utf8_lossy(bytes).contains("Example Domain"))
    {
        return Err(BrowserError::FetchFailed.into());
    }
    for denied in [
        "http://127.0.0.1/",
        "http://169.254.169.254/",
        "http://[::1]/",
        "file:///etc/passwd",
    ] {
        if !matches!(
            runtime.acquire(denied, &policy, &authority, &cancel).await,
            Err(BrowserError::DestinationDenied)
        ) {
            return Err(BrowserError::DestinationDenied.into());
        }
    }
    // Exact inputs and results only; never print page bodies or engine stderr.
    println!(
        "{}",
        serde_json::json!({
            "probe":"PF-30-S01-real-smoke", "result":"pass",
            "engine":format!("{:?}", runtime.engine_kind()), "image_id":runtime.image_id(),
            "health":"enforcing", "public_html_bytes":page.untrusted_html.map(|body| body.len()),
            "private_and_file_denials":4,
            "remaining":"adversarial containment, cancellation/recovery and platform matrix"
        })
    );
    Ok(())
}
