use super::*;
use codex_protocol::ThreadId;
use pretty_assertions::assert_eq;

#[test]
fn pf_25_s01_held_grants_show_scope_uses_and_expiry() {
    let expires = Local
        .with_ymd_and_hms(2026, 10, 7, 14, 32, 0)
        .single()
        .expect("local time")
        .timestamp();
    let grant = |uses_left| HeldGrant {
        thread: ThreadId::new(),
        grant_id: "id".to_string(),
        label: "cat ../home/team-notes.txt".to_string(),
        uses_left,
        expires_at_unix_seconds: expires,
    };
    assert_eq!(
        held_lines(&[grant(Some(1)), grant(Some(2)), grant(None)]),
        vec![
            "Grant: `cat ../home/team-notes.txt` without the protected-path rules · 1 run left · expires 14:32",
            "Grant: `cat ../home/team-notes.txt` without the protected-path rules · 2 runs left · expires 14:32",
            "Grant: `cat ../home/team-notes.txt` without the protected-path rules · any run · expires 14:32",
        ]
    );
    assert_eq!(held_lines(&[]), Vec::<String>::new());
}

#[test]
fn pf_25_s01_errors_read_as_sentences() {
    assert_eq!(capitalized("not granted: x"), "Not granted: x");
    assert_eq!(capitalized(""), "");
}

/// Only the grant review calls Core's `confirm`: no other source in the
/// workspace names it (finding of the PF-25-S01 review: the function is
/// public, so this pins the convention).
#[test]
fn pf_25_s01_only_the_grant_review_confirms() {
    fn walk(dir: &std::path::Path, found: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            if path.is_dir() {
                if name != "target" && name != "node_modules" && name != ".git" {
                    walk(&path, found);
                }
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                found.push(path);
            }
        }
    }
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("codex-rs");
    let mut files = Vec::new();
    walk(workspace, &mut files);
    let callers: Vec<String> = files
        .iter()
        .filter(|path| {
            let text = std::fs::read_to_string(path).unwrap_or_default();
            let braced = ["security_grant::{", "grant_offer::{"]
                .iter()
                .any(|module| {
                    text.split(module).skip(1).any(|rest| {
                        rest.split('}').next().is_some_and(|names| {
                            names.split([',', ' ', '\n']).any(|name| name == "confirm")
                        })
                    })
                });
            braced
                || text.contains("security_grant::confirm")
                || text.contains("grant_offer::confirm")
        })
        .filter_map(|path| path.strip_prefix(workspace).ok())
        .map(|path| path.display().to_string())
        // Tests drive Core's confirm directly; they are not product callers.
        .filter(|path| !path.ends_with("_tests.rs") && !path.contains("/tests/"))
        .collect();
    assert_eq!(callers, vec!["tui/src/security/grant_view.rs".to_string()]);
}
