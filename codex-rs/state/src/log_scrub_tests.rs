use pretty_assertions::assert_eq;

use super::*;
use crate::runtime::test_support::unique_temp_dir;

fn temp_dir() -> io::Result<PathBuf> {
    let dir = unique_temp_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[test]
fn scrub_text_redacts_each_leaked_form_and_is_idempotent() {
    let cases = [
        (
            r#"ModelProviderInfo { experimental_bearer_token: Some("fake-bearer-1"), env_key: None }"#,
            r#"ModelProviderInfo { experimental_bearer_token: Some("REDACTED"), env_key: None }"#,
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
            "url=https://user:fake-pass@api.example.com/v1/models?key=fake-4&api-version=2025",
            "url=https://REDACTED@api.example.com/v1/models?key=REDACTED&api-version=REDACTED",
        ),
        (
            "authorization: Bearer fake-token-555 sent",
            "authorization: Bearer REDACTED sent",
        ),
        (
            "export GITHUB_TOKEN=fake-6 done",
            "export GITHUB_TOKEN=REDACTED done",
        ),
        (
            "key sk-proj-abcdefghijklmnopqrstuvwxyz0123 used",
            "key REDACTED used",
        ),
    ];
    for (leaked, expected) in cases {
        let scrubbed = scrub_text(leaked);
        assert_eq!(scrubbed.as_deref(), Some(expected), "{leaked}");
        assert_eq!(scrub_text(expected), None, "idempotent: {expected}");
    }
    for clean in [
        "Request completed status=200 url=https://api.example.com/v1/responses",
        r#"experimental_bearer_token: Some("<redacted>"), "X-Sentinel": "<redacted>""#,
        "input_tokens=120 output_tokens=7",
    ] {
        assert_eq!(scrub_text(clean), None, "{clean}");
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
