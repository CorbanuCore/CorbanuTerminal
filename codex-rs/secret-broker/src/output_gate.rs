//! PF-28-S01 central secret and protected-output gate.
//!
//! One registry holds every active managed secret value (Core's own
//! credentials, sign-in tokens, vault values Core resolved). Every protected
//! sink (model requests and responses, tool results, transcripts, traces,
//! errors, audit, snapshots, exports and diagnostics) passes its text through
//! [`OutputGate::scrub`] before persistence or presentation.
//!
//! Guarantees, bounded on purpose:
//! - exact values and their JSON, percent and base64 (any alignment) and hex
//!   encodings are removed whole, including overlapping and chunk-split
//!   occurrences ([`StreamScrubber`]);
//! - seed phrases, private keys and financial values are never shown in part:
//!   any match withholds the whole payload (a derived view needs a separate
//!   authorization, not string redaction);
//! - registration never evicts a live value. Rotation adds the new value
//!   before the old one is retired, and a retired value stays protected until
//!   its last [`SecretLease`] ends. When capacity is exhausted, registration
//!   fails and the caller must not use the value;
//! - payloads larger than [`MAX_SCAN_BYTES`] are withheld, not passed.
//!
//! There is no universal detector: values in unknown encodings (compressed,
//! encrypted, split across separate fields) are not found. Raw values never
//! leave this process; agent processes get none of them.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use regex::bytes::Regex;
use regex::bytes::RegexBuilder;
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::RwLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use zeroize::Zeroizing;

/// Upper bound on distinct representations across all live values.
pub const MAX_REPRESENTATIONS: usize = 4096;
/// Upper bound on the bytes of all representations (the compiled matcher).
pub const MAX_REPRESENTATION_TOTAL_BYTES: usize = 4 * 1024 * 1024;
/// Payloads larger than this are withheld instead of scanned.
pub const MAX_SCAN_BYTES: usize = 16 * 1024 * 1024;
/// Values shorter than this cannot be told apart from ordinary text.
pub const MIN_VALUE_BYTES: usize = 3;
/// Values shorter than this match only as a whole word and get no encodings.
pub const MIN_SUBSTRING_BYTES: usize = 6;
/// Shortest encoded (base64/hex) form searched for.
const MIN_ENCODED_BYTES: usize = 8;
/// Longest value whose encodings are generated; longer values match raw/JSON.
const MAX_ENCODED_VALUE_BYTES: usize = 4096;
/// Padding characters removed after a base64 core.
const BASE64_SLACK: usize = 2;
/// Longest representation kept (a pasted certificate chain, for example).
const MAX_REPRESENTATION_BYTES: usize = 64 * 1024;

/// What kind of secret a value is. Everything except `Operational` is never
/// disclosable in part: a match withholds the whole payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SecretClass {
    Operational,
    SeedPhrase,
    PrivateKey,
    Financial,
}

impl SecretClass {
    fn withholds_payload(self) -> bool {
        !matches!(self, Self::Operational)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Operational => "credential",
            Self::SeedPhrase => "seed phrase",
            Self::PrivateKey => "private key",
            Self::Financial => "financial value",
        }
    }
}

/// Where gated output goes. Diagnostic-class sinks also apply bounded
/// key-shape patterns; model and tool sinks match managed values only, so
/// ordinary code that looks like a key is not rewritten for the agent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OutputSink {
    ModelRequest,
    ModelResponse,
    ToolResult,
    Transcript,
    Presentation,
    Trace,
    Error,
    Audit,
    Snapshot,
    Export,
    Diagnostic,
}

