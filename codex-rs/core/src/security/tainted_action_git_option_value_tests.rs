//! Regression coverage for issue #1975.

use super::*;
use crate::sandboxing::SandboxPermissions;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use std::path::PathBuf;

const HOME: &str = "/home/fixture/.corbanu";
const USER_HOME: &str = "/home/fixture";

/// A fixture path, on drive `C:` on Windows (#1240).
fn fixture(path: &str) -> PathBuf {
    if cfg!(windows) && !Path::new(path).is_absolute() {
        PathBuf::from(format!("C:{}", path.replace('/', "\\")))
    } else {
        PathBuf::from(path)
    }
}

/// The verdict on `bash -lc <line>` in `/work`.
fn script(line: &str) -> Option<ProtectedActionKind> {
    let action = ApprovalAction::Shell {
        id: "call".into(),
        environment_id: "local".into(),
        command: ["bash", "-lc", line].map(str::to_string).to_vec(),
        cwd: PathUri::from_abs_path(
            &AbsolutePathBuf::from_absolute_path_checked(fixture("/work")).expect("absolute"),
        ),
        sandbox_permissions: SandboxPermissions::UseDefault,
        additional_permissions: None,
        justification: None,
    };
    classify_with(&action, &fixture(HOME), Some(&fixture(USER_HOME)))
}

/// Lines whose verdict is not `expected`.
fn wrong(lines: &[&str], expected: Option<ProtectedActionKind>) -> Vec<String> {
    lines
        .iter()
        .filter(|line| script(line) != expected)
        .map(|line| format!("{line} -> {:?}", script(line)))
        .collect()
}

/// #1975: an ordinary brace value must not hide the remote after `--`.
#[test]
fn issue_1975_brace_option_value_before_dashes_asks() {
    let lines = [
        "git fetch -o {} -- e\"$r\"",
        "git fetch -o '{}' -- e\"$r\"",
        "git pull -o {} -- e\"$r\"",
        "git fetch -o {} -- \"$r\"",
    ];
    assert_eq!(
        wrong(&lines, Some(ProtectedActionKind::UnseenCode)),
        Vec::<String>::new()
    );
}

#[test]
fn issue_1975_option_value_controls() {
    assert_eq!(
        wrong(
            &["git fetch -o x -- e\"$r\"", "git fetch -o {} e\"$r\""],
            Some(ProtectedActionKind::UnseenCode)
        ),
        Vec::<String>::new()
    );
    assert_eq!(
        wrong(
            &[
                "git fetch -o {} -- origin",
                "r=origin; git fetch -o {} -- \"$r\""
            ],
            /*expected*/ None
        ),
        Vec::<String>::new()
    );
}
