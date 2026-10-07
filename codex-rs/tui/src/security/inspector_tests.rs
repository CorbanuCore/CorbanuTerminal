use codex_protocol::ThreadId;
use pretty_assertions::assert_eq;

use super::*;
use crate::legacy_core::security_inspection::AgentFacts;
use crate::legacy_core::security_inspection::Denial;
use crate::legacy_core::security_inspection::GrantFacts;
use crate::legacy_core::security_inspection::TreeFacts;

pub(crate) const NOW: i64 = 1_000_000;

pub(crate) fn input(active: ChosenLevel) -> InspectorInput {
    InspectorInput {
        codex_home: PathBuf::from("/nonexistent/corbanu-home"),
        thread: None,
        configured: SecurityLevel::Permissive,
        outside_user_config: SecurityLevel::Permissive,
        active,
        boundary: (active == ChosenLevel::Aggressive)
            .then_some(Boundary::Clean { activated_at: 0 }),
        current: std::array::from_fn(|index| format!("session value {index}")),
        sandbox: "macOS Seatbelt",
        destination_policy: true,
        network_broker: false,
        mcp_servers: vec!["docs".to_string()],
        hooks: false,
    }
}

pub(crate) fn saved(level: ChosenLevel) -> Saved {
    Saved {
        level: StoredLevel::Chosen(level),
        nested: NestedAgents::Refuse,
    }
}

pub(crate) fn tree(level: SecurityLevel, agents: Vec<AgentFacts>) -> TreeFacts {
    TreeFacts {
        in_force: level,
        next_start: level,
        kill_switch: false,
        epoch: 2,
        revocation_generation: 1,
        agents,
    }
}

/// Aggressive with every runtime control observed working.
pub(crate) fn healthy(root: ThreadId) -> RuntimeFacts {
    RuntimeFacts {
        observed_at: NOW,
        policy: PolicyFacts::Live(tree(
            SecurityLevel::Aggressive,
            vec![AgentFacts {
                thread: root,
                depth: 0,
                level: SecurityLevel::Aggressive,
                stricter_than_session: false,
                stopped: false,
            }],
        )),
        grants: Vec::new(),
        taint: TaintFacts::Generation(0),
        denials: Vec::new(),
        launch_contract: ContractFacts::Armed { hardened: true },
        output_gate: ControlFacts::Enforcing,
        model_broker: ControlFacts::Off,
    }
}

fn verdict(input: &InspectorInput, saved: &Saved, facts: &RuntimeFacts, now: i64) -> Badge {
    badge(
        input,
        saved,
        facts,
        &sections(input, saved, facts, now),
        now,
    )
}

fn find<'a>(sections: &'a [Section], label: &str) -> &'a Row {
    sections
        .iter()
        .flat_map(|section| &section.rows)
        .find(|row| row.label == label)
        .unwrap_or_else(|| panic!("no row {label}"))
}

#[test]
fn pf_41_s01_green_only_when_every_required_control_is_observed() {
    let input = input(ChosenLevel::Aggressive);
    let saved = saved(ChosenLevel::Aggressive);
    let facts = healthy(ThreadId::new());
    assert_eq!(
        verdict(&input, &saved, &facts, NOW),
        Badge::Protected("Aggressive")
    );

    // Resolved configuration alone is not observation: without the live
    // policy and runtime controls the badge is never green.
    let config_only = RuntimeFacts {
        policy: PolicyFacts::Stored {
            level: SecurityLevel::Aggressive,
            kill_switch: false,
            unreadable: false,
        },
        taint: TaintFacts::NotObserved,
        launch_contract: ContractFacts::Off,
        output_gate: ControlFacts::Off,
        ..facts
    };
    assert_eq!(
        verdict(&input, &saved, &config_only, NOW),
        Badge::Partial(
            "Aggressive",
            vec![
                "Live policy".to_string(),
                "Secretless launch".to_string(),
                "Output gate".to_string(),
                "Untrusted content".to_string(),
            ]
        )
    );
}

#[test]
fn pf_41_s01_core_lagging_behind_the_launch_level_is_degraded() {
    let input = input(ChosenLevel::Aggressive);
    let saved = saved(ChosenLevel::Aggressive);
    let mut facts = healthy(ThreadId::new());
    if let PolicyFacts::Live(tree) = &mut facts.policy {
        tree.in_force = SecurityLevel::Permissive;
        tree.agents[0].level = SecurityLevel::Permissive;
    }
    let sections = sections(&input, &saved, &facts, NOW);
    assert_eq!(
        find(&sections, "Untrusted content").value,
        "labelled, but Core is Permissive: protected actions are not gated"
    );
    assert_eq!(
        badge(&input, &saved, &facts, &sections, NOW),
        Badge::Degraded(
            "Aggressive",
            vec![
                "Core enforces Permissive; Aggressive's protected-action gates are not active"
                    .to_string(),
                "Untrusted content".to_string(),
            ]
        )
    );

    facts.taint = TaintFacts::Unreadable;
    if let PolicyFacts::Live(tree) = &mut facts.policy {
        tree.in_force = SecurityLevel::Aggressive;
        tree.agents[0].level = SecurityLevel::Aggressive;
    }
    assert_eq!(
        verdict(&input, &saved, &facts, NOW),
        Badge::Degraded("Aggressive", vec!["Untrusted content".to_string()])
    );
}

