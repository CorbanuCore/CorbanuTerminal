//! PF-41-S01: read-only runtime facts for the `/security` inspector.
//!
//! [`observe`] reports what this process enforces right now, apart from what
//! configuration asks for: the live policy tree and its agents, held grants,
//! the session's taint, the process-wide launch, output and credential
//! controls, and recent denials. Nothing here changes authority: there is no
//! way to raise, lower, grant, revoke or clear a stop through this module.
//!
//! Every value is secret-free by construction: levels, counters, thread ids,
//! timestamps and fixed text. Grant ids, actor tokens, nonces, paths,
//! commands and free-form error text are never returned.

use std::collections::VecDeque;
use std::path::Path;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::sync::Weak;

use codex_protocol::ThreadId;
pub use codex_security_policy::SecurityLevel;

use super::ingress::NativeIngress;
use super::launch_contract::LaunchDenied;
use super::tainted_action::ProtectedActionKind;

/// Denials kept for the inspector; older ones are dropped.
const MAX_DENIALS: usize = 32;

/// Everything the inspector shows that only Core can observe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeFacts {
    pub observed_at: i64,
    pub policy: PolicyFacts,
    pub grants: Vec<GrantFacts>,
    pub taint: TaintFacts,
    pub denials: Vec<Denial>,
    pub launch_contract: ContractFacts,
    pub output_gate: ControlFacts,
    pub model_broker: ControlFacts,
}

/// The security policy of the inspected session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyFacts {
    /// The session's policy tree runs in this process.
    Live(TreeFacts),
    /// No tree here holds the session (not started yet, or a remote app
    /// server): what the next start reads from the stored state.
    Stored {
        level: SecurityLevel,
        kill_switch: bool,
        /// The stored state could not be read: the next start enforces
        /// Aggressive and the kill switch until it is repaired.
        unreadable: bool,
    },
    /// The tree exists but cannot be read (poisoned or not initialized).
    Unreadable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeFacts {
    pub in_force: SecurityLevel,
    pub next_start: SecurityLevel,
    pub kill_switch: bool,
    /// Moves on every committed level change.
    pub epoch: u64,
    /// Moves on every revocation.
    pub revocation_generation: u64,
    /// Root first, then children by depth.
    pub agents: Vec<AgentFacts>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentFacts {
    pub thread: ThreadId,
    /// 0 for the session's root agent.
    pub depth: usize,
    pub level: SecurityLevel,
    /// Held to a stricter level than the session's (inherited or configured).
    pub stricter_than_session: bool,
    /// Denied everything (kill switch or unreadable state at start).
    pub stopped: bool,
}

/// A live grant. The id and the exact operation are not exposed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrantFacts {
    pub thread: ThreadId,
    pub surface: &'static str,
    pub expires_at: i64,
    pub used: u64,
    pub limit: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaintFacts {
    /// Untrusted-content labels (`source_envelopes`) are off for the session.
    Off,
    /// No session client of this process reports for the thread.
    NotObserved,
    /// Count of recorded batches of content without standing; 0 is clean.
    Generation(u64),
}

/// One process-wide control, as observed in this process.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlFacts {
    /// Not turned on for this process.
    Off,
    /// Running and checked.
    Enforcing,
    /// Turned on but cannot protect: the text is fixed and secret-free.
    Degraded(&'static str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContractFacts {
    Off,
    Armed { hardened: bool },
}

/// A recent refusal. `reason` and `outcome` are fixed text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Denial {
    pub at: i64,
    pub thread: Option<ThreadId>,
    pub reason: &'static str,
    pub outcome: &'static str,
}

static DENIALS: LazyLock<Mutex<VecDeque<Denial>>> = LazyLock::new(Default::default);
type IngressEntry = (ThreadId, Weak<Mutex<NativeIngress>>);
static INGRESS: LazyLock<Mutex<Vec<IngressEntry>>> = LazyLock::new(Default::default);

/// Read the facts for `thread` (the session the inspector was opened in) on
/// `codex_home`. `configured` is the session's configured Core level.
pub fn observe(
    codex_home: &Path,
    configured: SecurityLevel,
    thread: Option<ThreadId>,
    now: i64,
) -> RuntimeFacts {
    let policy =
        match thread.and_then(|thread| super::transition::live_controller(codex_home, thread)) {
            Some(controller) => controller
                .tree_facts()
                .map_or(PolicyFacts::Unreadable, PolicyFacts::Live),
            None => {
                let recovered = super::recovery::recover(codex_home, configured);
                PolicyFacts::Stored {
                    level: recovered.level,
                    kill_switch: recovered.revocations.kill_switch_active
                        || recovered.unreadable.is_some(),
                    unreadable: recovered.unreadable.is_some(),
                }
            }
        };
    let grants = match &policy {
        PolicyFacts::Live(tree) if !tree.kill_switch => tree
            .agents
            .iter()
            .flat_map(|agent| {
                super::aggressive::held(agent.thread, tree.epoch, tree.revocation_generation, now)
            })
            .collect(),
        // Grants are never stored and end with the kill switch.
        PolicyFacts::Live(_) | PolicyFacts::Stored { .. } | PolicyFacts::Unreadable => Vec::new(),
    };
    // Denials of the session, its children, and those tied to no thread.
    let threads: Vec<ThreadId> = match &policy {
        PolicyFacts::Live(tree) => tree.agents.iter().map(|agent| agent.thread).collect(),
        PolicyFacts::Stored { .. } | PolicyFacts::Unreadable => thread.into_iter().collect(),
    };
    RuntimeFacts {
        observed_at: now,
        policy,
        grants,
        taint: thread.map_or(TaintFacts::NotObserved, taint_for),
        denials: recent_denials(&threads),
        launch_contract: super::launch_contract::active().map_or(ContractFacts::Off, |contract| {
            ContractFacts::Armed {
                hardened: contract.hardened(),
            }
        }),
        output_gate: if super::disclosure_gate::active().is_some() {
            ControlFacts::Enforcing
        } else {
            ControlFacts::Off
        },
        model_broker: match (
            codex_model_provider::model_key_broker_required(),
            codex_model_provider::model_key_broker_installed(),
        ) {
            (_, true) => ControlFacts::Enforcing,
            (true, false) => {
                ControlFacts::Degraded("no broker is running; provider keys are not sent")
            }
            (false, false) => ControlFacts::Off,
        },
    }
}

