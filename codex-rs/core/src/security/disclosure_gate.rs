//! PF-28-S01 disclosure gate for Core (feature `secret_output_gate`).
//!
//! Arms the process-wide [`output_gate`] with Core's own managed values and
//! gates Core's sinks: model requests (turns, compaction), recorded history
//! (tool results and model output), transcript (rollout) items and client
//! events (including streamed deltas and errors, and again at delivery so
//! events sent on a cloned sender are covered too). While armed, rollout
//! traces and shell snapshots are off and tool-result telemetry is gated. The
//! TUI log file, feedback buffer, log database and prompt history are gated
//! in their own crates through the same registry.
//!
//! Managed values are captured at arm time (secret-looking environment
//! variables, URL passwords, sign-in files and secret-named config values),
//! whenever the sign-in store loads or saves tokens (login, refresh, keyring)
//! and when this process reveals a vault credential. Raw values stay in this
//! process.

use codex_protocol::protocol::EventMsg;
use codex_protocol::secretless_launch;
use codex_secret_broker::output_gate;
use codex_secret_broker::output_gate::OutputGate;
use codex_secret_broker::output_gate::OutputSink;
use codex_secret_broker::output_gate::SecretClass;
use codex_secret_broker::output_gate::StreamScrubber;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::path::Path;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::OnceLock;
use zeroize::Zeroizing;

/// Environment values shorter than this are flags, not credentials.
const MIN_ENV_VALUE_BYTES: usize = 8;
/// Open streams tracked at once; the oldest is dropped (its held-back tail is
/// simply not shown; the completed item carries the full gated text).
const MAX_OPEN_STREAMS: usize = 512;

/// Secret-looking names whose values are session plumbing, not credentials.
const NON_CREDENTIAL_ENV_PREFIXES: &[&str] = &["XDG_", "GNOME_KEYRING_", "DBUS_"];
const NON_CREDENTIAL_ENV_NAMES: &[&str] = &[
    "DESKTOP_SESSION",
    "SESSION_MANAGER",
    "XAUTHORITY",
    "KEYTIMEOUT",
];
const NON_CREDENTIAL_ENV_SUFFIXES: &[&str] = &[
    "_SOCK",
    "_PID",
    "_ADDR",
    "_PATH",
    "_DIR",
    "_FILE",
    "_URL",
    "_URI",
    "_HOST",
    "_USER",
    "_USERNAME",
    "_NAME",
    "_EMAIL",
    "_REGION",
    "_ENDPOINT",
];
/// Fragments that make a secret-looking name ordinary (`GIT_AUTHOR_NAME`
/// contains `AUTH`; `KEYBOARD` contains `KEY`).
const NON_CREDENTIAL_ENV_FRAGMENTS: &[&str] = &["AUTHOR", "KEYBOARD", "KEYMAP", "KEYCHAIN"];
/// Values at least this long need no mix of character classes.
const LONG_SECRET_BYTES: usize = 16;

/// Key fragments that mark a value in a sign-in or config file as secret.
const SECRET_KEY_FRAGMENTS: &[&str] = &[
    "TOKEN",
    "SECRET",
    "PASSWORD",
    "PASSWD",
    "API_KEY",
    "APIKEY",
    "AUTHORIZATION",
    "CREDENTIAL",
    "PRIVATE_KEY",
    "ACCESS_KEY",
    "COOKIE",
];

/// Sign-in files under `CODEX_HOME` whose secret-named values are managed.
const SIGN_IN_FILES: &[&str] = &["auth.json", "provider_auth.json", ".credentials.json"];

static SEEDED: OnceLock<Result<(), String>> = OnceLock::new();

