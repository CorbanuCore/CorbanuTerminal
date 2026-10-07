//! Path resolution and protected-path classification for the post-taint
//! classifier. Lexical first; bounded, cached filesystem lookups then follow
//! symlinks and other non-canonical spellings of a home. Lookups never touch
//! locations that could reach the network or raise an OS privacy prompt.

use super::ProtectedActionKind;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

/// Folders whose contents are credentials wherever they are.
const CREDENTIAL_SEGMENTS: &[&str] = &[".ssh", ".aws", ".gnupg", ".kube"];
/// Credential folders only in the user's home (projects often have their own).
const USER_CREDENTIAL_SEGMENTS: &[&str] = &[".docker", ".azure", ".config/gh", ".config/gcloud"];
const CREDENTIAL_FILES: &[&str] = &[
    ".credentials.json",
    ".netrc",
    ".git-credentials",
    ".npmrc",
    ".pypirc",
    // Network tool settings: may hold credentials or upload directives.
    ".curlrc",
    ".wgetrc",
    "id_rsa",
    "id_ecdsa",
    "id_ed25519",
    "id_dsa",
];
/// Corbanu home folder names (besides the configured `CODEX_HOME`).
const HOME_SEGMENTS: &[&str] = &[".corbanu", ".codex", ".pfterminal"];
/// Uncached filesystem lookups one classification may make. Past it the
/// classification fails closed.
const FS_LOOKUP_BUDGET: usize = 2048;
/// Never looked up: automounts and network or removable volumes (a lookup can
/// send traffic or block), unless inside an allowed root. On macOS `/home`
/// is an automount too.
const NO_LOOKUP_PREFIXES: &[&str] = &["/net", "/network", "/volumes", "/mnt", "/media"];
/// User folders behind macOS privacy prompts.
const PRIVACY_FOLDERS: &[&str] = &[
    "desktop",
    "documents",
    "downloads",
    "library",
    "movies",
    "music",
    "pictures",
];

pub(super) struct Homes {
    /// Original spellings, used to expand `$CODEX_HOME`, `$HOME` and `~`.
    codex_home_raw: String,
    user_home_raw: Option<String>,
    /// Lowercase spellings to match against: as configured and canonical.
    codex_homes: Vec<String>,
    user_homes: Vec<String>,
    /// Lowercase folders where lookups are always allowed (the action's
    /// working folder, the homes, temp folders).
    lookup_roots: RefCell<Vec<String>>,
    cache: RefCell<HashMap<String, Option<String>>>,
    lookups: Cell<usize>,
    exhausted: Cell<bool>,
}

impl Homes {
    pub(super) fn new(codex_home: &Path, user_home: Option<&Path>) -> Self {
        let raw = |path: &Path| path.to_string_lossy().trim_end_matches('/').to_string();
        let codex_home_raw = raw(codex_home);
        let user_home_raw = user_home.map(raw);
        let mut lookup_roots = vec![
            "/tmp".to_string(),
            "/private/tmp".to_string(),
            "/var/folders".to_string(),
            "/private/var/folders".to_string(),
        ];
        lookup_roots.extend(
            std::env::temp_dir()
                .to_str()
                .map(|temp| normalize_path(&temp.to_lowercase())),
        );
        if !codex_home_raw.is_empty() {
            lookup_roots.push(normalize_path(&codex_home_raw.to_lowercase()));
        }
        let homes = Self {
            codex_home_raw,
            user_home_raw,
            codex_homes: Vec::new(),
            user_homes: Vec::new(),
            lookup_roots: RefCell::new(lookup_roots),
            cache: RefCell::new(HashMap::new()),
            lookups: Cell::new(0),
            exhausted: Cell::new(false),
        };
        let spellings = |raw: &str| {
            let mut spellings = vec![normalize_path(&raw.to_lowercase())];
            if let Some(canonical) = homes.canonical(raw)
                && !spellings.contains(&canonical)
            {
                spellings.push(canonical);
            }
            spellings
        };
        let codex_homes = if homes.codex_home_raw.is_empty() {
            Vec::new()
        } else {
            spellings(&homes.codex_home_raw)
        };
        let user_homes = homes
            .user_home_raw
            .as_deref()
            .map(spellings)
            .unwrap_or_default();
        homes
            .lookup_roots
            .borrow_mut()
            .extend(codex_homes.iter().cloned());
        Self {
            codex_homes,
            user_homes,
            ..homes
        }
    }

