use pretty_assertions::assert_eq;

use super::*;
use crate::security::level::ChosenLevel;
use crate::security::level::LevelContext;
use crate::security::level::NestedAgents;

fn record(home: &std::path::Path, level: &str) {
    std::fs::write(
        home.join("security_state.json"),
        format!(
            r#"{{"version":1,"level":"{level}","revocations":{{"schema_version":1,"generation":0,"kill_switch_active":false}}}}"#
        ),
    )
    .unwrap();
}

/// #750 review r2: `/status` reads Core's record and configured level as
/// they are now: Moderate that Core keeps counts, and Moderate lowered to
/// Permissive in `/security` is not shown as continuing.
#[test]
fn issue_750_status_follows_cores_record_and_configured_level() {
    let home = tempfile::tempdir().unwrap();
    let context = LevelContext {
        codex_home: home.path().to_path_buf(),
        picker_enabled: true,
        active: ChosenLevel::Moderate,
        preflight_enabled: false,
        boundary: None,
    };
    // Started under Moderate saved with Core's record.
    crate::security::level::save(home.path(), ChosenLevel::Moderate, NestedAgents::Refuse).unwrap();
    record(home.path(), "moderate");
    assert_eq!(
        level_summary(&context, SecurityLevel::Permissive, /*thread*/ None),
        "Moderate active (/security)"
    );
    // Lowered to Permissive in /security: both records say Permissive.
    crate::security::level::save(home.path(), ChosenLevel::Permissive, NestedAgents::Refuse)
        .unwrap();
    record(home.path(), "permissive");
    assert_eq!(
        level_summary(&context, SecurityLevel::Permissive, /*thread*/ None),
        "Moderate active; Permissive saved for next start (/security)"
    );
    // `[security] level = "moderate"` in config, nothing saved.
    let config_only = tempfile::tempdir().unwrap();
    let context = LevelContext {
        codex_home: config_only.path().to_path_buf(),
        ..context
    };
    assert_eq!(
        level_summary(&context, SecurityLevel::Moderate, /*thread*/ None),
        "Moderate active (/security)"
    );
}
