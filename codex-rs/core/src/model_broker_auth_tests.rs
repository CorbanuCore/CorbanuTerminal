use super::*;
use pretty_assertions::assert_eq;

use ProviderApiKeyHeader as ModelAuthHeader;

fn binding(host: &str, port: u16, path_prefix: &str, header: ModelAuthHeader) -> Binding {
    Binding {
        host: host.to_string(),
        port,
        path_prefix: path_prefix.to_string(),
        header,
    }
}

#[test]
fn pf_27_s05_base_url_binds_origin_and_path_prefix() {
    assert_eq!(
        binding_for_base_url("https://api.openai.com/v1", ProviderApiKeyHeader::Bearer),
        Some(binding(
            "api.openai.com",
            443,
            "/v1",
            ModelAuthHeader::Bearer
        ))
    );
    assert_eq!(
        binding_for_base_url(
            "https://API.Z.AI:8443/api/coding/paas/v4/",
            ProviderApiKeyHeader::Bearer
        ),
        Some(binding(
            "api.z.ai",
            8443,
            "/api/coding/paas/v4",
            ModelAuthHeader::Bearer
        ))
    );
    assert_eq!(
        binding_for_base_url("https://api.anthropic.com", ProviderApiKeyHeader::XApiKey),
        Some(binding(
            "api.anthropic.com",
            443,
            "/",
            ModelAuthHeader::XApiKey
        ))
    );
    // Plain HTTP, IPv6 literals, queries and garbage are not brokered.
    for url in [
        "http://localhost:11434/v1",
        "https://[::1]/v1",
        "https://api.example.com/v1?key=1",
        "not a url",
    ] {
        assert_eq!(
            binding_for_base_url(url, ProviderApiKeyHeader::Bearer),
            None,
            "{url}"
        );
    }
}

#[test]
fn pf_27_s05_requests_are_rewritten_to_plain_http_for_the_broker() {
    assert_eq!(
        BrokerRewrite::for_url("https://api.openai.com/v1/responses?stream=true"),
        Some(BrokerRewrite {
            host: "api.openai.com".to_string(),
            port: 443,
            path_and_query: "/v1/responses?stream=true".to_string(),
            broker_url: "http://api.openai.com:443/v1/responses?stream=true".to_string(),
        })
    );
    assert_eq!(
        BrokerRewrite::for_url("https://api.z.ai:8443/v4/chat/completions"),
        Some(BrokerRewrite {
            host: "api.z.ai".to_string(),
            port: 8443,
            path_and_query: "/v4/chat/completions".to_string(),
            broker_url: "http://api.z.ai:8443/v4/chat/completions".to_string(),
        })
    );
    for url in [
        "http://api.openai.com/v1/responses",
        "https://[::1]/v1",
        "https://a.example/v1#x",
    ] {
        assert_eq!(BrokerRewrite::for_url(url), None, "{url}");
    }
}

fn brokered_request(base_url: &str, source: BrokeredKeySource) -> BrokeredAuthRequest {
    BrokeredAuthRequest {
        base_url: base_url.to_string(),
        header: ProviderApiKeyHeader::Bearer,
        source,
        extra_headers: HeaderMap::new(),
    }
}

fn provider_key() -> BrokeredKeySource {
    BrokeredKeySource::ProviderKey {
        account: None,
        provider_key_id: "ZAI_API_KEY".to_string(),
        env_vars: vec!["ZAI_API_KEY".to_string()],
    }
}

fn held_value() -> BrokeredKeySource {
    BrokeredKeySource::Value {
        key: codex_model_provider::ProviderApiKey {
            value: "sk-pf27s05-held".to_string(),
            header: ProviderApiKeyHeader::Bearer,
        },
        slot: None,
    }
}

/// The error a brokered credential use fails with; it ends the turn.
fn refusal(broker: &CoreModelKeyBroker, base_url: &str, source: BrokeredKeySource) -> String {
    match broker.auth(brokered_request(base_url, source)) {
        Ok(_) => panic!("no auth may be produced"),
        Err(error) => {
            assert!(!error.is_retryable(), "{error:?}");
            let message = error.to_string();
            assert!(message.starts_with("Fatal error: "), "{message}");
            message
        }
    }
}