/// Arms the gate once per process with Core's managed values. Fails closed:
/// if the values cannot all be admitted the session must not start.
pub(crate) fn arm(codex_home: &Path) -> Result<(), String> {
    SEEDED
        .get_or_init(|| {
            let mut values = env_values(std::env::vars_os());
            for file in SIGN_IN_FILES {
                values.extend(json_file_values(&codex_home.join(file), file));
            }
            values.extend(toml_file_values(&codex_home.join("config.toml")));
            let entries: Vec<(&str, SecretClass, &str)> = values
                .iter()
                .map(|(label, value)| (label.as_str(), SecretClass::Operational, value.as_str()))
                .collect();
            let gate = output_gate::global();
            gate.register_all(&entries)
                .and_then(|_| gate.arm())
                .map_err(|err| format!("secret_output_gate: {err}"))?;
            tracing::info!(
                values = gate.value_count(),
                representations = gate.representation_count(),
                "PF-28-S01 secret output gate armed"
            );
            Ok(())
        })
        .clone()
}

/// The armed gate, if any.
pub(crate) fn active() -> Option<&'static OutputGate> {
    output_gate::active()
}

/// True when the serialized value would carry a managed secret. For
/// payloads that cannot be rebuilt after gating; the caller refuses them.
pub(crate) fn would_disclose<T: Serialize>(sink: OutputSink, value: &T) -> bool {
    let Some(gate) = active() else {
        return false;
    };
    match serde_json::to_string(value) {
        Ok(serialized) => gate.scrub(sink, &serialized).is_some(),
        Err(_) => true,
    }
}

/// Gates an event at delivery to the client: the single choke point that
/// also covers producers holding a cloned event sender. Streamed deltas were
/// held back where they were produced; here each chunk is scrubbed on its
/// own. `None` drops the event.
pub(crate) fn gate_delivered(msg: EventMsg) -> Option<EventMsg> {
    let Some(gate) = active() else {
        return Some(msg);
    };
    let text = |text: &mut String, sink| match gate.scrub(sink, text) {
        Some((_, provenance)) if provenance.withheld => false,
        Some((scrubbed, _)) => {
            *text = scrubbed;
            true
        }
        None => true,
    };
    match msg {
        EventMsg::ExecCommandOutputDelta(mut event) => {
            match gate.scrub_bytes(OutputSink::ToolResult, &event.chunk) {
                Some((_, provenance)) if provenance.withheld => None,
                Some((chunk, _)) => {
                    event.chunk = chunk;
                    Some(EventMsg::ExecCommandOutputDelta(event))
                }
                None => Some(EventMsg::ExecCommandOutputDelta(event)),
            }
        }
        EventMsg::AgentMessageContentDelta(mut event) => {
            text(&mut event.delta, OutputSink::ModelResponse)
                .then_some(EventMsg::AgentMessageContentDelta(event))
        }
        EventMsg::PlanDelta(mut event) => {
            text(&mut event.delta, OutputSink::ModelResponse).then_some(EventMsg::PlanDelta(event))
        }
        EventMsg::ReasoningContentDelta(mut event) => {
            text(&mut event.delta, OutputSink::ModelResponse)
                .then_some(EventMsg::ReasoningContentDelta(event))
        }
        EventMsg::ReasoningRawContentDelta(mut event) => {
            text(&mut event.delta, OutputSink::ModelResponse)
                .then_some(EventMsg::ReasoningRawContentDelta(event))
        }
        msg => present(gate, msg),
    }
}

/// Gates an event for presentation on paths that do not stream (raw sends).
/// `None` drops the event.
pub(crate) fn gate_presented(msg: EventMsg) -> Option<EventMsg> {
    match active() {
        Some(gate) => present(gate, msg),
        None => Some(msg),
    }
}

/// Gates a non-streamed event for presentation. An event that cannot be
/// rebuilt with redaction markers is sent with the affected text replaced by
/// a placeholder (ids and structure kept), so approvals, completions and turn ends still
/// arrive; only if even that fails is it replaced by an error notice.
fn present(gate: &OutputGate, msg: EventMsg) -> Option<EventMsg> {
    match gate_value_with(gate, OutputSink::Presentation, &msg) {
        Gated::Unchanged => Some(msg),
        Gated::Changed(msg) => Some(msg),
        Gated::Withheld => match msg {
            EventMsg::TurnComplete(mut event) => {
                event.last_agent_message = None;
                event.error = None;
                Some(EventMsg::TurnComplete(event))
            }
            msg @ EventMsg::TurnAborted(_) => Some(msg),
            msg => Some(
                strip_value(gate, OutputSink::Presentation, &msg).unwrap_or_else(|| {
                    EventMsg::Error(codex_protocol::protocol::ErrorEvent {
                        message: "Corbanu withheld an event that held a protected value."
                            .to_string(),
                        codex_error_info: None,
                    })
                }),
            ),
        },
    }
}

