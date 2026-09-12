//! Campaign Tracker's Task Node views and typed TUI observation adapter.
use super::*;
use codex_tasknode_session::ActiveSession;
use codex_tasknode_session::Client;
use codex_tasknode_session::tracker::TrackerStore;
use codex_tasknode_session::tracker::{self};
use codex_vault::Vault;
use serde_json::Value;
use serde_json::json;
use uuid::Uuid;

const VIEW: &str = "campaign-tracker";
#[cfg(test)]
#[path = "campaign_tracker_tests.rs"]
mod tests;
pub(crate) fn start(tx: &AppEventSender) {
    let tx = tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            tx.send(AppEvent::CampaignTrackerTick);
        }
    });
}
pub(super) fn item(
    name: &str,
    description: &str,
    path: String,
    body: Option<Value>,
) -> SelectionItem {
    SelectionItem {
        name: name.to_string(),
        description: Some(description.to_string()),
        actions: vec![Box::new(move |tx| {
            tx.send(AppEvent::CampaignTrackerOpen {
                path: path.clone(),
                body: body.clone(),
            })
        })],
        dismiss_on_select: false,
        ..Default::default()
    }
}
impl ChatWidget {
    pub(super) fn tracker_selection(&mut self, params: SelectionViewParams) {
        if self.bottom_pane.active_view_id() == Some(VIEW) {
            self.bottom_pane
                .replace_selection_view_if_active(VIEW, params);
        } else {
            self.show_selection_view(params);
        }
    }
    fn tracker_session(&self) -> Result<(ActiveSession, TrackerStore, Client), String> {
        let home = self.config.codex_home.as_path();
        let scope = tasknode_menu::tasknode_session_scope(&self.config);
        let vault = self
            .campaign_tracker_vault
            .get_or_init(|| Vault::new(home.to_path_buf()));
        let session = codex_tasknode_session::load_scoped(vault, &scope)
            .map_err(|e| e.to_string())?
            .active
            .filter(|s| !s.is_expired())
            .ok_or("Link Task Node in this profile first.")?;
        let requested = std::env::var("PFT_TASKNODE_ORIGIN")
            .or_else(|_| std::env::var("TASKNODE_ORIGIN"))
            .unwrap_or_else(|_| session.origin.clone());
        let client = Client::for_session(&session, &requested).map_err(|e| e.to_string())?;
        let cache_key =
            serde_json::to_string(&(home, scope.profile(), &session.account_id, &session.origin))
                .map_err(|e| e.to_string())?;
        let mut stores = self.campaign_tracker_stores.borrow_mut();
        let store = if let Some(store) = stores.get(&cache_key) {
            store.clone()
        } else {
            let store = TrackerStore::new(home, &scope, &session)?;
            stores.insert(cache_key, store.clone());
            store
        };
        Ok((session, store, client))
    }
    fn tracker_api_key(&self) -> String {
        if let Some(key) = codex_model_provider_info::corbanu_api_key_from_env() {
            return key;
        }
        codex_login::provider_api_key_from_auth_storage(
            self.config.codex_home.as_path(),
            codex_model_provider_info::PFTERMINAL_PLAN_API_KEY_ENV_VAR,
            self.config.cli_auth_credentials_store_mode,
            self.config.auth_keyring_backend_kind(),
        )
        .ok()
        .flatten()
        .unwrap_or_default()
    }
    pub(crate) fn open_campaign_tracker(&mut self, path: String, mut body: Option<Value>) {
        if self.tracker_local_workflow(&path, body.as_ref()) {
            return;
        }
        if path == "/sync" {
            self.campaign_tracker_sync();
            return;
        }
        let mut recovery_body = body.clone();
        if let Some(Value::Object(fields)) = &mut recovery_body {
            fields.remove("apiKey");
        }
        let enrollment = if path == "/enrollment" {
            body.as_ref().and_then(|b| b["enabled"].as_bool())
        } else {
            None
        };
        if enrollment == Some(true)
            && let Some(body) = &mut body
        {
            body["apiKey"] = json!(self.tracker_api_key());
        }
        self.tracker_selection(SelectionViewParams {
            view_id: Some(VIEW),
            title: Some("Campaign Tracker".to_string()),
            subtitle: Some("Loading authorized activity…".to_string()),
            items: vec![],
            ..Default::default()
        });
        let route = format!("{}{path}", tracker::API);
        self.spawn_tasknode_value_request(
            "campaign-tracker",
            move |client| client.tracker_request(&route, body.as_ref()),
            move |result| AppEvent::CampaignTrackerResult {
                path,
                enrollment,
                body: recovery_body,
                result,
            },
        );
    }
    pub(crate) fn campaign_tracker_result(
        &mut self,
        path: String,
        enrollment: Option<bool>,
        body: Option<Value>,
        result: Result<Value, String>,
    ) {
        let data = match result {
            Ok(value) => value,
            Err(error) => {
                self.tracker_request_failed(&path, body, error);
                return;
            }
        };
        if data["grantId"].is_string()
            || data["campaignId"].is_string()
            || data["annotationId"].is_string()
            || path == "/revoke"
            || path == "/delete"
        {
            self.add_info_message("Campaign Tracker change saved.".to_string(), None);
            let next = if data["grantId"].is_string() || path == "/revoke" {
                "/grants"
            } else if data["campaignId"].is_string() {
                "/campaigns"
            } else if path == "/delete" {
                "/activity"
            } else {
                "/status"
            };
            self.open_campaign_tracker(next.to_string(), None);
            return;
        }
        if let Some(enabled) = enrollment {
            match self.tracker_session().and_then(|(_,store,_)|store.enroll(&tracker::workspace_id(self.config.cwd.as_path()),enabled)) {
                Ok(())=>self.add_info_message(if enabled {"Campaign Tracker recording enabled for this workspace. Prompts are preserved; sharing needs an explicit history grant."}else{"Campaign Tracker recording paused for this workspace."}.to_string(),None),
                Err(error)=>self.add_error_message(error),
            }
            self.campaign_tracker_sync();
            self.open_campaign_tracker("/status".to_string(), None);
            return;
        }
        let mut items = vec![
            item(
                "Activity totals",
                "Observed prompts, runs, actions and verified task outcomes",
                "/metrics".to_string(),
                None,
            ),
            item(
                "My activity",
                "Preserved prompts, compact summaries and observed actions",
                "/activity".to_string(),
                None,
            ),
            item(
                "Access & replay",
                "View explicit history grants and collaborators",
                "/grants".to_string(),
                None,
            ),
            item(
                "Access audit",
                "See who read, exported or changed your history",
                "/audit".to_string(),
                None,
            ),
        ];
        items.push(item(
            "Direct reports & collaborators",
            "Choose a Task Node handle to share or inspect activity",
            "/team".to_string(),
            None,
        ));
        items.push(item(
            "Campaigns",
            "Group work around a named objective",
            "/campaigns".to_string(),
            None,
        ));
        let mut subtitle =
            "Prompts + summaries · permissioned history · observed TUI activity".to_string();
        if path == "/status" {
            let workspace = tracker::workspace_id(self.config.cwd.as_path());
            if let Some(enrollments) = data["enrollments"].as_array() {
                let enabled = enrollments
                    .iter()
                    .any(|e| e["workspace_id"] == workspace && e["enabled"] == true);
                if let Ok((_, store, _)) = self.tracker_session() {
                    let _ = store.enroll(&workspace, enabled);
                }
            }
            let local = self
                .tracker_session()
                .and_then(|(_, store, _)| store.state())
                .ok();
            let enabled = local
                .as_ref()
                .is_some_and(|s| s.workspaces.contains(&workspace));
            let pending = local.as_ref().map_or(0, |s| s.pending.len());
            let gaps = local.as_ref().map_or(0, |s| s.gap_count);
            subtitle = format!(
                "@{} · {} events · {pending} pending · {gaps} capture gaps · recording {}",
                data["handle"].as_str().unwrap_or("unassigned"),
                data["usage"]["events"],
                if enabled { "ON" } else { "OFF" }
            );
            if let Some(error) = local.and_then(|s| s.last_error) {
                subtitle.push_str(&format!(" · {error}"));
            }
            items.push(item(
                if enabled {
                    "Pause this workspace"
                } else {
                    "Enable recording for this workspace"
                },
                "Preserve prompts and summarized outputs; no sharing until you grant access",
                "/enrollment".to_string(),
                Some(json!({"workspaceId":workspace,"enabled":!enabled})),
            ));
            items.push(item(
                "Sync now",
                "Retry the encrypted local outbox",
                "/sync".to_string(),
                None,
            ));
        } else if path.starts_with("/metrics") {
            subtitle = data["coverage"]
                .as_str()
                .unwrap_or("Observed activity")
                .to_string();
            items.insert(0, item("Read activity totals", if data["complete"] == true { "All visible records included" } else { "Partial totals: scan limit reached" }, "/_document".to_string(), Some(json!({"text": format!("Activity totals\n\n{}\n\n{}\nComplete: {}", serde_json::to_string_pretty(&data["metrics"]).unwrap_or_default(), subtitle, data["complete"])}))));
        } else if path == "/team" {
            for member in data["members"].as_array().into_iter().flatten() {
                let handle = member["identity"]["hiveHandle"].as_str().unwrap_or("");
                let relationship = member["relationship"].as_str().unwrap_or("collaborator");
                items.push(item(
                    &format!("Share with @{handle}"),
                    relationship,
                    "/_share".to_string(),
                    Some(json!({"handle":handle})),
                ));
                items.push(item(
                    &format!("View @{handle}"),
                    "Requires this person's explicit Campaign Tracker grant",
                    format!(
                        "/activity?accountId={}",
                        urlencoding::encode(member["accountId"].as_str().unwrap_or(""))
                    ),
                    None,
                ));
            }
        } else if path == "/campaigns" {
            items.push(item(
                "Create campaign",
                "Name the outcome you are working toward",
                "/_new_campaign".to_string(),
                None,
            ));
            for campaign in data["items"].as_array().into_iter().flatten() {
                items.push(item(campaign["title"].as_str().unwrap_or("Campaign"),campaign["objective"].as_str().unwrap_or(""),"/_document".to_string(),Some(json!({"text":format!("{}\n\n{}\n\nMembers: {}\nTasks: {}\nMembership does not grant prompt access.",campaign["title"],campaign["objective"],campaign["members"],campaign["taskIds"])}))));
            }
        } else if path == "/grants" {
            for grant in data["items"].as_array().into_iter().flatten() {
                let subject = grant["subject_account_id"].as_str().unwrap_or("");
                let owner = self
                    .tracker_session()
                    .ok()
                    .and_then(|(s, _, _)| s.account_id)
                    .is_some_and(|a| a == subject);
                let handle = grant[if owner {
                    "viewer_handle"
                } else {
                    "subject_handle"
                }]
                .as_str()
                .unwrap_or("member");
                if !grant["revoked_at"].is_null()
                    || grant["expires_at"]
                        .as_str()
                        .and_then(|date| chrono::DateTime::parse_from_rfc3339(date).ok())
                        .is_some_and(|date| date < chrono::Utc::now())
                {
                    items.push(SelectionItem {
                        name: format!("Inactive grant · @{handle}"),
                        description: Some(
                            "This grant no longer authorizes history access.".to_string(),
                        ),
                        ..Default::default()
                    });
                } else if owner {
                    items.push(item(
                        &format!("Revoke @{handle}"),
                        "Stop subsequent reads and replay immediately",
                        "/revoke".to_string(),
                        Some(json!({"grantId":grant["grant_id"]})),
                    ));
                } else {
                    items.push(item(
                        &format!("Replay @{handle}"),
                        "Only the explicitly granted history and fields are visible",
                        format!("/activity?accountId={}", urlencoding::encode(subject)),
                        None,
                    ));
                }
            }
        } else if path.starts_with("/activity") || path.starts_with("/replay") {
            items.clear();
            let records = data["items"].as_array().cloned().unwrap_or_default();
            let human = records
                .iter()
                .filter(|r| r["kind"] == "human_prompt")
                .count();
            subtitle = format!(
                "{} visible actions · {human} human prompts on this page · agent runtime is not human working time",
                records.len()
            );
            items.push(item(
                "Search activity",
                "Search only fields you are allowed to read",
                "/_search".to_string(),
                Some(json!({"path":path})),
            ));
            for record in &records {
                let kind = record["kind"].as_str().unwrap_or("action");
                let title = record["summary"]["title"].as_str().unwrap_or(kind);
                let content = if record["promptWithheld"] == true {
                    "Prompt access not granted"
                } else {
                    record["content"]
                        .as_str()
                        .filter(|s| !s.is_empty())
                        .or_else(|| record["summary"]["summary"].as_str())
                        .unwrap_or("Summary pending or metadata-only action")
                };
                let account = record["accountId"].as_str().unwrap_or("");
                let session = record["sessionId"].as_str().unwrap_or("");
                let description = format!(
                    "{} · @{} · repo {} · /goal {}\n{}\n{}",
                    record["occurredAt"].as_str().unwrap_or(""),
                    record["handleAtExecution"].as_str().unwrap_or(""),
                    record["repository"]["label"].as_str().unwrap_or("unknown"),
                    record["goal"]["active"],
                    content,
                    record["facts"]
                );
                let mut entry = item(
                    title,
                    &description,
                    format!(
                        "/replay?accountId={}&sessionId={}",
                        urlencoding::encode(account),
                        urlencoding::encode(session)
                    ),
                    None,
                );
                entry.selected_description = Some(description.clone());
                items.push(entry);
                items.push(item(
                    "Read full action",
                    "Open the scrollable prompt and evidence view",
                    "/_document".to_string(),
                    Some(json!({"text": description})),
                ));
                if record["kind"] == "human_prompt"
                    && record["capabilities"].as_array().is_some_and(|caps| {
                        caps.contains(&json!("review")) && caps.contains(&json!("prompt"))
                    })
                {
                    items.push(item(
                        "Read prompt reviews",
                        "View versioned scores and mapping corrections",
                        format!(
                            "/annotations?accountId={}&id={}",
                            urlencoding::encode(account),
                            urlencoding::encode(record["id"].as_str().unwrap_or(""))
                        ),
                        None,
                    ));
                    if self
                        .tracker_session()
                        .ok()
                        .and_then(|(s, _, _)| s.account_id)
                        .as_deref()
                        == Some(account)
                    {
                        items.push(item("Correct task mapping", "Link your owned Task Node task IDs with a rationale", "/_mapping".to_string(), Some(json!({"accountId":account,"id":record["id"],"revision":record["revision"]}))));
                    }
                    items.push(item("Review this prompt", "Score five dimensions with a versioned human rationale", "/_review".to_string(), Some(json!({"accountId":account,"id":record["id"],"revision":record["revision"]}))));
                }
            }
            if let Some(cursor) = data.get("nextCursor").filter(|c| !c.is_null()) {
                let separator = if path.contains('?') { '&' } else { '?' };
                items.push(item(
                    "Older activity",
                    "Load the next page",
                    format!(
                        "{path}{separator}before={}&beforeId={}",
                        urlencoding::encode(cursor["before"].as_str().unwrap_or("")),
                        urlencoding::encode(cursor["beforeId"].as_str().unwrap_or(""))
                    ),
                    None,
                ));
            }
        } else if path.starts_with("/annotations") {
            items.insert(
                0,
                item(
                    "Read reviews and corrections",
                    "Versioned human assessments; missing context remains visible",
                    "/_document".to_string(),
                    Some(json!({"text":serde_json::to_string_pretty(&data).unwrap_or_default()})),
                ),
            );
        } else if path == "/audit" {
            items.clear();
            for record in data["items"].as_array().into_iter().flatten() {
                items.push(SelectionItem {
                    name: record["action"].as_str().unwrap_or("audit").to_string(),
                    description: Some(format!(
                        "{} · actor {} · {}",
                        record["occurred_at"], record["actor_account_id"], record["resource_id"]
                    )),
                    ..Default::default()
                });
            }
        } else {
            self.open_campaign_tracker("/status".to_string(), None);
            return;
        }
        items.push(item(
            "Tracker overview",
            "Return to status and recording controls",
            "/status".to_string(),
            None,
        ));
        self.tracker_selection(SelectionViewParams {
            view_id: Some(VIEW),
            title: Some("Campaign Tracker".to_string()),
            subtitle: Some(subtitle),
            items,
            is_searchable: true,
            allow_number_shortcuts: false,
            ..Default::default()
        });
    }
    pub(crate) fn campaign_tracker_sync(&mut self) {
        if self.campaign_tracker_syncing {
            return;
        }
        let Ok((_, store, client)) = self.tracker_session() else {
            self.campaign_tracker_indicator = None;
            return;
        };
        let Ok(state) = store.state() else {
            return;
        };
        let enabled = state
            .workspaces
            .contains(&tracker::workspace_id(self.config.cwd.as_path()));
        self.campaign_tracker_indicator = if enabled || !state.pending.is_empty() {
            Some(format!(
                "Tracker {} · {} pending{}",
                if enabled { "REC" } else { "paused" },
                state.pending.len(),
                if state.last_error.is_some() || state.gap_count > 0 {
                    " · gap/error"
                } else {
                    ""
                }
            ))
        } else {
            None
        };
        self.refresh_status_surfaces();
        if state.pending.is_empty() {
            return;
        }
        let key = self.tracker_api_key();
        if key.is_empty() {
            let _ = store.failure(
                "Corbanu subscription credential unavailable; activity remains encrypted locally.",
            );
            return;
        }
        self.campaign_tracker_syncing = true;
        let tx = self.app_event_tx.clone();
        let identity = client.identity();
        std::thread::spawn(move || {
            let result = store.sync(&client, &key);
            tx.send(AppEvent::CampaignTrackerSync { identity, result });
        });
    }
    pub(crate) fn campaign_tracker_sync_result(
        &mut self,
        identity: String,
        result: Result<usize, String>,
    ) {
        self.campaign_tracker_syncing = false;
        let progressed = result.as_ref().is_ok_and(|count| *count > 0);
        if !self
            .tracker_session()
            .is_ok_and(|(_, _, client)| client.identity() == identity)
        {
            return;
        }
        if self.bottom_pane.active_view_id() == Some(VIEW) {
            if let Err(error) = result {
                self.add_error_message(format!("Campaign Tracker: {error}"));
            }
            self.open_campaign_tracker("/status".to_string(), None);
        }
        if progressed {
            self.campaign_tracker_sync();
        }
    }
    pub(crate) fn tracker_capture_goal_prompt(&mut self, objective: &str) {
        self.campaign_tracker_turn = Some(Uuid::new_v4().to_string());
        self.tracker_capture(
            "human_prompt",
            &format!("/goal {objective}"),
            json!({"action":"goal_objective"}),
        );
    }
    pub(super) fn tracker_capture(&mut self, kind: &str, content: &str, facts: Value) {
        let cwd = self.config.cwd.to_path_buf();
        self.tracker_capture_at(kind, content, facts, &cwd);
    }
    fn tracker_capture_at(&mut self, kind: &str, content: &str, mut facts: Value, cwd: &Path) {
        let Ok((session, store, _)) = self.tracker_session() else {
            return;
        };
        let workspace = tracker::workspace_id(self.config.cwd.as_path());
        if !store
            .state()
            .is_ok_and(|s| s.workspaces.contains(&workspace))
        {
            return;
        }
        let mut secrets = vec![session.terminal_token, self.tracker_api_key()];
        secrets.extend(
            std::env::vars()
                .filter(|(k, _)| {
                    ["_KEY", "_TOKEN", "_SECRET", "_PASSWORD"]
                        .iter()
                        .any(|suffix| k.ends_with(suffix))
                })
                .map(|(_, v)| v),
        );
        let content = tracker::redact(content, &secrets);
        tracker::redact_value(&mut facts, &secrets);
        let session_id = self
            .thread_id()
            .map(|id| id.to_string())
            .unwrap_or_else(|| "unassigned".to_string());
        let turn = self
            .campaign_tracker_turn
            .get_or_insert_with(|| Uuid::new_v4().to_string())
            .clone();
        let repository = repository_metadata(cwd);
        let goal = self
            .current_goal_status
            .as_ref()
            .map(super::goal_status::GoalStatusState::tracker_metadata)
            .unwrap_or_else(|| json!({"active":false}));
        let chunks = if kind == "agent_output" {
            tracker::output_chunks(&content)
        } else {
            vec![content]
        };
        for content in chunks {
            let event = json!({"sessionId":session_id,"turnId":turn,"workspaceId":workspace,"kind":kind,"content":content,"facts":facts,"goal":goal,"repository":repository,"taskIds":[],"coverage":"observed_tui","model":self.config.model.clone().unwrap_or_default()});
            if let Err(error) = store.capture(event) {
                self.add_error_message(format!("Campaign Tracker: {error}"));
                break;
            }
        }
        self.campaign_tracker_sync();
    }
    pub(super) fn tracker_item(&mut self, item: &ThreadItem) {
        match item {
            ThreadItem::AgentMessage {text,..}=>self.tracker_capture("agent_output",text,json!({})),
            ThreadItem::CommandExecution {command,cwd,exit_code,duration_ms,status,..}=>self.tracker_capture_at("tool","",json!({"action":command.chars().take(200).collect::<String>(),"exitCode":exit_code,"durationMs":duration_ms,"status":format!("{status:?}")}),Path::new(cwd.as_str())),
            ThreadItem::FileChange {changes,status,..}=>self.tracker_capture("tool","",json!({"action":format!("file changes: {}",changes.len()),"status":format!("{status:?}")})),
            ThreadItem::McpToolCall {server,tool,status,duration_ms,..}=>self.tracker_capture("tool","",json!({"action":format!("{server}/{tool}"),"status":format!("{status:?}"),"durationMs":duration_ms})),
            ThreadItem::DynamicToolCall {tool,status,duration_ms,..}=>self.tracker_capture("tool","",json!({"action":tool,"status":format!("{status:?}"),"durationMs":duration_ms})),
            ThreadItem::SubAgentActivity {agent_thread_id,agent_path,kind,..}=>self.tracker_capture("tool","",json!({"action":format!("subagent {kind:?}"),"parentAgentId":agent_thread_id,"artifact":agent_path})),
            ThreadItem::CollabAgentToolCall {tool,status,sender_thread_id,..}=>self.tracker_capture("tool","",json!({"action":format!("{tool:?}"),"status":format!("{status:?}"),"parentAgentId":sender_thread_id})),
            _=>{},
        }
    }
}
fn repository_metadata(cwd: &Path) -> Value {
    fn git(cwd: &Path, args: &[&str]) -> Option<String> {
        let result = std::process::Command::new("git")
            .arg("-C")
            .arg(cwd)
            .args(args)
            .output()
            .ok()?;
        result
            .status
            .success()
            .then(|| String::from_utf8_lossy(&result.stdout).trim().to_string())
    }
    let label = cwd
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "workspace".to_string());
    let raw = git(cwd, &["remote", "get-url", "origin"]).unwrap_or_default();
    let remote = if let Some(path) = raw.strip_prefix("git@github.com:") {
        format!("https://github.com/{path}")
    } else if let Ok(mut url) = url::Url::parse(&raw) {
        let _ = url.set_username("");
        let _ = url.set_password(None);
        url.set_query(None);
        url.set_fragment(None);
        if url.scheme() == "https" {
            url.to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    json!({"id":tracker::workspace_id(cwd),"label":label,"remote":remote,"branch":git(cwd,&["branch","--show-current"]).unwrap_or_default(),"commit":git(cwd,&["rev-parse","HEAD"]).unwrap_or_default(),"dirty":git(cwd,&["status","--porcelain","--untracked-files=no"]).is_some_and(|s|!s.is_empty())})
}
