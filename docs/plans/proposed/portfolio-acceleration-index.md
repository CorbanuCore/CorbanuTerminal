---
title: "Index-creation API, replayability and creator ecosystem"
status: draft
change_class: product-initiative
priority: P2
owner: "Alex Good (index owner, proposed)"
parallel_sprint_limit: 1
integration_owner: "Jim Ricketts (proposed; confirmation required)"
activation_authority: "Travis Good"
activation_basis: "September 10 call follow-up approved for planning updates; API delivery and creator economics activation remain pending"
target_release: "TBD"
deadline: "TBD"
created: 2026-09-09
updated: 2026-09-10
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "First-party backtesting skill — TO BUILD"
  requirement_excerpt: "Record data version, code, parameters, model, and environment for replay."
implementation_worktrees: []
---

# Index-creation API, replayability and creator ecosystem

Policy: repository-root `AGENTS.md`. Lifecycle: [plans](../index.md).
Portfolio: [source coverage, sequencing and human capacity](../portfolio-2026-09-09.md).
This is a **draft phased product plan**, not an executable assignment or a shipped-feature claim. Its stable filename retains the earlier acceleration-index provenance.

## Activation record

- Status: draft; active slot: none; target release/date: not promised.
- Planning authority: user's 2026-09-09 request. Product activation: Travis Good, pending.
- Proposed priority/owners are recommendations, not new assignments or reordered P0 commitments.
- Gate: Alex identifies the existing index prototype, repositories and input rights. S01 freezes the API contract; S03 resolves product authority, implementation coordinates and the spec amendment before S04 delivery. Creator economics remains a later separate decision.

## User pain

A single paper index is narrower than Alex's requested API for creating user-defined indexes. We need a reusable, recoverable API built on his existing pipeline, with honest replay evidence and a separately gated creator ecosystem.

## Product intent and ideal flow

Inspect the existing prototype, define a general-purpose create-index API, qualify replay using one acceleration-index example, and deliver a disabled-by-default API candidate after the product decision. Then evaluate creator claims, fee sharing and leaderboards as an optional sequential phase.
Entry is the first sprint's approved contract; success, failure and return-use are defined below.

## Product linkage

- Exact heading: **First-party backtesting skill — TO BUILD** in [the product spec](../../corbanu-product-spec.md).
- Requirement excerpt: “Record data version, code, parameters, model, and environment for replay.”
- Feature: **PF-70**; outcome: a recoverable index-creation API with explicit input/version/replay provenance; creator economics requires its own go/no-go.
- Source: earlier call 02:28:15–02:31:12; latest call 02:09–06:16. See the [September 10 amendment](../call-followup-2026-09-10.md).
- The replay citation supplies context, not authorization for a general index service or creator payouts. The additional context is **Corbanu API — TO BUILD**, “Usage debits the balance using an explicit versioned price”. Alex/Travis must approve the concrete index scope and amend the product spec before S04 readiness; existing inference prices do not establish index or creator fees.

## Scope

- In: Existing prototype and API contract; replayability/paper validation; product decision; flag-gated non-trading API candidate; optional creator ownership/economics/leaderboard decision.
- Out: Live trading, tokens, creator payouts or creator implementation, inventing holdings/weights/fees, public data upload, claiming neutrality without a defined hedge, or treating determinism as regulatory clearance.
- Creator implementation is not hidden in S04. A go decision at S05 must add bounded implementation/qualification sprints and explicit financial authority before any code or payout work.

## Invariants

- No credential, proprietary strategy, customer log or financial record is disclosed by default; only approved redacted/public/synthetic inputs leave the host.
- A model finding or a passing test is not authority to publish, spend, trade, change permissions, merge or release.
- Unknowns remain unknown; negative findings and no-go decisions count as useful research, not fabricated completion.
- No product-runtime mutation, account action or live integration is included in the research/pilot decision unless a separately activated sprint explicitly authorizes it.

## Ownership and implementation worktrees

| Proposed accountable role | Worktree | Branch | Base | Scope |
| --- | --- | --- | --- | --- |
| Alex Good (index owner, proposed) | UNALLOCATED | UNALLOCATED | UNALLOCATED | PF-70; one sprint at a time |
| Jim Ricketts, receiving integrator (confirm) | UNALLOCATED | UNALLOCATED | UNALLOCATED | Source reconciliation and combined evidence |

