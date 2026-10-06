//! Path resolution and protected-path classification for the post-taint
//! classifier. Lexical first; a bounded number of filesystem lookups then
//! follow symlinks and other non-canonical spellings of a home.

use super::ProtectedActionKind;
use std::cell::Cell;
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
    "id_rsa",
    "id_ecdsa",
    "id_ed25519",
    "id_dsa",
];
/// Corbanu home folder names (besides the configured `CODEX_HOME`).
const HOME_SEGMENTS: &[&str] = &[".corbanu", ".codex", ".pfterminal"];
/// Filesystem lookups one classification may make before it stays lexical.
const FS_LOOKUP_BUDGET: usize = 256;

pub(super) struct Homes {
    /// Original spellings, used to expand `$CODEX_HOME`, `$HOME` and `~`.
    codex_home_raw: String,
    user_home_raw: Option<String>,
    /// Lowercase spellings to match against: as configured and canonical.
    codex_homes: Vec<String>,
    user_homes: Vec<String>,
    lookups: Cell<usize>,
}

impl Homes {
    pub(super) fn new(codex_home: &Path, user_home: Option<&Path>) -> Self {
        let homes = Self {
            codex_home_raw: codex_home
                .to_string_lossy()
                .trim_end_matches('/')
                .to_string(),
            user_home_raw: user_home
                .map(|home| home.to_string_lossy().trim_end_matches('/').to_string()),
            codex_homes: Vec::new(),
            user_homes: Vec::new(),
            lookups: Cell::new(0),
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
        Self {
            codex_homes,
            user_homes,
            ..homes
        }
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

    /// Whether `path` is the user's home, or a folder holding it or the
    /// Corbanu home: copying or archiving it recursively takes credentials.
    pub(super) fn holds_a_home(&self, path: &str) -> bool {
        let lexical = normalize_path(&path.to_lowercase());
        let canonical = self.canonical(path);
        [Some(lexical), canonical]
            .into_iter()
            .flatten()
            .any(|path| {
                let holds = |home: &String| {
                    path == "/" || *home == path || home.starts_with(&format!("{path}/"))
                };
                self.user_homes.iter().any(holds) || self.codex_homes.iter().any(holds)
            })
    }

    /// Canonical lowercase spelling of `path`, following symlinks. A path
    /// that does not exist yet keeps its missing tail on top of its nearest
    /// existing folder. `None` once the lookup budget is spent.
    pub(super) fn canonical(&self, path: &str) -> Option<String> {
        let mut existing = PathBuf::from(normalize_path(path));
        let mut tail: Vec<String> = Vec::new();
        loop {
            if self.lookups.get() >= FS_LOOKUP_BUDGET {
                return None;
            }
            self.lookups.set(self.lookups.get() + 1);
            if let Ok(canonical) = std::fs::canonicalize(&existing) {
                let mut canonical = canonical.to_string_lossy().into_owned();
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
        let segments: Vec<&str> = path
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect();
        let name = segments.last().copied().unwrap_or_default();
        // Credential folders in the user's home, matched segment by segment
        // so a glob such as `~/.dock*` still counts.
        let under_user_home = |folder: &str| {
            self.user_homes.iter().any(|home| {
                path.strip_prefix(home.as_str())
                    .filter(|rest| rest.starts_with('/'))
                    .is_some_and(|rest| {
                        let mut actual = rest.split('/').filter(|segment| !segment.is_empty());
                        folder.split('/').all(|expected| {
                            actual.next().is_some_and(|got| {
                                glob_matches(got, expected)
                                    // `*` matches non-dot folder names.
                                    || (!expected.starts_with('.') && glob_any(got, expected))
                            })
                        })
                    })
            })
        };
        if segments
            .iter()
            .any(|segment| matches_any(segment, CREDENTIAL_SEGMENTS))
            || USER_CREDENTIAL_SEGMENTS
                .iter()
                .any(|folder| under_user_home(folder))
            || CREDENTIAL_FILES.iter().any(|file| glob_matches(name, file))
            || (glob_matches(name, "hosts.yml")
                && segments.iter().any(|segment| glob_any(segment, "gh")))
            || name.starts_with("id_rsa")
            || name.starts_with("id_ed25519")
        {
            return Some(ProtectedActionKind::Credentials);
        }
        let below_home = self.below_home(&path, &segments)?;
        let first = below_home
            .split('/')
            .find(|segment| !segment.is_empty())
            .unwrap_or("");
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

    /// The part of `path` below a Corbanu home, if it is inside one: the
    /// configured home as a whole-segment run anywhere in the path, or a
    /// default home folder name (or a glob that could match one) as a segment.
    fn below_home<'a>(&self, path: &'a str, segments: &[&str]) -> Option<&'a str> {
        for home in &self.codex_homes {
            let found = path.match_indices(home.as_str()).find_map(|(index, _)| {
                let rest = &path[index + home.len()..];
                (rest.is_empty() || rest.starts_with('/')).then_some(rest)
            });
            if found.is_some() {
                return found;
            }
        }
        let index = segments
            .iter()
            .position(|segment| matches_any(segment, HOME_SEGMENTS))?;
        let marker = format!("/{}", segments[index]);
        path.match_indices(&marker)
            .map(|(at, _)| &path[at + marker.len()..])
            .find(|rest| rest.is_empty() || rest.starts_with('/'))
    }
}

/// Replace `$NAME` (and `${NAME}`, whose braces the lexer already removed)
/// when `NAME` is not followed by more identifier characters.
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

/// The glob match itself, leading wildcards included.
fn glob_any(pattern: &str, name: &str) -> bool {
    fn matches(pattern: &[char], name: &[char]) -> bool {
        match pattern.first() {
            None => name.is_empty(),
            Some('*') => (0..=name.len()).any(|skip| matches(&pattern[1..], &name[skip..])),
            Some('?') => !name.is_empty() && matches(&pattern[1..], &name[1..]),
            Some('[') => match pattern.iter().position(|ch| *ch == ']') {
                Some(close) => !name.is_empty() && matches(&pattern[close + 1..], &name[1..]),
                None => name.first() == Some(&'[') && matches(&pattern[1..], &name[1..]),
            },
            Some(ch) => name.first() == Some(ch) && matches(&pattern[1..], &name[1..]),
        }
    }
    let pattern: Vec<char> = pattern.chars().collect();
    let name: Vec<char> = name.chars().collect();
    matches(&pattern, &name)
}
