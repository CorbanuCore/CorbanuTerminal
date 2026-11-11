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
        project_security_warnings: Vec::new(),
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
        // #391: Aggressive turns the model key broker on.
        model_broker: ControlFacts::Enforcing,
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

/// #391: Aggressive turns the model key broker on where it runs, so a broker
/// that config turned off leaves Aggressive partial; Permissive needs none.
#[test]
fn sec_391_model_key_broker_off_under_aggressive_is_partial() {
    let facts = RuntimeFacts {
        model_broker: ControlFacts::Off,
        ..healthy(ThreadId::new())
    };
    let input = input(ChosenLevel::Aggressive);
    let saved = saved(ChosenLevel::Aggressive);
    let off = sections(&input, &saved, &facts, NOW);
    let broker = find(&off, "Model key broker");
    assert_eq!(
        (broker.state, broker.value.as_str()),
        (
            State::Off,
            "off: Core reads provider keys itself (broker_model_auth is off in config, or this process started before the level was chosen)"
        )
    );
    let expected = if cfg!(any(target_os = "macos", target_os = "linux", windows)) {
        Badge::Partial("Aggressive", vec!["Model key broker".to_string()])
    } else {
        Badge::Protected("Aggressive")
    };
    assert_eq!(badge(&input, &saved, &facts, &off, NOW), expected);

    // A broker that did not start is degraded.
    let failed = RuntimeFacts {
        model_broker: ControlFacts::Degraded(
            "the broker did not start; model requests are refused, never sent directly",
        ),
        ..facts
    };
    let degraded = sections(&input, &saved, &failed, NOW);
    assert_eq!(find(&degraded, "Model key broker").state, State::Degraded);
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
fn pf_41_s01_stale_health_is_not_green_and_broker_crash_is_degraded() {
    let input = input(ChosenLevel::Aggressive);
    let saved = saved(ChosenLevel::Aggressive);
    let facts = healthy(ThreadId::new());
    // #1036: stale, not degraded.
    assert_eq!(
        verdict(&input, &saved, &facts, NOW + STALE_AFTER_SECONDS + 15),
        Badge::Stale("Aggressive", "45s ago".to_string())
    );

    // An installed broker is not observed healthy, and not counted as missing.
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
            vec!["the rest of Aggressive's session controls apply after restart".to_string()]
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
        surface: "one command may write hooks and start-up files",
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
            "main session: one command may write hooks and start-up files; expires in 4m; used 1 of 2"
                .to_string(),
            // #442: what a grant never lifts.
            "Protected reads: never granted under Aggressive, not even once: credential files (.env, .env.* and .envrc files, also ones created later; not .env.example, .env.sample or .env.template; SSH, cloud and CLI tokens) and Corbanu's sign-in, vault, config and history stay unreadable to agent commands; switch to Permissive in /security to allow them. On Linux and Windows a command is stopped when such a file appears while it runs; a process already waiting for that file can read it in the moment before the stop"
                .to_string(),
            // #442 (Travis, 2026-10-12): what Grant once lifts, and what not.
            format!("Grant once: {}", crate::security::aggressive::GRANT_ONCE_LIFTS),
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
            "Financial preview and approvals",
            "Isolated retrieval",
            "Screened search routing",
            "Brokered browser login",
            "Quarantine",
            "Untrusted-content detectors",
            "Agent Sweep"
        ]
        .map(|label| (label.to_string(), State::NotAvailable))
    );
    assert_eq!(
        find(&sections, "`!` shell commands").value,
        "run outside the OS sandbox with the environment allowlist; allowed because you configured or started it"
    );
}

/// #437 (AGG-24): beside unobserved brokers the header claimed "every row
/// verified at launch". The launch check covers the session rows only.
#[test]
fn sec_437_active_row_claims_only_the_launch_check() {
    let facts = RuntimeFacts {
        model_broker: ControlFacts::Off,
        ..healthy(ThreadId::new())
    };
    let input = input(ChosenLevel::Aggressive);
    let rows = sections(&input, &saved(ChosenLevel::Aggressive), &facts, NOW);
    let active = find(&rows, "Active this run");
    assert!(!active.value.contains("every row"), "{}", active.value);
    assert!(
        active
            .value
            .contains("rows marked \"checked at launch\" passed the launch check"),
        "{}",
        active.value
    );
    assert_eq!(find(&rows, "Model key broker").state, State::Off);
}

