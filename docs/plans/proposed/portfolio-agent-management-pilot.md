---
title: "Native agent management and independent acceptance pilot"
status: draft
change_class: product-initiative
priority: P1
owner: "Jim Ricketts (integration lead, proposed)"
parallel_sprint_limit: 1
integration_owner: "Jim Ricketts (proposed; confirmation required)"
activation_authority: "Travis Good"
activation_basis: "2026-09-09 request authorizes planning and Fable review only; activation decision pending"
target_release: "TBD"
deadline: "TBD"
created: 2026-09-09
updated: 2026-09-10
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "Shipping MVP — LIVE"
  requirement_excerpt: "model-aware delegation, durable mailboxes, supervision, resume, and recovery."
implementation_worktrees: []
---

# Native agent management and independent acceptance pilot

Policy: repository-root `AGENTS.md`. Lifecycle: [plans](../index.md).
Portfolio: [source coverage, sequencing and human capacity](../portfolio-2026-09-09.md).
This is a fully specified **draft pilot plan**, not an executable assignment or a shipped-feature claim.

## Activation record

- Status: draft; active slot: none; target release/date: not promised.
- Planning authority: user's 2026-09-09 request. Product activation: Travis Good, pending.
- Proposed priority/owners are recommendations, not new assignments or reordered P0 commitments.
- Gate: Reconcile active work before allocating any pilot; choose one permitted workload, spend cap, named acceptance tester and live Task Node write policy.

## User pain

The limiting resource is trustworthy acceptance and human decision capacity, not available machines or the number of agents.

## Product intent and ideal flow

