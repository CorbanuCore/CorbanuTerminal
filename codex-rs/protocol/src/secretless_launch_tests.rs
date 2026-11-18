use super::*;
use pretty_assertions::assert_eq;

#[test]
fn pf_27_s02_secret_and_helper_names_are_never_inherited() {
    for name in [
        "OPENAI_API_KEY",
        "GITHUB_TOKEN",
        "gh_token",
        "AWS_PROFILE",
        "AWS_SECRET_ACCESS_KEY",
        "DATABASE_PASSWORD",
        "PGPASSFILE",
        "SSH_AUTH_SOCK",
        "GIT_ASKPASS",
        "GIT_CONFIG_PARAMETERS",
        "BASH_ENV",
        "ENV",
        "ZDOTDIR",
        "PROMPT_COMMAND",
        "BASH_FUNC_x%%",
        "LD_PRELOAD",
        "DYLD_INSERT_LIBRARIES",
        "LD_LIBRARY_PATH",
        "NODE_OPTIONS",
        "CORBANU_VAULT_LABEL",
        "CODEX_API_KEY",
        "HOMEBREW_GITHUB_API_TOKEN",
        "DBUS_SESSION_BUS_ADDRESS",
        "NPM_CONFIG__AUTH",
        "MY_PAT",
        "UNLISTED_TOOL_SETTING",
    ] {
        assert!(!is_launch_env_name_allowed(name), "{name} must be stripped");
    }
}

#[test]
fn pf_27_s02_ordinary_tool_variables_survive() {
    for name in [
        "PATH",
        "HOME",
        "LANG",
        "LC_ALL",
        "TERM",
        "TMPDIR",
        "XDG_CONFIG_HOME",
        "CODEX_HOME",
        "CODEX_THREAD_ID",
        "CARGO_HOME",
        "JAVA_HOME",
        "PYTHONPATH",
        "ANDROID_SDK_ROOT",
        "VIRTUAL_ENV",
        "HTTPS_PROXY",
        "https_proxy",
        "GH_HOST",
        "SSL_CERT_FILE",
    ] {
        assert!(is_launch_env_name_allowed(name), "{name} must survive");
    }
}

#[test]
fn pf_27_s02_url_passwords_are_detected() {
    assert!(value_has_url_password("http://user:hunter2@proxy:8080"));
    assert!(value_has_url_password(
        "postgres://app:s3cret@db.internal/app?sslmode=require"
    ));
    assert!(!value_has_url_password("http://127.0.0.1:3128"));
    assert!(!value_has_url_password("https://user@example.com/path"));
    assert!(!value_has_url_password("https://example.com/a:b@c"));
    assert!(!value_has_url_password("plain value"));
}

#[test]
fn pf_27_s02_password_in_a_later_url_is_detected() {
    for value in [
        "a=http://x http://u:p@y",
        "http://a.example,http://u:p@b.example",
        "localhost;https://u:p@proxy:3128",
        "https://example.com/next?to=http://u:p@other.example",
    ] {
        assert!(value_has_url_password(value), "{value} must be detected");
    }
}

#[test]
fn pf_27_s02_schemeless_proxy_password_is_detected() {
    for value in [
        "user:pass@proxy:3128",
        "user:pass@proxy",
        "user:p%40ss@10.0.0.1:8080",
        "user:p@ss@proxy:3128",
        "alice@corp.example:pass@proxy:3128",
        "user:pass@[::1]:3128",
        "user:pass@proxy.example.com/",
        "--proxy=user:pass@proxy.example.com:3128",
        "localhost, user:pass@proxy:3128",
        r"DOMAIN\user:pass@proxy:8080",
        "user:p^ss{x}|y@proxy:3128",
        "\"user:pass@proxy:3128\"",
        "(user:pass@proxy.example.com.:3128)",
    ] {
        assert!(value_has_url_password(value), "{value} must be detected");
    }
}

fn assert_kept(class: &str, values: &[&str]) {
    for value in values {
        assert!(
            !value_has_url_password(value),
            "{class}: {value} has no password and must be kept"
        );
    }
}

