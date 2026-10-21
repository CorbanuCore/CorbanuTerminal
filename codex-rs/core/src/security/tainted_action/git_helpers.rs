//! #1159: git by another name. A dashed helper (`git-config`, the
//! `/usr/lib/git-core/git-log` in git's exec path, `git-lfs`) runs as `git
//! config`, `git log`, `git lfs`; `git.exe` (and `git-config.exe`) is git on
//! Windows. The git checks judge them as the git line they stand for.

use super::shell;

/// What `word` names, read as git: `Some(None)` for git itself under another
/// name (`git.exe`, `C:\Git\cmd\git`), `Some(Some(subcommand))` for a dashed
/// helper, its subcommand in lower case. `git` and `/usr/bin/git` are already
/// seen as git: `None`, like any other word.
pub(super) fn git_name(word: &str) -> Option<Option<String>> {
    let base = word
        .trim_end_matches(['/', '\\'])
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(word)
        .to_lowercase();
    let base = base.strip_suffix(".exe").unwrap_or(&base);
    if base == "git" {
        return (!shell::basename(word).eq_ignore_ascii_case("git")).then_some(None);
    }
    base.strip_prefix("git-")
        .filter(|subcommand| !subcommand.is_empty())
        .map(|subcommand| Some(subcommand.to_string()))
}

#[cfg(test)]
mod tests {
    use super::git_name;
    use pretty_assertions::assert_eq;

    #[test]
    fn git_names() {
        let helper = |name: &str| Some(Some(name.to_string()));
        for (word, want) in [
            ("git-config", helper("config")),
            ("/usr/lib/git-core/git-config", helper("config")),
            ("/opt/x/GIT-Log", helper("log")),
            ("git-lfs", helper("lfs")),
            ("git-config.exe", helper("config")),
            (
                "C:\\Git\\mingw64\\libexec\\git-core\\git-log.exe",
                helper("log"),
            ),
            ("git.exe", Some(None)),
            ("C:/Program Files/Git/cmd/git.exe", Some(None)),
            ("C:\\Git\\cmd\\GIT.EXE", Some(None)),
            ("C:\\Git\\cmd\\git", Some(None)),
            ("git", None),
            ("/usr/bin/git", None),
            ("GIT", None),
            ("git-", None),
            ("legit-x", None),
            (".git-blame-ignore-revs", None),
            ("digit.exe", None),
        ] {
            assert_eq!(git_name(word), want, "{word}");
        }
    }
}