#[test]
fn pf_41_s01_stale_health_and_broker_crash_are_degraded() {
    let input = input(ChosenLevel::Aggressive);
    let saved = saved(ChosenLevel::Aggressive);
    let facts = healthy(ThreadId::new());
    assert_eq!(
        verdict(&input, &saved, &facts, NOW + STALE_AFTER_SECONDS + 15),
        Badge::Degraded(
            "Aggressive",
            vec!["status observed 45s ago; press r".to_string()]
        )
    );

    // An installed broker is not observed healthy, and is not required.
    let installed = RuntimeFacts {
        model_broker: ControlFacts::Enforcing,
        ..facts.clone()
    };
    let installed_sections = sections(&input, &saved, &installed, NOW);
    assert_eq!(
        find(&installed_sections, "Model key broker").state,
        State::Unobserved
    );
    assert_eq!(
        badge(&input, &saved, &installed, &installed_sections, NOW),
        Badge::Protected("Aggressive")
    );

    let crashed = RuntimeFacts {
        model_broker: ControlFacts::Degraded("no broker is running; provider keys are not sent"),
        launch_contract: ContractFacts::Armed { hardened: false },
        ..facts
    };
    let sections = sections(&input, &saved, &crashed, NOW);
    assert_eq!(find(&sections, "Model key broker").state, State::Degraded);
    assert_eq!(
        badge(&input, &saved, &crashed, &sections, NOW),
        Badge::Degraded(
            "Aggressive",
            vec![
                "Secretless launch".to_string(),
                "Model key broker".to_string()
            ]
        )
    );
}

#[test]
fn pf_41_s01_unsupported_platform_and_unclean_boundary_are_degraded() {
    let input = InspectorInput {
        sandbox: "none",
        boundary: Some(Boundary::NotClean {
            blockers: vec!["Shell profile exports OPENAI_API_KEY".to_string()],
        }),
        ..input(ChosenLevel::Aggressive)
    };
    let saved = saved(ChosenLevel::Aggressive);
    let facts = healthy(ThreadId::new());
    let sections = sections(&input, &saved, &facts, NOW);
    assert_eq!(
        find(&sections, "Sandbox backend").value,
        "none on this platform: agent commands are not contained"
    );
    // Only the summary: blocker details (names, paths) stay in the review.
    assert_eq!(
        find(&sections, "Protected boundary").value,
        "protected boundary not clean: 1 blocker; details in the Aggressive review"
    );
    assert_eq!(
        badge(&input, &saved, &facts, &sections, NOW),
        Badge::Degraded(
            "Aggressive",
            vec![
                "Sandbox backend".to_string(),
                "Protected boundary".to_string()
            ]
        )
    );
}

#[test]
fn pf_41_s01_kill_switch_and_unreadable_state_block() {
    let input = input(ChosenLevel::Aggressive);
    let saved = saved(ChosenLevel::Aggressive);
    let mut facts = healthy(ThreadId::new());
    if let PolicyFacts::Live(tree) = &mut facts.policy {
        tree.kill_switch = true;
    }
    assert_eq!(
        verdict(&input, &saved, &facts, NOW),
        Badge::Blocked("kill switch on: every protected action is denied".to_string())
    );
    if let PolicyFacts::Live(tree) = &mut facts.policy {
        tree.kill_switch = false;
        tree.agents[0].stopped = true;
    }
    assert!(matches!(
        verdict(&input, &saved, &facts, NOW),
        Badge::Blocked(reason) if reason.starts_with("this session is stopped")
    ));
    facts.policy = PolicyFacts::Stored {
        level: SecurityLevel::Aggressive,
        kill_switch: true,
        unreadable: true,
    };
    assert_eq!(
        verdict(&input, &saved, &facts, NOW),
        Badge::Blocked(
            "security state unreadable: Aggressive and the kill switch apply".to_string()
        )
    );
}

