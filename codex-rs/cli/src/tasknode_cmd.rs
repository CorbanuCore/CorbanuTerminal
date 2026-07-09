use anyhow::Context;
use clap::Args;
use clap::Parser;
use clap::Subcommand;
use codex_core::config::ConfigBuilder;
use codex_tasknode_client::TaskNodeClient;
use codex_tasknode_client::TaskNodeClientError;
use codex_tasknode_client::TaskNodeLocalError;
use codex_tasknode_client::TaskNodeLocalSession;
use codex_tasknode_client::TaskNodeRawResponse as TaskNodeResponse;
use codex_tasknode_client::resolve_origin;
use codex_tasknode_client::tasknode_parse_sse_block;
use codex_tasknode_client::tasknode_sse_drain_blocks;
use codex_tasknode_client::tasknode_sse_drain_remainder;
use codex_utils_cli::CliConfigOverrides;
use serde_json::Map;
use serde_json::Value;
use serde_json::json;
use std::future::Ready;
use std::future::ready;
use std::io::IsTerminal;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use thiserror::Error;

#[derive(Debug, Parser)]
pub(crate) struct TaskNodeCli {
    #[clap(skip)]
    pub config_overrides: CliConfigOverrides,

    /// Emit JSON. This helper always emits JSON; the flag is accepted for scripts.
    #[arg(long, global = true, default_value_t = false)]
    pub json: bool,

    /// Override Task Node origin. Defaults to PFT_TASKNODE_ORIGIN, TASKNODE_ORIGIN, saved session origin, or production.
    #[arg(long, global = true)]
    pub origin: Option<String>,

