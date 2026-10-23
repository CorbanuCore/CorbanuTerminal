use super::layered_security_floor;
use super::load_config_toml_with_layer_stack;
use crate::security::recovery;
use codex_config::ConfigLayerSource;
use codex_config::ConfigLayerStack;
use codex_config::ConfigLoadError;
use codex_config::ConfigLoadOptions;
use codex_config::LoaderOverrides;
use codex_config::NoopThreadConfigLoader;
use codex_config::config_toml::ConfigToml;
use codex_config::loader::load_config_layers_state;
use codex_exec_server::LOCAL_FS;
use codex_security_policy::RevocationState;
use codex_security_policy::SecurityLevel;
use codex_utils_absolute_path::AbsolutePathBuf;
use std::path::PathBuf;

struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    home: PathBuf,
    project: PathBuf,
}

impl Fixture {
    fn new(project_config: &str, trusted: bool, user_security: &str) -> Self {
        let temp = tempfile::tempdir().expect("fixture directory");
        let root = dunce::canonicalize(temp.path()).expect("canonical fixture root");
        let home = root.join("profile");
        let project = root.join("project");
        std::fs::create_dir_all(&home).expect("fixture profile");
        std::fs::create_dir_all(project.join(".codex")).expect("fixture project");
        std::fs::create_dir_all(root.join("account")).expect("fixture account");
        std::fs::create_dir_all(root.join("managed")).expect("fixture managed input directory");
        std::fs::write(project.join(".fixture-root"), "").expect("project root marker");
        let trust = if trusted { "trusted" } else { "untrusted" };
        let project_key = toml::Value::String(project.display().to_string());
        std::fs::write(
            home.join("config.toml"),
            format!(
                "model = \"fixture-user\"\nmodel_provider = \"fixture-owner\"\n\
                 project_root_markers = [\".fixture-root\"]\n\
                 {user_security}\n[projects.{project_key}]\ntrust_level = \"{trust}\"\n"
            ),
        )
        .expect("synthetic user config");
        std::fs::write(project.join(".codex/config.toml"), project_config)
            .expect("synthetic project config");
        let saved =
            recovery::DurableSecurityState::new(SecurityLevel::Moderate, RevocationState::new());
        std::fs::write(
            home.join(recovery::STATE_FILE),
            serde_json::to_vec(&saved).expect("serialize synthetic saved state"),
        )
        .expect("synthetic saved Moderate state");
        Self {
            _temp: temp,
            root,
            home,
            project,
        }
    }

    fn loader_overrides(&self) -> LoaderOverrides {
        LoaderOverrides {
            managed_config_path: Some(self.root.join("managed/legacy.toml")),
            system_config_path: Some(self.root.join("managed/config.toml")),
            system_requirements_path: Some(self.root.join("managed/requirements.toml")),
            user_home_dir: Some(self.root.join("account")),
            #[cfg(target_os = "macos")]
            managed_preferences_base64: Some(String::new()),
            macos_managed_config_requirements_base64: Some(String::new()),
            ..Default::default()
        }
    }

    async fn load(&self, strict: bool) -> std::io::Result<ConfigLayerStack> {
        let overrides = self.loader_overrides();
        load_config_layers_state(
            LOCAL_FS.as_ref(),
            &self.home,
            Some(AbsolutePathBuf::from_absolute_path(&self.project)?),
            &[],
            ConfigLoadOptions {
                loader_overrides: overrides,
                strict_config: strict,
                ..Default::default()
            },
            &NoopThreadConfigLoader,
        )
        .await
    }

    /// The bootstrap load the TUI and `corbanu exec` run first at startup.
    async fn load_bootstrap(&self, strict: bool) -> std::io::Result<()> {
        self.load_bootstrap_with(strict, Vec::new()).await
    }

    async fn load_bootstrap_with(
        &self,
        strict: bool,
        cli_overrides: Vec<(String, toml::Value)>,
    ) -> std::io::Result<()> {
        load_config_toml_with_layer_stack(
            &self.home,
            Some(&AbsolutePathBuf::from_absolute_path(&self.project)?),
            cli_overrides,
            ConfigLoadOptions {
                loader_overrides: self.loader_overrides(),
                strict_config: strict,
                ..Default::default()
            },
        )
        .await
        .map(|_| ())
    }

    // Exercise the native config preflight and durable recovery without starting
    // providers, the TUI, a database or any credential/financial services.
    fn recovered_level(&self, stack: &ConfigLayerStack) -> SecurityLevel {
        let cfg: ConfigToml = stack
            .effective_config()
            .try_into()
            .expect("typed preflight");
        let settings = cfg.security.unwrap_or_default();
        settings.validate().expect("versioned security settings");
        let configured = settings.level.max(layered_security_floor(stack));
        let recovered = recovery::recover(&self.home, configured);
        assert!(recovered.unreadable.is_none());
        recovered.level
    }