No worktree or worker slot is reserved. Before readiness, confirm actual owner, exact repository/branch/40-character base, literal write scopes including registration/tests/manifests, and receiving gate. Cross-repository work requires separate explicit coordinates and authority.

## Useful code references

| Existing path | Purpose |
| --- | --- |
| `docs/corbanu-product-spec.md` | Existing boundary to inspect/reuse; not a claim that this feature already exists |
| Alex-owned index prototype, SEC/transcript cache and price pipeline | Location, owner, commit and rights unresolved; S01 must identify them, not reconstruct a guessed duplicate |

Research/artifact paths in the sprint table are planned files, not existing implementations. External harness/research locations must be identified by the owner; absence blocks that sprint rather than licensing a guessed rebuild.

## Native lifecycle and upstream-touch record

- Observed planning checkout: `/Volumes/CorbanuDrive/Corbanu/CorbanuTerminal`, branch `recovery/corbanu-drive-2026-09-02`, HEAD `6f8f4ce46446e951437e1ce7e7c4a526e5b9e546`.
- Historical source only. Publication uses main baseline `3cec54d9917b776bedaffb586247d7cd7c633df6`; the [scrum packet](../scrum-2026-09-10.md) records receiving state and gates. Future implementation coordinates remain unallocated.
- Canonical upstream: `https://github.com/openai/codex.git`; verified upstream SHA for this future candidate: unresolved, blocks code readiness.
- S01–S03 and S05 produce contracts, experiments and decisions. S04 is future API code delivery, blocked until the actual owning repository, upstream baseline where applicable, exact paths, authorized scope and nonzero test commands are recorded. No external code path is guessed in this draft.
- Follow [upstream integration](../upstream-integration.md). Shared migrations, module registration, lockfiles and config schemas require serial ownership.

## Sprint execution map

All records belong to the single feature **PF-70**. Dependencies are hard prerequisites, not suggestions. All results are pending.

| Sprint | Record / outcome | Depends on | Planned output | Evidence |
| --- | --- | --- | --- | --- |
| PF-70-S01 | [Existing prototype and index API contract](../../sprints/current/portfolio-acceleration-index/pf-70-s01-index-methodology-and-tradability.md) | none | docs/research/acceleration-index/methodology.md | pending |
| PF-70-S02 | [Replayability, paper index and stresses](../../sprints/current/portfolio-acceleration-index/pf-70-s02-historical-paper-index-and-stresses.md) | PF-70-S01 | docs/research/acceleration-index/paper-results.md | pending |
| PF-70-S03 | [Index product decision](../../sprints/current/portfolio-acceleration-index/pf-70-s03-index-product-decision.md) | PF-70-S02 | docs/research/acceleration-index/decision.md | pending |
| PF-70-S04 | [Flag-gated index API candidate](../../sprints/current/portfolio-acceleration-index/pf-70-s04-flag-gated-index-api-candidate.md) | PF-70-S03 | Candidate in S03-resolved repository; qa/portfolio/acceleration-index/api-candidate.md | pending |
| PF-70-S05 | [Creator ownership and economics decision](../../sprints/current/portfolio-acceleration-index/pf-70-s05-creator-ownership-and-economics-decision.md) | PF-70-S04 | docs/research/acceleration-index/creator-decision.md | pending |

## Acceptance flows

| Flow | Starting state / action | Expected result and pass criterion |
| --- | --- | --- |
| Success | Approved API contract and opt-in test candidate | A user creates and retrieves an index with pinned inputs, methodology, cost and supported replay guarantee; creator features remain disabled. |
| Failure/cancel | Flag off, invalid inputs, stale data or exhausted budget | No hidden index creation or charge; partial results and unsupported guarantees are explicit. Paper stresses include delisting, liquidity and hedge failure. |
| Recovery/resume | Interrupted request or missing packet | Stable request identity retrieves/reconciles the prior result without duplicate charge; missing/corrupt inputs fail visibly rather than silently changing membership. |

## Implementation sequence

