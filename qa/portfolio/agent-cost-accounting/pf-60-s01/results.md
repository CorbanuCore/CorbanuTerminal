# Local verification receipt — PF-60-S01 proposal v1

## Native Anthropic known-subtotal correction — 2026-09-11

Bounded fix under Travis's standing implementation/review-agent authority, in
the existing active PF-60 plan and PF-60-S01 (`in_progress`) allocation. Product
citation: **Product measurement** — “No commercial performance numbers have
been supplied.” Current corbanu-terminal-development skill, root AGENTS, plan and
sprint rules, active PF-60 plan and S01 were read; no nested AGENTS apply to these
four QA paths. No shared plan/sprint edits or new allocation were made.

Starting worktree was clean at `58b214a0ab1f220a5a768dd605697f7bae54c0ba`, branch
`workstream/accounting-pf60-s01-20260911`, in
`/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-pf60-s01-20260911`.
That committed baseline already includes the reviewed partial-cache validation
fix. This receipt supersedes the 41-test candidate below; its earlier receipts
are historical. The existing cache-sum validation and all 41 tests are preserved.

The parent finding was reproduced by removing `cache_creation_input_tokens`
from every `child-a-1` observation in memory. The original oracle returned
`3/1,000,000 USD`, losing independently measured noncached input 50. Correct
known cost is `(50 * 3 + 10 * 0.3) / 1,000,000 = 153/1,000,000 USD`.
Read remains 10; write, inclusive input, output, total and full cost remain null.
The reasons are `write:usage_unknown`, `output:usage_unknown` and
`attempt_not_complete`. The pricing reason `input:usage_unknown` is no longer
appropriate because the disjoint noncached pricing bucket is measured; inclusive
input remains unknown independently. Root known cost becomes
`(220 + 153 + 636 + 0) / 1,000,000 = 0.001009 USD`; root inclusive input has
known subtotal 340 and one unknown attempt, so its total remains null.

`normalize` validates native Anthropic components and carries raw noncached input
as private `_noncached_input` for `estimate`. `Replay.rows` emits only the existing
six usage fields plus existing cost/billed fields; checkpoints keep raw source
observations. This private count never contributes separately to token totals.
Only the explicit native `anthropic` wire dialect sets it. Chat/Responses still
require a complete inclusive split to derive noncached input, even with provider
name `anthropic`; unsupported compatible dialects remain rejected. Existing
contract semantics suffice; contract, QA README and fixture goldens are unchanged.

The added matrix tests all 27 combinations of noncached `{null, 0, 50}`, read
`{null, 0, 10}` and write `{null, 0, 20}`, each with omission and explicit-null
forms (54 subcases). Output 2 contributes an independent `2 * 6 = 12` micro-USD
in that matrix. The input-only arithmetic for the positive/unknown cases is:

| Noncached / read / write | Known input-bucket subtotal (micro-USD) |
| --- | --- |
| 50 / 10 / unknown | `50 * 3 + 10 * 0.3 = 153` |
| 50 / unknown / 20 | `50 * 3 + 20 * 3.75 = 225` |
| unknown / 10 / 20 | `10 * 0.3 + 20 * 3.75 = 78` |
| 50 / unknown / unknown | `50 * 3 = 150` |
| unknown / 10 / unknown | `10 * 0.3 = 3` |
| unknown / unknown / 20 | `20 * 3.75 = 75` |
| unknown / unknown / unknown | `0` known subtotal; full cost unknown |
| 50 / 10 / 20 | `150 + 3 + 75 = 228`; inclusive input 80 |

A measured zero bucket requires no price but cannot fill an absent measurement.
All-zero completed input/read/write/output costs zero without prices; omitting
write still leaves full cost and inclusive input unknown. Missing noncached
price with measured 50/read 10/output 0 retains `3/1,000,000` known cost and
`input:price_missing`, not `input:usage_unknown`. Invalid noncached counts are
rejected by normalization even when a missing cache field prevents a total.