    fn project_path(&self) -> String {
        self.project
            .join(".codex")
            .join("config.toml")
            .display()
            .to_string()
    }
}

#[tokio::test]
async fn incomplete_project_permissive_allows_saved_moderate_restart() {
    for strict in [false, true] {
        let fixture = Fixture::new(
            "[security]\nlevel = \"permissive\"\n\n[features]\nbroker_model_auth = false\n",
            /*trusted*/ true,
            "",
        );
        let state_path = fixture.home.join(recovery::STATE_FILE);
        let before = std::fs::read(&state_path).expect("saved state bytes");
        let stack = fixture.load(strict).await.expect("project config loads");
        assert_eq!(fixture.recovered_level(&stack), SecurityLevel::Moderate);
        let warnings = stack.startup_warnings().expect("project diagnostics");
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains(&fixture.project_path()));
        assert!(warnings[0].contains("[security]"));
        assert!(warnings[0].contains("Ignored"));
        assert!(warnings[0].contains("version"));
        assert_eq!(
            stack.effective_config()["features"]["broker_model_auth"].as_bool(),
            Some(false)
        );
        assert_eq!(
            std::fs::read(&state_path).expect("saved state bytes"),
            before
        );
    }
}

#[tokio::test]
async fn incomplete_project_security_does_not_inherit_user_version() {
    let fixture = Fixture::new(
        "[security]\nlevel = \"permissive\"\n",
        /*trusted*/ true,
        "[security]\nversion = 1\nlevel = \"moderate\"\n",
    );
    let stack = fixture.load(/*strict*/ false).await.expect("load project");
    let cfg: ConfigToml = stack
        .effective_config()
        .try_into()
        .expect("typed preflight");
    assert_eq!(
        cfg.security.expect("trusted settings").level,
        SecurityLevel::Moderate
    );
    assert_eq!(fixture.recovered_level(&stack), SecurityLevel::Moderate);
    assert!(stack.startup_warnings().expect("warnings")[0].contains(&fixture.project_path()));
}

#[tokio::test]
async fn versioned_project_security_preserves_both_floor_directions() {
    for (project_level, expected) in [
        ("permissive", SecurityLevel::Moderate),
        ("aggressive", SecurityLevel::Aggressive),
    ] {
        let fixture = Fixture::new(
            &format!("[security]\nversion = 1\nlevel = \"{project_level}\"\n"),
            /*trusted*/ true,
            "[security]\nversion = 1\nlevel = \"moderate\"\n",
        );
        let stack = fixture
            .load(/*strict*/ true)
            .await
            .expect("valid project settings");
        assert_eq!(layered_security_floor(&stack), expected);
        assert_eq!(fixture.recovered_level(&stack), expected);
        assert!(stack.startup_warnings().unwrap_or_default().is_empty());
    }
}

#[tokio::test]
async fn untrusted_project_security_stays_disabled() {
    let fixture = Fixture::new(
        "[security]\nlevel = \"permissive\"\n",
        /*trusted*/ false,
        "",
    );
    let stack = fixture
        .load(/*strict*/ true)
        .await
        .expect("disabled project");
    assert!(
        stack
            .layers_high_to_low()
            .iter()
            .all(|layer| !matches!(layer.name, ConfigLayerSource::Project { .. }))
    );
    assert_eq!(fixture.recovered_level(&stack), SecurityLevel::Moderate);
    assert!(stack.startup_warnings().unwrap_or_default().is_empty());
}

#[tokio::test]
async fn trusted_user_missing_security_version_still_errors() {
    let fixture = Fixture::new(
        "",
        /*trusted*/ true,
        "[security]\nlevel = \"permissive\"\n",
    );
    let stack = fixture.load(/*strict*/ false).await.expect("raw layers");
    let error = stack
        .effective_config()
        .try_into::<ConfigToml>()
        .expect_err("user version required");
    assert!(error.to_string().contains("missing field `version`"));
    assert!(fixture.load(/*strict*/ true).await.is_err());
}

#[tokio::test]
async fn other_invalid_project_security_remains_an_error() {
    for security in [
        "level = \"aggressive\"",
        "level = \"permissive\"\nunknown = true",
        "version = \"invalid\"\nlevel = \"permissive\"",
        "version = 1\nlevel = \"unknown\"",
    ] {
        let fixture = Fixture::new(
            &format!("[security]\n{security}\n"),
            /*trusted*/ true,
            "",
        );
        let stack = fixture.load(/*strict*/ false).await.expect("raw layers");
        assert!(stack.effective_config().try_into::<ConfigToml>().is_err());
        assert!(fixture.load(/*strict*/ true).await.is_err());
    }
}

