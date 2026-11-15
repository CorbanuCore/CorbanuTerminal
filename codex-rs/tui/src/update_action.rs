#[cfg(any(not(debug_assertions), test))]
use codex_install_context::InstallContext;

/// Update action the CLI should perform after the TUI exits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateAction {
    /// Replace the local daemon after restoring the terminal.
    Daemon(DaemonUpdateSource),
    /// Update via the canonical Corbanu Terminal Unix installer.
    StandaloneUnix,
    /// Update via the canonical Corbanu Terminal PowerShell installer.
    StandaloneWindows,
}

impl UpdateAction {
    #[cfg(any(not(debug_assertions), test))]
    pub(crate) fn from_install_context(_context: &InstallContext) -> Self {
        canonical_installer_action()
    }

    /// Returns the list of command-line arguments for invoking the update.
    pub fn command_args(self) -> (&'static str, &'static [&'static str]) {
        match self {
            // The daemon menu runs the selected Corbanu executable; this name is only a label.
            UpdateAction::Daemon(source) => ("corbanu", source.command_args()),
            UpdateAction::StandaloneUnix => (
                "sh",
                &[
                    "-c",
                    r#"s=$(curl -fsSL https://github.com/CorbanuCore/CorbanuTerminal/releases/latest/download/install.sh) && CORBANU_NON_INTERACTIVE=1 eval "$s""#,
                ],
            ),
            UpdateAction::StandaloneWindows => (
                "powershell",
                &[
                    "-ExecutionPolicy",
                    "Bypass",
                    "-c",
                    "$env:CORBANU_NON_INTERACTIVE=1; irm https://github.com/CorbanuCore/CorbanuTerminal/releases/latest/download/install.ps1 | iex",
                ],
            ),
        }
    }

    /// Returns string representation of the command-line arguments for invoking the update.
    pub fn command_str(self) -> String {
        let (command, args) = self.command_args();
        shlex::try_join(std::iter::once(command).chain(args.iter().copied()))
            .unwrap_or_else(|_| format!("{command} {}", args.join(" ")))
    }
}

#[cfg(any(not(debug_assertions), test))]
fn canonical_installer_action() -> UpdateAction {
    if cfg!(windows) {
        UpdateAction::StandaloneWindows
    } else {
        UpdateAction::StandaloneUnix
    }
}

#[cfg(any(not(debug_assertions), test))]
#[cfg_attr(all(debug_assertions, test), allow(dead_code))]
pub fn get_update_action() -> Option<UpdateAction> {
    Some(UpdateAction::from_install_context(InstallContext::current()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_install_context::InstallMethod;
    use codex_install_context::StandalonePlatform;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use pretty_assertions::assert_eq;

    #[test]
    fn maps_install_context_to_update_action() {
        let native_release_dir =
            AbsolutePathBuf::from_absolute_path(std::env::temp_dir().join("native-release"))
                .expect("temp dir path should be absolute");

        for method in [
            InstallMethod::Other,
            InstallMethod::Npm,
            InstallMethod::Bun,
            InstallMethod::VitePlus,
            InstallMethod::Pnpm,
            InstallMethod::Brew,
            InstallMethod::Standalone {
                platform: StandalonePlatform::Unix,
                release_dir: native_release_dir.clone(),
                resources_dir: Some(native_release_dir.join("codex-resources")),
            },
            InstallMethod::Standalone {
                platform: StandalonePlatform::Windows,
                release_dir: native_release_dir.clone(),
                resources_dir: Some(native_release_dir.join("codex-resources")),
            },
        ] {
            assert_eq!(
                UpdateAction::from_install_context(&InstallContext {
                    method,
                    package_layout: None,
                }),
                canonical_installer_action()
            );
        }
    }

    #[test]
    fn standalone_update_commands_rerun_latest_installer() {
        assert_eq!(
            UpdateAction::StandaloneUnix.command_args(),
            (
                "sh",
                &[
                    "-c",
                    r#"s=$(curl -fsSL https://github.com/CorbanuCore/CorbanuTerminal/releases/latest/download/install.sh) && CORBANU_NON_INTERACTIVE=1 eval "$s""#
                ][..],
            )
        );
        assert_eq!(
            UpdateAction::StandaloneWindows.command_args(),
            (
                "powershell",
                &[
                    "-ExecutionPolicy",
                    "Bypass",
                    "-c",
                    "$env:CORBANU_NON_INTERACTIVE=1; irm https://github.com/CorbanuCore/CorbanuTerminal/releases/latest/download/install.ps1 | iex"
                ][..],
            )
        );
    }

    #[cfg(unix)]
    #[test]
    fn standalone_unix_update_fails_when_installer_download_fails() {
        let (command, args) = UpdateAction::StandaloneUnix.command_args();
        let offline_command = args[1].replace("https://github.com", "file:///dev/null");
        assert_ne!(offline_command, args[1], "installer URL must be replaced");
        let output = std::process::Command::new(command)
            .args([args[0], &offline_command])
            .env_clear()
            .env("HOME", "/dev/null")
            .env("PATH", "/usr/bin:/bin")
            .output()
            .expect("run standalone Unix update command");

        assert!(!output.status.success(), "{output:?}");
    }
}

/// Package source explicitly selected by the user in the daemon menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonUpdateSource {
    PublicStable,
    ThisCli,
}

impl DaemonUpdateSource {
    pub fn command_args(self) -> &'static [&'static str] {
        match self {
            Self::PublicStable => &["app-server", "daemon", "update"],
            Self::ThisCli => &["app-server", "daemon", "update", "--from-cli", "--yes"],
        }
    }
}
