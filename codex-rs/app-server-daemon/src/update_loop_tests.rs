use std::sync::Mutex;

use pretty_assertions::assert_eq;

use super::InstallerHttp;
use super::InstallerResponse;
use super::fetch_installer_script;
use super::install_latest_standalone;
use super::update_modes_for_identities;
use crate::RestartMode;
use crate::UpdaterRefreshMode;
use crate::managed_install::executable_identity_from_bytes;

#[test]
fn unchanged_updater_uses_version_based_restart() {
    assert_eq!(
        update_modes_for_identities(
            &executable_identity_from_bytes(b"same"),
            &executable_identity_from_bytes(b"same"),
        ),
        (RestartMode::IfVersionChanged, UpdaterRefreshMode::None)
    );
}

#[test]
fn changed_updater_forces_refresh_even_when_version_may_match() {
    assert_eq!(
        update_modes_for_identities(
            &executable_identity_from_bytes(b"old"),
            &executable_identity_from_bytes(b"new"),
        ),
        (
            RestartMode::Always,
            UpdaterRefreshMode::ReexecIfManagedBinaryChanged,
        )
    );
}

const DISABLED_MESSAGE: &str = "Corbanu Terminal daemon self-install/update is disabled until the managed installer supports Corbanu Terminal. Install or update Corbanu Terminal manually using the instructions at https://github.com/CorbanuCore/CorbanuTerminal#install. Then restart the daemon.";

#[tokio::test]
async fn managed_installer_neither_fetches_nor_executes_script() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let marker = temp.path().join("installer-ran");
    let quoted_marker = marker.display().to_string().replace('\'', "'\\''");
    let script = format!("#!/bin/sh\ntouch '{quoted_marker}'\n").into_bytes();
    let http = FakeInstallerHttp::new(InstallerResponse::Success(script));

    let result = install_latest_standalone(&http).await;

    assert_eq!(
        (
            http.requested_urls(),
            marker.exists(),
            result.err().map(|error| error.to_string()),
        ),
        (
            Vec::<String>::new(),
            false,
            Some(DISABLED_MESSAGE.to_string())
        )
    );
}

#[tokio::test]
async fn disabled_installer_does_not_request_even_an_error_response() {
    let http = FakeInstallerHttp::new(InstallerResponse::Unsuccessful { status: 503 });

    let error = fetch_installer_script(&http)
        .await
        .expect_err("managed installer is disabled");

    assert_eq!(
        (http.requested_urls(), error.to_string()),
        (Vec::<String>::new(), DISABLED_MESSAGE.to_string())
    );
}

#[tokio::test]
async fn updater_entry_point_refuses_before_waiting_or_fetching() {
    let factory = codex_http_client::HttpClientFactory::new(
        codex_http_client::OutboundProxyPolicy::ReqwestDefault,
    );
    // The old loop waits five minutes before fetching. Bound this regression
    // test well below that delay so fail-before cannot reach the real network.
    let result = tokio::time::timeout(
        std::time::Duration::from_millis(100),
        crate::run_pid_update_loop(factory),
    )
    .await
    .expect("disabled updater must return immediately");

    assert_eq!(
        result.expect_err("managed updater is disabled").to_string(),
        DISABLED_MESSAGE
    );
}

struct FakeInstallerHttp {
    response: InstallerResponse,
    requested_urls: Mutex<Vec<String>>,
}

impl FakeInstallerHttp {
    fn new(response: InstallerResponse) -> Self {
        Self {
            response,
            requested_urls: Mutex::new(Vec::new()),
        }
    }

    fn requested_urls(&self) -> Vec<String> {
        self.requested_urls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl InstallerHttp for FakeInstallerHttp {
    async fn get(&self, url: &str) -> anyhow::Result<InstallerResponse> {
        self.requested_urls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(url.to_string());
        Ok(self.response.clone())
    }
}
