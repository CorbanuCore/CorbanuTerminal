//! #855: a vault lock this process may not open fails the command with the OS
//! error, and is recorded in the #616 home-access log.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;

use anyhow::Result;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

#[test]
fn an_unopenable_vault_lock_is_reported_and_recorded() -> Result<()> {
    let user_home = TempDir::new()?;
    let user_home = user_home.path().canonicalize()?;
    let codex_home = user_home.join("corbanu-home");
    let secrets = codex_home.join("secrets");
    std::fs::create_dir_all(&secrets)?;
    let lock = secrets.join(".vault.lock");
    std::fs::write(&lock, "")?;
    std::fs::set_permissions(&lock, std::fs::Permissions::from_mode(0o000))?;
    if std::fs::OpenOptions::new().write(true).open(&lock).is_ok() {
        // Running as root: permissions do not apply.
        return Ok(());
    }

    let output = std::process::Command::new(codex_utils_cargo_bin::cargo_bin("corbanu")?)
        .args(["vault", "auth-helper", "moonshot_api_key"])
        .env_remove("CORBANU_HOME")
        .env_remove("PFTERMINAL_HOME")
        .env("CODEX_HOME", &codex_home)
        .env("HOME", &user_home)
        .env("CORBANU_TEST_NO_NATIVE_KEYRING", "1")
        .output()?;
    std::fs::set_permissions(&lock, std::fs::Permissions::from_mode(0o600))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        (output.status.success(), output.stdout.is_empty()),
        (false, true),
        "stderr:\n{stderr}"
    );
    let denied = std::io::Error::from_raw_os_error(13);
    assert!(
        stderr.contains(&format!(
            "failed to open vault lock {}: {denied}",
            lock.display()
        )),
        "{stderr}"
    );
    let log = if cfg!(target_os = "macos") {
        user_home.join("Library/Logs/Corbanu/home-access.log")
    } else {
        user_home.join(".local/state/corbanu/home-access.log")
    };
    let record = std::fs::read_to_string(&log)?;
    for fact in [
        format!("operation: open vault lock {}", lock.display()),
        format!("error: {denied}"),
    ] {
        assert!(record.contains(&fact), "{fact} missing from:\n{record}");
    }
    assert!(!record.contains("moonshot_api_key"), "{record}");
    Ok(())
}