/// Placeholder for a field whose value cannot hold a redaction marker.
const WITHHELD_FIELD: &str = "[WITHHELD]";

/// Rebuilds `value` with every string that holds a managed value replaced
/// by a fixed placeholder, or emptied where the placeholder does not fit
/// (byte fields).
fn strip_value<T: Serialize + DeserializeOwned>(
    gate: &OutputGate,
    sink: OutputSink,
    value: &T,
) -> Option<T> {
    fn strip(gate: &OutputGate, sink: OutputSink, value: &mut serde_json::Value, with: &str) {
        match value {
            serde_json::Value::String(text) if gate.scrub(sink, text).is_some() => {
                *text = with.to_string();
            }
            serde_json::Value::Array(items) => {
                items
                    .iter_mut()
                    .for_each(|item| strip(gate, sink, item, with));
            }
            serde_json::Value::Object(map) => {
                gate_keys(map, |key| gate.scrub(sink, key).map(|_| with.to_string()));
                map.values_mut()
                    .for_each(|item| strip(gate, sink, item, with));
            }
            _ => {}
        }
    }
    let original = serde_json::to_value(value).ok()?;
    [WITHHELD_FIELD, ""].into_iter().find_map(|with| {
        let mut json = original.clone();
        strip(gate, sink, &mut json, with);
        // Found only across fields or in a number: nothing to replace.
        if survives_walk(gate, sink, &json) {
            return None;
        }
        serde_json::from_value(json).ok()
    })
}

/// Result of gating a structured value.
pub(crate) enum Gated<T> {
    Unchanged,
    Changed(T),
    /// The value could not be rebuilt after gating; drop it.
    Withheld,
}

/// Gates every string in a serializable value (items, events, rollout
/// lines). A quick scan of the serialized form decides whether the walk is
/// needed at all.
fn gate_value_with<T: Serialize + DeserializeOwned>(
    gate: &OutputGate,
    sink: OutputSink,
    value: &T,
) -> Gated<T> {
    let Ok(serialized) = serde_json::to_string(value) else {
        return Gated::Unchanged;
    };
    if gate.scrub(sink, &serialized).is_none() {
        return Gated::Unchanged;
    }
    let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&serialized) else {
        return Gated::Withheld;
    };
    let changed = scrub_json(gate, sink, &mut json);
    // Still found after every key and string was gated (a number, or a value
    // spanning fields): fail closed.
    if survives_walk(gate, sink, &json) {
        tracing::warn!("PF-28-S01: withheld a {} value", sink.as_str());
        return Gated::Withheld;
    }
    if !changed {
        // Only the size of the whole triggered the quick scan.
        return Gated::Unchanged;
    }
    match serde_json::from_value(json) {
        Ok(value) => Gated::Changed(value),
        Err(err) => {
            tracing::warn!("PF-28-S01: withheld a {} value: {err}", sink.as_str());
            Gated::Withheld
        }
    }
}

/// Gates a list in place, dropping values that cannot be rebuilt.
pub(crate) fn gate_values<T: Serialize + DeserializeOwned + Clone>(
    sink: OutputSink,
    values: &[T],
) -> Option<Vec<T>> {
    gate_values_with(active()?, sink, values)
}

fn gate_values_with<T: Serialize + DeserializeOwned + Clone>(
    gate: &OutputGate,
    sink: OutputSink,
    values: &[T],
) -> Option<Vec<T>> {
    let mut changed = false;
    let mut out = Vec::with_capacity(values.len());
    for value in values {
        match gate_value_with(gate, sink, value) {
            Gated::Unchanged => out.push(value.clone()),
            Gated::Changed(value) => {
                changed = true;
                out.push(value);
            }
            Gated::Withheld => changed = true,
        }
    }
    changed.then_some(out)
}

