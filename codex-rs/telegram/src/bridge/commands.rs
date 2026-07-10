use std::collections::BTreeSet;

use codex_app_server_protocol::AskForApproval;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::GitDiffToRemoteParams;
use codex_app_server_protocol::GitDiffToRemoteResponse;
use codex_app_server_protocol::Model;
use codex_app_server_protocol::ModelListParams;
use codex_app_server_protocol::ModelListResponse;
use codex_app_server_protocol::SkillsListParams;
use codex_app_server_protocol::SkillsListResponse;
use codex_app_server_protocol::ThreadCompactStartParams;
use codex_app_server_protocol::ThreadCompactStartResponse;
use codex_app_server_protocol::ThreadSettingsUpdateParams;
use codex_app_server_protocol::ThreadSettingsUpdateResponse;
use teloxide::types::ChatId;

use super::BridgeRuntime;
use crate::model_selection::CatalogModel;
use crate::model_selection::available_models;
use crate::model_selection::known_model_source;
use crate::model_selection::missing_provider_credential;
use crate::model_selection::provider_for_model;
use crate::model_selection::resolve_model;

impl BridgeRuntime {
    pub(super) async fn handle_model(
        &mut self,
        chat_id: ChatId,
        args: String,
    ) -> anyhow::Result<()> {
        let models = self.list_models().await?;
        let catalog = catalog_models(&models.data);
        let arg = args.trim();
        if arg.is_empty() {
            let (active_model, active_provider) = self.active_model_settings(chat_id).await;
            let text = render_model_list(active_model.as_deref(), &active_provider, &catalog);
            self.send_text(chat_id, &text).await?;
            return Ok(());
        }

        let (old_model, old_provider) = self.active_model_settings(chat_id).await;
        let resolution = resolve_model(arg, &catalog);

        // Reject at selection time. `model/list` is the only catalog we have, so a model
        // that is neither in it nor a known alias would be forwarded verbatim and fail at
        // turn time with a remote "Unknown model" — long after the user was told the
        // switch succeeded.
        if known_model_source(&resolution.model, &catalog).is_none() {
            self.send_text(
                chat_id,
                &format!(
                    "Unknown model: {}\n\nIt is not in this provider's catalog and not a known \
                     alias, so a turn would fail at run time. Send /model with no argument to \
                     list what the active provider serves.",
                    resolution.model
                ),
            )
            .await?;
            return Ok(());
        }

        let choice = provider_for_model(&resolution.model, &old_provider);

        // Reject at selection time when the provider that would serve this model has no
        // usable credential. Otherwise every subsequent turn dies on a missing env var and
        // the failure surfaces nowhere near the `/model` that caused it.
        if let Some(missing) = missing_provider_credential(
            &choice.provider,
            &self.config.model_providers,
            &self.config,
        ) {
            let remediation = missing.instructions.clone().unwrap_or_else(|| {
                format!(
                    "Set {} for the {} provider.",
                    missing.env_key, missing.provider
                )
            });
            self.send_text(
                chat_id,
                &format!(
                    "Not switching to {}.\n\nIt would run on provider {}, which needs {}. No key \
                     was found in the environment or the stored provider keys, so every turn \
                     would fail.\n\n{remediation}\n\nNote: /model selects a model, not a \
                     provider. Set model_provider in config.toml to change provider.",
                    resolution.model, choice.provider, missing.env_key
                ),
            )
            .await?;
            return Ok(());
        }

        self.sessions
            .set_model(chat_id, resolution.model.clone(), choice.provider.clone())
            .await?;
        let applied_to_thread = self
            .apply_thread_settings_update_if_thread_loaded(
                chat_id,
                ThreadSettingsUpdateParams {
                    model: Some(resolution.model.clone()),
                    model_provider: Some(choice.provider.clone()),
                    ..ThreadSettingsUpdateParams::default()
                },
            )
            .await?;
        let old_model = model_label(old_model.as_deref());
        let suffix = if applied_to_thread {
            "Updated the current thread for subsequent turns."
        } else {
            "Saved for the next thread."
        };
        // Only claim a provider change when one actually happened.
        let headline = if choice.changed {
            format!(
                "Model changed: {old_model} ({old_provider}) -> {} ({}).",
                resolution.model, choice.provider
            )
        } else {
            format!(
                "Model changed: {old_model} -> {}.\nProvider unchanged: {}.",
                resolution.model, choice.provider
            )
        };
        self.send_text(chat_id, &format!("{headline}\n{suffix}"))
            .await
    }

