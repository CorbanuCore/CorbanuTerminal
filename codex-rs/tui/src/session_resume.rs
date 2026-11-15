//! Resolve saved-session state needed before resuming or forking a thread.
//!
//! The app-server API owns thread metadata. This module coordinates the TUI-specific
//! cwd prompt using the cwd reported by the selected thread and checks Guardian
//! eligibility for interactive CLI resume.

use std::path::Path;
use std::path::PathBuf;

use crate::app_server_session::AppServerSession;
use crate::cwd_prompt;
use crate::cwd_prompt::CwdPromptAction;
use crate::cwd_prompt::CwdPromptOutcome;
use crate::legacy_core::config::Config;
use crate::tui::Tui;
use codex_config::types::ResumeCwdMode;
use codex_protocol::ThreadId;
use codex_rollout::open_rollout_line_reader;
use codex_utils_path as path_utils;
use serde::Deserialize;
use serde_json::Value;
use std::io;

#[derive(Default)]
struct RolloutResumeState {
    provider_account: Option<RecordedProviderAccount>,
}

/// PF-84: the named account a thread ran on, from its newest turn context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecordedProviderAccount {
    pub(crate) provider_id: String,
    pub(crate) account: String,
}

impl RecordedProviderAccount {
    /// `<provider>:<account>`, the form `ConfigOverrides` and the app server use.
    pub(crate) fn selection(&self) -> String {
        format!("{}:{}", self.provider_id, self.account)
    }
}

#[derive(Deserialize)]
struct TurnContextAccountState {
    #[serde(default)]
    model_provider: Option<String>,
    #[serde(default)]
    provider_account: Option<String>,
}

#[derive(Deserialize)]
struct RawRecord {
    #[serde(rename = "type")]
    item_type: String,
    payload: Option<Value>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ResolveCwdOutcome {
    Continue(Option<PathBuf>),
    /// An interactive prompt was shown, so previously cached authentication may be stale.
    ContinueAfterPrompt(PathBuf),
    Exit,
}

pub(crate) struct ResumeCwdContext<'path> {
    pub(crate) current_cwd: &'path Path,
    pub(crate) remembered_current_cwd: &'path Path,
    pub(crate) allow_remember_current: bool,
    pub(crate) mode: Option<ResumeCwdMode>,
}

pub(crate) fn effective_resume_cwd_mode(
    configured_mode: Option<ResumeCwdMode>,
    cwd_override: Option<&Path>,
) -> Option<ResumeCwdMode> {
    if cwd_override.is_some() {
        Some(ResumeCwdMode::Current)
    } else {
        configured_mode
    }
}

pub(crate) async fn validate_resume_target(
    app_server: &mut AppServerSession,
    thread_id: ThreadId,
) -> color_eyre::Result<()> {
    let Ok(thread) = app_server
        .thread_read(thread_id, /*include_turns*/ false)
        .await
    else {
        // Preserve existing resume behavior when this metadata check is unavailable.
        return Ok(());
    };
    if codex_state::is_guardian_review_source(&thread.source.into()) {
        let parent_hint = thread
            .parent_thread_id
            .map_or_else(String::new, |parent_id| {
                format!("\nResume its parent session instead: codex resume {parent_id}")
            });
        color_eyre::eyre::bail!("Cannot resume a Guardian review thread.{parent_hint}");
    }
    Ok(())
}

pub(crate) async fn read_session_cwd(
    app_server: &mut AppServerSession,
    thread_id: ThreadId,
) -> Option<PathBuf> {
    match app_server
        .thread_read(thread_id, /*include_turns*/ false)
        .await
    {
        Ok(thread) => Some(thread.cwd.to_path_buf()),
        Err(err) => {
            tracing::warn!(%thread_id, %err, "Failed to read session cwd from app server");
            None
        }
    }
}

/// PF-84 (#545): the account the thread at `path` ran on, read the same way
/// the app server picks it on resume (the newest turn context only), so the
/// TUI can check it before deciding whether provider onboarding is needed.
pub(crate) async fn read_recorded_provider_account(path: &Path) -> Option<RecordedProviderAccount> {
    read_rollout_resume_state(path)
        .await
        .ok()
        .and_then(|state| state.provider_account)
}

