# Local verification receipt — PF-60-S01 proposal v1

Executor: requested accounting worker, workstream 2; self-check only. Independent
reviewer, named-human acceptance and combined-tree verification: pending manager.
Environment: macOS worker checkout, Python 3.14.4, standard library; no provider
or account connections. Worktree and branch are recorded in [handoff.md](handoff.md).
Verified starting HEAD: `f17e5a54daba11c6da2554f14b62f11ace752142`.

## Executed commands and actual output

All commands ran from the worker repository root. Expected exits were zero;
each listed exit was zero. UTC timestamps identify the tool batches, not provider
request times. Fixture dates are fabricated data.

| UTC batch | Exact command | Actual result |
| --- | --- | --- |
| 2026-09-11T09:52:31Z | `python3 -m unittest discover -s qa/portfolio/agent-cost-accounting/pf-60-s01 -p 'test_*.py'` | Initial candidate: `Ran 32 tests in 0.052s`, `OK`. Superseded by the corrected Chat fixture below. |
| 2026-09-11T09:54:54Z | `python3 -m unittest discover -s qa/portfolio/agent-cost-accounting/pf-60-s01 -p 'test_*.py'` | Final contract/fixture/code digests below: `Ran 33 tests in 0.047s`, `OK`. Nonempty discovery confirmed. |
| 2026-09-11T09:55:09Z | `python3 docs/plans/check.py` | `plans: active 3/3; available slots 0` |
| 2026-09-11T09:55:09Z | `python3 docs/sprints/check.py` | `sprints: current 115; archived 121` |
| 2026-09-11T09:55:09Z | `git diff --check` | No diagnostics. New files were still untracked; this alone did not inspect their whitespace. Staged check is required before commit. |
| 2026-09-11T09:55:09Z | `git diff --cached --check` | No diagnostics; index still empty at this point. Final staged scope/whitespace inspection accompanies the commit receipt. |
| 2026-09-11T09:57:14Z | `python3 -m unittest discover -s qa/portfolio/agent-cost-accounting/pf-60-s01 -p 'test_*.py'` | Final staged candidate: `Ran 33 tests in 0.046s`, `OK`. |
| 2026-09-11T09:57:14Z | `python3 docs/plans/check.py` | `plans: active 3/3; available slots 0` |
| 2026-09-11T09:57:14Z | `python3 docs/sprints/check.py` | `sprints: current 115; archived 121` |
| 2026-09-11T09:57:14Z | `git diff --check` | No diagnostics; no unstaged implementation changes. |
| 2026-09-11T09:57:14Z | `git diff --cached --check` | No diagnostics; all seven new scoped files staged and inspected. |

Final `git diff --cached --name-only` and `git status --short` contained exactly
the seven new files listed in handoff, all in the two exclusive output directories.
Only these verification receipt statements were appended after that test batch;
the five digested contract/fixture/oracle files did not change.

The initial passing fixture incorrectly assumed Responses wire for the Corbanu
API compatibility route. Source self-check at `create_pfterminal_plan_provider`
showed `WireApi::Chat`; raw fields and expected cache-write unknowns were corrected,
and a regression assertion was added. A fixture pass alone would not have caught
that source-mapping error. No failed assertions were hidden or empty selectors used.

## Candidate artifact identities

SHA-256 from `shasum -a 256` at 2026-09-11T09:54:54Z. The final worker commit is
reported in the delivery receipt; these digests pin the actual tested inputs and
oracle without a self-referential commit hash inside its own file.

| Path (relative to repository root) | SHA-256 |
| --- | --- |
| `docs/research/agent-cost-accounting/contract.md` | `15a3060103980595cce252960d18be0daf4b05d95375dbe2f7e51f4f4eb1eb8e` |
| `qa/portfolio/agent-cost-accounting/pf-60-s01/fixtures.json` | `96ba9416a8d0e69436fdd07c4ca6ddbb20cc30ece757b983eb1de2d708ca95dd` |
| `qa/portfolio/agent-cost-accounting/pf-60-s01/reference.py` | `97722dbd0f42c4e85a52e30ca706812597756359f11e2c0d3ff1f0a30df35ef5` |
| `qa/portfolio/agent-cost-accounting/pf-60-s01/test_contract.py` | `3a895b3432059ba50546b2fb9a43d6bbabbd09415fae224a54c631edec70a7a6` |
| `qa/portfolio/agent-cost-accounting/pf-60-s01/README.md` | `f348a66c48e742492a6878f815c9b56493699b59e1ca4b9657a9d652f63191e1` |

## Expected versus actual and negative evidence

All six literal request-attempt golden rows matched. Root subtree: 3 logical
requests / 4 attempts, known estimate `0.001084 USD`, full estimate null with
2 unknown attempts; separate known billed `0.000300 USD`, billed total null.
Input total 420, read total 90; write known 60/full null, output known 60/full
null, reasoning known 8/full null, total tokens known 400/full null.
Historical priced attempt retained `0.000110 USD`; historical missing split
retained only known output estimate `0.000040 USD` and null full cost.

Duplicate streams replayed twice, reordered revisions, all six retry revision
permutations and a fresh-process JSON checkpoint reconstruction agreed exactly.
Conflicting duplicate replay, invalid numeric values, cache/reasoning/total
inconsistencies, lineage/retry cycles, unknown owners, duplicate attempts,
overlapping/duplicate/unsupported prices and duplicate invoice allocation raised
the expected errors. Missing prices, timestamps and measurement fields remained
unknown. The full counterexample matrix and explicit arithmetic are in
[README.md](README.md); executable assertions are in [test_contract.py](test_contract.py).

Limitations: all tests exercise the synthetic reference oracle, not a native
adapter. Original sequence and complete declared attempt membership are supplied
by the fixture; unknown coverage outside that supplied set is not modeled. No
database writes, kill/fsync durability, Rust compilation, product UI, TUI keys,
live TensorCash/Isometric workflows, benchmark, release, provider billing or
human acceptance was tested or claimed. Immutable price storage and retention
remain proposals. No extra agents/models/reviewers were invoked.

Remote CI: manager reported GitHub Actions jobs for the merged planning commit
were not started because the account is locked due to a billing issue. This was
not independently queried by the worker and is not a local test failure or pass.
Manager also reported remote-main receipt commit
`7bb0697cf8a5e88fc50bc232b65fea8d77f6da4a` after this worker's starting base.
The worker did not pull, retry CI, edit workflows, push or merge.
