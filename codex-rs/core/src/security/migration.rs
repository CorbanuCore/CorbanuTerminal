//! PF-29-S02 human-reviewed credential migration (feature
//! `protected_mode_preflight`).
//!
//! Moves the raw credentials the [preflight](super::preflight) found in shell
//! profiles into the encrypted vault and replaces each line with a vault
//! reference (`export NAME="$(corbanu vault auth-helper LABEL)"`). Nothing
//! happens until a human confirms the [`MigrationPlan`] preview.
//!
//! Guarantees, bounded on purpose:
//! - one migration at a time: the journal (`security_migration.toml`) is
//!   created exclusively and holds labels, paths and stages, never a value;
//! - values go into the vault before any file changes; the vault's encrypted
//!   copy is the only recovery data; there is no plaintext backup;
//! - a line is rewritten only while it still holds the value that was moved,
//!   so an edit made after the preview (a later owner) is never overwritten;
//! - files are replaced atomically (`0600`, synced, directory synced);
//! - an interrupted migration keeps blocking protected levels (the preflight
//!   reports the journal) until [`recover`] rolls it forward. Recovery never
//!   rolls back to plaintext and never changes the security level.
//!
//! Only shell-profile exports are migrated. Every other blocker is listed as
//! unsupported with what the human must do; all moved or exposed credentials
//! are listed for rotation, because agents may have read them before.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use zeroize::Zeroizing;

use super::inventory::Disposition;
use super::inventory::FindingKind;
use super::preflight::Preflight;

pub const JOURNAL_FILE: &str = "security_migration.toml";
const JOURNAL_VERSION: u32 = 1;
const LABEL_PREFIX: &str = "migrated/";
/// Shell profiles are small; anything larger is not rewritten.
const MAX_PROFILE_BYTES: u64 = 1024 * 1024;

pub fn journal_path(codex_home: &Path) -> PathBuf {
    codex_home.join(JOURNAL_FILE)
}

/// One credential to move.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannedMove {
    pub finding_id: String,
    pub path: PathBuf,
    /// 1-based line in `path`.
    pub line: usize,
    pub name: String,
    /// Vault label the value moves to.
    pub label: String,
    /// Shown in the preview: where it was, without the value.
    pub location: String,
}

impl PlannedMove {
    /// What replaces the value in the profile.
    pub fn reference(&self) -> String {
        if self.is_fish() {
            format!("(corbanu vault auth-helper {})", self.label)
        } else {
            format!("\"$(corbanu vault auth-helper {})\"", self.label)
        }
    }

    fn is_fish(&self) -> bool {
        self.path.extension().is_some_and(|ext| ext == "fish")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedItem {
    pub finding_id: String,
    pub location: String,
    pub action: &'static str,
}

/// The preview a human confirms. No values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationPlan {
    pub moves: Vec<PlannedMove>,
    pub unsupported: Vec<UnsupportedItem>,
}

impl MigrationPlan {
    /// From the blockers of a reviewed preflight. `taken` holds vault labels
    /// already in use, so a new label never replaces an existing credential.
    pub fn from_preflight(preflight: &Preflight, taken: &BTreeSet<String>) -> Self {
        let mut used = taken.clone();
        let mut moves = Vec::new();
        let mut unsupported = Vec::new();
        for finding in preflight.inventory.blocking() {
            let check = match (finding.kind, &finding.source_line, finding.paths.first()) {
                (FindingKind::ShellProfileExport, Some((line, name)), Some(path)) => {
                    Some(migratable(path, *line, name))
                }
                _ => None,
            };
            match (finding.kind, &finding.source_line, finding.paths.first()) {
                (FindingKind::ShellProfileExport, Some(_), Some(_))
                    if matches!(check, Some(Err(_))) =>
                {
                    unsupported.push(UnsupportedItem {
                        finding_id: finding.id.clone(),
                        location: finding.location.clone(),
                        action: match check {
                            Some(Err(action)) => action,
                            _ => "edit it by hand",
                        },
                    });
                }
                (FindingKind::ShellProfileExport, Some((line, name)), Some(path)) => {
                    let label = unique_label(name, &mut used);
                    moves.push(PlannedMove {
                        finding_id: finding.id.clone(),
                        path: path.clone(),
                        line: *line,
                        name: name.clone(),
                        label,
                        location: finding.location.clone(),
                    });
                }
                (kind, _, _) => unsupported.push(UnsupportedItem {
                    finding_id: finding.id.clone(),
                    location: finding.location.clone(),
                    action: unsupported_action(kind, finding.disposition),
                }),
            }
        }
        Self { moves, unsupported }
    }

