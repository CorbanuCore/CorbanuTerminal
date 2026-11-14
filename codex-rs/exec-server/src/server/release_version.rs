//! Include the startup-cached package release version in executor metadata.

use codex_build_info::BuildInfo;

use crate::protocol::EnvironmentInfo;

pub(super) fn local_environment_info() -> EnvironmentInfo {
    EnvironmentInfo {
        executor_version: BuildInfo::get().version().to_string(),
        // #2050 (D510): includes this executor host's own platform sandbox.
        ..crate::environment::local_environment_info()
    }
}
