---
title: "Repeatable cross-harness quality, runtime and cost benchmarks"
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
  heading: "Accountable sequencing"
  requirement_excerpt: "correctness, end-to-end runtime, and spend are recorded, and a threshold regression blocks release."
implementation_worktrees: []
---

# Repeatable cross-harness quality, runtime and cost benchmarks

Policy: repository-root `AGENTS.md`. Lifecycle: [plans](../index.md).
Portfolio: [source coverage, sequencing and human capacity](../portfolio-2026-09-09.md).
This is a fully specified **draft delivery plan**, not an executable assignment or a shipped-feature claim.

## Activation record

- Status: draft; active slot: none; target release/date: not promised.
- Planning authority: user's 2026-09-09 request. Product activation: Travis Good, pending.
- Proposed priority/owners are recommendations, not new assignments or reordered P0 commitments.
- Gate: Approve campaign spend/runtime caps and current comparator versions; existing release thresholds remain binding.

## User pain

One-off tests do not reveal recurring quality, wall-time and cost regressions across Corbanu, Hermes, Kilo and model choices.

## Product intent and ideal flow

Run a pinned, reproducible campaign with blinded outcome scoring, comparable settings, failures retained and an explicit release decision.
Entry is the first sprint's approved contract; success, failure and return-use are defined below.

## Product linkage

- Exact heading: **Accountable sequencing** in [the product spec](../../corbanu-product-spec.md).
- Requirement excerpt: “correctness, end-to-end runtime, and spend are recorded, and a threshold regression blocks release.”
- Feature: **PF-63**; outcome: Run a pinned, reproducible campaign with blinded outcome scoring, comparable settings, failures retained and an explicit release decision.
- Source: transcript discussion at 01:49:00–01:51:57 and 01:58:15–02:02:41; see portfolio for attribution caveats.
- This citation supplies product context, not authorization for additional scope. Any new schema/public contract is reviewed and approved in S01 before implementation readiness.

## Scope

- In: Comparable campaign specification; Resumable runner and evidence adapters; First bounded campaign and gate rehearsal.
- Out: Declaring a harness best without comparable evidence, changing production models, or relaxing existing every-three-releases benchmark policy.

## Invariants

- No credential, proprietary strategy, customer log or financial record is disclosed by default; only approved redacted/public/synthetic inputs leave the host.
- A model finding or a passing test is not authority to publish, spend, trade, change permissions, merge or release.
- Unknowns remain unknown; negative findings and no-go decisions count as useful research, not fabricated completion.
- Extend existing product/native contracts; do not create parallel usage, orchestration, auth or release authorities.

## Ownership and implementation worktrees

| Proposed accountable role | Worktree | Branch | Base | Scope |
| --- | --- | --- | --- | --- |
| Jim Ricketts (proposed) | UNALLOCATED | UNALLOCATED | UNALLOCATED | PF-63; one sprint at a time |
| Jim Ricketts, receiving integrator (confirm) | UNALLOCATED | UNALLOCATED | UNALLOCATED | Source reconciliation and combined evidence |

No worktree or worker slot is reserved. Before readiness, confirm actual owner, exact repository/branch/40-character base, literal write scopes including registration/tests/manifests, and receiving gate. Cross-repository work requires separate explicit coordinates and authority.

## Useful code references

| Existing path | Purpose |
| --- | --- |
| `docs/benchmarking/index.md` | Existing boundary to inspect/reuse; not a claim that this feature already exists |
| `docs/benchmarking/coding-tests.md` | Existing boundary to inspect/reuse; not a claim that this feature already exists |
| `scripts/native-provider-tui-benchmark` | Existing boundary to inspect/reuse; not a claim that this feature already exists |
| `scripts/hammer-reduction-benchmark` | Existing boundary to inspect/reuse; not a claim that this feature already exists |
| `benchmarks/README.md` | Existing boundary to inspect/reuse; not a claim that this feature already exists |
| `benchmarks/coding/runner.py` and `benchmarks/coding/tests/test_runner.py` | Existing boundary to inspect/reuse; not a claim that this feature already exists |

Research/artifact paths in the sprint table are planned files, not existing implementations. External harness/research locations must be identified by the owner; absence blocks that sprint rather than licensing a guessed rebuild.

## Native lifecycle and upstream-touch record

