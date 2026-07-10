use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use chrono::DateTime;
use chrono::Duration;
use chrono::NaiveTime;
use chrono::Timelike;
use chrono::Utc;
use codex_protocol::ThreadId;
use crossterm::event::KeyCode;
use ratatui::text::Line;
use serde::Deserialize;
use serde::Serialize;

use crate::app::App;
use crate::app_event::AppEvent;
use crate::bottom_pane::SelectionItem;
use crate::bottom_pane::SelectionShortcutAction;
use crate::bottom_pane::SelectionViewParams;
use crate::bottom_pane::custom_prompt_view::CustomPromptSubmitMode;
use crate::bottom_pane::custom_prompt_view::CustomPromptView;
use crate::bottom_pane::popup_consts::standard_popup_hint_line;
use crate::claude_panes::CODEX_MAIN_PANE_ID;
use crate::claude_panes::ClaudePaneStatus;
use crate::key_hint;
use crate::spawn_orchestration::SpawnTaskTarget;
use crate::spawn_orchestration::node_id_pane;
use crate::spawn_orchestration::node_id_thread;
use crate::spawn_orchestration::pane_node_id;
use crate::spawn_orchestration::thread_node_id;

const ORCHESTRATE_FENCE_OPEN: &str = "```pfterminal-orchestrate";
const ORCHESTRATE_FENCE_CLOSE: &str = "```";
const DEFAULT_EXPIRY_SECONDS: i64 = 4 * 60 * 60;
const DEFAULT_MAX_FIRES: u32 = 20;
const DEFAULT_COOLDOWN_S: u64 = 60;
const DEFAULT_STOP_MARKER: &str = "WHIP_DONE";
const BUILTIN_KEEP_GOING_WHIP: &str = "keep-going";
const BUILTIN_KEEP_GOING_PATH: &str = "built-in:keep-going";
const BUILTIN_KEEP_GOING_INSTRUCTION: &str =
    "Continue the assigned work. If finished, report done and emit WHIP_DONE.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum WhipMode {
    Review,
    Auto,
}

impl WhipMode {
    fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "review" => Ok(Self::Review),
            "auto" => Ok(Self::Auto),
            other => Err(format!(
                "Unknown whip mode `{other}`; expected review or auto."
            )),
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Review => "review",
            Self::Auto => "auto",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum WhipState {
    #[default]
    Armed,
    Paused,
    Exhausted,
    Expired,
    Detached,
}

impl WhipState {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Armed => "armed",
            Self::Paused => "paused",
            Self::Exhausted => "exhausted",
            Self::Expired => "expired",
            Self::Detached => "detached",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Whip {
    pub(crate) id: String,
    pub(crate) holder: Option<String>,
    pub(crate) target: String,
    pub(crate) instructions: String,
    pub(crate) mode: WhipMode,
    pub(crate) expires_at: Option<DateTime<Utc>>,
    pub(crate) max_fires: u32,
    pub(crate) cooldown_s: u64,
    pub(crate) stop_marker: String,
    #[serde(default)]
    pub(crate) fires: u32,
    #[serde(default)]
    pub(crate) last_fire_utc: Option<DateTime<Utc>>,
    #[serde(default)]
    pub(crate) state: WhipState,
    #[serde(default)]
    pub(crate) last_idle_generation_fired: Option<u64>,
    #[serde(default)]
    pub(crate) empty_output_fires: u32,
    #[serde(default)]
    pub(crate) consecutive_failed_turns: u32,
    #[serde(default)]
    pub(crate) pending_review_fire: Option<u32>,
    #[serde(default)]
    pub(crate) ignored_review_fires: u32,
    #[serde(default)]
    pub(crate) expiry_notified: bool,
}

impl Whip {
    fn new(
        id: String,
        holder: Option<String>,
        target: String,
        instructions: String,
        options: ResolvedAttachOptions,
        now: DateTime<Utc>,
    ) -> Self {
        let expires_at = options
            .expiry
            .unwrap_or_else(|| Some(now + Duration::seconds(DEFAULT_EXPIRY_SECONDS)));
        Self {
            id,
            holder,
            target,
            instructions,
            mode: options.mode.unwrap_or(WhipMode::Review),
            expires_at,
            max_fires: options.max_fires.unwrap_or(DEFAULT_MAX_FIRES),
            cooldown_s: options.cooldown_s.unwrap_or(DEFAULT_COOLDOWN_S),
            stop_marker: options
                .stop_marker
                .unwrap_or_else(|| DEFAULT_STOP_MARKER.to_string()),
            fires: 0,
            last_fire_utc: None,
            state: WhipState::Armed,
            last_idle_generation_fired: None,
            empty_output_fires: 0,
            consecutive_failed_turns: 0,
            pending_review_fire: None,
            ignored_review_fires: 0,
            expiry_notified: false,
        }
    }

