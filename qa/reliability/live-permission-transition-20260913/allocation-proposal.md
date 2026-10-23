# Proposed existing-plan allocation — approval pending

This is a proposal, not an executable sprint. Canonical plan/sprint files have not been changed. Parent approval must be recorded before runtime implementation.

| Field | Proposed allocation |
| --- | --- |
| Class | Product initiative; live authorization application boundary |
| Plan | `docs/plans/active/p0-security-levels.md` (already active) |
| Single feature | PF-22, “Effective runtime policy and agent inheritance” |
| New sprint | PF-22-S03, “Live permission application boundary”; proposed `docs/sprints/current/p0-security-levels/pf-22-s03-live-permission-application-boundary.md` |
| Draft status | Proposal only, no reservation; canonical record must be created from `docs/sprints/SPRINT_TEMPLATE.md` |
| Dependencies | PF-22-S02, PF-21-S02 (both archived completed); no dependency on or resumption of PF-27 |
| Owner / lane | This assigned worker / `live-permission-transition` |
| Worktree | `/Volumes/CorbanuDrive/Corbanu/worktrees/live-permission-transition-20260913` |
| Branch | `fix/live-permission-transition-20260913` |
| Base | `005cc644f59b1e762e5497b329e106c67925d4ed` |
| Receiving owner | Parent integrator, coordinating the existing security owner; exact named owner recorded on approval |
| Target | Reviewable isolated candidate; no release or live-instance replacement |
| Size | Target <=500 changed lines; stop before exceeding 800 or adding an unlisted path; bring evidence to parent |

## Parent-owned prerequisite

At this base, checkers report three active plans and no free slot. PF-27-S04 is blocked and retains the security reservation; PF-60-S02 is blocked; PF-80-S01 is in progress. Historical portfolio prose does not override front matter or the September 13 pause.

Recommend parent explicitly preserve PF-27's paused handoff and return its reservation to draft, without resuming its owner or work. Record a PF-22-only follow-up in the existing plan, exact coordinates, narrow write scope and receiving gate; then allocate PF-22-S03 as the sole executable security sprint. No fourth plan, no extra within-plan sprint, no use of PF-80 to disguise this security repair. Parent must accept the extension of PF-22 to the requested low-level `/permissions` boundary; the plan currently excludes replacing `/permissions` and changing Permissive behavior. The proposal preserves configured policies and the existing surface but changes when a requested policy can be accepted. It cannot silently amend that contract.

Run both plan and sprint checkers after the amendment and before dispatch. If the parent cannot allocate this scope under the existing feature, leave this proposal non-executable and return the specific scope decision; do not invent a reservation.

## Proposed mechanical mandate

1. Centralize validation and an atomic permission-transition admission check at the two native settings mutation paths. Treat approval policy, reviewer, permission profile (including identity and constraints), sandbox level and authority-affecting environment/root changes as one coherent candidate. No global `Never`; preserve independent platform, tool and security restrictions.
2. Accept authority changes only when the affected execution scope is quiescent. Serialize with context/task admission and teardown, tool authorization/startup, outstanding approvals, background process reservations/lifetimes, and shared network/MCP updates. Hold no locks across approval waits. An incompatible stale context must not be admitted after commit. A read-only list of running processes followed by a separate commit is insufficient.
3. Refuse both increases and decreases while busy before mutating effective state, services or persisted state. Do not auto-approve, cancel, reissue or replay an outstanding tool approval. Existing already-authorized operations keep their original authority only while the proposed change is explicitly refused; never claim a restriction is applied while broader work remains. No accepted downgrade may admit work under old broader authority.
4. Show effective settings from the exact successful backend result. During the request, distinguish the requested value from effective authority; on invalid/busy/rejected/cancelled failure keep or restore the last confirmed value. Do not persist a pending request. Serialize UI selections or correlate replies so an older acknowledgement cannot overwrite a later accepted selection. RPC submission alone is not success.
5. Provide the explicit application path: finish work or use existing human stop controls, ensure remaining relevant tools/background terminals are idle, then retry. Do not stop anything automatically. If quiescence cannot be proved, refuse with an actionable reason. Preserve history, completed effects and original pending approval decisions across retry/recovery.
6. Add synchronized regression coverage for both directions, invalid mixed-field updates, no-op settings, racing admission, background startup/teardown, approval waits, both settings entrypoints, stale responses and recovery. Reconcile the existing MCP live-authority tests explicitly. Use parent-frozen acceptance input unchanged.

