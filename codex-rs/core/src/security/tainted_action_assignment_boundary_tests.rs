//! #2192: an assignment the shell never makes. A word that starts with a
//! character the shell does not split on (`,`, U+00A0) is a command word,
//! not an assignment, so `,r=o; git fetch e"$r"` leaves `r` unset and the
//! remote unseen. In its own file: `tainted_action_tests.rs` is close to the
//! repository's blob size limit.

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

/// The four verdicts on `<shell> -lc <line>` in `/work`: bash and zsh, each
/// as the host runs it and as its argv names it.
fn verdicts(line: &str) -> [Option<ProtectedActionKind>; 4] {
    let judge = |shell: &str, as_named: bool| {
        let action = ApprovalAction::Shell {
            id: "call".into(),
            environment_id: "local".into(),
            command: [shell, "-lc", line].map(str::to_string).to_vec(),
            cwd: PathUri::from_abs_path(
                &AbsolutePathBuf::from_absolute_path_checked(fixture("/work")).expect("absolute"),
            ),
            sandbox_permissions: SandboxPermissions::UseDefault,
            additional_permissions: None,
            justification: None,
        };
        let (home, user_home) = (fixture(HOME), fixture(USER_HOME));
        if as_named {
            classify_as_named_with(&action, &home, Some(&user_home))
        } else {
            classify_with(&action, &home, Some(&user_home))
        }
    };
    [
        judge("bash", false),
        judge("bash", true),
        judge("zsh", false),
        judge("zsh", true),
    ]
}

/// Lines with any of the four verdicts other than `expected`.
fn wrong(lines: &[&str], expected: Option<ProtectedActionKind>) -> Vec<String> {
    lines
        .iter()
        .filter_map(|line| {
            let got = verdicts(line);
            got.iter()
                .any(|verdict| *verdict != expected)
                .then(|| format!("{line:?} -> {got:?}"))
        })
        .collect()
}

/// The issue's two lines, the same first word after other characters the
/// shell does not split on (U+2003, a vertical tab), and `r=o` after a
/// comma inside a word. Each was quiet on main in all four readings. The controls keep their verdicts: a real assignment (after a
/// space, a tab or `;`) still names the remote `eo`, and an unset `r`
/// still asks.
#[test]
fn issue_2192_assignment_after_a_non_blank_character_asks() {
    let asks = [
        ",r=o; git fetch e\"$r\"",
        "\u{a0}r=o; git fetch e\"$r\"",
        "\u{2003}r=o; git fetch e\"$r\"",
        "\u{b}r=o; git fetch e\"$r\"",
        "x=1,r=o; git fetch e\"$r\"",
        "a=1 ,r=o; git fetch e\"$r\"",
        "true; ,r=o; git fetch e\"$r\"",
    ];
    assert_eq!(
        wrong(&asks, Some(ProtectedActionKind::UnseenCode)),
        Vec::<String>::new()
    );

    let quiet = [
        "r=o; git fetch e\"$r\"",
        " r=o; git fetch e\"$r\"",
        "\tr=o; git fetch e\"$r\"",
        "true;r=o; git fetch e\"$r\"",
        "r=o; echo a,b; git fetch e\"$r\"",
        "export X=0,1; git fetch origin",
        "r=o, ; git fetch e\"$r\"",
    ];
    assert_eq!(wrong(&quiet, None), Vec::<String>::new());
    assert_eq!(
        wrong(
            &["git fetch e\"$r\""],
            Some(ProtectedActionKind::UnseenCode)
        ),
        Vec::<String>::new()
    );
}