    /// Lookups are allowed under `folder` (the action's working folder).
    pub(super) fn allow_lookups_under(&self, folder: &str) {
        let folder = normalize_path(&folder.to_lowercase());
        let mut roots = self.lookup_roots.borrow_mut();
        if !roots.contains(&folder) {
            roots.push(folder);
        }
    }

    /// A lookup budget ran out, so some path was not followed.
    pub(super) fn exhausted(&self) -> bool {
        self.exhausted.get()
    }

    pub(super) fn codex_home(&self) -> &str {
        &self.codex_home_raw
    }

    pub(super) fn user_home(&self) -> Option<&str> {
        self.user_home_raw.as_deref()
    }

    /// Expand `~`, `~user`, `$HOME` and `$CODEX_HOME` spellings and make a
    /// relative word absolute against `cwd`. Keeps the word's case.
    pub(super) fn resolve(&self, word: &str, cwd: &str) -> String {
        let mut word = word.to_string();
        for (name, value) in [
            ("CODEX_HOME", Some(&self.codex_home_raw)),
            ("HOME", self.user_home_raw.as_ref()),
        ] {
            if let Some(value) = value {
                word = replace_variable(&word, name, value);
            }
        }
        if let Some(home) = &self.user_home_raw
            && let Some(rest) = word.strip_prefix('~')
        {
            let (user, tail) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
            if user.is_empty() {
                word = format!("{home}{tail}");
            } else if let Some(parent) = Path::new(home).parent() {
                // `~name` is a sibling of the user's home.
                word = format!("{}/{user}{tail}", parent.to_string_lossy());
            }
        }
        if word.starts_with('/') || cwd.is_empty() {
            normalize_path(&word)
        } else {
            normalize_path(&format!("{cwd}/{word}"))
        }
    }

    /// Classify a resolved path, also through its canonical form (symlinks,
    /// `/var` vs `/private/var`, letter case on case-insensitive volumes).
    pub(super) fn classify_resolved(&self, path: &str) -> Option<ProtectedActionKind> {
        self.classify_path(path).or_else(|| {
            self.canonical(path)
                .and_then(|canonical| self.classify_path(&canonical))
        })
    }

    /// Whether `path` (globs allowed) is the user's home, or a folder holding
    /// it or the Corbanu home: copying or archiving it takes credentials.
    pub(super) fn holds_a_home(&self, path: &str) -> bool {
        let lexical = normalize_path(&path.to_lowercase());
        let canonical = self.canonical(path);
        [Some(lexical), canonical]
            .into_iter()
            .flatten()
            .any(|path| {
                let pattern = segments(&path);
                let holds = |home: &String| {
                    let home = segments(home);
                    pattern.len() <= home.len()
                        && pattern
                            .iter()
                            .zip(&home)
                            .all(|(pattern, segment)| glob_any(pattern, segment))
                };
                self.user_homes.iter().any(holds) || self.codex_homes.iter().any(holds)
            })
    }

    /// Whether filesystem lookups may touch `path`.
    pub(super) fn lookups_allowed(&self, path: &str) -> bool {
        let lower = normalize_path(&path.to_lowercase());
        let under = |root: &str| lower == root || lower.starts_with(&format!("{root}/"));
        if self.lookup_roots.borrow().iter().any(|root| under(root)) {
            return true;
        }
        if NO_LOOKUP_PREFIXES.iter().any(|prefix| under(prefix))
            || (cfg!(target_os = "macos") && under("/home"))
        {
            return false;
        }
        !self.user_homes.iter().any(|home| {
            PRIVACY_FOLDERS
                .iter()
                .any(|folder| under(&format!("{home}/{folder}")))
        })
    }