impl OutputSink {
    fn applies_patterns(self) -> bool {
        matches!(
            self,
            Self::Trace
                | Self::Error
                | Self::Audit
                | Self::Snapshot
                | Self::Export
                | Self::Diagnostic
        )
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ModelRequest => "model_request",
            Self::ModelResponse => "model_response",
            Self::ToolResult => "tool_result",
            Self::Transcript => "transcript",
            Self::Presentation => "presentation",
            Self::Trace => "trace",
            Self::Error => "error",
            Self::Audit => "audit",
            Self::Snapshot => "snapshot",
            Self::Export => "export",
            Self::Diagnostic => "diagnostic",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RegisterError {
    #[error("a managed value shorter than {MIN_VALUE_BYTES} bytes cannot be protected")]
    TooShort,
    #[error("the output gate is full ({limit} representations); the value was not admitted")]
    CapacityExhausted { limit: usize },
}

/// Identifies one registered value without exposing it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SecretHandle(u64);

/// Provenance attached to gated output: what the gate did, never the value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provenance {
    pub sink: OutputSink,
    /// Distinct redacted ranges.
    pub redacted: usize,
    /// Pattern-only redactions (diagnostic-class sinks).
    pub pattern_redacted: usize,
    /// The whole payload was replaced.
    pub withheld: bool,
    /// Labels of the values found (labels are names, not secrets).
    pub labels: Vec<String>,
}

impl Provenance {
    fn new(sink: OutputSink) -> Self {
        Self {
            sink,
            redacted: 0,
            pattern_redacted: 0,
            withheld: false,
            labels: Vec::new(),
        }
    }

    pub fn changed(&self) -> bool {
        self.withheld || self.redacted > 0 || self.pattern_redacted > 0
    }
}

/// Bounded key shapes for diagnostic-class sinks.
static KEY_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"-----BEGIN [A-Z ]{0,24}PRIVATE KEY-----(?s:.*?)(?:-----END [A-Z ]{0,24}PRIVATE KEY-----|\z)",
        r"\bsk-[A-Za-z0-9_\-]{20,256}",
        r"\bAKIA[0-9A-Z]{16}\b",
        r"\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{36,255}\b",
        r"\bgithub_pat_[A-Za-z0-9_]{22,255}\b",
        r"\bxox[abprs]-[A-Za-z0-9\-]{10,256}",
        r"(?i)\bBearer\s+[A-Za-z0-9._~+/\-]{16,4096}=*",
    ]
    .into_iter()
    .map(|pattern| match Regex::new(pattern) {
        Ok(regex) => regex,
        // Constant patterns; covered by tests.
        Err(err) => panic!("invalid key pattern `{pattern}`: {err}"),
    })
    .collect()
});

struct Entry {
    id: u64,
    label: String,
    class: SecretClass,
    reps: Vec<Zeroizing<Vec<u8>>>,
    /// Indices of `reps` that are base64 cores.
    base64: std::ops::Range<usize>,
    whole_word: bool,
    /// Registrations holding this value (the same value admitted under
    /// several labels or owners shares one entry); retired at zero.
    owners: usize,
    leases: usize,
    retired: bool,
}

struct Rep {
    bytes: Zeroizing<Vec<u8>>,
    label: Arc<str>,
    class: SecretClass,
    whole_word: bool,
    /// A base64 core: the partial characters around it also carry bits of
    /// the value and are removed with it.
    base64: bool,
}

/// Compiled view of the live values; scans hold an `Arc` so a concurrent
/// rotation cannot remove a value from a scan in progress.
#[derive(Default)]
struct Snapshot {
    substring: Option<Regex>,
    whole_word: Option<Regex>,
    /// Sorted by bytes for lookup of a match's owner.
    reps: Vec<Rep>,
    max_len: usize,
}

#[derive(Default)]
struct State {
    entries: Vec<Entry>,
    next_id: u64,
}

impl State {
    fn distinct_reps(&self) -> BTreeSet<&[u8]> {
        self.entries
            .iter()
            .flat_map(|entry| entry.reps.iter().map(|rep| rep.as_slice()))
            .collect()
    }
}

struct Inner {
    armed: AtomicBool,
    state: Mutex<State>,
    snapshot: RwLock<Arc<Snapshot>>,
}

/// The registry and scanner. Cheap to clone; clones share state.
#[derive(Clone)]
pub struct OutputGate {
    inner: Arc<Inner>,
}

impl std::fmt::Debug for OutputGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let values = self.state().entries.len();
        f.debug_struct("OutputGate")
            .field("armed", &self.is_armed())
            .field("values", &values)
            .finish()
    }
}

static GLOBAL: LazyLock<OutputGate> = LazyLock::new(OutputGate::new_disarmed);

