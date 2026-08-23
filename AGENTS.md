# Corbanu Terminal development policy

| Field | Value |
| --- | --- |
| Policy version | 1.1 |
| Updated | 2026-08-23 |
| Policy owner | Lead developer, as assigned in the [product roles table](docs/corbanu-product-spec.md#ownership-and-decision-rights) |
| Product authority | The decision roles in the product specification |
| Amendment rule | Changes to product scope or hard release gates require the product decision process defined in the product specification |

Corbanu Terminal is a Codex fork designed for agentic trading. This file contains
only repository-wide Corbanu rules. Rust implementation guidance is scoped to
[`codex-rs/AGENTS.md`](codex-rs/AGENTS.md).

Use the `corbanu-terminal-development` skill for product behavior, planning,
interactive QA, documentation, benchmarks, and releases.

## Canonical sources

Each rule has one owner. Linked files may collect evidence or provide fields;
they must not restate policy.

| Concern | Canonical source |
| --- | --- |
| Product outcomes, status, sequencing, and decision roles | [Product specification](docs/corbanu-product-spec.md) |
| Change classification and repository-wide release rules | This file |
| Active-plan limit and lifecycle | [Plan process](plans/README.md) |
| Plan evidence fields | [Plan template](plans/PLAN_TEMPLATE.md) |
| Competitive benchmark cadence, method, and ledger | [Benchmark tracker](benchmarks/README.md) |
| Shipped user guidance | `docs/` |
| Release-candidate evidence and human sign-off | `qa/release/<version>/` |
| Rust implementation conventions | [`codex-rs/AGENTS.md`](codex-rs/AGENTS.md) |

The product specification defines **what**. An active plan defines **how** for a
product initiative. Documentation describes **what is finished**. QA and
benchmark artifacts prove the claims.

## Change classes

Classify work before editing. When uncertain, use the higher class.

| Class | Definition | Required record |
| --- | --- | --- |
| **Routine** | Process text, internal cleanup, or implementation work that cannot change user-visible behavior, security, money, data disclosure, persistent state, or an external contract. | Task or change description; focused checks as applicable. |
| **Bounded fix** | Restores already-authorized behavior without adding a user goal or changing a security, financial, data, persistence, or compatibility boundary. | Product-spec heading and a change record in the issue, PR, or release evidence. |
| **Product initiative** | Adds or materially changes a user goal, interaction, authorization boundary, financial/data flow, persistent state, compatibility surface, or multiple-worktree workflow. | Active plan governed by `plans/README.md`. |
| **Release** | Packages accepted routine work, bounded fixes, and product initiatives into a versioned candidate. It does not consume an active-plan slot. | `qa/release/<version>/` release record. |

Urgent reliability and security repairs may proceed as bounded fixes while two
product initiatives are active. They may not disguise new product scope, and
they do not bypass release gates. Any change to authorization, vault access,
financial action, or protected-data disclosure is a product initiative even
when reported as a bug.

Every bounded fix or product initiative must cite the exact product-spec heading
and a short requirement excerpt. Generated Markdown anchors are navigational,
not identity; heading text plus the excerpt is the durable reference. If the
product specification does not authorize the outcome, obtain a product decision
before implementation.

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
- A named human tester must sign the affected release-candidate flows after
  automated and TUI qualification. If no tester is available, the release
  waits.

Use the repository's [$test-tui skill](.codex/skills/test-tui/SKILL.md).

## Live-codebase qualification

Use TensorCash for systems and layer-one workflows and Isometric Game for
visual and interactive workflows.

| Repository | Canonical origin | Path input |
| --- | --- | --- |
| TensorCash | `https://github.com/agtico/tensorcash.git` | `CORBANU_TENSORCASH_REPO` or an exact path recorded in the plan/release record |
| Isometric Game | `https://github.com/goodalexander/isometricgame.git` | `CORBANU_ISOMETRIC_REPO` or an exact path recorded in the plan/release record |

Never encode a contributor's home directory as repository policy. Resolve and
record the path and base commit before testing.

Feature-level acceptance uses every applicable repository; the plan explains
why a repository is not applicable. Release-level qualification is broader: its
suite must include at least one real Corbanu TUI workflow in **each** default
repository, even when an individual feature touches only one.

Create disposable test worktrees from recorded base commits. Chaotic edits are
allowed only in those disposable worktrees, never in the canonical checkout.

## Release gate

The release owner named in `qa/release/<version>/` is accountable for collecting
evidence, auditing change classifications, and stopping shipment. Classification
disputes go to product authority; product authority cannot convert missing
evidence into a pass.

A release may ship only when its record shows:

1. included work is classified and linked to the product specification;
2. the final tree passes applicable automated, integration, and snapshot tests;
3. affected interactive flows pass true-TUI QA with keys sent;
4. the release suite passes in both default live repositories;
5. a named human tester accepts the affected flows;
6. shipped-feature documentation matches the candidate; and
7. the [competitive benchmark gate](benchmarks/README.md) is passing when due.

A missing required artifact, failed human acceptance, P0 security finding, or
due benchmark that is failed or incomplete blocks shipment. There is no waiver
for those hard gates. Development and repair work may continue while a gate is
pending; publishing the release may not.

## Documentation

Apply the “Steve Jobs review” test: make every user page clear, focused, and
useful enough to present directly to a user.

- Begin with the pain being solved, then explain the finished user flow.
- Feature docs describe only behavior verified in a shipped build or accepted
  release candidate. Put unfinished implementation work in active plans and raw
  evidence under `qa/release/<version>/`.
- Each feature doc cites the exact product-spec heading and short requirement
  excerpt it implements; an optional anchor is only a navigation aid.
- The product specification is the sole product-doc exception: it may describe
  future outcomes when their status is explicit.

## Resolved ambiguities

| Previous conflict | Resolution |
| --- | --- |
| Active-plan cap versus urgent fixes | The limit applies to product initiatives. Routine work and bounded fixes do not consume a slot; they cannot introduce new scope. |
| Active-plan cap versus a release train | A release record aggregates work and does not consume an active-plan slot. |
| Finished-only docs versus a roadmap in `docs/` | Feature docs are finished-only; the status-labeled product specification is the sole exception. |
| Corbanu docs versus the inherited “docs live elsewhere” rule | That upstream rule is not adopted. Root `docs/` is the official Corbanu documentation tree. |
| Feature-specific repository applicability versus testing both repositories | A feature uses applicable repos; every release suite includes a generic real workflow in both. |
| Automated tests versus true-TUI proof | Both are required for affected interactive behavior; neither substitutes for the other. |
| Formatting after tests | Fix and formatting tools run before final affected tests and TUI qualification. |
| Fixed local paths versus cross-platform support | Repository locations are inputs recorded by the plan/release record, never personal absolute paths in policy. |
| Exact Markdown anchors versus durable product linkage | Heading text plus a requirement excerpt identifies the product decision; anchors are optional navigation. |
| Missing competitor infrastructure versus benchmark success | A due run without all three auditable lanes is incomplete and blocks shipment, as does a Corbanu failure. Competitor task failure on a functioning lane is a valid result. |
