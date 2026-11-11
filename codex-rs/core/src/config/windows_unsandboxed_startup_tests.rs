//! #1976 (D491): every way a protected level arrives (a user, managed or
//! project config layer, the stored level, the Aggressive recovery of an
//! unreadable store) sets `security_level`, which the refusals use, and on
//! Windows without the Windows sandbox adds the start-up notice.

use super::ConfigBuilder;
use crate::security::recovery;
use codex_config::LoaderOverrides;
use codex_security_policy::RevocationState;
use codex_security_policy::SecurityLevel;
use pretty_assertions::assert_eq;

#[derive(Default)]
struct Case {
    user: &'static str,
    managed: &'static str,
    project: &'static str,
    stored: Option<Vec<u8>>,
}

async fn load(case: Case) -> std::io::Result<(SecurityLevel, Vec<String>)> {
    let temp = tempfile::tempdir()?;
    let root = dunce::canonicalize(temp.path())?;
    let home = root.join("home");
    let project = root.join("project");
    let managed = root.join("managed");
    for dir in [
        &home,
        &project.join(".codex"),
        &managed,
        &root.join("account"),
    ] {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(project.join(".git"), "")?;
    let project_key = toml::Value::String(project.display().to_string());
    std::fs::write(
        home.join("config.toml"),
        format!(
            "{}\n[projects.{project_key}]\ntrust_level = \"trusted\"\n",
            case.user
        ),
    )?;
    std::fs::write(project.join(".codex/config.toml"), case.project)?;
    std::fs::write(managed.join("legacy.toml"), case.managed)?;
    if let Some(stored) = &case.stored {
        std::fs::write(home.join(recovery::STATE_FILE), stored)?;
    }
    let config = ConfigBuilder::default()
        .codex_home(home)
        .fallback_cwd(Some(project))
        .loader_overrides(LoaderOverrides {
            managed_config_path: Some(managed.join("legacy.toml")),
            system_config_path: Some(managed.join("config.toml")),
            system_requirements_path: Some(managed.join("requirements.toml")),
            user_home_dir: Some(root.join("account")),
            #[cfg(target_os = "macos")]
            managed_preferences_base64: Some(String::new()),
            macos_managed_config_requirements_base64: Some(String::new()),
            ..Default::default()
        })
        .build()
        .await?;
    let notices = config
        .startup_warnings
        .iter()
        .filter(|warning| warning.contains("needs the default Windows sandbox"))
        .cloned()
        .collect();
    Ok((config.security_level, notices))
}

#[tokio::test]
async fn issue_1976_every_protected_level_source_gets_the_windows_notice() -> std::io::Result<()> {
    let stored_moderate = serde_json::to_vec(&recovery::DurableSecurityState::new(
        SecurityLevel::Moderate,
        RevocationState::new(),
    ))?;
    let moderate = "[security]\nversion = 1\nlevel = \"moderate\"\n";
    let aggressive = "[security]\nversion = 1\nlevel = \"aggressive\"\n";
    let cases = [
        (
            "user",
            Case {
                user: moderate,
                ..Default::default()
            },
            SecurityLevel::Moderate,
        ),
        (
            "managed",
            Case {
                managed: aggressive,
                ..Default::default()
            },
            SecurityLevel::Aggressive,
        ),
        (
            "project",
            Case {
                project: moderate,
                ..Default::default()
            },
            SecurityLevel::Moderate,
        ),
        (
            "stored",
            Case {
                stored: Some(stored_moderate),
                ..Default::default()
            },
            SecurityLevel::Moderate,
        ),
        (
            "unreadable store",
            Case {
                stored: Some(b"not json".to_vec()),
                ..Default::default()
            },
            SecurityLevel::Aggressive,
        ),
    ];
    let mut wrong = Vec::new();
    for (source, case, expected) in cases {
        let (level, notices) = load(case).await?;
        let name = if expected == SecurityLevel::Aggressive {
            "Aggressive"
        } else {
            "Moderate"
        };
        let notified =
            notices.len() == 1 && notices[0].starts_with(&format!("Security level {name} needs"));
        if level != expected || notified != cfg!(windows) || (!cfg!(windows) && !notices.is_empty())
        {
            wrong.push(format!("{source}: {level:?} {notices:?}"));
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}

/// Permissive, and a protected level with a Windows sandbox selected, get
/// no notice.
#[tokio::test]
async fn issue_1976_no_notice_under_permissive_or_with_a_windows_sandbox() -> std::io::Result<()> {
    for user in [
        "",
        "[security]\nversion = 1\nlevel = \"moderate\"\n[windows]\nsandbox = \"elevated\"\n",
        "[security]\nversion = 1\nlevel = \"aggressive\"\n[windows]\nsandbox = \"unelevated\"\n",
    ] {
        let (_, notices) = load(Case {
            user,
            ..Default::default()
        })
        .await?;
        assert_eq!(notices, Vec::<String>::new(), "{user}");
    }
    Ok(())
}