/// The process-wide gate. Registrations are accepted before arming (so values
/// resolved early are not missed); scanning starts once [`OutputGate::arm`]
/// runs (feature `secret_output_gate`).
pub fn global() -> &'static OutputGate {
    &GLOBAL
}

/// The process-wide gate when armed.
pub fn active() -> Option<&'static OutputGate> {
    let gate = global();
    gate.is_armed().then_some(gate)
}

/// Gates text through the process-wide gate when it is armed; `None` when
/// unchanged. For sinks outside Core (logs, feedback, prompt history).
pub fn scrub_if_armed(sink: OutputSink, text: &str) -> Option<String> {
    active()?.scrub(sink, text).map(|(text, _)| text)
}

impl OutputGate {
    /// An armed gate (tests and embedded use).
    pub fn new() -> Self {
        let gate = Self::new_disarmed();
        // An empty registry always compiles.
        let _ = gate.arm();
        gate
    }

    fn new_disarmed() -> Self {
        Self {
            inner: Arc::new(Inner {
                armed: AtomicBool::new(false),
                state: Mutex::new(State::default()),
                snapshot: RwLock::new(Arc::new(Snapshot::default())),
            }),
        }
    }

    /// Starts scanning. Idempotent and one-way. Fails (and stays disarmed)
    /// only if the values registered so far cannot be compiled.
    pub fn arm(&self) -> Result<(), RegisterError> {
        let state = self.state();
        if self.is_armed() {
            return Ok(());
        }
        let snapshot = compile(&state).ok_or(RegisterError::CapacityExhausted {
            limit: MAX_REPRESENTATIONS,
        })?;
        self.publish(snapshot);
        self.inner.armed.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub fn is_armed(&self) -> bool {
        self.inner.armed.load(Ordering::SeqCst)
    }

    /// Admits a value. A value already registered under any label returns its
    /// existing handle. Fails closed: when the gate is full nothing is
    /// evicted and the value is not admitted.
    pub fn register(
        &self,
        label: &str,
        class: SecretClass,
        value: &str,
    ) -> Result<SecretHandle, RegisterError> {
        self.register_all(&[(label, class, value)])
            .map(|handles| handles[0])
    }

    /// Admits several values with one recompile, all or nothing: on any
    /// failure none of them is admitted and existing values are untouched.
    pub fn register_all(
        &self,
        values: &[(&str, SecretClass, &str)],
    ) -> Result<Vec<SecretHandle>, RegisterError> {
        let mut state = self.state();
        let before = state.entries.len();
        let mut changed = false;
        let mut handles = Vec::with_capacity(values.len());
        let mut failure = None;
        for (label, class, value) in values {
            match admit(&mut state, label, *class, value) {
                Ok((handle, admitted_change)) => {
                    changed |= admitted_change;
                    handles.push(handle);
                }
                Err(err) => {
                    failure = Some(err);
                    break;
                }
            }
        }
        if failure.is_none() && changed && self.rebuild(&state).is_err() {
            failure = Some(RegisterError::CapacityExhausted {
                limit: MAX_REPRESENTATIONS,
            });
        }
        if let Some(err) = failure {
            // Class upgrades of existing values are kept (stricter is safe);
            // new entries are dropped and the previous snapshot stays.
            state.entries.truncate(before);
            let _ = self.rebuild(&state);
            return Err(err);
        }
        Ok(handles)
    }

    /// Replaces `old` with a new value: the new value is admitted first, so
    /// there is no window where neither is protected. If admission fails the
    /// old value stays registered.
    pub fn rotate(
        &self,
        old: SecretHandle,
        label: &str,
        class: SecretClass,
        new_value: &str,
    ) -> Result<SecretHandle, RegisterError> {
        let handle = self.register(label, class, new_value)?;
        if handle != old {
            self.retire(old);
        }
        Ok(handle)
    }

    /// Holds a value protected for the lifetime of an in-flight request or
    /// response even if it is retired meanwhile.
    pub fn lease(&self, handle: SecretHandle) -> Option<SecretLease> {
        let mut state = self.state();
        let entry = state
            .entries
            .iter_mut()
            .find(|entry| entry.id == handle.0)?;
        entry.leases += 1;
        Some(SecretLease {
            gate: self.clone(),
            handle,
        })
    }

    /// Drops one registration of a value; it stops being protected once no
    /// other registration and no lease holds it.
    pub fn retire(&self, handle: SecretHandle) {
        let mut state = self.state();
        let Some(entry) = state.entries.iter_mut().find(|entry| entry.id == handle.0) else {
            return;
        };
        entry.owners = entry.owners.saturating_sub(1);
        if entry.owners > 0 {
            return;
        }
        entry.retired = true;
        if entry.leases == 0 {
            state.entries.retain(|entry| entry.id != handle.0);
            // On failure the previous (larger) snapshot stays in force.
            let _ = self.rebuild(&state);
        }
    }

    fn release(&self, handle: SecretHandle) {
        let mut state = self.state();
        let Some(entry) = state.entries.iter_mut().find(|entry| entry.id == handle.0) else {
            return;
        };
        entry.leases = entry.leases.saturating_sub(1);
        if entry.retired && entry.leases == 0 {
            state.entries.retain(|entry| entry.id != handle.0);
            let _ = self.rebuild(&state);
        }
    }

    /// Number of live (registered, not fully retired) values.
    pub fn value_count(&self) -> usize {
        self.state().entries.len()
    }

    /// Number of distinct representations searched for.
    pub fn representation_count(&self) -> usize {
        self.state().distinct_reps().len()
    }

    /// Gates one text payload. `None` means unchanged (the common case).
    pub fn scrub(&self, sink: OutputSink, input: &str) -> Option<(String, Provenance)> {
        let (bytes, provenance) = self.scrub_bytes(sink, input.as_bytes())?;
        let text = match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(err) => String::from_utf8_lossy(err.as_bytes()).into_owned(),
        };
        Some((text, provenance))
    }

