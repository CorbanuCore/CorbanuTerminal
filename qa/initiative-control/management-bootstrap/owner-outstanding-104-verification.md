# owner-outstanding-104 verification

Read `docs/development/test-isolation.md` before testing. Built the disposable
venv under `env -i` at
`/private/tmp/owner-outstanding-104.1c7HfA/venv` using exactly
`scripts/initiative_control/requirements.txt`: markdown-it-py 3.0.0,
mdurl 0.1.2 and slack-sdk 3.44.1. Commands ran from the assigned repository root.

Every Python/Node test or observation command used an empty environment plus:

```text
HOME=/private/tmp/owner-outstanding-104.1c7HfA/home
CODEX_HOME=/private/tmp/owner-outstanding-104.1c7HfA/home
CORBANU_HOME=/private/tmp/owner-outstanding-104.1c7HfA/home
PFTERMINAL_HOME=/private/tmp/owner-outstanding-104.1c7HfA/home
CORBANU_TEST_NO_NATIVE_KEYRING=1
TMPDIR=/private/tmp
PATH=/opt/homebrew/bin:/usr/bin:/bin
PYTHONPATH=scripts/initiative_control:qa/initiative-control/management-bootstrap
PYTHONDONTWRITEBYTECODE=1
```

`python` below means the disposable venv's `bin/python -B`.
All stores/profiles, fake SDK credentials and local HTTP listeners were fixtures.

## Commands and evidence

```sh
python -m unittest discover -v -s scripts/initiative_control -p 'test_*.py'
python -m unittest -v test_owner_daemon test_owner_tmux test_decision_feed test_attention test_owner_promotion_94 test_owner_preflight_98
node scripts/initiative_control/test_facilities_js.cjs
python qa/initiative-control/management-bootstrap/owner_outstanding_104_observe.py
git diff --check
git status --short
```

Read-only AST parsing also checked both changed implementation modules, both
changed test modules and the observation script (5 valid Python ASTs).
No workspace formatter ran, and no out-of-scope file changed.

| Check | Actual result | Raw evidence |
| --- | --- | --- |
| First targeted regressions | 8 run, 5 errors, 10.863s, exit 1 | [first](owner-outstanding-104-targeted-first.txt) |
| Corrected targeted regressions | 8 passed, 10.828s, exit 0 | [second](owner-outstanding-104-targeted-second.txt) |
| First full discovery | 856 run, 855 passed, 1 failure, 471.517s, exit 1 | [first discovery](owner-outstanding-104-suite.txt) |
| Corrected history assertion replay | 1 passed, 1.076s, exit 0 | [replay](owner-outstanding-104-poison-replay.txt) |
| Final full discovery | 856 passed, 475.056s, exit 0; no failures/errors/skips | [final discovery](owner-outstanding-104-suite-final.txt) |
| Final owner/feed/renderer/promotion checks | 257 passed, 99.845s, exit 0 | [focused](owner-outstanding-104-focused.txt) |
| JavaScript renderer regression | 1 program passed, exit 0 | [renderer](owner-outstanding-104-renderer.txt) |
| Disposable observation harness | 5 observations, exit 0, empty stderr | [observations](owner-outstanding-104-observations.jsonl) |
| Source checks | 5 Python ASTs valid; git diff --check passed | Read-only checks |

The focused count overlaps discovery: 133 owner-daemon, 52 owner-TMUX,
38 feed, 22 attention/renderer, 8 promotion and 4 preflight tests. Those
are additional executions, not additional unique test coverage. Relative to
the 850-test base, the revised discovery has 6 new regression methods.

The [five observation rows](owner-outstanding-104-observations.jsonl) cover held,
quarantined, stale with quarantine, stale without quarantine, and fine states,
including actual renderer notice strings. Its [stderr](owner-outstanding-104-observations-stderr.txt)
is retained. The observation is a supporting disposable fixture run, not an
independent code-blind acceptance execution or a true-TUI qualification.

## Nonzero accounting

Nonzero top-level verification commands: **2**. The first targeted
attempt exited 1 (8 tests, 5 errors in 3 methods); the first discovery exited 1
(856 tests, 1 failure). The six unsuccessful unittest outcomes remain recorded.

