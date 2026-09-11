---
title: "Campaign Tracker and agent ROI integration"
status: draft
change_class: product-initiative
priority: P2
owner: "Alex Good (campaign owner, proposed)"
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
  heading: "Product measurement"
  requirement_excerpt: "No commercial performance numbers have been supplied."
implementation_worktrees: []
---

# Campaign Tracker and agent ROI integration

Policy: repository-root `AGENTS.md`. Lifecycle: [plans](../index.md).
Portfolio: [source coverage, sequencing and human capacity](../portfolio-2026-09-09.md).
This is a fully specified **draft pilot plan**, not an executable assignment or a shipped-feature claim.

## Activation record

- Status: draft; active slot: none; target release/date: not promised.
- Planning authority: user's 2026-09-09 request. Product activation: Travis Good, pending.
- Proposed priority/owners are recommendations, not new assignments or reordered P0 commitments.
- Gate: inspect receiving main's Campaign Tracker local pilot first; Alex confirms the authoritative tracker/schema and allowed export. Use PF-60 accounting definitions before any spend-derived ROI.

- Additional activation gate: the current spec authorizes the Campaign Tracker local pilot, not this initiative's complete spend-derived ROI scope. Travis must approve any additional concrete outcome and necessary product-spec amendment before Terminal-plan activation, or explicitly move genuinely external tracker work into separately owned records. Reclassification must not evade existing WIP limits.

## User pain

Activity summaries are not business outcomes, and Alex already has a Campaign Tracker that should be inspected before another dashboard is built.

## Product intent and ideal flow

Map the existing tracker, define evidence-backed outcome/cost attribution and demonstrate an offline integration using synthetic or consented redacted records.
Entry is the first sprint's approved contract; success, failure and return-use are defined below.

## Product linkage

- Exact heading: **Product measurement** in [the product spec](../../corbanu-product-spec.md).
- Requirement excerpt: “No commercial performance numbers have been supplied.”
- Feature: **PF-72**; outcome: Map the existing tracker, define evidence-backed outcome/cost attribution and demonstrate an offline integration using synthetic or consented redacted records.
- Source: transcript discussion at 01:31:00–01:38:00; see portfolio for attribution caveats.
- This citation supplies product context, not authorization for additional scope. The bounded pilot is an enabling/editorial workflow. Product changes or external actions need an explicit subsequent scope decision.

## Scope

- In: Existing tracker and outcome contract; Offline attribution and import fixture; Native integration decision and pilot handoff.
- Out: Replacing Alex's tracker, copying raw logs to external models, claiming causal ROI from activity counts or deploying another dashboard.

## Invariants

- No credential, proprietary strategy, customer log or financial record is disclosed by default; only approved redacted/public/synthetic inputs leave the host.
- A model finding or a passing test is not authority to publish, spend, trade, change permissions, merge or release.
- Unknowns remain unknown; negative findings and no-go decisions count as useful research, not fabricated completion.
- No product-runtime mutation, account action or live integration is included in the research/pilot decision unless a separately activated sprint explicitly authorizes it.

## Ownership and implementation worktrees

| Proposed accountable role | Worktree | Branch | Base | Scope |
| --- | --- | --- | --- | --- |
| Alex Good (campaign owner, proposed) | UNALLOCATED | UNALLOCATED | UNALLOCATED | PF-72; one sprint at a time |
| Jim Ricketts, receiving integrator (confirm) | UNALLOCATED | UNALLOCATED | UNALLOCATED | Source reconciliation and combined evidence |

No worktree or worker slot is reserved. Before readiness, confirm actual owner, exact repository/branch/40-character base, literal write scopes including registration/tests/manifests, and receiving gate. Cross-repository work requires separate explicit coordinates and authority.

## Useful code references

| Existing path | Purpose |
| --- | --- |
| `docs/corbanu-product-spec.md`, section “Campaign Tracker — LOCAL PILOT CANDIDATE” | Existing capture/sync/replay scope to reuse, not replace |
| `qa/campaign-tracker/2026-09-06/README.md` | Receiving main's qualification limits and remaining pilot gates |
| `codex-rs/tasknode-session/src/lib.rs` | Existing boundary to inspect/reuse; not a claim that this feature already exists |
| `docs/features/tasknode.md` | Existing boundary to inspect/reuse; not a claim that this feature already exists |
| `codex-rs/app-server/src/request_processors/token_usage_replay.rs` | Existing boundary to inspect/reuse; not a claim that this feature already exists |

Research/artifact paths in the sprint table are planned files, not existing implementations. External harness/research locations must be identified by the owner; absence blocks that sprint rather than licensing a guessed rebuild.

## Native lifecycle and upstream-touch record