    /// Gates one byte payload (command output). `None` means unchanged.
    pub fn scrub_bytes(&self, sink: OutputSink, input: &[u8]) -> Option<(Vec<u8>, Provenance)> {
        if !self.is_armed() {
            return None;
        }
        let snapshot = self.snapshot();
        let mut provenance = Provenance::new(sink);
        if input.len() > MAX_SCAN_BYTES {
            provenance.withheld = true;
            let marker = format!(
                "[WITHHELD: {} bytes is too large to check for secrets]",
                input.len()
            );
            return Some((marker.into_bytes(), provenance));
        }
        let found = find_all(&snapshot, input);
        if let Some(withheld) = found.iter().find(|m| m.class.withholds_payload()) {
            provenance.withheld = true;
            provenance.labels = vec![withheld.label.to_string()];
            return Some((withheld_marker(withheld.class, &withheld.label), provenance));
        }
        let mut output = if found.is_empty() {
            Cow::Borrowed(input)
        } else {
            Cow::Owned(redact_ranges(input, &found, &mut provenance))
        };
        if sink.applies_patterns() {
            for pattern in KEY_PATTERNS.iter() {
                if let Cow::Owned(replaced) =
                    pattern.replace_all(output.as_ref(), b"[REDACTED:key-pattern]".as_slice())
                {
                    provenance.pattern_redacted += 1;
                    output = Cow::Owned(replaced);
                }
            }
        }
        match output {
            Cow::Borrowed(_) => None,
            Cow::Owned(bytes) => Some((bytes, provenance)),
        }
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        match self.inner.state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn snapshot(&self) -> Arc<Snapshot> {
        match self.inner.snapshot.read() {
            Ok(guard) => Arc::clone(&guard),
            Err(poisoned) => Arc::clone(&poisoned.into_inner()),
        }
    }

    /// Recompiles under the state lock so snapshots are published in order.
    /// Before arming nothing is compiled.
    fn rebuild(&self, state: &State) -> Result<(), ()> {
        if !self.is_armed() {
            return Ok(());
        }
        let snapshot = compile(state).ok_or(())?;
        self.publish(snapshot);
        Ok(())
    }

    fn publish(&self, snapshot: Snapshot) {
        let snapshot = Arc::new(snapshot);
        match self.inner.snapshot.write() {
            Ok(mut guard) => *guard = snapshot,
            Err(poisoned) => *poisoned.into_inner() = snapshot,
        }
    }
}

impl Default for OutputGate {
    fn default() -> Self {
        Self::new()
    }
}

/// Keeps one value protected until dropped.
pub struct SecretLease {
    gate: OutputGate,
    handle: SecretHandle,
}

impl Drop for SecretLease {
    fn drop(&mut self) {
        self.gate.release(self.handle);
    }
}

/// Gates a stream delivered in chunks. Bytes that could be the start of a
/// managed value are held back until the next chunk shows whether they are,
/// so no prefix or suffix of a value is ever emitted. After a withheld-class
/// match the rest of the stream is withheld.
#[derive(Default)]
pub struct StreamScrubber {
    carry: Zeroizing<Vec<u8>>,
    withheld: bool,
}

impl StreamScrubber {
    pub fn new() -> Self {
        Self::default()
    }

