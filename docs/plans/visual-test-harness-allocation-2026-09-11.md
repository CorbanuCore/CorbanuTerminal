# Visual QA follow-up allocation — September 11

## Authority and state

The existing task “Set up and build PF13 branch” (`01a04a32-b01b-7ad2-91b1-0f7ff2b11456`)
relayed Travis's approval to place screenshot/RTX inference QA infrastructure
under delivery control, with setup instructions and a command harness eventually
committed on `feat/provider-reauth-health`. This manager owns that allocation.
This is a **planning-only** amendment to the product initiative, not implementation,
permission to bypass a tool denial, a new worker launch or a release.

[PF-81-S01](../sprints/current/initiative-delivery-control/pf-81-s01-visual-test-harness.md)
is a distinct harness feature in the existing
[active delivery-control plan](active/initiative-delivery-control.md). It is not
provider-auth work and does not expand Godel's projection/progress-adapter scope.
PF-80-S01 remains in progress; accounting and PF13 are undisturbed. PF-79 beta
drafts retain their existing dependencies and order. The manager must choose
one eligible follow-up at a time, never infer concurrency from disjoint paths.

## Verified destination and preservation

| Field | September 11 observation / intended allocation |
| --- | --- |
| Existing owner | PF13/provider-reauth task above; no new model/review worker |
| Worktree | `/Volumes/CorbanuDrive/Corbanu/worktrees/provider-reauth-health` |
| Branch / observed HEAD | `feat/provider-reauth-health` / `eb7af932bc8cc859122b5b2dbef223fbd45ea25c` |
| Main at inspection | `7bb0697cf8a5e88fc50bc232b65fea8d77f6da4a` |
| Merge base with main | `295aed26e53b17f919f7199ae1c9748b1b1250ba` |
| Future worker writes | `scripts/visual_test_harness/`, `qa/visual-test-harness/` only |
| Pending unrelated writes | Modified `humanTest.html`; untracked `qa/provider-auth/pf-58/luna-yellow-20260911/`; untouched |

The branch predates the planning merge. Its verified HEAD is a draft destination
pin, not proof that it contains the current plan or is ready to execute. Do not
reset, stash, cherry-pick into or merge main into the dirty owner checkout on
the owner's behalf. Existing evidence must remain separate from the harness
commit; reading this allocation does not overwrite local policy or allocate
shared-plan writes to the worker.

## Activation and integration instructions

1. Owner may preserve this planning handoff now; do not implement the draft.
   Wait for PF-80-S01 completion/archive, including its required acceptance,
   and manager selection of PF-81 as the sole next reserved delivery sprint.
2. At activation, owner preserves/commits existing work separately. Manager and
   owner reconcile current main/planning commits into the intended branch,
   inspect conflicts, and record the actual updated base in both plan and sprint.
   Do not cherry-pick this amendment alone onto the old branch: it relies on
   main's three-stream allocation and deferred-provider-plan baseline.
3. Manager validates exact owner, dependency, coordinates and disjoint scope,
   runs both governance checkers, then marks the sprint ready/in_progress.
   The old PF-58 review allowance remains exhausted; obtain a new explicit
   review decision before any required independent-review gate.
4. Existing owner implements only PF-81's remaining tasks and commits the two
   literal directories separately on `feat/provider-reauth-health`. Return
   commit, protocol version, tests, evidence and limitations. Do not push main,
   deploy an RTX service, contact a live inference endpoint or exercise a denied
   surface under this allocation.
5. Manager inspects the exact scoped diff against its parent, selects only the
   harness commit for receiving-main integration, reruns focused and governance
   suites on the combined tree, and records review/human gates accurately.
   Never merge the full provider branch as a shortcut. Shared CI/report adapters
   need a separately reviewed manager-owned change.

## Safety and acceptance boundaries

Synthetic fixture and recorded-response tests can qualify the harness contract,
not native Corbanu behavior, live RTX inference or restricted application tests.
In particular, a Computer Use denial for Terminal cannot be bypassed through
another driver, model, CLI, screenshot method or remote shell. Model output is
untrusted input; a validated action still requires an allowed fixture target.
Credentials, Keychain, private screenshots, wallet data and arbitrary-shell
actions are excluded. Live targets, recipients and data permission must be
resolved before any later live phase. No paid inference, public publication,
beta launch or financial action is authorized by this planning change.

## Planning verification and dependency receipt

- `python3 docs/plans/check.py`: passed, three active plans.
- `python3 docs/sprints/check.py`: passed, 116 current / 121 archived records;
  the existing three sprint reservations are unchanged.
- `python3 -m unittest discover -s docs/plans/tests -p 'test_*.py'`: five passed.
- `python3 -m unittest discover -s docs/sprints/tests -p 'test_*.py'`: 22 passed.
- Initial discovery from each parent directory found zero tests; those runs
  are not counted as evidence. The explicit test directories above were rerun.
- Manager inspected the six-file planning/nav diff; no runtime code changed.
  No new model/review worker was requested or launched for this amendment.

During allocation, Godel returned local commit
`f7159c16723f59d021f46c7b66fdb8401ac777e4`, reporting 48 focused and 27 governance
tests passing. This is a worker handoff, not manager-reviewed integration or
PF-80-S01 completion. Its native sending, human/live gates and source cutover
remain pending. PF-81 therefore remains draft. The earlier main CI jobs could
not start because GitHub reported an account billing lock; no CI pass inferred.

No independent review, actual-key TUI proof, live-repository test, human
sign-off, GPU benchmark or release completion is asserted by creating this draft.