    /// Canonical lowercase spelling of `path`, following symlinks. A path
    /// that does not exist yet keeps its missing tail on top of its nearest
    /// existing folder. `None` where lookups are not allowed or the budget
    /// is spent (which marks the classification as incomplete).
    pub(super) fn canonical(&self, path: &str) -> Option<String> {
        let path = normalize_path(path);
        if !self.lookups_allowed(&path) {
            return None;
        }
        let mut existing = PathBuf::from(&path);
        let mut tail: Vec<String> = Vec::new();
        loop {
            let key = existing.to_string_lossy().into_owned();
            let cached = self.cache.borrow().get(&key).cloned();
            let found = match cached {
                Some(found) => found,
                None => {
                    if self.lookups.get() >= FS_LOOKUP_BUDGET {
                        self.exhausted.set(true);
                        return None;
                    }
                    self.lookups.set(self.lookups.get() + 1);
                    let found = std::fs::canonicalize(&existing)
                        .ok()
                        .map(|canonical| canonical.to_string_lossy().into_owned());
                    self.cache.borrow_mut().insert(key, found.clone());
                    found
                }
            };
            if let Some(mut canonical) = found {
                for segment in tail.iter().rev() {
                    canonical = format!("{}/{segment}", canonical.trim_end_matches('/'));
                }
                return Some(normalize_path(&canonical.to_lowercase()));
            }
            let segment = existing.file_name()?.to_string_lossy().into_owned();
            tail.push(segment);
            if !existing.pop() {
                return None;
            }
        }
    }

    /// The protected kind of one absolute path. Glob segments count as any
    /// protected folder they could match.
    pub(super) fn classify_path(&self, path: &str) -> Option<ProtectedActionKind> {
        let path = normalize_path(&path.to_lowercase());
        let segments = segments(&path);
        let name = segments.last().copied().unwrap_or_default();
        if segments
            .iter()
            .any(|segment| matches_any(segment, CREDENTIAL_SEGMENTS))
            || USER_CREDENTIAL_SEGMENTS.iter().any(|folder| {
                self.user_homes.iter().any(|home| {
                    below(&segments, home).is_some_and(|rest| {
                        let expected: Vec<&str> = folder.split('/').collect();
                        rest.len() >= expected.len()
                            && expected.iter().zip(rest).all(|(expected, got)| {
                                glob_matches(got, expected)
                                    // `*` matches non-dot folder names.
                                    || (!expected.starts_with('.') && glob_any(got, expected))
                            })
                    })
                })
            })
            || CREDENTIAL_FILES.iter().any(|file| glob_matches(name, file))
            || (glob_matches(name, "hosts.yml")
                && segments.iter().any(|segment| glob_any(segment, "gh")))
            || name.starts_with("id_rsa")
            || name.starts_with("id_ed25519")
        {
            return Some(ProtectedActionKind::Credentials);
        }
        let below_home = self.below_home(&segments)?;
        let first = below_home.first().copied().unwrap_or("");
        // Agent worktrees kept under the home are ordinary workspaces.
        if first == "worktrees" {
            return None;
        }
        if name.contains("vault") || first.contains("vault") {
            return Some(ProtectedActionKind::Vault);
        }
        if name.starts_with("auth") || name.contains("credential") || name.ends_with(".key") {
            return Some(ProtectedActionKind::Credentials);
        }
        Some(ProtectedActionKind::SecurityPolicy)
    }