/// Gates a model request: input items and instructions. `None` when
/// unchanged.
pub(crate) fn gate_prompt(
    prompt: &crate::client_common::Prompt,
) -> Option<crate::client_common::Prompt> {
    gate_prompt_with(active()?, prompt)
}

fn gate_prompt_with(
    gate: &OutputGate,
    prompt: &crate::client_common::Prompt,
) -> Option<crate::client_common::Prompt> {
    let input = gate_values_with(gate, OutputSink::ModelRequest, &prompt.input);
    let instructions = gate.scrub(OutputSink::ModelRequest, &prompt.base_instructions.text);
    if input.is_none() && instructions.is_none() {
        return None;
    }
    let mut gated = prompt.clone();
    if let Some(input) = input {
        gated.input = input;
    }
    if let Some((text, provenance)) = instructions {
        log_provenance(&provenance);
        gated.base_instructions.text = text;
    }
    Some(gated)
}

fn scrub_json(gate: &OutputGate, sink: OutputSink, value: &mut serde_json::Value) -> bool {
    match value {
        serde_json::Value::String(text) => match gate.scrub(sink, text) {
            Some((scrubbed, provenance)) => {
                log_provenance(&provenance);
                *text = scrubbed;
                true
            }
            None => false,
        },
        serde_json::Value::Array(items) => items.iter_mut().fold(false, |changed, item| {
            scrub_json(gate, sink, item) | changed
        }),
        serde_json::Value::Object(map) => {
            let mut changed = gate_keys(map, |key| {
                gate.scrub(sink, key).map(|(scrubbed, provenance)| {
                    log_provenance(&provenance);
                    scrubbed
                })
            });
            for item in map.values_mut() {
                changed |= scrub_json(gate, sink, item);
            }
            changed
        }
        _ => false,
    }
}

/// True when a managed value is still present after every key and string
/// was gated: in a number, or spanning fields. A document over the scan
/// limit cannot be rescanned whole (the size alone would hit), so its number
/// leaves are checked one by one.
fn survives_walk(gate: &OutputGate, sink: OutputSink, json: &serde_json::Value) -> bool {
    fn number_hit(gate: &OutputGate, sink: OutputSink, value: &serde_json::Value) -> bool {
        match value {
            serde_json::Value::Number(number) => gate.scrub(sink, &number.to_string()).is_some(),
            serde_json::Value::Array(items) => {
                items.iter().any(|item| number_hit(gate, sink, item))
            }
            serde_json::Value::Object(map) => map.values().any(|item| number_hit(gate, sink, item)),
            _ => false,
        }
    }
    match serde_json::to_string(json) {
        Ok(text) if text.len() <= output_gate::MAX_SCAN_BYTES => gate.scrub(sink, &text).is_some(),
        Ok(_) => number_hit(gate, sink, json),
        Err(_) => true,
    }
}

/// Replaces object keys that hold a managed value (map keys can carry one:
/// MCP structured content, paths in patch events). A key that collides after
/// replacement gets a numeric suffix so no entry is lost.
fn gate_keys(
    map: &mut serde_json::Map<String, serde_json::Value>,
    replace: impl Fn(&str) -> Option<String>,
) -> bool {
    let renamed: Vec<(String, String)> = map
        .keys()
        .filter_map(|key| replace(key).map(|new| (key.clone(), new)))
        .collect();
    if renamed.is_empty() {
        return false;
    }
    for (old, new) in renamed {
        let Some(item) = map.remove(&old) else {
            continue;
        };
        let mut key = new.clone();
        let mut suffix = 2;
        while map.contains_key(&key) {
            key = format!("{new}#{suffix}");
            suffix += 1;
        }
        map.insert(key, item);
    }
    true
}

fn log_provenance(provenance: &output_gate::Provenance) {
    tracing::info!(
        sink = provenance.sink.as_str(),
        redacted = provenance.redacted,
        patterns = provenance.pattern_redacted,
        withheld = provenance.withheld,
        labels = %provenance.labels.join(","),
        "PF-28-S01 gated output"
    );
}