#[test]
fn pf_27_s02_user_without_password_is_kept() {
    assert_kept(
        "user only",
        &[
            "user@host",
            "user@proxy:3128",
            "http://user@proxy:3128/ user@other:8080",
            // Empty password: kept here, while `http://user:@proxy` stays
            // flagged by the original `://` rule (stricter-only).
            "user:@proxy:3128",
        ],
    );
}

#[test]
fn pf_27_s02_host_and_port_lists_are_kept() {
    assert_kept(
        "host:port",
        &[
            "proxy:3128",
            "proxy.example.com:3128",
            "10.0.0.1:8080",
            "localhost,127.0.0.1,.example.com",
            "http://a.example:80,http://b.example:8080",
        ],
    );
}

#[test]
fn pf_27_s02_ipv6_literals_are_kept() {
    assert_kept(
        "IPv6",
        &[
            "[::1]:3128",
            "::1",
            "user@[::1]:3128",
            "http://[::1]:3128",
            "[fe80::1%25en0]:80",
        ],
    );
}

#[test]
fn pf_27_s02_email_addresses_are_kept() {
    assert_kept(
        "email",
        &[
            "alice@example.com",
            "alice@example.com, bob@example.org",
            "Alice <alice@example.com>",
        ],
    );
}

#[test]
fn pf_27_s02_scp_style_git_remotes_are_kept() {
    assert_kept(
        "scp-style remote",
        &[
            "git@github.com:org/repo.git",
            "git@gitlab.example.com:group/sub/repo",
        ],
    );
}

#[test]
fn pf_27_s02_opaque_scheme_uris_are_kept() {
    assert_kept(
        "scheme:user@host URI",
        &[
            "mailto:alice@example.com",
            "MAILTO:alice@example.com",
            "sip:alice@example.com",
            "xmpp:alice@example.com",
            "npm:lodash@latest",
            "group:devs@example.com",
            "serviceAccount:sa@proj.iam.gserviceaccount.com",
        ],
    );
}

#[test]
fn pf_27_s02_image_digests_and_package_specs_are_kept() {
    assert_kept(
        "image digest / package spec",
        &[
            "ubuntu:22.04@sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "registry.example.com/team/app:1.2@sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "npm:lodash@4.17.21",
            "python@3.12",
        ],
    );
}

#[test]
fn pf_27_s02_paths_and_prompts_are_kept() {
    assert_kept(
        "path / prompt",
        &[
            "/opt/homebrew/opt/node@20/bin:/opt/homebrew/opt/python@3.12/bin:/usr/bin",
            r"C:\Users\a:b@c",
            r"\t:\u@\h \w$ ",
            "https://example.com/a:b@c",
        ],
    );
}

#[test]
fn pf_27_s02_retain_keeps_exempt_names_but_drops_url_passwords() {
    let mut env = HashMap::from([
        ("PATH".to_string(), "/usr/bin".to_string()),
        ("OPENAI_API_KEY".to_string(), "sk-raw".to_string()),
        ("MY_SETTING".to_string(), "configured".to_string()),
        (
            "HTTPS_PROXY".to_string(),
            "http://user:pw@corp-proxy:3128".to_string(),
        ),
        ("UNLISTED".to_string(), "x".to_string()),
    ]);
    retain_launch_env(&mut env, |name| name == "MY_SETTING");
    let mut names = env.keys().cloned().collect::<Vec<_>>();
    names.sort();
    assert_eq!(names, vec!["MY_SETTING".to_string(), "PATH".to_string()]);
}

