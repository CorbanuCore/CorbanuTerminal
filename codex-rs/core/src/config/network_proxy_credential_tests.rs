use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::RevocationEvent;
use codex_security_policy::RevocationReason;
use codex_security_policy::RevocationTarget;
use pretty_assertions::assert_eq;

use super::*;

fn human() -> PolicyPrincipal {
    PolicyPrincipal::new(PrincipalKind::Human, "human:owner").expect("human")
}

#[test]
fn credential_authority_revoke_during_use_linearizes_after_the_active_resolution() {
    let revocations = RwLock::new(RevocationState::new());

    with_current_revocations(&revocations, |current| {
        assert_eq!(current.generation, 0);
        assert!(
            revocations.try_write().is_err(),
            "revocation must not mutate the state observed by an active resolution"
        );
    })
    .expect("active resolution");

    let event = RevocationEvent::new(
        human(),
        RevocationTarget::AllActiveAuthority,
        RevocationReason::HumanRequest,
        /*created_at_unix_seconds*/ 101,
    )
    .expect("revocation event");
    revocations
        .write()
        .expect("revocation write after resolution")
        .apply(&event)
        .expect("apply revocation");

    with_current_revocations(&revocations, |current| {
        assert_eq!(current.generation, 1);
    })
    .expect("next resolution observes revocation");
}

#[test]
fn pf_27_s04_broker_failures_map_without_raw_credential_fallback() {
    assert_eq!(
        map_broker_client_error(BrokerClientError::Denied),
        IsolatedCredentialDispatchError::Denied
    );
    assert_eq!(
        map_broker_client_error(BrokerClientError::Cancelled),
        IsolatedCredentialDispatchError::Cancelled
    );
    assert_eq!(
        map_broker_client_error(BrokerClientError::OutcomeUnknown),
        IsolatedCredentialDispatchError::OutcomeUnknown
    );
    assert_eq!(
        map_broker_client_error(BrokerClientError::Unavailable),
        IsolatedCredentialDispatchError::Unavailable
    );
}

mod pf_27_s04 {
    use crate::config::ConfigBuilder;
    use crate::config::ConfigOverrides;
    use codex_network_proxy::NetworkProxyAuditMetadata;
    use codex_network_proxy::ScopedCredentialInjectionError;
    use http::HeaderMap;
    use http::HeaderValue;
    use http::header::AUTHORIZATION;
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;
    use tempfile::TempDir;

    const SYNTHETIC_OPENAI_KEY: &str = "sk-pf27-synthetic-canary-000000000000000000000000000000";

    async fn network_config(features: &str) -> std::io::Result<crate::config::Config> {
        let codex_home = TempDir::new()?;
        let cwd = TempDir::new()?;
        std::fs::write(
            codex_home.path().join(crate::config::CONFIG_TOML_FILE),
            format!(
                r#"
sandbox_mode = "workspace-write"

[sandbox_workspace_write]
network_access = true

[features]
network_proxy = true
{features}
"#
            ),
        )?;
        ConfigBuilder::without_managed_config_for_tests()
            .codex_home(codex_home.path().to_path_buf())
            .harness_overrides(ConfigOverrides {
                cwd: Some(cwd.path().to_path_buf()),
                ..Default::default()
            })
            .build()
            .await
    }

    #[tokio::test]
    async fn pf_27_s04_pf_27_s01_isolated_broker_flag_is_off_by_default() -> std::io::Result<()> {
        let config = network_config("").await?;
        let network = config
            .permissions
            .network
            .as_ref()
            .expect("managed network proxy");
        assert!(!network.isolated_credential_broker_enabled());
        Ok(())
    }

    #[tokio::test]
    async fn pf_27_s04_pf_27_s01_isolated_broker_flag_keeps_raw_values_out_of_core()
    -> std::io::Result<()> {
        let config = network_config("isolated_credential_broker = true").await?;
        let network = config
            .permissions
            .network
            .as_ref()
            .expect("managed network proxy");
        assert!(network.isolated_credential_broker_enabled());

        // The child sees only a dummy, and Core's in-process injection path has
        // no raw value to add: only the broker process can substitute it.
        let state = network.build_state_with_audit_metadata(NetworkProxyAuditMetadata::default())?;
        let mut env = HashMap::from([(
            "OPENAI_API_KEY".to_string(),
            SYNTHETIC_OPENAI_KEY.to_string(),
        )]);
        state.virtualize_child_credentials(&mut env);
        let dummy = env["OPENAI_API_KEY"].clone();
        assert_ne!(dummy, SYNTHETIC_OPENAI_KEY);

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {dummy}")).expect("header"),
        );
        let original = headers.clone();
        let result = state.inject_request_credentials(
            "https",
            "api.openai.com",
            /*port*/ 443,
            "POST",
            "/v1/responses",
            &mut headers,
        );
        assert!(matches!(
            result,
            Ok(()) | Err(ScopedCredentialInjectionError::IsolatedBrokerUnavailable)
        ));
        assert_eq!(headers, original);
        assert!(headers.values().all(|value| {
            !value
                .as_bytes()
                .windows(SYNTHETIC_OPENAI_KEY.len())
                .any(|window| window == SYNTHETIC_OPENAI_KEY.as_bytes())
        }));
        Ok(())
    }
}