Discovery's failure was
`test_slack_transport.TransportTests.test_poison_shapes_are_quarantined_and_following_human_reply_is_delivered`.
Its prior assertion required the entire quarantine object to be identical after
review. The revised assertion requires unchanged audit history and a cleared
outstanding summary. The isolated replay passed 1 test in 1.076s, exit 0
([log](owner-outstanding-104-poison-replay.txt)).
The initial full log is preserved at [first discovery](owner-outstanding-104-suite.txt).

Final discovery and focused gates have **0 failures, 0 errors, 0 skips**;
final failure names: none. Including the separate inspection command below,
there were **3 nonzero top-level process commands** (2 test commands and
1 inspection), plus **2 tool rejections**. Expected negative-path child exits
asserted inside unit tests are not top-level campaign failures.

The first targeted attempt's five errors occurred in three new methods:

- `test_supervisor_reason_precedes_quarantine_without_hiding_count`: three
  subtest errors (unavailable, stale, flush-failed), because the fixture used
  `+00:00` timestamps instead of this protocol's required `Z` timestamps.
- `test_route_at_or_after_deadline_delivers_retained_reply_once`: one subtest
  error (delay 60), for the same fixture timestamp problem.
- `test_outstanding_quarantine_survives_pruning_then_exact_review_clears_it`:
  one error because the store lock translates the injected OSError into
  `decisions.Invalid`. The corrected case injects failure at the final
  qualification write and asserts that the unresolved summary survives.

The original failed attempt remains in
[the first targeted log](owner-outstanding-104-targeted-first.txt). The
[second targeted log](owner-outstanding-104-targeted-second.txt) retains the
successful replay. No failed attempt was overwritten.

One initial read-only nested-AGENTS search returned exit 1 (no matches).
One JavaScript orchestration call was rejected for invalid string syntax before
any edit tool executed. A separate redundant structured edit was rejected because
its old/new strings were identical; it made no change. These tool rejections
are not process exits or test passes.

## Changed lines and source identity

Relative to assigned base `6ed3300006a133c6ea1b862504109a5b924fd76a`:

| File | Added | Deleted | SHA-256 |
| --- | ---: | ---: | --- |
| scripts/initiative_control/decision_manager.py | 16 | 12 | `8f50fe55a4eb57bfae9620515a823e19f3ac7d22d0a277a0c7a57de6f2ad56df` |
| scripts/initiative_control/slack_transport.py | 39 | 3 | `6ae01a6058e2a8e65b7d878b1f4421f51da71552cc3d2bec741e0cf007c1d114` |
| scripts/initiative_control/test_decision_manager.py | 60 | 5 | `ab1bae87486c0318abc890e2691372f44ab36f3119bdd87543f83208d02964fa` |
| scripts/initiative_control/test_slack_transport.py | 70 | 3 | `460753521cc9eb3bbfeb13b09dce89ecdee6398a7c15d25ba6352076f8941bac` |
| Source/test total | 185 | 23 | |

All 12 new QA files are under the allowed management-bootstrap directory:

| Artifact | Added lines |
| --- | ---: |
| owner-outstanding-104-focused.txt | 289 |
| owner-outstanding-104-observations-stderr.txt | 0 |
| owner-outstanding-104-observations.jsonl | 5 |
| owner-outstanding-104-poison-replay.txt | 6 |
| owner-outstanding-104-renderer.txt | 1 |
| owner-outstanding-104-suite-final.txt | 911 |
| owner-outstanding-104-suite.txt | 921 |
| owner-outstanding-104-targeted-first.txt | 73 |
| owner-outstanding-104-targeted-second.txt | 13 |
| owner-outstanding-104-verification.md | 141 |
| owner-outstanding-104.md | 107 |
| owner_outstanding_104_observe.py | 51 |

QA total: **2518 added lines**. All-files total: **2703 added / 23 deleted**,
across 4 modified Python files and 12 new QA artifacts.

The source/test tree stayed fixed during the final full/focused gates.
No live store/listener/coordinator, native credential store, personal profile,
Rust test, commit, push, deployment or acceptance approval was used.
