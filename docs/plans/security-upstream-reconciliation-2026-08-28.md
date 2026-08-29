# Security planning: upstream reconciliation — 2026-08-28

This routine planning/process merge preserves accepted upstream work while adding
the architecture review's stronger requirements to explicit follow-up sprints.
It does not implement or qualify a runtime feature, activate another sprint,
or authorize a release. The [active plan](active/p0-security-levels.md) owns scope.

## Exact merge inputs

| Input | Revision |
| --- | --- |
| Published review branch, first parent | `1b98a170a00bd7152709d947b657e991109d588e` |
| Upstream `main`, second parent | `1bdc515bff48a4d9048dae7d06c6214e884265bc` |
| Output branch | `codex/security-architecture-review-20260828` |
| Implementation allocation retained from upstream | `feat/p0-security-levels`, base `7cc15ae0762664d6d01765de407329887da9f876`; exact worktree in the active plan |
| Frozen Permissive baseline (not the worktree base) | `3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb` |

The merge commit containing this record identifies the final tree. Remote `main`
is not a publication target of this reconciliation.

## Conflict decisions

- Preserve upstream runtime code, CI, scripts, benchmark scanner fixes, typed
  tmux driver and completed tmux work-package evidence unchanged.
- Preserve PF-15–22 and PF-13-S01 completed archives, their original execution
  orders 1–9 and all accompanying evidence. Remove only their redundant current
  planning copies; original records remain in the archive and Git history.
- Keep the review's full protected-mode scope, bounded-concurrency process and
  checker, source review, architecture requirements and unfinished feature sprints.
- Preserve upstream PF-13-S02 readiness and Mac allocation; no new readiness is
  inferred from this merge. Preparation/follow-up drafts remain unallocated.
- Retain upstream's PF-26 typed tmux contract: named Rust credential-boundary test,
  required Ubuntu lane, capture proxy/scanner, separate text/Enter sends,
  transport-only canary, zero retries, forbidden-surface scan and cleanup.
- Preserve the raw Opus assessment and historical review graph snapshots. Counts
  from those snapshots are not silently rewritten to describe this merged tree.

## New work, not retroactive acceptance

| New draft | Added requirement | First integration gate |
| --- | --- | --- |
| PF-13-S06 | Request/aggregate usage reservations and trusted metering | PF-13-S03 |
| PF-19-S02 | Dispatch/open-channel revocation fence; immediate restriction despite audit/finality failure | PF-41-S03 / PF-22-S02 |
| PF-20-S02 | Controller-owned authoritative state; tamper/rollback/crash protection | PF-41-S03 / PF-22-S02 |
| PF-21-S02 | Expanded independent compatibility controls and reviewed upstream drift | PF-22-S02 / PF-26-S01 |
| PF-22-S02 | Protected runtime integration, containment readiness and named upstream seams | PF-13-S03 and protected-surface consumers |

PF-15–18 require no duplicate foundation sprint. Historical S01 passes do not
prove new S02/S06 cases. The current index has 64 security records plus seven
unrelated Autoreview records; archives contain nine completed foundations plus
72 cancelled firewall records. The security graph has 73 nodes, longest total
chain 34 and remaining-only chain 31. Node counts are not a calendar estimate.

## Validation

Final merged-tree planning checks pass:

| Check | Result |
| --- | --- |
| Plan/sprint lifecycle | One active plan; 71 current / 81 archived records across the repository |
| Plan/sprint checker tests | 3 + 19 passed |
| Portable skills | 25 mirrored files match; 3 checker tests passed |
| Retained compatibility harness / canary scanner | 8 + 2 tests passed |
| Strict MkDocs and Prettier 3.5.3 | Passed after fixing a merged navigation line; expected excluded-archive notices remain informational |
| Graph/index/navigation audit | 73 unique security nodes; correct order and required integration edges; only PF-13-S02 ready |
| Upstream preservation | Runtime/CI/scripts/benchmark/research/tmux paths unchanged; 20 archive/evidence/baseline blobs match upstream |
| Historical evidence integrity | Existing planning snapshots unchanged; raw Opus SHA-256 unchanged |
| Whitespace | `git diff --check` passed |

Raw outputs and the graph/blob manifest are recorded under
`qa/security-levels/planning/upstream-reconciliation-2026-08-28/`.
No runtime test pass is inferred from documentation validation.

## Qualification still outstanding

This record describes the earlier `main` reconciliation. The later Sprint 13
integration is recorded in the active plan and supersedes its 64/9 and ready-S02
portfolio counts without rewriting this historical validation artifact.

The retained PF-22-S01 evidence explicitly records a non-green workspace run:
15,788 ran, 15,617 passed, 169 failed, two timed out and 28 were skipped.
That historical result is neither replaced nor relabeled by this merge.
The owning repair work, final affected suites, true-TUI runs, all-platform
containment, both live repositories, independent review, human acceptance and
due benchmark evidence remain required by the plan and release policy.
