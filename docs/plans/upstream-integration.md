# Upstream integration contract

This is the canonical engineering process for keeping Corbanu changes separable
from upstream Codex. Product linkage: **Product principles** — “Maintain
continuous Codex parity without removing Corbanu-specific behavior.” Travis
Good approved these integration requirements on 2026-08-27. They define required
work and evidence, not a claim that the current fork is upgrade-qualified.

## Architecture boundaries

- Keep Corbanu policy and feature logic in focused modules or crates. Extend
  existing vault, security-policy, protocol, and context types before adding
  another authority or persistence system.
- Keep upstream-facing adapters thin: translate native inputs/events into typed
  product contracts and return decisions/results. Do not scatter product policy
  across transport, tool handlers, compaction, and UI branches.
- Reuse native thread creation, cancellation, mailbox, history, and resume
  lifecycles. Product roles, exact-runtime selection, and review policy compose
  above that lifecycle; they do not introduce a second agent scheduler.
- Separate richer Corbanu requests from provider-reserved wire schemas. Provider
  encoding is an adapter decision, not a model-authored failed probe and retry.
- Preserve lineage at existing context/persistence seams and centralize
  protected-action decisions. A text marker or model classification cannot
  substitute for host authority. Core policy cannot depend on TUI implementation.
- Prefer the smallest seam that supports the feature. A generic plugin framework,
  wholesale rewrite, or second agent scheduler is not implied by this rule.

## Required upstream-touch record

Every new or amended implementation sprint records the following in its plan
and links its applicable rows. Bounded fixes record the same fields in their
change/evidence record when they touch upstream-owned code. Documentation-only
work may record not applicable with a reason.

| Field | Required content |
| --- | --- |
| Baseline | Canonical upstream repository URL and verified full commit SHA; fork base and candidate SHA recorded separately |
| Footprint | Literal changed files/directories, owning feature/sprint, and accountable integration owner |
| Boundary | Native interface used, product-owned module, dependency direction, and why each upstream edit is necessary |
| Compatibility | Provider wire, permission, history, persistence, cancellation, and resume invariants affected, or justified non-applicability |
| Verification | Exact contract/regression commands, expected assertions, platform applicability, and evidence locations |
| Upgrade handling | Upstream candidate SHA, merge/rebase disposition of each adapter, removed patches, unresolved conflicts, and qualification results |

Do not mistake a fork HEAD, package version, remote name, or date for the
upstream baseline. If the checkout is shallow or ancestry is unavailable, mark
the upstream SHA unresolved. Resolve it with repository history and maintainer
evidence before affected new implementation becomes ready; do not guess it.
Record file/symbol drift discovered during allocation before executing a draft.

Already-completed sprints keep their historical evidence. Existing qualification
work may continue collecting tests, but an unresolved upstream baseline or missing
adapter proof cannot be represented as a passing upstream qualification. New
implementation must satisfy this contract; no retroactive pass is inferred.

## Parallel ownership

Use the existing [sprint concurrency contract](../sprints/index.md#bounded-concurrency).
An integration owner lands shared interfaces, module registrations, dependency
manifests/lockfiles, and shared test registration before independent consumers.
Include those files in write scopes; separate implementation directories alone
do not establish independence. Shared-file discoveries require serialization
or a revised dependency before edits, including collisions across plans.

Browser backend construction may overlap content and confidentiality work after
its shared contracts and early harness are complete. Browser facade integration
is a later join, not permission for two lanes to edit ingress or tool schemas.

## Qualification of an upstream update

The integration owner records both the current fork baseline and the exact
proposed upstream commit. Assess the update in a disposable worktree, never by
experimentally rebasing a contributor's active checkout. Preserve user changes.
For each footprint row, retain, adapt, or remove the fork patch with a reason.

Run formatting/fixes before the final candidate evidence. The qualification
record includes:

1. Adapter contract tests for native tool schemas, provider routing, child
   lifecycle, cancellation, and history isolation where affected.
2. Provenance through compaction/memory/resume; current authority epochs, grants,
   revocation, and fail-closed behavior through every affected adapter.
3. A complete Core run plus affected crate/integration/snapshot tests using the
   repository test workflow. Separate unrelated existing failures from passes;
   failures remain unresolved until triaged and the applicable gate passes.
4. Applicable Linux/macOS/Windows credential and browser backend evidence;
   unsupported backends must prove explicit denial, not an unsafe fallback.
5. True-TUI success, cancel/failure, recovery, and resume in both default live
   repositories for the integrated release candidate, including affected native
   delegation and remote-terminal flows.
6. Existing human, documentation, release, and due benchmark evidence required
   by their canonical owners. This process does not waive or duplicate those gates.

Record actual commands, environment, commits, results, and artifact paths. A
clean merge or successful build alone is not compatibility proof. Missing
required evidence blocks upstream acceptance; it does not prevent isolated
diagnosis or preparation. This contract promises reviewable integration and
regression detection, not conflict-free upgrades.

## Reliability investigation boundaries

Distinguish terminal/SSH attachment, remote process lifetime, app-server
connectivity, and provider streaming before attributing reconnect symptoms to
upstream or the fork. Record exact remote binaries/configuration and timestamped,
redacted logs. Local desktop versions do not identify a remote Linux process.
Transport recovery must not widen authority or blindly replay side effects.

The current remote Linux/tmux report and diagnostic matrix are tracked in
`qa/reliability/2026-08-27-linux-tmux-reconnect.md`. Diagnosis remains separate
from security feature delivery; a future fix is classified under root policy
once its cause and affected boundary are known.