This is a design to prove, not a claim that a mutex alone solves authority changes. Child agents, remote execution, detached work and shared-service consumers require a scope audit: if they are covered by the setting and can retain old authority, either include them in the proven quiescence check or stop for a revised allocation. Do not silently narrow the promise.

## Literal proposed source/test boundary

These are candidate paths for parent approval, not edits already made. Product policy belongs in a focused private module with thin native adapters. No dependency or wire-schema change is preauthorized.

- `codex-rs/core/src/session/mod.rs`
- `codex-rs/core/src/session/session.rs`
- `codex-rs/core/src/session/handlers.rs`
- `codex-rs/core/src/session/turn_context.rs`
- `codex-rs/core/src/session/permission_transition.rs` (new private module)
- `codex-rs/core/src/session/permission_transition_tests.rs` (new sibling tests)
- `codex-rs/core/src/session/tests.rs` (existing MCP regression reconciliation only)
- `codex-rs/core/src/tasks/mod.rs`
- `codex-rs/core/src/unified_exec/process_manager.rs`
- `codex-rs/core/src/tools/orchestrator.rs` (admission lifetime seam only, if required)
- `codex-rs/core/tests/suite/mod.rs` (test registration)
- `codex-rs/core/tests/suite/permission_transition.rs` (new native integration tests)
- `codex-rs/tui/src/app/config_persistence.rs`
- `codex-rs/tui/src/app/thread_settings.rs`
- `codex-rs/tui/src/app/app_server_events.rs`
- `codex-rs/tui/src/chatwidget/permissions_menu.rs`
- `codex-rs/tui/src/app/tests.rs`
- `codex-rs/tui/src/app/snapshots/` (only newly generated permission-transition snapshots; exact filenames must be frozen before handoff)
- `qa/reliability/live-permission-transition-20260913/`

App-server's `request_processors/turn_processor.rs` and `bespoke_event_handling.rs`, plus protocol events, are inspected adapter candidates but **not included** in the initial write allocation. Verify existing completion/error correlation first. If it is inadequate, parent must add exact adapter, schema and test paths and the worker must read `codex-rs/APP_SERVER_GUIDE.md` before those edits. Similarly return any new MCP/network/agent admission seam before expanding scope. These are concrete design gates, not permission to implement only part of the safety boundary.

Parent-only planning paths: active P0 plan, PF-27-S04 reservation/handoff record, proposed PF-22-S03 record and `docs/sprints/current/p0-security-levels/index.md`. Existing accepted PF-22 archives stay unchanged. Parent serializes shared Core/TUI work and audits literal overlap against its live allocations before approval.

## Upstream-touch and integration gate

Fork base is verified above. Canonical source is `https://github.com/openai/codex`; exact applicable upstream SHA remains unresolved. The old plan calls `1bdc515bff48a4d9048dae7d06c6214e884265bc` upstream, but local commit subject is `Merge pull request #117 from CorbanuCore/feat/p0-security-levels`, so that is not sufficient OpenAI provenance. Local `refs/remotes/upstream/main` is `1d74c3ba1ee98be2025ab066dcc3fd654fe8a3b6`; remote name alone is not canonical verification. Parent/worker must resolve the seam baseline from maintained repository evidence before sprint readiness; no fetch or upstream integration was performed.

Preserve native Op/turn/cancellation/history/persistence interfaces and approval semantics. Any changed response semantics need adapter regression proof. Receiving parent audits scope, obtains fresh Fable review, resolves findings, integrates only under separately recorded receiving authority and reruns affected Core/TUI/API checks on the combined tree. Worker has no push/merge authority. True-TUI and independent code-blind execution remain separate mandatory gates, with no acceptance by the implementer.

## Done

- [x] Verified isolated coordinates and classified before editing.
- [x] Inspected mutation, snapshot, exec, MCP, background-process and UI seams.
- [x] Compared candidate approaches and recorded proposed boundary and limits.
- [x] Linked and verified hash of independent frozen F01–F11.
- [x] Recorded occupied reservations, pause, exact candidate paths and parent next action.

## Remaining

- [ ] Parent approves the concrete boundary and plan amendment; records sole reservation, named owner and exact scopes.
- [ ] Resolve canonical upstream baseline and admission/correlation scope gates; pass plan/sprint checks before runtime dispatch.
- [ ] Implement only the approved sprint checklist, then format and run locked pinned regressions with private cache.
- [ ] Fresh Fable review, isolated independent functional execution, evidence review and combined-tree gates.
- [ ] Record live-repository applicability, true-TUI evidence, human acceptance, documentation and release/benchmark status honestly; no release authorized here.
