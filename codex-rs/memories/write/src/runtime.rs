use codex_core::CodexThread;
use codex_core::NewThread;
use codex_core::Prompt;
use codex_core::StartThreadOptions;
use codex_core::ThreadManager;
use codex_core::config::Config;
use codex_core::detached_memory_responses_metadata;
use codex_core::resolve_installation_id;
use codex_core::memory_stage_one::StageOneMemoryClient;
use codex_core::memory_stage_one::StageOneMemoryError;
use codex_core::memory_stage_one::StageOneMemoryDenial;
use codex_core::memory_stage_one::StageOneMemoryRequest;
use codex_login::AuthManager;
use codex_login::CodexAuth;
use codex_login::auth_env_telemetry::collect_auth_env_telemetry;
use codex_login::default_client::originator;
use codex_model_provider::ModelProvider;
use codex_model_provider::SharedModelProvider;
use codex_model_provider::create_model_provider;
use codex_otel::SessionTelemetry;
use codex_otel::TelemetryAuthMode;
use codex_protocol::SessionId;
use codex_protocol::ThreadId;
use codex_protocol::config_types::ReasoningSummary;
use codex_protocol::openai_models::ModelInfo;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::protocol::InternalSessionSource;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::ThreadSource;
use codex_protocol::protocol::TokenUsage;
use codex_protocol::user_input::UserInput;
use codex_state::StateRuntime;
use codex_terminal_detection::user_agent;
use std::sync::Arc;
use std::time::Duration;

pub(crate) struct SpawnedConsolidationAgent {
    pub(crate) thread_id: ThreadId,
    pub(crate) thread: Arc<CodexThread>,
}

#[derive(Clone, Debug)]
pub(crate) struct StageOneRequestContext {
    pub(crate) model_info: ModelInfo,
    pub(crate) session_telemetry: SessionTelemetry,
    pub(crate) reasoning_effort: Option<ReasoningEffort>,
    pub(crate) reasoning_summary: ReasoningSummary,
    pub(crate) service_tier: Option<String>,
}

impl StageOneRequestContext {
    pub(crate) fn start_timer(&self, name: &str) -> Option<codex_otel::Timer> {
        self.session_telemetry.start_timer(name, &[]).ok()
    }

    pub(crate) fn counter(&self, name: &str, inc: i64, tags: &[(&str, &str)]) {
        self.session_telemetry.counter(name, inc, tags);
    }

    pub(crate) fn histogram(&self, name: &str, value: i64, tags: &[(&str, &str)]) {
        self.session_telemetry.histogram(name, value, tags);
    }
}

pub(crate) struct MemoryStartupContext {
    thread_id: ThreadId,
    thread: Arc<CodexThread>,
    thread_manager: Arc<ThreadManager>,
    auth_manager: Arc<AuthManager>,
    provider: SharedModelProvider,
    session_telemetry: SessionTelemetry,
}

fn build_session_telemetry(
    auth_manager: &AuthManager,
    thread_id: ThreadId,
    config: &Config,
    source: SessionSource,
    model: &str,
    originator: String,
) -> SessionTelemetry {
    let auth = auth_manager.auth_cached();
    let auth = auth.as_ref();
    let auth_mode = auth.map(CodexAuth::auth_mode).map(TelemetryAuthMode::from);
    let account_id = auth.and_then(CodexAuth::get_account_id);
    let account_email = auth.and_then(CodexAuth::get_account_email);
    let auth_env_telemetry = collect_auth_env_telemetry(
        &config.model_provider,
        auth_manager.codex_api_key_env_enabled(),
    );
    SessionTelemetry::new(
        thread_id,
        model,
        model,
        account_id,
        account_email,
        auth_mode,
        originator,
        config.otel.log_user_prompt,
        user_agent(),
        source,
    )
    .with_auth_env(auth_env_telemetry.to_otel_metadata())
}

