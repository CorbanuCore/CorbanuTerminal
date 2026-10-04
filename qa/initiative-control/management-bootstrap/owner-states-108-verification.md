# owner-states-108 verification

Read `docs/development/test-isolation.md` before testing. Created the disposable
venv under `env -i` at `/private/tmp/owner-states-108.KcyBPB/venv` and installed
`scripts/initiative_control/requirements.txt`. The installation log is
[owner-states-108-venv.txt](owner-states-108-venv.txt).
All gate commands ran from the assigned repository root.

Every gate and observation command used `env -i` with only:

```text
HOME=/private/tmp/owner-states-108.KcyBPB/home
CODEX_HOME=/private/tmp/owner-states-108.KcyBPB/home
CORBANU_HOME=/private/tmp/owner-states-108.KcyBPB/home
PFTERMINAL_HOME=/private/tmp/owner-states-108.KcyBPB/home
CORBANU_TEST_NO_NATIVE_KEYRING=1
TMPDIR=/private/tmp
PATH=/opt/homebrew/bin:/usr/bin:/bin
PYTHONPATH=scripts/initiative_control:qa/initiative-control/management-bootstrap
PYTHONDONTWRITEBYTECODE=1
```

Here `python` means
`/private/tmp/owner-states-108.KcyBPB/venv/bin/python -B`.

```sh
python -m unittest -v \
  test_slack_transport.TransportTests.test_quiet_legacy_unknown_requires_explicit_review_and_clears_durably \
  test_slack_transport.TransportTests.test_quiet_legacy_unknown_failed_review_write_preserves_unknown \
  test_slack_transport.TransportTests.test_quiet_legacy_unknown_rejects_inexact_explicit_reviews
python -m unittest discover -v -s scripts/initiative_control -p 'test_*.py'
python -m unittest -v test_owner_daemon test_owner_tmux test_decision_feed test_attention test_control test_owner_promotion_94 test_owner_preflight_98
node scripts/initiative_control/test_facilities_js.cjs
python qa/initiative-control/management-bootstrap/owner_states_108_observe.py
git diff --check
git status --short
```

Only disposable stores/profiles and synthetic SDK endpoints were used. No
formatter was needed or run; the Python source and tests remained unchanged
from the start of the full gate onward. Focused and discovery suites used
independent disposable fixtures. The observation harness only captures the
current output and does not assert that the requested UI behavior is fixed.

| Check | Actual result | Evidence |
| --- | --- | --- |
| New quiet-legacy regressions | 3 passed, 3.183s, exit 0 | [targeted](owner-states-108-targeted.txt) |
| Full discovery | 862 passed, 475.210s, exit 0; no failures/errors/skips | [discovery](owner-states-108-suite.txt) |
| Owner/feed/attention/control/promotion/preflight | 296 passed, 99.283s, exit 0 | [focused](owner-states-108-focused.txt) |
| JavaScript renderer regression | 1 program passed, exit 0 | [renderer](owner-states-108-renderer.txt) |
| Disposable renderer observations | 8 scenarios, exit 0, empty stderr | [observations](owner-states-108-observations.jsonl), [stderr](owner-states-108-observations-stderr.txt) |

Focused counts: 133 owner-daemon, 52 owner-TMUX, 38 feed, 22 attention,
39 control, 8 promotion and 4 preflight. These overlap discovery; they are
additional executions, not additional unique coverage.

## Nonzero accounting

One read-only inspection command exited 2 because its `rg` arguments included
nonexistent `scripts/initiative_control/slack_bridge.py`. The dependent reads
after its `&&` were not executed; they were subsequently run successfully
against existing files. This was not a test failure.

Final accounting: **1 nonzero top-level inspection command**, **0 nonzero
test/process gates**, **0 unittest failures**, **0 errors**, **0 skips**,
**0 tool rejections**. Failure names: **none**. Expected negative-path child
exits inside passing tests are not top-level campaign failures.

## Source changes

Relative to assigned base `e839c466a376cfaf684cb7f11cb9ec143a8eafd4`:

| File | Added | Deleted | SHA-256 |
| --- | ---: | ---: | --- |
| scripts/initiative_control/slack_transport.py | 4 | 1 | `a9096907b71589af2d3296c5efc0563768ce046089d9bfe2afd70c123a188b3c` |
| scripts/initiative_control/test_slack_transport.py | 62 | 0 | `19dd2b1a70466c27635c07467fce0e080b507d249f74f136534fe65d6e1dcdfd` |
| Source/test total | 66 | 1 | |

The production change begins at line 768; new fixture/regression coverage begins
at line 1561. All other new files are evidence under the explicitly allowed
management-bootstrap directory. No commit or push was made.

This return does not establish the missing distinct rendered notices, independent
code-blind acceptance, packaged true-TUI qualification or named-human acceptance.
See [the return](owner-states-108.md) for the scope conflict and literal notices.