/// #749: under Moderate the rows say what is enforced: the level is
/// Moderate (not "Permissive" beside "Moderate in force"), the nested-agents
/// row describes Moderate, a broker that is off counts (D2), and a saved
/// Moderate that waits for the restart is never shown as Permissive.
#[test]
fn issue_749_moderate_rows_match_what_is_enforced() {
    let root = ThreadId::new();
    let moderate = RuntimeFacts {
        policy: PolicyFacts::Live(tree(
            SecurityLevel::Moderate,
            vec![AgentFacts {
                thread: root,
                depth: 0,
                level: SecurityLevel::Moderate,
                stricter_than_session: false,
                stopped: false,
            }],
        )),
        ..healthy(root)
    };
    let input = input(ChosenLevel::Moderate);
    let saved = saved(ChosenLevel::Moderate);
    let rows = sections(&input, &saved, &moderate, NOW);
    let active = find(&rows, "Active this run");
    assert!(active.value.starts_with("Moderate"), "{}", active.value);
    assert!(
        find(&rows, "Live policy")
            .value
            .starts_with("Moderate in force"),
        "{}",
        find(&rows, "Live policy").value
    );
    let nested = find(&rows, "Nested agents");
    assert!(
        !nested.value.contains("Aggressive is enforced"),
        "{}",
        nested.value
    );
    assert_eq!(nested.value, super::super::moderate::NESTED);
    assert_eq!(
        verdict(&input, &saved, &moderate, NOW),
        Badge::Protected("Moderate")
    );
    let broker_off = RuntimeFacts {
        model_broker: ControlFacts::Off,
        ..moderate.clone()
    };
    assert_eq!(
        verdict(&input, &saved, &broker_off, NOW),
        Badge::Partial("Moderate", vec!["Model key broker".to_string()])
    );
    // #1036: an old observation of a healthy Moderate is stale, not
    // "Moderate degraded"; one of a partial Moderate too.
    assert_eq!(
        verdict(&input, &saved, &moderate, NOW + 33),
        Badge::Stale("Moderate", "33s ago".to_string())
    );
    assert_eq!(
        verdict(&input, &saved, &broker_off, NOW + 101),
        Badge::Stale("Moderate", "1m ago".to_string())
    );

    // Saved, Core raised now (D4), launched Permissive: the broker waits.
    let started_permissive = self::input(ChosenLevel::Permissive);
    assert_eq!(
        verdict(&started_permissive, &saved, &moderate, NOW),
        Badge::Partial(
            "Moderate",
            vec!["Moderate's model-key broker applies after restart".to_string()]
        )
    );
    // Saved, not applied yet: not shown as Permissive.
    let permissive = RuntimeFacts {
        policy: PolicyFacts::Live(tree(SecurityLevel::Permissive, Vec::new())),
        ..healthy(root)
    };
    assert_eq!(
        verdict(&started_permissive, &saved, &permissive, NOW),
        Badge::Degraded(
            "Permissive",
            vec!["Moderate is saved but not active until restart".to_string()]
        )
    );
}

/// #839 (MOD-20, MOD-24): under Moderate the top line follows what Moderate
/// enforces. The network proxy's credential broker, which Moderate does not
/// turn on, no longer makes a healthy Moderate "off or not observed" next to
/// a Model keys row saying the broker holds the keys; a model request the
/// model-key broker refused makes it degraded. Aggressive is unchanged.
#[test]
fn issue_839_moderate_top_line_matches_the_model_key_broker() {
    let root = ThreadId::new();
    let moderate = RuntimeFacts {
        policy: PolicyFacts::Live(tree(
            SecurityLevel::Moderate,
            vec![AgentFacts {
                thread: root,
                depth: 0,
                level: SecurityLevel::Moderate,
                stricter_than_session: false,
                stopped: false,
            }],
        )),
        ..healthy(root)
    };
    let mut input = input(ChosenLevel::Moderate);
    input.network_broker = true;
    let saved = saved(ChosenLevel::Moderate);
    let rows = sections(&input, &saved, &moderate, NOW);
    let network = find(&rows, "Network credential broker");
    assert!(!network.required, "{network:?}");
    assert!(
        network.value.contains("agent commands' requests"),
        "{}",
        network.value
    );
    assert_eq!(
        verdict(&input, &saved, &moderate, NOW),
        Badge::Protected("Moderate")
    );
    let refused = RuntimeFacts {
        model_broker: ControlFacts::Degraded(
            "the last model request of this session or one of its agents was refused: its provider URL cannot be brokered",
        ),
        ..moderate
    };
    assert_eq!(
        verdict(&input, &saved, &refused, NOW),
        Badge::Degraded("Moderate", vec!["Model key broker".to_string()])
    );

    // Aggressive still needs it observed when it is on.
    let mut aggressive = self::input(ChosenLevel::Aggressive);
    aggressive.network_broker = true;
    assert_eq!(
        verdict(
            &aggressive,
            &self::saved(ChosenLevel::Aggressive),
            &healthy(root),
            NOW
        ),
        Badge::Partial("Aggressive", vec!["Network credential broker".to_string()])
    );
}

/// #846 (MOD-29): launched under Moderate, Aggressive confirmed and Core
/// already Aggressive, before the restart: the project's `.env` files are
/// not denied yet (Aggressive's sandbox starts at the restart), and the
/// "Protected reads" row says so instead of claiming they are unreadable.
#[test]
fn issue_846_protected_reads_before_the_restart_leave_out_project_env_files() {
    let root = ThreadId::new();
    let facts = healthy(root);
    let input = input(ChosenLevel::Moderate);
    let before_restart = sections(&input, &saved(ChosenLevel::Aggressive), &facts, NOW);
    let reads = find(&before_restart, "Protected reads");
    assert!(
        reads
            .value
            .contains("credential files in your home folder (SSH, cloud and CLI tokens)"),
        "{}",
        reads.value
    );
    assert!(
        reads.value.ends_with(
            "The project's credential files (.env, .env.* and .envrc files) are denied to agent commands only after you restart Corbanu Terminal"
        ),
        "{}",
        reads.value
    );
    assert!(
        !reads.value.contains("also ones created later"),
        "{}",
        reads.value
    );
    // The next start is not Aggressive (another level saved since, or Core
    // Aggressive from config only): no protection is promised after it.
    let not_launched = sections(&input, &saved(ChosenLevel::Permissive), &facts, NOW);
    let reads = find(&not_launched, "Protected reads");
    assert!(
        reads.value.ends_with(
            "The project's credential files (.env, .env.* and .envrc files) are not denied in this session: Aggressive's sandbox is not on"
        ),
        "{}",
        reads.value
    );
    // Launched under Aggressive: as before.
    let launched = sections(
        &super::tests::input(ChosenLevel::Aggressive),
        &saved(ChosenLevel::Aggressive),
        &facts,
        NOW,
    );
    let reads = find(&launched, "Protected reads");
    assert!(
        reads.value.starts_with(
            "never granted under Aggressive, not even once: credential files (.env, .env.* and .envrc files, also ones created later;"
        ),
        "{}",
        reads.value
    );
}