struct OpenStream {
    scrubber: StreamScrubber,
    /// The last delta, reused to emit the held-back tail.
    template: EventMsg,
    item_id: String,
    /// The turn (submission id) the stream belongs to.
    scope: String,
    order: u64,
}

/// Withheld streams evicted for capacity stay withheld (bounded).
const MAX_WITHHELD_TOMBSTONES: usize = 4096;

#[derive(Default)]
struct Streams {
    open: HashMap<String, OpenStream>,
    withheld: HashSet<String>,
    withheld_order: VecDeque<String>,
    next_order: u64,
}

static STREAMS: LazyLock<Mutex<Streams>> = LazyLock::new(|| Mutex::new(Streams::default()));

fn streams() -> std::sync::MutexGuard<'static, Streams> {
    match STREAMS.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Gates one client event of turn `scope`. Streamed deltas are held back
/// where they could end in part of a managed value; the tail is released
/// (gated) just before the event that completes the stream, or the turn.
/// Returns the events to deliver, in order (empty when everything is held
/// back or withheld).
pub(crate) fn gate_event(scope: &str, msg: EventMsg) -> Vec<EventMsg> {
    match active() {
        Some(gate) => gate_event_with(gate, scope, msg),
        None => vec![msg],
    }
}

fn gate_event_with(gate: &OutputGate, scope: &str, msg: EventMsg) -> Vec<EventMsg> {
    let mut out = Vec::new();
    // Keys and every finisher carry the turn: a sub-session's already-gated
    // stream forwarded into its parent's turn has its own scrubber, and the
    // child's events never drain the parent's (or the reverse).
    match msg {
        EventMsg::ExecCommandOutputDelta(mut event) => {
            let key = format!("exec\0{scope}\0{}\0{:?}", event.call_id, event.stream);
            let template = EventMsg::ExecCommandOutputDelta(event.clone());
            event.chunk = push_stream(
                gate,
                key,
                (scope, &event.call_id),
                template,
                &event.chunk,
                /*utf8*/ false,
            );
            if !event.chunk.is_empty() {
                out.push(EventMsg::ExecCommandOutputDelta(event));
            }
        }
        EventMsg::AgentMessageContentDelta(mut event) => {
            let key = format!("agent\0{scope}\0{}", event.item_id);
            let template = EventMsg::AgentMessageContentDelta(event.clone());
            event.delta = push_text(gate, key, (scope, &event.item_id), template, &event.delta);
            if !event.delta.is_empty() {
                out.push(EventMsg::AgentMessageContentDelta(event));
            }
        }
        EventMsg::PlanDelta(mut event) => {
            let key = format!("plan\0{scope}\0{}", event.item_id);
            let template = EventMsg::PlanDelta(event.clone());
            event.delta = push_text(gate, key, (scope, &event.item_id), template, &event.delta);
            if !event.delta.is_empty() {
                out.push(EventMsg::PlanDelta(event));
            }
        }
        EventMsg::ReasoningContentDelta(mut event) => {
            let key = format!(
                "reasoning\0{scope}\0{}\0{}",
                event.item_id, event.summary_index
            );
            // A new summary section ends the earlier ones of the same item.
            let section = key.clone();
            out.extend(finish_streams(gate, |key, stream| {
                stream.scope == scope
                    && stream.item_id == event.item_id
                    && key.starts_with("reasoning\0")
                    && key != section.as_str()
            }));
            let template = EventMsg::ReasoningContentDelta(event.clone());
            event.delta = push_text(gate, key, (scope, &event.item_id), template, &event.delta);
            if !event.delta.is_empty() {
                out.push(EventMsg::ReasoningContentDelta(event));
            }
        }
        EventMsg::ReasoningRawContentDelta(mut event) => {
            let key = format!("raw\0{scope}\0{}\0{}", event.item_id, event.content_index);
            let template = EventMsg::ReasoningRawContentDelta(event.clone());
            event.delta = push_text(gate, key, (scope, &event.item_id), template, &event.delta);
            if !event.delta.is_empty() {
                out.push(EventMsg::ReasoningRawContentDelta(event));
            }
        }
        msg => {
            match &msg {
                EventMsg::ExecCommandEnd(end) => {
                    out.extend(finish_streams(gate, |_, stream| {
                        stream.scope == scope && stream.item_id == end.call_id
                    }));
                }
                EventMsg::ItemCompleted(completed) => {
                    let id = completed.item.id();
                    out.extend(finish_streams(gate, |_, stream| {
                        stream.scope == scope && stream.item_id == id
                    }));
                }
                EventMsg::AgentReasoningSectionBreak(event) => {
                    out.extend(finish_streams(gate, |key, stream| {
                        stream.scope == scope
                            && stream.item_id == event.item_id
                            && key.starts_with("reasoning\0")
                    }));
                }
                // Turns that end without completing their items (abort,
                // error) still release every tail they hold.
                EventMsg::TurnComplete(event) => {
                    out.extend(finish_streams(gate, |_, stream| {
                        stream.scope == event.turn_id
                    }));
                }
                EventMsg::TurnAborted(event) => {
                    let turn = event.turn_id.as_deref().unwrap_or(scope);
                    out.extend(finish_streams(gate, |_, stream| stream.scope == turn));
                }
                _ => {}
            }
            out.extend(present(gate, msg));
        }
    }
    out
}