#[tokio::test]
async fn malformed_project_toml_keeps_file_specific_error() {
    let fixture = Fixture::new(
        "[security\nlevel = \"permissive\"",
        /*trusted*/ true,
        "",
    );
    let error = fixture
        .load(/*strict*/ false)
        .await
        .expect_err("invalid TOML");
    assert!(error.to_string().contains(&fixture.project_path()));
}

#[tokio::test]
async fn ordinary_project_precedence_and_denied_provider_are_preserved() {
    let fixture = Fixture::new(
        "model = \"fixture-project\"\nmodel_provider = \"untrusted-provider\"\n",
        /*trusted*/ true,
        "",
    );
    let stack = fixture
        .load(/*strict*/ false)
        .await
        .expect("ordinary project config");
    let cfg: ConfigToml = stack.effective_config().try_into().expect("typed config");
    assert_eq!(cfg.model.as_deref(), Some("fixture-project"));
    assert_eq!(cfg.model_provider.as_deref(), Some("fixture-owner"));
    assert_eq!(fixture.recovered_level(&stack), SecurityLevel::Moderate);
    assert!(stack.startup_warnings().expect("warning")[0].contains("model_provider"));
}

#[tokio::test]
async fn unversioned_project_security_startup_error_names_the_project_file() {
    // #845 option (a): these tables are still refused, but the startup error
    // must point at the project file so the person can fix it.
    for (security, message) in [
        ("level = \"aggressive\"", "missing field `version`"),
        ("level = \"moderate\"", "missing field `version`"),
        (
            "level = \"permissive\"\nunknown = true",
            "unknown field `unknown`",
        ),
    ] {
        let fixture = Fixture::new(
            &format!("[security]\n{security}\n"),
            /*trusted*/ true,
            "",
        );
        let error = fixture
            .load_bootstrap(/*strict*/ false)
            .await
            .expect_err("unversioned project security is refused");
        let config_error = error
            .get_ref()
            .and_then(|error| error.downcast_ref::<ConfigLoadError>())
            .map(ConfigLoadError::config_error)
            .unwrap_or_else(|| panic!("no file-specific error for {security:?}: {error}"));
        assert_eq!(
            config_error.path.display().to_string(),
            fixture.project_path(),
            "{security:?}"
        );
        assert!(config_error.message.contains(message), "{config_error:?}");
        assert!(
            error.to_string().contains(&fixture.project_path()),
            "{error}"
        );
    }
}

#[tokio::test]
async fn bootstrap_load_still_accepts_versioned_and_dropped_project_security() {
    for project in [
        "[security]\nversion = 1\nlevel = \"aggressive\"\n",
        "[security]\nlevel = \"permissive\"\n",
    ] {
        let fixture = Fixture::new(project, /*trusted*/ true, "");
        fixture
            .load_bootstrap(/*strict*/ false)
            .await
            .unwrap_or_else(|error| panic!("{project:?}: {error}"));
    }
}

fn file_error(error: &std::io::Error) -> Option<&codex_config::ConfigError> {
    error
        .get_ref()
        .and_then(|error| error.downcast_ref::<ConfigLoadError>())
        .map(ConfigLoadError::config_error)
}

#[tokio::test]
async fn dropped_project_security_is_not_blamed_for_another_failure() {
    // #850 drops this table with a warning; a failure elsewhere must keep
    // its own message instead of a "missing field `version`" for it.
    let fixture = Fixture::new(
        "[security]\nlevel = \"permissive\"\n",
        /*trusted*/ true,
        "",
    );
    let error = fixture
        .load_bootstrap_with(
            /*strict*/ false,
            vec![(
                "model_context_window".to_string(),
                toml::Value::String("bogus".to_string()),
            )],
        )
        .await
        .expect_err("bad override");
    assert!(!error.to_string().contains("version"), "{error}");
    assert!(error.to_string().contains("bogus"), "{error}");
    assert!(file_error(&error).is_none(), "{error}");

    // A real error in the same project file is the one reported.
    let fixture = Fixture::new(
        "model_context_window = \"bogus\"\n\n[security]\nlevel = \"permissive\"\n",
        /*trusted*/ true,
        "",
    );
    let error = fixture
        .load_bootstrap(/*strict*/ false)
        .await
        .expect_err("bad project value");
    let config_error = file_error(&error).expect("file-specific error");
    assert_eq!(
        config_error.path.display().to_string(),
        fixture.project_path()
    );
    assert!(config_error.message.contains("bogus"), "{config_error:?}");
    assert_eq!(config_error.range.start.line, 1, "{config_error:?}");
}

#[tokio::test]
async fn untrusted_unversioned_project_security_is_neither_loaded_nor_blamed() {
    let fixture = Fixture::new(
        "[security]\nlevel = \"aggressive\"\n",
        /*trusted*/ false,
        "",
    );
    fixture
        .load_bootstrap(/*strict*/ false)
        .await
        .expect("an untrusted project layer is disabled");
}