    pub(super) async fn handle_approvals(
        &mut self,
        chat_id: ChatId,
        args: String,
    ) -> anyhow::Result<()> {
        let arg = args.trim();
        if arg.is_empty() {
            let policy = self.active_approval_policy(chat_id).await;
            self.send_text(
                chat_id,
                &format!(
                    "Approval policy: {}\nAvailable: untrusted, on-failure, on-request, never",
                    approval_policy_name(policy)
                ),
            )
            .await?;
            return Ok(());
        }

        let Some(new_policy) = parse_approval_policy(arg) else {
            self.send_text(
                chat_id,
                "Usage: /approvals [untrusted|on-failure|on-request|never]",
            )
            .await?;
            return Ok(());
        };

        let old_policy = self.active_approval_policy(chat_id).await;
        self.sessions
            .set_approval_policy(chat_id, new_policy)
            .await?;
        let applied_to_thread = self
            .apply_thread_settings_update_if_thread_loaded(
                chat_id,
                ThreadSettingsUpdateParams {
                    approval_policy: Some(new_policy),
                    ..ThreadSettingsUpdateParams::default()
                },
            )
            .await?;
        let suffix = if applied_to_thread {
            "Updated the current thread for subsequent turns."
        } else {
            "Saved for the next thread."
        };
        self.send_text(
            chat_id,
            &format!(
                "Approval policy changed: {} -> {}.\n{suffix}",
                approval_policy_name(old_policy),
                approval_policy_name(new_policy)
            ),
        )
        .await
    }

    pub(super) async fn compact_thread(&mut self, chat_id: ChatId) -> anyhow::Result<()> {
        let Some(thread_id) = self.sessions.thread_id(chat_id).await else {
            self.send_text(chat_id, "No active thread to compact.")
                .await?;
            return Ok(());
        };
        if !self.sessions.thread_loaded(chat_id).await {
            self.resume_thread(chat_id, thread_id.clone()).await?;
        }
        let request_id = self.request_ids.next();
        let _: ThreadCompactStartResponse = self
            .request_typed(
                ClientRequest::ThreadCompactStart {
                    request_id,
                    params: ThreadCompactStartParams { thread_id },
                },
                "thread/compact/start",
            )
            .await?;
        self.send_text(chat_id, "Compaction requested.").await
    }

    pub(super) async fn send_diff(&mut self, chat_id: ChatId) -> anyhow::Result<()> {
        let request_id = self.request_ids.next();
        let response: GitDiffToRemoteResponse = self
            .request_typed(
                ClientRequest::GitDiffToRemote {
                    request_id,
                    params: GitDiffToRemoteParams {
                        cwd: self.config.cwd.to_path_buf(),
                    },
                },
                "gitDiffToRemote",
            )
            .await?;
        if response.diff.trim().is_empty() {
            self.send_text(chat_id, "No diff to remote.").await?;
            return Ok(());
        }
        self.send_text(chat_id, &format!("```diff\n{}\n```", response.diff))
            .await
    }

    pub(super) async fn list_skills(&mut self, chat_id: ChatId) -> anyhow::Result<()> {
        let request_id = self.request_ids.next();
        let response: SkillsListResponse = self
            .request_typed(
                ClientRequest::SkillsList {
                    request_id,
                    params: SkillsListParams {
                        cwds: Vec::new(),
                        force_reload: false,
                    },
                },
                "skills/list",
            )
            .await?;
        let names = response
            .data
            .into_iter()
            .flat_map(|entry| entry.skills)
            .map(|skill| skill.name)
            .collect::<BTreeSet<_>>();
        if names.is_empty() {
            self.send_text(chat_id, "No skills discovered.").await?;
            return Ok(());
        }
        let mut text = String::from("Skills:");
        for name in names {
            text.push_str("\n- ");
            text.push_str(&name);
        }
        self.send_text(chat_id, &text).await
    }

