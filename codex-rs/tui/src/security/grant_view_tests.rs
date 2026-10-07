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
