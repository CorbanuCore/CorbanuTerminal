use pretty_assertions::assert_eq;

use super::*;
use crate::runtime::test_support::unique_temp_dir;

fn temp_dir() -> io::Result<PathBuf> {
    let dir = unique_temp_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn scrub(text: &str) -> Option<String> {
    scrub_bytes(text.as_bytes())
}

#[test]
fn scrub_redacts_each_leaked_form_and_is_idempotent() {
    let cases = [
        (
            r#"ModelProviderInfo { experimental_bearer_token: Some("fake-bearer-1"), env_key: None }"#,
            r#"ModelProviderInfo { experimental_bearer_token: Some("REDACTED"), env_key: None }"#,
        ),
        (
            r#"http_headers: Some({"X-Sentinel": "fake-h"}), query_params: Some({"sig": "fake-q", "v": "2"})"#,
            r#"http_headers: Some({"X-Sentinel": "REDACTED"}), query_params: Some({"sig": "REDACTED", "v": "REDACTED"})"#,
        ),
        (
            r#"headers={"x-request-id": "req-1", "set-cookie": "__cf_bm=fake-cookie; Path=/", "X-Api-Key": "fake-2"}"#,
            r#"headers={"x-request-id": "req-1", "set-cookie": "REDACTED", "X-Api-Key": "REDACTED"}"#,
        ),
        (
            r#"line="{\"authorization\": \"Bearer fake-3\"}""#,
            r#"line="{\"authorization\": \"REDACTED\"}""#,
        ),
        (
            r#"spawn_child_async: "sh" ["-c"] {"PATH": "/usr/bin", "MNEMONIC": "fake words"}"#,
            r#"spawn_child_async: "sh" ["-c"] {"PATH": "REDACTED", "MNEMONIC": "REDACTED"}"#,
        ),
        (
            "url=https://user:fake-pass@api.example.com/v1/models?key=fake-4&api-version=2025",
            "url=https://REDACTED@api.example.com/v1/models?key=REDACTED&api-version=REDACTED",
        ),
        (
            "authorization: Bearer fake-token-555 sent",
            "authorization: Bearer REDACTED sent",
        ),
        (
            "Authorization: Basic ZmFrZTpmYWtl",
            "Authorization: Basic REDACTED",
        ),
        (
            "export GITHUB_TOKEN=fake-6 done",
            "export GITHUB_TOKEN=REDACTED done",
        ),
        (
            r#"set DB_PASS="fake-7" and"#,
            r#"set DB_PASS="REDACTED" and"#,
        ),
        (
            r#"env API_KEY=\"fake-8\" end"#,
            r#"env API_KEY=\"REDACTED\" end"#,
        ),
        (
            r#"MNEMONIC="abandon ability able about" SEED='two words' TOKEN=\"a b\""#,
            r#"MNEMONIC="REDACTED" SEED='REDACTED' TOKEN=\"REDACTED\""#,
        ),
        (
            r#"{"refresh_tokens": "fake-rt", "input_tokens": "12"}"#,
            r#"{"refresh_tokens": "REDACTED", "input_tokens": "12"}"#,
        ),
        (
            r#"body="{\\\"token\\\": \\\"fake-9\\\"}""#,
            r#"body="{\\\"token\\\": \\\"REDACTED\\\"}""#,
        ),
        (
            "remote https://fake-pat-10@git.example.com/x.git",
            "remote https://REDACTED@git.example.com/x.git",
        ),
        (
            "key sk-proj-abcdefghijklmnopqrstuvwxyz0123 used",
            "key REDACTED used",
        ),
    ];
    for (leaked, expected) in cases {
        assert_eq!(scrub(leaked).as_deref(), Some(expected), "{leaked}");
        assert_eq!(scrub(expected), None, "idempotent: {expected}");
    }
    for clean in [
        "Request completed status=200 url=https://api.example.com/v1/responses",
        r#"experimental_bearer_token: Some("<redacted>"), "X-Sentinel": "<redacted>""#,
        r#"session_id: Some("0199-abc"), "auth_mode": "chatgpt", input_tokens=120"#,
        "basic functionality works",
        "┌─ résumé ─┐ ünïcode",
    ] {
        assert_eq!(scrub(clean), None, "{clean}");
    }
}

#[test]
fn log_file_is_masked_in_place_once() -> io::Result<()> {
    let dir = temp_dir()?;
    let path = dir.join("codex-tui.log");
    let leaked = "2026 DEBUG Request completed url=https://x.test/v1?key=fake-file-key headers={\"set-cookie\": \"fake-file-cookie\"}\n";
    let clean = "2026 INFO plain line\n";
    std::fs::write(&path, format!("{clean}{leaked}{clean}"))?;

    assert_eq!(scrub_log_file_once(&path)?, Some(2));
    let masked = std::fs::read_to_string(&path)?;
    assert_eq!(masked.len(), clean.len() * 2 + leaked.len());
    assert!(!masked.contains("fake-file-key") && !masked.contains("fake-file-cookie"));
    assert!(masked.contains("?key=*************") && masked.starts_with(clean));
    assert!(log_file_scrub_marker(&path).exists());

    // The marker makes later starts skip the file.
    std::fs::write(&path, leaked)?;
    assert_eq!(scrub_log_file_once(&path)?, None);
    assert_eq!(std::fs::read_to_string(&path)?, leaked);
    Ok(())
}

#[test]
fn missing_log_file_counts_as_scrubbed() -> io::Result<()> {
    let dir = temp_dir()?;
    let path = dir.join("codex-tui.log");
    assert_eq!(scrub_log_file_once(&path)?, Some(0));
    assert!(log_file_scrub_marker(&path).exists());
    Ok(())
}

/// #398: older builds logged sandboxed command lines verbatim.
#[test]
fn sandbox_command_logs_are_masked_once() -> io::Result<()> {
    let dir = temp_dir()?;
    let leaked = "[2026-10-01 10:00:00.000 codex.exe] START: curl.exe -H \"Authorization: Bearer fake-old-bearer-0001\" --password fake-old-pass-0002 -u admin:fake-old-user-0003 https://x/?api_key=fake-old-query-0004\n";
    let clean = "[2026-10-01 10:00:01.000 codex.exe] SUCCESS: cmd.exe /c echo ok\n";
    let log = dir.join("sandbox.2026-10-01.log");
    let other = dir.join("setup_marker.json");
    std::fs::write(&log, format!("{clean}{leaked}"))?;
    std::fs::write(&other, leaked)?;

    assert_eq!(scrub_sandbox_logs_once(&dir)?, Some(4));
    let masked = std::fs::read_to_string(&log)?;
    assert_eq!(masked.len(), clean.len() + leaked.len());
    assert!(masked.starts_with(clean), "{masked}");
    assert!(!masked.contains("fake-old"), "{masked}");
    assert!(
        masked.contains("START: curl.exe -H \"Authorization: ****"),
        "{masked}"
    );
    assert_eq!(std::fs::read_to_string(&other)?, leaked);

    // The marker makes later starts skip the directory.
    std::fs::write(&log, leaked)?;
    assert_eq!(scrub_sandbox_logs_once(&dir)?, None);
    assert_eq!(std::fs::read_to_string(&log)?, leaked);
    assert_eq!(scrub_sandbox_logs_once(&dir.join("missing"))?, Some(0));
    Ok(())
}