impl MemoryStartupContext {
    pub(crate) fn new(
        thread_manager: Arc<ThreadManager>,
        auth_manager: Arc<AuthManager>,
        thread_id: ThreadId,
        thread: Arc<CodexThread>,
        config: &Config,
        source: SessionSource,
    ) -> Self {
        let provider = create_model_provider(
            config.model_provider.clone(),
            Some(Arc::clone(&auth_manager)),
        );
        Self::new_with_provider(
            thread_manager,
            auth_manager,
            thread_id,
            thread,
            config,
            source,
            provider,
        )
    }

    #[cfg(test)]
    pub(crate) fn new_for_testing(
        thread_manager: Arc<ThreadManager>,
        auth_manager: Arc<AuthManager>,
        thread_id: ThreadId,
        thread: Arc<CodexThread>,
        config: &Config,
        source: SessionSource,
        provider: SharedModelProvider,
    ) -> Self {
        Self::new_with_provider(
            thread_manager,
            auth_manager,
            thread_id,
            thread,
            config,
            source,
            provider,
        )
    }

    fn new_with_provider(
        thread_manager: Arc<ThreadManager>,
        auth_manager: Arc<AuthManager>,
        thread_id: ThreadId,
        thread: Arc<CodexThread>,
        config: &Config,
        source: SessionSource,
        provider: SharedModelProvider,
    ) -> Self {
        let model = config.model.as_deref().unwrap_or("unknown");
        let session_telemetry = build_session_telemetry(
            &auth_manager,
            thread_id,
            config,
            source,
            model,
            originator().value,
        );

        Self {
            thread_id,
            thread,
            thread_manager,
            auth_manager,
            provider,
            session_telemetry,
        }
    }

    pub(crate) fn thread_id(&self) -> ThreadId {
        self.thread_id
    }

    pub(crate) fn state_db(&self) -> Option<Arc<StateRuntime>> {
        self.thread.state_db()
    }

    pub(crate) fn provider(&self) -> &dyn ModelProvider {
        self.provider.as_ref()
    }

    pub(crate) fn stage_one_provider(&self, config: &Config) -> SharedModelProvider {
        if self.provider.info() == &config.model_provider {
            Arc::clone(&self.provider)
        } else {
            create_model_provider(config.model_provider.clone(), Some(Arc::clone(&self.auth_manager)))
        }
    }

    pub(crate) async fn current_stage_one_config(&self, config: &Config) -> Result<Arc<Config>, StageOneMemoryError> {
        let snapshot = self.thread.config_snapshot().await;
        let mut current = config.clone();
        if snapshot.model_provider_id != config.model_provider_id {
            current.model_provider = config.model_providers.get(&snapshot.model_provider_id)
                .cloned().ok_or(StageOneMemoryDenial::ProviderChanged)?;
            current.model_provider_id = snapshot.model_provider_id;
        }
        current.model = Some(snapshot.model);
        // The factory asserts this snapshot against the live host again. It is
        // not a capability to choose an arbitrary replacement provider.
        self.stage_one_client(&current).await?;
        Ok(Arc::new(current))
    }

    pub(crate) fn counter(&self, name: &str, inc: i64, tags: &[(&str, &str)]) {
        self.session_telemetry.counter(name, inc, tags);
    }

    pub(crate) fn histogram(&self, name: &str, value: i64, tags: &[(&str, &str)]) {
        self.session_telemetry.histogram(name, value, tags);
    }

    pub(crate) fn start_timer(&self, name: &str) -> Option<codex_otel::Timer> {
        self.session_telemetry.start_timer(name, &[]).ok()
    }

