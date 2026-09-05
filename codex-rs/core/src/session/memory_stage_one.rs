//! Host-side snapshots for the detached, denial-only memory worker.

use super::session::Session;
use crate::codex_thread::ThreadConfigSnapshot;
use crate::config::Config;
use crate::memory_stage_one::StageOneMemoryDenial;
use codex_model_provider_info::ModelProviderInfo;
use codex_protocol::ThreadId;
use codex_security_policy::SecurityLevel;
use std::sync::Arc;

pub(crate) struct MemoryStageOneConfiguration {
    pub(crate) config: Arc<Config>,
    pub(crate) thread: ThreadConfigSnapshot,
    pub(crate) level: SecurityLevel,
    pub(crate) runtime_nonce: [u8; 16],
    pub(crate) session_id: String,
    pub(crate) kill_switch_active: bool,
}

impl Session {
    pub(crate) async fn memory_stage_one_configuration(
        &self,
        expected_owner: ThreadId,
        expected_provider: &ModelProviderInfo,
    ) -> Result<MemoryStageOneConfiguration, StageOneMemoryDenial> {
        if self.thread_id != expected_owner {
            return Err(StageOneMemoryDenial::OwnerMismatch);
        }
        let state = self.state.lock().await;
        let configuration = &state.session_configuration;
        if &configuration.provider != expected_provider {
            return Err(StageOneMemoryDenial::ProviderChanged);
        }
        let policy = self
            .services
            .agent_control
            .effective_security_policy()
            .snapshot_for_agent(self.thread_id)
            .map_err(|_| StageOneMemoryDenial::PolicyUnavailable)?;
        let mut config = (*configuration.original_config_do_not_use).clone();
        config.model_provider = configuration.provider.clone();
        Ok(MemoryStageOneConfiguration {
            config: Arc::new(config),
            thread: configuration.thread_config_snapshot(),
            level: policy.level,
            runtime_nonce: policy.runtime_nonce,
            session_id: policy.session_id.as_str().to_owned(),
            kill_switch_active: policy.kill_switch_active,
        })
    }
}
