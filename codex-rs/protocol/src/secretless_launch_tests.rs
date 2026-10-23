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