- Observed planning checkout: `/Volumes/CorbanuDrive/Corbanu/CorbanuTerminal`, branch `recovery/corbanu-drive-2026-09-02`, HEAD `6f8f4ce46446e951437e1ce7e7c4a526e5b9e546`.
- Historical source only. Publication uses main baseline `3cec54d9917b776bedaffb586247d7cd7c633df6`; the [scrum packet](../scrum-2026-09-10.md) records receiving state and gates. Future implementation coordinates remain unallocated.
- Canonical upstream: `https://github.com/openai/codex.git`; verified upstream SHA for this future candidate: unresolved, blocks code readiness.
- Inspect the existing references above; keep product-owned implementation behind thin native adapters, preserving event/replay/schema contracts. Exact files, compatibility tests and retain/adapt/remove dispositions are filled at S01 handoff before code readiness.
- Follow [upstream integration](../upstream-integration.md). Shared migrations, module registration, lockfiles and config schemas require serial ownership.

## Sprint execution map

All records belong to the single feature **PF-63**. Dependencies are hard prerequisites, not suggestions. All results are pending.

| Sprint | Record / outcome | Depends on | Planned output | Evidence |
| --- | --- | --- | --- | --- |
| PF-63-S01 | [Comparable campaign specification](../../sprints/current/portfolio-recurring-benchmarks/pf-63-s01-comparable-campaign-specification.md) | none | docs/research/recurring-benchmarks/contract.md | pending |
| PF-63-S02 | [Resumable runner and evidence adapters](../../sprints/current/portfolio-recurring-benchmarks/pf-63-s02-resumable-runner-and-evidence-adapters.md) | PF-63-S01 | benchmarks/coding/runner.py and benchmarks/coding/tests/test_runner.py | pending |
| PF-63-S03 | [First bounded campaign and gate rehearsal](../../sprints/current/portfolio-recurring-benchmarks/pf-63-s03-first-bounded-campaign-and-gate-rehearsal.md) | PF-63-S02 | qa/portfolio/recurring-benchmarks/qualification.md | pending |

## Acceptance flows

| Flow | Starting state / action | Expected result and pass criterion |
| --- | --- | --- |
| Success | Approved inputs; execute bounded sprint sequence | Each campaign records task, harness/model version, repetitions, correctness, end-to-end time, failures and measured/unknown cost. |
| Failure/cancel | Missing input, rejected gate or interrupted work | Timeouts, unavailable models, unsupported tasks and missing spend data remain in the denominator and report. |
| Recovery/resume | Reopen from a recorded checkpoint | Resume interrupted campaigns without counting duplicate attempts as new successful runs. |

## Implementation sequence

1. **Comparable campaign specification:** An evaluator can score identical artifacts without knowing the producing model; required release catalog remains included.
2. **Resumable runner and evidence adapters:** An interrupted/resumed fixture campaign has the same logical run set as an uninterrupted campaign.
3. **First bounded campaign and gate rehearsal:** A deliberately regressed candidate is rejected; every accepted score has reproducible evidence.

## Automated evidence

- Every sprint: `python3 docs/plans/check.py; python3 docs/sprints/check.py` from repository root; `git diff --check`.
- Focused commands and artifact checks are explicit in each sprint. A newly specified selector must be registered, shown to run nonzero tests and resolved before readiness; an empty test filter is not evidence.
- Evidence includes input/candidate digests, command, timestamps, exit status, expected versus actual result and limitations. Checklists alone do not prove behavior.
- The benchmark authority is `benchmarks/README.md`: a due cycle requires both three-way competitive live-repo evidence and the full coding task/model matrix. Missing spend/runtime/route makes it incomplete and release-blocking. Read the receiving release ledger at activation; this planning publication neither resets the cadence nor asserts a new bootstrap state.

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

- Approve campaign spend/runtime caps and current comparator versions; existing release thresholds remain binding.
- Before activation: reconcile canonical worktree/release, inherited ledger errors and active slots. The [three-initiative operating-model transition](../scrum-2026-09-10.md#operating-model) is separate from this draft publication; follow the receiving policy until it lands.
- Before each next sprint: predecessor evidence accepted/archived, its handoff resolves concrete inputs and any newly discovered decisions; otherwise stop.
- Cross-plan IDs in the table must exist and be completed in the receiving ledger. A draft dependency does not activate either plan.
- Before dispatch: agreed budget, named human acceptance owner, exact scope and privacy permissions. No capacity or deadline is assumed from hardware ownership alone.

## Release linkage

Target release and `qa/release/<version>/` are unresolved. Required final-tree tests, true-TUI, applicable live repositories, named human acceptance and due benchmarks block release.

## Completion

- [ ] All sprint outputs are accepted and linked; no expected evidence is invented.
- [ ] Scope, inputs, dependencies, exact allocation and authority are current.
- [ ] Automated/artifact checks and applicable actual-key tests pass.
- [ ] Named human accepts the exact candidate or decision package.
- [ ] Any implementation has finished docs, release linkage and all required release/benchmark evidence.
- [ ] Remaining implementation ideas are separately gated; no hard gate is silently waived.
