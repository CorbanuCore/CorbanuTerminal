//! #816 (AGG-04, second targeted run): after Aggressive is confirmed and
//! before the restart, the inspector's rows say approved commands already
//! stay inside the session's sandbox with network off (#799), and a host
//! check agreed, while screen 07, the review and the level list said both
//! start at the restart. Those screens now say what holds from confirmation,
//! from the same facts as the inspector's rows ([`BeforeRestart`]), and list
//! for the restart only what does not hold yet.

use pretty_assertions::assert_eq;

use super::issue_443_tests::key;
use super::issue_443_tests::picker;
use super::issue_443_tests::render;
use super::issue_443_tests::saved_live;
use super::issue_443_tests::text;
use super::*;

/// The session of the AGG-04 run and the host check: a workspace-write
/// sandbox with /tmp, network off, the untrusted-content labels on, web
/// search cached, no secretless launch and no broker.
fn agg_04_session() -> BeforeRestart {
    BeforeRestart {
        sandbox_kept: Some("commands can write to the current folder, /private/tmp".to_string()),
        widenable: false,
        network_off: true,
        web_search_off: false,
        environment: false,
        broker: false,
    }
}

fn line_with<'a>(screen: &'a str, needle: &str) -> &'a str {
    screen
        .lines()
        .find(|line| line.contains(needle))
        .unwrap_or_else(|| panic!("no line with {needle:?}: {screen}"))
}

#[test]
fn issue_816_screen_07_says_the_sandbox_and_network_hold_now() {
    let root = tempfile::tempdir().unwrap();
    let mut picker = picker(&root, /*live*/ true);
    picker.set_before_restart(agg_04_session());
    saved_live(&mut picker);
    let saved = text(&picker);
    assert_eq!(
        line_with(&saved, "Also in force now"),
        "Also in force now in this session: approved and allow-listed commands stay inside this session's sandbox (commands can write to the current folder, /private/tmp) and network is off for agent commands."
    );
    assert_eq!(
        line_with(&saved, "Starts when you restart"),
        "Starts when you restart: the Aggressive sandbox (writes only in the current folder), the project's credential files (.env, .env.* and .envrc) denied to agent commands, web search off, secret-like environment variables removed and login profiles and shell snapshots unused, model keys held by the credential broker, and the child-agent rules. Until then this session keeps your own settings for these, and commands already running keep the permissions they started with until they end."
    );
    insta::assert_snapshot!(
        "issue_816_saved_aggressive_before_restart",
        render(&picker, /*width*/ 80)
    );

    // Esc, then /security again: the level list says the same (this test's
    // session has no thread, so approvals are not held there).
    picker.screen = Screen::List { note: None };
    let list = text(&picker);
    assert_eq!(
        line_with(&list, "Saved: Aggressive."),
        "Saved: Aggressive. Core's protections (vault refusal, protected paths) apply in this session now; also in force now: approved and allow-listed commands stay inside this session's sandbox and network is off for agent commands; Aggressive's sandbox, approval, network (web search), environment, model-key and child-agent rows take effect when you restart Corbanu Terminal."
    );
    insta::assert_snapshot!(
        "issue_816_list_aggressive_before_restart",
        render(&picker, /*width*/ 80)
    );
}

#[test]
fn issue_816_review_says_what_holds_from_confirmation() {
    let root = tempfile::tempdir().unwrap();
    let mut picker = picker(&root, /*live*/ true);
    picker.set_before_restart(agg_04_session());
    picker.handle_key_event(key(KeyCode::Up));
    picker.handle_key_event(key(KeyCode::Enter));
    // Choosing reads the basis again; a running session's tree is live.
    picker.basis.live = true;
    let review = text(&picker);
    let paragraph = line_with(&review, "apply as soon as you confirm");
    assert_eq!(
        paragraph,
        "Core's protections below apply as soon as you confirm, and approval for every command from each pane's next turn. From then until the restart, also: approved and allow-listed commands stay inside this session's sandbox (commands can write to the current folder, /private/tmp) and network is off for agent commands. The rest of the rows above take effect when you restart Corbanu Terminal (you can restart right after saving); until then this session keeps its current settings for the rest, apart from Core's protected-path denials, and commands already running keep the permissions they started with. Your config.toml is not modified."
    );
    // The paragraph as the review renders it at 80 columns (the rest of the
    // review depends on the platform's preflight).
    let rendered = render(&picker, /*width*/ 80);
    let lines = rendered.lines().collect::<Vec<_>>();
    let first = lines
        .iter()
        .position(|line| line.starts_with("Core's protections below apply"))
        .expect("paragraph");
    let last = first
        + lines[first..]
            .iter()
            .position(|line| line.contains("modified."))
            .expect("paragraph end");
    insta::assert_snapshot!(
        "issue_816_review_at_confirmation",
        lines[first..=last].join("\n")
    );
}