#[test]
fn pf_41_s01_conflicting_config_and_saved_level_are_shown_not_hidden() {
    // Saved Permissive, but a project layer sets Aggressive for Core.
    let input = InspectorInput {
        configured: SecurityLevel::Aggressive,
        outside_user_config: SecurityLevel::Aggressive,
        ..input(ChosenLevel::Permissive)
    };
    let saved = saved(ChosenLevel::Permissive);
    let facts = healthy(ThreadId::new());
    let sections = sections(&input, &saved, &facts, NOW);
    assert_eq!(
        (
            find(&sections, "Saved").value.as_str(),
            find(&sections, "Core config").value.as_str()
        ),
        (
            "Permissive",
            "Aggressive; a project, profile, -c or managed layer sets Aggressive"
        )
    );
    // Core enforces Aggressive, so the badge follows Core, not the picker,
    // but this session's own values were not checked against it.
    assert_eq!(
        badge(&input, &saved, &facts, &sections, NOW),
        Badge::Partial(
            "Aggressive",
            vec!["session controls not checked at launch".to_string()]
        )
    );

    // Core already Aggressive, Aggressive saved, but this run started
    // Permissive: the session's own controls are not active yet.
    let started_permissive = self::input(ChosenLevel::Permissive);
    assert_eq!(
        verdict(
            &started_permissive,
            &self::saved(ChosenLevel::Aggressive),
            &healthy(ThreadId::new()),
            NOW
        ),
        Badge::Degraded(
            "Aggressive",
            vec!["Aggressive's session controls apply after restart".to_string()]
        )
    );

    // Aggressive saved but this run is Permissive: never shown as protected.
    let permissive = self::input(ChosenLevel::Permissive);
    let facts = RuntimeFacts {
        policy: PolicyFacts::Live(tree(SecurityLevel::Permissive, Vec::new())),
        ..healthy(ThreadId::new())
    };
    assert_eq!(
        verdict(
            &permissive,
            &self::saved(ChosenLevel::Aggressive),
            &facts,
            NOW
        ),
        Badge::Degraded(
            "Permissive",
            vec!["Aggressive is saved but not active until restart".to_string()]
        )
    );
    assert_eq!(
        verdict(
            &permissive,
            &self::saved(ChosenLevel::Permissive),
            &facts,
            NOW
        ),
        Badge::Permissive
    );
}

#[test]
fn pf_41_s01_children_grants_taint_and_denials_are_correlated() {
    let root = ThreadId::new();
    let child = ThreadId::new();
    let child_id = child.to_string();
    let child_name = format!("agent {}", &child_id[child_id.len() - 8..]);
    let mut facts = healthy(root);
    if let PolicyFacts::Live(tree) = &mut facts.policy {
        tree.in_force = SecurityLevel::Moderate;
        tree.agents[0].level = SecurityLevel::Moderate;
        tree.agents.push(AgentFacts {
            thread: child,
            depth: 1,
            level: SecurityLevel::Aggressive,
            stricter_than_session: true,
            stopped: false,
        });
    }
    facts.grants = vec![GrantFacts {
        thread: root,
        surface: "one command without the protected-path rules",
        expires_at: NOW + 240,
        used: 1,
        limit: Some(2),
    }];
    facts.taint = TaintFacts::Generation(3);
    facts.denials = vec![Denial {
        at: NOW - 5,
        thread: Some(child),
        reason: "vault access",
        outcome: "refused: kill switch on",
    }];
    let input = input(ChosenLevel::Aggressive);
    let sections = sections(&input, &saved(ChosenLevel::Aggressive), &facts, NOW);
    let values = |title: &str| {
        sections
            .iter()
            .find(|section| section.title == title)
            .map(|section| {
                section
                    .rows
                    .iter()
                    .map(|row| format!("{}: {}", row.label, row.value))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };
    assert_eq!(
        values("Grants"),
        vec![
            "main session: one command without the protected-path rules; expires in 4m; used 1 of 2"
                .to_string()
        ]
    );
    assert_eq!(
        values("Agents")[..2],
        [
            "main session: Moderate".to_string(),
            format!(
                "{child_name}: Aggressive, stricter than the session (inherited or configured)"
            ),
        ]
    );
    assert_eq!(
        values("Recent denials"),
        vec![format!(
            "5s ago: vault access: refused: kill switch on ({child_name})"
        )]
    );
    assert!(
        find(&sections, "Untrusted content")
            .value
            .starts_with("labelled; session tainted (3 batches)")
    );
}

#[test]
fn pf_41_s01_unsandboxed_routes_are_not_contained_and_p1_controls_not_available() {
    let input = InspectorInput {
        hooks: true,
        ..input(ChosenLevel::Aggressive)
    };
    let sections = sections(
        &input,
        &saved(ChosenLevel::Aggressive),
        &healthy(ThreadId::new()),
        NOW,
    );
    let states = |title: &str| {
        sections
            .iter()
            .find(|section| section.title == title)
            .map(|section| {
                section
                    .rows
                    .iter()
                    .map(|row| (row.label.clone(), row.state))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };
    assert_eq!(
        states("Not contained"),
        [
            "MCP servers",
            "Hooks",
            "`!` shell commands",
            "App-server command/exec"
        ]
        .map(|label| (label.to_string(), State::NotContained))
    );
    assert_eq!(
        states("Not available"),
        [
            "Screened search routing",
            "Brokered browser login",
            "Quarantine",
            "Agent Sweep"
        ]
        .map(|label| (label.to_string(), State::NotAvailable))
    );
    assert_eq!(
        find(&sections, "`!` shell commands").value,
        "run outside the OS sandbox with the environment allowlist; allowed because you configured or started it"
    );
}
