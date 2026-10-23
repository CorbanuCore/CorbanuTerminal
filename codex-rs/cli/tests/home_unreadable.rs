//! #616: the Claude Plan token helper reports an unreadable Corbanu home with
//! its possible causes and diagnostics, not as "macOS privacy". Here its own
//! sandbox denies the read, which waiting cannot fix, so it exits 1 (an
//! unexplained denial exits 75, which `corbanu exec` waits out).
#![cfg(target_os = "macos")]

use anyhow::Result;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

#[test]
fn token_helper_reports_an_unreadable_home_with_diagnostics() -> Result<()> {
    if std::env::var_os("CODEX_SANDBOX").is_some() {
        // Seatbelt cannot be nested.
        return Ok(());
    }
    let user_home = TempDir::new()?;
    let user_home = user_home.path().canonicalize()?;
    let codex_home = user_home.join("corbanu-home");
    std::fs::create_dir(&codex_home)?;
    // Deny reading the home the way a revoked volume does: EPERM on opendir.
    let profile = format!(
        "(version 1)(allow default)(deny file-read-data (literal {:?}))",
        codex_home.display().to_string()
    );

    let output = std::process::Command::new("/usr/bin/sandbox-exec")
        .arg("-p")
        .arg(profile)
        .arg(codex_utils_cargo_bin::cargo_bin("corbanu")?)
        .arg("internal-claude-oauth-token")
        .env_remove("CORBANU_HOME")
        .env_remove("PFTERMINAL_HOME")
        .env_remove("SSH_CONNECTION")
        .env("CODEX_HOME", &codex_home)
        .env("HOME", &user_home)
        .env("CORBANU_TEST_NO_NATIVE_KEYRING", "1")
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    if output.status.code() == Some(71)
        && stderr.contains("sandbox-exec: sandbox_apply: Operation not permitted")
    {
        // Already sandboxed (Bazel's darwin-sandbox): nesting is unavailable.
        return Ok(());
    }
    assert_eq!(
        (output.status.code(), output.stdout.is_empty()),
        (Some(1), true),
        "stderr:\n{stderr}"
    );
    let log = user_home.join("Library/Logs/Corbanu/home-access.log");
    assert_eq!(
        stderr.lines().next(),
        Some(
            format!(
                "Corbanu could not read its home (Operation not permitted): {} (Operation not \
                 permitted (os error 1)).",
                codex_home.display()
            )
            .as_str()
        )
    );
    assert!(
        stderr.contains(&format!(
            "Details: {}\nThis process's sandbox denies reading it.",
            log.display()
        )),
        "{stderr}"
    );
    let record = std::fs::read_to_string(&log)?;
    for fact in [
        format!("operation: read_dir (opendir) {}", codex_home.display()),
        "error: Operation not permitted (os error 1)".to_string(),
        "sandbox_check(file-read-data): denied".to_string(),
    ] {
        assert!(record.contains(&fact), "{fact} missing from:\n{record}");
    }
    Ok(())
}