    /// Bytes currently held back.
    pub fn pending(&self) -> usize {
        self.carry.len()
    }

    /// True once a withheld-class value was seen: the rest is withheld.
    pub fn is_withheld(&self) -> bool {
        self.withheld
    }

    /// Adds a chunk; returns the bytes safe to emit now. `utf8` keeps the cut
    /// on a character boundary for text streams.
    pub fn push(
        &mut self,
        gate: &OutputGate,
        sink: OutputSink,
        chunk: &[u8],
        utf8: bool,
    ) -> Vec<u8> {
        if !gate.is_armed() {
            return chunk.to_vec();
        }
        if self.withheld {
            return Vec::new();
        }
        let mut buffer = Zeroizing::new(Vec::with_capacity(self.carry.len() + chunk.len()));
        buffer.extend_from_slice(&self.carry);
        buffer.extend_from_slice(chunk);
        let snapshot = gate.snapshot();
        // Hold back only the longest tail that could still grow into a value,
        // plus the byte before it (a base64 partial character or a whole-word
        // boundary). A value found near the end is held with its boundary
        // byte so trailing base64 partial characters and padding go with it.
        let len = buffer.len();
        let mut cut = match longest_partial_suffix(&snapshot, &buffer) {
            0 => len,
            partial => len - partial.saturating_add(1).min(len),
        };
        // Never cut through a value, nor emit one whose trailing partial
        // characters or padding may still arrive; moving the cut back only
        // ever holds more.
        let found = find_all(&snapshot, &buffer);
        loop {
            let mut moved = false;
            for m in &found {
                let near_end = m.end + BASE64_SLACK + 1 >= len;
                let to = m.start.saturating_sub(1);
                if m.start < cut && (m.end > cut || near_end) && to < cut {
                    cut = to;
                    moved = true;
                }
            }
            if !moved {
                break;
            }
        }
        if utf8 {
            while cut > 0 && cut < buffer.len() && (buffer[cut] & 0xC0) == 0x80 {
                cut -= 1;
            }
            // A character still incomplete at the cut waits for its rest.
            if let Some(lead) = (cut.saturating_sub(3)..cut)
                .rev()
                .find(|&at| (buffer[at] & 0xC0) != 0x80)
            {
                let width = match buffer[lead] {
                    0xF0.. => 4,
                    0xE0.. => 3,
                    0xC0.. => 2,
                    _ => 1,
                };
                if cut - lead < width {
                    cut = lead;
                }
            }
        }
        let (ready, rest) = buffer.split_at(cut);
        self.carry = Zeroizing::new(rest.to_vec());
        self.emit(gate, sink, ready)
    }

    /// Ends the stream; returns whatever is left, gated.
    pub fn finish(&mut self, gate: &OutputGate, sink: OutputSink) -> Vec<u8> {
        let rest = std::mem::take(&mut *self.carry);
        let rest = Zeroizing::new(rest);
        if self.withheld {
            return Vec::new();
        }
        self.emit(gate, sink, &rest)
    }

