//! #2015: the config an agent started from this thread by a client (a
//! detached review) runs with. It is this session's current effective
//! settings, never the config the process started with: the approval policy
//! and reviewer, the permission profile, the cwd and workspace roots, the
//! Windows sandbox and the security level in force, including what changed
//! since the thread started.

use codex_protocol::config_types::WindowsSandboxLevel;

use super::session::Session;
use crate::config::Config;
use crate::windows_sandbox::WindowsSandboxLevelExt;

impl Session {
    pub(crate) async fn detached_agent_config(&self) -> Config {
        let mut config = {
            let state = self.state.lock().await;
            let configuration = &state.session_configuration;
            let mut config = Self::build_effective_session_config(configuration);
            raise_windows_sandbox(&mut config, configuration.windows_sandbox_level);
            config
        };
        // A stricter level committed during this session holds for it until
        // the next start; the new agent starts at least there.
        if let Ok(policy) = self
            .services
            .agent_control
            .effective_security_policy()
            .snapshot_for_agent(self.thread_id)
        {
            config.security_level = config.security_level.max(policy.level);
        }
        config
    }
}

fn windows_sandbox_rank(level: WindowsSandboxLevel) -> u8 {
    match level {
        WindowsSandboxLevel::Disabled => 0,
        WindowsSandboxLevel::RestrictedToken => 1,
        WindowsSandboxLevel::Elevated => 2,
    }
}

/// Selects `level`'s Windows sandbox in `config` when it is stronger than
/// the one `config` selects; never lowers it.
fn raise_windows_sandbox(config: &mut Config, level: WindowsSandboxLevel) {
    if windows_sandbox_rank(level) <= windows_sandbox_rank(WindowsSandboxLevel::from_config(config))
    {
        return;
    }
    match level {
        WindowsSandboxLevel::Elevated => config.set_windows_elevated_sandbox_enabled(/*value*/ true),
        WindowsSandboxLevel::RestrictedToken => config.set_windows_sandbox_enabled(/*value*/ true),
        WindowsSandboxLevel::Disabled => {}
    }
}

#[cfg(test)]
#[path = "detached_agent_config_tests.rs"]
mod tests;