    /// Paths whose access is restricted to the owner after the move.
    pub fn restricted_files(&self) -> Vec<PathBuf> {
        self.moves
            .iter()
            .map(|planned| planned.path.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}

fn unsupported_action(kind: FindingKind, disposition: Disposition) -> &'static str {
    match (kind, disposition) {
        (_, Disposition::RemoveFromContext) => {
            "remove the secret from this memory file, or delete the file"
        }
        (FindingKind::McpSecretLiteral | FindingKind::ProviderSecretLiteral, _) => {
            "Corbanu cannot rewrite this config value yet: read it from an environment variable instead (env_vars, bearer_token_env_var or env_key), then rotate it"
        }
        (FindingKind::EnvironmentVariable, _) => {
            "it is set outside Corbanu: remove it where it is set and restart"
        }
        (FindingKind::ConfigLiteral, _) => "stop passing it on the command line",
        _ => "move it into the vault yourself, or remove it",
    }
}

fn unique_label(name: &str, used: &mut BTreeSet<String>) -> String {
    let base: String = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    let mut label = format!("{LABEL_PREFIX}{base}");
    let mut suffix = 2;
    while used.contains(&label) {
        label = format!("{LABEL_PREFIX}{base}-{suffix}");
        suffix += 1;
    }
    used.insert(label.clone());
    label
}

/// Where values go. Implemented over the vault by the caller.
pub trait CredentialStore {
    /// Adds a new credential; must fail if the label exists.
    fn put(&self, label: &str, name: &str, origin: &str, value: &str) -> Result<(), String>;
    /// The stored value, for checking a line before it is rewritten.
    fn get(&self, label: &str) -> Result<Option<Zeroizing<String>>, String>;
    /// Labels already in use.
    fn labels(&self) -> Result<BTreeSet<String>, String> {
        Ok(BTreeSet::new())
    }
}

/// Points where a test (or the debug-only demo hook) stops the migration as
/// if the process had crashed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailPoint {
    /// After the journal is created, before any value is stored.
    Prepared,
    /// After the first value is stored.
    Stored,
    /// After the first file is rewritten, before the journal records it.
    Rewritten,
    /// After every file is rewritten, before the journal is committed.
    Committed,
}

impl FailPoint {
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "prepared" => Some(Self::Prepared),
            "stored" => Some(Self::Stored),
            "rewritten" => Some(Self::Rewritten),
            "committed" => Some(Self::Committed),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EntryState {
    Planned,
    Stored,
    Rewritten,
    /// Changed after the preview; left alone and reported.
    Skipped,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct JournalEntry {
    finding_id: String,
    path: PathBuf,
    line: usize,
    name: String,
    label: String,
    location: String,
    /// The file's size, modification time and inode when the human
    /// confirmed. Until a value is stored nothing in the file is ours, so a
    /// different stamp means someone else edited it.
    file_stamp: String,
    state: EntryState,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    version: u32,
    id: String,
    /// Bumped on every write; a reader can tell a stale copy.
    generation: u64,
    started_at: i64,
    entries: Vec<JournalEntry>,
}

/// What happened, for the human. No values.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MigrationOutcome {
    /// `(location, label)` for every credential now referenced from the vault.
    pub moved: Vec<(String, String)>,
    /// `(location, reason)` for lines left alone.
    pub skipped: Vec<(String, String)>,
    /// Variable names whose old values must be rotated at their provider.
    pub rotate: Vec<String>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MigrationError {
    #[error("another credential migration is unfinished; finish it from /security first")]
    InProgress,
    #[error(
        "the migration stopped at {stage}: {reason}. Values already moved are in the vault; nothing was rolled back. Open /security to finish it"
    )]
    Interrupted { stage: &'static str, reason: String },
    #[error(
        "the migration record {0} cannot be read. Check the profiles it names by hand (each line should hold a vault reference or its original value), then delete that file"
    )]
    Journal(String),
}