    pub(super) async fn active_model_settings(&self, chat_id: ChatId) -> (Option<String>, String) {
        let model = self
            .sessions
            .model(chat_id)
            .await
            .or_else(|| self.config.model.clone());
        let provider = self
            .sessions
            .model_provider(chat_id)
            .await
            .unwrap_or_else(|| self.config.model_provider_id.clone());
        let provider = model
            .as_deref()
            .map(|model| provider_for_model(model, &provider).provider)
            .unwrap_or(provider);
        (model, provider)
    }

    pub(super) async fn active_approval_policy(&self, chat_id: ChatId) -> AskForApproval {
        self.sessions
            .approval_policy(chat_id)
            .await
            .unwrap_or_else(|| self.config.permissions.approval_policy.value().into())
    }

    async fn list_models(&mut self) -> anyhow::Result<ModelListResponse> {
        let request_id = self.request_ids.next();
        self.request_typed(
            ClientRequest::ModelList {
                request_id,
                params: ModelListParams {
                    cursor: None,
                    limit: None,
                    include_hidden: Some(true),
                },
            },
            "model/list",
        )
        .await
    }

    async fn apply_thread_settings_update_if_thread_loaded(
        &mut self,
        chat_id: ChatId,
        mut params: ThreadSettingsUpdateParams,
    ) -> anyhow::Result<bool> {
        let Some(thread_id) = self.sessions.thread_id(chat_id).await else {
            return Ok(false);
        };
        if !self.sessions.thread_loaded(chat_id).await {
            self.resume_thread(chat_id, thread_id.clone()).await?;
        }
        params.thread_id = thread_id;
        let request_id = self.request_ids.next();
        let _: ThreadSettingsUpdateResponse = self
            .request_typed(
                ClientRequest::ThreadSettingsUpdate { request_id, params },
                "thread/settings/update",
            )
            .await?;
        Ok(true)
    }
}

fn catalog_models(models: &[Model]) -> Vec<CatalogModel> {
    models
        .iter()
        .map(|model| CatalogModel {
            id: model.id.clone(),
            model: model.model.clone(),
            display_name: model.display_name.clone(),
        })
        .collect()
}

fn render_model_list(
    active_model: Option<&str>,
    active_provider: &str,
    catalog: &[CatalogModel],
) -> String {
    let active_model_label = model_label(active_model);
    let active_source = active_model.and_then(|model| known_model_source(model, catalog));
    let mut text = format!("Active model: {active_model_label}\nProvider: {active_provider}");
    match (active_model, active_source) {
        (Some(_), Some(source)) => text.push_str(&format!("\nSource: {source}")),
        (Some(_), None) => text.push_str("\nSource: pass-through; not in catalog or alias table"),
        (None, _) => {}
    }
    text.push_str("\n\nAvailable models:");
    for model in available_models(catalog) {
        let marker = if active_model == Some(model.model.as_str()) {
            "*"
        } else {
            "-"
        };
        let source = if model.in_catalog { "catalog" } else { "alias" };
        let aliases = if model.aliases.is_empty() {
            String::new()
        } else {
            format!(" aliases: {}", model.aliases.join(", "))
        };
        text.push_str(&format!(
            "\n{marker} {} ({}) [{source}{aliases}]",
            model.display_name, model.model
        ));
    }
    text
}

fn model_label(model: Option<&str>) -> &str {
    model.unwrap_or("server default")
}

fn parse_approval_policy(arg: &str) -> Option<AskForApproval> {
    match arg.trim().to_ascii_lowercase().as_str() {
        "untrusted" => Some(AskForApproval::UnlessTrusted),
        "on-failure" => Some(AskForApproval::OnFailure),
        "on-request" => Some(AskForApproval::OnRequest),
        "never" => Some(AskForApproval::Never),
        _ => None,
    }
}

fn approval_policy_name(policy: AskForApproval) -> &'static str {
    match policy {
        AskForApproval::UnlessTrusted => "untrusted",
        AskForApproval::OnFailure => "on-failure",
        AskForApproval::OnRequest => "on-request",
        AskForApproval::Granular { .. } => "granular",
        AskForApproval::Never => "never",
    }
}
