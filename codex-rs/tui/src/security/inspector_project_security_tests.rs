use super::*;
use crate::legacy_core::config::ConfigBuilder;
use crate::legacy_core::config::ConfigOverrides;
use crate::legacy_core::config::load_config_toml_with_layer_stack;
use codex_config::ConfigLoadOptions;
use codex_config::LoaderOverrides;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;

struct Fixture {
    _temp: tempfile::TempDir,
    home: PathBuf,
    project: PathBuf,
    account: PathBuf,
}

impl Fixture {
    fn new(project_config: &str, trust: &str) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(temp.path()).unwrap();
        let home = root.join("profile");
        let project = root.join("project");
        let account = root.join("account");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(project.join(".codex")).unwrap();
        std::fs::create_dir_all(&account).unwrap();
        std::fs::write(project.join(".fixture-root"), "").unwrap();
        let project_key = toml::Value::String(project.display().to_string());
        std::fs::write(
            home.join("config.toml"),
            format!(
                "project_root_markers = [\".fixture-root\"]\n\
                 approval_policy = \"never\"\nsandbox_mode = \"read-only\"\n\
                 [security]\nversion = 1\nlevel = \"moderate\"\n\
                 [projects.{project_key}]\ntrust_level = \"{trust}\"\n"
            ),
        )
        .unwrap();
        std::fs::write(project.join(".codex/config.toml"), project_config).unwrap();
        Self {
            _temp: temp,
            home,
            project,
            account,
        }
    }

    fn loader_overrides(&self) -> LoaderOverrides {
        LoaderOverrides {
            user_home_dir: Some(self.account.clone()),
            ..LoaderOverrides::without_managed_config_for_tests()
        }
    }

    async fn load(&self) -> std::io::Result<Config> {
        // Exercise the same bootstrap preflight the TUI runs before ConfigBuilder.
        let cwd = AbsolutePathBuf::from_absolute_path(&self.project)?;
        load_config_toml_with_layer_stack(
            &self.home,
            Some(&cwd),
            Vec::new(),
            ConfigLoadOptions {
                loader_overrides: self.loader_overrides(),
                ..Default::default()
            },
        )
        .await?;
        ConfigBuilder::default()
            .codex_home(self.home.clone())
            .harness_overrides(ConfigOverrides {
                cwd: Some(self.project.clone()),
                ..Default::default()
            })
            .loader_overrides(self.loader_overrides())
            .build()
            .await
    }
}

fn ignored_row(path: &std::path::Path) -> Row {
    Row {
        label: "Project security".to_string(),
        state: State::Info,
        value: format!(
            "Ignored incomplete project [security] table in {}: \
             level = \"permissive\" without version cannot override trusted security settings.",
            path.display()
        ),
        source: "configured: ignored project table",
        required: false,
    }
}

fn project_rows(config: &Config) -> Vec<Row> {
    let input = InspectorInput::from_config(
        config,
        /*thread*/ None,
        config.security_level,
        SecurityLevel::Permissive,
    );
    let saved = tests::saved(ChosenLevel::Moderate);
    let mut facts = tests::healthy(ThreadId::new());
    facts.policy = PolicyFacts::Stored {
        level: SecurityLevel::Moderate,
        kill_switch: false,
        unreadable: false,
    };
    let sections = sections(&input, &saved, &facts, tests::NOW);
    let rows = sections
        .iter()
        .filter(|section| section.title == "Level")
        .flat_map(|section| &section.rows)
        .filter(|row| row.label == "Project security")
        .cloned()
        .collect();
    let mut without_diagnostics = sections.clone();
    for section in &mut without_diagnostics {
        section.rows.retain(|row| row.label != "Project security");
    }
    assert_eq!(
        badge(&input, &saved, &facts, &sections, tests::NOW),
        badge(&input, &saved, &facts, &without_diagnostics, tests::NOW),
        "diagnostics cannot change the protection verdict"
    );
    rows
}

#[tokio::test]
async fn ignored_permissive_project_table_is_attributed_without_changing_authority() {
    let fixture = Fixture::new("[security]\nlevel = \"permissive\"\n", "trusted");
    let before = std::fs::read(fixture.home.join("config.toml")).unwrap();
    let config = fixture.load().await.unwrap();
    assert_eq!(config.security_level, SecurityLevel::Moderate);
    let expected = vec![ignored_row(&fixture.project.join(".codex/config.toml"))];
    assert!(config.startup_warnings.contains(&expected[0].value));
    assert_eq!(project_rows(&config), expected);
    assert_eq!(
        project_rows(&config),
        expected,
        "reopening cannot accumulate rows"
    );
    assert_eq!(
        std::fs::read(fixture.home.join("config.toml")).unwrap(),
        before
    );
}

