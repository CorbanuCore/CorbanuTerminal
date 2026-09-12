---
title: "2. Accounting — unified agent cost and usage"
status: active
change_class: product-initiative
priority: P1
owner: "Codex accounting contract lane; Travis Good accountable"
parallel_sprint_limit: 1
integration_owner: "Codex management; Travis Good accepts the contract"
activation_authority: "Travis Good"
activation_basis: "Travis selected accounting September 11 and requested an Astra High subagent after the main planning merge; contract and synthetic fixtures first, no runtime enablement"
target_release: "TBD"
deadline: "TBD"
created: 2026-09-09
updated: 2026-09-11
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "Product measurement"
  requirement_excerpt: "No commercial performance numbers have been supplied."
implementation_worktrees:
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911"
    branch: "integrate/management-workstreams-20260911"
    base_commit: "c33d47f6ccd00a64fcb05a057472d8ca9f0139d4"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-pf60-s01-20260911"
    branch: "workstream/accounting-pf60-s01-20260911"
    base_commit: "c33d47f6ccd00a64fcb05a057472d8ca9f0139d4"
---

# 2. Accounting — unified agent cost and usage

Policy: repository-root `AGENTS.md`. Lifecycle: [plans](../index.md).
Portfolio: [source coverage, sequencing and human capacity](../portfolio-2026-09-09.md).
This is active workstream 2. S01 is accepted/archived; S02's first isolated
native-state journal increment is reviewed and integrated locally. Production wiring, live collection
and runtime human acceptance remain gated. The first checkout is the receiving
manager; the second is the reused independent accounting worker.
Dispatch follows the verified main merge; an allocation is not a running-agent claim.

Current receiving result: reviewed S01 candidate `1c5978690` is integrated
locally at `0415a00dc3`; 48 fixture tests pass on the combined tree. The
[manager handoff](../workstream-manager-handoff-2026-09-11.md#accounting-decision)
records Travis's explicit approval of the v1 defaults in this task. Do not
re-ask that decision. The reviewed [S02 handoff](../../research/agent-cost-accounting/s02-allocation.md)
resolves exact first-increment boundaries; S01 is now archived and S02 is selected.
Native candidate `3f39d7a65` passed independent Astra High review and was received
at `c33d47f6c`; all 202 state tests passed there. The manager owns the next
same-sprint allocation; this is not full S02 or user-interface readiness.

## Activation record

- Status: active; slot 2/3; target release/date not promised.
- Authority: Travis's September 11 workstream selection and Astra High kickoff request.
- Codex owns contract preparation; Travis approved the v1 defaults in this task on September 11, including vocabulary, retention/replay, exact USD estimates and unknown handling.
- S01 handoff and archive are complete. S02 starts with test-only native-state persistence, not billing, a collector, production migration or live prices.

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

| Accountable role | Worktree | Branch | Base | Scope |
| --- | --- | --- | --- | --- |
| Astra High accounting exact pricing | `/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-pf60-s01-20260911` | `workstream/accounting-pf60-s01-20260911` | `c33d47f6ccd00a64fcb05a057472d8ca9f0139d4` | PF-60-S02 four-file test-only exact quotation increment |
| Codex management / Travis acceptance | Manager checkout in front matter | Recorded above | Recorded above | Shared plan and receiving evidence |

S02 is allocated to one Astra High subagent. The next literal four-file scope is
in the sprint and [exact-pricing allocation](../../research/agent-cost-accounting/exact-pricing-allocation.md);
the manager serially delegates only the child module declaration in accounting.rs.
No production migrations, API/Core wiring, manifests or other state writers are
allocated. The idle worker fast-forwards to
the reviewed receiving allocation before dispatch; its historical branch name
does not change sprint ownership. Manager owns shared plans/integration.

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
- Historical source only. This amendment receives main `295aed26e53b17f919f7199ae1c9748b1b1250ba`; the [current handoff](../main-workstreams-2026-09-11.md) supersedes the earlier scrum allocation. Preserve main's Corbanu API balance semantics, not legacy Plan entitlements.
- Canonical upstream: `https://github.com/openai/codex.git`; locally verified common ancestor `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`. Source seams, cached upstream identity and unintegrated newer tip are recorded in the reviewed S02 handoff; no upstream upgrade qualified.
- Inspect the existing references above; keep product-owned implementation behind thin native adapters, preserving event/replay/schema contracts. Exact files, compatibility tests and retain/adapt/remove dispositions are filled at S01 handoff before code readiness.
- Follow [upstream integration](../upstream-integration.md). Shared migrations, module registration, lockfiles and config schemas require serial ownership.

## Sprint execution map

All records belong to the single feature **PF-60**. Dependencies are hard prerequisites, not suggestions. S01 contract/fixture review and defaults approval are recorded; runtime evidence remains pending.

| Sprint | Record / outcome | Depends on | Planned output | Evidence |
| --- | --- | --- | --- | --- |
| PF-60-S01 | [Accounting contract and golden fixtures](../../sprints/archive/portfolio-agent-cost-accounting/pf-60-s01-accounting-contract-and-golden-fixtures.md) | none | docs/research/agent-cost-accounting/contract.md | Accepted local contract/fixtures and reviewed handoff; archived |
| PF-60-S02 | [Idempotent usage persistence and replay](../../sprints/current/portfolio-agent-cost-accounting/pf-60-s02-idempotent-usage-persistence-and-replay.md) | PF-60-S01 | Native journal then exact quotation, still test-only | Journal reviewed/integrated; 202 state tests pass; pricing allocated, full S02 pending |
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

Travis approved S01's v1 defaults on September 11 in this task; the linked manager
handoff records the exact decision and candidate. Native/runtime human testing
is still pending. Agent review never substitutes for that later acceptance.

## Documentation

Research outputs stay under the declared research/QA paths. Finished-feature docs change only after qualification and name the verified candidate and product-spec citation. Unimplemented possibilities remain proposals.

## Dependencies, decisions, and blockers

- Approved defaults are recorded; reconcile the receiving branch and finish exact adapter/schema allocation before selecting the next migration number.
- S01 is accepted/archived and defaults approved. S02's first native journal increment is reviewed/integrated; manager allocates the next bounded increment, with production/live scope still OFF.
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
