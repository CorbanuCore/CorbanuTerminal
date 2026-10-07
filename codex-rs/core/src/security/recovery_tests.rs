use codex_protocol::SessionId;
use codex_protocol::ThreadId;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::RevocationEvent;
use codex_security_policy::RevocationReason;
use codex_security_policy::RevocationState;
use codex_security_policy::RevocationTarget;
use codex_security_policy::SecurityLevel;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

use super::*;
use crate::agent::AgentControl;

fn revoked(kill: bool) -> RevocationState {
    let mut revocations = RevocationState::new();
    let human = PolicyPrincipal::new(PrincipalKind::Human, "human:test").unwrap();
    revocations
        .apply(
            &RevocationEvent::new(
                human,
                if kill {
                    RevocationTarget::KillSwitch { active: true }
                } else {
                    RevocationTarget::AllActiveAuthority
                },
                RevocationReason::SecurityLevelChange,
                /*created_at_unix_seconds*/ 10,
            )
            .unwrap(),
        )
        .unwrap();
    revocations
}

fn save(home: &std::path::Path, state: DurableSecurityState) {
    HomeTransitionStore::new(home)
        .update(&mut |_| Ok(state.clone()))
        .unwrap();
}

/// (level in force, kill switch, revocation generation) of a session started
/// from `recovery`.
fn started(recovery: Recovery) -> (SecurityLevel, bool, u64) {
    let root = ThreadId::new();
    let control = AgentControl::default()
        .with_session_id(SessionId::from(root), /*max_threads*/ 1)
        .with_recovered_security_policy(recovery, root, /*inherits_from_spawn_parent*/ false)
        .unwrap();
    let snapshot = control
        .effective_security_policy()
        .snapshot_for_agent(root)
        .unwrap();
    (
        snapshot.level,
        snapshot.kill_switch_active,
        snapshot.revocation_generation,
    )
}

#[test]
fn security_recovery_without_a_file_is_the_configured_level() {
    let home = TempDir::new().unwrap();
    assert_eq!(
        recover(home.path(), SecurityLevel::Moderate),
        Recovery {
            level: SecurityLevel::Moderate,
            revocations: RevocationState::new(),
            unreadable: None,
            home: Some(home.path().to_path_buf()),
        }
    );
}

#[test]
fn security_recovery_restart_keeps_the_stricter_level_and_generation() {
    let home = TempDir::new().unwrap();
    let state = DurableSecurityState::new(SecurityLevel::Aggressive, revoked(/*kill*/ false));
    save(home.path(), state);
    // The configured level is lowered offline: the stored one still wins.
    let recovery = recover(home.path(), SecurityLevel::Permissive);
    assert_eq!(
        recovery,
        Recovery {
            level: SecurityLevel::Aggressive,
            revocations: revoked(/*kill*/ false),
            unreadable: None,
            home: Some(home.path().to_path_buf()),
        }
    );
    assert_eq!(started(recovery), (SecurityLevel::Aggressive, false, 1));
    let config = std::fs::read_to_string(home.path().join("config.toml")).unwrap();
    assert!(config.contains("level = \"aggressive\""), "{config}");
}

#[test]
fn security_recovery_kill_switch_survives_restart() {
    let home = TempDir::new().unwrap();
    save(
        home.path(),
        DurableSecurityState::new(SecurityLevel::Moderate, revoked(/*kill*/ true)),
    );
    let recovery = recover(home.path(), SecurityLevel::Moderate);
    assert_eq!(started(recovery), (SecurityLevel::Moderate, true, 1));
}

/// A downgrade writes the state file, then `config.toml`. A crash between
/// them leaves the stricter configured level for the next start.
#[test]
fn security_recovery_crash_between_downgrade_writes_stays_strict() {
    let home = TempDir::new().unwrap();
    let state = DurableSecurityState::new(SecurityLevel::Permissive, revoked(/*kill*/ false));
    std::fs::write(
        home.path().join(STATE_FILE),
        serde_json::to_vec(&state).unwrap(),
    )
    .unwrap();
    assert_eq!(
        recover(home.path(), SecurityLevel::Aggressive).level,
        SecurityLevel::Aggressive
    );
}

#[test]
fn security_recovery_tampered_state_enforces_aggressive_and_the_kill_switch() {
    let mut forged = serde_json::to_value(DurableSecurityState::new(
        SecurityLevel::Permissive,
        revoked(/*kill*/ false),
    ))
    .unwrap();
    // A generation rolled back without its events, an unknown level, junk.
    forged["revocations"]["generation"] = serde_json::json!(0);
    let mut unknown = forged.clone();
    unknown["level"] = serde_json::json!("relaxed");
    for contents in [
        serde_json::to_vec(&forged).unwrap(),
        serde_json::to_vec(&unknown).unwrap(),
        b"{".to_vec(),
    ] {
        let home = TempDir::new().unwrap();
        std::fs::write(home.path().join(STATE_FILE), contents).unwrap();
        let recovery = recover(home.path(), SecurityLevel::Permissive);
        assert_eq!(recovery.level, SecurityLevel::Aggressive);
        assert!(
            recovery
                .warning()
                .is_some_and(|warning| warning.contains("kill switch")),
            "{recovery:?}"
        );
        assert_eq!(started(recovery), (SecurityLevel::Aggressive, true, 0));
    }
}

/// A downgrade whose `config.toml` mirror cannot be written reports that the
/// next start keeps the stricter level (`config.toml` still says it), and it
/// does.
#[test]
fn security_recovery_downgrade_without_config_keeps_the_stricter_level() {
    let home = TempDir::new().unwrap();
    // `config.toml` cannot be edited when it is a folder.
    std::fs::create_dir(home.path().join("config.toml")).unwrap();
    save(
        home.path(),
        DurableSecurityState::new(SecurityLevel::Aggressive, RevocationState::new()),
    );
    let result = HomeTransitionStore::new(home.path()).update(&mut |_| {
        Ok(DurableSecurityState::new(
            SecurityLevel::Permissive,
            RevocationState::new(),
        ))
    });
    assert!(
        matches!(&result, Err(super::super::transition::TransitionError::Persist(reason)) if reason.contains("config.toml")),
        "{result:?}"
    );
    assert_eq!(
        recover(home.path(), SecurityLevel::Aggressive).level,
        SecurityLevel::Aggressive
    );
}
