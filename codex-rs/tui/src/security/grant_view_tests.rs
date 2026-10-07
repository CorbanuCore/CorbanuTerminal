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
        command: vec![
            "/bin/bash".to_string(),
            "-lc".to_string(),
            "cat ../home/team-notes.txt".to_string(),
        ],
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

/// Only the grant review calls Core's `confirm`: no other product source
/// names it, imports it, or imports its module under another name (the
/// function is public, so this pins the convention). Skipped where the
/// workspace sources are not available (Bazel runfiles).
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
    if !workspace.join("core/src/security/grant_offer.rs").exists() {
        eprintln!("workspace sources not available; skipped");
        return;
    }
    let mut files = Vec::new();
    walk(workspace, &mut files);
    // Allowed: the review itself, Core's module, and re-exports of it.
    let allowed = [
        "tui/src/security/grant_view.rs",
        "core/src/security/grant_offer.rs",
        "core/src/security/mod.rs",
        "core/src/lib.rs",
        "app-server-client/src/lib.rs",
    ];
    let callers: Vec<String> = files
        .iter()
        .filter(|path| {
            let text = std::fs::read_to_string(path)
                .unwrap_or_default()
                .split_whitespace()
                .collect::<String>();
            text.contains("security_grant::confirm")
                || text.contains("grant_offer::confirm")
                || ["security_grant", "grant_offer"].iter().any(|module| {
                    text.contains(&format!("{module}::*"))
                        || text.contains(&format!("{module}as"))
                        || text.contains(&format!("{module};"))
                        || text.split(&format!("{module}::{{")).skip(1).any(|rest| {
                            rest.split('}').next().is_some_and(|names| {
                                names
                                    .split(',')
                                    .any(|name| name == "confirm" || name.starts_with("confirmas"))
                            })
                        })
                })
        })
        .filter_map(|path| path.strip_prefix(workspace).ok())
        .map(|path| path.display().to_string())
        // Tests drive Core's confirm directly; they are not product callers.
        .filter(|path| !path.ends_with("_tests.rs") && !path.contains("/tests/"))
        .collect();
    // The detector still finds the one real caller.
    assert!(
        callers.contains(&"tui/src/security/grant_view.rs".to_string()),
        "{callers:?}"
    );
    let callers: Vec<&String> = callers
        .iter()
        .filter(|path| !allowed.contains(&path.as_str()))
        .collect();
    assert_eq!(callers, Vec::<&String>::new());
}
