//! D541 (a) through the real `sandbox` command on Windows: sandbox state that
//! says `use_mxc = false` is refused when the child's own config selects MXC,
//! and the command never runs.

#![cfg(windows)]

use std::process::Command;

use codex_mcp::SandboxState;
use codex_protocol::models::PermissionProfile;
use codex_utils_absolute_path::AbsolutePathBuf;
use codex_utils_path_uri::PathUri;
use tempfile::TempDir;

#[test]
fn false_sandbox_state_refuses_configured_mxc() -> anyhow::Result<()> {
    let codex_home = TempDir::new()?;
    let cwd = TempDir::new()?;
    std::fs::write(
        codex_home.path().join("config.toml"),
        "[windows]\nsandbox = \"mxc\"\n",
    )?;
    let marker = cwd.path().join("launched.txt");
    let state = SandboxState {
        permission_profile: PermissionProfile::read_only(),
        codex_linux_sandbox_exe: None,
        sandbox_cwd: PathUri::from_abs_path(&AbsolutePathBuf::from_absolute_path(cwd.path())?),
        use_legacy_landlock: false,
        use_mxc: false,
    };

    let output = Command::new(codex_utils_cargo_bin::cargo_bin("codex")?)
        .env_remove("CORBANU_HOME")
        .env("CODEX_HOME", codex_home.path())
        .env("CORBANU_TEST_NO_NATIVE_KEYRING", "1")
        .current_dir(cwd.path())
        .args([
            "sandbox",
            "--sandbox-state-json",
            &serde_json::to_string(&state)?,
            "--",
            "cmd",
            "/c",
            "echo launched>",
        ])
        .arg(&marker)
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{output:?}");
    assert!(stderr.contains("use_mxc = false"), "{stderr}");
    assert!(!marker.exists(), "the command must not run");
    Ok(())
}
