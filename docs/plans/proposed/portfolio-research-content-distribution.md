---
title: "Stock research and content distribution workflow"
status: draft
change_class: product-initiative
priority: P2
owner: "Alex Good (research/editorial lead, proposed)"
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
  heading: "Discovery and activation"
  requirement_excerpt: "Corbanu Terminal is distributed through **Corbanu.com**, the newsletter and media property for on-chain stock ideas."
implementation_worktrees: []
---

# Stock research and content distribution workflow

Policy: repository-root `AGENTS.md`. Lifecycle: [plans](../index.md).
Portfolio: [source coverage, sequencing and human capacity](../portfolio-2026-09-09.md).
This is a fully specified **draft pilot plan**, not an executable assignment or a shipped-feature claim.

## Activation record

- Status: draft; active slot: none; target release/date: not promised.
- Planning authority: user's 2026-09-09 request. Product activation: Travis Good, pending.
- Proposed priority/owners are recommendations, not new assignments or reordered P0 commitments.
- Gate: Alex selects topic/audience; data rights, conflicts and financial-content review must be resolved before any public release.

- Additional activation gate: the current spec does not authorize this initiative's complete scope. Travis must approve the concrete outcome and a product-spec amendment before Terminal-plan activation, or explicitly move genuinely external editorial work into separately owned records. Reclassification must not evade existing WIP limits.

## User pain

Research, stock ideas and Terminal demonstrations can reinforce each other, but sourcing, claims and publication approval need clear ownership.

## Product intent and ideal flow

Produce one internally reviewed research-to-content packet with sources, counterthesis, reproducible supporting analysis and an explicit publish decision.
Entry is the first sprint's approved contract; success, failure and return-use are defined below.

## Product linkage

- Exact heading: **Discovery and activation** in [the product spec](../../corbanu-product-spec.md).
- Requirement excerpt: “Corbanu Terminal is distributed through **Corbanu.com**, the newsletter and media property for on-chain stock ideas.”
- Feature: **PF-74**; outcome: Produce one internally reviewed research-to-content packet with sources, counterthesis, reproducible supporting analysis and an explicit publish decision.
- Source: transcript discussion at Research/content discussion across the recording; 01:51:57–01:57:29 adoption context; see portfolio for attribution caveats.
- This citation supplies product context, not authorization for additional scope. The bounded pilot is an enabling/editorial workflow. Product changes or external actions need an explicit subsequent scope decision.

## Scope

- In: Editorial brief and source contract; One evidence-backed draft packet; Human review and distribution decision.
- Out: Publishing, investment recommendations tailored to the user, scraping restricted data without rights, growth spam or claims about unshipped features.

## Invariants

- No credential, proprietary strategy, customer log or financial record is disclosed by default; only approved redacted/public/synthetic inputs leave the host.
- A model finding or a passing test is not authority to publish, spend, trade, change permissions, merge or release.
- Unknowns remain unknown; negative findings and no-go decisions count as useful research, not fabricated completion.
- No product-runtime mutation, account action or live integration is included in the research/pilot decision unless a separately activated sprint explicitly authorizes it.

## Ownership and implementation worktrees

| Proposed accountable role | Worktree | Branch | Base | Scope |
| --- | --- | --- | --- | --- |
| Alex Good (research/editorial lead, proposed) | UNALLOCATED | UNALLOCATED | UNALLOCATED | PF-74; one sprint at a time |
| Jim Ricketts, receiving integrator (confirm) | UNALLOCATED | UNALLOCATED | UNALLOCATED | Source reconciliation and combined evidence |

No worktree or worker slot is reserved. Before readiness, confirm actual owner, exact repository/branch/40-character base, literal write scopes including registration/tests/manifests, and receiving gate. Cross-repository work requires separate explicit coordinates and authority.

## Useful code references

| Existing path | Purpose |
| --- | --- |
| `docs/corbanu-product-spec.md` | Existing boundary to inspect/reuse; not a claim that this feature already exists |
| `docs/benchmarking/index.md` | Existing boundary to inspect/reuse; not a claim that this feature already exists |

Research/artifact paths in the sprint table are planned files, not existing implementations. External harness/research locations must be identified by the owner; absence blocks that sprint rather than licensing a guessed rebuild.

## Native lifecycle and upstream-touch record

- Observed planning checkout: `/Volumes/CorbanuDrive/Corbanu/CorbanuTerminal`, branch `recovery/corbanu-drive-2026-09-02`, HEAD `6f8f4ce46446e951437e1ce7e7c4a526e5b9e546`.
- Historical source only. Publication uses main baseline `3cec54d9917b776bedaffb586247d7cd7c633df6`; the [scrum packet](../scrum-2026-09-10.md) records receiving state and gates. Future implementation coordinates remain unallocated.
- Canonical upstream: `https://github.com/openai/codex.git`; verified upstream SHA for this future candidate: unresolved, blocks code readiness.
- This bounded scope produces research/QA artifacts, not a new scheduler/provider/runtime. Runtime touch is not applicable to artifact-only sprints. Any experiment executable must have its isolated workspace and exact commands approved at its preceding contract gate.
- Follow [upstream integration](../upstream-integration.md). Shared migrations, module registration, lockfiles and config schemas require serial ownership.

## Sprint execution map

All records belong to the single feature **PF-74**. Dependencies are hard prerequisites, not suggestions. All results are pending.

| Sprint | Record / outcome | Depends on | Planned output | Evidence |
| --- | --- | --- | --- | --- |
| PF-74-S01 | [Editorial brief and source contract](../../sprints/current/portfolio-research-content-distribution/pf-74-s01-editorial-brief-and-source-contract.md) | none | docs/research/research-content-distribution/brief.md | pending |
| PF-74-S02 | [One evidence-backed draft packet](../../sprints/current/portfolio-research-content-distribution/pf-74-s02-one-evidence-backed-draft-packet.md) | PF-74-S01 | docs/research/research-content-distribution/draft-packet.md | pending |
| PF-74-S03 | [Human review and distribution decision](../../sprints/current/portfolio-research-content-distribution/pf-74-s03-human-review-and-distribution-decision.md) | PF-74-S02 | docs/research/research-content-distribution/decision.md | pending |

## Acceptance flows

| Flow | Starting state / action | Expected result and pass criterion |
| --- | --- | --- |
| Success | Approved inputs; execute bounded sprint sequence | Every material factual statement traces to a dated source; model opinion and uncertainty are visibly distinguished. |
| Failure/cancel | Missing input, rejected gate or interrupted work | Conflicting evidence, stale prices, unsupported claims or missing rights block the affected claim or publication. |
| Recovery/resume | Reopen from a recorded checkpoint | Resume from a claim/source ledger; refreshed sources create a new draft version rather than silently replacing approved copy. |

## Implementation sequence

1. **Editorial brief and source contract:** The brief permits a negative conclusion and does not promise a preferred stock outcome.
2. **One evidence-backed draft packet:** An independent reviewer can follow each material claim to evidence and reproduce any calculation.
3. **Human review and distribution decision:** A named human accepts exact content or it stays unpublished; no agent claims that review equals authorization.

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

- Alex selects topic/audience; data rights, conflicts and financial-content review must be resolved before any public release.
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