Partial replay/checkpoint sequence: input 50 gives 150 micro-USD; null input
plus read 10 preserves 50 and gives 153; explicit input 0 replaces 50 and gives
3; input 60/write 20/output 2 completes at `180 + 3 + 75 + 12 = 270` micro-USD,
inclusive input 90 and total 92. Duplicate ingestion, a JSON reopen at each
partial state and all 24 revision permutations agree without additive charging.
This is oracle replay only, not native persistence or process-crash proof.

Commands ran from the worker root with `PYTHONDONTWRITEBYTECODE=1` for every
Python command, preserving the literal write scope.

| UTC batch | Command | Actual result / exit |
| --- | --- | --- |
| 2026-09-11T22:37:14Z | `python3 -m unittest discover -s qa/portfolio/agent-cost-accounting/pf-60-s01 -p 'test_*.py'` | Baseline: 41 tests, 0.054s, OK / 0. |
| 2026-09-11T22:38:59Z | Same discovery, seven added methods, unchanged oracle | 48 tests, 0.057s, 27 failures across five new methods, no errors / 1. All 41 existing tests pass. |
| 2026-09-11T22:39:17Z | Same discovery, corrected oracle | 48 tests (41 existing + 7 new), 0.068s, OK / 0. |
| Completed 2026-09-11T22:39:36Z | `python3 docs/plans/check.py` | `plans: active 3/3; available slots 0` / 0. |
| Completed 2026-09-11T22:39:36Z | `python3 docs/sprints/check.py` | `sprints: current 115; archived 121` / 0. |
| Completed 2026-09-11T22:39:36Z | `git diff --check` | No diagnostics / 0. |

Before-fix failures: one parent reproduction, 20 matrix subcases, one missing
price reason, four invalid raw counts, and one initial partial replay subtotal.
No failed expectations were removed; all six original literal golden rows and
the complete original root aggregate remain unchanged and pass.

| Artifact | SHA-256 |
| --- | --- |
| `reference.py` | `ad2bcbb5d140c29a52ca471496e711af4eaaefb59e2e5b1c97b03eb9338272c1` |
| `test_contract.py` | `4f8e828d97f1300a8358ad764cf3bd834b04ace0f423ca23cc506aa66f5ea3e5` |
| `fixtures.json` (unchanged) | `96ba9416a8d0e69436fdd07c4ca6ddbb20cc30ece757b983eb1de2d708ca95dd` |
| `contract.md` (unchanged) | `15a3060103980595cce252960d18be0daf4b05d95375dbe2f7e51f4f4eb1eb8e` |
| `README.md` (unchanged) | `f348a66c48e742492a6878f815c9b56493699b59e1ca4b9657a9d652f63191e1` |

Only `reference.py`, `test_contract.py`, `results.md` and `handoff.md` change.
Final receipt-file hashes and the SHA-256 of the entire full-index patch are
reported in the delivery message after freezing these files, avoiding a
self-referential digest. Reproduce the patch digest from this unchanged HEAD
with `git diff --binary --full-index --no-ext-diff --no-textconv HEAD -- | shasum -a 256`.
Final whitespace and scope checks accompany that delivery. The patch remains
unstaged and uncommitted for the parent's independent review before a scoped
local commit; this is executor self-check, not independent review.

No agents/models/reviewers, network services, credentials, private logs, live
bills or product runtime were accessed. Native/TUI/live-repository/benchmark
qualification and human acceptance remain untested. Shared ledgers, source
cutover, S02 activation, schema/retention/price policy, push/merge/release and
contract acceptance remain outside this follow-up. Local governance counts are
worker-only; the parent's different current-sprint count is not reconciled here.

## Bounded cache-invariant correction — 2026-09-11

