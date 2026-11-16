//! #2324: `ExecEnvPolicy::drop_url_password_values` (PF-27-S02's value rule).
use std::collections::HashMap;

use codex_protocol::config_types::ShellEnvironmentPolicyInherit;
use pretty_assertions::assert_eq;

use super::EnvironmentInfo;
use super::ExecEnvPolicy;

/// Synthetic values only. `true` = embeds a URL password and must be dropped.
const URL_CASES: &[(&str, &str, bool)] = &[
    (
        "HTTP_PROXY",
        "http://fixture-user:synthetic-password@proxy.invalid:3128",
        true,
    ),
    (
        "HTTPS_PROXY",
        "http://fixture%40user:synthetic%3Apass%2Fword%40x@proxy.invalid:3128",
        true,
    ),
    (
        "ALL_PROXY",
        "socks5://fixture-user:synthetic-password@[2001:db8::1]:1080",
        true,
    ),
    ("FTP_PROXY", "http://[2001:db8::1]:3128", false),
    (
        "GITHUB_API_URL",
        "https://fixture%3Auser@api.invalid",
        false,
    ),
    ("GH_HOST", "https://fixture-user@[::1]:8443/api", false),
    (
        "GITHUB_SERVER_URL",
        "https://github.invalid/path?next=a:b@c",
        false,
    ),
];

fn policy(drop_url_password_values: bool) -> ExecEnvPolicy {
    ExecEnvPolicy {
        inherit: ShellEnvironmentPolicyInherit::All,
        ignore_default_excludes: true,
        exclude: Vec::new(),
        r#set: HashMap::from([(
            "SET_BY_POLICY".to_string(),
            "http://fixture-user:synthetic-password@set.invalid".to_string(),
        )]),
        include_only: Vec::new(),
        drop_url_password_values,
    }
}

fn inherited() -> HashMap<String, String> {
    URL_CASES
        .iter()
        .map(|(name, value, _)| (name.to_string(), value.to_string()))
        .chain([(
            "SET_BY_POLICY".to_string(),
            "http://fixture-user:synthetic-password@set.invalid".to_string(),
        )])
        .collect()
}

#[test]
fn drops_only_inherited_url_passwords() {
    let mut env = inherited();
    policy(/*drop_url_password_values*/ true).retain_inherited_values(&mut env);
    // As `retain_launch_env`, names the policy sets are filtered too.
    let expected: HashMap<String, String> = URL_CASES
        .iter()
        .filter(|(_, _, has_password)| !has_password)
        .map(|(name, value, _)| (name.to_string(), value.to_string()))
        .collect();
    assert_eq!(env, expected);
}

#[test]
fn unset_flag_keeps_every_value() {
    let mut env = inherited();
    policy(/*drop_url_password_values*/ false).retain_inherited_values(&mut env);
    assert_eq!(env, inherited());
}

#[test]
fn flag_is_compatible_with_older_peers() {
    // Omitted when unset, so older executors see an unchanged request ...
    let unset =
        serde_json::to_value(policy(/*drop_url_password_values*/ false)).expect("serialize");
    assert_eq!(unset.get("dropUrlPasswordValues"), None);
    let set = serde_json::to_value(policy(/*drop_url_password_values*/ true)).expect("serialize");
    assert_eq!(set["dropUrlPasswordValues"], true);
    // ... and a request from an older client means "unset".
    let mut old = unset;
    old.as_object_mut().unwrap().remove("dropUrlPasswordValues");
    let parsed: ExecEnvPolicy = serde_json::from_value(old).expect("deserialize");
    assert!(!parsed.drop_url_password_values);
    // Current executors advertise the capability; older ones' info omits it.
    assert!(
        EnvironmentInfo::local()
            .capabilities
            .env_url_password_filter
    );
    let mut info = serde_json::to_value(EnvironmentInfo::local()).expect("serialize");
    info["capabilities"]
        .as_object_mut()
        .unwrap()
        .remove("envUrlPasswordFilter");
    let info: EnvironmentInfo = serde_json::from_value(info).expect("deserialize");
    assert!(!info.capabilities.env_url_password_filter);
}
