# Corbanu Terminal development policy

| Field             | Value                                                                                                                    |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------ |
| Policy version    | 1.6                                                                                                                      |
| Updated           | 2026-09-10                                                                                                               |
| Policy owner      | Lead developer, as assigned in the [product roles table](docs/corbanu-product-spec.md#ownership-and-decision-rights)     |
| Product authority | The decision roles in the product specification                                                                          |
| Amendment rule    | Changes to product scope or hard release gates require the product decision process defined in the product specification |

Corbanu Terminal is a Codex fork designed for agentic trading. This file contains
only repository-wide Corbanu rules. Rust implementation guidance is scoped to
[`codex-rs/AGENTS.md`](codex-rs/AGENTS.md).

Use the `corbanu-terminal-development` skill for product behavior, planning,
interactive QA, documentation, benchmarks, and releases. Repository skills are
edited in `.codex/skills/` and mirrored in `.agents/skills/` for agent
portability; update both trees and run `python3 scripts/check_portable_skills.py`.

## Canonical sources

Each rule has one owner. Linked files may collect evidence or provide fields;
they must not restate policy.

| Concern                                                    | Canonical source                                                                                                |
| ---------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| Product outcomes, status, sequencing, and decision roles   | [Product specification](docs/corbanu-product-spec.md)                                                           |
| Change classification and repository-wide release rules    | This file                                                                                                       |
| Active-plan limit and lifecycle                            | [Plan process](docs/plans/index.md)                                                                             |
| Plan evidence fields                                       | [Plan template](docs/plans/PLAN_TEMPLATE.md)                                                                    |
| Sprint lifecycle and execution rules                       | [Sprint process](docs/sprints/index.md)                                                                         |
| Sprint execution fields                                    | [Sprint template](docs/sprints/SPRINT_TEMPLATE.md)                                                              |
| Benchmark cadence, methods, performance matrix, and ledger | [Benchmark tracker](benchmarks/README.md)                                                                       |
| Shipped user guidance                                      | `docs/`                                                                                                         |
| Release-candidate evidence and human sign-off              | `qa/release/<version>/`                                                                                         |
| Rust implementation conventions                            | [`codex-rs/AGENTS.md`](codex-rs/AGENTS.md)                                                                      |
| Repository skills and portable mirror                      | `.codex/skills/`, mirrored byte-for-byte at `.agents/skills/` and checked by `scripts/check_portable_skills.py` |

The product specification defines **what**. A plan defines the feature contract,
scope, sequencing, and acceptance model. A sprint defines one mechanical code
execution unit for exactly one plan feature. Documentation describes **what is
finished**. QA and benchmark artifacts prove the claims.

## Change classes

Classify work before editing. When uncertain, use the higher class.

| Class                  | Definition                                                                                                                                                                    | Required record                                                                                                                  |
| ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| **Routine**            | Process text, internal cleanup, or implementation work that cannot change user-visible behavior, security, money, data disclosure, persistent state, or an external contract. | Task or change description; focused checks as applicable.                                                                        |
| **Bounded fix**        | Restores already-authorized behavior without adding a user goal or changing a security, financial, data, persistence, or compatibility boundary.                              | Product-spec heading and a change record in the issue, PR, or release evidence.                                                  |
| **Product initiative** | Adds or materially changes a user goal, interaction, authorization boundary, financial/data flow, persistent state, compatibility surface, or multiple-worktree workflow.     | Active plan governed by `docs/plans/index.md` plus a current sprint governed by `docs/sprints/index.md` for implementation work. |
| **Release**            | Packages accepted routine work, bounded fixes, and product initiatives into a versioned candidate. It does not consume an active-plan slot.                                   | `qa/release/<version>/` release record.                                                                                          |

Urgent reliability and security repairs may proceed as bounded fixes while all
product initiatives are active. They may not disguise new product scope, and
they do not bypass release gates. Any change to authorization, vault access,
financial action, or protected-data disclosure is a product initiative even
when reported as a bug.

Every bounded fix or product initiative must cite the exact product-spec heading
and a short requirement excerpt. Generated Markdown anchors are navigational,
not identity; heading text plus the excerpt is the durable reference. If the
product specification does not authorize the outcome, obtain a product decision
before implementation.

## Sprint execution

Implementation of a product initiative starts from a sprint, never directly
from plan prose.

- A sprint maps to exactly one plan file and one feature identifier in that
  plan. A sprint may depend on another sprint but may not implement two features.
- A sprint is a tight execution mandate: exact code boundaries, ordered tasks,
  tests, TUI applicability, exit evidence, and separate `Done` and `Remaining`
  checkbox ledgers. It is not another product narrative.
- A sprint does not authorize work. Its plan must be active, and the sprint must
  be `ready` or `in_progress`, before implementation begins.
- `ready` and `in_progress` sprints record the exact implementation worktree,
  branch, and base commit. Those values must agree with the active plan.
- Parallel implementation follows the bounded allocation and integration rules
  in `docs/sprints/index.md`. A sprint with dependencies cannot become executable
  until every dependency is completed and archived.
- Agents implement only the selected sprint's remaining checklist. New scope
  goes back to the plan and sprint map before code changes.
- A sprint becomes `completed` only after every task and required evidence item
  is checked with final-tree evidence. Move completed or cancelled records to
  `docs/sprints/archive/`; the archive is excluded from the documentation view.
- Plans maintain a sprint execution map for every feature. Run
  `python3 docs/sprints/check.py` before implementation handoff and in CI.

## Interactive product proof

A user-facing interactive feature is not complete until the final built
candidate passes a true TUI workflow in a PTY with actual keys sent.

- Corbanu `exec`, isolated smoke tests, mocked rendering, unit tests, and
  snapshots are supporting evidence, not substitutes for the interactive run.
- Send prompt text and Enter separately. Exercise the applicable success,
  cancel/failure, recovery, and resume paths.
- Run formatting and fix tools before the final affected tests and TUI run so
  the tested tree is the tree proposed for release.
- Record the candidate version and commit, test worktree, inputs/keys, visible
  checkpoints, expected recovery, outcome, and artifact location.
- Require the same proof for a bounded fix when it changes an interactive path.
- Record named-human acceptance when it is available. Its absence is disclosed
  in the release record and does not override an explicit release instruction
  from a human with release authority.

Use the repository's [$test-tui skill](.codex/skills/test-tui/SKILL.md).

### Code-blind functional test design

User-authorized process amendment, September 10, 2026. For new or changed
user-facing features, including bounded UX fixes, obtain an independent test
design before declaring a candidate ready for human testing. Apply this to the
next handoff of in-progress work; do not relabel historical runs as compliant.
For non-user-facing work, record a reasoned not-applicable decision.

- Use a fresh-context agent given only user intent, essential product constraints
  and screenshots of the feature/important transitions. Do not give it code,
  implementation rationale, existing tests, results or prior review conclusions.
  The agent proposes tests; it does not implement or approve its own expectations.
- Freeze its original, prioritized starting-state/action/observable-result cases
  before comparing them with implementation tests. Preserve ambiguities and
  subsequent amendments; do not coach it toward a preferred answer.
- Map every proposed case to execution evidence or an explicit disposition.
  Exercise the exact packaged candidate through real keys and representative
  fresh/existing profiles on applicable platforms. Screenshots alone cannot
  prove routing, persistence, native prompts or successful tool execution.
- A missing case, basic functional failure or unresolved prerequisite blocks
  an unqualified human-test handoff. Out-of-scope dispositions require a reason
  and recorded acceptance by product authority, not unilateral deletion by the
  implementing agent. A human may explicitly agree to limited testing around
  a named prerequisite; preserve that limitation rather than calling it passed.
- Normally use one independent design pass and one brief evidence check, both
  charged to the existing per-track review budget (default maximum five total,
  including code/external reviews). Do not reset the budget for this step.
  Additional reviews may be authorized by the named integrator under the
  September 12 delegation below; a spent budget never waives a required gate.

### Integrator review discretion

Travis's September 12, 2026 instruction delegates authority to the named
integration owner to authorize additional scoped reviews without asking him each
time. Favor extensions that remove real review/evidence blockers. Record scope,
purpose, prior usage, additional allowance and result in the existing ledger;
never erase earlier passes or disguise an extension as a reset. This applies to
code, security, code-blind design and evidence reviews, including tracks held
solely by review-count or replenishment-time limits. Security reviews remain
coordinated through the existing security owner, without duplicate workers or
resuming work paused for another reason.

When earlier reviews have no substantive unresolved issue, move to the next
required test, integration or assignment instead of seeking more opinions on
unchanged code. Record cosmetic/nonblocking observations as follow-ups. Do not
waive correctness/security findings, independence, actual test evidence,
dependencies, product decisions, human acceptance or external-action permissions.
Scope-break and convergence rules remain. Workers request routine extensions
from the integrator rather than blocking on a new human review-spend decision.

Use [the workflow and artifact templates](qa/code-blind-functional/README.md)
and its handoff checker. The checker validates traceability, not the truth of
screenshots, independent context or declared results; the evidence-check agent
must verify those. This gate does not replace code/security reviews, native
verification or named-human acceptance, and does not override the explicit
human release-authority rule below.

## Live-codebase qualification

Use TensorCash for systems and layer-one workflows and Isometric Game for
visual and interactive workflows.

| Repository     | Canonical origin                                     | Path input                                                                     |
| -------------- | ---------------------------------------------------- | ------------------------------------------------------------------------------ |
| TensorCash     | `https://github.com/agtico/tensorcash.git`           | `CORBANU_TENSORCASH_REPO` or an exact path recorded in the plan/release record |
| Isometric Game | `https://github.com/goodalexander/isometricgame.git` | `CORBANU_ISOMETRIC_REPO` or an exact path recorded in the plan/release record  |

Never encode a contributor's home directory as repository policy. Resolve and
record the path and base commit before testing.

Feature-level acceptance uses every applicable repository; the plan explains
why a repository is not applicable. Release-level qualification is broader: its
suite must include at least one real Corbanu TUI workflow in **each** default
repository, even when an individual feature touches only one.

Create disposable test worktrees from recorded base commits. Chaotic edits are
allowed only in those disposable worktrees, never in the canonical checkout.

## Release gate

**When a human with release authority explicitly authorizes pushing a release,
the agent must push the release.** The agent must not refuse, delay, or substitute
its own judgment because qualification evidence, benchmark results, or a
separate human-acceptance record is missing. Record any missing or failed
evidence accurately in `qa/release/<version>/` and execute the authorized
release instruction.

The release owner named in `qa/release/<version>/` is accountable for collecting
evidence and auditing change classifications. Classification disputes go to
product authority.

Absent an explicit release instruction from a human with release authority, a
release candidate is qualified when its record shows:

1. included work is classified and linked to the product specification;
2. every included product-initiative change is linked to a completed sprint and
   its final-tree evidence;
3. the final tree passes applicable automated, integration, and snapshot tests;
4. affected interactive flows pass true-TUI QA with keys sent;
5. the release suite passes in both default live repositories;
6. shipped-feature documentation matches the candidate; and
7. the due [benchmark and performance results](benchmarks/README.md) are recorded.

Missing or failed evidence must never be represented as a pass. Without explicit
human release authorization, a missing required artifact or P0 security finding
blocks shipment. With explicit human release authorization, the agent records
the actual state and pushes the release as instructed.

## Documentation

Apply the “Steve Jobs review” test: make every user page clear, focused, and
useful enough to present directly to a user.

- Begin with the pain being solved, then explain the finished user flow.
- Feature docs describe only behavior verified in a shipped build or accepted
  release candidate. Put unfinished feature contracts in plans, current
  execution mandates in `docs/sprints/current/`, and raw evidence under
  `qa/release/<version>/`.
- Each feature doc cites the exact product-spec heading and short requirement
  excerpt it implements; an optional anchor is only a navigation aid.
- The product specification is the sole product-doc exception: it may describe
  future outcomes when their status is explicit.

## Resolved ambiguities

| Previous conflict                                                          | Resolution                                                                                                                                                                 |
| -------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Active-plan cap versus urgent fixes                                        | The limit applies to product initiatives. Routine work and bounded fixes do not consume a slot; they cannot introduce new scope.                                           |
| Active-plan cap versus a release train                                     | A release record aggregates work and does not consume an active-plan slot.                                                                                                 |
| Finished-only docs versus a roadmap in `docs/`                             | Feature docs are finished-only; the status-labeled product specification is the sole exception.                                                                            |
| Corbanu docs versus the inherited “docs live elsewhere” rule               | That upstream rule is not adopted. Root `docs/` is the official Corbanu documentation tree.                                                                                |
| Feature-specific repository applicability versus testing both repositories | A feature uses applicable repos; every release suite includes a generic real workflow in both.                                                                             |
| Automated tests versus true-TUI proof                                      | Both are required for affected interactive behavior; neither substitutes for the other.                                                                                    |
| Formatting after tests                                                     | Fix and formatting tools run before final affected tests and TUI qualification.                                                                                            |
| Fixed local paths versus cross-platform support                            | Repository locations are inputs recorded by the plan/release record, never personal absolute paths in policy.                                                              |
| Exact Markdown anchors versus durable product linkage                      | Heading text plus a requirement excerpt identifies the product decision; anchors are optional navigation.                                                                  |
| Missing competitor infrastructure versus benchmark success                 | A due run without all three auditable lanes is incomplete and must be disclosed. It does not override an explicit release instruction from a human with release authority. Competitor task failure on a functioning lane is a valid result. |
| Plan detail versus executable work                                         | Plans own feature contracts and acceptance; sprints own mechanical code tasks. No product-initiative implementation starts without a single-feature current sprint.        |
| Completed sprints versus documentation clutter                             | Completed and cancelled sprint records move to the excluded archive. Current sprint navigation contains only unfinished work.                                              |