    fn emit(&mut self, gate: &OutputGate, sink: OutputSink, ready: &[u8]) -> Vec<u8> {
        match gate.scrub_bytes(sink, ready) {
            None => ready.to_vec(),
            Some((bytes, provenance)) => {
                if provenance.withheld {
                    self.withheld = true;
                    self.carry = Zeroizing::new(Vec::new());
                }
                bytes
            }
        }
    }
}

/// Adds one value to `state` without recompiling. Returns its handle and
/// whether the compiled matcher must change.
fn admit(
    state: &mut State,
    label: &str,
    class: SecretClass,
    value: &str,
) -> Result<(SecretHandle, bool), RegisterError> {
    let value = value.trim();
    if value.len() < MIN_VALUE_BYTES {
        return Err(RegisterError::TooShort);
    }
    if let Some(entry) = state
        .entries
        .iter_mut()
        .find(|entry| entry.reps.first().map(|rep| rep.as_slice()) == Some(value.as_bytes()))
    {
        // Same value, possibly a stricter class: keep the strictest.
        if entry.retired {
            entry.retired = false;
            entry.owners = 1;
        } else {
            entry.owners += 1;
        }
        let upgraded = class > entry.class;
        if upgraded {
            entry.class = class;
        }
        return Ok((SecretHandle(entry.id), upgraded));
    }
    let (reps, base64) = representations(value);
    let distinct = state.distinct_reps();
    let added = reps
        .iter()
        .filter(|rep| !distinct.contains(rep.as_slice()))
        .collect::<Vec<_>>();
    let count = distinct.len() + added.len();
    let bytes = distinct.iter().map(|rep| rep.len()).sum::<usize>()
        + added.iter().map(|rep| rep.len()).sum::<usize>();
    drop(distinct);
    if count > MAX_REPRESENTATIONS || bytes > MAX_REPRESENTATION_TOTAL_BYTES {
        return Err(RegisterError::CapacityExhausted {
            limit: MAX_REPRESENTATIONS,
        });
    }
    let id = state.next_id;
    state.next_id += 1;
    state.entries.push(Entry {
        id,
        label: sanitize_label(label),
        class,
        reps,
        base64,
        whole_word: value.len() < MIN_SUBSTRING_BYTES,
        owners: 1,
        leases: 0,
        retired: false,
    });
    Ok((SecretHandle(id), true))
}

struct Match {
    start: usize,
    end: usize,
    label: Arc<str>,
    class: SecretClass,
}

/// Every occurrence, overlapping ones included (the scan restarts one byte
/// after each match start). A whole-word value at the very end of the input
/// counts as a match: at a stream cut the next byte is not known yet.
fn find_all(snapshot: &Snapshot, input: &[u8]) -> Vec<Match> {
    let mut found = Vec::new();
    for (regex, whole_word) in [
        (snapshot.substring.as_ref(), false),
        (snapshot.whole_word.as_ref(), true),
    ] {
        let Some(regex) = regex else {
            continue;
        };
        let mut at = 0;
        while at < input.len() {
            let Some(m) = regex.find_at(input, at) else {
                break;
            };
            at = m.start() + 1;
            if whole_word
                && ((m.start() > 0 && is_word_byte(input[m.start() - 1]))
                    || input.get(m.end()).is_some_and(|byte| is_word_byte(*byte)))
            {
                continue;
            }
            let Some(rep) = lookup(snapshot, m.as_bytes()) else {
                continue;
            };
            let (mut start, mut end) = (m.start(), m.end());
            if rep.base64 {
                // One partial character on each side, then padding.
                if start > 0 && is_base64_byte(input[start - 1]) {
                    start -= 1;
                }
                if input.get(end).is_some_and(|byte| is_base64_byte(*byte)) {
                    end += 1;
                }
                let padded = end + BASE64_SLACK;
                while end < input.len() && end < padded && input[end] == b'=' {
                    end += 1;
                }
            }
            found.push(Match {
                start,
                end,
                label: Arc::clone(&rep.label),
                class: rep.class,
            });
        }
    }
    found.sort_by_key(|m| (m.start, m.end));
    found
}

/// Length of the longest suffix of `buffer` that is a proper prefix of some
/// representation (0 when no tail could grow into a value). `reps` is sorted,
/// so the first representation not below a suffix is the only candidate that
/// can start with it.
fn longest_partial_suffix(snapshot: &Snapshot, buffer: &[u8]) -> usize {
    let longest = snapshot.max_len.saturating_sub(1).min(buffer.len());
    for start in buffer.len() - longest..buffer.len() {
        let suffix = &buffer[start..];
        let index = snapshot
            .reps
            .partition_point(|rep| rep.bytes.as_slice() <= suffix);
        // Every representation that starts with `suffix` and is longer sorts
        // right after `suffix` itself.
        if snapshot
            .reps
            .get(index)
            .is_some_and(|rep| rep.bytes.starts_with(suffix))
        {
            return suffix.len();
        }
    }
    0
}

fn lookup<'a>(snapshot: &'a Snapshot, bytes: &[u8]) -> Option<&'a Rep> {
    snapshot
        .reps
        .binary_search_by(|rep| rep.bytes.as_slice().cmp(bytes))
        .ok()
        .map(|index| &snapshot.reps[index])
}