fn push_text(
    gate: &OutputGate,
    key: String,
    owner: (&str, &str),
    template: EventMsg,
    delta: &str,
) -> String {
    let bytes = push_stream(
        gate,
        key,
        owner,
        template,
        delta.as_bytes(),
        /*utf8*/ true,
    );
    match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(err) => String::from_utf8_lossy(err.as_bytes()).into_owned(),
    }
}

fn push_stream(
    gate: &OutputGate,
    key: String,
    (scope, item_id): (&str, &str),
    template: EventMsg,
    chunk: &[u8],
    utf8: bool,
) -> Vec<u8> {
    let mut streams = streams();
    if streams.withheld.contains(&key) {
        return Vec::new();
    }
    let order = streams.next_order;
    streams.next_order += 1;
    if !streams.open.contains_key(&key) && streams.open.len() >= MAX_OPEN_STREAMS {
        let oldest = streams
            .open
            .iter()
            .min_by_key(|(_, stream)| stream.order)
            .map(|(key, _)| key.clone());
        if let Some(oldest) = oldest
            && let Some(evicted) = streams.open.remove(&oldest)
            && evicted.scrubber.is_withheld()
        {
            // Oldest tombstones go first.
            while streams.withheld_order.len() >= MAX_WITHHELD_TOMBSTONES {
                if let Some(expired) = streams.withheld_order.pop_front() {
                    streams.withheld.remove(&expired);
                }
            }
            streams.withheld_order.push_back(oldest.clone());
            streams.withheld.insert(oldest);
        }
    }
    let stream = streams.open.entry(key).or_insert_with(|| OpenStream {
        scrubber: StreamScrubber::new(),
        template: template.clone(),
        item_id: item_id.to_string(),
        scope: scope.to_string(),
        order,
    });
    stream.template = template;
    stream.order = order;
    let sink = if utf8 {
        OutputSink::ModelResponse
    } else {
        OutputSink::ToolResult
    };
    stream.scrubber.push(gate, sink, chunk, utf8)
}