    #[command(subcommand)]
    pub command: TaskNodeCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum TaskNodeCommand {
    /// Link this terminal session to Task Node through GitHub.
    Link(LinkArgs),

    /// Show linked account, wallet, server flags, and task counts.
    Status,

    /// Show linked-wallet PFT balance.
    Balance(BalanceArgs),

    /// Show recent rewarded tasks.
    Rewards(RewardsCli),

    /// Work with Task Node chat.
    Chat(ChatCli),

    /// Read or save the Task Node context document.
    Context(ContextCli),

    /// Create a new task request.
    Request(RequestCli),

    /// List or inspect active task-generation requests.
    Requests(RequestsCli),

    /// List Task Node tasks by tab.
    Tasks(TasksCli),

    /// Inspect or mutate one Task Node task.
    Task(TaskCli),

    /// Respond to verification requests.
    Verification(VerificationCli),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct LinkArgs {
    /// Poll an existing pending link request until it completes or times out.
    #[arg(long, default_value_t = false)]
    poll: bool,

    /// Print local Task Node link state without contacting Task Node.
    #[arg(long, default_value_t = false)]
    status: bool,

    /// Replace an existing linked or pending session with a new link request.
    #[arg(long, default_value_t = false)]
    relink: bool,

    /// Maximum seconds to wait with --poll.
    #[arg(long, default_value_t = 300)]
    timeout: u64,

    /// Do not open the verification URL in a browser.
    #[arg(long, default_value_t = false)]
    no_browser: bool,
}

#[derive(Debug, Args)]
pub(crate) struct BalanceArgs {
    /// Force a fresh balance lookup.
    #[arg(long, default_value_t = false)]
    force: bool,
}

#[derive(Debug, Args)]
pub(crate) struct RewardsCli {
    #[command(subcommand)]
    action: RewardsCommand,
}

#[derive(Debug, Subcommand)]
enum RewardsCommand {
    /// List recent rewards.
    List(LimitArgs),
}

#[derive(Debug, Args)]
pub(crate) struct ChatCli {
    #[command(subcommand)]
    action: ChatCommand,
}

#[derive(Debug, Subcommand)]
enum ChatCommand {
    /// List standard Task Node chat threads.
    #[clap(alias = "conversations")]
    List(LimitArgs),

    /// Read a chat thread.
    History(ChatHistoryArgs),

    /// Search chat threads.
    Search(ChatSearchArgs),

    /// Send a Private Thinking chat message.
    Send(ChatSendArgs),
}

#[derive(Debug, Args)]
struct ChatHistoryArgs {
    conversation_id: String,

    #[arg(long, default_value_t = 120)]
    limit: u16,
}

#[derive(Debug, Args)]
struct ChatSearchArgs {
    query: String,

    #[arg(long, default_value_t = 20)]
    limit: u8,
}

#[derive(Debug, Args)]
struct ChatSendArgs {
    /// Message text. Use --message-file for multiline prompts.
    #[arg(long)]
    message: Option<String>,

    /// Read message text from a file.
    #[arg(long, value_name = "PATH")]
    message_file: Option<PathBuf>,

    /// Existing conversation id. Omit to create a new terminal chat id.
    #[arg(long)]
    conversation_id: Option<String>,

    /// Chat mode. Defaults to Private Thinking.
    #[arg(long, default_value = "Private Thinking")]
    mode: String,

    /// Stream SSE events as JSON lines.
    #[arg(long, default_value_t = false)]
    stream: bool,

    /// Preflight through the backend without calling the model, when the server supports it.
    #[arg(long, default_value_t = false)]
    dry_run: bool,
}

#[derive(Debug, Args)]
pub(crate) struct ContextCli {
    #[command(subcommand)]
    action: ContextCommand,
}

#[derive(Debug, Subcommand)]
enum ContextCommand {
    /// Read the current context document.
    Get,

    /// Save a new context document body.
    Save(ContextSaveArgs),
}

#[derive(Debug, Args)]
struct ContextSaveArgs {
    /// Read context body from this file.
    #[arg(long, value_name = "PATH")]
    body_file: PathBuf,

    /// Current revision from `tasknode context get`.
    #[arg(long)]
    revision: u64,

    /// Optional document title.
    #[arg(long)]
    title: Option<String>,
}

#[derive(Debug, Args)]
pub(crate) struct RequestCli {
    #[command(subcommand)]
    action: RequestCommand,
}

#[derive(Debug, Subcommand)]
enum RequestCommand {
    /// Create a new personal task request.
    Create(RequestCreateArgs),
}

#[derive(Debug, Args)]
struct RequestCreateArgs {
    /// Task request text. Use --body-file for multiline requests.
    #[arg(long)]
    text: Option<String>,

    /// Read task request text from a file.
    #[arg(long, value_name = "PATH")]
    body_file: Option<PathBuf>,

    /// Task request kind.
    #[arg(long, default_value = "personal")]
    kind: String,

    /// Source title recorded in Task Node.
    #[arg(long, default_value = "PFTerminal JSON helper")]
    source_title: String,
}

#[derive(Debug, Args)]
pub(crate) struct RequestsCli {
    #[command(subcommand)]
    action: RequestsCommand,
}

#[derive(Debug, Subcommand)]
enum RequestsCommand {
    /// List active task-generation requests.
    List(LimitArgs),

    /// Show one task request.
    Show(RequestShowArgs),
}

#[derive(Debug, Args)]
struct RequestShowArgs {
    request_id: String,
}

#[derive(Debug, Args)]
pub(crate) struct TasksCli {
    #[command(subcommand)]
    action: TasksCommand,
}

#[derive(Debug, Subcommand)]
enum TasksCommand {
    /// List tasks in a tab.
    List(TasksListArgs),
}

#[derive(Debug, Args)]
struct TasksListArgs {
    /// Task tab: outstanding, verification, refused, rewarded, etc.
    #[arg(long, default_value = "outstanding")]
    tab: String,
}

#[derive(Debug, Args)]
pub(crate) struct TaskCli {
    #[command(subcommand)]
    action: TaskCommand,
}

#[derive(Debug, Subcommand)]
enum TaskCommand {
    /// Show one task, including terminal-rendered brief text.
    Show(TaskIdArgs),

    /// Accept one task.
    Accept(TaskIdArgs),

    /// Refuse one task.
    Refuse(TaskRefuseArgs),

    /// Cancel one accepted task.
    Cancel(TaskIdArgs),

    /// Submit initial evidence or follow-up evidence for one task.
    Evidence(TaskEvidenceArgs),
}

#[derive(Debug, Args)]
struct TaskIdArgs {
    task_id: String,
}

#[derive(Debug, Args)]
struct TaskRefuseArgs {
    task_id: String,

    /// Refusal reason text.
    #[arg(long)]
    reason: Option<String>,

    /// Read refusal reason from a file.
    #[arg(long, value_name = "PATH")]
    reason_file: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct TaskEvidenceArgs {
    task_id: String,

    /// Evidence summary text.
    #[arg(long)]
    summary: Option<String>,

    /// Read evidence summary from a file.
    #[arg(long, value_name = "PATH")]
    body_file: Option<PathBuf>,

    /// Additional artifact. Accepts a URL or type=value, repeatable.
    #[arg(long = "artifact")]
    artifacts: Vec<String>,
}

#[derive(Debug, Args)]
pub(crate) struct VerificationCli {
    #[command(subcommand)]
    action: VerificationCommand,
}

#[derive(Debug, Subcommand)]
enum VerificationCommand {
    /// Submit a verification response for one task.
    Respond(TaskEvidenceArgs),
}

#[derive(Debug, Args)]
struct LimitArgs {
    #[arg(long, default_value_t = 20)]
    limit: u8,
}

pub(crate) async fn run(command: TaskNodeCli) -> anyhow::Result<()> {
    let result = run_inner(command).await;
    match result {
        Ok(exit_code) => {
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
            Ok(())
        }
        Err(err) => {
            print_json(&json!({
                "ok": false,
                "error": "tasknode_helper_error",
                "message": err.to_string(),
            }))?;
            std::process::exit(1);
        }
    }
}

async fn run_inner(command: TaskNodeCli) -> anyhow::Result<i32> {
    let _json_flag = command.json;
    let TaskNodeCli {
        config_overrides,
        origin,
        json: _,
        command,
    } = command;

    if let TaskNodeCommand::Link(args) = command {
        return run_link_command(config_overrides, origin, args).await;
    }

    let client = match tasknode_client_from_cli(config_overrides, origin).await {
        Ok(client) => client,
        Err(err) => {
            if let Some(session_err) = err.downcast_ref::<TaskNodeCommandSessionError>() {
                return emit_session_error(session_err);
            }
            return Err(err);
        }
    };

    match command {
        TaskNodeCommand::Link(_) => unreachable!("link command is handled before client loading"),
        TaskNodeCommand::Status => {
            emit_response(client.get("/api/terminal/tasknode/status").await?)
        }
        TaskNodeCommand::Balance(args) => {
            let path = if args.force {
                "/api/terminal/tasknode/balance?force=1"
            } else {
                "/api/terminal/tasknode/balance"
            };
            emit_response(client.get(path).await?)
        }
        TaskNodeCommand::Rewards(cli) => match cli.action {
            RewardsCommand::List(args) => emit_response(
                client
                    .get(&format!(
                        "/api/terminal/tasknode/rewards?limit={}",
                        limit(args.limit, 1, 50)
                    ))
                    .await?,
            ),
        },
        TaskNodeCommand::Chat(cli) => run_chat_command(&client, cli).await,
        TaskNodeCommand::Context(cli) => run_context_command(&client, cli).await,
        TaskNodeCommand::Request(cli) => run_request_command(&client, cli).await,
        TaskNodeCommand::Requests(cli) => run_requests_command(&client, cli).await,
        TaskNodeCommand::Tasks(cli) => run_tasks_command(&client, cli).await,
        TaskNodeCommand::Task(cli) => run_task_command(&client, cli).await,
        TaskNodeCommand::Verification(cli) => run_verification_command(&client, cli).await,
    }
}

async fn run_chat_command(client: &TaskNodeClient, cli: ChatCli) -> anyhow::Result<i32> {
    match cli.action {
        ChatCommand::List(args) => emit_response(
            client
                .get(&format!(
                    "/api/terminal/tasknode/chat/conversations?limit={}",
                    limit(args.limit, 1, 50)
                ))
                .await?,
        ),
        ChatCommand::History(args) => emit_response(
            client
                .get(&format!(
                    "/api/terminal/tasknode/chat/history?conversationId={}&limit={}",
                    urlencoding::encode(&args.conversation_id),
                    limit_u16(args.limit, 1, 200)
                ))
                .await?,
        ),
        ChatCommand::Search(args) => emit_response(
            client
                .get(&format!(
                    "/api/terminal/tasknode/chat/search?q={}&limit={}",
                    urlencoding::encode(&args.query),
                    limit(args.limit, 1, 50)
                ))
                .await?,
        ),
        ChatCommand::Send(args) => {
            let message = read_text_input(args.message, args.message_file, "chat message")?;
            let conversation_id = args
                .conversation_id
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(new_chat_id);
            let body = json!({
                "conversationId": conversation_id,
                "message": message,
                "mode": args.mode,
                "dryRun": args.dry_run,
            });
            if args.stream {
                client
                    .post_sse_jsonl("/api/terminal/tasknode/chat/stream", &body)
                    .await
            } else {
                emit_response(
                    client
                        .post("/api/terminal/tasknode/chat/send", &body)
                        .await?,
                )
            }
        }
    }
}

async fn run_context_command(client: &TaskNodeClient, cli: ContextCli) -> anyhow::Result<i32> {
    match cli.action {
        ContextCommand::Get => emit_response(client.get("/api/terminal/tasknode/context").await?),
        ContextCommand::Save(args) => {
            let body_text = read_file_required(&args.body_file, "context body")?;
            let mut body = Map::new();
            body.insert("body".to_string(), Value::String(body_text));
            body.insert("revision".to_string(), Value::from(args.revision));
            body.insert(
                "source".to_string(),
                Value::String("pfterminal-cli".to_string()),
            );
            if let Some(title) = args.title.filter(|value| !value.trim().is_empty()) {
                body.insert("title".to_string(), Value::String(title));
            }
            emit_response(
                client
                    .post("/api/terminal/tasknode/context", &Value::Object(body))
                    .await?,
            )
        }
    }
}

async fn run_request_command(client: &TaskNodeClient, cli: RequestCli) -> anyhow::Result<i32> {
    match cli.action {
        RequestCommand::Create(args) => {
            let detail = read_text_input(args.text, args.body_file, "task request")?;
            let body = json!({
                "userDetailText": detail,
                "requestedTaskKind": args.kind,
                "source": "pfterminal-cli",
                "sourceConversationTitle": args.source_title,
                "idempotencyKey": idempotency_key("request"),
            });
            emit_response(
                client
                    .post("/api/terminal/tasknode/requests", &body)
                    .await?,
            )
        }
    }
}

async fn run_requests_command(client: &TaskNodeClient, cli: RequestsCli) -> anyhow::Result<i32> {
    match cli.action {
        RequestsCommand::List(args) => emit_response(
            client
                .get(&format!(
                    "/api/terminal/tasknode/requests?limit={}",
                    limit(args.limit, 1, 50)
                ))
                .await?,
        ),
        RequestsCommand::Show(args) => emit_response(
            client
                .get(&format!(
                    "/api/terminal/tasknode/requests/{}",
                    urlencoding::encode(&args.request_id)
                ))
                .await?,
        ),
    }
}

async fn run_tasks_command(client: &TaskNodeClient, cli: TasksCli) -> anyhow::Result<i32> {
    match cli.action {
        TasksCommand::List(args) => {
            let response = client
                .get(&format!(
                    "/api/terminal/tasknode/tasks?tab={}",
                    urlencoding::encode(&args.tab)
                ))
                .await?;
            if let Some(error) = unknown_tab_error(&args.tab, &response) {
                print_json(&error)?;
                return Ok(1);
            }
            emit_response(response)
        }
    }
}

/// The server answers *any* `tab` value with `{"ok":true,"tasks":[]}`, so a typo is
/// indistinguishable from "you have no tasks" — which an agent reads as a clean,
/// authoritative empty result and acts on. Reject a tab the server did not report in
/// `counts`, but only when it also returned nothing, so a genuinely new server-side
/// tab keeps working without a client release.
fn unknown_tab_error(tab: &str, response: &TaskNodeResponse) -> Option<Value> {
    if !response_is_ok(response) {
        return None;
    }
    let counts = response.body.get("counts")?.as_object()?;
    if counts.contains_key(tab) {
        return None;
    }
    let returned_nothing = response
        .body
        .get("tasks")
        .and_then(Value::as_array)
        .is_some_and(|tasks| tasks.is_empty());
    if !returned_nothing {
        return None;
    }
    let known: Vec<&str> = counts.keys().map(String::as_str).collect();
    Some(json!({
        "ok": false,
        "error": "tasknode_unknown_tab",
        "tab": tab,
        "knownTabs": known,
        "message": format!(
            "Unknown tab `{tab}`: the server reported no such tab and returned no tasks. \
             An empty result here means the tab name is wrong, not that you have no tasks. \
             Valid tabs: {}.",
            known.join(", ")
        ),
    }))
}

async fn run_task_command(client: &TaskNodeClient, cli: TaskCli) -> anyhow::Result<i32> {
    match cli.action {
        TaskCommand::Show(args) => emit_response(task_detail(client, &args.task_id).await?),
        TaskCommand::Accept(args) => {
            emit_response(task_action(client, &args.task_id, "accept", None).await?)
        }
        TaskCommand::Refuse(args) => {
            let reason = read_optional_text_input(args.reason, args.reason_file, "refusal reason")?;
            emit_response(task_action(client, &args.task_id, "refuse", reason).await?)
        }
        TaskCommand::Cancel(args) => {
            emit_response(task_action(client, &args.task_id, "cancel", None).await?)
        }
        TaskCommand::Evidence(args) => emit_response(task_evidence(client, args).await?),
    }
}

async fn run_verification_command(
    client: &TaskNodeClient,
    cli: VerificationCli,
) -> anyhow::Result<i32> {
    match cli.action {
        VerificationCommand::Respond(args) => emit_response(task_evidence(client, args).await?),
    }
}

async fn task_detail(client: &TaskNodeClient, task_id: &str) -> anyhow::Result<TaskNodeResponse> {
    client
        .get(&format!(
            "/api/terminal/tasknode/tasks/{}",
            urlencoding::encode(task_id)
        ))
        .await
}

async fn task_action(
    client: &TaskNodeClient,
    task_id: &str,
    action: &str,
    reason: Option<String>,
) -> anyhow::Result<TaskNodeResponse> {
    let mut body = Map::new();
    body.insert("action".to_string(), Value::String(action.to_string()));
    body.insert(
        "source".to_string(),
        Value::String("pfterminal-cli".to_string()),
    );
    body.insert(
        "idempotencyKey".to_string(),
        Value::String(idempotency_key(action)),
    );
    if let Some(reason) = reason.filter(|value| !value.trim().is_empty()) {
        body.insert("reason".to_string(), Value::String(reason));
    }
    client
        .post(
            &format!(
                "/api/terminal/tasknode/tasks/{}/action",
                urlencoding::encode(task_id)
            ),
            &Value::Object(body),
        )
        .await
}

async fn task_evidence(
    client: &TaskNodeClient,
    args: TaskEvidenceArgs,
) -> anyhow::Result<TaskNodeResponse> {
    let summary = read_text_input(args.summary, args.body_file, "task evidence")?;
    let body = json!({
        "summary": summary,
        "evidence": evidence_items_from_summary_and_artifacts(&summary, &args.artifacts),
        "source": "pfterminal-cli",
        "idempotencyKey": idempotency_key("evidence"),
    });
    client
        .post(
            &format!(
                "/api/terminal/tasknode/tasks/{}/evidence",
                urlencoding::encode(&args.task_id)
            ),
            &body,
        )
        .await
}

async fn run_link_command(
    config_overrides: CliConfigOverrides,
    origin_override: Option<String>,
    args: LinkArgs,
) -> anyhow::Result<i32> {
    let codex_home = codex_home_from_overrides(config_overrides).await?;
    run_link_command_for_codex_home(codex_home.as_path(), origin_override, args)
}

fn run_link_command_for_codex_home(
    codex_home: &Path,
    origin_override: Option<String>,
    args: LinkArgs,
) -> anyhow::Result<i32> {
    match link_action(&args)? {
        TaskNodeLinkAction::Status => emit_link_status(codex_home),
        TaskNodeLinkAction::Poll => poll_link_command(codex_home, origin_override, args.timeout),
        TaskNodeLinkAction::Start => start_link_command(codex_home, origin_override, args),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskNodeLinkAction {
    Start,
    Poll,
    Status,
}

fn link_action(args: &LinkArgs) -> anyhow::Result<TaskNodeLinkAction> {
    if args.poll && args.status {
        anyhow::bail!("Use only one of --poll or --status.");
    }
    if args.relink && (args.poll || args.status) {
        anyhow::bail!("Use --relink only when starting a new Task Node link.");
    }
    if args.poll {
        Ok(TaskNodeLinkAction::Poll)
    } else if args.status {
        Ok(TaskNodeLinkAction::Status)
    } else {
        Ok(TaskNodeLinkAction::Start)
    }
}

fn emit_link_status(codex_home: &Path) -> anyhow::Result<i32> {
    match TaskNodeLocalSession::load_optional(codex_home) {
        Ok(session) => {
            print_json(&link_state_json(session.as_ref(), true))?;
            Ok(0)
        }
        Err(err) => emit_tasknode_local_error(Some(codex_home), err),
    }
}

/// Guidance attached to every `pending` link response. An automated driver (a
/// chat harness, a script) sees the `verificationUrl` but has no way to know it
/// must (a) surface that URL to a human and (b) poll *without* blocking. Left
/// implicit, the common failure is to swallow the URL and fire a long foreground
/// `--poll`, which pins the single turn a chat connector allows and makes the
/// whole assistant look hung. State the recipe in-band so it cannot be missed.
const LINK_PENDING_NEXT_STEP: &str = "Show `verificationUrl` to the user; they authorize it with GitHub in any browser (a phone works). Then confirm with `pfterminal tasknode link --poll --timeout 0`, which checks once and returns immediately — safe to repeat from a chat turn. Do NOT run a long foreground `--poll` inside a chat turn: it blocks the turn until the link completes or times out.";

fn start_link_command(
    codex_home: &Path,
    origin_override: Option<String>,
    args: LinkArgs,
) -> anyhow::Result<i32> {
    let existing = match TaskNodeLocalSession::load_optional(codex_home) {
        Ok(session) => session,
        Err(err) => return emit_tasknode_local_error(Some(codex_home), err),
    };
    if let Some(session) = &existing {
        if session.terminal_token.is_some() && !args.relink {
            print_json(&json!({
                "ok": false,
                "error": "tasknode_already_linked",
                "state": "linked",
                "message": "Task Node is already linked. Use --relink to replace the local session.",
                "githubUsername": session.github_username.clone(),
                "expiresAt": session.expires_at.clone(),
                "origin": session.origin.clone(),
            }))?;
            return Ok(1);
        }
        if session.pending_request_id.is_some() && !args.relink {
            print_json(&link_state_json(Some(session), true))?;
            return Ok(0);
        }
    }

    let origin = resolve_origin(
        origin_override,
        existing.as_ref().map(|session| session.origin.as_str()),
    );
    let started =
        match TaskNodeClient::new_without_token_for_origin(origin.clone()).start_github_link() {
            Ok(started) => started,
            Err(err) => {
                print_json(&json!({
                    "ok": false,
                    "error": "tasknode_link_start_failed",
                    "message": err.to_string(),
                    "origin": origin,
                }))?;
                return Ok(1);
            }
        };
    let session = TaskNodeLocalSession {
        origin: origin.clone(),
        account_id: None,
        github_username: None,
        terminal_token: None,
        expires_at: None,
        pending_request_id: Some(started.request_id.clone()),
        pending_poll_token: Some(started.poll_token),
        pending_verification_url: Some(started.verification_url.clone()),
    };
    if let Err(err) = session.save(codex_home) {
        return emit_tasknode_local_error(Some(codex_home), err);
    }

    let mut browser_opened = false;
    let mut browser_error = None;
    if should_open_browser(args.no_browser) {
        match webbrowser::open(&started.verification_url) {
            Ok(()) => browser_opened = true,
            Err(err) => browser_error = Some(err.to_string()),
        }
    }

    let mut body = json!({
        "ok": true,
        "state": "pending",
        "verificationUrl": started.verification_url,
        "requestId": started.request_id,
        "origin": origin,
        "browserOpened": browser_opened,
        "message": "Task Node link started; authorization is pending.",
        "nextStep": LINK_PENDING_NEXT_STEP,
    });
    if let Some(err) = browser_error {
        body["browserError"] = Value::String(err);
    }
    print_json(&body)?;
    Ok(0)
}

fn poll_link_command(
    codex_home: &Path,
    origin_override: Option<String>,
    timeout_secs: u64,
) -> anyhow::Result<i32> {
    let mut session = match TaskNodeLocalSession::load_optional(codex_home) {
        Ok(Some(session)) => session,
        Ok(None) => {
            print_json(&unlinked_json(Some(codex_home)))?;
            return Ok(1);
        }
        Err(err) => return emit_tasknode_local_error(Some(codex_home), err),
    };
    if session.terminal_token.is_some() {
        print_json(&link_state_json(Some(&session), true))?;
        return Ok(0);
    }
    let Some(request_id) = session.pending_request_id.clone() else {
        print_json(&json!({
            "ok": false,
            "error": "tasknode_no_pending_link",
            "state": "unlinked",
            "message": "No pending Task Node link request is stored. Run `pfterminal tasknode link`.",
        }))?;
        return Ok(1);
    };
    let Some(poll_token) = session.pending_poll_token.clone() else {
        print_json(&json!({
            "ok": false,
            "error": "tasknode_no_pending_link",
            "state": "unlinked",
            "message": "The stored Task Node link request is missing poll state. Run `pfterminal tasknode link --relink`.",
        }))?;
        return Ok(1);
    };

    let origin = resolve_origin(origin_override, Some(&session.origin));
    let client = TaskNodeClient::new_without_token_for_origin(origin.clone());
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    let mut backoff = Duration::from_secs(1);
    loop {
        match client.poll_session(&request_id, &poll_token) {
            Ok(poll) => {
                session.origin = origin;
                session.apply_terminal_session(poll);
                if let Err(err) = session.save(codex_home) {
                    return emit_tasknode_local_error(Some(codex_home), err);
                }
                print_json(&link_state_json(Some(&session), true))?;
                return Ok(0);
            }
            Err(TaskNodeClientError::Pending) => {
                let now = Instant::now();
                if now >= deadline {
                    print_json(&json!({
                        "ok": false,
                        "error": "tasknode_link_timeout",
                        "state": "pending",
                        "message": "Task Node link is still pending.",
                        "verificationUrl": session.pending_verification_url.clone(),
                        "requestId": request_id,
                        "origin": session.origin.clone(),
                        "nextStep": LINK_PENDING_NEXT_STEP,
                    }))?;
                    return Ok(1);
                }
                let remaining = deadline.saturating_duration_since(now);
                std::thread::sleep(backoff.min(remaining));
                backoff = (backoff * 2).min(Duration::from_secs(5));
            }
            Err(TaskNodeClientError::Rejected(message)) => {
                print_json(&json!({
                    "ok": false,
                    "error": "tasknode_link_rejected",
                    "state": "pending",
                    "message": message,
                    "verificationUrl": session.pending_verification_url.clone(),
                    "requestId": request_id,
                    "origin": session.origin.clone(),
                }))?;
                return Ok(1);
            }
            Err(TaskNodeClientError::Http(message)) => {
                print_json(&json!({
                    "ok": false,
                    "error": "tasknode_link_poll_failed",
                    "state": "pending",
                    "message": message,
                    "verificationUrl": session.pending_verification_url.clone(),
                    "requestId": request_id,
                    "origin": session.origin.clone(),
                }))?;
                return Ok(1);
            }
        }
    }
}

fn should_open_browser(no_browser: bool) -> bool {
    !no_browser && std::io::stdout().is_terminal()
}

fn link_state_json(session: Option<&TaskNodeLocalSession>, ok: bool) -> Value {
    match session {
        Some(session) if session.terminal_token.is_some() => json!({
            "ok": ok,
            "state": "linked",
            "githubUsername": session.github_username.clone(),
            "expiresAt": session.expires_at.clone(),
            "origin": session.origin.clone(),
        }),
        Some(session) if session.pending_request_id.is_some() => json!({
            "ok": ok,
            "state": "pending",
            "verificationUrl": session.pending_verification_url.clone(),
            "requestId": session.pending_request_id.clone(),
            "origin": session.origin.clone(),
            "nextStep": LINK_PENDING_NEXT_STEP,
        }),
        _ => json!({
            "ok": ok,
            "state": "unlinked",
        }),
    }
}

/// The session is keyed to `CODEX_HOME`, so "not linked" is ambiguous: it can mean
/// *never linked*, or *linked, but you are looking in a different home*. The second
/// case is easy to hit — a service (the Telegram connector) sets `CODEX_HOME`
/// explicitly while an interactive shell falls back to the default — and the naive
/// advice ("run `tasknode link`") makes it worse by minting a second session the
/// service cannot see. Always name the directory that was actually searched.
fn unlinked_json(codex_home: Option<&Path>) -> Value {
    let home = match codex_home {
        Some(path) => path.display().to_string(),
        None => codex_core::config::find_codex_home()
            .map(|path| path.as_path().display().to_string())
            .unwrap_or_else(|_| "<unresolved>".to_string()),
    };
    json!({
        "ok": false,
        "error": "tasknode_unlinked",
        "state": "unlinked",
        "codexHome": home,
        "message": format!(
            "Task Node is not linked under CODEX_HOME={home}. If a service linked with a \
             different CODEX_HOME, export that same value instead of re-linking — a second \
             link creates a session the service cannot see. Otherwise run \
             `pfterminal tasknode link`."
        ),
    })
}

fn emit_tasknode_local_error(
    codex_home: Option<&Path>,
    err: TaskNodeLocalError,
) -> anyhow::Result<i32> {
    match err {
        TaskNodeLocalError::NotFound => {
            print_json(&unlinked_json(codex_home))?;
        }
        TaskNodeLocalError::VaultUnavailable(detail) => {
            print_json(&json!({
                "ok": false,
                "error": "tasknode_vault_unavailable",
                "state": "vault-unavailable",
                "message": "Task Node credential vault is unavailable.",
                "detail": detail,
            }))?;
        }
        TaskNodeLocalError::Corrupt(detail) => {
            print_json(&json!({
                "ok": false,
                "error": "tasknode_session_corrupt",
                "state": "corrupt",
                "message": "Task Node local session is corrupt.",
                "detail": detail,
            }))?;
        }
    }
    Ok(1)
}

fn emit_session_error(err: &TaskNodeCommandSessionError) -> anyhow::Result<i32> {
    match err {
        TaskNodeCommandSessionError::Unlinked => {
            print_json(&unlinked_json(None))?;
        }
        TaskNodeCommandSessionError::Pending { verification_url } => {
            print_json(&json!({
                "ok": false,
                "error": "tasknode_link_pending",
                "state": "pending",
                "message": "Task Node link is pending. Finish GitHub auth, then run `pfterminal tasknode link --poll`.",
                "verificationUrl": verification_url,
            }))?;
        }
        TaskNodeCommandSessionError::VaultUnavailable { detail } => {
            print_json(&json!({
                "ok": false,
                "error": "tasknode_vault_unavailable",
                "state": "vault-unavailable",
                "message": "Task Node credential vault is unavailable.",
                "detail": detail,
            }))?;
        }
        TaskNodeCommandSessionError::Corrupt { detail } => {
            print_json(&json!({
                "ok": false,
                "error": "tasknode_session_corrupt",
                "state": "corrupt",
                "message": "Task Node local session is corrupt.",
                "detail": detail,
            }))?;
        }
        TaskNodeCommandSessionError::MissingToken => {
            print_json(&json!({
                "ok": false,
                "error": "tasknode_missing_terminal_token",
                "state": "unlinked",
                "message": "Task Node session is missing a terminal token. Run `pfterminal tasknode link --relink`.",
            }))?;
        }
    }
    Ok(1)
}

#[derive(Debug, Error)]
enum TaskNodeCommandSessionError {
    #[error("Task Node is not linked")]
    Unlinked,
    #[error("Task Node link is pending")]
    Pending { verification_url: String },
    #[error("Task Node credential vault is unavailable: {detail}")]
    VaultUnavailable { detail: String },
    #[error("invalid local Task Node session: {detail}")]
    Corrupt { detail: String },
    #[error("Task Node session is missing a terminal token")]
    MissingToken,
}

async fn tasknode_client_from_cli(
    config_overrides: CliConfigOverrides,
    origin_override: Option<String>,
) -> anyhow::Result<TaskNodeClient> {
    let codex_home = codex_home_from_overrides(config_overrides).await?;
    let session = load_tasknode_session(codex_home.as_path()).map_err(anyhow::Error::new)?;
    let token = session.terminal_token.clone().ok_or_else(|| {
        anyhow::anyhow!("Task Node session is missing a terminal token. Run /tasknode link.")
    })?;
    let origin = resolve_origin(origin_override, Some(&session.origin));
    Ok(TaskNodeClient::new_with_origin(origin, token))
}

async fn codex_home_from_overrides(
    config_overrides: CliConfigOverrides,
) -> anyhow::Result<PathBuf> {
    let cli_kv_overrides = config_overrides
        .parse_overrides()
        .map_err(anyhow::Error::msg)?;
    let config = ConfigBuilder::default()
        .cli_overrides(cli_kv_overrides)
        .build()
        .await?;
    Ok(config.codex_home.to_path_buf())
}

trait TaskNodeCliClientExt {
    fn get(&self, path: &str) -> Ready<anyhow::Result<TaskNodeResponse>>;
    fn post(&self, path: &str, body: &Value) -> Ready<anyhow::Result<TaskNodeResponse>>;
    fn post_sse_jsonl(&self, path: &str, body: &Value) -> Ready<anyhow::Result<i32>>;
}

impl TaskNodeCliClientExt for TaskNodeClient {
    fn get(&self, path: &str) -> Ready<anyhow::Result<TaskNodeResponse>> {
        ready(self.get_raw_json(path).map_err(tasknode_client_error))
    }

    fn post(&self, path: &str, body: &Value) -> Ready<anyhow::Result<TaskNodeResponse>> {
        ready(
            self.post_raw_json(path, body)
                .map_err(tasknode_client_error),
        )
    }

    fn post_sse_jsonl(&self, path: &str, body: &Value) -> Ready<anyhow::Result<i32>> {
        ready(tasknode_post_sse_jsonl(self, path, body))
    }
}

fn load_tasknode_session(
    codex_home: &std::path::Path,
) -> Result<TaskNodeLocalSession, TaskNodeCommandSessionError> {
    let session = TaskNodeLocalSession::load(codex_home).map_err(|err| match err {
        TaskNodeLocalError::NotFound => TaskNodeCommandSessionError::Unlinked,
        TaskNodeLocalError::VaultUnavailable(detail) => {
            TaskNodeCommandSessionError::VaultUnavailable { detail }
        }
        TaskNodeLocalError::Corrupt(detail) => TaskNodeCommandSessionError::Corrupt { detail },
    })?;
    if session.terminal_token.is_none() {
        if let Some(url) = session
            .pending_verification_url
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            return Err(TaskNodeCommandSessionError::Pending {
                verification_url: url.to_string(),
            });
        }
        return Err(TaskNodeCommandSessionError::MissingToken);
    }
    Ok(session)
}

fn tasknode_post_sse_jsonl(
    client: &TaskNodeClient,
    path: &str,
    body: &Value,
) -> anyhow::Result<i32> {
    let mut request = streaming_http_client()?
        .post(client.url_for_path(path))
        .json(body);
    if let Some(token) = client.bearer_token() {
        request = request.bearer_auth(token);
    }
    let mut response = request.send().map_err(reqwest_error)?;
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();
    if !(200..300).contains(&status) || !content_type.contains("text/event-stream") {
        return emit_response(parse_blocking_response(response)?);
    }

    let mut stdout = std::io::stdout();
    let mut buffer = String::new();
    let mut saw_done = false;
    let mut exit_code = 0;
    let mut chunk = [0u8; 8192];
    loop {
        let read = response
            .read(&mut chunk)
            .context("failed reading Task Node chat stream")?;
        if read == 0 {
            break;
        }
        buffer.push_str(&String::from_utf8_lossy(&chunk[..read]));
        for block in tasknode_sse_drain_blocks(&mut buffer) {
            if let Some((event, data)) =
                tasknode_parse_sse_block(&block).map_err(tasknode_client_error)?
            {
                if event == "done" {
                    saw_done = true;
                } else if event == "error" {
                    exit_code = 1;
                }
                writeln!(
                    stdout,
                    "{}",
                    serde_json::to_string(&json!({ "event": event, "data": data }))?
                )?;
                stdout.flush()?;
            }
        }
    }
    for block in tasknode_sse_drain_remainder(&mut buffer) {
        if let Some((event, data)) =
            tasknode_parse_sse_block(&block).map_err(tasknode_client_error)?
        {
            if event == "done" {
                saw_done = true;
            } else if event == "error" {
                exit_code = 1;
            }
            writeln!(
                stdout,
                "{}",
                serde_json::to_string(&json!({ "event": event, "data": data }))?
            )?;
        }
    }
    stdout.flush()?;
    if !saw_done && exit_code == 0 {
        print_json(&json!({
            "ok": false,
            "error": "tasknode_stream_incomplete",
            "message": "Task Node chat stream ended without a done event.",
        }))?;
        return Ok(1);
    }
    Ok(exit_code)
}

fn parse_blocking_response(
    mut response: reqwest::blocking::Response,
) -> anyhow::Result<TaskNodeResponse> {
    let status = response.status().as_u16();
    let mut text = String::new();
    response
        .read_to_string(&mut text)
        .context("failed reading Task Node response")?;
    let body = serde_json::from_str::<Value>(&text).unwrap_or_else(|_| {
        json!({
            "ok": false,
            "error": "tasknode_non_json_response",
            "message": text,
            "httpStatus": status,
        })
    });
    Ok(TaskNodeResponse { status, body })
}

fn tasknode_client_error(err: TaskNodeClientError) -> anyhow::Error {
    anyhow::anyhow!(err.to_string())
}

fn streaming_http_client() -> anyhow::Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(reqwest_error)
}

fn emit_response(response: TaskNodeResponse) -> anyhow::Result<i32> {
    print_json(&response.body)?;
    Ok(if response_is_ok(&response) { 0 } else { 1 })
}

fn response_is_ok(response: &TaskNodeResponse) -> bool {
    (200..300).contains(&response.status)
        && response.body.get("ok").and_then(Value::as_bool) != Some(false)
}

fn print_json(value: &Value) -> anyhow::Result<()> {
    let mut stdout = std::io::stdout();
    writeln!(stdout, "{}", serde_json::to_string(value)?)?;
    stdout.flush()?;
    Ok(())
}

fn read_text_input(
    inline: Option<String>,
    file: Option<PathBuf>,
    label: &str,
) -> anyhow::Result<String> {
    match (inline, file) {
        (Some(_), Some(_)) => anyhow::bail!(
            "Provide either --message/--text/--summary or a file for {label}, not both."
        ),
        (Some(text), None) => require_nonempty_text(text, label),
        (None, Some(path)) => read_file_required(&path, label),
        (None, None) => anyhow::bail!("{label} is required."),
    }
}

fn read_optional_text_input(
    inline: Option<String>,
    file: Option<PathBuf>,
    label: &str,
) -> anyhow::Result<Option<String>> {
    match (inline, file) {
        (Some(_), Some(_)) => {
            anyhow::bail!("Provide either inline text or a file for {label}, not both.")
        }
        (Some(text), None) => Ok(Some(require_nonempty_text(text, label)?)),
        (None, Some(path)) => Ok(Some(read_file_required(&path, label)?)),
        (None, None) => Ok(None),
    }
}

fn read_file_required(path: &PathBuf, label: &str) -> anyhow::Result<String> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed reading {label} file {}", path.display()))?;
    require_nonempty_text(text, label)
}

fn require_nonempty_text(text: String, label: &str) -> anyhow::Result<String> {
    if text.trim().is_empty() {
        anyhow::bail!("{label} is empty.");
    }
    Ok(text)
}

fn idempotency_key(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("pfterminal-cli:{prefix}:{}:{nanos}", std::process::id())
}

fn new_chat_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("chat_cli_{}_{}", std::process::id(), nanos)
}

fn limit(value: u8, min: u8, max: u8) -> u8 {
    value.clamp(min, max)
}

fn limit_u16(value: u16, min: u16, max: u16) -> u16 {
    value.clamp(min, max)
}

fn evidence_items_from_summary_and_artifacts(summary: &str, artifacts: &[String]) -> Vec<Value> {
    let mut items = artifacts
        .iter()
        .filter_map(|artifact| evidence_item_from_artifact(artifact))
        .collect::<Vec<_>>();
    for url in summary
        .split_whitespace()
        .filter(|part| part.starts_with("http://") || part.starts_with("https://"))
        .take(5)
    {
        if !items
            .iter()
            .any(|item| evidence_item_value(item) == Some(url))
        {
            items.push(evidence_item_from_value(infer_artifact_type(url), url));
        }
    }
    items
}

fn evidence_item_from_artifact(artifact: &str) -> Option<Value> {
    let trimmed = artifact.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some((kind, value)) = trimmed.split_once('=') {
        let value = value.trim();
        if value.is_empty() {
            return None;
        }
        return Some(evidence_item_from_value(kind.trim(), value));
    }
    Some(evidence_item_from_value(
        infer_artifact_type(trimmed),
        trimmed,
    ))
}

fn evidence_item_from_value(kind: &str, value: &str) -> Value {
    if value.starts_with("http://") || value.starts_with("https://") {
        json!({ "type": kind, "url": value })
    } else {
        json!({ "type": kind, "value": value })
    }
}

fn evidence_item_value(item: &Value) -> Option<&str> {
    item.get("url")
        .or_else(|| item.get("value"))
        .or_else(|| item.get("text"))
        .and_then(Value::as_str)
}

fn infer_artifact_type(value: &str) -> &'static str {
    if value.contains("github.com/") && value.contains("/pull/") {
        "github_pr"
    } else if value.contains("github.com/") && value.contains("/commit/") {
        "git_commit"
    } else if value.starts_with("http://") || value.starts_with("https://") {
        "url"
    } else {
        "text"
    }
}