Demonstrate a native-first, two-builder maximum workflow with an independent tester, durable evidence and a one-hour daily human review budget.
Entry is the first sprint's approved contract; success, failure and return-use are defined below.
The two-builder ceiling bounds this calibration experiment, not the user's
approved eventual three-initiative model. See the [operating-model transition](../scrum-2026-09-10.md#operating-model).

## Product linkage

- Exact heading: **Shipping MVP — LIVE** in [the product spec](../../corbanu-product-spec.md).
- Requirement excerpt: “model-aware delegation, durable mailboxes, supervision, resume, and recovery.”
- Feature: **PF-75**; outcome: Demonstrate a native-first, two-builder maximum workflow with an independent tester, durable evidence and a one-hour daily human review budget.
- Source: transcript discussion at User's management request; Alex 01:46:46–01:49:00 and 02:25:25–02:26:45; see portfolio for attribution caveats.
- This citation supplies product context, not authorization for additional scope. The bounded pilot is an enabling/editorial workflow. Product changes or external actions need an explicit subsequent scope decision.

## Scope

- In: Native capability and authority contract; Human-style acceptance and tester calibration; Bounded native end-to-end rehearsal; Two-week pilot operating decision.
- Out: A second scheduler, unattended merges/releases, new task systems, broad credentials, automatic Task Node posting or the claim that testing is bulletproof.

## Invariants

- No credential, proprietary strategy, customer log or financial record is disclosed by default; only approved redacted/public/synthetic inputs leave the host.
- A model finding or a passing test is not authority to publish, spend, trade, change permissions, merge or release.
- Unknowns remain unknown; negative findings and no-go decisions count as useful research, not fabricated completion.
- No product-runtime mutation, account action or live integration is included in the research/pilot decision unless a separately activated sprint explicitly authorizes it.

## Ownership and implementation worktrees

| Proposed accountable role | Worktree | Branch | Base | Scope |
| --- | --- | --- | --- | --- |
| Jim Ricketts (integration lead, proposed) | UNALLOCATED | UNALLOCATED | UNALLOCATED | PF-75; one sprint at a time |
| Jim Ricketts, receiving integrator (confirm) | UNALLOCATED | UNALLOCATED | UNALLOCATED | Source reconciliation and combined evidence |

No worktree or worker slot is reserved. Before readiness, confirm actual owner, exact repository/branch/40-character base, literal write scopes including registration/tests/manifests, and receiving gate. Cross-repository work requires separate explicit coordinates and authority.

## Useful code references

| Existing path | Purpose |
| --- | --- |
| `codex-rs/core/src/agent/control/mailbox.rs` | Existing boundary to inspect/reuse; not a claim that this feature already exists |
| `codex-rs/core/src/agent/control/execution.rs` | Existing boundary to inspect/reuse; not a claim that this feature already exists |
| `codex-rs/core/src/agent/registry.rs` | Existing boundary to inspect/reuse; not a claim that this feature already exists |
| `codex-rs/tasknode-session/src/lib.rs` | Existing boundary to inspect/reuse; not a claim that this feature already exists |
| `docs/tmuxHarness.md` | Existing boundary to inspect/reuse; not a claim that this feature already exists |
| `codex-rs/cli/src/tasknode_cmd.rs` | Native JSON task lifecycle, evidence preflight and returned lifecycle state |
| `codex-rs/tasknode-session/src/recovery.rs`, `client.rs`, `commands.rs` | Shared scoped login/client and durable task-request recovery |
| `codex-rs/tasknode-session/src/tracker.rs` | Existing encrypted Campaign Tracker outbox |
| `codex-rs/tui/src/chatwidget/tasknode_menu.rs`, `campaign_tracker.rs` | Human controls, opt-in TUI capture and explicit coverage limits |

## September 10 native integration amendment

The [call follow-up](../call-followup-2026-09-10.md#native-task-node-boundary)
records the inspected native interfaces and the required mapping, lifecycle,
capture-coverage and recovery contract. S01 specifies those boundaries; S02
freezes user-only tests; S03 rehearses them dry-run; S04 decides whether one
bounded missing adapter deserves an implementation amendment. No second login
store, tracker or dispatcher is assumed. Basic operational visibility has no
PF-60/ROI prerequisite. PF-72 consumes attributable outcomes later.

The HTML map remains a private read-only projection with a single publisher,
approximately half-hour freshness, machine/coverage labels, blocker and agent
logs, rendered sprint descriptions and human-test links. An existing external
goal-only adapter must be reconciled, not treated as absent or as a full task
lifecycle client. Existing operator code/location must be pinned before reuse.

Research/artifact paths in the sprint table are planned files, not existing implementations. External harness/research locations must be identified by the owner; absence blocks that sprint rather than licensing a guessed rebuild.

## Native lifecycle and upstream-touch record

- Observed planning checkout: `/Volumes/CorbanuDrive/Corbanu/CorbanuTerminal`, branch `recovery/corbanu-drive-2026-09-02`, HEAD `6f8f4ce46446e951437e1ce7e7c4a526e5b9e546`.
- Historical source only. Publication uses main baseline `3cec54d9917b776bedaffb586247d7cd7c633df6`; the [scrum packet](../scrum-2026-09-10.md) records receiving state and gates. Future implementation coordinates remain unallocated.
- Canonical upstream: `https://github.com/openai/codex.git`; verified upstream SHA for this future candidate: unresolved, blocks code readiness.
- This bounded scope produces research/QA artifacts, not a new scheduler/provider/runtime. Runtime touch is not applicable to artifact-only sprints. Any experiment executable must have its isolated workspace and exact commands approved at its preceding contract gate.
- Follow [upstream integration](../upstream-integration.md). Shared migrations, module registration, lockfiles and config schemas require serial ownership.

## Sprint execution map

All records belong to the single feature **PF-75**. Dependencies are hard prerequisites, not suggestions. All results are pending.

| Sprint | Record / outcome | Depends on | Planned output | Evidence |
| --- | --- | --- | --- | --- |
| PF-75-S01 | [Native capability and authority contract](../../sprints/current/portfolio-agent-management-pilot/pf-75-s01-native-capability-and-authority-contract.md) | none | docs/research/agent-management-pilot/contract.md | pending |
| PF-75-S02 | [Human-style acceptance and tester calibration](../../sprints/current/portfolio-agent-management-pilot/pf-75-s02-human-style-acceptance-and-tester-calibration.md) | PF-75-S01 | docs/research/agent-management-pilot/acceptance-contract.md | pending |
| PF-75-S03 | [Bounded native end-to-end rehearsal](../../sprints/current/portfolio-agent-management-pilot/pf-75-s03-bounded-native-end-to-end-rehearsal.md) | PF-75-S02 | qa/portfolio/agent-management-pilot/rehearsal.md | pending |
| PF-75-S04 | [Two-week pilot operating decision](../../sprints/current/portfolio-agent-management-pilot/pf-75-s04-two-week-pilot-operating-decision.md) | PF-75-S03 | docs/research/agent-management-pilot/decision.md | pending |

## Acceptance flows

| Flow | Starting state / action | Expected result and pass criterion |
| --- | --- | --- |
| Success | Approved inputs; execute bounded sprint sequence | A coordinator hands off bounded work, detects stalled/failed workers, rejects bad evidence and presents at most three actionable human decisions per day. |
| Failure/cancel | Missing input, rejected gate or interrupted work | Expired login, duplicate dispatch, reviewer mismatch, stale binary, lease loss, quota exhaustion and malformed evidence cannot produce a completed task. |
| Recovery/resume | Reopen from a recorded checkpoint | Restart coordinator and a worker; reconstruct native state and artifacts without duplicate actions or a hidden credential repair. |

## Implementation sequence

1. **Native capability and authority contract:** One source owns execution state; Task Node is a coordination/evidence projection, not a second dispatcher.
2. **Human-style acceptance and tester calibration:** All four seeded failures are detected; a previously qualified positive recovery control passes on the recorded candidate without privileged assistance.
3. **Bounded native end-to-end rehearsal:** No duplicate execution or false completion; new work stops when queue/budget caps are reached; all evidence traces to one candidate.
4. **Two-week pilot operating decision:** The human selects whether to activate a pilot; proposed scheduling does not create an automation or start workers.

## Automated evidence

- Every sprint: `python3 docs/plans/check.py; python3 docs/sprints/check.py` from repository root; `git diff --check`.
- Focused commands and artifact checks are explicit in each sprint. A newly specified selector must be registered, shown to run nonzero tests and resolved before readiness; an empty test filter is not evidence.
- Evidence includes input/candidate digests, command, timestamps, exit status, expected versus actual result and limitations. Checklists alone do not prove behavior.
- All evidence fields remain pending until the actual final tree or artifact is checked; this planning pass does not execute the future sprint tests.

## True-TUI evidence

Required for applicable runtime/rehearsal journeys: use the final built candidate in an isolated actual PTY with `RUST_LOG=trace` and private log directory. Send literal input and Enter separately. Exercise the success, failure/cancel and recovery/resume rows above; capture reachable controls, exact model/provider, binary hash, base SHA, keys, checkpoints and cleanup. A tester may use only public user controls; fixture setup is a separate operator role. Shell repair of auth or hidden state invalidates user recovery proof. Corbanu `exec`, a model review and snapshots alone do not qualify these flows. Result: pending.

## Live-repository applicability

| Repository | Applicability for this scope | Checkout/base | Result |
| --- | --- | --- | --- |
| TensorCash | Required where the selected runtime/QA journey affects live repository work | UNALLOCATED; resolve before qualification | pending |
| Isometric Game | Required where the selected runtime/QA journey affects live repository work | UNALLOCATED; resolve before qualification | pending |

Release-level live-repository gates remain intact regardless of this plan's bounded applicability.

## Human acceptance

Proposed accountable owner reviews the exact artifact/candidate; Travis or named delegate decides product scope. Named tester, date, digest, observed flows, result and evidence are **pending**. Agent review is advisory and never substitutes for required human acceptance.

## Documentation

Research outputs stay under the declared research/QA paths. Finished-feature docs change only after qualification and name the verified candidate and product-spec citation. Unimplemented possibilities remain proposals.

## Dependencies, decisions, and blockers

- Reconcile active work before allocating any pilot; choose one permitted workload, spend cap, named acceptance tester and live Task Node write policy.
- Before activation: reconcile canonical worktree/release, inherited ledger errors and active slots. The [three-initiative operating-model transition](../scrum-2026-09-10.md#operating-model) is separate from this draft publication; follow the receiving policy until it lands.
- Calibration dependency: S01 must supply a previously qualified positive recovery journey and evidence on the pinned candidate. S02 tests that positive control plus four seeded negatives. If an unfinished auth flow is required as the positive control, add its receiving-branch sprint dependency after reconciliation; PF-75 never implements the auth repair.
- Before each next sprint: predecessor evidence accepted/archived, its handoff resolves concrete inputs and any newly discovered decisions; otherwise stop.
- Cross-plan IDs in the table must exist and be completed in the receiving ledger. A draft dependency does not activate either plan.
- Before dispatch: agreed budget, named human acceptance owner, exact scope and privacy permissions. No capacity or deadline is assumed from hardware ownership alone.

## Release linkage

This bounded plan produces a decision/research/pilot package, not a Terminal release. Sprint artifact acceptance is not a shipped initiative. Keep the plan in the appropriate lifecycle until an authorized lifecycle decision is recorded; if code is later approved, add release gates rather than marking the idea shipped.

## Completion

- [ ] All sprint outputs are accepted and linked; no expected evidence is invented.
- [ ] Scope, inputs, dependencies, exact allocation and authority are current.
- [ ] Automated/artifact checks and applicable actual-key tests pass.
- [ ] Named human accepts the exact candidate or decision package.
- [ ] Any implementation has finished docs, release linkage and all required release/benchmark evidence.
- [ ] Remaining implementation ideas are separately gated; no hard gate is silently waived.
