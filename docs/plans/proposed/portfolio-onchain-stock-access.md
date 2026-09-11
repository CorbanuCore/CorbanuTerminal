---
title: "On-chain stock access and broker-wrapper feasibility"
status: draft
change_class: product-initiative
priority: P2
owner: "Alex Good (product/commercial lead, proposed)"
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
  heading: "Brokerage-agent layer — TO INTEGRATE"
  requirement_excerpt: "Default access is read-only."
implementation_worktrees: []
---

# On-chain stock access and broker-wrapper feasibility

Policy: repository-root `AGENTS.md`. Lifecycle: [plans](../index.md).
Portfolio: [source coverage, sequencing and human capacity](../portfolio-2026-09-09.md).
This is a fully specified **draft decision plan**, not an executable assignment or a shipped-feature claim.

## Activation record

- Status: draft; active slot: none; target release/date: not promised.
- Planning authority: user's 2026-09-09 request. Product activation: Travis Good, pending.
- Proposed priority/owners are recommendations, not new assignments or reordered P0 commitments.
- Gate: Qualified review of venue terms, jurisdiction, eligibility, custody, instrument rights and marketing is required before live enablement.

## User pain

Alex's stock access idea needs verified instruments, fees, eligibility and operating responsibilities before a wrapper can be specified.

## Product intent and ideal flow

Produce a current evidence matrix and read-only paper journey for candidate stock-access routes, then obtain a product/legal gate.
Entry is the first sprint's approved contract; success, failure and return-use are defined below.

## Product linkage

- Exact heading: **Brokerage-agent layer — TO INTEGRATE** in [the product spec](../../corbanu-product-spec.md).
- Requirement excerpt: “Default access is read-only.”
- Feature: **PF-68**; outcome: Produce a current evidence matrix and read-only paper journey for candidate stock-access routes, then obtain a product/legal gate.
- Source: transcript discussion at 01:00:25–01:03:37; see portfolio for attribution caveats.
- This citation supplies product context, not authorization for additional scope. Only evidence and a decision package are planned. Any resulting live product requires an explicit scope/spec decision and implementation amendment.

## Scope

Latest-call follow-up (00:00–02:09 and 45:00–46:07): PF-68-S01 adds an
entity/individual eligibility and onboarding-requirements matrix, available
instrument inventory, minimums, fees and liquidity, all verified from current
primary sources. The call's numbers and vendor names are leads, not facts.
No account/entity is opened. PF-70 can design and test a non-trading index API
without this plan; any later tradability/distribution claim needs its diligence.

- In: Venue and instrument diligence; Read-only user and risk journey; Wrapper economics and launch gate.
- Out: Opening accounts/entities, trading, signing, deposits, evading eligibility or claiming tokenized exposure equals shareholder ownership.

## Invariants

- No credential, proprietary strategy, customer log or financial record is disclosed by default; only approved redacted/public/synthetic inputs leave the host.
- A model finding or a passing test is not authority to publish, spend, trade, change permissions, merge or release.
- Unknowns remain unknown; negative findings and no-go decisions count as useful research, not fabricated completion.
- No product-runtime mutation, account action or live integration is included in the research/pilot decision unless a separately activated sprint explicitly authorizes it.

## Ownership and implementation worktrees

| Proposed accountable role | Worktree | Branch | Base | Scope |
| --- | --- | --- | --- | --- |
| Alex Good (product/commercial lead, proposed) | UNALLOCATED | UNALLOCATED | UNALLOCATED | PF-68; one sprint at a time |
| Jim Ricketts, receiving integrator (confirm) | UNALLOCATED | UNALLOCATED | UNALLOCATED | Source reconciliation and combined evidence |

No worktree or worker slot is reserved. Before readiness, confirm actual owner, exact repository/branch/40-character base, literal write scopes including registration/tests/manifests, and receiving gate. Cross-repository work requires separate explicit coordinates and authority.

## Useful code references

| Existing path | Purpose |
| --- | --- |
| `docs/corbanu-product-spec.md` | Existing boundary to inspect/reuse; not a claim that this feature already exists |
| `docs/features/wallet-plan.md` | Existing boundary to inspect/reuse; not a claim that this feature already exists |

Research/artifact paths in the sprint table are planned files, not existing implementations. External harness/research locations must be identified by the owner; absence blocks that sprint rather than licensing a guessed rebuild.

## Native lifecycle and upstream-touch record

- Observed planning checkout: `/Volumes/CorbanuDrive/Corbanu/CorbanuTerminal`, branch `recovery/corbanu-drive-2026-09-02`, HEAD `6f8f4ce46446e951437e1ce7e7c4a526e5b9e546`.
- Historical source only. Publication uses main baseline `3cec54d9917b776bedaffb586247d7cd7c633df6`; the [scrum packet](../scrum-2026-09-10.md) records receiving state and gates. Future implementation coordinates remain unallocated.
- Canonical upstream: `https://github.com/openai/codex.git`; verified upstream SHA for this future candidate: unresolved, blocks code readiness.
- This bounded scope produces research/QA artifacts, not a new scheduler/provider/runtime. Runtime touch is not applicable to artifact-only sprints. Any experiment executable must have its isolated workspace and exact commands approved at its preceding contract gate.
- Follow [upstream integration](../upstream-integration.md). Shared migrations, module registration, lockfiles and config schemas require serial ownership.

## Sprint execution map

All records belong to the single feature **PF-68**. Dependencies are hard prerequisites, not suggestions. All results are pending.

| Sprint | Record / outcome | Depends on | Planned output | Evidence |
| --- | --- | --- | --- | --- |
| PF-68-S01 | [Venue and instrument diligence](../../sprints/current/portfolio-onchain-stock-access/pf-68-s01-venue-and-instrument-diligence.md) | none | docs/research/onchain-stock-access/venue-matrix.md | pending |
| PF-68-S02 | [Read-only user and risk journey](../../sprints/current/portfolio-onchain-stock-access/pf-68-s02-read-only-user-and-risk-journey.md) | PF-68-S01 | docs/research/onchain-stock-access/paper-journey.md | pending |
| PF-68-S03 | [Wrapper economics and launch gate](../../sprints/current/portfolio-onchain-stock-access/pf-68-s03-wrapper-economics-and-launch-gate.md) | PF-68-S02 | docs/research/onchain-stock-access/decision.md | pending |

## Acceptance flows

| Flow | Starting state / action | Expected result and pass criterion |
| --- | --- | --- |
| Success | Approved inputs; execute bounded sprint sequence | Each candidate route identifies the actual instrument, rights, fees, liquidity, custody, redemption and eligibility evidence. |
| Failure/cancel | Missing input, rejected gate or interrupted work | Unavailable instrument, restricted user or missing redemption path is an explicit exclusion, not bypassed. |
| Recovery/resume | Reopen from a recorded checkpoint | Interrupted paper research resumes from dated instrument/source records and refreshes stale terms. |

## Implementation sequence

1. **Venue and instrument diligence:** Every candidate is classified supported, excluded or unknown with dated evidence.
2. **Read-only user and risk journey:** The user can distinguish market exposure from ownership and identify all unresolved exit risks.
3. **Wrapper economics and launch gate:** A go does not authorize live transactions; exact next permissions and blocked questions are listed.

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

- Qualified review of venue terms, jurisdiction, eligibility, custody, instrument rights and marketing is required before live enablement.
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