pub(crate) async fn resolve_cwd_for_resume_or_fork(
    tui: &mut Tui,
    config: &Config,
    history_cwd: Option<PathBuf>,
    action: CwdPromptAction,
    cwd_context: ResumeCwdContext<'_>,
) -> color_eyre::Result<ResolveCwdOutcome> {
    if matches!(cwd_context.mode, Some(ResumeCwdMode::Current)) {
        return Ok(ResolveCwdOutcome::Continue(Some(
            cwd_context.remembered_current_cwd.to_path_buf(),
        )));
    }
    let Some(history_cwd) = history_cwd else {
        if matches!(cwd_context.mode, Some(ResumeCwdMode::Session)) {
            color_eyre::eyre::bail!(
                "failed to determine the working directory recorded for the selected session"
            );
        }
        return Ok(ResolveCwdOutcome::Continue(None));
    };
    match cwd_context.mode {
        Some(ResumeCwdMode::Session) => {
            return Ok(ResolveCwdOutcome::Continue(Some(history_cwd)));
        }
        Some(ResumeCwdMode::Current) | None => {}
    }
    if cwds_differ(cwd_context.current_cwd, &history_cwd) {
        // The prompt reads terminal input through the shared event broker.
        let mut events = tui.event_stream();
        let selection_outcome = cwd_prompt::run_cwd_selection_prompt(
            tui,
            config,
            action,
            cwd_context.current_cwd,
            &history_cwd,
            cwd_context.remembered_current_cwd,
            cwd_context.allow_remember_current,
            &mut events,
        )
        .await?;
        return Ok(match selection_outcome {
            CwdPromptOutcome::Selection(selection) => ResolveCwdOutcome::ContinueAfterPrompt(
                selection
                    .selected_cwd(
                        cwd_context.current_cwd,
                        &history_cwd,
                        cwd_context.remembered_current_cwd,
                    )
                    .to_path_buf(),
            ),
            CwdPromptOutcome::Exit => ResolveCwdOutcome::Exit,
        });
    }
    Ok(ResolveCwdOutcome::Continue(Some(history_cwd)))
}

pub(crate) fn cwds_differ(current_cwd: &Path, session_cwd: &Path) -> bool {
    !path_utils::paths_match_after_normalization(current_cwd, session_cwd)
}