/// Every form from #2603's fixtures and its four security reviews, with the
/// password that must be among the redaction candidates (#2401, #2604).
const ISSUE_2401_FORMS: &[(&str, &[&str])] = &[
    (
        "http://a:First-2401@one https://b:Second-2401@two/path",
        &["First-2401", "Second-2401"],
    ),
    (
        "https://public.example,user:Proxy-2401@proxy:3128;https://u:Last-2401@[::1]/",
        &["Proxy-2401", "Last-2401"],
    ),
    (
        "u:Proxy-2401@[2001:db8::1]:3128;http://v:Last-2401@[::1]/",
        &["Proxy-2401", "Last-2401"],
    ),
    (
        "user:Raw%40Pass-2401@[2001:db8::1]:3128 https://u:Next%3aPass-2401@host",
        &["Raw%40Pass-2401", "Next%3aPass-2401"],
    ),
    (
        "https://example.com/?next=http://u:Nested-2401@proxy",
        &["Nested-2401"],
    ),
    (
        "http://u:First,Part;2401@host;u:Second-2401@proxy",
        &["First,Part;2401", "Second-2401"],
    ),
    ("http://u:Pass@word,2401@host", &["Pass@word,2401"]),
    (
        "http://u:Pass@word,2401@host https://u:Pass@word;2401@other",
        &["Pass@word,2401", "Pass@word;2401"],
    ),
    (
        "http://u:Pass@word,2401@host;u:Second-2401@proxy",
        &["Pass@word,2401", "Second-2401"],
    ),
    (
        "http://pub.example,user:Pass,word-2401@proxy:3128",
        &["Pass,word-2401"],
    ),
    (
        "imap://user;AUTH=PLAIN:Pass;word-2401@mail.example.com",
        &["Pass;word-2401"],
    ),
    (
        "http://pub.example,user:Pa\"ss-2401@proxy:3128",
        &["Pa\"ss-2401"],
    ),
    ("http://pub.example,user:Pass-2401@proxy)/x", &["Pass-2401"]),
    (
        "http://pub.example:8080,user:Pass-2401@proxy)/x",
        &["Pass-2401"],
    ),
    ("http://pub.example,user:Pass-2401@proxy'?q", &["Pass-2401"]),
    (
        "http://pub.example:8080,user:Pass-2401@proxy'?q",
        &["Pass-2401"],
    ),
    (
        "http://user@corp;AUTH=PLAIN:Pass-2401@proxy)/x",
        &["Pass-2401"],
    ),
    ("user:Secret-2401@proxy)", &["Secret-2401"]),
    ("user:Secret-2401@proxy)/x", &["Secret-2401"]),
    (
        "http://u:Secret-2401@proxy:3128,mail@example.com",
        &["Secret-2401"],
    ),
    ("user:p@ss-2401@proxy", &["p@ss-2401"]),
    ("http://u:pa/ss-2401@host", &["pa/ss-2401"]),
    ("http://u:pa?ss#2401@host", &["pa?ss#2401"]),
    ("http://u://Slash-2401@host", &["//Slash-2401"]),
    ("'user:Quoted-2401@proxy'", &["Quoted-2401"]),
    (
        "<user:Angled-2401@proxy.example.com.:3128>",
        &["Angled-2401"],
    ),
];

#[test]
fn issue_2401_every_reviewed_form_is_a_candidate_and_launch_filtered() {
    for (value, passwords) in ISSUE_2401_FORMS {
        let candidates = url_password_candidates(value, 8);
        for password in *passwords {
            assert!(
                candidates.contains(password),
                "{value}: {password} missing from {candidates:?}"
            );
        }
        assert!(value_has_url_password(value), "{value}");
        let mut env = HashMap::from([("HTTP_PROXY".to_string(), (*value).to_string())]);
        retain_launch_env(&mut env, |_| false);
        assert!(env.is_empty(), "{value} must be dropped at launch");
    }
}

#[test]
fn issue_2401_empty_passwords_stay_flagged_by_the_original_rule() {
    // Stricter-only: the original `://` rule still drops these.
    for value in ["http://user:@proxy:3128", "http://user:   @host"] {
        assert!(value_has_url_password(value), "{value}");
        assert_eq!(url_password_candidates(value, 1), Vec::<&str>::new());
    }
}