/// Releases the gated tails of every stream `select` picks, oldest first.
fn finish_streams(gate: &OutputGate, select: impl Fn(&str, &OpenStream) -> bool) -> Vec<EventMsg> {
    let mut streams = streams();
    let keys: Vec<String> = streams
        .open
        .iter()
        .filter(|(key, stream)| select(key, stream))
        .map(|(key, _)| key.clone())
        .collect();
    for key in &keys {
        streams.withheld.remove(key);
    }
    let mut finished: Vec<OpenStream> = keys
        .into_iter()
        .filter_map(|key| streams.open.remove(&key))
        .collect();
    drop(streams);
    finished.sort_by_key(|stream| stream.order);
    let mut out = Vec::new();
    for mut stream in finished {
        let utf8 = !matches!(stream.template, EventMsg::ExecCommandOutputDelta(_));
        let sink = if utf8 {
            OutputSink::ModelResponse
        } else {
            OutputSink::ToolResult
        };
        let tail = stream.scrubber.finish(gate, sink);
        if tail.is_empty() {
            continue;
        }
        let text = || String::from_utf8_lossy(&tail).into_owned();
        let event = match stream.template {
            EventMsg::ExecCommandOutputDelta(mut event) => {
                event.chunk = tail.clone();
                EventMsg::ExecCommandOutputDelta(event)
            }
            EventMsg::AgentMessageContentDelta(mut event) => {
                event.delta = text();
                EventMsg::AgentMessageContentDelta(event)
            }
            EventMsg::PlanDelta(mut event) => {
                event.delta = text();
                EventMsg::PlanDelta(event)
            }
            EventMsg::ReasoningContentDelta(mut event) => {
                event.delta = text();
                EventMsg::ReasoningContentDelta(event)
            }
            EventMsg::ReasoningRawContentDelta(mut event) => {
                event.delta = text();
                EventMsg::ReasoningRawContentDelta(event)
            }
            _ => continue,
        };
        out.push(event);
    }
    out
}

fn is_credential_env_name(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    secretless_launch::is_secret_looking_name(&upper)
        && !NON_CREDENTIAL_ENV_NAMES.contains(&upper.as_str())
        && !NON_CREDENTIAL_ENV_FRAGMENTS
            .iter()
            .any(|fragment| upper.contains(fragment))
        && !NON_CREDENTIAL_ENV_PREFIXES
            .iter()
            .any(|prefix| upper.starts_with(prefix))
        && !NON_CREDENTIAL_ENV_SUFFIXES
            .iter()
            .any(|suffix| upper.ends_with(suffix))
}

/// Long enough, shaped like a credential rather than a word, not an
/// existing path, and not a variable name (`GITHUB_TOKEN` as a config value
/// names where the secret lives). Redacting ordinary text would corrupt what
/// the model reads and edits, so seeding errs towards real credentials.
fn is_plausible_secret(value: &str) -> bool {
    let names_a_variable = value.contains('_')
        && value.starts_with(|ch: char| ch.is_ascii_uppercase())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_');
    value.len() >= MIN_ENV_VALUE_BYTES
        && (value.len() >= LONG_SECRET_BYTES || character_classes(value) >= 2)
        && !(value.starts_with('/') && Path::new(value).exists())
        && !names_a_variable
}

/// Lowercase, uppercase, digit and other: how many appear.
fn character_classes(value: &str) -> usize {
    let has = |test: fn(&u8) -> bool| usize::from(value.bytes().any(|byte| test(&byte)));
    has(u8::is_ascii_lowercase)
        + has(u8::is_ascii_uppercase)
        + has(u8::is_ascii_digit)
        + has(|byte| !byte.is_ascii_alphanumeric())
}

type Labeled = Vec<(String, Zeroizing<String>)>;

/// Secret-looking environment values and passwords embedded in URLs.
pub(crate) fn env_values<I>(vars: I) -> Labeled
where
    I: IntoIterator<Item = (std::ffi::OsString, std::ffi::OsString)>,
{
    let mut values = Labeled::new();
    for (name, value) in vars {
        let (Some(name), Some(value)) = (name.to_str(), value.to_str()) else {
            continue;
        };
        let value = value.trim();
        if is_credential_env_name(name)
            && !value.contains(char::is_whitespace)
            && is_plausible_secret(value)
        {
            values.push((format!("env:{name}"), Zeroizing::new(value.to_string())));
        }
        if let Some(password) = url_password(value) {
            values.push((format!("env:{name}:password"), password));
        }
        for (param, secret) in url_query_secrets(value) {
            values.push((format!("env:{name}:{param}"), secret));
        }
    }
    values
}

fn url_password(value: &str) -> Option<Zeroizing<String>> {
    if !secretless_launch::value_has_url_password(value) {
        return None;
    }
    let (_, rest) = value.split_once("://")?;
    let authority = rest.split(['/', '?', '#']).next()?;
    let (userinfo, _) = authority.rsplit_once('@')?;
    let (_, password) = userinfo.split_once(':')?;
    let password = password.trim();
    is_plausible_secret(password).then(|| Zeroizing::new(password.to_string()))
}

