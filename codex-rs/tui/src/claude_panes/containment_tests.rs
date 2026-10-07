use std::collections::BTreeMap;

use pretty_assertions::assert_eq;

use super::*;

#[test]
fn launch_env_keeps_only_the_allowlist_without_proxies_then_adds_the_pane() {
    let inherited = [
        ("PATH", "/usr/bin"),
        ("HOME", "/Users/someone"),
        ("LANG", "en_US.UTF-8"),
        ("ANTHROPIC_API_KEY", "sk-ant-not-real"),
        ("ZAI_API_KEY", "zai-not-real"),
        ("GITHUB_TOKEN", "ghp-not-real"),
        ("CLAUDE_CODE_OAUTH_TOKEN", "oauth-not-real"),
        ("ANTHROPIC_BASE_URL", "https://example.invalid"),
        ("HTTPS_PROXY", "http://127.0.0.1:9999"),
        ("http_proxy", "http://127.0.0.1:9998"),
        ("NO_PROXY", "localhost"),
        ("WS_PROXY", "http://127.0.0.1:9997"),
        ("DYLD_INSERT_LIBRARIES", "/tmp/x.dylib"),
        ("CLAUDE_CONFIG_DIR", "/Users/someone/.claude"),
    ]
    .map(|(name, value)| (name.to_string(), value.to_string()));
    let pane = BTreeMap::from([
        (
            "HTTP_PROXY".to_string(),
            "http://127.0.0.1:4242".to_string(),
        ),
        ("CLAUDE_CONFIG_DIR".to_string(), "/state/config".to_string()),
        (
            "ANTHROPIC_AUTH_TOKEN".to_string(),
            "bridge-token".to_string(),
        ),
    ]);

    let mut env = launch_env(inherited, &pane).into_iter().collect::<Vec<_>>();
    env.sort();
    let expected = [
        ("ANTHROPIC_AUTH_TOKEN", "bridge-token"),
        ("CLAUDE_CONFIG_DIR", "/state/config"),
        ("HOME", "/Users/someone"),
        ("HTTP_PROXY", "http://127.0.0.1:4242"),
        ("LANG", "en_US.UTF-8"),
        ("PATH", "/usr/bin"),
    ]
    .map(|(name, value)| (name.to_string(), value.to_string()));
    assert_eq!(env, expected.to_vec());
}

#[test]
fn state_dir_is_outside_codex_home_and_keyed_by_home_and_pane() {
    let codex_home = Path::new("/tmp/corbanu-home-a");
    let first = state_dir(codex_home, "claude-1", None).unwrap();
    assert!(first.is_absolute());
    assert!(!first.starts_with(codex_home));
    assert!(first.ends_with("claude-1"));
    assert_ne!(first, state_dir(codex_home, "claude-2", None).unwrap());
    assert_ne!(
        first,
        state_dir(Path::new("/tmp/corbanu-home-b"), "claude-1", None).unwrap()
    );
}

#[test]
fn base_profile_writes_only_the_pane_folder_and_its_state_with_network_off() {
    let cwd = tempfile::tempdir().unwrap();
    let state_root = tempfile::tempdir().unwrap();
    let state = state_root.path().join("pane-a");
    let containment = ClaudeContainment {
        state_dir: state.clone(),
        panes_dir: cwd.path().join("panes"),
        linux_sandbox_exe: None,
    };
    let profile = base_profile(&containment).unwrap();
    let (file_system, network) = profile.to_runtime_permissions();
    let cwd = cwd.path();
    assert!(!network.is_enabled());
    assert!(file_system.can_write_path_with_cwd(&cwd.join("file"), cwd));
    assert!(file_system.can_write_path_with_cwd(&state.join("config/x"), cwd));
    assert!(file_system.can_read_path_with_cwd(&state.join("config/x"), cwd));
    assert!(!file_system.can_read_path_with_cwd(&state_root.path().join("pane-b/config/x"), cwd));
    // Corbanu's settings for the turn are read-only; other panes' records
    // are unreadable.
    let settings = containment.settings_path();
    assert!(file_system.can_read_path_with_cwd(&settings, cwd));
    assert!(!file_system.can_write_path_with_cwd(&settings, cwd));
    assert!(
        !file_system
            .can_read_path_with_cwd(&containment.panes_dir.join("other/turn-0001.jsonl"), cwd)
    );
    assert!(file_system.can_read_path_with_cwd(Path::new("/usr/bin/env"), cwd));
    for outside in ["/tmp/corbanu-contained-probe", "/usr/local/corbanu-probe"] {
        assert!(
            !file_system.can_write_path_with_cwd(Path::new(outside), cwd),
            "{outside}"
        );
    }
    assert!(!file_system.can_write_path_with_cwd(&cwd.join(".git/config"), cwd));
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[test]
fn contained_launch_is_refused_without_the_secretless_contract() {
    if crate::legacy_core::external_agent_contract_armed() {
        return;
    }
    let cwd = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let containment = ClaudeContainment {
        state_dir: state.path().to_path_buf(),
        panes_dir: cwd.path().join("panes"),
        linux_sandbox_exe: None,
    };
    let err = contain(
        &containment,
        "claude",
        &["-p".to_string()],
        &BTreeMap::new(),
        cwd.path(),
        /*bridge_port*/ 4242,
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("secretless_agent_launch"),
        "{err:#}"
    );
}