/// Runs a confirmed plan. The caller has already rechecked the preflight for
/// drift since the preview.
pub fn run(
    codex_home: &Path,
    plan: &MigrationPlan,
    store: &dyn CredentialStore,
    fail_at: Option<FailPoint>,
) -> Result<MigrationOutcome, MigrationError> {
    let mut journal = Journal {
        version: JOURNAL_VERSION,
        id: uuid::Uuid::new_v4().to_string(),
        generation: 0,
        started_at: now_ms(),
        entries: plan
            .moves
            .iter()
            .map(|planned| JournalEntry {
                finding_id: planned.finding_id.clone(),
                path: planned.path.clone(),
                line: planned.line,
                name: planned.name.clone(),
                label: planned.label.clone(),
                location: planned.location.clone(),
                file_stamp: file_stamp(&planned.path),
                state: EntryState::Planned,
            })
            .collect(),
    };
    create_journal(codex_home, &mut journal)?;
    stop_if(fail_at, FailPoint::Prepared, "prepared")?;
    roll_forward(codex_home, &mut journal, store, fail_at)
}

/// Finishes an interrupted migration. `Ok(None)` when there is none.
pub fn recover(
    codex_home: &Path,
    store: &dyn CredentialStore,
) -> Result<Option<MigrationOutcome>, MigrationError> {
    let path = journal_path(codex_home);
    let contents = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(MigrationError::Journal(format!(
                "{}: {err}",
                path.display()
            )));
        }
    };
    let mut journal: Journal = toml::from_str(&contents)
        .map_err(|err| MigrationError::Journal(format!("{}: {}", path.display(), err.message())))?;
    if journal.version != JOURNAL_VERSION {
        return Err(MigrationError::Journal(format!(
            "{}: unsupported version {}",
            path.display(),
            journal.version
        )));
    }
    roll_forward(codex_home, &mut journal, store, /*fail_at*/ None).map(Some)
}

fn stop_if(
    fail_at: Option<FailPoint>,
    point: FailPoint,
    stage: &'static str,
) -> Result<(), MigrationError> {
    if fail_at == Some(point) {
        return Err(MigrationError::Interrupted {
            stage,
            reason: "injected failure (test)".to_string(),
        });
    }
    Ok(())
}

fn interrupted(stage: &'static str) -> impl Fn(String) -> MigrationError {
    move |reason| MigrationError::Interrupted { stage, reason }
}

/// Stores every planned value, rewrites every stored line, then commits.
/// Each step is recorded before the next one starts, so running this again
/// after any interruption converges on the same result.
fn roll_forward(
    codex_home: &Path,
    journal: &mut Journal,
    store: &dyn CredentialStore,
    fail_at: Option<FailPoint>,
) -> Result<MigrationOutcome, MigrationError> {
    for index in 0..journal.entries.len() {
        if journal.entries[index].state != EntryState::Planned {
            continue;
        }
        let entry = journal.entries[index].clone();
        let next = match store.get(&entry.label).map_err(interrupted("store"))? {
            // Stored before an interruption, not yet recorded.
            Some(_) => EntryState::Stored,
            // Edited since the confirmation: the line may hold someone
            // else's value now.
            None if file_stamp(&entry.path) != entry.file_stamp => EntryState::Skipped,
            None => match read_line_value(&entry).map_err(interrupted("store"))? {
                Some(value) => {
                    store
                        .put(&entry.label, &entry.name, &entry.location, &value)
                        .map_err(interrupted("store"))?;
                    EntryState::Stored
                }
                None => EntryState::Skipped,
            },
        };
        journal.entries[index].state = next;
        write_journal(codex_home, journal).map_err(interrupted("store"))?;
        stop_if(fail_at, FailPoint::Stored, "store")?;
    }

    let mut by_file = BTreeMap::<PathBuf, Vec<usize>>::new();
    for (index, entry) in journal.entries.iter().enumerate() {
        if entry.state == EntryState::Stored {
            by_file.entry(entry.path.clone()).or_default().push(index);
        }
    }
    for (path, indices) in by_file {
        let states = rewrite_file(&path, &journal.entries, &indices, store)
            .map_err(interrupted("rewrite"))?;
        stop_if(fail_at, FailPoint::Rewritten, "rewrite")?;
        for (index, state) in indices.into_iter().zip(states) {
            journal.entries[index].state = state;
        }
        write_journal(codex_home, journal).map_err(interrupted("rewrite"))?;
    }
    stop_if(fail_at, FailPoint::Committed, "commit")?;

    let outcome = outcome(journal);
    match std::fs::remove_file(journal_path(codex_home)) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => return Err(interrupted("commit")(err.to_string())),
    }
    sync_dir(codex_home).map_err(interrupted("commit"))?;
    Ok(outcome)
}