#[test]
fn issue_2401_ordinary_prose_registers_nothing_long() {
    // Not URL-password shaped: no candidates at all.
    for value in [
        "status: ready contact: alice@example.com git@host:repo",
        "alice@example.com, bob@example.org",
        "Alice <alice@example.com> at 10:30",
        "mailto:alice@example.com npm:package@latest",
        "https://host/path:word@other",
        "https://github.invalid/path?next=a:b@c",
        "https://host/#frag=a:b@c",
        "/opt/homebrew/opt/node@20/bin:/usr/bin",
    ] {
        assert!(!value_has_url_password(value), "{value}");
        assert_eq!(
            url_password_candidates(value, 1),
            Vec::<&str>::new(),
            "{value}"
        );
    }
    // Shaped like `user:password@host`: dropped at launch, but every
    // candidate is below the eight-byte redaction minimum.
    for (value, short) in [
        ("10:30@home", vec!["30"]),
        ("a:b@c", vec!["b"]),
        ("meet at 10:30@home", vec!["30"]),
    ] {
        assert!(value_has_url_password(value), "{value}");
        assert_eq!(url_password_candidates(value, 1), short, "{value}");
        assert_eq!(
            url_password_candidates(value, 8),
            Vec::<&str>::new(),
            "{value}"
        );
    }
    // Once a value is flagged, every reading that ends at an `@` in the same
    // word is a candidate; prose around a credential stays short.
    assert_eq!(
        url_password_candidates("meet at 10:30@home, then a:b@c", 1),
        vec!["30", "b"],
    );
    assert_eq!(
        url_password_candidates(
            "note: call at 10:30@office, ann:Secret-2401@corp.example",
            8
        ),
        vec!["Secret-2401"],
    );
    // Flagged by the original `://` rule (its authority runs to the last
    // `@`), but no `:` there follows a user name.
    let value = "http://user@host user@[::1]:3128 mailto:alice@example.com";
    assert!(value_has_url_password(value));
    assert_eq!(url_password_candidates(value, 1), Vec::<&str>::new());
    // Drive paths are not `user:password@host`.
    assert!(!value_has_url_password("C:/Users/a@b.com/x"));
}

#[test]
fn issue_2401_credential_lists_and_compact_json_stay_small() {
    // Forty credential URLs without whitespace: each password is a
    // candidate, ahead of the longer readings.
    let urls: Vec<String> = (0..40)
        .map(|index| format!("http://u{index}:Pw-{index}-Synth@h{index}.example"))
        .collect();
    let list = urls.join(",");
    let candidates = url_password_candidates(&list, 8);
    assert_eq!(candidates.len(), MAX_URL_PASSWORD_CANDIDATES);
    for (index, candidate) in candidates.iter().take(40).enumerate() {
        assert_eq!(*candidate, format!("Pw-{index}-Synth"));
    }
    // Compact JSON: keys and values hold `:` and quotes, but only the URL's
    // user name is followed by a password.
    let fields: Vec<String> = (0..80)
        .map(|index| format!("\"k{index:02}\":\"v{index:02}\""))
        .collect();
    let json = format!(
        "{{{},\"db\":\"postgres://svc:Synth-Pw-2401@db.internal:5432/app\",\"admin\":\"ops@example.com\"}}",
        fields.join(",")
    );
    assert_eq!(
        url_password_candidates(&json, 8),
        vec![
            "Synth-Pw-2401",
            "Synth-Pw-2401@db.internal:5432/app\",\"admin\":\"ops",
        ],
    );
}

#[test]
fn issue_2401_candidate_work_is_bounded() {
    let value = ",user:Secret-2401@proxy".repeat(20_000);
    assert!(value_has_url_password(&value));
    let candidates = url_password_candidates(&value, 8);
    assert!(candidates.contains(&"Secret-2401"));
    assert!(candidates.len() <= MAX_URL_PASSWORD_CANDIDATES);
    let colons = "u:x@".repeat(50_000);
    assert!(url_password_candidates(&colons, 8).len() <= MAX_URL_PASSWORD_CANDIDATES);
}

#[test]
fn issue_2401_first_url_password_reads_as_before() {
    assert_eq!(
        first_url_password("http://u: Spaced-2401 @proxy"),
        Some("Spaced-2401")
    );
    assert_eq!(
        first_url_password("http://u:Pass@word,2401@host"),
        Some("Pass@word,2401")
    );
    assert_eq!(first_url_password("http://u:@host"), None);
    assert_eq!(first_url_password("https://user@host"), None);
    assert_eq!(first_url_password("user:Secret-2401@proxy"), None);
}
