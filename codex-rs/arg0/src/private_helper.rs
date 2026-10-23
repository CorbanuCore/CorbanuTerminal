//! The private helpers a Corbanu process starts by re-executing itself with a
//! marker as argv[1]. [`crate::arg0_dispatch`] and the CLI's home preflight
//! both classify argv[1] here, so they cannot drift apart (#858).

use std::ffi::OsStr;

use codex_apply_patch::CODEX_CORE_APPLY_PATCH_ARG1;
#[cfg(unix)]
use codex_exec_server::CODEX_ARG0_EXEC_HELPER_ARG1;
use codex_exec_server::CODEX_FS_HELPER_ARG1;
#[cfg(target_os = "windows")]
use codex_windows_sandbox::CODEX_WINDOWS_SANDBOX_ARG1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrivateHelper {
    #[cfg(any(unix, windows))]
    CredentialBroker,
    #[cfg(unix)]
    Arg0Exec,
    FsHelper,
    #[cfg(target_os = "windows")]
    WindowsSandbox,
    ApplyPatch,
}

impl PrivateHelper {
    /// The helper `argv1` selects on this platform, if any.
    pub(crate) fn from_argv1(argv1: &OsStr) -> Option<Self> {
        #[cfg(any(unix, windows))]
        if argv1 == codex_network_proxy::CODEX_CREDENTIAL_BROKER_ARG1 {
            return Some(Self::CredentialBroker);
        }
        #[cfg(unix)]
        if argv1 == CODEX_ARG0_EXEC_HELPER_ARG1 {
            return Some(Self::Arg0Exec);
        }
        if argv1 == CODEX_FS_HELPER_ARG1 {
            return Some(Self::FsHelper);
        }
        #[cfg(target_os = "windows")]
        if argv1 == CODEX_WINDOWS_SANDBOX_ARG1 {
            return Some(Self::WindowsSandbox);
        }
        if argv1 == CODEX_CORE_APPLY_PATCH_ARG1 {
            return Some(Self::ApplyPatch);
        }
        None
    }

    /// Whether a parent starts this helper inside a sandbox, which may deny
    /// the Corbanu home on purpose.
    ///
    /// The credential broker is started by Core, outside any sandbox Core
    /// does not already run in, and confines itself only later. Its view of
    /// the home is Core's, which Core's own preflight has just read, so an
    /// unreadable home there is a real failure worth reporting.
    fn runs_sandboxed(self) -> bool {
        match self {
            #[cfg(unix)]
            Self::Arg0Exec => true,
            #[cfg(target_os = "windows")]
            Self::WindowsSandbox => true,
            Self::FsHelper | Self::ApplyPatch => true,
            #[cfg(any(unix, windows))]
            Self::CredentialBroker => false,
        }
    }
}

/// Whether `argv1` selects a private helper that its parent runs inside a
/// sandbox (the file helper, the arg0 exec helper, apply-patch, the Windows
/// sandbox wrapper). Such a sandbox may deny the Corbanu home on purpose, so
/// the CLI's home preflight must not run for it: it would report a
/// deliberate denial as an unreadable home and stop the helper.
pub fn is_private_helper_argv1(argv1: &OsStr) -> bool {
    PrivateHelper::from_argv1(argv1).is_some_and(PrivateHelper::runs_sandboxed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn every_dispatched_marker_is_classified() {
        let mut markers = vec![
            (CODEX_FS_HELPER_ARG1, Some(PrivateHelper::FsHelper), true),
            (
                CODEX_CORE_APPLY_PATCH_ARG1,
                Some(PrivateHelper::ApplyPatch),
                true,
            ),
            ("--version", None, false),
            ("exec", None, false),
            ("", None, false),
        ];
        // The broker keeps the preflight: it is not started in a sandbox.
        #[cfg(any(unix, windows))]
        markers.push((
            codex_network_proxy::CODEX_CREDENTIAL_BROKER_ARG1,
            Some(PrivateHelper::CredentialBroker),
            false,
        ));
        #[cfg(unix)]
        markers.push((
            CODEX_ARG0_EXEC_HELPER_ARG1,
            Some(PrivateHelper::Arg0Exec),
            true,
        ));
        #[cfg(target_os = "windows")]
        markers.push((
            CODEX_WINDOWS_SANDBOX_ARG1,
            Some(PrivateHelper::WindowsSandbox),
            true,
        ));
        for (argv1, helper, sandboxed) in markers {
            let argv1 = OsStr::new(argv1);
            assert_eq!(PrivateHelper::from_argv1(argv1), helper, "{argv1:?}");
            assert_eq!(is_private_helper_argv1(argv1), sandboxed, "{argv1:?}");
        }
    }
}