/// PF-27-S05 / PF-27-S06: the branch platforms without the broker take
/// (`start` returns this broker off Unix). Every credential use fails; none
/// is attached directly.
#[test]
fn pf_27_s05_platform_without_broker_refuses_every_credential() {
    let broker = CoreModelKeyBroker::unsupported();
    for source in [provider_key(), held_value()] {
        let error = refusal(&broker, "https://api.z.ai/api/paas/v4", source);
        assert!(error.contains("not available on this platform"), "{error}");
    }
}

#[cfg(not(any(unix, windows)))]
#[test]
fn pf_27_s05_non_unix_start_is_the_refusing_broker() {
    let broker = CoreModelKeyBroker::start(BrokerSettings {
        runtime_dir: std::env::temp_dir(),
        scrub_responses: false,
        program: None,
        store_home: std::env::temp_dir(),
        env_names: Vec::new(),
        origin: BrokerModelAuthOrigin::Config,
    });
    let error = refusal(&broker, "https://api.z.ai/api/paas/v4", held_value());
    assert!(error.contains("not available on this platform"), "{error}");
}

/// PF-27-S09 / #391: a broker that cannot start fails every credential use
/// (nothing is sent directly) with a message naming the setting and how to
/// change level, and the provider keys it would have been handed are still
/// removed from Core's environment.
#[cfg(any(unix, windows))]
#[test]
fn sec_391_broker_that_cannot_start_fails_closed_and_scrubs_keys() {
    let name = format!("SEC391_CORE_KEY_{}", std::process::id());
    // SAFETY: a variable unique to this test; nothing else reads it.
    unsafe { std::env::set_var(&name, "synthetic-sec391-core-key") };
    let home = tempfile::tempdir().expect("home");
    let broker = CoreModelKeyBroker::start(BrokerSettings {
        runtime_dir: home.path().join("run"),
        scrub_responses: false,
        program: Some(home.path().join("missing-corbanu.exe")),
        store_home: home.path().to_path_buf(),
        env_names: vec![name.clone()],
        origin: BrokerModelAuthOrigin::AggressiveLevel,
    });
    assert_eq!(std::env::var_os(&name), None);
    assert!(broker.unavailable_reason().is_some());
    for source in [provider_key(), held_value()] {
        let error = refusal(&broker, "https://api.z.ai/api/paas/v4", source);
        for expected in [
            "model requests are refused: the isolated credential broker did not start",
            "Security level Aggressive turns broker_model_auth on",
            "choose Permissive in /security",
            "`broker_model_auth = false`",
        ] {
            assert!(error.contains(expected), "{expected}: {error}");
        }
    }
}

/// #391: the fail-closed message names the setting, and says how to change
/// level only when the level turned it on.
#[test]
fn sec_391_not_started_message_names_the_setting_and_the_level() {
    for (cause, origin, present, absent) in [
        (
            StartFailure::Unavailable,
            BrokerModelAuthOrigin::AggressiveLevel,
            vec!["did not start", "Security level Aggressive", "/security"],
            vec![],
        ),
        (
            StartFailure::Unavailable,
            BrokerModelAuthOrigin::Config,
            vec![
                "did not start",
                "broker_model_auth is on in your configuration",
            ],
            vec!["/security", "Aggressive"],
        ),
        (
            StartFailure::Unavailable,
            BrokerModelAuthOrigin::Policy,
            vec!["did not start", "required by a managed policy"],
            vec!["/security", "Aggressive", "`broker_model_auth = false`"],
        ),
        (
            StartFailure::PipeSquatted,
            BrokerModelAuthOrigin::AggressiveLevel,
            vec![
                "held or served by another process",
                "sent nothing",
                "Security level Aggressive",
            ],
            vec![],
        ),
    ] {
        let broker = CoreModelKeyBroker::not_started(cause, origin);
        assert!(broker.unavailable_reason().is_some());
        let error = refusal(&broker, "https://api.z.ai/api/paas/v4", held_value());
        assert!(error.contains("nothing is sent without it"), "{error}");
        assert!(error.contains("broker_model_auth"), "{error}");
        for text in present {
            assert!(error.contains(text), "{cause:?} {origin:?} {text}: {error}");
        }
        for text in absent {
            assert!(
                !error.contains(text),
                "{cause:?} {origin:?} {text}: {error}"
            );
        }
    }
    assert_eq!(
        CoreModelKeyBroker::unsupported().unavailable_reason(),
        Some("the broker does not run on this system; model requests are refused")
    );
}