    fn is_armed(&self) -> bool {
        self.state == WhipState::Armed
    }
}

#[derive(Debug, Clone, Default)]
struct ResolvedAttachOptions {
    mode: Option<WhipMode>,
    expiry: Option<Option<DateTime<Utc>>>,
    max_fires: Option<u32>,
    cooldown_s: Option<u64>,
    stop_marker: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HolderArg {
    Me,
    None,
    Target(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OrchestrateCommand {
    Status,
    Attach {
        target: String,
        whip_name: String,
        mode: Option<WhipMode>,
        expiry: Option<ExpiryArg>,
        max_fires: Option<u32>,
        cooldown_s: Option<u64>,
        holder: Option<HolderArg>,
    },
    Detach(String),
    Pause(String),
    Resume(String),
    Extend {
        id: String,
        duration: DurationArg,
    },
    Fire(String),
    Test(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExpiryArg {
    Duration(DurationArg),
    UntilTodayOrTomorrow { hour: u32, minute: u32 },
    Unlimited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DurationArg {
    seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OrchestrateBlock {
    pub(crate) command: OrchestrateCommand,
}

#[derive(Debug, Clone)]
struct WhipDocDefaults {
    mode: Option<WhipMode>,
    max_fires: Option<u32>,
    cooldown_s: Option<u64>,
    stop_marker: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WhipInstructionEntry {
    name: String,
    description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OrchestrateTargetEntry {
    target: String,
    node_id: String,
    name: String,
    description: String,
    is_current: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FireDestination {
    Native(ThreadId),
    ClaudePane(String),
}

#[derive(Debug, Clone)]
struct FirePlan {
    whip_id: String,
    mode: WhipMode,
    destination: FireDestination,
    task: String,
    target_idle_generation: u64,
    destination_label: String,
}

pub(crate) fn parse_orchestrate_command(input: &str) -> Result<OrchestrateCommand, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(OrchestrateCommand::Status);
    }
    let mut parts = trimmed.split_whitespace();
    let action = parts.next().unwrap_or_default().to_ascii_lowercase();
    match action.as_str() {
        "status" => Ok(OrchestrateCommand::Status),
        "attach" => {
            let target = parts
                .next()
                .ok_or_else(|| orchestrate_usage().to_string())?
                .to_string();
            let whip_name = parts
                .next()
                .ok_or_else(|| orchestrate_usage().to_string())?
                .to_string();
            let mut mode = None;
            let mut expiry = None;
            let mut max_fires = None;
            let mut cooldown_s = None;
            let mut holder = None;
            while let Some(flag) = parts.next() {
                match flag {
                    "--mode" => {
                        let value = parts
                            .next()
                            .ok_or_else(|| "Missing value after --mode.".to_string())?;
                        mode = Some(WhipMode::parse(value)?);
                    }
                    "--for" => {
                        let value = parts
                            .next()
                            .ok_or_else(|| "Missing value after --for.".to_string())?;
                        expiry = Some(if value.eq_ignore_ascii_case("unlimited") {
                            ExpiryArg::Unlimited
                        } else {
                            ExpiryArg::Duration(parse_duration_arg(value)?)
                        });
                    }
                    "--until" => {
                        let value = parts
                            .next()
                            .ok_or_else(|| "Missing value after --until.".to_string())?;
                        expiry = Some(parse_until_arg(value)?);
                    }
                    "--max" => {
                        let value = parts
                            .next()
                            .ok_or_else(|| "Missing value after --max.".to_string())?;
                        max_fires = Some(parse_positive_u32(value, "--max")?);
                    }
                    "--cooldown" => {
                        let value = parts
                            .next()
                            .ok_or_else(|| "Missing value after --cooldown.".to_string())?;
                        cooldown_s = Some(parse_duration_arg(value)?.seconds.max(0) as u64);
                    }
                    "--holder" => {
                        let value = parts
                            .next()
                            .ok_or_else(|| "Missing value after --holder.".to_string())?;
                        holder = Some(parse_holder_arg(value));
                    }
                    other => return Err(format!("Unknown /orchestrate attach option `{other}`.")),
                }
            }
            Ok(OrchestrateCommand::Attach {
                target,
                whip_name,
                mode,
                expiry,
                max_fires,
                cooldown_s,
                holder,
            })
        }
        "detach" => one_arg(action.as_str(), parts).map(OrchestrateCommand::Detach),
        "pause" => one_arg(action.as_str(), parts).map(OrchestrateCommand::Pause),
        "resume" => one_arg(action.as_str(), parts).map(OrchestrateCommand::Resume),
        "fire" => one_arg(action.as_str(), parts).map(OrchestrateCommand::Fire),
        "test" => one_arg(action.as_str(), parts).map(OrchestrateCommand::Test),
        "extend" => {
            let id = parts
                .next()
                .ok_or_else(|| "Usage: /orchestrate extend <id> <duration>".to_string())?
                .to_string();
            let duration = parts
                .next()
                .ok_or_else(|| "Usage: /orchestrate extend <id> <duration>".to_string())
                .and_then(parse_duration_arg)?;
            if parts.next().is_some() {
                return Err("Usage: /orchestrate extend <id> <duration>".to_string());
            }
            Ok(OrchestrateCommand::Extend { id, duration })
        }
        _ => Err(orchestrate_usage().to_string()),
    }
}

pub(crate) fn extract_orchestrate_blocks(text: &str) -> (String, Vec<OrchestrateBlock>) {
    let mut visible = String::new();
    let mut blocks = Vec::new();
    let mut rest = text;
    while let Some(start_index) = rest.find(ORCHESTRATE_FENCE_OPEN) {
        visible.push_str(&rest[..start_index]);
        let block = &rest[start_index..];
        let Some(header_end) = block.find('\n') else {
            visible.push_str(block);
            rest = "";
            break;
        };
        let content_start = header_end + 1;
        let Some(close_index) = block[content_start..].find(ORCHESTRATE_FENCE_CLOSE) else {
            visible.push_str(block);
            rest = "";
            break;
        };
        let content_end = content_start + close_index;
        let content = &block[content_start..content_end];
        if let Some(command) = parse_orchestrate_block_content(content) {
            blocks.push(OrchestrateBlock { command });
        }
        rest = &block[content_end + ORCHESTRATE_FENCE_CLOSE.len()..];
    }
    visible.push_str(rest);
    (visible.trim().to_string(), blocks)
}

fn parse_orchestrate_block_content(content: &str) -> Option<OrchestrateCommand> {
    let fields = yamlish_fields(content);
    let action = fields.get("action")?.trim().to_ascii_lowercase();
    match action.as_str() {
        "attach" => {
            let target = fields.get("target")?.trim().to_string();
            let whip_name = fields
                .get("whip")
                .or_else(|| fields.get("whip_name"))?
                .trim()
                .to_string();
            let mode = fields
                .get("mode")
                .map(|value| WhipMode::parse(value))
                .transpose()
                .ok()?;
            let holder = fields.get("holder").map(|value| parse_holder_arg(value));
            let expiry = fields
                .get("for")
                .map(|value| {
                    if value.trim().eq_ignore_ascii_case("unlimited") {
                        Ok(ExpiryArg::Unlimited)
                    } else {
                        parse_duration_arg(value).map(ExpiryArg::Duration)
                    }
                })
                .or_else(|| {
                    fields
                        .get("until")
                        .map(|value| parse_until_arg(value.trim()))
                })
                .transpose()
                .ok()?;
            Some(OrchestrateCommand::Attach {
                target,
                whip_name,
                mode,
                expiry,
                max_fires: fields
                    .get("max")
                    .and_then(|value| value.trim().parse::<u32>().ok()),
                cooldown_s: fields.get("cooldown").and_then(|value| {
                    parse_duration_arg(value)
                        .ok()
                        .map(|duration| duration.seconds.max(0) as u64)
                }),
                holder,
            })
        }
        "detach" | "pause" | "resume" | "fire" | "test" => {
            let id = fields
                .get("id")
                .or_else(|| fields.get("target"))
                .or_else(|| fields.get("whip"))?
                .trim()
                .to_string();
            match action.as_str() {
                "detach" => Some(OrchestrateCommand::Detach(id)),
                "pause" => Some(OrchestrateCommand::Pause(id)),
                "resume" => Some(OrchestrateCommand::Resume(id)),
                "fire" => Some(OrchestrateCommand::Fire(id)),
                "test" => Some(OrchestrateCommand::Test(id)),
                _ => None,
            }
        }
        "extend" => {
            let id = fields.get("id")?.trim().to_string();
            let duration = fields
                .get("duration")
                .and_then(|value| parse_duration_arg(value).ok())?;
            Some(OrchestrateCommand::Extend { id, duration })
        }
        _ => None,
    }
}

pub(crate) fn orchestrate_usage() -> &'static str {
    "Usage: /orchestrate [status|attach <target> <whip-name> [--mode review|auto] [--for 4h|--until HH:MM|--for unlimited] [--max N] [--cooldown S] [--holder me|none]|detach <id|target>|pause <id>|resume <id>|extend <id> <duration>|fire <id>|test <id>]"
}

pub(crate) fn format_whip_status(whips: &HashMap<String, Whip>, now: DateTime<Utc>) -> String {
    if whips.is_empty() {
        return "No whips attached.".to_string();
    }
    let mut ordered: Vec<_> = whips.values().collect();
    ordered.sort_by(|a, b| a.id.cmp(&b.id));
    let mut out = String::from("Whips:\n");
    for whip in ordered {
        let holder = whip.holder.as_deref().unwrap_or("none");
        let expiry = match whip.expires_at {
            Some(expires_at) if expires_at <= now => "expired".to_string(),
            Some(expires_at) => format!("expires {}", expires_at.format("%H:%MZ")),
            None => "unlimited".to_string(),
        };
        let _ = writeln!(
            out,
            "- {}: {} -> {} using {} ({}, {}/{}, {}, {})",
            whip.id,
            holder,
            whip.target,
            whip.instructions,
            whip.mode.label(),
            whip.fires,
            whip.max_fires,
            whip.state.label(),
            expiry,
        );
    }
    out.trim_end().to_string()
}

pub(crate) fn resolve_whip_instruction_path(
    codex_home: &Path,
    cwd: &Path,
    name: &str,
) -> Result<Option<PathBuf>, String> {
    validate_whip_name(name)?;
    let file_name = if name.ends_with(".md") {
        name.to_string()
    } else {
        format!("{name}.md")
    };
    let project_path = cwd.join(".pfterminal").join("whips").join(&file_name);
    if project_path.exists() {
        return Ok(Some(project_path));
    }
    let global_path = codex_home.join("whips").join(file_name);
    Ok(global_path.exists().then_some(global_path))
}

fn normalize_whip_name(name: &str) -> &str {
    name.trim()
        .strip_suffix(".md")
        .unwrap_or_else(|| name.trim())
}

fn is_builtin_keep_going_name(name: &str) -> bool {
    normalize_whip_name(name) == BUILTIN_KEEP_GOING_WHIP
}

fn scan_whip_instruction_dir(
    dir: &Path,
    source: &str,
    seen: &mut HashSet<String>,
    entries: &mut Vec<WhipInstructionEntry>,
) {
    let Ok(read_dir) = fs::read_dir(dir) else {
        return;
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let Some(name) = file_name.strip_suffix(".md") else {
            continue;
        };
        if validate_whip_name(name).is_err() || !seen.insert(name.to_string()) {
            continue;
        }
        entries.push(WhipInstructionEntry {
            name: name.to_string(),
            description: format!("{source}: {}", path.display()),
        });
    }
}

fn available_whip_instruction_entries(codex_home: &Path, cwd: &Path) -> Vec<WhipInstructionEntry> {
    let mut seen = HashSet::new();
    let mut entries = Vec::new();
    scan_whip_instruction_dir(
        &cwd.join(".pfterminal").join("whips"),
        "project",
        &mut seen,
        &mut entries,
    );
    scan_whip_instruction_dir(&codex_home.join("whips"), "global", &mut seen, &mut entries);
    if seen.insert(BUILTIN_KEEP_GOING_WHIP.to_string()) {
        entries.push(WhipInstructionEntry {
            name: BUILTIN_KEEP_GOING_WHIP.to_string(),
            description: "built-in: continue work until WHIP_DONE".to_string(),
        });
    }
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    entries
}

fn validate_whip_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Whip name cannot be empty.".to_string());
    }
    if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains("..") {
        return Err(format!(
            "Invalid whip name `{name}`; use a basename from the whips directory."
        ));
    }
    Ok(())
}

fn read_whip_instruction(
    codex_home: &Path,
    cwd: &Path,
    name: &str,
) -> Result<(PathBuf, String), String> {
    let path = match resolve_whip_instruction_path(codex_home, cwd, name)? {
        Some(path) => path,
        None if is_builtin_keep_going_name(name) => {
            return Ok((
                PathBuf::from(BUILTIN_KEEP_GOING_PATH),
                BUILTIN_KEEP_GOING_INSTRUCTION.to_string(),
            ));
        }
        None => return Err(format!("No whip instruction file found for `{name}`.")),
    };
    let contents = fs::read_to_string(&path)
        .map_err(|err| format!("Failed to read whip `{}`: {err}", path.display()))?;
    if contents.trim().is_empty() {
        return Err(format!(
            "Whip instruction file `{}` is empty.",
            path.display()
        ));
    }
    Ok((path, contents))
}

fn orchestrate_guided_attach_args(target: &str, duration_arg: &str, whip_name: &str) -> String {
    format!(
        "attach {target} {whip_name} --mode review --for {duration_arg} --max {DEFAULT_MAX_FIRES} --cooldown {DEFAULT_COOLDOWN_S}s --holder me"
    )
}

fn truncate_for_orchestrate_display(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

pub(crate) fn whip_suffix_for_target(
    whips: &HashMap<String, Whip>,
    target_node_id: &str,
) -> String {
    let Some(whip) = whips
        .values()
        .find(|whip| whip.target == target_node_id && whip.state == WhipState::Armed)
    else {
        return String::new();
    };
    let expiry = whip
        .expires_at
        .map(|expires_at| format!(", expires {}", expires_at.format("%H:%MZ")))
        .unwrap_or_else(|| ", unlimited".to_string());
    format!(
        "; whip={}({}, {}/{}{})",
        whip.instructions,
        whip.mode.label(),
        whip.fires,
        whip.max_fires,
        expiry
    )
}

fn orchestrate_shortcut_command(key: char, args: String) -> SelectionShortcutAction {
    SelectionShortcutAction {
        key: key_hint::plain(KeyCode::Char(key)),
        action: Box::new(move |tx| {
            tx.send(AppEvent::HandleOrchestrateCommand { args: args.clone() });
        }),
        dismiss_on_select: true,
    }
}

fn orchestrate_shortcut_extend(key: char, whip_id: String) -> SelectionShortcutAction {
    SelectionShortcutAction {
        key: key_hint::plain(KeyCode::Char(key)),
        action: Box::new(move |tx| {
            tx.send(AppEvent::OpenOrchestrateExtendDurationPicker {
                whip_id: whip_id.clone(),
            });
        }),
        dismiss_on_select: true,
    }
}

fn orchestrate_command_item(name: &str, description: &str, args: String) -> SelectionItem {
    SelectionItem {
        name: name.to_string(),
        description: Some(description.to_string()),
        actions: vec![Box::new(move |tx| {
            tx.send(AppEvent::HandleOrchestrateCommand { args: args.clone() });
        })],
        dismiss_on_select: true,
        ..Default::default()
    }
}

fn suggested_whip_name(instructions: &str) -> String {
    let mut slug = String::new();
    for ch in instructions
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("custom whip")
        .chars()
    {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if (ch.is_whitespace() || ch == '-' || ch == '_') && !slug.ends_with('-') {
            slug.push('-');
        }
        if slug.len() >= 32 {
            break;
        }
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "custom-whip".to_string()
    } else {
        slug.to_string()
    }
}

fn save_global_whip_instruction(
    codex_home: &Path,
    requested_name: &str,
    instructions: &str,
) -> Result<String, String> {
    validate_whip_name(requested_name)?;
    let base = normalize_whip_name(requested_name).to_string();
    let dir = codex_home.join("whips");
    fs::create_dir_all(&dir)
        .map_err(|err| format!("Failed to create `{}`: {err}", dir.display()))?;
    for suffix in 1..1000 {
        let candidate = if suffix == 1 {
            base.clone()
        } else {
            format!("{base}-{suffix}")
        };
        validate_whip_name(&candidate)?;
        let path = dir.join(format!("{candidate}.md"));
        if path.exists() || is_builtin_keep_going_name(&candidate) {
            continue;
        }
        let mut body = instructions.trim().to_string();
        body.push('\n');
        fs::write(&path, body)
            .map_err(|err| format!("Failed to write whip `{}`: {err}", path.display()))?;
        return Ok(candidate);
    }
    Err(format!("Could not find an unused whip name for `{base}`."))
}

impl App {
    pub(crate) fn handle_orchestrate_command(&mut self, args: String) {
        let trimmed = args.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("status") {
            self.open_orchestrate_status_view();
            return;
        }
        if trimmed.eq_ignore_ascii_case("attach") {
            self.open_orchestrate_target_picker();
            return;
        }
        match parse_orchestrate_command(&args) {
            Ok(command) => self.apply_orchestrate_command(command, CommandOrigin::User),
            Err(err) => self.chat_widget.add_error_message(err),
        }
    }

    pub(crate) fn open_orchestrate_status_view(&mut self) {
        let now = Utc::now();
        let mut whips: Vec<_> = self.orchestrate_whips.values().collect();
        whips.sort_by(|left, right| left.id.cmp(&right.id));

        let mut items = Vec::new();
        items.push(SelectionItem {
            name: "+ Attach whip".to_string(),
            description: Some("Pick target, duration, and instruction set.".to_string()),
            display_shortcut: Some(key_hint::plain(KeyCode::Char('n'))),
            actions: vec![Box::new(|tx| {
                tx.send(AppEvent::OpenOrchestrateTargetPicker);
            })],
            dismiss_on_select: true,
            ..Default::default()
        });

        if whips.is_empty() {
            items.push(SelectionItem {
                name: "No whips attached".to_string(),
                description: Some("Use the first row to attach one.".to_string()),
                is_disabled: true,
                ..Default::default()
            });
        } else {
            for whip in whips {
                let whip_id = whip.id.clone();
                let pause_resume_args = if whip.state == WhipState::Paused {
                    format!("resume {whip_id}")
                } else {
                    format!("pause {whip_id}")
                };
                let expiry = match whip.expires_at {
                    Some(expires_at) if expires_at <= now => "expired".to_string(),
                    Some(expires_at) => format!("expires {}", expires_at.format("%H:%MZ")),
                    None => "unlimited".to_string(),
                };
                let holder = whip
                    .holder
                    .as_deref()
                    .and_then(|node_id| self.spawn_node_title(node_id))
                    .unwrap_or_else(|| "none".to_string());
                let target = self
                    .spawn_node_title(&whip.target)
                    .unwrap_or_else(|| whip.target.clone());
                items.push(SelectionItem {
                    name: format!("{}: {}", whip.id, target),
                    description: Some(format!(
                        "{}; holder {}; {}; {}/{} fires; {}; {}",
                        whip.state.label(),
                        holder,
                        whip.mode.label(),
                        whip.fires,
                        whip.max_fires,
                        expiry,
                        whip.instructions
                    )),
                    search_value: Some(format!(
                        "{} {} {} {}",
                        whip.id, target, holder, whip.instructions
                    )),
                    is_current: whip.state == WhipState::Armed,
                    actions: vec![Box::new({
                        let whip_id = whip_id.clone();
                        move |tx| {
                            tx.send(AppEvent::OpenOrchestrateWhipDetails {
                                whip_id: whip_id.clone(),
                            });
                        }
                    })],
                    selected_shortcuts: vec![
                        orchestrate_shortcut_command('d', format!("detach {whip_id}")),
                        orchestrate_shortcut_command('p', pause_resume_args),
                        orchestrate_shortcut_extend('e', whip_id.clone()),
                        orchestrate_shortcut_command('f', format!("fire {whip_id}")),
                        orchestrate_shortcut_command('t', format!("test {whip_id}")),
                    ],
                    dismiss_on_select: false,
                    ..Default::default()
                });
            }
        }

        self.chat_widget.show_selection_view(SelectionViewParams {
            view_id: Some("orchestrate-status"),
            title: Some("Orchestrate".to_string()),
            subtitle: Some("Whips attach follow-up instructions to idle panes.".to_string()),
            footer_hint: Some(Line::from(
                "Enter details · d detach · p pause/resume · e extend · f fire · t test · n new whip",
            )),
            items,
            ..Default::default()
        });
    }

    pub(crate) fn open_orchestrate_target_picker(&mut self) {
        let holder_node = self.current_holder_node().ok();
        let entries = self.orchestrate_target_entries();
        let mut items = Vec::new();
        for entry in entries {
            let target = entry.target.clone();
            let disabled_reason = if holder_node.as_deref() == Some(entry.node_id.as_str()) {
                Some("Current pane cannot hold a review whip for itself.".to_string())
            } else {
                None
            };
            items.push(SelectionItem {
                name: entry.name.clone(),
                description: Some(entry.description.clone()),
                is_current: entry.is_current,
                disabled_reason,
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::OpenOrchestrateDurationPicker {
                        target: target.clone(),
                    });
                })],
                dismiss_on_select: true,
                search_value: Some(format!(
                    "{} {} {}",
                    entry.name, entry.description, entry.target
                )),
                ..Default::default()
            });
        }
        if items.is_empty() {
            items.push(SelectionItem {
                name: "No target panes available".to_string(),
                description: Some("Create a Codex or Claude pane first.".to_string()),
                is_disabled: true,
                ..Default::default()
            });
        }

        self.chat_widget.show_selection_view(SelectionViewParams {
            view_id: Some("orchestrate-target"),
            title: Some("Attach Whip - Target".to_string()),
            subtitle: Some("Choose the pane that should be checked when it goes idle.".to_string()),
            footer_hint: Some(standard_popup_hint_line()),
            items,
            is_searchable: true,
            search_placeholder: Some("Search panes".to_string()),
            ..Default::default()
        });
    }

    pub(crate) fn open_orchestrate_duration_picker(&mut self, target: String) {
        let choices = [
            ("15 minutes", "15m", "Short follow-up check."),
            ("1 hour", "1h", "Default short watchdog window."),
            ("4 hours", "4h", "Matches the CLI default."),
            ("8 hours", "8h", "Long-running local work."),
            (
                "Unlimited",
                "unlimited",
                "Stays armed until detached or exhausted.",
            ),
        ];
        let items = choices
            .into_iter()
            .map(|(label, duration_arg, description)| {
                let target = target.clone();
                let duration_arg = duration_arg.to_string();
                let duration_label = label.to_string();
                SelectionItem {
                    name: label.to_string(),
                    description: Some(description.to_string()),
                    is_default: duration_arg == "1h",
                    actions: vec![Box::new(move |tx| {
                        tx.send(AppEvent::OpenOrchestrateWhipPicker {
                            target: target.clone(),
                            duration_arg: duration_arg.clone(),
                            duration_label: duration_label.clone(),
                        });
                    })],
                    dismiss_on_select: true,
                    ..Default::default()
                }
            })
            .collect();

        self.chat_widget.show_selection_view(SelectionViewParams {
            view_id: Some("orchestrate-duration"),
            title: Some("Attach Whip - Duration".to_string()),
            subtitle: Some(format!(
                "Target: {}",
                self.orchestrate_target_label(&target)
            )),
            footer_hint: Some(standard_popup_hint_line()),
            items,
            ..Default::default()
        });
    }

    pub(crate) fn open_orchestrate_whip_picker(
        &mut self,
        target: String,
        duration_arg: String,
        duration_label: String,
    ) {
        let entries =
            available_whip_instruction_entries(self.config.codex_home.as_ref(), &self.config.cwd);
        let mut items: Vec<SelectionItem> = entries
            .into_iter()
            .map(|entry| {
                let target = target.clone();
                let duration_arg = duration_arg.clone();
                let duration_label = duration_label.clone();
                let whip_name = entry.name.clone();
                SelectionItem {
                    name: entry.name,
                    description: Some(entry.description),
                    is_default: whip_name == BUILTIN_KEEP_GOING_WHIP,
                    actions: vec![Box::new(move |tx| {
                        tx.send(AppEvent::OpenOrchestrateConfirm {
                            target: target.clone(),
                            duration_arg: duration_arg.clone(),
                            duration_label: duration_label.clone(),
                            whip_name: whip_name.clone(),
                        });
                    })],
                    dismiss_on_select: true,
                    ..Default::default()
                }
            })
            .collect();
        items.push(SelectionItem {
            name: "Write new...".to_string(),
            description: Some("Type standing instructions now and save them globally.".to_string()),
            actions: vec![Box::new({
                let target = target.clone();
                let duration_arg = duration_arg.clone();
                let duration_label = duration_label.clone();
                move |tx| {
                    tx.send(AppEvent::OpenOrchestrateWriteWhipPrompt {
                        target: target.clone(),
                        duration_arg: duration_arg.clone(),
                        duration_label: duration_label.clone(),
                    });
                }
            })],
            dismiss_on_select: true,
            ..Default::default()
        });

        self.chat_widget.show_selection_view(SelectionViewParams {
            view_id: Some("orchestrate-whip"),
            title: Some("Attach Whip - Instructions".to_string()),
            subtitle: Some(format!(
                "{} for {}",
                self.orchestrate_target_label(&target),
                duration_label
            )),
            footer_hint: Some(standard_popup_hint_line()),
            items,
            is_searchable: true,
            search_placeholder: Some("Search whips".to_string()),
            ..Default::default()
        });
    }

    pub(crate) fn open_orchestrate_confirm(
        &mut self,
        target: String,
        duration_arg: String,
        duration_label: String,
        whip_name: String,
    ) {
        let target_label = self.orchestrate_target_label(&target);
        let replacement =
            self.resolve_orchestrate_target_node(&target)
                .ok()
                .and_then(|target_node| {
                    self.orchestrate_whips
                        .values()
                        .find(|whip| {
                            whip.target == target_node && whip.state != WhipState::Detached
                        })
                        .map(|whip| whip.id.clone())
                });
        let attach_label = if let Some(id) = replacement.as_deref() {
            format!("Attach and replace {id}")
        } else {
            "Attach whip".to_string()
        };
        let mut items = vec![SelectionItem {
            name: attach_label,
            description: Some(format!(
                "{} -> {}; review mode; max {}; cooldown {}s",
                whip_name, target_label, DEFAULT_MAX_FIRES, DEFAULT_COOLDOWN_S
            )),
            actions: vec![Box::new({
                let args = orchestrate_guided_attach_args(&target, &duration_arg, &whip_name);
                move |tx| {
                    tx.send(AppEvent::HandleOrchestrateCommand { args: args.clone() });
                }
            })],
            dismiss_on_select: true,
            ..Default::default()
        }];
        items.push(SelectionItem {
            name: "Cancel".to_string(),
            description: Some("Leave whips unchanged.".to_string()),
            actions: Vec::new(),
            dismiss_on_select: true,
            ..Default::default()
        });

        self.chat_widget.show_selection_view(SelectionViewParams {
            view_id: Some("orchestrate-confirm"),
            title: Some("Attach Whip - Confirm".to_string()),
            subtitle: Some(format!(
                "{} on {} for {}",
                whip_name, target_label, duration_label
            )),
            footer_hint: Some(standard_popup_hint_line()),
            items,
            ..Default::default()
        });
    }

    pub(crate) fn open_orchestrate_write_whip_prompt(
        &mut self,
        target: String,
        duration_arg: String,
        duration_label: String,
    ) {
        let tx = self.app_event_tx.clone();
        let template = "Continue the assigned work.\n\nIf the target is genuinely finished, report done and emit WHIP_DONE.".to_string();
        let view = CustomPromptView::new(
            "Write Whip Instructions".to_string(),
            "Standing instructions for follow-up turns".to_string(),
            template,
            Some(format!(
                "{} for {}",
                self.orchestrate_target_label(&target),
                duration_label
            )),
            Box::new(move |instructions: String| {
                tx.send(AppEvent::OpenOrchestrateSaveWhipPrompt {
                    target: target.clone(),
                    duration_arg: duration_arg.clone(),
                    duration_label: duration_label.clone(),
                    instructions,
                });
            }),
        )
        .with_submit_mode(CustomPromptSubmitMode::CtrlD);
        self.chat_widget.show_custom_prompt_view(view);
    }

    pub(crate) fn open_orchestrate_save_whip_prompt(
        &mut self,
        target: String,
        duration_arg: String,
        duration_label: String,
        instructions: String,
    ) {
        let tx = self.app_event_tx.clone();
        let suggested_name = suggested_whip_name(&instructions);
        let view = CustomPromptView::new(
            "Save Whip As".to_string(),
            "Basename, for example keep-going".to_string(),
            suggested_name,
            Some("Saved under ~/.pfterminal/whips".to_string()),
            Box::new(move |requested_name: String| {
                tx.send(AppEvent::SaveOrchestrateWhipAndConfirm {
                    target: target.clone(),
                    duration_arg: duration_arg.clone(),
                    duration_label: duration_label.clone(),
                    requested_name,
                    instructions: instructions.clone(),
                });
            }),
        );
        self.chat_widget.show_custom_prompt_view(view);
    }

    pub(crate) fn save_orchestrate_whip_and_open_confirm(
        &mut self,
        target: String,
        duration_arg: String,
        duration_label: String,
        requested_name: String,
        instructions: String,
    ) {
        match save_global_whip_instruction(
            self.config.codex_home.as_ref(),
            &requested_name,
            &instructions,
        ) {
            Ok(whip_name) => {
                self.chat_widget.add_info_message(
                    format!("Saved whip `{whip_name}`."),
                    Some("Saved globally under ~/.pfterminal/whips.".to_string()),
                );
                self.open_orchestrate_confirm(target, duration_arg, duration_label, whip_name);
            }
            Err(err) => self.chat_widget.add_error_message(err),
        }
    }

    pub(crate) fn open_orchestrate_extend_duration_picker(&mut self, whip_id: String) {
        let choices = [
            ("15 minutes", "15m", "Add a short follow-up window."),
            ("1 hour", "1h", "Add one working window."),
            ("4 hours", "4h", "Add the default budget window."),
            ("8 hours", "8h", "Add an overnight window."),
        ];
        let items = choices
            .into_iter()
            .map(|(label, duration_arg, description)| {
                let args = format!("extend {whip_id} {duration_arg}");
                SelectionItem {
                    name: label.to_string(),
                    description: Some(description.to_string()),
                    actions: vec![Box::new(move |tx| {
                        tx.send(AppEvent::HandleOrchestrateCommand { args: args.clone() });
                    })],
                    dismiss_on_select: true,
                    ..Default::default()
                }
            })
            .collect();

        self.chat_widget.show_selection_view(SelectionViewParams {
            view_id: Some("orchestrate-extend"),
            title: Some("Extend Whip".to_string()),
            subtitle: Some(whip_id),
            footer_hint: Some(standard_popup_hint_line()),
            items,
            ..Default::default()
        });
    }

    pub(crate) fn open_orchestrate_whip_details(&mut self, whip_id: String) {
        let Some(whip) = self.orchestrate_whips.get(&whip_id).cloned() else {
            self.chat_widget
                .add_error_message(format!("No whip found for `{whip_id}`."));
            return;
        };
        let target = self.node_label(&whip.target);
        let holder = whip
            .holder
            .as_deref()
            .map(|holder| self.node_label(holder))
            .unwrap_or_else(|| "none".to_string());
        let expiry = match whip.expires_at {
            Some(expires_at) if expires_at <= Utc::now() => "expired".to_string(),
            Some(expires_at) => format!("expires {}", expires_at.format("%H:%MZ")),
            None => "unlimited".to_string(),
        };
        let pause_resume = if whip.state == WhipState::Paused {
            ("Resume", format!("resume {whip_id}"))
        } else {
            ("Pause", format!("pause {whip_id}"))
        };
        let items = vec![
            SelectionItem {
                name: pause_resume.0.to_string(),
                description: Some(format!("{} whip {}.", pause_resume.0, whip_id)),
                actions: vec![Box::new({
                    let args = pause_resume.1;
                    move |tx| tx.send(AppEvent::HandleOrchestrateCommand { args: args.clone() })
                })],
                dismiss_on_select: true,
                ..Default::default()
            },
            SelectionItem {
                name: "Extend...".to_string(),
                description: Some("Add time to the current expiry.".to_string()),
                actions: vec![Box::new({
                    let whip_id = whip_id.clone();
                    move |tx| {
                        tx.send(AppEvent::OpenOrchestrateExtendDurationPicker {
                            whip_id: whip_id.clone(),
                        });
                    }
                })],
                dismiss_on_select: true,
                ..Default::default()
            },
            orchestrate_command_item(
                "Fire now",
                "Send the whip turn immediately.",
                format!("fire {whip_id}"),
            ),
            orchestrate_command_item(
                "Test",
                "Preview the task this whip would send.",
                format!("test {whip_id}"),
            ),
            orchestrate_command_item("Detach", "Detach this whip.", format!("detach {whip_id}")),
            SelectionItem {
                name: "Attach new whip".to_string(),
                description: Some("Start the guided attach flow.".to_string()),
                display_shortcut: Some(key_hint::plain(KeyCode::Char('n'))),
                actions: vec![Box::new(|tx| {
                    tx.send(AppEvent::OpenOrchestrateTargetPicker)
                })],
                dismiss_on_select: true,
                ..Default::default()
            },
        ];

        self.chat_widget.show_selection_view(SelectionViewParams {
            view_id: Some("orchestrate-details"),
            title: Some(format!("Whip {whip_id}")),
            subtitle: Some(format!(
                "{holder} -> {target}; {}; {}/{} fires; {}; {}",
                whip.mode.label(),
                whip.fires,
                whip.max_fires,
                expiry,
                whip.instructions
            )),
            footer_hint: Some(standard_popup_hint_line()),
            items,
            ..Default::default()
        });
    }

    fn orchestrate_target_entries(&self) -> Vec<OrchestrateTargetEntry> {
        let mut entries = Vec::new();
        for (thread_id, entry) in self.agent_navigation.ordered_threads() {
            let node_id = thread_node_id(thread_id);
            let name = self.thread_label(thread_id);
            let status = if entry.is_closed {
                "done"
            } else if entry.is_running {
                "running"
            } else {
                "idle"
            };
            let mut description = format!("Codex; {status}; {}", thread_id);
            if let Some(task) = entry.last_task_message.as_deref() {
                description.push_str(&format!(
                    "; task: {}",
                    truncate_for_orchestrate_display(task, 90)
                ));
            }
            if let Some(result) = entry.last_result_message.as_deref() {
                description.push_str(&format!(
                    "; result: {}",
                    truncate_for_orchestrate_display(result, 90)
                ));
            }
            entries.push(OrchestrateTargetEntry {
                target: thread_id.to_string(),
                node_id,
                name,
                description,
                is_current: self.claude_panes.active_user_pane_id() == CODEX_MAIN_PANE_ID
                    && self.active_thread_id == Some(thread_id),
            });
        }

        for pane in self.claude_panes.panes() {
            let status = match pane.status {
                ClaudePaneStatus::Idle => "idle",
                ClaudePaneStatus::Running => "running",
            };
            let mut description = format!(
                "Claude Code {}; {status}; {}",
                pane.profile.status_model_label(),
                pane.id
            );
            if let Some(turn_status) = pane.latest_turn_status {
                description.push_str(&format!("; latest: {}", turn_status.label()));
            }
            if let Some(usage) = pane.latest_usage_summary.as_deref() {
                description.push_str(&format!(
                    "; usage: {}",
                    truncate_for_orchestrate_display(usage, 80)
                ));
            }
            if let Some(task) = pane.latest_task_message.as_deref() {
                description.push_str(&format!(
                    "; task: {}",
                    truncate_for_orchestrate_display(task, 90)
                ));
            }
            if let Some(result) = pane.latest_result_message.as_deref() {
                description.push_str(&format!(
                    "; result: {}",
                    truncate_for_orchestrate_display(result, 90)
                ));
            }
            entries.push(OrchestrateTargetEntry {
                target: pane.id.clone(),
                node_id: pane_node_id(&pane.id),
                name: pane.title.clone(),
                description,
                is_current: self.claude_panes.active_user_pane_id() == pane.id,
            });
        }

        entries.sort_by(|left, right| left.name.cmp(&right.name));
        entries
    }

    fn orchestrate_target_label(&self, target: &str) -> String {
        self.resolve_orchestrate_target_node(target)
            .ok()
            .and_then(|node_id| self.spawn_node_title(&node_id))
            .unwrap_or_else(|| target.to_string())
    }

    pub(crate) fn dispatch_orchestrate_blocks_from_text(
        &mut self,
        source_node_id: &str,
        text: &str,
    ) -> bool {
        let (_visible, blocks) = extract_orchestrate_blocks(text);
        if blocks.is_empty() {
            return false;
        }
        for block in blocks {
            self.apply_orchestrate_command(block.command, CommandOrigin::Agent(source_node_id));
        }
        true
    }

    pub(crate) fn whip_status_suffix_for_target(&self, target_node_id: &str) -> String {
        whip_suffix_for_target(&self.orchestrate_whips, target_node_id)
    }

    pub(crate) fn note_whip_target_started(&mut self, target_node_id: &str) {
        self.orchestrate_idle_generation_by_target
            .entry(target_node_id.to_string())
            .or_insert(0);
    }

    pub(crate) fn note_whip_target_idle_with_fire_control(
        &mut self,
        target_node_id: &str,
        last_output: Option<&str>,
        allow_fire: bool,
        turn_succeeded: bool,
    ) {
        let generation = self
            .orchestrate_idle_generation_by_target
            .entry(target_node_id.to_string())
            .or_insert(0);
        *generation = generation.saturating_add(1);
        let generation = *generation;
        self.pause_matching_whips_on_stop_marker(target_node_id, last_output);
        self.pause_spinning_whips_on_empty_output(target_node_id, last_output);
        self.pause_spinning_whips_on_failed_turn(target_node_id, turn_succeeded);
        self.note_whip_holder_idle(target_node_id);
        if allow_fire {
            self.evaluate_whips_for_target(target_node_id, generation, FireTrigger::Edge);
        }
    }

    pub(crate) fn note_whip_holder_dispatched(
        &mut self,
        holder_node_id: &str,
        target_node_id: &str,
    ) {
        let holder_node_id = normalize_orchestrate_node_id(holder_node_id);
        for whip in self.orchestrate_whips.values_mut().filter(|whip| {
            whip.mode == WhipMode::Review
                && whip.holder.as_deref() == Some(holder_node_id.as_str())
                && whip.target == target_node_id
        }) {
            whip.pending_review_fire = None;
            whip.ignored_review_fires = 0;
        }
    }

    pub(crate) fn sweep_orchestrate_whips(&mut self) {
        let now = Utc::now();
        let ids: Vec<String> = self.orchestrate_whips.keys().cloned().collect();
        for id in ids {
            self.expire_whip_if_needed(&id, now);
        }
        let target_generations: Vec<(String, u64)> = self
            .orchestrate_whips
            .values()
            .filter(|whip| whip.state == WhipState::Armed)
            .map(|whip| {
                let generation = self
                    .orchestrate_idle_generation_by_target
                    .get(&whip.target)
                    .copied()
                    .unwrap_or(0);
                (whip.target.clone(), generation)
            })
            .collect();
        for (target, generation) in target_generations {
            self.evaluate_whips_for_target(&target, generation, FireTrigger::Tick);
        }
    }

    fn apply_orchestrate_command(
        &mut self,
        command: OrchestrateCommand,
        origin: CommandOrigin<'_>,
    ) {
        if let Err(err) = self.authorize_orchestrate_command(&command, origin) {
            self.chat_widget.add_error_message(err);
            return;
        }
        match command {
            OrchestrateCommand::Status => self.chat_widget.add_info_message(
                format_whip_status(&self.orchestrate_whips, Utc::now()),
                None,
            ),
            OrchestrateCommand::Attach {
                target,
                whip_name,
                mode,
                expiry,
                max_fires,
                cooldown_s,
                holder,
            } => {
                match self.attach_whip(
                    target, whip_name, mode, expiry, max_fires, cooldown_s, holder, origin,
                ) {
                    Ok(message) => self.chat_widget.add_info_message(message, None),
                    Err(err) => self.chat_widget.add_error_message(err),
                }
            }
            OrchestrateCommand::Detach(id_or_target) => {
                self.set_whip_state_by_ref(&id_or_target, WhipState::Detached, "detached")
            }
            OrchestrateCommand::Pause(id_or_target) => {
                self.set_whip_state_by_ref(&id_or_target, WhipState::Paused, "paused")
            }
            OrchestrateCommand::Resume(id_or_target) => {
                self.set_whip_state_by_ref(&id_or_target, WhipState::Armed, "resumed")
            }
            OrchestrateCommand::Extend { id, duration } => {
                let Some(whip) = self.orchestrate_whips.get_mut(&id) else {
                    self.chat_widget
                        .add_error_message(format!("No whip found for `{id}`."));
                    return;
                };
                let base = whip
                    .expires_at
                    .filter(|expiry| *expiry > Utc::now())
                    .unwrap_or_else(Utc::now);
                whip.expires_at = Some(base + Duration::seconds(duration.seconds));
                whip.expiry_notified = false;
                let expires = whip
                    .expires_at
                    .map(|value| value.to_rfc3339())
                    .unwrap_or_else(|| "unlimited".to_string());
                self.persist_pane_state();
                self.chat_widget
                    .add_info_message(format!("Extended {id}; expires_at={expires}."), None);
            }
            OrchestrateCommand::Fire(id) => match self.plan_whip_fire(&id, FireTrigger::Manual) {
                Ok(plan) => self.execute_whip_fire(plan, FireTrigger::Manual),
                Err(err) => self.chat_widget.add_error_message(err),
            },
            OrchestrateCommand::Test(id) => match self.plan_whip_fire(&id, FireTrigger::Test) {
                Ok(plan) => self.chat_widget.add_info_message(
                    format!(
                        "Whip {} would send a {} turn to {}:\n{}",
                        plan.whip_id,
                        if self
                            .orchestrate_whips
                            .get(&plan.whip_id)
                            .is_some_and(|whip| whip.mode == WhipMode::Review)
                        {
                            "review"
                        } else {
                            "task"
                        },
                        plan.destination_label,
                        plan.task
                    ),
                    None,
                ),
                Err(err) => self.chat_widget.add_error_message(err),
            },
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn attach_whip(
        &mut self,
        target: String,
        whip_name: String,
        mode: Option<WhipMode>,
        expiry: Option<ExpiryArg>,
        max_fires: Option<u32>,
        cooldown_s: Option<u64>,
        holder: Option<HolderArg>,
        origin: CommandOrigin<'_>,
    ) -> Result<String, String> {
        let (instruction_path, instruction_text) = read_whip_instruction(
            self.config.codex_home.as_ref(),
            &self.config.cwd,
            &whip_name,
        )?;
        let doc_defaults = parse_whip_doc_defaults(&instruction_text);
        let target_node_id = self.resolve_orchestrate_target_node(&target)?;
        let mut resolved_mode = mode.or(doc_defaults.mode).unwrap_or(WhipMode::Review);
        let holder_arg = holder.unwrap_or_else(|| match origin {
            CommandOrigin::Agent(node) => HolderArg::Target(node.to_string()),
            CommandOrigin::User => HolderArg::Me,
        });
        let holder_node_id = match holder_arg {
            HolderArg::None => {
                resolved_mode = WhipMode::Auto;
                None
            }
            HolderArg::Me => Some(match origin {
                CommandOrigin::User => self.current_holder_node()?,
                CommandOrigin::Agent(node) => normalize_orchestrate_node_id(node),
            }),
            HolderArg::Target(value) => Some(self.resolve_orchestrate_target_node(&value)?),
        };
        if resolved_mode == WhipMode::Review && holder_node_id.is_none() {
            return Err(
                "Review-mode whips require a holder; use --holder me or --mode auto.".to_string(),
            );
        }
        if holder_node_id.as_deref() == Some(target_node_id.as_str()) {
            return Err("A whip holder cannot be the same pane as its target.".to_string());
        }
        let resolved_expiry = expiry
            .map(|value| resolve_expiry_arg(value, Utc::now()))
            .transpose()?;
        let options = ResolvedAttachOptions {
            mode: Some(resolved_mode),
            expiry: resolved_expiry,
            max_fires: max_fires.or(doc_defaults.max_fires),
            cooldown_s: cooldown_s.or(doc_defaults.cooldown_s),
            stop_marker: doc_defaults.stop_marker,
        };
        let replaced: Vec<String> = self
            .orchestrate_whips
            .values()
            .filter(|whip| whip.target == target_node_id && whip.state != WhipState::Detached)
            .map(|whip| whip.id.clone())
            .collect();
        for id in replaced {
            self.orchestrate_whips.remove(&id);
        }
        let id = self.next_whip_id();
        let whip = Whip::new(
            id.clone(),
            holder_node_id,
            target_node_id.clone(),
            whip_name.clone(),
            options,
            Utc::now(),
        );
        self.orchestrate_whips.insert(id.clone(), whip);
        self.orchestrate_idle_generation_by_target
            .entry(target_node_id)
            .or_insert(0);
        self.persist_pane_state();
        Ok(format!(
            "Attached {id} to `{target}` with `{}` ({}) from {}.",
            whip_name,
            resolved_mode.label(),
            instruction_path.display()
        ))
    }

    fn set_whip_state_by_ref(&mut self, id_or_target: &str, state: WhipState, action: &str) {
        let Some(id) = self.find_whip_id(id_or_target) else {
            self.chat_widget
                .add_error_message(format!("No whip found for `{id_or_target}`."));
            return;
        };
        let removed_target = if state == WhipState::Detached {
            self.orchestrate_whips.remove(&id).map(|whip| whip.target)
        } else {
            None
        };
        if let Some(whip) = self.orchestrate_whips.get_mut(&id) {
            whip.state = state;
            if state == WhipState::Armed {
                whip.expiry_notified = false;
            }
        }
        if let Some(target) = removed_target
            && !self
                .orchestrate_whips
                .values()
                .any(|whip| whip.target == target)
        {
            self.orchestrate_idle_generation_by_target.remove(&target);
        }
        self.persist_pane_state();
        self.chat_widget
            .add_info_message(format!("Whip {id} {action}."), None);
    }

    fn evaluate_whips_for_target(
        &mut self,
        target_node_id: &str,
        idle_generation: u64,
        trigger: FireTrigger,
    ) {
        let ids: Vec<String> = self
            .orchestrate_whips
            .values()
            .filter(|whip| whip.target == target_node_id)
            .map(|whip| whip.id.clone())
            .collect();
        for id in ids {
            match self.plan_whip_fire_for_generation(&id, idle_generation, trigger) {
                Ok(plan) => self.execute_whip_fire(plan, trigger),
                Err(err) if matches!(trigger, FireTrigger::Manual | FireTrigger::Test) => {
                    self.chat_widget.add_error_message(err);
                }
                Err(_) => {}
            }
        }
    }

    fn plan_whip_fire(
        &mut self,
        id_or_target: &str,
        trigger: FireTrigger,
    ) -> Result<FirePlan, String> {
        let id = self
            .find_whip_id(id_or_target)
            .ok_or_else(|| format!("No whip found for `{id_or_target}`."))?;
        let target = self
            .orchestrate_whips
            .get(&id)
            .map(|whip| whip.target.clone())
            .ok_or_else(|| format!("No whip found for `{id}`."))?;
        let generation = self
            .orchestrate_idle_generation_by_target
            .get(&target)
            .copied()
            .unwrap_or(0);
        self.plan_whip_fire_for_generation(&id, generation, trigger)
    }

    fn plan_whip_fire_for_generation(
        &mut self,
        id: &str,
        idle_generation: u64,
        trigger: FireTrigger,
    ) -> Result<FirePlan, String> {
        self.expire_whip_if_needed(id, Utc::now());
        let whip = self
            .orchestrate_whips
            .get(id)
            .ok_or_else(|| format!("No whip found for `{id}`."))?
            .clone();
        if !whip.is_armed() {
            return Err(format!("Whip {id} is {}.", whip.state.label()));
        }
        if whip.fires >= whip.max_fires {
            self.mark_whip_terminal(id, WhipState::Exhausted, "max fires reached");
            return Err(format!("Whip {id} is exhausted."));
        }
        if !matches!(trigger, FireTrigger::Manual | FireTrigger::Test) {
            if whip.last_idle_generation_fired == Some(idle_generation) {
                return Err(format!("Whip {id} already fired for this idle period."));
            }
            if let Some(last_fire) = whip.last_fire_utc
                && Utc::now() - last_fire < Duration::seconds(whip.cooldown_s as i64)
            {
                return Err(format!("Whip {id} is inside cooldown."));
            }
        }
        if !self.target_node_is_idle(&whip.target) {
            return Err(format!("Whip target `{}` is not idle.", whip.target));
        }
        let (instruction_path, instruction_text) = read_whip_instruction(
            self.config.codex_home.as_ref(),
            &self.config.cwd,
            &whip.instructions,
        )?;
        let destination_node = match whip.mode {
            WhipMode::Auto => whip.target.clone(),
            WhipMode::Review => {
                let Some(holder) = whip.holder.clone() else {
                    return Err(format!("Whip {id} has no holder."));
                };
                if !matches!(trigger, FireTrigger::Test) && !self.target_node_is_idle(&holder) {
                    return Err(format!("Whip holder `{holder}` is not idle."));
                }
                holder
            }
        };
        let destination = self.fire_destination_for_node(&destination_node)?;
        let destination_label = self.node_label(&destination_node);
        let target_label = self.node_label(&whip.target);
        let task = match whip.mode {
            WhipMode::Auto => auto_whip_task(&whip, &instruction_text, &instruction_path),
            WhipMode::Review => {
                review_whip_task(&whip, &target_label, &instruction_text, &instruction_path)
            }
        };
        Ok(FirePlan {
            whip_id: id.to_string(),
            mode: whip.mode,
            destination,
            task,
            target_idle_generation: idle_generation,
            destination_label,
        })
    }

    fn execute_whip_fire(&mut self, plan: FirePlan, trigger: FireTrigger) {
        let (fires, max_fires, exhausted) = {
            let Some(whip) = self.orchestrate_whips.get_mut(&plan.whip_id) else {
                return;
            };
            if !matches!(trigger, FireTrigger::Test) {
                whip.fires = whip.fires.saturating_add(1);
                whip.last_fire_utc = Some(Utc::now());
                whip.last_idle_generation_fired = Some(plan.target_idle_generation);
            }
            let exhausted = whip.fires >= whip.max_fires;
            if exhausted && !matches!(trigger, FireTrigger::Test) {
                whip.state = WhipState::Exhausted;
                whip.expiry_notified = true;
            }
            if plan.mode == WhipMode::Review && !matches!(trigger, FireTrigger::Test) {
                whip.pending_review_fire = Some(whip.fires);
            }
            (whip.fires, whip.max_fires, exhausted)
        };
        match plan.destination {
            FireDestination::Native(thread_id) => {
                self.app_event_tx.send(AppEvent::SubmitSpawnAgentTask {
                    thread_id,
                    task: plan.task,
                });
            }
            FireDestination::ClaudePane(pane_id) => {
                self.app_event_tx.send(AppEvent::SubmitSpawnClaudePaneTask {
                    pane_id,
                    task: plan.task,
                });
            }
        }
        self.persist_pane_state();
        self.chat_widget.add_info_message(
            format!(
                "Whip {} fired to {} ({}/{}).",
                plan.whip_id, plan.destination_label, fires, max_fires
            ),
            None,
        );
        if exhausted && !matches!(trigger, FireTrigger::Test) {
            self.chat_widget.add_info_message(
                format!("Whip {} exhausted: max fires reached.", plan.whip_id),
                None,
            );
        }
    }

    fn pause_matching_whips_on_stop_marker(
        &mut self,
        target_node_id: &str,
        last_output: Option<&str>,
    ) {
        let Some(output) = last_output else {
            return;
        };
        let ids: Vec<String> = self
            .orchestrate_whips
            .values()
            .filter(|whip| {
                whip.target == target_node_id
                    && whip.state == WhipState::Armed
                    && output.contains(&whip.stop_marker)
            })
            .map(|whip| whip.id.clone())
            .collect();
        for id in ids {
            self.mark_whip_terminal(&id, WhipState::Paused, "stop marker seen");
        }
    }

    fn pause_spinning_whips_on_empty_output(
        &mut self,
        target_node_id: &str,
        last_output: Option<&str>,
    ) {
        let Some(output) = last_output else {
            return;
        };
        let mut paused = Vec::new();
        for whip in self
            .orchestrate_whips
            .values_mut()
            .filter(|whip| whip.target == target_node_id && whip.state == WhipState::Armed)
        {
            if output.trim().is_empty() {
                whip.empty_output_fires = whip.empty_output_fires.saturating_add(1);
            } else {
                whip.empty_output_fires = 0;
            }
            if whip.empty_output_fires >= 2 {
                paused.push(whip.id.clone());
            }
        }
        for id in paused {
            self.mark_whip_terminal(&id, WhipState::Paused, "empty output loop");
        }
    }

    fn pause_spinning_whips_on_failed_turn(
        &mut self,
        target_node_id: &str,
        turn_succeeded: bool,
    ) {
        let mut paused = Vec::new();
        for whip in self
            .orchestrate_whips
            .values_mut()
            .filter(|whip| whip.target == target_node_id && whip.state == WhipState::Armed)
        {
            if turn_succeeded {
                whip.consecutive_failed_turns = 0;
            } else {
                whip.consecutive_failed_turns = whip.consecutive_failed_turns.saturating_add(1);
            }
            if whip.consecutive_failed_turns >= 2 {
                paused.push(whip.id.clone());
            }
        }
        for id in paused {
            self.mark_whip_terminal(&id, WhipState::Paused, "two consecutive failed turns");
        }
    }

    fn note_whip_holder_idle(&mut self, holder_node_id: &str) {
        let holder_node_id = normalize_orchestrate_node_id(holder_node_id);
        let mut pause = Vec::new();
        for whip in self.orchestrate_whips.values_mut().filter(|whip| {
            whip.mode == WhipMode::Review
                && whip.state == WhipState::Armed
                && whip.holder.as_deref() == Some(holder_node_id.as_str())
                && whip.pending_review_fire.is_some()
        }) {
            whip.pending_review_fire = None;
            whip.ignored_review_fires = whip.ignored_review_fires.saturating_add(1);
            if whip.ignored_review_fires >= 2 {
                pause.push(whip.id.clone());
            }
        }
        for id in pause {
            self.mark_whip_terminal(&id, WhipState::Paused, "holder ignored two review fires");
        }
    }

    fn expire_whip_if_needed(&mut self, id: &str, now: DateTime<Utc>) {
        let should_expire = self.orchestrate_whips.get(id).is_some_and(|whip| {
            whip.state == WhipState::Armed
                && whip.expires_at.is_some_and(|expires_at| expires_at <= now)
        });
        if should_expire {
            self.mark_whip_terminal(id, WhipState::Expired, "expired");
        }
    }

    fn authorize_orchestrate_command(
        &self,
        command: &OrchestrateCommand,
        origin: CommandOrigin<'_>,
    ) -> Result<(), String> {
        let CommandOrigin::Agent(agent_node) = origin else {
            return Ok(());
        };
        let agent_node = normalize_orchestrate_node_id(agent_node);
        match command {
            OrchestrateCommand::Status => Ok(()),
            OrchestrateCommand::Pause(id_or_target) | OrchestrateCommand::Detach(id_or_target) => {
                self.ensure_agent_controls_whip(id_or_target, &agent_node)
            }
            OrchestrateCommand::Extend { id, .. } => {
                self.ensure_agent_controls_whip(id, &agent_node)
            }
            OrchestrateCommand::Attach { target, expiry, .. } => {
                self.ensure_agent_attach_expiry_allowed(*expiry)?;
                let target_node_id = self.resolve_orchestrate_target_node(target)?;
                for whip in self.orchestrate_whips.values().filter(|whip| {
                    whip.target == target_node_id && whip.state != WhipState::Detached
                }) {
                    match whip.holder.as_deref() {
                        Some(holder) if holder == agent_node => {}
                        Some(holder) => {
                            return Err(format!(
                                "Agent `{agent_node}` cannot replace whip {} held by `{holder}`.",
                                whip.id
                            ));
                        }
                        None => {
                            return Err(format!(
                                "Agent `{agent_node}` cannot replace user-owned holderless whip {}.",
                                whip.id
                            ));
                        }
                    }
                }
                Ok(())
            }
            OrchestrateCommand::Resume(_)
            | OrchestrateCommand::Fire(_)
            | OrchestrateCommand::Test(_) => {
                Err("Only the user can resume, fire, or test whips.".to_string())
            }
        }
    }

    fn ensure_agent_controls_whip(
        &self,
        id_or_target: &str,
        agent_node: &str,
    ) -> Result<(), String> {
        let id = self
            .find_whip_id(id_or_target)
            .ok_or_else(|| format!("No whip found for `{id_or_target}`."))?;
        let Some(whip) = self.orchestrate_whips.get(&id) else {
            return Err(format!("No whip found for `{id}`."));
        };
        if whip.holder.as_deref() == Some(agent_node) || whip.target == agent_node {
            return Ok(());
        }
        Err(format!(
            "Agent `{agent_node}` cannot control whip {id}; it neither holds nor targets that whip."
        ))
    }

    fn ensure_agent_attach_expiry_allowed(&self, expiry: Option<ExpiryArg>) -> Result<(), String> {
        let Some(expiry) = expiry else {
            return Ok(());
        };
        match expiry {
            ExpiryArg::Unlimited => Err("Agent-origin whips cannot be unlimited.".to_string()),
            ExpiryArg::Duration(duration) if duration.seconds > DEFAULT_EXPIRY_SECONDS => {
                Err("Agent-origin whips cannot request a duration longer than 4h.".to_string())
            }
            ExpiryArg::UntilTodayOrTomorrow { .. } => {
                let now = Utc::now();
                let Some(expires_at) = resolve_expiry_arg(expiry, now)? else {
                    return Err("Agent-origin whips cannot be unlimited.".to_string());
                };
                if expires_at - now > Duration::seconds(DEFAULT_EXPIRY_SECONDS) {
                    return Err(
                        "Agent-origin whips cannot request a duration longer than 4h.".to_string(),
                    );
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn mark_whip_terminal(&mut self, id: &str, state: WhipState, reason: &str) {
        let Some(whip) = self.orchestrate_whips.get_mut(id) else {
            return;
        };
        if whip.state == state && whip.expiry_notified {
            return;
        }
        whip.state = state;
        whip.expiry_notified = true;
        self.persist_pane_state();
        self.chat_widget
            .add_info_message(format!("Whip {id} {}: {reason}.", state.label()), None);
    }

    fn resolve_orchestrate_target_node(&self, target: &str) -> Result<String, String> {
        if (target == CODEX_MAIN_PANE_ID || target == pane_node_id(CODEX_MAIN_PANE_ID))
            && let Some(thread_id) = self.primary_thread_id
        {
            return Ok(thread_node_id(thread_id));
        }
        match self.resolve_spawn_task_target(target)? {
            SpawnTaskTarget::Native(thread_id) | SpawnTaskTarget::UnavailableNative(thread_id) => {
                Ok(thread_node_id(thread_id))
            }
            SpawnTaskTarget::ClaudePane(pane_id) => Ok(pane_node_id(&pane_id)),
        }
    }

    fn fire_destination_for_node(&self, node_id: &str) -> Result<FireDestination, String> {
        if let Some(thread_id) = node_id_thread(node_id) {
            return Ok(FireDestination::Native(thread_id));
        }
        if let Some(pane_id) = node_id_pane(node_id) {
            if pane_id == CODEX_MAIN_PANE_ID {
                return self
                    .primary_thread_id
                    .map(FireDestination::Native)
                    .ok_or_else(|| "Codex Main is not loaded.".to_string());
            }
            if self
                .claude_panes
                .panes()
                .iter()
                .any(|pane| pane.id == pane_id)
            {
                return Ok(FireDestination::ClaudePane(pane_id.to_string()));
            }
        }
        Err(format!("Whip destination `{node_id}` is not loaded."))
    }

    fn current_holder_node(&self) -> Result<String, String> {
        let active_pane = self.claude_panes.active_user_pane_id();
        if active_pane != CODEX_MAIN_PANE_ID {
            return Ok(pane_node_id(active_pane));
        }
        self.active_thread_id
            .or(self.primary_thread_id)
            .map(thread_node_id)
            .ok_or_else(|| "No current Codex pane is available as whip holder.".to_string())
    }

    fn target_node_is_idle(&self, node_id: &str) -> bool {
        if let Some(thread_id) = node_id_thread(node_id) {
            return self
                .agent_navigation
                .get(&thread_id)
                .is_some_and(|entry| !entry.is_running && !entry.is_closed);
        }
        if let Some(pane_id) = node_id_pane(node_id) {
            if pane_id == CODEX_MAIN_PANE_ID {
                return self
                    .primary_thread_id
                    .and_then(|thread_id| self.agent_navigation.get(&thread_id))
                    .is_some_and(|entry| !entry.is_running && !entry.is_closed);
            }
            return self
                .claude_panes
                .panes()
                .iter()
                .find(|pane| pane.id == pane_id)
                .is_some_and(|pane| pane.status != crate::claude_panes::ClaudePaneStatus::Running);
        }
        false
    }

    fn node_label(&self, node_id: &str) -> String {
        self.spawn_node_title(node_id)
            .unwrap_or_else(|| node_id.to_string())
    }

    fn next_whip_id(&mut self) -> String {
        self.orchestrate_next_whip_seq = self.orchestrate_next_whip_seq.saturating_add(1);
        format!("whip-{}", self.orchestrate_next_whip_seq)
    }

    fn find_whip_id(&self, id_or_target: &str) -> Option<String> {
        if self.orchestrate_whips.contains_key(id_or_target) {
            return Some(id_or_target.to_string());
        }
        let resolved_target = self.resolve_orchestrate_target_node(id_or_target).ok();
        self.orchestrate_whips
            .values()
            .filter(|whip| whip.state != WhipState::Detached)
            .find(|whip| {
                resolved_target
                    .as_ref()
                    .is_some_and(|target| whip.target == *target)
                    || whip.target == id_or_target
            })
            .map(|whip| whip.id.clone())
    }
}

#[derive(Debug, Clone, Copy)]
enum CommandOrigin<'a> {
    User,
    Agent(&'a str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FireTrigger {
    Edge,
    Tick,
    Manual,
    Test,
}

fn auto_whip_task(whip: &Whip, instructions: &str, path: &Path) -> String {
    format!(
        "Whip #{} fire {}/{} ({})\nInstruction source: {}\n\n{}",
        whip.id,
        whip.fires.saturating_add(1),
        whip.max_fires,
        Utc::now().to_rfc3339(),
        path.display(),
        instructions.trim()
    )
}

fn normalize_orchestrate_node_id(node_or_pane_id: &str) -> String {
    if node_or_pane_id.starts_with("thread:") || node_or_pane_id.starts_with("pane:") {
        node_or_pane_id.to_string()
    } else {
        pane_node_id(node_or_pane_id)
    }
}

fn review_whip_task(whip: &Whip, target_label: &str, instructions: &str, path: &Path) -> String {
    format!(
        "Whip-review turn for {}.\nTarget: {}\nWhip document: {}\nFire budget: {}/{}\nTime left: {}\n\nTarget is idle. Review the target's last result and decide the next directive. If more work is needed, dispatch it through the normal pfterminal-send-task block to the target. If done, emit a pfterminal-orchestrate block to pause or detach this whip.\n\nWhip instructions:\n{}",
        whip.id,
        target_label,
        path.display(),
        whip.fires.saturating_add(1),
        whip.max_fires,
        whip.expires_at
            .map(|expires_at| expires_at.to_rfc3339())
            .unwrap_or_else(|| "unlimited".to_string()),
        instructions.trim()
    )
}

fn parse_holder_arg(value: &str) -> HolderArg {
    if value.eq_ignore_ascii_case("me") {
        HolderArg::Me
    } else if value.eq_ignore_ascii_case("none") {
        HolderArg::None
    } else {
        HolderArg::Target(value.trim().to_string())
    }
}

fn one_arg<'a>(action: &str, mut parts: impl Iterator<Item = &'a str>) -> Result<String, String> {
    let value = parts
        .next()
        .ok_or_else(|| format!("Usage: /orchestrate {action} <id|target>"))?
        .to_string();
    if parts.next().is_some() {
        return Err(format!("Usage: /orchestrate {action} <id|target>"));
    }
    Ok(value)
}

fn parse_positive_u32(value: &str, flag: &str) -> Result<u32, String> {
    let parsed = value
        .parse::<u32>()
        .map_err(|_| format!("Invalid value for {flag}: `{value}`."))?;
    if parsed == 0 {
        return Err(format!("{flag} must be greater than zero."));
    }
    Ok(parsed)
}

fn parse_duration_arg(value: &str) -> Result<DurationArg, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("Duration cannot be empty.".to_string());
    }
    let (number, multiplier) = match trimmed.chars().last().unwrap_or_default() {
        's' | 'S' => (&trimmed[..trimmed.len() - 1], 1),
        'm' | 'M' => (&trimmed[..trimmed.len() - 1], 60),
        'h' | 'H' => (&trimmed[..trimmed.len() - 1], 60 * 60),
        'd' | 'D' => (&trimmed[..trimmed.len() - 1], 24 * 60 * 60),
        _ => (trimmed, 1),
    };
    let amount = number
        .parse::<i64>()
        .map_err(|_| format!("Invalid duration `{value}`."))?;
    if amount <= 0 {
        return Err("Duration must be greater than zero.".to_string());
    }
    Ok(DurationArg {
        seconds: amount.saturating_mul(multiplier),
    })
}

fn parse_until_arg(value: &str) -> Result<ExpiryArg, String> {
    let parsed = NaiveTime::parse_from_str(value, "%H:%M")
        .map_err(|_| "Expected --until HH:MM in UTC.".to_string())?;
    Ok(ExpiryArg::UntilTodayOrTomorrow {
        hour: parsed.hour(),
        minute: parsed.minute(),
    })
}

fn resolve_expiry_arg(
    value: ExpiryArg,
    now: DateTime<Utc>,
) -> Result<Option<DateTime<Utc>>, String> {
    match value {
        ExpiryArg::Duration(duration) => Ok(Some(now + Duration::seconds(duration.seconds))),
        ExpiryArg::Unlimited => Ok(None),
        ExpiryArg::UntilTodayOrTomorrow { hour, minute } => {
            let today = now.date_naive();
            let Some(naive_time) = NaiveTime::from_hms_opt(hour, minute, 0) else {
                return Err("Invalid --until time.".to_string());
            };
            let mut expiry = today.and_time(naive_time).and_utc();
            if expiry <= now {
                expiry += Duration::days(1);
            }
            Ok(Some(expiry))
        }
    }
}

fn parse_whip_doc_defaults(text: &str) -> WhipDocDefaults {
    let mut defaults = WhipDocDefaults {
        mode: None,
        max_fires: None,
        cooldown_s: None,
        stop_marker: None,
    };
    let trimmed = text.trim_start();
    if !trimmed.starts_with("---") {
        return defaults;
    }
    let mut lines = trimmed.lines();
    let _ = lines.next();
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            match key.trim().to_ascii_lowercase().as_str() {
                "mode" => defaults.mode = WhipMode::parse(value).ok(),
                "max_fires" | "max" => defaults.max_fires = value.trim().parse::<u32>().ok(),
                "cooldown_s" | "cooldown" => {
                    defaults.cooldown_s = parse_duration_arg(value)
                        .ok()
                        .map(|duration| duration.seconds.max(0) as u64)
                }
                "stop_marker" => defaults.stop_marker = Some(value.trim().to_string()),
                _ => {}
            }
        }
    }
    defaults
}

fn yamlish_fields(content: &str) -> HashMap<String, String> {
    let mut fields = HashMap::new();
    for line in content.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        fields.insert(key.trim().to_ascii_lowercase(), value.trim().to_string());
    }
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_attach_command_with_bounds() {
        let parsed = parse_orchestrate_command(
            "attach Krimp quant --mode auto --for 2h --max 7 --cooldown 30s --holder none",
        )
        .expect("parse");

        assert_eq!(
            parsed,
            OrchestrateCommand::Attach {
                target: "Krimp".to_string(),
                whip_name: "quant".to_string(),
                mode: Some(WhipMode::Auto),
                expiry: Some(ExpiryArg::Duration(DurationArg { seconds: 7200 })),
                max_fires: Some(7),
                cooldown_s: Some(30),
                holder: Some(HolderArg::None),
            }
        );
    }

    #[test]
    fn extracts_orchestrate_fenced_blocks_from_visible_text() {
        let text = "before\n```pfterminal-orchestrate\naction: attach\ntarget: Krimp\nwhip: quant\nmode: auto\n```\nafter";

        let (visible, blocks) = extract_orchestrate_blocks(text);

        assert_eq!(visible, "before\n\nafter");
        assert_eq!(blocks.len(), 1);
        assert!(matches!(
            &blocks[0].command,
            OrchestrateCommand::Attach { target, whip_name, mode, .. }
                if target == "Krimp" && whip_name == "quant" && *mode == Some(WhipMode::Auto)
        ));
    }

    #[test]
    fn doc_frontmatter_overrides_defaults() {
        let defaults = parse_whip_doc_defaults(
            "---\nmode: auto\nmax_fires: 3\ncooldown_s: 2m\nstop_marker: DONE\n---\n# whip: x",
        );

        assert_eq!(defaults.mode, Some(WhipMode::Auto));
        assert_eq!(defaults.max_fires, Some(3));
        assert_eq!(defaults.cooldown_s, Some(120));
        assert_eq!(defaults.stop_marker.as_deref(), Some("DONE"));
    }

    #[test]
    fn guided_attach_args_parse_to_attach_command() {
        let args = orchestrate_guided_attach_args("claude-123", "1h", BUILTIN_KEEP_GOING_WHIP);
        let parsed = parse_orchestrate_command(&args).expect("parse");

        assert_eq!(
            parsed,
            OrchestrateCommand::Attach {
                target: "claude-123".to_string(),
                whip_name: BUILTIN_KEEP_GOING_WHIP.to_string(),
                mode: Some(WhipMode::Review),
                expiry: Some(ExpiryArg::Duration(DurationArg { seconds: 3600 })),
                max_fires: Some(DEFAULT_MAX_FIRES),
                cooldown_s: Some(DEFAULT_COOLDOWN_S),
                holder: Some(HolderArg::Me),
            }
        );
    }

    #[test]
    fn builtin_keep_going_is_available_without_file() {
        let codex_home = tempfile::tempdir().expect("codex home");
        let cwd = tempfile::tempdir().expect("cwd");

        let (path, contents) =
            read_whip_instruction(codex_home.path(), cwd.path(), BUILTIN_KEEP_GOING_WHIP)
                .expect("builtin whip");

        assert_eq!(path, PathBuf::from(BUILTIN_KEEP_GOING_PATH));
        assert_eq!(contents, BUILTIN_KEEP_GOING_INSTRUCTION);
    }

    #[test]
    fn global_keep_going_file_overrides_builtin() {
        let codex_home = tempfile::tempdir().expect("codex home");
        let cwd = tempfile::tempdir().expect("cwd");
        let global_whips = codex_home.path().join("whips");
        fs::create_dir_all(&global_whips).expect("global whips dir");
        let global_path = global_whips.join("keep-going.md");
        fs::write(&global_path, "global override").expect("write global whip");

        let (path, contents) =
            read_whip_instruction(codex_home.path(), cwd.path(), BUILTIN_KEEP_GOING_WHIP)
                .expect("global whip");

        assert_eq!(path, global_path);
        assert_eq!(contents, "global override");
    }

    #[test]
    fn project_keep_going_file_overrides_global_file() {
        let codex_home = tempfile::tempdir().expect("codex home");
        let cwd = tempfile::tempdir().expect("cwd");
        let global_whips = codex_home.path().join("whips");
        fs::create_dir_all(&global_whips).expect("global whips dir");
        fs::write(global_whips.join("keep-going.md"), "global override")
            .expect("write global whip");
        let project_whips = cwd.path().join(".pfterminal").join("whips");
        fs::create_dir_all(&project_whips).expect("project whips dir");
        let project_path = project_whips.join("keep-going.md");
        fs::write(&project_path, "project override").expect("write project whip");

        let (path, contents) =
            read_whip_instruction(codex_home.path(), cwd.path(), "keep-going.md")
                .expect("project whip");

        assert_eq!(path, project_path);
        assert_eq!(contents, "project override");
    }

    #[test]
    fn available_whip_entries_include_builtin_once() {
        let codex_home = tempfile::tempdir().expect("codex home");
        let cwd = tempfile::tempdir().expect("cwd");
        let entries = available_whip_instruction_entries(codex_home.path(), cwd.path());

        assert_eq!(
            entries
                .iter()
                .filter(|entry| entry.name == BUILTIN_KEEP_GOING_WHIP)
                .count(),
            1
        );
        assert!(
            entries
                .iter()
                .any(|entry| entry.description.starts_with("built-in:"))
        );
    }
}
