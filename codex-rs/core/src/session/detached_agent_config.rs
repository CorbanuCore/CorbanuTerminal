//! #2015 (D519, D711): an agent a client starts from this thread. Upstream
//! replaced the detached review with `thread/fork` + `turn/start`; the fork
//! still runs with this session's current effective settings, never the
//! config the process started with: the approval policy and reviewer, the
//! permission profile, the Windows sandbox and the security level in force,
//! including what changed since the thread started.

use codex_protocol::config_types::WindowsSandboxLevel;
use codex_security_policy::SecurityLevel;

use super::session::Session;
use crate::config::Config;
use crate::security::SecurityPolicyError;
use crate::windows_sandbox::WindowsSandboxLevelExt;
use crate::windows_sandbox::raise_windows_sandbox;

impl Session {
    /// This session's current effective config: its settings now, with the
    /// Windows sandbox and security level in force.
    pub(crate) async fn detached_agent_config(&self) -> Config {
        let mut config = {
            let state = self.state.lock().await;
            let configuration = &state.session_configuration;
            let mut config = self.build_effective_session_config(configuration);
            raise_windows_sandbox(&mut config, configuration.windows_sandbox_level);
            config
        };
        // A stricter level committed during this session holds for it until
        // the next start; the new agent starts at least there. A policy state
        // that can no longer be read fails closed.
        // A session starts only on the local backend, which holds the policy.
        let policy = self.services.agent_control.local().map(|control| {
            control
                .effective_security_policy()
                .snapshot_for_agent(self.thread_id)
        });
        match policy {
            Some(Ok(policy)) => config.security_level = config.security_level.max(policy.level),
            // No level in force for this thread: the configured one stands.
            Some(Err(
                SecurityPolicyError::RuntimeNotInitialized | SecurityPolicyError::UnknownAgent(_),
            )) => {}
            Some(Err(_)) | None => config.security_level = SecurityLevel::Aggressive,
        }
        refresh_windows_unsandboxed_notice(&mut config);
        config
    }

    /// D711 (a): `config` is for a thread a client forks from this one. It
    /// never runs below this thread's current Windows sandbox or security
    /// level, whatever the fork's own config layers or request asked for;
    /// a stricter fork config stays stricter.
    pub(crate) async fn hold_fork_to_current_floor(&self, config: &mut Config) {
        let parent = self.detached_agent_config().await;
        raise_windows_sandbox(config, WindowsSandboxLevel::from_config(&parent));
        config.security_level = config.security_level.max(parent.security_level);
        refresh_windows_unsandboxed_notice(config);
    }
}

/// Config was loaded before the live security level or Windows sandbox
/// changed. Replace only this derived notice, preserving unrelated warnings.
fn refresh_windows_unsandboxed_notice(config: &mut Config) {
    let old_notices = [SecurityLevel::Moderate, SecurityLevel::Aggressive]
        .map(crate::security::windows_unsandboxed::startup_notice);
    config
        .startup_warnings
        .retain(|warning| !old_notices.contains(warning));
    if crate::security::windows_unsandboxed::applies(
        config.security_level,
        WindowsSandboxLevel::from_config(config),
    ) {
        config
            .startup_warnings
            .push(crate::security::windows_unsandboxed::startup_notice(
                config.security_level,
            ));
    }
}

#[cfg(test)]
#[path = "detached_agent_config_tests.rs"]
mod tests;