/// A session that holds none of them (no sandbox, or one commands can
/// leave): nothing more is claimed, and the restart list is as before.
#[test]
fn issue_816_nothing_more_is_claimed_when_the_session_does_not_hold_it() {
    let root = tempfile::tempdir().unwrap();
    let mut picker = picker(&root, /*live*/ true);
    saved_live(&mut picker);
    let saved = text(&picker);
    assert!(!saved.contains("Also in force now"), "{saved}");
    assert_eq!(
        line_with(&saved, "Starts when you restart"),
        "Starts when you restart: the Aggressive sandbox (writes only in the current folder, approved commands stay inside it), the project's credential files (.env, .env.* and .envrc) denied to agent commands, network and web search off, secret-like environment variables removed and login profiles and shell snapshots unused, model keys held by the credential broker, and the child-agent rules. Until then this session keeps your own settings for these, and commands already running keep the permissions they started with until they end."
    );
    picker.screen = Screen::List { note: None };
    let list = text(&picker);
    assert!(!list.contains("also in force now"), "{list}");
    assert!(
        list.contains(
            "Aggressive's sandbox, approval, network, environment, model-key and child-agent rows take effect when you restart"
        ),
        "{list}"
    );
}

/// Everything the session already holds (its own settings or Core's rules)
/// is named as in force now and left out of the restart list.
#[test]
fn issue_816_everything_held_now_is_not_listed_for_the_restart() {
    let root = tempfile::tempdir().unwrap();
    let mut picker = picker(&root, /*live*/ true);
    picker.set_before_restart(BeforeRestart {
        sandbox_kept: Some("commands can write to the current folder".to_string()),
        widenable: false,
        network_off: true,
        web_search_off: true,
        environment: true,
        broker: true,
    });
    saved_live(&mut picker);
    let saved = text(&picker);
    assert_eq!(
        line_with(&saved, "Also in force now"),
        "Also in force now in this session: approved and allow-listed commands stay inside this session's sandbox (commands can write to the current folder), network is off for agent commands and web search is off, secret-like environment variables are removed and login profiles and shell snapshots are not used, and model keys are held by the credential broker."
    );
    assert!(
        saved.contains(
            "Starts when you restart: the Aggressive sandbox (writes only in the current folder), the project's credential files (.env, .env.* and .envrc) denied to agent commands, and the child-agent rules."
        ),
        "{saved}"
    );
    picker.screen = Screen::List { note: None };
    let list = text(&picker);
    assert!(
        list.contains(
            "; Aggressive's sandbox, approval and child-agent rows take effect when you restart"
        ),
        "{list}"
    );
}

/// Review r1: with permission-request tools on, a request you approve can
/// widen the sandbox (and turn network on), so the folders are not the
/// limit and network is not claimed off.
#[test]
fn issue_816_permission_requests_qualify_the_sandbox() {
    let root = tempfile::tempdir().unwrap();
    let mut picker = picker(&root, /*live*/ true);
    picker.set_before_restart(BeforeRestart {
        sandbox_kept: Some("commands can write to the current folder".to_string()),
        widenable: true,
        network_off: false,
        web_search_off: false,
        environment: false,
        broker: false,
    });
    saved_live(&mut picker);
    let saved = text(&picker);
    assert_eq!(
        line_with(&saved, "Also in force now"),
        "Also in force now in this session: approved and allow-listed commands stay inside this session's sandbox (commands can write to the current folder; a permission request you approve can widen it)."
    );
    assert!(
        line_with(&saved, "Starts when you restart")
            .starts_with("Starts when you restart: the Aggressive sandbox (writes only in the current folder), the project's credential files (.env, .env.* and .envrc) denied to agent commands, network and web search off,"),
        "{saved}"
    );
}

/// #846 (MOD-29): the project's `.env` files are denied by Aggressive's
/// sandbox profile, which starts at the restart, not by Core's rules that
/// apply at confirmation (a host check read `.env` before the restart). The
/// saved screen says so: "now" names only the credential files in the home
/// folder, and the restart list names the project's.
#[test]
fn issue_846_saved_screen_says_project_env_files_wait_for_the_restart() {
    let root = tempfile::tempdir().unwrap();
    // MOD-29 started from Moderate.
    for (active, before_restart) in [
        (ChosenLevel::Moderate, Some(agg_04_session())),
        (ChosenLevel::Moderate, None),
        (ChosenLevel::Permissive, None),
    ] {
        let mut picker = picker(&root, /*live*/ true);
        picker.active = active;
        if let Some(before_restart) = before_restart {
            picker.set_before_restart(before_restart);
        }
        saved_live(&mut picker);
        let saved = text(&picker);
        let now = line_with(&saved, "Active now in this session");
        assert!(!now.contains("sign-in and credential files"), "{now}");
        assert!(
            now.contains(
                "sign-in files, credential files in your home folder such as .ssh, .aws and .netrc,"
            ),
            "{now}"
        );
        assert!(
            line_with(&saved, "Starts when you restart").contains(
                "the project's credential files (.env, .env.* and .envrc) denied to agent commands"
            ),
            "{saved}"
        );
    }
    // The review uses the same words for what applies at confirmation.
    let mut picker = picker(&root, /*live*/ true);
    picker.active = ChosenLevel::Moderate;
    picker.basis.live = true;
    picker.screen = Screen::Review(ChosenLevel::Aggressive);
    let review = text(&picker);
    assert!(
        review.contains("credential files in your home folder such as .ssh, .aws and .netrc"),
        "{review}"
    );
    assert!(!review.contains("sign-in and credential files"), "{review}");
}
