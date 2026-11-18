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
fn issue_2401_extracts_every_url_password() {
    let cases: &[(&str, &[&str])] = &[
        (
            "http://a:First-2401@one https://b:Second-2401@two/path",
            &["First-2401", "Second-2401"],
        ),
        (
            "https://public.example,user:Proxy-2401@proxy:3128;https://u:Last-2401@[::1]/",
            &["Last-2401", "Proxy-2401"],
        ),
        (
            "user:Raw%40Pass-2401@[2001:db8::1]:3128 https://u:Next%3aPass-2401@host",
            &["Next%3aPass-2401", "Raw%40Pass-2401"],
        ),
        (
            "https://example.com/?next=http://u:Nested-2401@proxy",
            &["Nested-2401"],
        ),
        (
            "http://u:First,Part;2401@host;u:Second-2401@proxy",
            &["First,Part;2401", "Second-2401"],
        ),
        ("user:p@ss@proxy", &["p@ss"]),
        ("http://u:p@host", &["p"]),
        ("http://user:@host user:@proxy http://user:   @host", &[]),
        ("https://user@host user@[::1]:3128", &[]),
        (
            "status: ready contact: alice@example.com git@host:repo",
            &[],
        ),
        ("mailto:alice@example.com npm:package@latest", &[]),
        ("https://host/path:word@other", &[]),
    ];
    for (value, expected) in cases {
        assert_eq!(
            url_passwords(value).collect::<Vec<_>>(),
            *expected,
            "{value}"
        );
        assert_eq!(
            value_has_url_password(value),
            !expected.is_empty(),
            "{value}"
        );
        let mut env = HashMap::from([("HTTP_PROXY".to_string(), (*value).to_string())]);
        retain_launch_env(&mut env, |_| false);
        assert_eq!(env.is_empty(), !expected.is_empty(), "{value}");
    }
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
            "user:@proxy:3128",
            "http://user:@proxy:3128",
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
