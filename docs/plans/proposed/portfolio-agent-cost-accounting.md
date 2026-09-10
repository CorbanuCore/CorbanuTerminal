---
title: "Unified agent cost and usage accounting"
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
  heading: "Product measurement"
  requirement_excerpt: "No commercial performance numbers have been supplied."
implementation_worktrees: []
---

# Unified agent cost and usage accounting

Policy: repository-root `AGENTS.md`. Lifecycle: [plans](../index.md).
Portfolio: [source coverage, sequencing and human capacity](../portfolio-2026-09-09.md).
This is a fully specified **draft delivery plan**, not an executable assignment or a shipped-feature claim.

## Activation record

- Status: draft; active slot: none; target release/date: not promised.
- Planning authority: user's 2026-09-09 request. Product activation: Travis Good, pending.
- Proposed priority/owners are recommendations, not new assignments or reordered P0 commitments.
- Gate: Approve the cost vocabulary, retention window, currency/price source and historical unknown policy; reconcile current branch before selecting the next migration number.

## User pain

Parent/child agents and different providers make total spend hard to explain; missing prices must not look like free work.

## Product intent and ideal flow

Inspect one run and its descendants, distinguish measured tokens, estimated cost, billed cost and Corbanu API balance, then reopen the same totals after restart.
Entry is the first sprint's approved contract; success, failure and return-use are defined below.

## Product linkage

- Exact heading: **Product measurement** in [the product spec](../../corbanu-product-spec.md).
- Requirement excerpt: “No commercial performance numbers have been supplied.”
- Feature: **PF-60**; outcome: Inspect one run and its descendants, distinguish measured tokens, estimated cost, billed cost and Corbanu API balance, then reopen the same totals after restart.
- Source: transcript discussion at 01:58:15–02:02:41; see portfolio for attribution caveats.
- This citation supplies product context, not authorization for additional scope. Any new schema/public contract is reviewed and approved in S01 before implementation readiness.

## Scope

- In: Accounting contract and golden fixtures; Idempotent usage persistence and replay; Inspectable run and campaign totals; Cost-accounting acceptance and handoff.
- Out: Changing prices, rebilling historical customers, collecting prompts, restoring legacy Plan allowances, or silently converting allowance to cash.

## Invariants

- No credential, proprietary strategy, customer log or financial record is disclosed by default; only approved redacted/public/synthetic inputs leave the host.
- A model finding or a passing test is not authority to publish, spend, trade, change permissions, merge or release.
- Unknowns remain unknown; negative findings and no-go decisions count as useful research, not fabricated completion.
- Extend existing product/native contracts; do not create parallel usage, orchestration, auth or release authorities.

## Ownership and implementation worktrees

| Proposed accountable role | Worktree | Branch | Base | Scope |
| --- | --- | --- | --- | --- |
| Jim Ricketts (proposed) | UNALLOCATED | UNALLOCATED | UNALLOCATED | PF-60; one sprint at a time |
| Jim Ricketts, receiving integrator (confirm) | UNALLOCATED | UNALLOCATED | UNALLOCATED | Source reconciliation and combined evidence |

No worktree or worker slot is reserved. Before readiness, confirm actual owner, exact repository/branch/40-character base, literal write scopes including registration/tests/manifests, and receiving gate. Cross-repository work requires separate explicit coordinates and authority.

## Useful code references

| Existing path | Purpose |
| --- | --- |
| `codex-rs/state/migrations/0041_provider_request_cache_usage.sql` | Existing boundary to inspect/reuse; not a claim that this feature already exists |
| `codex-rs/app-server/src/request_processors/token_usage_replay.rs` | Existing boundary to inspect/reuse; not a claim that this feature already exists |
| `codex-rs/tui/src/chatwidget/usage.rs` | Existing boundary to inspect/reuse; not a claim that this feature already exists |
| `codex-rs/tui/src/token_usage.rs` | Existing boundary to inspect/reuse; not a claim that this feature already exists |

Research/artifact paths in the sprint table are planned files, not existing implementations. External harness/research locations must be identified by the owner; absence blocks that sprint rather than licensing a guessed rebuild.

## Native lifecycle and upstream-touch record

- Observed planning checkout: `/Volumes/CorbanuDrive/Corbanu/CorbanuTerminal`, branch `recovery/corbanu-drive-2026-09-02`, HEAD `6f8f4ce46446e951437e1ce7e7c4a526e5b9e546`.
- Historical source only. Publication uses main baseline `3cec54d9917b776bedaffb586247d7cd7c633df6`; the [scrum packet](../scrum-2026-09-10.md) records receiving state and gates. Future implementation coordinates remain unallocated.
- Canonical upstream: `https://github.com/openai/codex.git`; verified upstream SHA for this future candidate: unresolved, blocks code readiness.
- Inspect the existing references above; keep product-owned implementation behind thin native adapters, preserving event/replay/schema contracts. Exact files, compatibility tests and retain/adapt/remove dispositions are filled at S01 handoff before code readiness.
- Follow [upstream integration](../upstream-integration.md). Shared migrations, module registration, lockfiles and config schemas require serial ownership.

## Sprint execution map

All records belong to the single feature **PF-60**. Dependencies are hard prerequisites, not suggestions. All results are pending.

| Sprint | Record / outcome | Depends on | Planned output | Evidence |
| --- | --- | --- | --- | --- |
| PF-60-S01 | [Accounting contract and golden fixtures](../../sprints/current/portfolio-agent-cost-accounting/pf-60-s01-accounting-contract-and-golden-fixtures.md) | none | docs/research/agent-cost-accounting/contract.md | pending |
| PF-60-S02 | [Idempotent usage persistence and replay](../../sprints/current/portfolio-agent-cost-accounting/pf-60-s02-idempotent-usage-persistence-and-replay.md) | PF-60-S01 | codex-rs/state/ (exact migration and runtime files allocated after S01) | pending |
| PF-60-S03 | [Inspectable run and campaign totals](../../sprints/current/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md) | PF-60-S02 | codex-rs/tui/src/chatwidget/usage.rs | pending |
| PF-60-S04 | [Cost-accounting acceptance and handoff](../../sprints/current/portfolio-agent-cost-accounting/pf-60-s04-cost-accounting-acceptance-and-handoff.md) | PF-60-S03 | qa/portfolio/agent-cost-accounting/qualification.md | pending |

## Acceptance flows

| Flow | Starting state / action | Expected result and pass criterion |
| --- | --- | --- |
| Success | Approved inputs; execute bounded sprint sequence | A root with two children has exactly three independently attributable request records, one nonduplicated aggregate and explicit unknown-cost markers. |
| Failure/cancel | Missing input, rejected gate or interrupted work | A duplicate event, interrupted request, missing price or denied provider produces no invented zero and no double charge. |
| Recovery/resume | Reopen from a recorded checkpoint | Kill after a recorded request, restart and reopen the run; totals and attribution agree without a manual database edit. |

## Implementation sequence

1. **Accounting contract and golden fixtures:** Every fixture has raw inputs, expected totals and provenance; no unknown value is rendered as zero.
2. **Idempotent usage persistence and replay:** Replay each fixture twice and after process restart; persisted and reconstructed totals equal the approved fixture exactly.
3. **Inspectable run and campaign totals:** The user can explain each displayed total using constituent requests without inspecting storage.
4. **Cost-accounting acceptance and handoff:** All cost flows pass on one recorded binary; unknown/estimated values remain visibly distinct.

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

- Approve the cost vocabulary, retention window, currency/price source and historical unknown policy; reconcile current branch before selecting the next migration number.
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
