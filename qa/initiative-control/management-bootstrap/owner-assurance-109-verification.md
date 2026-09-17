# owner-assurance-109 verification

Read `docs/development/test-isolation.md` before any test. All gate commands
ran from the assigned repository root. None of the four changed production/test
files was edited or formatted after the targeted tests began. Evidence harnesses
were added and executed afterward. No workspace formatter was run.

Disposable environment: `/private/tmp/owner-assurance-109.iCvOEZ`.
The venv was created with `env -i` and `/opt/homebrew/bin/python3 -m venv`,
then requirements were installed under `env -i` from
`scripts/initiative_control/requirements.txt`. See
[installation log](owner-assurance-109-venv.txt). Both commands received only
`HOME=<disposable>/home`, `PATH=/opt/homebrew/bin:/usr/bin:/bin`, and
`TMPDIR=/private/tmp`.

All Python tests and both evidence harnesses ran with `env -i` and only:

```text
HOME=/private/tmp/owner-assurance-109.iCvOEZ/home
CODEX_HOME=/private/tmp/owner-assurance-109.iCvOEZ/home
CORBANU_HOME=/private/tmp/owner-assurance-109.iCvOEZ/home
PFTERMINAL_HOME=/private/tmp/owner-assurance-109.iCvOEZ/home
CORBANU_TEST_NO_NATIVE_KEYRING=1
TMPDIR=/private/tmp
PATH=/opt/homebrew/bin:/usr/bin:/bin
PYTHONPATH=scripts/initiative_control:qa/initiative-control/management-bootstrap
PYTHONDONTWRITEBYTECODE=1
```

The JavaScript check used the same environment except the two Python variables.
Here `python` means
`/private/tmp/owner-assurance-109.iCvOEZ/venv/bin/python -B`.

```sh
python -m unittest -v \
  test_slack_transport.TransportTests.test_invalid_review_does_not_pin_healthy_quiet_journal \
  test_slack_transport.TransportTests.test_qualifying_journal_recovers_only_with_exact_review \
  test_slack_transport.TransportTests.test_review_rechecks_undrained_events_after_auth \
  test_slack_transport.TransportTests.test_quiet_legacy_unknown_rejects_inexact_explicit_reviews \
  test_decision_manager.ManagerTests.test_supervisor_uncertainty_downgrades_fresh_projection_and_rendering
python -m unittest discover -v -s scripts/initiative_control -p 'test_*.py'
python -m unittest -v test_owner_daemon test_owner_tmux test_decision_feed test_attention test_control test_owner_promotion_94 test_owner_preflight_98
node scripts/initiative_control/test_facilities_js.cjs
python qa/initiative-control/management-bootstrap/owner_assurance_109_observe.py
python qa/initiative-control/management-bootstrap/owner_assurance_109_reproduce.py
git diff --check
git status --short
```

| Check | Result | Raw evidence |
| --- | --- | --- |
| Targeted regressions | 5 passed, 7.236s, exit 0 | [targeted](owner-assurance-109-targeted.txt) |
| Full discovery | 866 passed, 481.753s, exit 0 | [discovery](owner-assurance-109-suite.txt) |
| Owner/feed/attention/control/promotion/preflight | 296 passed, 98.800s, exit 0 | [focused](owner-assurance-109-focused.txt) |
| JavaScript renderer | 1 program passed, exit 0 | [renderer](owner-assurance-109-renderer.txt) |
| Actual rendered notices | 7 observations, exit 0; records unresolved defects rather than passing them | [observations](owner-assurance-109-observations.jsonl), [stderr](owner-assurance-109-observations-stderr.txt) |
| Assigned-base regression and recovery | 1 replay, exit 0; base bug reproduced, current recovery and unchanged invalid-input journal verified | [base replay](owner-assurance-109-base-regression.json), [stderr](owner-assurance-109-base-regression-stderr.txt) |

Focused counts: 133 owner-daemon, 52 owner-TMUX, 38 feed, 22 attention,
39 control, 8 promotion and 4 preflight. Focused and targeted runs overlap
full discovery; counts are executions, not additional unique tests. Each test
owns disposable stores/processes. All injected observations and SDK data are
synthetic. The common transport fixture now declares an explicit fresh
supervisor observation; missing-observation tests remove it and separately
exercise missing, malformed and unreadable states.

## Nonzero accounting

Final accounting: **1 nonzero top-level read-only inspection command**,
**0 nonzero test/evidence gates**, **0 unittest failures**, **0 errors**,
**0 skips**, **0 tool rejections**. Failure names: **none**.

One initial read-only inspection command exited 1 because `rg --files -g AGENTS.md scripts qa` found no nested AGENTS files.
The preceding policy read, status and HEAD commands succeeded. This was not a
test failure. Expected negative-path child statuses reported inside passing
tests (including the preflight rehearsal recipe exit 2) are not top-level gate
failures. No raw attempts were overwritten and no test retry was performed.

## Source/test line changes

Relative to assigned base `bc4493331f390801ef14016caeceae5c2ca94bab`:

| File | Added | Deleted | Location | SHA-256 |
| --- | ---: | ---: | --- | --- |
| `scripts/initiative_control/decision_manager.py` | 2 | 0 | line 185 | `954469ca9acdd961f7960761260fe38d3c37bdfd9f32c87266e6c0e24eccba02` |
| `scripts/initiative_control/slack_transport.py` | 18 | 8 | lines 749 and 786 | `ca30056fa9cf03321afa486d1281e304ab516dc690052ed3310d1cece349b03c` |
| `scripts/initiative_control/test_decision_manager.py` | 41 | 0 | lines 1412 and 1461 | `78464c6e74caec2693e64fe8bf8a0b99fbae46cfd59a3fbbce0887f76f3353a1` |
| `scripts/initiative_control/test_slack_transport.py` | 67 | 0 | lines 296 and 1627 | `4b003590995baa92063ff2affb6eadec73ecc638642a088638207cead08c0557` |
| Total | 128 | 8 | | |

All additional files are new `owner-assurance-109-*` evidence or
`owner_assurance_109_*` harnesses under the allowed management-bootstrap
folder. No paths outside the dispatch allowlist changed.

See [the return](owner-assurance-109.md) for remaining renderer/feed blockers,
exact notices, duplicate pairs, operator recovery and the absence of packaged
TUI/code-blind/human qualification. Passing automated checks do not qualify
those unresolved behaviors.