/// PF-84 (#545): the newest turn context's provider account in the rollout at
/// `path`. Upstream reads the session cwd from the app server, so only the
/// account is read here, the same way the app server picks it on resume.
async fn read_rollout_resume_state(path: &Path) -> io::Result<RolloutResumeState> {
    let mut reader = open_rollout_line_reader(path).await?;
    let mut state = RolloutResumeState::default();
    let mut saw_record = false;

    while let Some(line) = reader.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(record) = serde_json::from_str::<RawRecord>(trimmed) else {
            continue;
        };
        saw_record = true;
        let Some(payload) = record.payload else {
            continue;
        };
        if record.item_type == "turn_context" {
            state.provider_account = serde_json::from_value::<TurnContextAccountState>(payload)
                .ok()
                .and_then(|turn_context| {
                    Some(RecordedProviderAccount {
                        provider_id: turn_context.model_provider?,
                        account: turn_context.provider_account?,
                    })
                });
        }
    }

    if saw_record {
        Ok(state)
    } else {
        Err(io::Error::other(format!(
            "rollout at {} is empty",
            path.display()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    fn rollout_line(
        timestamp: &str,
        item_type: &str,
        payload: serde_json::Value,
    ) -> serde_json::Value {
        serde_json::json!({
            "timestamp": timestamp,
            "type": item_type,
            "payload": payload,
        })
    }

    fn write_rollout_lines(path: &Path, lines: &[serde_json::Value]) -> std::io::Result<()> {
        let mut text = String::new();
        for line in lines {
            text.push_str(&serde_json::to_string(line).expect("serialize rollout"));
            text.push('\n');
        }
        std::fs::write(path, text)
    }

    #[tokio::test]
    async fn recorded_provider_account_is_the_newest_turn_contexts() -> std::io::Result<()> {
        let temp_dir = TempDir::new()?;
        let cwd = temp_dir.path().join("session");
        let turn = |account: Option<&str>| {
            let mut payload = serde_json::json!({
                "cwd": cwd.clone(),
                "model": "glm-5.3-flash",
                "model_provider": "zai",
            });
            if let Some(account) = account {
                payload["provider_account"] = serde_json::json!(account);
            }
            rollout_line("t", "turn_context", payload)
        };
        let rollout_path = temp_dir.path().join("rollout.jsonl");

        write_rollout_lines(&rollout_path, &[turn(Some("personal")), turn(Some("work"))])?;
        assert_eq!(
            read_recorded_provider_account(&rollout_path).await,
            Some(RecordedProviderAccount {
                provider_id: "zai".to_string(),
                account: "work".to_string(),
            })
        );

        // Like the app server, an older turn's account does not apply when the
        // newest turn recorded none (named accounts were off).
        write_rollout_lines(&rollout_path, &[turn(Some("work")), turn(None)])?;
        assert_eq!(read_recorded_provider_account(&rollout_path).await, None);
        Ok(())
    }

    #[tokio::test]
    async fn configured_resume_cwd_skips_prompt() -> color_eyre::Result<()> {
        let temp_dir = TempDir::new()?;
        let session_cwd = temp_dir.path().join("session");
        let config = crate::legacy_core::config::ConfigBuilder::default()
            .codex_home(temp_dir.path().to_path_buf())
            .build()
            .await?;
        let current_cwd = config.cwd.to_path_buf();
        let mut tui = crate::tui::test_support::make_test_tui()?;

        for (cwd_mode, expected_cwd) in [
            (ResumeCwdMode::Current, current_cwd.clone()),
            (ResumeCwdMode::Session, session_cwd.clone()),
        ] {
            let outcome = resolve_cwd_for_resume_or_fork(
                &mut tui,
                &config,
                Some(session_cwd.clone()),
                CwdPromptAction::Fork,
                ResumeCwdContext {
                    current_cwd: &current_cwd,
                    remembered_current_cwd: &current_cwd,
                    allow_remember_current: true,
                    mode: Some(cwd_mode),
                },
            )
            .await?;

            assert_eq!(outcome, ResolveCwdOutcome::Continue(Some(expected_cwd)));
        }
        Ok(())
    }

    #[tokio::test]
    async fn matching_resume_cwd_skips_prompt_without_configured_mode() -> color_eyre::Result<()> {
        let temp_dir = TempDir::new()?;
        let config = crate::legacy_core::config::ConfigBuilder::default()
            .codex_home(temp_dir.path().to_path_buf())
            .build()
            .await?;
        let current_cwd = config.cwd.to_path_buf();
        let mut tui = crate::tui::test_support::make_test_tui()?;

        let outcome = resolve_cwd_for_resume_or_fork(
            &mut tui,
            &config,
            Some(current_cwd.clone()),
            CwdPromptAction::Resume,
            ResumeCwdContext {
                current_cwd: &current_cwd,
                remembered_current_cwd: &current_cwd,
                allow_remember_current: true,
                mode: None,
            },
        )
        .await?;

        assert_eq!(outcome, ResolveCwdOutcome::Continue(Some(current_cwd)));
        Ok(())
    }

    #[tokio::test]
    async fn configured_session_cwd_rejects_missing_metadata() -> color_eyre::Result<()> {
        let temp_dir = TempDir::new()?;
        let config = crate::legacy_core::config::ConfigBuilder::default()
            .codex_home(temp_dir.path().to_path_buf())
            .build()
            .await?;
        let current_cwd = config.cwd.to_path_buf();
        let mut tui = crate::tui::test_support::make_test_tui()?;

        let error = resolve_cwd_for_resume_or_fork(
            &mut tui,
            &config,
            /*history_cwd*/ None,
            CwdPromptAction::Resume,
            ResumeCwdContext {
                current_cwd: &current_cwd,
                remembered_current_cwd: &current_cwd,
                allow_remember_current: true,
                mode: Some(ResumeCwdMode::Session),
            },
        )
        .await
        .expect_err("session mode should reject unavailable metadata");

        assert_eq!(
            error.to_string(),
            "failed to determine the working directory recorded for the selected session"
        );
        Ok(())
    }
}
