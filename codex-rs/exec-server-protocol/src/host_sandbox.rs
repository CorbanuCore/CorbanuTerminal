//! #2050 (D510): what an executor reports about its own platform sandbox, so
//! a client on another machine can apply #1976's rule (D491) where the work
//! actually runs. Under Moderate and Aggressive, a Windows machine confines
//! agent commands and file tools only through a Windows sandbox; a client
//! sending work there must refuse it when that machine has none.

use codex_protocol::config_types::WindowsSandboxLevel;
use codex_utils_path_uri::PathConvention;
use serde::Deserialize;
use serde::Serialize;

use crate::EnvironmentInfo;

/// The executor host's operating system and Windows sandbox state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostSandboxInfo {
    pub os: HostOs,
    /// Windows only: whether the default (elevated) Windows sandbox is set up
    /// on the executor host, for the Corbanu home it runs with.
    #[serde(default)]
    pub windows_sandbox_set_up: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HostOs {
    Windows,
    Macos,
    Linux,
    #[serde(other)]
    Other,
}

impl HostOs {
    /// The operating system this process runs on.
    pub const fn current() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(target_os = "linux") {
            Self::Linux
        } else {
            Self::Other
        }
    }
}

/// Whether work sent to an executor runs inside a platform sandbox there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostSandboxVerdict {
    /// Linux or macOS, or Windows with a sandbox for the requested level.
    Confined,
    /// Windows with no sandbox for the requested level: the client sends no
    /// Windows sandbox, or the elevated one is not set up there.
    WindowsWithoutSandbox,
    /// The executor did not report its sandbox (it predates #2050) and does
    /// not clearly run on Linux or macOS, or reported a system this client
    /// does not know.
    Unreported,
}

impl EnvironmentInfo {
    /// #2050: whether work sent with `windows_sandbox_level` (the client's,
    /// which the executor applies) runs inside a platform sandbox there.
    pub fn host_sandbox_verdict(
        &self,
        windows_sandbox_level: WindowsSandboxLevel,
    ) -> HostSandboxVerdict {
        let Some(host) = self.host_sandbox else {
            // An older executor: only one that clearly runs on Linux or
            // macOS (a POSIX working directory or shell, nothing of Windows)
            // keeps working; Windows or unclear is refused (fail closed).
            return if self.reports_posix_host() {
                HostSandboxVerdict::Confined
            } else {
                HostSandboxVerdict::Unreported
            };
        };
        match host.os {
            HostOs::Linux | HostOs::Macos => return HostSandboxVerdict::Confined,
            // A system this client does not know: it cannot tell (fail
            // closed).
            HostOs::Other => return HostSandboxVerdict::Unreported,
            HostOs::Windows => {}
        }
        match windows_sandbox_level {
            WindowsSandboxLevel::Disabled => HostSandboxVerdict::WindowsWithoutSandbox,
            // The non-admin sandbox needs no setup and refuses the protected
            // levels' read restrictions itself (#300), as on a local Windows.
            WindowsSandboxLevel::RestrictedToken => HostSandboxVerdict::Confined,
            WindowsSandboxLevel::Elevated if host.windows_sandbox_set_up => {
                HostSandboxVerdict::Confined
            }
            WindowsSandboxLevel::Elevated => HostSandboxVerdict::WindowsWithoutSandbox,
        }
    }

    fn reports_posix_host(&self) -> bool {
        let shell_name = self.shell.name.to_ascii_lowercase();
        let shell_path = self.shell.path.as_str();
        let windows_shell = matches!(shell_name.as_str(), "powershell" | "pwsh" | "cmd")
            || shell_path.contains('\\')
            || shell_path.as_bytes().get(1) == Some(&b':');
        let cwd = self
            .cwd
            .as_ref()
            .and_then(codex_utils_path_uri::PathUri::infer_path_convention);
        if windows_shell || cwd == Some(PathConvention::Windows) {
            return false;
        }
        cwd == Some(PathConvention::Posix) || shell_path.starts_with('/')
    }
}

#[cfg(test)]
#[path = "host_sandbox_tests.rs"]
mod tests;
