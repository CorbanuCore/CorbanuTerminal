//! #2015: the config an agent started from this thread by a client (a
//! detached review) runs with. It is this session's current effective
//! settings, never the config the process started with: the approval policy
//! and reviewer, the permission profile, the cwd and workspace roots, the
//! Windows sandbox and the security level in force, including what changed
//! since the thread started.

use codex_protocol::config_types::WindowsSandboxLevel;
use codex_security_policy::SecurityLevel;

use super::session::Session;
use crate::config::Config;
use crate::security::SecurityPolicyError;
use crate::windows_sandbox::WindowsSandboxLevelExt;
use crate::windows_sandbox::raise_windows_sandbox;

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
        // the next start; the new agent starts at least there. A policy state
        // that can no longer be read fails closed.
        match self
            .services
            .agent_control
            .effective_security_policy()
            .snapshot_for_agent(self.thread_id)
        {
            Ok(policy) => config.security_level = config.security_level.max(policy.level),
            // No level in force for this thread: the configured one stands.
            Err(
                SecurityPolicyError::RuntimeNotInitialized | SecurityPolicyError::UnknownAgent(_),
            ) => {}
            Err(_) => config.security_level = SecurityLevel::Aggressive,
        }
        // Config was loaded before the parent's live security level changed.
        // Replace only this derived notice, preserving unrelated warnings.
        let old_notices = [SecurityLevel::Moderate, SecurityLevel::Aggressive]
            .map(crate::security::windows_unsandboxed::startup_notice);
        config
            .startup_warnings
            .retain(|warning| !old_notices.contains(warning));
        if crate::security::windows_unsandboxed::applies(
            config.security_level,
            WindowsSandboxLevel::from_config(&config),
        ) {
            config
                .startup_warnings
                .push(crate::security::windows_unsandboxed::startup_notice(
                    config.security_level,
                ));
        }
        config
    }
}

#[cfg(test)]
#[path = "detached_agent_config_tests.rs"]
mod tests;