- Observed planning checkout: `/Volumes/CorbanuDrive/Corbanu/CorbanuTerminal`, branch `recovery/corbanu-drive-2026-09-02`, HEAD `6f8f4ce46446e951437e1ce7e7c4a526e5b9e546`.
- Historical source only. Publication uses main baseline `3cec54d9917b776bedaffb586247d7cd7c633df6`; the [scrum packet](../scrum-2026-09-10.md) records receiving state and gates. Future implementation coordinates remain unallocated.
- Canonical upstream: `https://github.com/openai/codex.git`; verified upstream SHA for this future candidate: unresolved, blocks code readiness.
- This bounded scope produces research/QA artifacts, not a new scheduler/provider/runtime. Runtime touch is not applicable to artifact-only sprints. Any experiment executable must have its isolated workspace and exact commands approved at its preceding contract gate.
- Follow [upstream integration](../upstream-integration.md). Shared migrations, module registration, lockfiles and config schemas require serial ownership.

## Sprint execution map

All records belong to the single feature **PF-72**. Dependencies are hard prerequisites, not suggestions. All results are pending.

| Sprint | Record / outcome | Depends on | Planned output | Evidence |
| --- | --- | --- | --- | --- |
| PF-72-S01 | [Existing tracker and outcome contract](../../sprints/current/portfolio-campaign-roi/pf-72-s01-existing-tracker-and-outcome-contract.md) | none | docs/research/campaign-roi/contract.md | pending |
| PF-72-S02 | [Offline attribution and import fixture](../../sprints/current/portfolio-campaign-roi/pf-72-s02-offline-attribution-and-import-fixture.md) | PF-72-S01, PF-60-S01 | docs/research/campaign-roi/fixture.md | pending |
| PF-72-S03 | [Native integration decision and pilot handoff](../../sprints/current/portfolio-campaign-roi/pf-72-s03-native-integration-decision-and-pilot-handoff.md) | PF-72-S02 | docs/research/campaign-roi/decision.md | pending |

## Acceptance flows

| Flow | Starting state / action | Expected result and pass criterion |
| --- | --- | --- |
| Success | Approved inputs; execute bounded sprint sequence | A campaign outcome links to work evidence, review status and measured/unknown costs without counting the same agent work twice. |
| Failure/cancel | Missing input, rejected gate or interrupted work | Missing tracker access, unverified outcomes, duplicated tasks and unknown spend remain visibly unscored. |
| Recovery/resume | Reopen from a recorded checkpoint | Re-import the same redacted batch without duplication; interrupted imports resume from stable IDs. |

## Implementation sequence

1. **Existing tracker and outcome contract:** A complete field map identifies existing fields, gaps and human acceptance; no new dashboard is assumed.
2. **Offline attribution and import fixture:** Duplicate re-import leaves totals unchanged; unknown cost and unverified revenue cannot create numeric ROI.
3. **Native integration decision and pilot handoff:** The recommendation reduces a measured workflow burden without a second source of task truth.

## Automated evidence

- Every sprint: `python3 docs/plans/check.py; python3 docs/sprints/check.py` from repository root; `git diff --check`.
- Focused commands and artifact checks are explicit in each sprint. A newly specified selector must be registered, shown to run nonzero tests and resolved before readiness; an empty test filter is not evidence.
- Evidence includes input/candidate digests, command, timestamps, exit status, expected versus actual result and limitations. Checklists alone do not prove behavior.
- All evidence fields remain pending until the actual final tree or artifact is checked; this planning pass does not execute the future sprint tests.

## True-TUI evidence

Not applicable to this plan's document-only outputs: no Terminal UI behavior is changed or claimed qualified. Paper/tabletop recovery is required by the sprint checks. Any later runtime implementation must add final-candidate actual-key success/failure/recovery/resume tests before activation; the documentary no-op is not a waiver for that implementation.

## Live-repository applicability

| Repository | Applicability for this scope | Checkout/base | Result |
| --- | --- | --- | --- |
| TensorCash | No runtime change; research/document-only scope | Not applicable | No product proof claimed |
| Isometric Game | No runtime change; research/document-only scope | Not applicable | No product proof claimed |

Release-level live-repository gates remain intact regardless of this plan's bounded applicability.

## Human acceptance

Proposed accountable owner reviews the exact artifact/candidate; Travis or named delegate decides product scope. Named tester, date, digest, observed flows, result and evidence are **pending**. Agent review is advisory and never substitutes for required human acceptance.

## Documentation

Research outputs stay under the declared research/QA paths. Finished-feature docs change only after qualification and name the verified candidate and product-spec citation. Unimplemented possibilities remain proposals.

## Dependencies, decisions, and blockers

- September 10: PF-75 owns operational task/run/machine mapping and native recovery/capture qualification. Reuse `tasknode_cmd.rs`, `tasknode-session/src/tracker.rs`, `chatwidget/campaign_tracker.rs` and the September 7 tracker QA. S01 can inventory fields before accounting; only S02's spend-derived attribution requires PF-60-S01. See the [native boundary](../call-followup-2026-09-10.md#native-task-node-boundary).

- Alex identifies the existing tracker/schema and allowed export; use PF-60 accounting definitions before any spend-derived ROI.
- Before activation: reconcile canonical worktree/release, inherited ledger errors and active slots. The [three-initiative operating-model transition](../scrum-2026-09-10.md#operating-model) is separate from this draft publication; follow the receiving policy until it lands.
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