    pub(crate) async fn stage_one_request_context(
        &self,
        config: &Config,
        model_name: &str,
        reasoning_effort: ReasoningEffort,
    ) -> StageOneRequestContext {
        let config_snapshot = self.thread.config_snapshot().await;
        let model_info = self
            .thread_manager
            .get_models_manager()
            .get_model_info(model_name, &config.to_models_manager_config())
            .await;
        let reasoning_summary = config
            .model_reasoning_summary
            .unwrap_or(model_info.default_reasoning_summary);

        StageOneRequestContext {
            model_info,
            session_telemetry: build_session_telemetry(
                &self.auth_manager,
                self.thread_id,
                config,
                config_snapshot.session_source,
                model_name,
                config_snapshot.originator,
            ),
            reasoning_effort: Some(reasoning_effort),
            reasoning_summary,
            service_tier: config_snapshot.service_tier,
        }
    }

    pub(crate) async fn stage_one_client(
        &self,
        config: &Config,
    ) -> Result<StageOneMemoryClient, StageOneMemoryError> {
        // These are assertions about this startup owner, not selectors. A provider
        // switch invalidates this run; the next startup obtains a fresh binding.
        self.thread.stage_one_memory_client(self.thread_id, &config.model_provider).await
    }

    pub(crate) async fn stream_stage_one_prompt(
        &self,
        config: &Config,
        prompt: &Prompt,
        context: &StageOneRequestContext,
    ) -> anyhow::Result<(String, Option<TokenUsage>, StageOneMemoryClient)> {
        let mut client = self.stage_one_client(config).await?;
        let installation_id = resolve_installation_id(&config.codex_home).await?;
        let config_snapshot = self.thread.config_snapshot().await;
        let session_source = config_snapshot.session_source;
        let session_id = SessionId::from(self.thread_id);
        let session_id_string = session_id.to_string();
        let window_id = format!("{}:0", self.thread_id);
        let responses_metadata = detached_memory_responses_metadata(
            installation_id,
            session_id_string,
            self.thread_id.to_string(),
            window_id,
            &session_source,
            &config.cwd,
            /*sandbox*/ None,
        )
        .await;
        let output = client.extract(StageOneMemoryRequest {
            prompt,
            model_info: &context.model_info,
            session_telemetry: &context.session_telemetry,
            reasoning_effort: context.reasoning_effort.clone(),
            reasoning_summary: context.reasoning_summary,
            service_tier: context.service_tier.clone(),
            responses_metadata: &responses_metadata,
        }).await?;
        Ok((output.text, output.token_usage, client))
    }

    pub(crate) async fn spawn_consolidation_agent(
        &self,
        config: Config,
        prompt: Vec<UserInput>,
    ) -> anyhow::Result<SpawnedConsolidationAgent> {
        let NewThread {
            thread_id, thread, ..
        } = self
            .thread_manager
            .start_thread(StartThreadOptions {
                session_source: Some(SessionSource::Internal(
                    InternalSessionSource::MemoryConsolidation,
                )),
                thread_source: Some(ThreadSource::MemoryConsolidation),
                ..StartThreadOptions::new(config)
            })
            .await?;

        let agent = SpawnedConsolidationAgent { thread_id, thread };
        if let Err(err) = agent
            .thread
            .submit(Op::UserInput {
                items: prompt,
                final_output_json_schema: None,
                responsesapi_client_metadata: None,
                additional_context: Default::default(),
                thread_settings: Default::default(),
            })
            .await
        {
            if let Err(shutdown_err) = self.shutdown_consolidation_agent(agent).await {
                tracing::warn!(
                    "failed to shut down consolidation agent after submit error: {shutdown_err}"
                );
            }
            return Err(err.into());
        }

        Ok(agent)
    }

    pub(crate) async fn shutdown_consolidation_agent(
        &self,
        agent: SpawnedConsolidationAgent,
    ) -> anyhow::Result<()> {
        let SpawnedConsolidationAgent { thread_id, thread } = agent;
        tokio::time::timeout(Duration::from_secs(10), thread.shutdown_and_wait())
            .await
            .map_err(|_| {
                anyhow::anyhow!("memory consolidation agent {thread_id} shutdown timed out")
            })??;

        self.thread_manager.remove_thread(&thread_id).await;

        Ok(())
    }
}
