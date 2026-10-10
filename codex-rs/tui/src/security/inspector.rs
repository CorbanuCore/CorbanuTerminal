//! PF-41-S01: what the effective-security inspector shows.
//!
//! Three kinds of fact are kept apart: what is *configured* (the saved level
//! and config layers), what was *resolved* for this session (its config,
//! verified at launch), and what is *observed* now in this process (Core's
//! live policy, runtime controls, grants, taint and denials). Configuration
//! alone never produces the green "Protected" badge, and a fact older than
//! [`STALE_AFTER_SECONDS`] cannot either.
//!
//! The inspector only reads. Nothing here saves a level, issues or revokes a
//! grant, or clears a stop.

use std::path::PathBuf;

use codex_features::Feature;
use codex_protocol::ThreadId;

use super::level::ChosenLevel;
use super::level::NestedAgents;
use super::level::StoredLevel;
use super::preflight::Boundary;
use crate::legacy_core::config::Config;
use crate::legacy_core::security_inspection::ContractFacts;
use crate::legacy_core::security_inspection::ControlFacts;
use crate::legacy_core::security_inspection::PolicyFacts;
use crate::legacy_core::security_inspection::RuntimeFacts;
use crate::legacy_core::security_inspection::SecurityLevel;
use crate::legacy_core::security_inspection::TaintFacts;

/// Observations older than this are shown as stale and cannot be green.
pub(crate) const STALE_AFTER_SECONDS: i64 = 30;
/// Recent denials shown.
const MAX_DENIALS_SHOWN: usize = 8;