/// #391: the default matrix (level x OS x the person's own setting x a
/// project's setting). Only Aggressive on an OS where the broker runs turns
/// `broker_model_auth` on; the person's own setting always wins, and a
/// project layer cannot turn the Aggressive default off.
#[test]
fn sec_391_default_matrix_level_os_and_explicit_setting() {
    use SecurityLevel::Aggressive;
    use SecurityLevel::Moderate;
    use SecurityLevel::Permissive;
    // The feature's value as config loading resolves it: the level's
    // setting, else the merged config (project layers outrank the user's).
    let resolve = |level, supported, own: Option<bool>, project: Option<bool>| {
        level_broker_setting(level, supported, own)
            .unwrap_or_else(|| project.or(own).unwrap_or(false))
    };
    for level in [Permissive, Moderate, Aggressive] {
        for supported in [true, false] {
            for own in [None, Some(true), Some(false)] {
                for project in [None, Some(true), Some(false)] {
                    let expected = if level == Aggressive && supported {
                        own.unwrap_or(true)
                    } else {
                        project.or(own).unwrap_or(false)
                    };
                    assert_eq!(
                        resolve(level, supported, own, project),
                        expected,
                        "{level:?} supported={supported} own={own:?} project={project:?}"
                    );
                }
            }
        }
    }
    // Spot checks of the rows that matter most.
    assert_eq!(level_broker_setting(Aggressive, true, None), Some(true));
    assert_eq!(
        level_broker_setting(Aggressive, true, Some(false)),
        Some(false)
    );
    assert_eq!(level_broker_setting(Aggressive, false, None), None);
    assert_eq!(level_broker_setting(Moderate, true, None), None);
    assert_eq!(level_broker_setting(Permissive, true, None), None);
    assert!(resolve(Aggressive, true, Some(true), Some(false)));
    assert!(!resolve(Aggressive, true, Some(false), Some(true)));
    // macOS and Linux (PF-27-S05) and Windows (PF-27-S09) run the broker.
    assert_eq!(
        LEVEL_DEFAULT_SUPPORTED,
        cfg!(any(target_os = "macos", target_os = "linux", windows))
    );
}

/// #391: a provider URL the broker cannot bind names the level when the
/// level turned the broker on.
#[test]
fn sec_391_unbindable_url_names_the_level() {
    let broker =
        CoreModelKeyBroker::unsupported().with_origin(BrokerModelAuthOrigin::AggressiveLevel);
    let error = refusal(&broker, "http://localhost:11434/v1", held_value());
    for expected in [
        "cannot be brokered",
        "Security level Aggressive turns broker_model_auth on",
        "choose Permissive in /security",
    ] {
        assert!(error.contains(expected), "{expected}: {error}");
    }
}