This follow-up supersedes the original oracle/test digests and 33-test candidate
below. Original receipts remain historical evidence. Starting worktree was clean
on `workstream/accounting-pf60-s01-20260911` at
`1a29602e8ab5605ed781dd5ddf1eb49e6033b872`; allocated ancestor
`295aed26e53b17f919f7199ae1c9748b1b1250ba` and original launch
`f17e5a54daba11c6da2554f14b62f11ace752142` were verified as HEAD ancestors.

Change class: bounded fix within active PF-60 / PF-60-S01 (`in_progress`).
Product citation: **Product measurement** — “No commercial performance numbers
have been supplied.” Scope: reject known cache sums above known inclusive input
without converting an absent component to zero. No contract vocabulary, prices,
retention, fixture data, runtime, schema, plan or sprint edits.

Commands ran from the recorded worker root using Python 3.14.4. Each Python
command below had the environment prefix `PYTHONDONTWRITEBYTECODE=1` to avoid
writing bytecode outside the literal four-file scope.

| UTC batch | Command | Observed result / exit |
| --- | --- | --- |
| 2026-09-11T22:25:04Z | `python3 -m unittest discover -s qa/portfolio/agent-cost-accounting/pf-60-s01 -p 'test_*.py'` | Original baseline: 33 tests, 0.046s, OK / 0. |
| After baseline, before 22:26:42Z | Same discovery, eight regressions added, original oracle | 41 tests, 0.055s, 13 failing subcases across three new methods / 1; all failures were expected missing ValueErrors. |
| 2026-09-11T22:26:42Z | Same discovery, corrected oracle | 41 tests (33 original + 8 added), 0.054s, OK / 0. |
| 2026-09-11T22:26:57Z | `python3 docs/plans/check.py` | `plans: active 3/3; available slots 0` / 0. |
| 2026-09-11T22:26:57Z | `python3 docs/sprints/check.py` | `sprints: current 115; archived 121` / 0. |
| 2026-09-11T22:26:57Z | `git diff --check` | No diagnostics / 0. |

The parent reports 116 current / 121 archived in its manager checkout. This
worker's verified baseline contains 115 / 121; no shared ledger reconciliation
was attempted. These successful local checks do not assert combined-tree results.

Sibling inspection covered both supported inclusive-input dialects, complete
cache sums whose individual components each fit input, zero input, missing/null
components, equality boundaries, unknown inclusive input, and Anthropic's
noncached-input dialect. Anthropic inclusive input still requires all three
components; its raw input 10 plus read 20 plus write 30 correctly becomes 60.
Chat has no supported measured write field, so reciprocal write-only coverage is
Responses-only. Replay checks each reconstructed revision; an invalid partial
split cannot be hidden by a later larger input. Valid partial replay retains
unknown fields and unknown full cost. `estimate` still requires all split
components before computing noncached input; no pricing change was necessary.
All original six literal golden rows and aggregate expectations still pass.

| Tested artifact | Corrected SHA-256 |
| --- | --- |
| `reference.py` | `fdbd73812777a823c9088c70d906e5a79d59b6355fcd458999dbf7a5d8e204d0` |
| `test_contract.py` | `d3759262be0ae31a474def4680f95bf7fde242973dd8307275fce25c951793d4` |
| `fixtures.json` (unchanged) | `96ba9416a8d0e69436fdd07c4ca6ddbb20cc30ece757b983eb1de2d708ca95dd` |

Only `reference.py`, `test_contract.py`, `results.md` and `handoff.md` changed.
Oracle diff: +3/-2 non-test source lines; tests: +102/-0 lines, eight methods.
Candidate remains uncommitted and unstaged for the parent's mandatory independent
structured review. Final patch identity and scope totals accompany the delivery.
This is executor self-check only; no agents, reviewers or model calls were
launched. Native/TUI/live-repository/benchmark and human acceptance evidence
remain untested; no S02 readiness, enablement or release claim follows.

## Original kickoff receipt (historical)

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
