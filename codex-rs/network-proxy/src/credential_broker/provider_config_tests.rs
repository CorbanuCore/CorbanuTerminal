use super::*;
use pretty_assertions::assert_eq;

fn provider(env: &[&str]) -> CredentialProviderConfig {
    CredentialProviderConfig {
        env: env.iter().map(ToString::to_string).collect(),
        patterns: vec!["vendor_[a-z]{8}".to_string()],
        url_prefixes: vec!["https://api.vendor.example".to_string()],
        ..CredentialProviderConfig::default()
    }
}

fn providers(entries: &[(&str, &[&str])]) -> BTreeMap<String, CredentialProviderConfig> {
    entries
        .iter()
        .map(|(id, env)| (id.to_string(), provider(env)))
        .collect()
}

#[test]
fn unbrokered_sources_are_rejected_with_provider_and_reason() {
    let error = reject_unbrokered_credential_providers(&providers(&[
        ("vendor", &["VENDOR_TOKEN"]),
        ("zz_other", &["OTHER_SECRET", "SECOND_SECRET"]),
    ]))
    .expect_err("an unbrokered source must be refused");
    assert_eq!(
        error,
        "unsupported network proxy credential provider `vendor` (VENDOR_TOKEN), `zz_other` \
         (OTHER_SECRET, SECOND_SECRET): Corbanu's credential engine does not broker these \
         environment variables. It brokers only its built-in credential variables \
         (GH_TOKEN, GITHUB_TOKEN, GH_ENTERPRISE_TOKEN, GITHUB_ENTERPRISE_TOKEN, OPENAI_API_KEY), \
         so these values would be withheld from every command. Remove them from \
         [features.network_proxy.credentials] in config.toml or the selected profile's config \
         file; to keep a variable away from commands, add it to \
         shell_environment_policy.exclude instead."
    );
}

#[test]
fn built_in_sources_a_provider_names_load_as_before() {
    // #2630: built-in keys a provider names stay with the hardened engine,
    // which masks them; the run-time guard drops one it leaves raw.
    for env in [
        &["OPENAI_API_KEY"][..],
        &["GH_TOKEN", "GITHUB_TOKEN"],
        &["GH_ENTERPRISE_TOKEN", "GITHUB_ENTERPRISE_TOKEN"],
    ] {
        assert_eq!(
            reject_unbrokered_credential_providers(&providers(&[("builtin", env)])),
            Ok(()),
            "{env:?}"
        );
    }
    assert_eq!(
        reject_unbrokered_credential_providers(&BTreeMap::new()),
        Ok(())
    );
}

#[test]
fn mixed_provider_names_only_its_unbrokered_sources() {
    let error = reject_unbrokered_credential_providers(&providers(&[
        ("builtin", &["GITHUB_TOKEN"]),
        ("mixed", &["GH_ENTERPRISE_TOKEN", "VENDOR_TOKEN"]),
    ]))
    .expect_err("the vendor source must be refused");
    assert!(
        error.starts_with("unsupported network proxy credential provider `mixed` (VENDOR_TOKEN): "),
        "{error}"
    );
}

#[test]
fn built_in_context_and_lookalike_keys_are_not_brokered_sources() {
    // Host context variables carry no credential the engine brokers, and on
    // Unix variable names are case-sensitive, as the run-time guard treats them.
    for key in ["GH_HOST", "OPENAI_BASE_URL", "GITHUB_TOKEN_BACKUP"] {
        assert!(
            reject_unbrokered_credential_providers(&providers(&[("p", &[key])])).is_err(),
            "{key}"
        );
    }
    assert_eq!(
        reject_unbrokered_credential_providers(&providers(&[("p", &["github_token"])])).is_ok(),
        cfg!(windows)
    );
}