/// #391 through config loading: Core's level, the stored `/security` level,
/// a config setting and a launch flag (`-c`) on this OS.
#[tokio::test]
async fn sec_391_config_load_applies_the_aggressive_default() {
    use crate::config::ConfigBuilder;
    use codex_config::LoaderOverrides;
    let expected_on = LEVEL_DEFAULT_SUPPORTED;
    let level_origin = if expected_on {
        BrokerModelAuthOrigin::AggressiveLevel
    } else {
        BrokerModelAuthOrigin::Config
    };
    let load = |config_toml: String, stored_aggressive: bool, cli: Vec<(String, toml::Value)>| async move {
        let home = tempfile::tempdir().expect("home");
        std::fs::write(home.path().join("config.toml"), config_toml).expect("config");
        if stored_aggressive {
            std::fs::write(
                home.path().join("security_level.toml"),
                "version = 1\nlevel = \"aggressive\"\n",
            )
            .expect("stored level");
        }
        let config = ConfigBuilder::default()
            .codex_home(home.path().to_path_buf())
            .cli_overrides(cli)
            .loader_overrides(LoaderOverrides::without_managed_config_for_tests())
            .build()
            .await
            .expect("config");
        (
            config.security_level,
            config
                .features
                .enabled(codex_features::Feature::BrokerModelAuth),
            config.broker_model_auth_origin,
        )
    };
    let level =
        |level: SecurityLevel| format!("[security]\nversion = 1\nlevel = \"{}\"\n", level.as_str());
    let off = "\n[features]\nbroker_model_auth = false\n";
    let flag_off = || {
        vec![(
            "features.broker_model_auth".to_string(),
            toml::Value::Boolean(false),
        )]
    };
    for (name, config_toml, stored, cli, expected) in [
        (
            "permissive",
            level(SecurityLevel::Permissive),
            false,
            Vec::new(),
            (
                SecurityLevel::Permissive,
                false,
                BrokerModelAuthOrigin::Config,
            ),
        ),
        (
            "moderate",
            level(SecurityLevel::Moderate),
            false,
            Vec::new(),
            (
                SecurityLevel::Moderate,
                false,
                BrokerModelAuthOrigin::Config,
            ),
        ),
        (
            "aggressive",
            level(SecurityLevel::Aggressive),
            false,
            Vec::new(),
            (SecurityLevel::Aggressive, expected_on, level_origin),
        ),
        (
            "aggressive, config off",
            level(SecurityLevel::Aggressive) + off,
            false,
            Vec::new(),
            (
                SecurityLevel::Aggressive,
                false,
                BrokerModelAuthOrigin::Config,
            ),
        ),
        (
            "aggressive, -c off",
            level(SecurityLevel::Aggressive),
            false,
            flag_off(),
            (
                SecurityLevel::Aggressive,
                false,
                BrokerModelAuthOrigin::Config,
            ),
        ),
        (
            "stored /security aggressive",
            String::new(),
            true,
            Vec::new(),
            (SecurityLevel::Permissive, expected_on, level_origin),
        ),
        (
            "moderate, stored /security aggressive",
            level(SecurityLevel::Moderate),
            true,
            Vec::new(),
            (SecurityLevel::Moderate, expected_on, level_origin),
        ),
        (
            "stored /security aggressive, config off",
            off.to_string(),
            true,
            Vec::new(),
            (
                SecurityLevel::Permissive,
                false,
                BrokerModelAuthOrigin::Config,
            ),
        ),
    ] {
        assert_eq!(load(config_toml, stored, cli).await, expected, "{name}");
    }
}

#[test]
fn pf_27_s05_failed_broker_and_unbindable_urls_fail_closed() {
    let failed = CoreModelKeyBroker::new(BrokerHandle::Failed(BrokerModelAuthError::Unavailable));
    let error = refusal(&failed, "https://api.z.ai/api/paas/v4", provider_key());
    assert!(error.contains("unavailable"), "{error}");

    // A URL the broker cannot bind is refused before any broker call.
    for base_url in ["http://localhost:11434/v1", "https://[::1]/v1"] {
        let error = refusal(&CoreModelKeyBroker::unsupported(), base_url, held_value());
        assert!(error.contains("cannot be brokered"), "{base_url}: {error}");
    }
}

/// PF-27-S09: what the Windows sender puts on the pipe for a rewritten URL.
#[test]
fn pf_27_s09_pipe_request_target_keeps_the_signed_path_and_query() {
    assert_eq!(
        pipe_request_target("http://api.z.ai:443/api/paas/v4/chat/completions?x=1&y=2"),
        Some((
            "api.z.ai:443".to_string(),
            "/api/paas/v4/chat/completions?x=1&y=2".to_string()
        ))
    );
    // The broker URL is what `broker_request` produced for this request.
    let rewrite =
        BrokerRewrite::for_url("https://api.z.ai/api/paas/v4/models?page=2").expect("rewrite");
    assert_eq!(
        pipe_request_target(&rewrite.broker_url),
        Some(("api.z.ai:443".to_string(), rewrite.path_and_query))
    );
    for url in [
        "https://api.z.ai:443/v1",
        "http://user:secret@api.z.ai:443/v1",
        "http://user@api.z.ai:443/v1",
        "http://api.z.ai:443/v1#fragment",
        "not a url",
    ] {
        assert_eq!(pipe_request_target(url), None, "{url}");
    }
}

/// #390: a broker refused because another process holds or serves its named
/// pipe fails every credential use with a message that says so; nothing is
/// sent directly.
#[test]
fn sec_390_squatted_broker_pipe_fails_closed_with_a_clear_error() {
    let squatted =
        CoreModelKeyBroker::new(BrokerHandle::Failed(BrokerModelAuthError::PipeSquatted));
    for source in [provider_key(), held_value()] {
        let error = refusal(&squatted, "https://api.z.ai/api/paas/v4", source);
        assert!(
            error.contains("held or served by another process"),
            "{error}"
        );
        assert!(error.contains("sent nothing"), "{error}");
    }
}