fn outcome(journal: &Journal) -> MigrationOutcome {
    let mut result = MigrationOutcome::default();
    for entry in &journal.entries {
        match entry.state {
            EntryState::Rewritten => {
                result
                    .moved
                    .push((entry.location.clone(), entry.label.clone()));
                result.rotate.push(entry.name.clone());
            }
            EntryState::Skipped => {
                result.skipped.push((
                    entry.location.clone(),
                    "the line changed after the preview; it was left as it is".to_string(),
                ));
                result.rotate.push(entry.name.clone());
            }
            // Not reached after a full roll-forward; reported, never hidden.
            EntryState::Planned | EntryState::Stored => {
                result.skipped.push((
                    entry.location.clone(),
                    "the migration did not reach this line".to_string(),
                ));
                result.rotate.push(entry.name.clone());
            }
        }
    }
    result.rotate.sort();
    result.rotate.dedup();
    result
}

fn file_stamp(path: &Path) -> String {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return "missing".to_string();
    };
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |elapsed| elapsed.as_nanos());
    #[cfg(unix)]
    let inode = {
        use std::os::unix::fs::MetadataExt as _;
        metadata.ino()
    };
    #[cfg(not(unix))]
    let inode = 0u64;
    format!("{}:{modified}:{inode}", metadata.len())
}

/// `(device, inode)`, to notice a file swapped in before a rename.
type FileIdentity = Option<(u64, u64)>;

fn identity(metadata: &std::fs::Metadata) -> FileIdentity {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;
        Some((metadata.dev(), metadata.ino()))
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        None
    }
}

/// More than one name for the file: replacing it by rename would leave the
/// plain text behind under the other names.
fn hard_linked(metadata: &std::fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;
        metadata.nlink() > 1
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        false
    }
}

/// Reads a profile without following a link, refusing anything but a small,
/// singly linked regular file.
fn read_profile(path: &Path) -> Result<(Zeroizing<String>, FileIdentity), String> {
    use std::io::Read as _;
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options
        .open(path)
        .map_err(|err| format!("{}: {err}", path.display()))?;
    let metadata = file.metadata().map_err(|err| err.to_string())?;
    if !metadata.is_file() || hard_linked(&metadata) {
        return Err(format!(
            "{} is not a singly linked regular file",
            path.display()
        ));
    }
    if metadata.len() > MAX_PROFILE_BYTES {
        return Err(format!("{} is too large to rewrite", path.display()));
    }
    let mut text = Zeroizing::new(String::new());
    file.take(MAX_PROFILE_BYTES)
        .read_to_string(&mut text)
        .map_err(|err| err.to_string())?;
    Ok((text, identity(&metadata)))
}

/// One shell assignment as migration understands it.
struct Assignment<'a> {
    /// The line without its line ending.
    content: &'a str,
    name: &'a str,
    value_start: usize,
    /// Fish syntax (`set`), where `\\` escapes inside single quotes.
    fish: bool,
}

/// `[export |declare -x ]NAME=value` or fish `set [-flags] NAME value`,
/// optionally indented. Byte offsets are valid in the original line.
fn parse_line(line: &str) -> Option<Assignment<'_>> {
    let content = line.trim_end_matches(['\n', '\r']);
    let indent = content.len() - content.trim_start().len();
    let rest = &content[indent..];
    let valid_name = |name: &str| {
        name.chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
            && name
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    };
    if let Some(after) = rest.strip_prefix("set ") {
        let base = indent + 4;
        let mut offset = 0;
        for token in after.split(' ') {
            let start = offset;
            offset += token.len() + 1;
            if token.is_empty() || token.starts_with('-') {
                continue;
            }
            if !valid_name(token) {
                return None;
            }
            let value = after.get(start + token.len()..)?;
            let skipped = value.len() - value.trim_start().len();
            return Some(Assignment {
                content,
                name: token,
                value_start: base + start + token.len() + skipped,
                fish: true,
            });
        }
        return None;
    }
    let (prefix, after) = ["export ", "declare -x "]
        .iter()
        .find_map(|prefix| rest.strip_prefix(prefix).map(|after| (prefix.len(), after)))
        .unwrap_or((0, rest));
    let skipped = after.len() - after.trim_start().len();
    let after = &after[skipped..];
    let (name, _) = after.split_once('=')?;
    valid_name(name).then_some(Assignment {
        content,
        name,
        value_start: indent + prefix + skipped + name.len() + 1,
        fish: false,
    })
}