fn redact_ranges(input: &[u8], found: &[Match], provenance: &mut Provenance) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());
    let mut position = 0;
    let mut index = 0;
    let mut labels: BTreeSet<String> = BTreeSet::new();
    while index < found.len() {
        let start = found[index].start;
        let mut end = found[index].end;
        let mut range_labels: BTreeSet<&str> = BTreeSet::new();
        range_labels.insert(&found[index].label);
        index += 1;
        while index < found.len() && found[index].start <= end {
            end = end.max(found[index].end);
            range_labels.insert(&found[index].label);
            index += 1;
        }
        output.extend_from_slice(&input[position.min(start)..start]);
        let names = range_labels.iter().copied().collect::<Vec<_>>().join(",");
        output.extend_from_slice(format!("[REDACTED:{names}]").as_bytes());
        labels.extend(range_labels.into_iter().map(str::to_string));
        provenance.redacted += 1;
        position = end;
    }
    output.extend_from_slice(&input[position.min(input.len())..]);
    provenance.labels = labels.into_iter().collect();
    output
}

fn withheld_marker(class: SecretClass, label: &str) -> Vec<u8> {
    format!(
        "[WITHHELD: output contained a protected {} ({label}); Corbanu does not disclose it]",
        class.as_str()
    )
    .into_bytes()
}

fn is_base64_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'-' | b'_')
}

fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn sanitize_label(label: &str) -> String {
    let cleaned: String = label
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | ':' | '/'))
        .take(64)
        .collect();
    if cleaned.is_empty() {
        "managed".to_string()
    } else {
        cleaned
    }
}

/// Raw value first, then its encodings, deduplicated; also the index range of
/// the base64 cores.
fn representations(value: &str) -> (Vec<Zeroizing<Vec<u8>>>, std::ops::Range<usize>) {
    let raw = value.as_bytes();
    let mut reps: Vec<Zeroizing<Vec<u8>>> = Vec::new();
    let push = |candidate: Vec<u8>, reps: &mut Vec<Zeroizing<Vec<u8>>>| {
        let candidate = Zeroizing::new(candidate);
        if candidate.len() <= MAX_REPRESENTATION_BYTES
            && !reps
                .iter()
                .any(|rep| rep.as_slice() == candidate.as_slice())
        {
            reps.push(candidate);
        }
    };
    push(raw.to_vec(), &mut reps);
    if let Ok(json) = serde_json::to_string(value) {
        let inner = Zeroizing::new(json);
        push(inner[1..inner.len() - 1].as_bytes().to_vec(), &mut reps);
    }
    for upper in [true, false] {
        push(percent_encode(raw, upper, /*form*/ false), &mut reps);
    }
    if raw.contains(&b' ') {
        push(percent_encode(raw, true, /*form*/ true), &mut reps);
    }
    let base64_start = reps.len();
    let mut base64_end = reps.len();
    if raw.len() >= MIN_SUBSTRING_BYTES && raw.len() <= MAX_ENCODED_VALUE_BYTES {
        for engine in [&STANDARD_NO_PAD, &URL_SAFE_NO_PAD] {
            for offset in 0..3usize {
                let mut shifted = Zeroizing::new(vec![0u8; offset]);
                shifted.extend_from_slice(raw);
                let encoded = Zeroizing::new(engine.encode(shifted.as_slice()));
                // Only characters whose six bits all come from the value.
                let start = (offset * 8).div_ceil(6);
                let end = (offset * 8 + raw.len() * 8) / 6;
                if end > start && end - start >= MIN_ENCODED_BYTES {
                    push(encoded.as_bytes()[start..end].to_vec(), &mut reps);
                }
            }
        }
        base64_end = reps.len();
        push(hex(raw, false), &mut reps);
        push(hex(raw, true), &mut reps);
    }
    (reps, base64_start..base64_end)
}