/// #391: a project's `.codex/config.toml` cannot turn the broker off under
/// Aggressive (a repository may raise protection, never lower it), while the
/// person's own config can.
#[tokio::test]
async fn sec_391_project_config_cannot_turn_the_aggressive_default_off() {
    use crate::config::ConfigBuilder;
    use crate::config::ConfigOverrides;
    use codex_config::LoaderOverrides;
    let root = tempfile::tempdir().expect("root");
    // No `\\?\` prefix on Windows, so the trust key matches the folder.
    let root_path = dunce::canonicalize(root.path()).expect("canonical root");
    let project = root_path.join("project");
    std::fs::create_dir_all(project.join(".git")).expect("project");
    std::fs::create_dir_all(project.join(".codex")).expect("project config dir");
    std::fs::write(
        project.join(".codex/config.toml"),
        "[features]\nbroker_model_auth = false\n",
    )
    .expect("project config");
    let load = |user_features: &'static str| {
        let home = root_path.join(format!("home-{}", user_features.len()));
        let project = project.clone();
        async move {
            std::fs::create_dir_all(&home).expect("home");
            std::fs::write(
                home.join("config.toml"),
                format!(
                    "[security]\nversion = 1\nlevel = \"aggressive\"\n\n[projects.{:?}]\ntrust_level = \"trusted\"\n{user_features}",
                    codex_config::loader::project_trust_key(&project)
                ),
            )
            .expect("user config");
            let config = ConfigBuilder::default()
                .codex_home(home)
                .harness_overrides(ConfigOverrides {
                    cwd: Some(project),
                    ..Default::default()
                })
                .loader_overrides(LoaderOverrides::without_managed_config_for_tests())
                .build()
                .await
                .expect("config");
            assert!(
                config
                    .config_layer_stack
                    .layers_high_to_low()
                    .iter()
                    .any(|layer| matches!(
                        layer.name,
                        codex_config::ConfigLayerSource::Project { .. }
                    )),
                "the project layer loads"
            );
            config
                .features
                .enabled(codex_features::Feature::BrokerModelAuth)
        }
    };
    assert_eq!(load("").await, LEVEL_DEFAULT_SUPPORTED);
    assert!(!load("\n[features]\nbroker_model_auth = false\n").await);
    // The person's explicit `true` also beats the project's `false`. Only on
    // an OS with the broker: elsewhere it would mark this process brokered.
    if LEVEL_DEFAULT_SUPPORTED {
        assert!(load("\n[features]\nbroker_model_auth = true\n").await);
    }
}

/// #391: a managed requirement wins over the level, both ways, and a pin
/// that keeps the broker on is named in the refusal message.
#[tokio::test]
async fn sec_391_managed_requirement_wins_and_is_named() {
    use crate::config::ConfigBuilder;
    use codex_config::test_support::CloudConfigBundleFixture;
    let load = |pin: bool, user: &'static str| async move {
        let home = tempfile::tempdir().expect("home");
        std::fs::write(
            home.path().join("config.toml"),
            format!("[security]\nversion = 1\nlevel = \"aggressive\"\n{user}"),
        )
        .expect("config");
        let config = ConfigBuilder::without_managed_config_for_tests()
            .codex_home(home.path().to_path_buf())
            .fallback_cwd(Some(home.path().to_path_buf()))
            .cloud_config_bundle(
                CloudConfigBundleFixture::loader_with_enterprise_requirement(format!(
                    "[features]\nbroker_model_auth = {pin}\n"
                )),
            )
            .build()
            .await
            .expect("config");
        (
            config
                .features
                .enabled(codex_features::Feature::BrokerModelAuth),
            config.broker_model_auth_origin,
        )
    };
    assert_eq!(
        load(false, "").await,
        (false, BrokerModelAuthOrigin::Config)
    );
    // A pin that keeps the broker on would mark this shared test process
    // brokered where the level sets nothing (no broker on this OS).
    if LEVEL_DEFAULT_SUPPORTED {
        assert_eq!(load(true, "").await, (true, BrokerModelAuthOrigin::Policy));
        assert_eq!(
            load(true, "\n[features]\nbroker_model_auth = false\n").await,
            (true, BrokerModelAuthOrigin::Policy)
        );
    }
}