fn reqwest_error(err: reqwest::Error) -> anyhow::Error {
    let mut message = err.to_string();
    let mut source = std::error::Error::source(&err);
    while let Some(err) = source {
        let part = err.to_string();
        if !part.is_empty() && !message.contains(&part) {
            message.push_str(": ");
            message.push_str(&part);
        }
        source = std::error::Error::source(err);
    }
    anyhow::anyhow!(message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::Mock;
    use wiremock::ResponseTemplate;
    use wiremock::matchers::method;
    use wiremock::matchers::path;

    #[test]
    fn infers_evidence_items_from_summary_urls_and_artifacts() {
        let items = evidence_items_from_summary_and_artifacts(
            "Implemented in https://github.com/postfiatorg/tasknodeofficial/pull/192 and commit https://github.com/postfiatorg/tasknodeofficial/commit/abc",
            &["log=terminal smoke passed".to_string()],
        );

        assert_eq!(items.len(), 3);
        assert_eq!(items[0].get("type").and_then(Value::as_str), Some("log"));
        assert_eq!(
            items[1].get("type").and_then(Value::as_str),
            Some("github_pr")
        );
        assert_eq!(
            items[2].get("type").and_then(Value::as_str),
            Some("git_commit")
        );
    }

    #[test]
    fn parses_sse_delta_and_done_blocks() {
        let mut buffer = String::new();
        buffer.push_str("event: delta\ndata: {\"delta\":\"hi\"}\n\n");
        buffer.push_str("event: done\ndata: {\"ok\":true}\n\n");

        let blocks = tasknode_sse_drain_blocks(&mut buffer);
        assert_eq!(blocks.len(), 2);

        let first = tasknode_parse_sse_block(&blocks[0])
            .expect("valid first block")
            .expect("first event");
        assert_eq!(first.0, "delta");
        assert_eq!(first.1.get("delta").and_then(Value::as_str), Some("hi"));

        let second = tasknode_parse_sse_block(&blocks[1])
            .expect("valid second block")
            .expect("second event");
        assert_eq!(second.0, "done");
        assert_eq!(second.1.get("ok").and_then(Value::as_bool), Some(true));
    }

    #[test]
    fn parses_crlf_sse_blocks() {
        let mut buffer = "event: error\r\ndata: {\"message\":\"failed\"}\r\n\r\n".to_string();
        let blocks = tasknode_sse_drain_blocks(&mut buffer);
        assert_eq!(blocks.len(), 1);
        let parsed = tasknode_parse_sse_block(&blocks[0])
            .expect("valid crlf block")
            .expect("event");
        assert_eq!(parsed.0, "error");
        assert_eq!(
            parsed.1.get("message").and_then(Value::as_str),
            Some("failed")
        );
    }

    fn tasks_response(tab: &str, tasks: Value) -> TaskNodeResponse {
        TaskNodeResponse {
            status: 200,
            body: json!({
                "ok": true,
                "tab": tab,
                "tasks": tasks,
                "counts": {"outstanding": 0, "verification": 0, "refused": 14, "rewarded": 36},
            }),
        }
    }

    #[test]
    fn unlinked_message_names_the_codex_home_it_searched() {
        let body = unlinked_json(Some(Path::new("/home/ubuntu/.codex")));

        assert_eq!(
            body.get("codexHome").and_then(Value::as_str),
            Some("/home/ubuntu/.codex")
        );
        let message = body
            .get("message")
            .and_then(Value::as_str)
            .expect("message present");
        // The dangerous advice is a bare "run link": it mints a second session in a
        // different home. The message must surface the home and the export path first.
        assert!(message.contains("/home/ubuntu/.codex"), "{message}");
        assert!(message.contains("CODEX_HOME"), "{message}");
    }

    #[test]
    fn unknown_tab_is_rejected_instead_of_looking_empty() {
        let response = tasks_response("zzznotatab", json!([]));

        let error = unknown_tab_error("zzznotatab", &response).expect("typo'd tab rejected");

        assert_eq!(
            error.get("error").and_then(Value::as_str),
            Some("tasknode_unknown_tab")
        );
        let known = error
            .get("knownTabs")
            .and_then(Value::as_array)
            .expect("knownTabs present");
        assert!(known.iter().any(|tab| tab == "rewarded"));
    }

    #[test]
    fn known_tab_with_zero_tasks_is_not_an_error() {
        // `outstanding: 0` is a real, empty tab. Rejecting it would be worse than
        // the bug we are fixing.
        let response = tasks_response("outstanding", json!([]));

        assert!(unknown_tab_error("outstanding", &response).is_none());
    }

    #[test]
    fn unlisted_tab_that_returns_tasks_still_passes_through() {
        // A tab the server adds later will not appear in `counts`; if it returns
        // tasks, the client must not veto it.
        let response = tasks_response("archived", json!([{"id": "task_1"}]));

        assert!(unknown_tab_error("archived", &response).is_none());
    }

    #[test]
    fn pending_link_state_carries_non_blocking_next_step() {
        let session = TaskNodeLocalSession {
            origin: "https://tasknode.example".to_string(),
            account_id: None,
            github_username: None,
            terminal_token: None,
            expires_at: None,
            pending_request_id: Some("req-1".to_string()),
            pending_poll_token: Some("poll-1".to_string()),
            pending_verification_url: Some("https://tasknode.example/auth/req-1".to_string()),
        };

        let body = link_state_json(Some(&session), true);

        assert_eq!(body.get("state").and_then(Value::as_str), Some("pending"));
        let next = body
            .get("nextStep")
            .and_then(Value::as_str)
            .expect("pending state must carry nextStep guidance");
        // The two failures this guidance exists to prevent: swallowing the URL,
        // and blocking the turn with a long foreground poll.
        assert!(
            next.contains("verificationUrl"),
            "nextStep must tell the driver to surface the URL: {next}"
        );
        assert!(
            next.contains("--timeout 0"),
            "nextStep must steer to the non-blocking single-shot poll: {next}"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn link_poll_timeout_leaves_pending_session_intact() -> anyhow::Result<()> {
        let server = wiremock::MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/auth/terminal/session"))
            .respond_with(ResponseTemplate::new(202))
            .expect(1)
            .mount(&server)
            .await;
        let codex_home = tempfile::tempdir()?;
        let pending = pending_session(&server.uri());
        pending.save(codex_home.path())?;

        let exit_code = run_link_command_for_codex_home(
            codex_home.path(),
            Some(server.uri()),
            link_args(
                /*poll*/ true, /*status*/ false, /*relink*/ false,
                /*timeout*/ 0,
            ),
        )?;

        assert_eq!(exit_code, 1);
        let loaded = TaskNodeLocalSession::load(codex_home.path())?;
        assert_eq!(loaded, pending);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn link_poll_rejected_exits_nonzero_and_keeps_pending_session() -> anyhow::Result<()> {
        let server = wiremock::MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/auth/terminal/session"))
            .respond_with(ResponseTemplate::new(410).set_body_json(json!({
                "message": "link request expired",
            })))
            .expect(1)
            .mount(&server)
            .await;
        let codex_home = tempfile::tempdir()?;
        let pending = pending_session(&server.uri());
        pending.save(codex_home.path())?;

        let exit_code = run_link_command_for_codex_home(
            codex_home.path(),
            Some(server.uri()),
            link_args(
                /*poll*/ true, /*status*/ false, /*relink*/ false,
                /*timeout*/ 30,
            ),
        )?;

        assert_eq!(exit_code, 1);
        let loaded = TaskNodeLocalSession::load(codex_home.path())?;
        assert_eq!(loaded, pending);
        Ok(())
    }

    #[test]
    fn link_refuses_to_overwrite_linked_session_without_relink() -> anyhow::Result<()> {
        let codex_home = tempfile::tempdir()?;
        let linked = TaskNodeLocalSession {
            origin: "https://tasknode.example".to_string(),
            account_id: Some("acct-1".to_string()),
            github_username: Some("octocat".to_string()),
            terminal_token: Some("terminal-secret".to_string()),
            expires_at: Some("2026-07-08T12:00:00Z".to_string()),
            pending_request_id: None,
            pending_poll_token: None,
            pending_verification_url: None,
        };
        linked.save(codex_home.path())?;

        let exit_code = run_link_command_for_codex_home(
            codex_home.path(),
            None,
            link_args(
                /*poll*/ false, /*status*/ false, /*relink*/ false,
                /*timeout*/ 300,
            ),
        )?;

        assert_eq!(exit_code, 1);
        let loaded = TaskNodeLocalSession::load(codex_home.path())?;
        assert_eq!(loaded, linked);
        Ok(())
    }

    fn pending_session(origin: &str) -> TaskNodeLocalSession {
        TaskNodeLocalSession {
            origin: origin.to_string(),
            account_id: None,
            github_username: None,
            terminal_token: None,
            expires_at: None,
            pending_request_id: Some("req-123".to_string()),
            pending_poll_token: Some("poll-secret".to_string()),
            pending_verification_url: Some("https://verify.example/link".to_string()),
        }
    }

    fn link_args(poll: bool, status: bool, relink: bool, timeout: u64) -> LinkArgs {
        LinkArgs {
            poll,
            status,
            relink,
            timeout,
            no_browser: true,
        }
    }
}