1. **Prototype/API contract:** Inventory existing code and define input universe, request/result schema, status, identity, errors, cost limits and index versioning. Acceleration is one example, not the only supported thesis.
2. **Replay and paper evidence:** Pin input packet, data rights, prompt/model/runtime and outputs; independently measure reproducibility and methodology stresses. Byte-for-byte inference is an experiment, not a promise.
3. **Product decision:** Approve or stop API delivery; freeze scope, allowed replay claims, price/budget rules, flag/rollback boundary, exact repository/test paths and named acceptance.
4. **API candidate:** Reuse the prototype behind an off-by-default enforced flag; test creation, retrieval, cancellation, retry/restart, authorization and charge reconciliation without live trading.
5. **Optional creator decision:** Define ownership/claims, attribution, fee-sharing ledger, abuse/dispute handling and leaderboard semantics with synthetic fixtures. Seek qualified review; separately plan implementation only after approval.

## Automated evidence

- Every sprint: `python3 docs/plans/check.py; python3 docs/sprints/check.py` from repository root; `git diff --check`.
- Focused commands and artifact checks are explicit in each sprint. A newly specified selector must be registered, shown to run nonzero tests and resolved before readiness; an empty test filter is not evidence.
- Evidence includes input/candidate digests, command, timestamps, exit status, expected versus actual result and limitations. Checklists alone do not prove behavior.
- All evidence fields remain pending until the actual final tree or artifact is checked; this planning pass does not execute the future sprint tests.

## True-TUI evidence

S01–S03/S05 are artifact-only. S04 requires final-candidate API recovery evidence and actual-key success/failure/cancel/recovery/resume tests for any Terminal integration. An API-only scope must record why TUI is not applicable at S03; this is not a blanket waiver. Named-human acceptance and final artifact hashes are pending.

## Live-repository applicability

| Repository | Applicability for this scope | Checkout/base | Result |
| --- | --- | --- | --- |
| TensorCash | Resolve at S03 for API/Terminal integration; artifact-only phases do not change runtime | UNALLOCATED | pending applicability decision |
| Isometric Game | Resolve at S03 if an interactive integration is included | UNALLOCATED | pending applicability decision |

Release-level live-repository gates remain intact regardless of this plan's bounded applicability.

## Human acceptance

Proposed accountable owner reviews the exact artifact/candidate; Travis or named delegate decides product scope. Named tester, date, digest, observed flows, result and evidence are **pending**. Agent review is advisory and never substitutes for required human acceptance.

## Documentation

Research outputs stay under the declared research/QA paths. Finished-feature docs change only after qualification and name the verified candidate and product-spec citation. Unimplemented possibilities remain proposals.

## Dependencies, decisions, and blockers

- S01 must locate Alex's prototype and approve input/data rights; absence blocks discovery, not permission to rebuild it.
- No PF-68 prerequisite for API design or non-trading paper/replay work. Instrument access remains unknown until diligence; consume PF-68-S03 before any future tradability/distribution claim.
- PF-65 privacy and PF-70 reproducibility are different claims. Reuse relevant findings; no blanket dependency on private inference and no rental or byte-perfect guarantee from the transcript alone.
- S03 is a hard gate for S04 implementation, exact cross-repository scope and release/benchmark applicability. Failed or inconclusive replay narrows the claimed guarantee or stops the candidate; it cannot be relabeled as verified inference.
- S05 economics, creator implementation and public launch require separate decisions; no fee rate or regulatory conclusion is approved here.
- Before activation: reconcile canonical worktree/release, inherited ledger errors and active slots. The [three-initiative operating-model transition](../scrum-2026-09-10.md#operating-model) is separate from this draft publication; follow the receiving policy until it lands.
- Before each next sprint: predecessor evidence accepted/archived, its handoff resolves concrete inputs and any newly discovered decisions; otherwise stop.
- Cross-plan IDs in the table must exist and be completed in the receiving ledger. A draft dependency does not activate either plan.
- Before dispatch: agreed budget, named human acceptance owner, exact scope and privacy permissions. No capacity or deadline is assumed from hardware ownership alone.

## Release linkage

S04 produces a disabled candidate, not an automatic release. S03 must name the owning service/Terminal release record, applicable benchmarks and rollout authority before delivery readiness. S05 is a decision package only. Accepted documents, merged code and enabled financial functionality remain distinct states.

## Completion

- [ ] All sprint outputs are accepted and linked; no expected evidence is invented.
- [ ] Scope, inputs, dependencies, exact allocation and authority are current.
- [ ] Automated/artifact checks and applicable actual-key tests pass.
- [ ] Named human accepts the exact candidate or decision package.
- [ ] Any implementation has finished docs, release linkage and all required release/benchmark evidence.
- [ ] Remaining implementation ideas are separately gated; no hard gate is silently waived.
