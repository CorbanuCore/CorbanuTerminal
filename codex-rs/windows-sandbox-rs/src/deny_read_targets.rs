use codex_utils_absolute_path::AbsolutePathBuf;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

/// A configured deny-read rule, resolved against the sandbox's cwd: an exact
/// path, or a glob pattern that is expanded to the paths it matches now.
///
/// #304/S1: the persistent deny-read sync removes an entry only once the rule
/// that created it is no longer configured, never because a glob scan did
/// not see a path (a sandboxed process can hide one, e.g. by holding its
/// folder open).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum DenyReadRule {
    Path(PathBuf),
    Glob(String),
}

impl DenyReadRule {
    /// Identifies the rule across launches: case and separators as Windows
    /// treats them.
    pub fn key(&self) -> String {
        match self {
            Self::Path(path) => format!("path:{}", path_key(path)),
            Self::Glob(pattern) => format!("glob:{}", path_key(Path::new(pattern))),
        }
    }
}

/// One rule and the paths it expands to for this launch (none for a glob
/// that matches nothing yet).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DenyReadRuleTargets {
    pub rule: DenyReadRule,
    pub paths: Vec<AbsolutePathBuf>,
}

/// A launch's deny-read rules with their expanded paths. A rule that matches
/// nothing is still listed: its earlier entries stay.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DenyReadTargets {
    rules: Vec<DenyReadRuleTargets>,
}

impl DenyReadTargets {
    /// Each path is its own exact rule.
    pub fn from_exact_paths(paths: impl IntoIterator<Item = AbsolutePathBuf>) -> Self {
        let mut targets = Self::default();
        for path in paths {
            targets.add(DenyReadRule::Path(path.to_path_buf()), vec![path]);
        }
        targets
    }

    /// Adds `rule` with `paths`, merged into an equal rule already listed.
    pub fn add(&mut self, rule: DenyReadRule, paths: Vec<AbsolutePathBuf>) {
        let key = rule.key();
        let index = match self.rules.iter().position(|known| known.rule.key() == key) {
            Some(index) => index,
            None => {
                self.rules.push(DenyReadRuleTargets {
                    rule,
                    paths: Vec::new(),
                });
                self.rules.len() - 1
            }
        };
        let known = &mut self.rules[index].paths;
        for path in paths {
            if !known.contains(&path) {
                known.push(path);
            }
        }
    }

    pub fn extend(&mut self, other: Self) {
        for DenyReadRuleTargets { rule, paths } in other.rules {
            self.add(rule, paths);
        }
    }

    pub fn rules(&self) -> &[DenyReadRuleTargets] {
        &self.rules
    }

    /// No rule at all (a launch whose profile denies nothing).
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Every expanded path, deduplicated, in rule order.
    pub fn paths(&self) -> Vec<AbsolutePathBuf> {
        let mut seen = HashSet::new();
        self.rules
            .iter()
            .flat_map(|rule| rule.paths.iter())
            .filter(|path| seen.insert(path.to_path_buf()))
            .cloned()
            .collect()
    }

    pub fn has_paths(&self) -> bool {
        self.rules.iter().any(|rule| !rule.paths.is_empty())
    }

    pub fn contains_path(&self, path: &AbsolutePathBuf) -> bool {
        self.rules.iter().any(|rule| rule.paths.contains(path))
    }
}

/// A path as Windows compares it: separators unified, no trailing separator,
/// ASCII case folded.
pub(crate) fn path_key(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::DenyReadRule;
    use super::DenyReadTargets;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use pretty_assertions::assert_eq;

    fn abs(path: &str) -> AbsolutePathBuf {
        AbsolutePathBuf::from_absolute_path(std::env::temp_dir().join(path)).expect("absolute")
    }

    #[test]
    fn rules_merge_by_key_and_keep_empty_globs() {
        let mut targets = DenyReadTargets::default();
        targets.add(
            DenyReadRule::Glob("C:\\ws\\**\\*.env".into()),
            vec![abs("a")],
        );
        targets.add(
            DenyReadRule::Glob("c:/WS/**/*.env".into()),
            vec![abs("a"), abs("b")],
        );
        targets.add(DenyReadRule::Glob("C:\\ws\\*.key".into()), Vec::new());
        assert_eq!(targets.rules().len(), 2);
        assert_eq!(targets.paths(), vec![abs("a"), abs("b")]);
        assert!(!targets.is_empty());

        let json = serde_json::to_string(&targets).expect("serialize");
        let parsed: DenyReadTargets = serde_json::from_str(&json).expect("parse");
        assert_eq!(parsed, targets);
    }
}