/// Called by a session client when untrusted-content labels are on, so the
/// inspector can read the thread's taint without holding the session.
pub(crate) fn register_ingress(thread: ThreadId, ingress: &std::sync::Arc<Mutex<NativeIngress>>) {
    let mut entries = INGRESS.lock().unwrap_or_else(PoisonError::into_inner);
    entries.retain(|(_, entry)| entry.strong_count() > 0);
    entries.push((thread, std::sync::Arc::downgrade(ingress)));
}

fn taint_for(thread: ThreadId) -> TaintFacts {
    let ingress = INGRESS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .rev()
        .filter(|(entry_thread, _)| *entry_thread == thread)
        .find_map(|(_, entry)| entry.upgrade());
    let Some(ingress) = ingress else {
        return TaintFacts::NotObserved;
    };
    match ingress.lock() {
        Ok(ingress) if ingress.labelled_mode() => {
            TaintFacts::Generation(ingress.taint_generation())
        }
        Ok(_) => TaintFacts::Off,
        // A poisoned registry counts as tainted, as the post-taint checks do.
        Err(_) => TaintFacts::Generation(u64::MAX),
    }
}

/// A protected action after untrusted content was not run.
pub(crate) fn record_protected_action(
    thread: Option<ThreadId>,
    kind: ProtectedActionKind,
    outcome: &'static str,
) {
    let outcome = match outcome {
        "refused_kill_switch" => "refused: kill switch on",
        "refused_approvals_off" => "refused: approvals are off",
        "refused_stale" => "refused: taint or policy changed after approval",
        "declined" => "declined by you",
        _ => return,
    };
    record(thread, kind.describe(), outcome);
}

/// Held while a person is asked about a protected action after untrusted
/// content. If the turn ends first (Esc interrupts it), the action did not
/// run and is recorded as declined.
pub(crate) struct PendingProtectedAction {
    thread: ThreadId,
    kind: ProtectedActionKind,
    answered: bool,
}

impl PendingProtectedAction {
    pub(crate) fn new(thread: ThreadId, kind: ProtectedActionKind) -> Self {
        Self {
            thread,
            kind,
            answered: false,
        }
    }

    /// The answer arrived and is recorded by the caller.
    pub(crate) fn answered(&mut self) {
        self.answered = true;
    }
}

impl Drop for PendingProtectedAction {
    fn drop(&mut self) {
        if !self.answered {
            record(
                Some(self.thread),
                self.kind.describe(),
                "declined by you (turn interrupted)",
            );
        }
    }
}

/// The secretless launch contract refused a command or input.
pub(crate) fn record_launch_denial(denied: &LaunchDenied) {
    let reason = match denied {
        LaunchDenied::UnsupportedPlatform => "unsupported platform",
        LaunchDenied::ProcessHardening => "process hardening failed",
        LaunchDenied::Unsandboxed => "command would run outside the OS sandbox",
        LaunchDenied::RemoteEnvironment => "remote environment cannot be checked",
        LaunchDenied::UnprotectableFileSystem => "unrestricted file access",
        LaunchDenied::ProtectedPathReadable(_) => "a protected path was readable",
        LaunchDenied::PolicyStoreWritable => "Corbanu's configuration was writable",
        LaunchDenied::LoginShell => "login shell",
        LaunchDenied::RawSecretInArgv => "managed secret in the command line",
        LaunchDenied::RawSecretInStdin => "managed secret in the input",
    };
    record(/*thread*/ None, reason, "launch refused");
}

fn record(thread: Option<ThreadId>, reason: &'static str, outcome: &'static str) {
    let mut denials = DENIALS.lock().unwrap_or_else(PoisonError::into_inner);
    if denials.len() == MAX_DENIALS {
        denials.pop_front();
    }
    denials.push_back(Denial {
        at: super::aggressive::now_unix_seconds(),
        thread,
        reason,
        outcome,
    });
}

/// Newest first: denials of `threads` and those not tied to a thread.
fn recent_denials(threads: &[ThreadId]) -> Vec<Denial> {
    DENIALS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .rev()
        .filter(|denial| denial.thread.is_none_or(|thread| threads.contains(&thread)))
        .cloned()
        .collect()
}

#[cfg(test)]
#[path = "inspection_tests.rs"]
mod tests;
