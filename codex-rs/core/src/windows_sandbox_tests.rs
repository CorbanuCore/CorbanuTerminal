use super::*;
use codex_features::Features;
use codex_features::FeaturesToml;
use codex_network_proxy::NetworkProxyConfig;
use codex_protocol::models::PermissionProfile;
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;

#[test]
fn elevated_flag_works_by_itself() {
    let mut features = Features::with_defaults();
    features.enable(Feature::WindowsSandboxElevated);

    assert_eq!(
        WindowsSandboxLevel::from_features(&features),
        WindowsSandboxLevel::Elevated
    );
}

#[test]
fn restricted_token_flag_works_by_itself() {
    let mut features = Features::with_defaults();
    features.enable(Feature::WindowsSandbox);

    assert_eq!(
        WindowsSandboxLevel::from_features(&features),
        WindowsSandboxLevel::RestrictedToken
    );
}

#[test]
fn no_flags_means_no_sandbox() {
    let features = Features::with_defaults();

    assert_eq!(
        WindowsSandboxLevel::from_features(&features),
        WindowsSandboxLevel::Disabled
    );
}

#[test]
fn elevated_wins_when_both_flags_are_enabled() {
    let mut features = Features::with_defaults();
    features.enable(Feature::WindowsSandbox);
    features.enable(Feature::WindowsSandboxElevated);

    assert_eq!(
        WindowsSandboxLevel::from_features(&features),
        WindowsSandboxLevel::Elevated
    );
}

#[test]
fn legacy_mode_prefers_elevated() {
    let mut entries = BTreeMap::new();
    entries.insert(
        "experimental_windows_sandbox".to_string(),
        /*value*/ true,
    );
    entries.insert("elevated_windows_sandbox".to_string(), /*value*/ true);

    assert_eq!(
        legacy_windows_sandbox_mode_from_entries(&entries),
        Some(WindowsSandboxModeToml::Elevated)
    );
}

#[test]
fn legacy_mode_supports_alias_key() {
    let mut entries = BTreeMap::new();
    entries.insert(
        "enable_experimental_windows_sandbox".to_string(),
        /*value*/ true,
    );

    assert_eq!(
        legacy_windows_sandbox_mode_from_entries(&entries),
        Some(WindowsSandboxModeToml::Unelevated)
    );
}

#[test]
fn resolve_windows_sandbox_mode_falls_back_to_legacy_keys() {
    let mut entries = BTreeMap::new();
    entries.insert(
        "experimental_windows_sandbox".to_string(),
        /*value*/ true,
    );
    let cfg = ConfigToml {
        features: Some(FeaturesToml::from(entries)),
        ..Default::default()
    };

    assert_eq!(
        resolve_windows_sandbox_mode(&cfg),
        Some(WindowsSandboxModeToml::Unelevated)
    );
}

#[test]
fn provisioning_settings_omit_the_disabled_socks_proxy() {
    let config = NetworkProxyConfig {
        enabled: true,
        proxy_url: "http://127.0.0.1:43128".to_string(),
        enable_socks5: false,
        socks_url: "socks5h://127.0.0.1:48081".to_string(),
        allow_local_binding: Some(true),
        ..Default::default()
    };
    let spec = crate::config::NetworkProxySpec::from_config_and_constraints(
        config,
        /*requirements*/ None,
        &PermissionProfile::workspace_write(),
    )
    .expect("managed proxy config should resolve");

    assert_eq!(
        provisioning_settings(Some(&spec)).expect("provisioning settings should resolve"),
        codex_windows_sandbox::WindowsSandboxProvisioningSettings {
            proxy_ports: vec![43128],
            allow_local_binding: true,
        }
    );
}

#[test]
fn provisioning_settings_are_empty_when_managed_network_is_disabled() {
    let spec = crate::config::NetworkProxySpec::from_config_and_constraints(
        NetworkProxyConfig::default(),
        /*requirements*/ None,
        &PermissionProfile::workspace_write(),
    )
    .expect("managed proxy config should resolve");

    assert_eq!(
        provisioning_settings(Some(&spec)).expect("provisioning settings should resolve"),
        codex_windows_sandbox::WindowsSandboxProvisioningSettings::default()
    );
}

#[test]
fn pf_33_s02_destination_guard_drops_local_binding_from_provisioning() {
    let mut config = NetworkProxyConfig {
        enabled: true,
        proxy_url: "http://127.0.0.1:43128".to_string(),
        enable_socks5: false,
        allow_local_binding: true,
        ..Default::default()
    };
    config.set_url_destination_policy(/*enabled*/ true);
    let spec = crate::config::NetworkProxySpec::from_config_and_constraints(
        config,
        /*requirements*/ None,
        &PermissionProfile::workspace_write(),
    )
    .expect("managed proxy config should resolve");

    assert_eq!(
        provisioning_settings(Some(&spec)).expect("provisioning settings should resolve"),
        codex_windows_sandbox::WindowsSandboxProvisioningSettings {
            proxy_ports: vec![43128],
            allow_local_binding: false,
        }
    );
}

/// #2000: a config built from the session's starting layers (as a role
/// rebuilds a child agent's) gets the session's raised Windows sandbox, and a
/// stronger one it already selects stays.
#[tokio::test]
async fn issue_2000_raise_windows_sandbox_only_raises() {
    let level = |config: &Config| WindowsSandboxLevel::from_config(config);
    let mut config = crate::config::test_config().await;
    config.permissions.windows_sandbox_mode = None;
    for feature in [Feature::WindowsSandbox, Feature::WindowsSandboxElevated] {
        config.features.disable(feature).expect("test config");
    }
    assert_eq!(level(&config), WindowsSandboxLevel::Disabled);
    let starting = config.clone();

    for target in [
        WindowsSandboxLevel::Disabled,
        WindowsSandboxLevel::RestrictedToken,
        WindowsSandboxLevel::Elevated,
    ] {
        let mut raised = starting.clone();
        raise_windows_sandbox(&mut raised, target);
        assert_eq!(level(&raised), target);
    }
    let mut elevated = starting;
    elevated.set_windows_elevated_sandbox_enabled(/*value*/ true);
    for lower in [
        WindowsSandboxLevel::Disabled,
        WindowsSandboxLevel::RestrictedToken,
    ] {
        let mut kept = elevated.clone();
        raise_windows_sandbox(&mut kept, lower);
        assert_eq!(level(&kept), WindowsSandboxLevel::Elevated);
    }
}