/// Secret-named query parameters of a URL (`?token=`, `?api_key=`, `?sig=`).
fn url_query_secrets(value: &str) -> Vec<(String, Zeroizing<String>)> {
    if !value.contains("://") {
        return Vec::new();
    }
    let Some((_, query)) = value.split_once('?') else {
        return Vec::new();
    };
    let query = query.split('#').next().unwrap_or_default();
    query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .filter(|(name, secret)| {
            let lower = name.to_ascii_lowercase();
            (is_secret_key(name) || matches!(lower.as_str(), "key" | "sig" | "signature"))
                && is_plausible_secret(secret)
        })
        .map(|(name, secret)| (name.to_string(), Zeroizing::new(secret.to_string())))
        .collect()
}

fn is_secret_key(key: &str) -> bool {
    let upper = key.to_ascii_uppercase().replace('-', "_");
    // `bearer_token_env_var = "GITHUB_TOKEN"` names a variable; it is not one.
    if upper.ends_with("_ENV_VAR") || upper.ends_with("ENV_KEY") || upper.ends_with("_ENV") {
        return false;
    }
    SECRET_KEY_FRAGMENTS
        .iter()
        .any(|fragment| upper.contains(fragment))
}

/// Secret-named string values in a JSON sign-in file. Everything under a
/// `tokens` object counts.
fn json_file_values(path: &Path, label: &str) -> Labeled {
    let Ok(text) = Zeroizing::new(std::fs::read_to_string(path).unwrap_or_default())
        .parse::<serde_json::Value>()
    else {
        return Labeled::new();
    };
    let mut values = Labeled::new();
    collect_json(&text, label, /*secret*/ false, &mut values);
    values
}

fn collect_json(value: &serde_json::Value, label: &str, secret: bool, out: &mut Labeled) {
    match value {
        serde_json::Value::String(text) if secret => push_secret(label, text, out),
        serde_json::Value::Array(items) => {
            for item in items {
                collect_json(item, label, secret, out);
            }
        }
        serde_json::Value::Object(map) => {
            for (key, item) in map {
                let child_secret = secret || is_secret_key(key) || key == "tokens";
                collect_json(item, &format!("{label}:{key}"), child_secret, out);
            }
        }
        _ => {}
    }
}

/// Secret-named string values in `config.toml` (bearer tokens, MCP server
/// `env` and header values).
fn toml_file_values(path: &Path) -> Labeled {
    let text = Zeroizing::new(std::fs::read_to_string(path).unwrap_or_default());
    let Ok(table) = toml::from_str::<toml::Table>(&text) else {
        return Labeled::new();
    };
    let mut values = Labeled::new();
    collect_toml(
        &toml::Value::Table(table),
        "config.toml",
        /*secret*/ false,
        &mut values,
    );
    values
}

fn collect_toml(value: &toml::Value, label: &str, secret: bool, out: &mut Labeled) {
    match value {
        toml::Value::String(text) if secret => push_secret(label, text, out),
        toml::Value::Array(items) => {
            for item in items {
                collect_toml(item, label, secret, out);
            }
        }
        toml::Value::Table(table) => {
            for (key, item) in table {
                let child_secret = secret || is_secret_key(key);
                collect_toml(item, &format!("{label}:{key}"), child_secret, out);
            }
        }
        _ => {}
    }
}

fn push_secret(label: &str, text: &str, out: &mut Labeled) {
    let text = text.trim();
    if !is_plausible_secret(text) {
        return;
    }
    out.push((label.to_string(), Zeroizing::new(text.to_string())));
    // `Authorization = "Bearer <token>"`: the token alone is the secret.
    if let Some((scheme, token)) = text.split_once(' ')
        && matches!(
            scheme.to_ascii_lowercase().as_str(),
            "bearer" | "basic" | "token"
        )
        && is_plausible_secret(token.trim())
    {
        out.push((label.to_string(), Zeroizing::new(token.trim().to_string())));
    }
}

#[cfg(test)]
#[path = "disclosure_gate_tests.rs"]
mod tests;
