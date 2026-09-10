---
title: "Plan and model-serving backend reconciliation"
status: draft
change_class: product-initiative
priority: P1
owner: "Jim Ricketts (proposed)"
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
  heading: "Corbanu API — TO BUILD"
  requirement_excerpt: "The paying wallet owns one balance shared by its API keys."
implementation_worktrees: []
---

# Plan and model-serving backend reconciliation

Policy: repository-root `AGENTS.md`. Lifecycle: [plans](../index.md).
Portfolio: [source coverage, sequencing and human capacity](../portfolio-2026-09-09.md).
This is a fully specified **draft decision plan**, not an executable assignment or a shipped-feature claim.
“Plan” in this historical discussion title identifies the backend lineage.
The receiving spec's target is **Corbanu API**, with wallet-funded balance and
keys, not restored Plan tiers or entitlements. Audit legacy surfaces as migration
discrepancies; this draft does not authorize migration, deletion or deployment.

## Activation record

- Status: draft; active slot: none; target release/date: not promised.
- Planning authority: user's 2026-09-09 request. Product activation: Travis Good, pending.
- Proposed priority/owners are recommendations, not new assignments or reordered P0 commitments.
- Gate: An operator must identify the authoritative deployment and authorize any production inspection; secret values stay outside the report.

## User pain

Public Terminal, private gateway, deployed service and unfinished branches can disagree about catalog, credential ownership and attribution.

## Product intent and ideal flow

Produce a verified current-state map, offline contract fixtures and a prioritized, separately approvable repair handoff.
Entry is the first sprint's approved contract; success, failure and return-use are defined below.

## Product linkage

- Exact heading: **Corbanu API — TO BUILD** in [the product spec](../../corbanu-product-spec.md).
- Requirement excerpt: “The paying wallet owns one balance shared by its API keys.”
- Feature: **PF-61**; outcome: Produce a verified current-state map, offline contract fixtures and a prioritized, separately approvable repair handoff.
- Source: transcript discussion at 01:58:15–02:02:41; see portfolio for attribution caveats.
- This citation supplies product context, not authorization for additional scope. Only evidence and a decision package are planned. Any resulting live product requires an explicit scope/spec decision and implementation amendment.

## Scope

- In: Repository and deployment truth map; Offline compatibility fixture pack; Repair scope and release decision.
- Out: Deployments, production billing changes, buying inference, merging old worktrees or claiming per-wallet xAPI is live.

## Invariants

- No credential, proprietary strategy, customer log or financial record is disclosed by default; only approved redacted/public/synthetic inputs leave the host.
- A model finding or a passing test is not authority to publish, spend, trade, change permissions, merge or release.
- Unknowns remain unknown; negative findings and no-go decisions count as useful research, not fabricated completion.
- No product-runtime mutation, account action or live integration is included in the research/pilot decision unless a separately activated sprint explicitly authorizes it.

## Ownership and implementation worktrees

| Proposed accountable role | Worktree | Branch | Base | Scope |
| --- | --- | --- | --- | --- |
| Jim Ricketts (proposed) | UNALLOCATED | UNALLOCATED | UNALLOCATED | PF-61; one sprint at a time |
| Jim Ricketts, receiving integrator (confirm) | UNALLOCATED | UNALLOCATED | UNALLOCATED | Source reconciliation and combined evidence |

No worktree or worker slot is reserved. Before readiness, confirm actual owner, exact repository/branch/40-character base, literal write scopes including registration/tests/manifests, and receiving gate. Cross-repository work requires separate explicit coordinates and authority.

## Useful code references

| Existing path | Purpose |
| --- | --- |
| `docs/features/wallet-plan.md` | Existing boundary to inspect/reuse; not a claim that this feature already exists |
| `codex-rs/tui/src/chatwidget/wallet_http.rs` | Existing boundary to inspect/reuse; not a claim that this feature already exists |
| `codex-rs/tui/src/chatwidget/wallet_usage.rs` | Existing boundary to inspect/reuse; not a claim that this feature already exists |

CorbanuPlan: src/app.ts, src/models.ts, src/xapi.ts, src/usage.ts, tests/xapi-routing.test.ts, tests/accounting.test.ts (read-only during this plan).

## Native lifecycle and upstream-touch record

- Observed planning checkout: `/Volumes/CorbanuDrive/Corbanu/CorbanuTerminal`, branch `recovery/corbanu-drive-2026-09-02`, HEAD `6f8f4ce46446e951437e1ce7e7c4a526e5b9e546`.
- Historical source only. Publication uses main baseline `3cec54d9917b776bedaffb586247d7cd7c633df6`; the [scrum packet](../scrum-2026-09-10.md) records receiving state and gates. Future implementation coordinates remain unallocated.
- Canonical upstream: `https://github.com/openai/codex.git`; verified upstream SHA for this future candidate: unresolved, blocks code readiness.
- This bounded scope produces research/QA artifacts, not a new scheduler/provider/runtime. Runtime touch is not applicable to artifact-only sprints. Any experiment executable must have its isolated workspace and exact commands approved at its preceding contract gate.
- Follow [upstream integration](../upstream-integration.md). Shared migrations, module registration, lockfiles and config schemas require serial ownership.

## Sprint execution map

All records belong to the single feature **PF-61**. Dependencies are hard prerequisites, not suggestions. All results are pending.

| Sprint | Record / outcome | Depends on | Planned output | Evidence |
| --- | --- | --- | --- | --- |
| PF-61-S01 | [Repository and deployment truth map](../../sprints/current/portfolio-plan-backend-reconciliation/pf-61-s01-repository-and-deployment-truth-map.md) | none | docs/research/plan-backend-reconciliation/current-state.md | pending |
| PF-61-S02 | [Offline compatibility fixture pack](../../sprints/current/portfolio-plan-backend-reconciliation/pf-61-s02-offline-compatibility-fixture-pack.md) | PF-61-S01 | docs/research/plan-backend-reconciliation/contracts.md | pending |
| PF-61-S03 | [Repair scope and release decision](../../sprints/current/portfolio-plan-backend-reconciliation/pf-61-s03-repair-scope-and-release-decision.md) | PF-61-S02 | docs/research/plan-backend-reconciliation/decision.md | pending |

## Acceptance flows

| Flow | Starting state / action | Expected result and pass criterion |
| --- | --- | --- |
| Success | Approved inputs; execute bounded sprint sequence | Every advertised model and payment/auth/usage boundary maps to a repository commit, deployment evidence or an explicit unknown. |
| Failure/cancel | Missing input, rejected gate or interrupted work | Unreachable backend, expired credential and mismatched catalog become reproducible findings, not inferred fixes. |
| Recovery/resume | Reopen from a recorded checkpoint | Resume an interrupted audit from its evidence manifest; stale observations are dated and rerun before decisions. |

## Implementation sequence

1. **Repository and deployment truth map:** Every boundary has an evidence pointer and owner; no branch is treated as deployed merely because code exists.
2. **Offline compatibility fixture pack:** All six failure classes map to existing tests or explicitly specified missing tests with expected values.
3. **Repair scope and release decision:** A named owner can select one repair with evidence and acceptance criteria; unresolved architecture decisions prevent readiness.

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

- An operator must identify the authoritative deployment and authorize any production inspection; secret values stay outside the report.
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