/// Controls that belong to the P1 hardening plan (decision 3, 2026-10-06).
const NOT_AVAILABLE: [(&str, &str); 4] = [
    ("Screened search routing", "PF-32"),
    ("Brokered browser login", "PF-37"),
    ("Quarantine", "PF-34"),
    ("Agent Sweep", "PF-40"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum State {
    /// Observed working in this process now.
    Enforcing,
    /// Taken from configuration (checked at launch where noted), not probed.
    Resolved,
    /// Turned on but not observed.
    Unobserved,
    Off,
    /// Turned on but cannot protect.
    Degraded,
    /// Allowed, but runs outside the OS sandbox.
    NotContained,
    /// Not built in this build.
    NotAvailable,
    Info,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Row {
    pub(crate) label: String,
    pub(crate) state: State,
    pub(crate) value: String,
    pub(crate) source: &'static str,
    /// Whether the row decides the badge under a protected level.
    pub(crate) required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Section {
    pub(crate) title: &'static str,
    pub(crate) rows: Vec<Row>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Badge {
    Permissive,
    /// Every required control was observed working.
    Protected(&'static str),
    /// The level is enforced; the named controls are off or unobserved.
    Partial(&'static str, Vec<String>),
    /// Protection cannot be claimed for the named reasons.
    Degraded(&'static str, Vec<String>),
    /// Everything is denied until a person acts.
    Blocked(String),
}

/// Session facts captured when `/security` opens; runtime facts are read
/// again on every refresh.
#[derive(Clone, Debug)]
pub(crate) struct InspectorInput {
    pub(crate) codex_home: PathBuf,
    pub(crate) thread: Option<ThreadId>,
    pub(crate) configured: SecurityLevel,
    pub(crate) outside_user_config: SecurityLevel,
    pub(crate) active: ChosenLevel,
    pub(crate) boundary: Option<Boundary>,
    pub(crate) current: super::current::CurrentValues,
    pub(crate) sandbox: &'static str,
    pub(crate) destination_policy: bool,
    pub(crate) network_broker: bool,
    pub(crate) mcp_servers: Vec<String>,
    pub(crate) hooks: bool,
}

impl InspectorInput {
    pub(crate) fn from_config(
        config: &Config,
        thread: Option<ThreadId>,
        configured: SecurityLevel,
        outside_user_config: SecurityLevel,
    ) -> Self {
        let context = super::level::context();
        let mut mcp_servers = config
            .mcp_servers
            .get()
            .iter()
            .filter(|(_, server)| server.enabled)
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        mcp_servers.sort();
        Self {
            codex_home: config.codex_home.to_path_buf(),
            thread,
            configured,
            outside_user_config,
            active: context.map_or(ChosenLevel::Permissive, |context| context.active),
            boundary: context.and_then(|context| context.boundary.clone()),
            current: super::current::current_values(config),
            sandbox: sandbox_name(config),
            destination_policy: config.features.enabled(Feature::UrlDestinationPolicy),
            network_broker: config.features.enabled(Feature::IsolatedCredentialBroker),
            mcp_servers,
            hooks: config.features.enabled(Feature::CodexHooks),
        }
    }

    pub(crate) fn observe(&self, now: i64) -> RuntimeFacts {
        crate::legacy_core::security_inspection::observe(
            &self.codex_home,
            self.configured,
            self.thread,
            now,
        )
    }
}

#[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
fn sandbox_name(config: &Config) -> &'static str {
    #[cfg(target_os = "windows")]
    let windows = crate::windows_sandbox::level_from_config(config)
        != codex_protocol::config_types::WindowsSandboxLevel::Disabled;
    // Only Windows has a sandbox that configuration can turn off.
    #[cfg(not(target_os = "windows"))]
    let windows = false;
    match codex_sandboxing::get_platform_sandbox(windows) {
        Some(codex_sandboxing::SandboxType::MacosSeatbelt) => "macOS Seatbelt",
        Some(codex_sandboxing::SandboxType::LinuxSeccomp) => "Linux bubblewrap and seccomp",
        Some(codex_sandboxing::SandboxType::WindowsRestrictedToken) => "Windows restricted token",
        Some(codex_sandboxing::SandboxType::None) | None => "none",
    }
}

/// The saved `/security` state, read again on each refresh.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Saved {
    pub(crate) level: StoredLevel,
    pub(crate) nested: NestedAgents,
}

impl Saved {
    pub(crate) fn load(input: &InspectorInput) -> Self {
        let (level, nested) = super::level::load_state(&input.codex_home);
        Self { level, nested }
    }
}

fn level_name(level: SecurityLevel) -> &'static str {
    match level {
        SecurityLevel::Permissive => "Permissive",
        SecurityLevel::Moderate => "Moderate",
        SecurityLevel::Aggressive => "Aggressive",
    }
}

fn on_off(on: bool) -> &'static str {
    if on { "on" } else { "off" }
}

fn row(label: &str, state: State, value: impl Into<String>, source: &'static str) -> Row {
    Row {
        label: label.to_string(),
        state,
        value: value.into(),
        source,
        required: false,
    }
}

fn required(mut row: Row) -> Row {
    row.required = true;
    row
}

fn ago(seconds: i64) -> String {
    match seconds.max(0) {
        seconds @ 0..60 => format!("{seconds}s ago"),
        seconds @ 60..3600 => format!("{}m ago", seconds / 60),
        seconds => format!("{}h ago", seconds / 3600),
    }
}

fn agent_name(thread: Option<ThreadId>, root: Option<ThreadId>) -> String {
    match thread {
        Some(thread) if Some(thread) == root => "main session".to_string(),
        Some(thread) => {
            let id = thread.to_string();
            format!("agent {}", &id[id.len().saturating_sub(8)..])
        }
        None => "this process".to_string(),
    }
}

/// The inspector's sections, top to bottom.
pub(crate) fn sections(
    input: &InspectorInput,
    saved: &Saved,
    facts: &RuntimeFacts,
    now: i64,
) -> Vec<Section> {
    let protected = in_force(input, facts) != SecurityLevel::Permissive;
    let launch_checked = input.active == ChosenLevel::Aggressive;
    let tree = match &facts.policy {
        PolicyFacts::Live(tree) => Some(tree),
        PolicyFacts::Stored { .. } | PolicyFacts::Unreadable => None,
    };
    let root = tree.and_then(|tree| tree.agents.first().map(|agent| agent.thread));

    let saved_value = match &saved.level {
        StoredLevel::Absent => "Permissive (nothing saved)".to_string(),
        StoredLevel::Chosen(level) => level.name().to_string(),
        StoredLevel::Invalid(_) => "unreadable: Aggressive is enforced".to_string(),
    };
    let mut configured = level_name(input.configured).to_string();
    if input.outside_user_config > SecurityLevel::Permissive {
        configured.push_str(&format!(
            "; a project, profile, -c or managed layer sets {}",
            level_name(input.outside_user_config)
        ));
    }
    let live = match &facts.policy {
        PolicyFacts::Live(tree) => {
            let stopped = tree.agents.first().is_some_and(|root| root.stopped);
            row(
                "Live policy",
                if tree.kill_switch || stopped {
                    State::Degraded
                } else {
                    State::Enforcing
                },
                format!(
                    "{} in force; next start {}; kill switch {}{}; generation {}.{}",
                    level_name(tree.in_force),
                    level_name(tree.next_start),
                    on_off(tree.kill_switch),
                    if stopped {
                        "; this session is stopped"
                    } else {
                        ""
                    },
                    tree.epoch,
                    tree.revocation_generation
                ),
                "observed: Core policy",
            )
        }
        PolicyFacts::Stored {
            level,
            kill_switch,
            unreadable,
        } => row(
            "Live policy",
            if *unreadable || *kill_switch {
                State::Degraded
            } else {
                State::Unobserved
            },
            if *unreadable {
                "security state unreadable: Aggressive and the kill switch apply until a level is confirmed in /security".to_string()
            } else {
                format!(
                    "not running for this session here (not started yet, or a remote app server); next start {}{}",
                    level_name(*level),
                    if *kill_switch { "; kill switch on" } else { "" }
                )
            },
            "stored state",
        ),
        PolicyFacts::Unreadable => row(
            "Live policy",
            State::Degraded,
            "cannot be read; actions are denied",
            "observed: Core policy",
        ),
    };
    let level = Section {
        title: "Level",
        rows: vec![
            row("Saved", State::Info, saved_value, "configured: /security"),
            row(
                "Core config",
                State::Info,
                configured,
                "configured: config layers",
            ),
            row(
                "Active this run",
                State::Resolved,
                if launch_checked {
                    "Aggressive, every row verified at launch"
                } else {
                    "Permissive"
                },
                "resolved: launch check",
            ),
            required(live),
        ],
    };

    let mut session = input
        .current
        .iter()
        .zip(super::aggressive::ROWS)
        .map(|(value, (label, _))| {
            row(
                label,
                State::Resolved,
                value.clone(),
                // The launch check leaves the model key broker to config.
                if launch_checked && label != super::aggressive::MODEL_KEYS_LABEL {
                    "resolved: session config, checked at launch"
                } else {
                    "resolved: session config"
                },
            )
        })
        .collect::<Vec<_>>();
    session.push(required(row(
        "Sandbox backend",
        if input.sandbox == "none" {
            State::Degraded
        } else {
            State::Resolved
        },
        if input.sandbox == "none" {
            "none on this platform: agent commands are not contained"
        } else {
            input.sandbox
        },
        "resolved: platform",
    )));
    session.push(row(
        "Egress destinations",
        if input.destination_policy {
            State::Resolved
        } else {
            State::Off
        },
        if input.destination_policy {
            "proxy destination policy on (url_destination_policy)"
        } else {
            "no destination policy (url_destination_policy off)"
        },
        "resolved: features",
    ));

    let runtime = Section {
        title: "Runtime controls",
        rows: runtime_rows(input, facts),
    };

    let grants = Section {
        title: "Grants",
        rows: match tree {
            None => vec![row(
                "Held",
                State::Info,
                "none: no live policy for this session",
                "observed: Core policy",
            )],
            Some(_) if facts.grants.is_empty() => vec![row(
                "Held",
                State::Info,
                "none. Grants are never saved: a restart, level change, revocation or the kill switch ends them",
                "observed: Core policy",
            )],
            Some(_) => facts
                .grants
                .iter()
                .map(|grant| {
                    row(
                        &agent_name(Some(grant.thread), root),
                        State::Enforcing,
                        format!(
                            "{}; expires in {}; used {}{}",
                            grant.surface,
                            ago(grant.expires_at - now).trim_end_matches(" ago"),
                            grant.used,
                            grant
                                .limit
                                .map(|limit| format!(" of {limit}"))
                                .unwrap_or_default()
                        ),
                        "observed: Core policy",
                    )
                })
                .collect(),
        },
    };

    let mut agents = tree.map_or_else(Vec::new, |tree| {
        tree.agents
            .iter()
            .map(|agent| {
                let mut value = level_name(agent.level).to_string();
                if agent.stricter_than_session {
                    value.push_str(", stricter than the session (inherited or configured)");
                }
                if agent.stopped {
                    value.push_str("; stopped: everything is denied");
                }
                row(
                    &agent_name(Some(agent.thread), root),
                    if agent.stopped {
                        State::Degraded
                    } else {
                        State::Enforcing
                    },
                    value,
                    "observed: Core policy",
                )
            })
            .collect()
    });
    agents.push(row(
        "Nested agents",
        State::Resolved,
        match saved.nested {
            NestedAgents::Refuse => {
                "refuse (default): agent commands that start `corbanu exec` or `review` are refused while Aggressive is enforced"
            }
            NestedAgents::Pass => {
                "pass: `corbanu exec` and `review` started by agent commands run with Aggressive enforced"
            }
        },
        "configured: /security",
    ));

    let allowlist = if matches!(facts.launch_contract, ContractFacts::Armed { .. }) {
        "with the environment allowlist"
    } else {
        "without the environment allowlist (secretless_agent_launch off)"
    };
    let outside = |what: &str| {
        format!(
            "{what} outside the OS sandbox {allowlist}; allowed because you configured or started it"
        )
    };
    let mcp = if input.mcp_servers.is_empty() {
        row(
            "MCP servers",
            State::Off,
            "none configured",
            "resolved: config",
        )
    } else {
        row(
            "MCP servers",
            State::NotContained,
            outside(&format!("{} run", input.mcp_servers.join(", "))),
            "resolved: config",
        )
    };
    let not_contained = Section {
        title: "Not contained",
        rows: vec![
            mcp,
            if input.hooks {
                row(
                    "Hooks",
                    State::NotContained,
                    outside("run"),
                    "resolved: features",
                )
            } else {
                row(
                    "Hooks",
                    State::Off,
                    "off (codex_hooks)",
                    "resolved: features",
                )
            },
            row(
                "`!` shell commands",
                State::NotContained,
                outside("run"),
                "fixed",
            ),
            row(
                "App-server command/exec",
                State::NotContained,
                outside("runs"),
                "fixed",
            ),
        ],
    };

    let not_available = Section {
        title: "Not available",
        rows: NOT_AVAILABLE
            .iter()
            .map(|(label, sprint)| {
                row(
                    label,
                    State::NotAvailable,
                    format!("not available in this build ({sprint})"),
                    "fixed",
                )
            })
            .collect(),
    };

    let denials = Section {
        title: "Recent denials",
        rows: if facts.denials.is_empty() {
            vec![row(
                "None",
                State::Info,
                "nothing refused in this process",
                "observed: this process",
            )]
        } else {
            facts
                .denials
                .iter()
                .take(MAX_DENIALS_SHOWN)
                .map(|denial| {
                    row(
                        &ago(now - denial.at),
                        State::Info,
                        format!(
                            "{}: {} ({})",
                            denial.reason,
                            denial.outcome,
                            agent_name(denial.thread, root)
                        ),
                        "observed: this process",
                    )
                })
                .collect()
        },
    };

    let mut all = vec![
        level,
        Section {
            title: "This session",
            rows: session,
        },
        runtime,
        grants,
        Section {
            title: "Agents",
            rows: agents,
        },
        not_contained,
        not_available,
        denials,
    ];
    if !protected {
        // Under Permissive nothing is required.
        for section in &mut all {
            for row in &mut section.rows {
                row.required = false;
            }
        }
    }
    all
}

fn runtime_rows(input: &InspectorInput, facts: &RuntimeFacts) -> Vec<Row> {
    // Core gates at the stricter of the session's configured level and
    // its live policy.
    let core_protected = input.configured.max(core_level(facts)) != SecurityLevel::Permissive;
    let aggressive_brokers = in_force(input, facts) == SecurityLevel::Aggressive
        && cfg!(any(target_os = "macos", target_os = "linux", windows));
    let control = |label: &str, facts: ControlFacts, on: &str, off: &str| match facts {
        ControlFacts::Enforcing => row(label, State::Enforcing, on, "observed: this process"),
        ControlFacts::Off => row(label, State::Off, off, "observed: this process"),
        ControlFacts::Degraded(reason) => {
            row(label, State::Degraded, reason, "observed: this process")
        }
    };
    let mut rows = vec![
        required(match facts.launch_contract {
            ContractFacts::Armed { hardened: true } => row(
                "Secretless launch",
                State::Enforcing,
                "environment allowlist and launch checks armed",
                "observed: this process",
            ),
            ContractFacts::Armed { hardened: false } => row(
                "Secretless launch",
                State::Degraded,
                "armed, but Core's process hardening failed: protected launches are refused",
                "observed: this process",
            ),
            ContractFacts::Off => row(
                "Secretless launch",
                State::Off,
                "off (secretless_agent_launch)",
                "observed: this process",
            ),
        }),
        required(control(
            "Output gate",
            facts.output_gate,
            "managed secrets are removed from output",
            "off (secret_output_gate)",
        )),
        match facts.model_broker {
            // Installed is not the same as healthy: the broker is not probed.
            ControlFacts::Enforcing => row(
                "Model key broker",
                State::Unobserved,
                "installed; its health is not probed",
                "installed: this process",
            ),
            // #391: Aggressive turns it on where the broker runs, so off
            // there means config turned it off, or the level was chosen
            // after this process started.
            other if aggressive_brokers => required(control(
                "Model key broker",
                other,
                "provider keys are brokered",
                "off: Core reads provider keys itself (broker_model_auth is off in config, or this process started before Aggressive)",
            )),
            other => control(
                "Model key broker",
                other,
                "provider keys are brokered",
                "off: Core reads provider keys itself (broker_model_auth)",
            ),
        },
        if input.network_broker {
            required(row(
                "Credential broker",
                State::Unobserved,
                "on in config; its health is not observed here",
                "resolved: features",
            ))
        } else {
            row(
                "Credential broker",
                State::Off,
                "off (isolated_credential_broker)",
                "resolved: features",
            )
        },
        required(match facts.taint {
            TaintFacts::Off => row(
                "Untrusted content",
                State::Off,
                "labels off (source_envelopes)",
                "observed: session",
            ),
            TaintFacts::Unreadable => row(
                "Untrusted content",
                State::Degraded,
                "the session's registry cannot be read; it is treated as tainted",
                "observed: session",
            ),
            TaintFacts::NotObserved => row(
                "Untrusted content",
                State::Unobserved,
                "no session reports here yet",
                "observed: session",
            ),
            // Post-taint checks run only under a protected Core level.
            TaintFacts::Generation(_) if !core_protected => row(
                "Untrusted content",
                State::Off,
                "labelled, but Core is Permissive: protected actions are not gated",
                "observed: session",
            ),
            TaintFacts::Generation(0) => row(
                "Untrusted content",
                State::Enforcing,
                "labelled; none in this session yet",
                "observed: session",
            ),
            TaintFacts::Generation(generation) => row(
                "Untrusted content",
                State::Enforcing,
                format!(
                    "labelled; session tainted ({generation} batches): protected actions need your fresh approval. Taint is kept with the conversation and stays on resume"
                ),
                "observed: session",
            ),
        }),
    ];
    rows.push(match &input.boundary {
        Some(Boundary::Clean { .. }) => required(row(
            "Protected boundary",
            State::Resolved,
            "preflight clean at launch; earlier conversations are not resumed",
            "resolved: launch preflight",
        )),
        // The summary only: blocker details (paths) stay in the review.
        Some(boundary @ (Boundary::NotClean { .. } | Boundary::Unverified(_))) => required(row(
            "Protected boundary",
            State::Degraded,
            format!("{}; details in the Aggressive review", boundary.summary()),
            "resolved: launch preflight",
        )),
        None => row(
            "Protected boundary",
            State::Off,
            "no preflight applies (protected_mode_preflight off, or not Aggressive)",
            "resolved: launch preflight",
        ),
    });
    rows
}

fn launch_level(input: &InspectorInput) -> SecurityLevel {
    match input.active {
        ChosenLevel::Permissive => SecurityLevel::Permissive,
        ChosenLevel::Aggressive => SecurityLevel::Aggressive,
    }
}

/// Core's level for the session: its root agent's when the tree is live.
fn core_level(facts: &RuntimeFacts) -> SecurityLevel {
    match &facts.policy {
        PolicyFacts::Live(tree) => tree.agents.first().map_or(tree.in_force, |root| root.level),
        PolicyFacts::Stored { level, .. } => *level,
        PolicyFacts::Unreadable => SecurityLevel::Aggressive,
    }
}

fn in_force(input: &InspectorInput, facts: &RuntimeFacts) -> SecurityLevel {
    launch_level(input).max(core_level(facts))
}

/// The one-line verdict. Green only when nothing is degraded, every required
/// row was observed enforcing or checked at launch, and the facts are fresh.
pub(crate) fn badge(
    input: &InspectorInput,
    saved: &Saved,
    facts: &RuntimeFacts,
    sections: &[Section],
    now: i64,
) -> Badge {
    match &facts.policy {
        PolicyFacts::Live(tree) if tree.kill_switch => {
            return Badge::Blocked("kill switch on: every protected action is denied".to_string());
        }
        PolicyFacts::Live(tree) if tree.agents.first().is_some_and(|root| root.stopped) => {
            return Badge::Blocked(
                "this session is stopped: the security state was unreadable at start; a level confirmed in /security applies after restart".to_string(),
            );
        }
        PolicyFacts::Stored {
            unreadable: true, ..
        } => {
            return Badge::Blocked(
                "security state unreadable: Aggressive and the kill switch apply".to_string(),
            );
        }
        PolicyFacts::Stored {
            kill_switch: true, ..
        } => return Badge::Blocked("kill switch on at the next start".to_string()),
        PolicyFacts::Live(_) | PolicyFacts::Stored { .. } | PolicyFacts::Unreadable => {}
    }
    let level = level_name(in_force(input, facts));
    if level == "Permissive" {
        return match saved.level.enforced() {
            ChosenLevel::Aggressive => Badge::Degraded(
                level,
                vec!["Aggressive is saved but not active until restart".to_string()],
            ),
            ChosenLevel::Permissive => Badge::Permissive,
        };
    }
    let mut degraded = Vec::new();
    let mut partial = Vec::new();
    // Core's level can rise at once, but Aggressive's sandbox, approval,
    // network and vault rows apply only from the next start.
    if input.active == ChosenLevel::Permissive {
        if saved.level.enforced() == ChosenLevel::Aggressive {
            degraded.push("Aggressive's session controls apply after restart".to_string());
        } else {
            // Core's level comes from config: this session's sandbox,
            // approval and network values were not checked against it.
            partial.push("session controls not checked at launch".to_string());
        }
    }
    // A level checked at launch whose protected-action gates Core does not
    // enforce (Core lags behind a saved Aggressive) is not protection.
    if matches!(facts.policy, PolicyFacts::Live(_)) && core_level(facts) < launch_level(input) {
        let missing = match core_level(facts) {
            SecurityLevel::Permissive => "protected-action gates",
            SecurityLevel::Moderate | SecurityLevel::Aggressive => "grant and protected-path rules",
        };
        degraded.push(format!(
            "Core enforces {}; {}'s {missing} are not active",
            level_name(core_level(facts)),
            level_name(launch_level(input))
        ));
    }
    if now - facts.observed_at > STALE_AFTER_SECONDS {
        degraded.push(format!(
            "status observed {}; press r",
            ago(now - facts.observed_at)
        ));
    }
    for row in sections.iter().flat_map(|section| &section.rows) {
        // Any degraded component counts; off or unobserved ones only when
        // the level needs them.
        match (row.required, row.state) {
            (_, State::Degraded) => degraded.push(row.label.clone()),
            (true, State::Off | State::Unobserved) => partial.push(row.label.clone()),
            _ => {}
        }
    }
    if !degraded.is_empty() {
        degraded.extend(partial);
        Badge::Degraded(level, degraded)
    } else if !partial.is_empty() {
        Badge::Partial(level, partial)
    } else {
        Badge::Protected(level)
    }
}

#[cfg(test)]
#[path = "inspector_tests.rs"]
pub(crate) mod tests;