    /// The segments below a Corbanu home, if the path is inside one: the
    /// configured home as a run of segments anywhere in the path (globs in
    /// the path count), or a default home folder name as a segment.
    fn below_home<'a>(&self, segments: &[&'a str]) -> Option<Vec<&'a str>> {
        for home in &self.codex_homes {
            let home = self::segments(home);
            if home.is_empty() || home.len() > segments.len() {
                continue;
            }
            let found = (0..=segments.len() - home.len()).find(|start| {
                segments[*start..start + home.len()]
                    .iter()
                    .zip(&home)
                    .all(|(pattern, segment)| glob_any(pattern, segment))
            });
            if let Some(start) = found {
                return Some(segments[start + home.len()..].to_vec());
            }
        }
        let index = segments
            .iter()
            .position(|segment| matches_any(segment, HOME_SEGMENTS))?;
        Some(segments[index + 1..].to_vec())
    }
}

fn segments(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

/// The segments of `path` below `home` (path segments may be globs).
fn below<'a, 'b>(path: &'b [&'a str], home: &str) -> Option<&'b [&'a str]> {
    let home = segments(home);
    (path.len() > home.len()
        && path
            .iter()
            .zip(&home)
            .all(|(pattern, segment)| glob_any(pattern, segment)))
    .then(|| &path[home.len()..])
}

/// Replace `$NAME` when `NAME` is not followed by more identifier characters.
pub(super) fn replace_variable(word: &str, name: &str, value: &str) -> String {
    let marker = format!("${name}");
    let mut out = String::new();
    let mut rest = word;
    while let Some(at) = rest.find(&marker) {
        let after = &rest[at + marker.len()..];
        out.push_str(&rest[..at]);
        if after
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            out.push_str(&marker);
        } else {
            out.push_str(value);
        }
        rest = after;
    }
    out.push_str(rest);
    out
}

/// Resolve `.`, `..` and repeated slashes without touching the filesystem.
pub(super) fn normalize_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            segment => parts.push(segment),
        }
    }
    format!("/{}", parts.join("/"))
}

fn matches_any(segment: &str, names: &[&str]) -> bool {
    names.iter().any(|name| glob_matches(segment, name))
}

/// Whether the shell glob `pattern` (`*`, `?`, `[...]`) could match `name`.
/// Without glob characters this is equality.
fn glob_matches(pattern: &str, name: &str) -> bool {
    // A leading wildcard never expands to a dot-name in the shell (no
    // `dotglob`), and treating `*` or `*.json` as protected would block
    // ordinary work: only an explicit prefix can name a protected file.
    if pattern.starts_with(['*', '?', '[']) {
        return pattern == name;
    }
    glob_any(pattern, name)
}

/// The glob match itself, leading wildcards included. Iterative with one
/// backtrack point, so it is linear-times-linear whatever the pattern.
pub(super) fn glob_any(pattern: &str, name: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let name: Vec<char> = name.chars().collect();
    // Length of the single-character token at `at` (`?`, `[...]` or literal).
    let token = |at: usize| match pattern[at] {
        '[' => pattern[at..]
            .iter()
            .position(|ch| *ch == ']')
            .map_or(1, |close| close + 1),
        _ => 1,
    };
    let matches_one = |at: usize, ch: char| match pattern[at] {
        '?' => true,
        '[' => token(at) > 1 || ch == '[',
        literal => literal == ch,
    };
    let (mut p, mut n) = (0, 0);
    let mut star: Option<(usize, usize)> = None;
    while n < name.len() {
        if p < pattern.len() && pattern[p] == '*' {
            star = Some((p, n));
            p += 1;
        } else if p < pattern.len() && matches_one(p, name[n]) {
            p += token(p);
            n += 1;
        } else if let Some((star_p, star_n)) = star {
            p = star_p + 1;
            n = star_n + 1;
            star = Some((star_p, star_n + 1));
        } else {
            return false;
        }
    }
    while p < pattern.len() && pattern[p] == '*' {
        p += 1;
    }
    p == pattern.len()
}
