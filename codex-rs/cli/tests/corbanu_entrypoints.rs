use anyhow::Result;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

fn output(binary: &str, args: &[&str], home_env: Option<(&str, &TempDir)>) -> Result<String> {
    let mut command = std::process::Command::new(codex_utils_cargo_bin::cargo_bin(binary)?);
    command.args(args);
    if let Some((variable, home)) = home_env {
        command.env(variable, home.path());
    }
    let output = command.output()?;
    assert!(
        output.status.success(),
        "{binary} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?)
}

#[test]
fn primary_and_legacy_entrypoints_report_the_same_version() -> Result<()> {
    let corbanu = output("corbanu", &["--version"], None)?;
    let pfterminal = output("pfterminal", &["--version"], None)?;

    assert_eq!(corbanu, pfterminal.replace("pfterminal", "corbanu"));
    assert!(corbanu.contains("0.1.30"));
    Ok(())
}

#[test]
fn corbanu_help_uses_the_new_primary_command() -> Result<()> {
    let help = output("corbanu", &["--help"], None)?;

    assert!(help.contains("Corbanu Terminal CLI"));
    assert!(help.contains("corbanu [OPTIONS]"));
    assert!(!help.contains("pfterminal [OPTIONS]"));
    Ok(())
}

#[test]
fn debug_aliases_report_the_same_version() -> Result<()> {
    let corbanu_home = TempDir::new()?;
    let legacy_home = TempDir::new()?;
    let corbanu = output(
        "corbanu-debug",
        &["--version"],
        Some(("CORBANU_DEBUG_HOME", &corbanu_home)),
    )?;
    let pfterminal = output(
        "pfterminal-debug",
        &["--version"],
        Some(("PFTERMINAL_DEBUG_HOME", &legacy_home)),
    )?;

    assert_eq!(corbanu, pfterminal.replace("pfterminal", "corbanu"));
    Ok(())
}

#[test]
fn corbanu_acp_reports_the_new_launcher_and_terminal() -> Result<()> {
    let version = output("corbanu-acp", &["--version"], None)?;
    let help = output("corbanu-acp", &["--help"], None)?;

    assert!(version.contains("corbanu-acp 0.1.30"));
    assert!(version.contains("corbanu:"));
    assert!(help.contains("run Corbanu Terminal as an ACP agent"));
    assert!(help.contains("corbanu-acp [ADAPTER_ARGS...]"));
    Ok(())
}