#[tokio::test]
async fn valid_versioned_project_table_is_not_reported_as_ignored() {
    let fixture = Fixture::new("[security]\nversion = 1\nlevel = \"moderate\"\n", "trusted");
    let config = fixture.load().await.unwrap();
    assert_eq!(config.security_level, SecurityLevel::Moderate);
    assert_eq!(project_rows(&config), Vec::<Row>::new());
}

#[tokio::test]
async fn invalid_project_table_still_fails_with_its_original_file_error() {
    let fixture = Fixture::new("[security]\nlevel = \"aggressive\"\n", "trusted");
    // This is the existing refused case: no merged layer supplies a version.
    // A versioned user table would instead make the merged table valid.
    let user = std::fs::read_to_string(fixture.home.join("config.toml")).unwrap();
    std::fs::write(
        fixture.home.join("config.toml"),
        user.replace("[security]\nversion = 1\nlevel = \"moderate\"\n", ""),
    )
    .unwrap();
    let error = match fixture.load().await {
        Ok(_) => panic!("an invalid security table must remain refused"),
        Err(error) => error,
    };
    assert!(
        error.to_string().contains("missing field `version`"),
        "{error}"
    );
    assert!(
        error.to_string().contains(
            &fixture
                .project
                .join(".codex/config.toml")
                .display()
                .to_string()
        ),
        "{error}"
    );
}

#[tokio::test]
async fn untrusted_project_table_keeps_existing_disabled_handling() {
    let fixture = Fixture::new("[security]\nlevel = \"permissive\"\n", "untrusted");
    let config = fixture.load().await.unwrap();
    assert_eq!(config.security_level, SecurityLevel::Moderate);
    assert_eq!(project_rows(&config), Vec::<Row>::new());
}

#[tokio::test]
async fn multiple_ignored_project_tables_keep_distinct_file_attribution() {
    let mut fixture = Fixture::new("[security]\nlevel = \"permissive\"\n", "trusted");
    let outer = fixture.project.join(".codex/config.toml");
    fixture.project = fixture.project.join("nested");
    std::fs::create_dir_all(fixture.project.join(".codex")).unwrap();
    let inner = fixture.project.join(".codex/config.toml");
    std::fs::write(&inner, "[security]\nlevel = \"permissive\"\n").unwrap();
    let config = fixture.load().await.unwrap();
    assert_eq!(config.security_level, SecurityLevel::Moderate);
    let mut actual = project_rows(&config);
    let mut expected = vec![ignored_row(&outer), ignored_row(&inner)];
    for row in &expected {
        assert!(config.startup_warnings.contains(&row.value));
    }
    actual.sort_by(|a, b| a.value.cmp(&b.value));
    expected.sort_by(|a, b| a.value.cmp(&b.value));
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn ignored_parent_and_valid_inner_remain_distinct() {
    let mut fixture = Fixture::new("[security]\nlevel = \"permissive\"\n", "trusted");
    let outer = fixture.project.join(".codex/config.toml");
    fixture.project = fixture.project.join("nested");
    std::fs::create_dir_all(fixture.project.join(".codex")).unwrap();
    std::fs::write(
        fixture.project.join(".codex/config.toml"),
        "[security]\nversion = 1\nlevel = \"moderate\"\n",
    )
    .unwrap();
    let config = fixture.load().await.unwrap();
    let expected = vec![ignored_row(&outer)];
    assert!(config.startup_warnings.contains(&expected[0].value));
    assert_eq!(config.security_level, SecurityLevel::Moderate);
    assert_eq!(project_rows(&config), expected);
}

#[tokio::test]
async fn normal_projects_have_no_ignored_security_diagnostic() {
    for project_config in [Some(""), None] {
        let fixture = Fixture::new(project_config.unwrap_or_default(), "trusted");
        if project_config.is_none() {
            std::fs::remove_file(fixture.project.join(".codex/config.toml")).unwrap();
        }
        let config = fixture.load().await.unwrap();
        assert_eq!(config.security_level, SecurityLevel::Moderate);
        assert_eq!(project_rows(&config), Vec::<Row>::new());
    }
}
