//! End-to-end behaviour of the `pfterminal-acp` launcher.
//!
//! These drive the real binary against a *fake* adapter, so they assert the
//! contract that matters to an ACP host without needing `codex-acp` or a
//! network. The three properties under test are the ones that silently break
//! an ACP session rather than failing loudly:
//!
//!   * stdout stays byte-clean when the launcher itself fails, because the
//!     host is parsing that stream as JSON-RPC;
//!   * the adapter's exit code arrives unmodified;
//!   * `CODEX_PATH` is what this launcher decided, not whatever the host
//!     happened to have in its environment.

#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use tempfile::TempDir;

fn launcher() -> Result<PathBuf> {
    Ok(codex_utils_cargo_bin::cargo_bin("pfterminal-acp")?)
}

/// Write an executable shell script and return its path.
fn write_script(dir: &Path, name: &str, body: &str) -> Result<PathBuf> {
    let path = dir.join(name);
    fs::write(&path, body)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
    Ok(path)
}

/// A fake PFTerminal. Never actually run — only resolved.
fn fake_pfterminal(dir: &Path) -> Result<PathBuf> {
    write_script(dir, "pfterminal", "#!/bin/sh\nexit 0\n")
}

#[test]
fn adapter_exit_code_is_preserved_exactly() -> Result<()> {
    let tmp = TempDir::new()?;
    let pft = fake_pfterminal(tmp.path())?;
    // 42 is chosen because it survives no plausible truncation by accident.
    let adapter = write_script(tmp.path(), "codex-acp", "#!/bin/sh\nexit 42\n")?;

    let output = std::process::Command::new(launcher()?)
        .env("PFTERMINAL_PATH", &pft)
        .env("CODEX_ACP_PATH", &adapter)
        .output()?;

    assert_eq!(
        output.status.code(),
        Some(42),
        "exit code must pass through"
    );
    Ok(())
}

#[test]
fn stdout_stays_clean_when_the_launcher_fails() -> Result<()> {
    let tmp = TempDir::new()?;
    let pft = fake_pfterminal(tmp.path())?;

    // Adapter deliberately absent: the launcher must complain on stderr and
    // leave stdout untouched, or it corrupts the host's JSON-RPC stream.
    let output = std::process::Command::new(launcher()?)
        .env("PFTERMINAL_PATH", &pft)
        .env("CODEX_ACP_PATH", tmp.path().join("does-not-exist"))
        .output()?;

    assert_eq!(output.status.code(), Some(127));
    assert!(
        output.stdout.is_empty(),
        "stdout must be empty on failure, got {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("pfterminal-acp:"),
        "the diagnostic belongs on stderr"
    );
    Ok(())
}

#[test]
fn codex_path_is_overridden_not_inherited() -> Result<()> {
    let tmp = TempDir::new()?;
    let pft = fake_pfterminal(tmp.path())?;
    let out_file = tmp.path().join("codex_path.txt");

    let adapter = write_script(
        tmp.path(),
        "codex-acp",
        &format!(
            "#!/bin/sh\nprintf '%s' \"$CODEX_PATH\" > {}\nexit 0\n",
            out_file.display()
        ),
    )?;

    let output = std::process::Command::new(launcher()?)
        .env("PFTERMINAL_PATH", &pft)
        .env("CODEX_ACP_PATH", &adapter)
        // A hostile/stale value that must not survive: inheriting it would
        // point the adapter at another agent while the client still believed
        // it was talking to PFTerminal.
        .env("CODEX_PATH", "/nonexistent/some-other-agent")
        .output()?;

    assert!(output.status.success());
    let seen = fs::read_to_string(&out_file)?;
    assert_ne!(seen, "/nonexistent/some-other-agent");
    assert_eq!(
        fs::canonicalize(&seen)?,
        fs::canonicalize(&pft)?,
        "CODEX_PATH must point at the resolved PFTerminal"
    );
    Ok(())
}

#[test]
fn arguments_reach_the_adapter_unchanged() -> Result<()> {
    let tmp = TempDir::new()?;
    let pft = fake_pfterminal(tmp.path())?;
    let out_file = tmp.path().join("argv.txt");

    // Newline-delimited so an argument containing spaces stays one argument.
    let adapter = write_script(
        tmp.path(),
        "codex-acp",
        &format!(
            "#!/bin/sh\nfor a in \"$@\"; do printf '%s\\n' \"$a\"; done > {}\nexit 0\n",
            out_file.display()
        ),
    )?;

    let output = std::process::Command::new(launcher()?)
        .env("PFTERMINAL_PATH", &pft)
        .env("CODEX_ACP_PATH", &adapter)
        .args(["--flag", "value with spaces", "--other=x,y"])
        .output()?;

    assert!(output.status.success());
    let seen: Vec<String> = fs::read_to_string(&out_file)?
        .lines()
        .map(str::to_string)
        .collect();
    assert_eq!(seen, vec!["--flag", "value with spaces", "--other=x,y"]);
    Ok(())
}

#[test]
fn version_reports_both_resolved_components() -> Result<()> {
    let tmp = TempDir::new()?;
    let pft = fake_pfterminal(tmp.path())?;
    let adapter = write_script(tmp.path(), "codex-acp", "#!/bin/sh\nexit 0\n")?;

    let output = std::process::Command::new(launcher()?)
        .env("PFTERMINAL_PATH", &pft)
        .env("CODEX_ACP_PATH", &adapter)
        .arg("--version")
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // A bare version number is useless for diagnosing a broken install; the
    // point of this output is which binaries were actually resolved.
    assert!(stdout.contains("pfterminal-acp"));
    assert!(stdout.contains("pfterminal:"));
    assert!(stdout.contains("codex-acp:"));
    Ok(())
}