impl Assignment<'_> {
    /// What follows a value token: nothing, or whitespace and a comment.
    fn ends_cleanly(&self, end: usize) -> bool {
        let tail = &self.content[end..];
        let trimmed = tail.trim_start();
        trimmed.is_empty() || (trimmed.starts_with('#') && trimmed.len() < tail.len())
    }

    /// The value token's range and its literal when it is one plain value:
    /// `'…'`, `"…"` without `$`, `` ` `` or `\`, or an unquoted word without
    /// shell syntax, followed by nothing but a comment. Anything else (a
    /// second command, several words, a substitution) is not migrated.
    fn literal(&self) -> Option<(std::ops::Range<usize>, &str)> {
        let value = &self.content[self.value_start..];
        let (len, literal) = match value.chars().next()? {
            quote @ ('"' | '\'') => {
                let close = value[1..].find(quote)? + 1;
                let inner = &value[1..close];
                let escapes = if quote == '"' {
                    inner.contains(['$', '`', '\\'])
                } else {
                    self.fish && inner.contains('\\')
                };
                if escapes {
                    return None;
                }
                (close + 1, inner)
            }
            _ => {
                let end = value.find(char::is_whitespace).unwrap_or(value.len());
                let word = &value[..end];
                // Shell syntax, globs, braces and `~` would change the value.
                if word.contains([
                    ';', '&', '|', '`', '$', '\\', '"', '\'', '(', ')', '<', '>', '~', '{', '}',
                    '*', '?', '[',
                ]) {
                    return None;
                }
                (end, word)
            }
        };
        let end = self.value_start + len;
        (!literal.is_empty() && self.ends_cleanly(end)).then_some((self.value_start..end, literal))
    }

    /// Whether the value is exactly `reference` (a rewrite that finished).
    fn holds(&self, reference: &str) -> bool {
        self.content[self.value_start..]
            .strip_prefix(reference)
            .is_some_and(|_| self.ends_cleanly(self.value_start + reference.len()))
    }
}

/// Whether a planned line can be migrated as it is now.
fn migratable(path: &Path, line: usize, name: &str) -> Result<(), &'static str> {
    let Ok((text, _)) = read_profile(path) else {
        return Err(
            "this profile cannot be rewritten safely (a link, several hard links, or unreadable); move the value into the vault yourself",
        );
    };
    let parsed = text
        .lines()
        .nth(line.saturating_sub(1))
        .and_then(parse_line);
    match parsed {
        Some(assignment) if assignment.name == name && assignment.literal().is_some() => Ok(()),
        _ => Err(
            "the line is not a single plain NAME=value (quoting, a second command or several words); edit it by hand",
        ),
    }
}

/// The literal value on the entry's line if it is still the same variable.
fn read_line_value(entry: &JournalEntry) -> Result<Option<Zeroizing<String>>, String> {
    let (text, _) = read_profile(&entry.path)?;
    Ok(text
        .lines()
        .nth(entry.line.saturating_sub(1))
        .and_then(parse_line)
        .filter(|assignment| assignment.name == entry.name)
        .and_then(|assignment| {
            assignment
                .literal()
                .map(|(_, value)| Zeroizing::new(value.to_string()))
        }))
}

