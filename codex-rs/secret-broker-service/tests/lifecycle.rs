#![cfg(target_os = "linux")]

use pretty_assertions::assert_eq;
use std::process::Command;

#[test]
fn pf_27_s01_production_service_requires_qualified_bootstrap() {
    let output =
        Command::new(codex_utils_cargo_bin::cargo_bin("codex-secret-broker-service").unwrap())
            .arg("--synthetic-inherited-socket")
            .output()
            .unwrap();
    assert_eq!(output.status.code(), Some(78));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("qualified OS bootstrap required")
    );
}

#[cfg(feature = "synthetic-fixture")]
#[path = "support/subprocess.rs"]
mod subprocess;