fn percent_encode(raw: &[u8], upper: bool, form: bool) -> Vec<u8> {
    let digits: &[u8; 16] = if upper {
        b"0123456789ABCDEF"
    } else {
        b"0123456789abcdef"
    };
    let mut out = Vec::with_capacity(raw.len() * 3);
    for &byte in raw {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(byte);
        } else if form && byte == b' ' {
            out.push(b'+');
        } else {
            out.extend_from_slice(&[
                b'%',
                digits[usize::from(byte >> 4)],
                digits[usize::from(byte & 15)],
            ]);
        }
    }
    out
}

fn hex(raw: &[u8], upper: bool) -> Vec<u8> {
    let digits: &[u8; 16] = if upper {
        b"0123456789ABCDEF"
    } else {
        b"0123456789abcdef"
    };
    raw.iter()
        .flat_map(|byte| {
            [
                digits[usize::from(byte >> 4)],
                digits[usize::from(byte & 15)],
            ]
        })
        .collect()
}

/// `None` when the matcher cannot be built (it would leave values unprotected).
fn compile(state: &State) -> Option<Snapshot> {
    let mut all: Vec<Rep> = state
        .entries
        .iter()
        .flat_map(|entry| {
            let label: Arc<str> = Arc::from(entry.label.as_str());
            entry
                .reps
                .iter()
                .enumerate()
                .map(move |(index, bytes)| Rep {
                    bytes: Zeroizing::new(bytes.to_vec()),
                    label: Arc::clone(&label),
                    class: entry.class,
                    whole_word: entry.whole_word,
                    base64: entry.base64.contains(&index),
                })
        })
        .collect();
    all.sort_by(|a, b| a.bytes.as_slice().cmp(b.bytes.as_slice()));
    // The same bytes under two values: strictest class, and substring
    // matching wins over whole-word.
    let mut reps: Vec<Rep> = Vec::with_capacity(all.len());
    for rep in all {
        match reps.last_mut() {
            Some(last) if last.bytes.as_slice() == rep.bytes.as_slice() => {
                last.class = last.class.max(rep.class);
                last.whole_word &= rep.whole_word;
                last.base64 |= rep.base64;
            }
            _ => reps.push(rep),
        }
    }
    let max_len = reps.iter().map(|rep| rep.bytes.len()).max().unwrap_or(0);
    Some(Snapshot {
        substring: alternation(reps.iter().filter(|rep| !rep.whole_word))?,
        whole_word: alternation(reps.iter().filter(|rep| rep.whole_word))?,
        reps,
        max_len,
    })
}

/// Longest-first literal alternation, so leftmost-first picks the longest
/// representation at each start. `Some(None)` when there is nothing to match.
fn alternation<'a>(reps: impl Iterator<Item = &'a Rep>) -> Option<Option<Regex>> {
    let mut literals: Vec<&[u8]> = reps.map(|rep| rep.bytes.as_slice()).collect();
    if literals.is_empty() {
        return Some(None);
    }
    literals.sort_by_key(|literal| std::cmp::Reverse(literal.len()));
    let mut pattern = Zeroizing::new(String::new());
    for (index, literal) in literals.iter().enumerate() {
        if index > 0 {
            pattern.push('|');
        }
        for &byte in *literal {
            if byte.is_ascii_alphanumeric() {
                pattern.push(char::from(byte));
            } else {
                pattern.push_str(&format!("\\x{byte:02X}"));
            }
        }
    }
    RegexBuilder::new(&pattern)
        .unicode(false)
        .size_limit(256 << 20)
        .dfa_size_limit(64 << 20)
        .build()
        .ok()
        .map(Some)
}

#[cfg(test)]
#[path = "output_gate_tests.rs"]
mod tests;