/// Replaces each stored line that still holds the stored value with its
/// reference. A line that already holds the reference counts as rewritten
/// (a crash after the rename); anything else is skipped.
fn rewrite_file(
    path: &Path,
    entries: &[JournalEntry],
    indices: &[usize],
    store: &dyn CredentialStore,
) -> Result<Vec<EntryState>, String> {
    let (text, read_identity) = read_profile(path)?;
    let read_stamp = file_stamp(path);
    let mut lines: Vec<String> = text.split_inclusive('\n').map(str::to_string).collect();
    let mut states = Vec::with_capacity(indices.len());
    let mut changed = false;
    for &index in indices {
        let entry = &entries[index];
        let planned = PlannedMove {
            finding_id: entry.finding_id.clone(),
            path: entry.path.clone(),
            line: entry.line,
            name: entry.name.clone(),
            label: entry.label.clone(),
            location: entry.location.clone(),
        };
        let reference = planned.reference();
        let Some(line) = lines.get_mut(entry.line.saturating_sub(1)) else {
            states.push(EntryState::Skipped);
            continue;
        };
        let Some(assignment) = parse_line(line).filter(|parsed| parsed.name == entry.name) else {
            states.push(EntryState::Skipped);
            continue;
        };
        if assignment.holds(&reference) {
            states.push(EntryState::Rewritten);
            continue;
        }
        let stored = store.get(&entry.label)?;
        let span = assignment.literal().and_then(|(span, value)| {
            stored
                .as_deref()
                .is_some_and(|stored| stored.as_str() == value)
                .then_some(span)
        });
        match span {
            Some(span) => {
                line.replace_range(span, &reference);
                changed = true;
                states.push(EntryState::Rewritten);
            }
            None => states.push(EntryState::Skipped),
        }
    }
    // The file must still be the one that was read: same identity, no new
    // hard link, no edit since.
    let current = std::fs::symlink_metadata(path).map_err(|err| err.to_string())?;
    if identity(&current) != read_identity
        || !current.is_file()
        || hard_linked(&current)
        || file_stamp(path) != read_stamp
    {
        return Err(format!(
            "{} changed while it was being rewritten",
            path.display()
        ));
    }
    if changed {
        replace_file(path, &Zeroizing::new(lines.concat()))?;
    } else {
        restrict(path)?;
    }
    Ok(states)
}

/// Atomic replace with owner-only access; no backup copy is kept.
fn replace_file(path: &Path, contents: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("{} has no parent", path.display()))?;
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|err| err.to_string())?;
    file.write_all(contents.as_bytes())
        .map_err(|err| err.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        file.as_file()
            .set_permissions(std::fs::Permissions::from_mode(0o600))
            .map_err(|err| err.to_string())?;
    }
    file.as_file().sync_all().map_err(|err| err.to_string())?;
    file.persist(path).map_err(|err| err.error.to_string())?;
    sync_dir(parent)
}

fn restrict(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .map_err(|err| err.to_string())?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

fn sync_dir(dir: &Path) -> Result<(), String> {
    #[cfg(unix)]
    std::fs::File::open(dir)
        .and_then(|dir| dir.sync_all())
        .map_err(|err| err.to_string())?;
    #[cfg(not(unix))]
    let _ = dir;
    Ok(())
}

fn create_journal(codex_home: &Path, journal: &mut Journal) -> Result<(), MigrationError> {
    let path = journal_path(codex_home);
    let contents =
        toml::to_string(journal).map_err(|err| interrupted("prepare")(err.to_string()))?;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o600);
    }
    let mut file = match options.open(&path) {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(MigrationError::InProgress);
        }
        Err(err) => return Err(interrupted("prepare")(format!("{}: {err}", path.display()))),
    };
    file.write_all(contents.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|err| interrupted("prepare")(format!("{}: {err}", path.display())))?;
    sync_dir(codex_home).map_err(interrupted("prepare"))
}

fn write_journal(codex_home: &Path, journal: &mut Journal) -> Result<(), String> {
    journal.generation += 1;
    let contents = toml::to_string(journal).map_err(|err| err.to_string())?;
    let path = journal_path(codex_home);
    let mut file = tempfile::NamedTempFile::new_in(codex_home).map_err(|err| err.to_string())?;
    file.write_all(contents.as_bytes())
        .map_err(|err| err.to_string())?;
    file.as_file().sync_all().map_err(|err| err.to_string())?;
    file.persist(&path).map_err(|err| err.error.to_string())?;
    sync_dir(codex_home)
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| {
            i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
        })
}
