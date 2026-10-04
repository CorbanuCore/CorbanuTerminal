# owner-rearm-106 verification

Read `docs/development/test-isolation.md` before testing. Built a disposable
venv under `env -i` at `/private/tmp/owner-rearm-106.DdvOmp/venv` using
`scripts/initiative_control/requirements.txt`: markdown-it-py 3.0.0,
mdurl 0.1.2 and slack-sdk 3.44.1. Installation output is retained in
[the venv log](owner-rearm-106-venv.txt). All commands ran from the assigned
repository root. Only disposable stores/profiles and synthetic SDK fixtures
were used.

Every Python/Node verification command used an empty environment plus:

```text
HOME=/private/tmp/owner-rearm-106.DdvOmp/home
CODEX_HOME=/private/tmp/owner-rearm-106.DdvOmp/home
CORBANU_HOME=/private/tmp/owner-rearm-106.DdvOmp/home
PFTERMINAL_HOME=/private/tmp/owner-rearm-106.DdvOmp/home
CORBANU_TEST_NO_NATIVE_KEYRING=1
TMPDIR=/private/tmp
PATH=/opt/homebrew/bin:/usr/bin:/bin
PYTHONPATH=scripts/initiative_control:qa/initiative-control/management-bootstrap
PYTHONDONTWRITEBYTECODE=1
```

Here `python` means `/private/tmp/owner-rearm-106.DdvOmp/venv/bin/python -B`.

```sh
python -m unittest -v \
  test_decision_manager.ManagerTests.test_legacy_quarantine_separates_unknown_history_from_outstanding \
  test_decision_manager.ManagerTests.test_reviewed_legacy_pruning_stays_unknown_after_unrelated_ingress \
  test_slack_transport.TransportTests.test_legacy_unknown_is_durable_and_exact_review_does_not_rearm \
  test_slack_transport.TransportTests.test_legacy_unknown_summary_rejects_invalid_counts \
  test_slack_transport.TransportTests.test_outstanding_quarantine_survives_pruning_then_exact_review_clears_it
python -m unittest discover -v -s scripts/initiative_control -p 'test_*.py'
python -m unittest -v test_owner_daemon test_owner_tmux test_decision_feed test_attention test_owner_promotion_94 test_owner_preflight_98
node scripts/initiative_control/test_facilities_js.cjs
python qa/initiative-control/management-bootstrap/owner_outstanding_104_observe.py
git diff --check
git status --short
```

Read-only AST parsing checked all four changed Python files: 4 valid ASTs.
No formatter was needed or run. The final implementation also preserves an
independent transport hold when history is unknown; that assertion was added
after targeted replay and before the final full/focused gates. Source/test
files have remained fixed since those final gates started.

| Check | Actual result | Raw evidence |
| --- | --- | --- |
| First targeted attempt | 5 run, 3 failure outcomes across 2 methods, 7.174s, exit 1 | [first targeted](owner-rearm-106-targeted.txt) |
| Corrected targeted replay | 5 passed, 6.713s, exit 0 | [replay](owner-rearm-106-targeted-replay.txt) |
| Final full discovery | 859 passed, 478.674s, exit 0; no failures/errors/skips | [discovery](owner-rearm-106-suite.txt) |
| Final owner/feed/renderer/promotion/preflight gate | 257 passed, 103.942s, exit 0; no failures/errors/skips | [focused](owner-rearm-106-focused.txt) |
| JavaScript renderer regression | 1 program passed, exit 0 | [renderer](owner-rearm-106-renderer.txt) |
| Existing disposable observation harness on final code | 5 observations, exit 0, empty stderr | [observations](owner-rearm-106-observations.jsonl), [stderr](owner-rearm-106-observations-stderr.txt) |

Focused results overlap discovery; they are additional executions, not unique
coverage: 133 owner-daemon, 52 owner-TMUX, 38 feed, 22 attention/renderer,
8 promotion and 4 preflight tests. Three new test methods were added in this
allocation; one prior legacy-projection test was corrected and renamed.

## Nonzero accounting

The first targeted attempt exited 1 with these exact failure names:

- `test_decision_manager.ManagerTests.test_reviewed_legacy_pruning_stays_unknown_after_unrelated_ingress`:
  one `advanced=True` subtest failure and one final hold assertion failure.
- `test_slack_transport.TransportTests.test_legacy_unknown_is_durable_and_exact_review_does_not_rearm`:
  one outstanding-count assertion failure.

The fixture's intended unrelated bot echo omitted the pinned app ID. The
transport correctly treated that incomplete echo as rejected intake. Added
the fixture app ID, then replayed successfully without changing implementation.
The failed attempt remains intact; it was not overwritten.

Final accounting: **1 nonzero top-level test/process command**, **3 unittest
failure outcomes in 2 methods**, **0 tool rejections**. Expected negative-path
child exits asserted inside tests are not top-level campaign failures.
Final discovery and focused gates: **0 failures, 0 errors, 0 skips**.
Final failure names: **none**.

## Changed lines and source identity

Relative to assigned base `8ef2e1c070f78093b295c4fe734e347c992eee39`:

| File | Added | Deleted | SHA-256 |
| --- | ---: | ---: | --- |
| scripts/initiative_control/decision_manager.py | 20 | 7 | `b5863bdd8dc16ccdd99beab41abbb35c4737de175e054bc13df94f22dfed1152` |
| scripts/initiative_control/slack_transport.py | 12 | 8 | `c99503828c1fd99bdc0f841f5e9c7690039178015faa53dbfc054a38dc311f66` |
| scripts/initiative_control/test_decision_manager.py | 42 | 2 | `4041ede7bebf69baa98dfd0b60462fed347eee96db022b0f1db94d51ff601312` |
| scripts/initiative_control/test_slack_transport.py | 50 | 0 | `3a5faafb1e64372f064cefec91a1b5f39494529d183d113b809aff4f5a882afe` |
| Source/test total | 124 | 17 | |

All 10 new evidence files are under the allowed management-bootstrap directory:

| Artifact | Added lines |
| --- | ---: |
| owner-rearm-106-focused.txt | 289 |
| owner-rearm-106-observations-stderr.txt | 0 |
| owner-rearm-106-observations.jsonl | 5 |
| owner-rearm-106-renderer.txt | 1 |
| owner-rearm-106-suite.txt | 914 |
| owner-rearm-106-targeted-replay.txt | 10 |
| owner-rearm-106-targeted.txt | 48 |
| owner-rearm-106-venv.txt | 12 |
| owner-rearm-106-verification.md | 116 |
| owner-rearm-106.md | 123 |

QA total: **1518 added lines**. All-files total: **1642 added / 17 deleted**,
across 4 modified Python files and 10 new QA artifacts.

This is worker verification for manager review. It does not establish the
missing distinct rendered notices, independent code-blind acceptance, packaged
true-TUI qualification or named-human acceptance. No live profile, credential
inspection, native credential prompt, Rust test, workspace formatter, commit
or push was used.
