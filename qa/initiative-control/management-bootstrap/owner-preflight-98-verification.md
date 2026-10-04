# owner-preflight-98 verification

Read `docs/development/test-isolation.md` before starting the campaign. Created
`/private/tmp/owner-preflight-98.npgR0E/venv` under `env -i`, with disposable HOME,
from exactly `scripts/initiative_control/requirements.txt`: markdown-it-py 3.0.0,
mdurl 0.1.2, slack-sdk 3.44.1. All commands run from the assigned repository root.
No Rust code/tests, workspace formatter or fix tool was used.

Every Python test command used `env -i` and this environment:

```text
HOME=/private/tmp/owner-preflight-98.npgR0E/home
CODEX_HOME=/private/tmp/owner-preflight-98.npgR0E/home
CORBANU_HOME=/private/tmp/owner-preflight-98.npgR0E/home
PFTERMINAL_HOME=/private/tmp/owner-preflight-98.npgR0E/home
CORBANU_TEST_NO_NATIVE_KEYRING=1
TMPDIR=/private/tmp
PATH=/opt/homebrew/bin:/usr/bin:/bin
PYTHONPATH=scripts/initiative_control:qa/initiative-control/management-bootstrap
PYTHONDONTWRITEBYTECODE=1
```

Use `/private/tmp/owner-preflight-98.npgR0E/venv/bin/python` for `python` below.
The tests use their existing synthetic profiles and disposable state; none of
these passing fixtures qualifies real worker inference or code-blind acceptance.

| Check | Command after isolated environment | Result / raw evidence |
| --- | --- | --- |
| Worked example and migration semantics | `python -B -m unittest -v test_owner_preflight_98` | 4 passed in 0.227s, exit 0; [raw log](owner-preflight-98-example-tests.txt) |
| Full requested discovery | `python -B -m unittest discover -v -s scripts/initiative_control -p 'test_*.py'` | **831 passed**, 446.477s, exit 0, no failures/errors/skips; [raw log](owner-preflight-98-suite.txt) |
| Owner, feed, renderer and promotion checks | `python -B -m unittest -v test_owner_daemon test_owner_tmux test_decision_feed test_attention test_owner_promotion_94 test_owner_preflight_98` | **257 passed**, 101.023s, exit 0, no failures/errors/skips; [raw log](owner-preflight-98-focused.txt) |
| Facilities JavaScript renderer | `node scripts/initiative_control/test_facilities_js.cjs` | One regression program passed, exit 0; [raw log](owner-preflight-98-renderer.txt) |

The Node check also ran under `env -i`, disposable HOME and the PATH above.
The new Python file passed AST parsing. Source formatting was complete before
tests; no formatting or runtime changes followed.

## RealTmux case accounting

Every `test_fable_launcher.RealTmux` case passed in the full run (11 total):

- `test_auth_failure_inactive_provider_and_stale_screen`
- `test_auth_rotation_before_child_is_rejected`
- `test_exception_after_session_creation_cleans_up`
- `test_forced_cleanup_stops_owned_descendant`
- `test_late_abort_invalidates_candidate_and_unrelated_process_survives`
- `test_normal_management_vocabulary_round_trip`
- `test_sigterm_returns_receipt_and_stops_tmux`
- `test_stdin_only_token_and_interrupted_buffer_cleanup`
- `test_timeout_and_partial_final_are_failed_attempts`
- `test_two_fresh_actual_tmux_runs`
- `test_wrong_metadata_secret_and_startup_exit`

The brief did not identify the five previously failing cases by name, so all 11
are recorded rather than guessing that subset. Sampled one-minute host loads
during the full run were 14.40625 and 9.7578125. This passing run does not refute
the reported failures at load 30 or establish load-independent reliability.
No retries or suppressed failures were needed.

## Nonzero and effect accounting

Focused breakdown: 133 owner daemon, 52 owner TMUX, 38 feed, 22 attention/renderer,
8 round-94 preflight and 4 round-98 worked-example cases. Counts overlap with
full discovery; they are not claimed as 1,088 unique tests. Failure names: **none**.

- **0 nonzero top-level verification commands**: example, full, focused and Node
  runs all exited 0. No failures/errors/skips in any unittest run; no retry.
- **2 nonzero top-level inspection commands**: an `rg --files -g AGENTS.md scripts
  qa` found no nested instructions (exit 1), and an `rg` lookup included nonexistent
  `scripts/initiative_control/preparation.py` (exit 2). A separate multi-command
  read also included a failing `sed` of that nonexistent path (embedded exit 2;
  the enclosing command exited 0). Thus **3 individual inspection subcommands**
  returned nonzero; none modified state or affected verification.
- Existing suite negative-path subprocesses/assertions remain in the raw logs;
  their expected refusals are not top-level verification failures or live effects.
- **0 live promotions, 0 real worker launches, 0 real ACK/START/RETURN, 0 native
  credential prompts, 0 new approvals, 0 commits, 0 pushes.** Synthetic test
  operations are not counted as live operations.

## Changed lines and scope

All seven additions are under `qa/initiative-control/management-bootstrap/`.
There are **0 production-code changes, 0 tracked-file modifications and 0 deletions**.
All existing path, preparation, allocation and promotion validators stay unchanged.

| New file | Added lines |
| --- | ---: |
| `owner-preflight-98.md` | 298 |
| `test_owner_preflight_98.py` | 95 |
| `owner-preflight-98-example-tests.txt` | 9 |
| `owner-preflight-98-focused.txt` | 289 |
| `owner-preflight-98-renderer.txt` | 1 |
| `owner-preflight-98-suite.txt` | 886 |
| `owner-preflight-98-verification.md` | 100 |
| **Total** | **1678** |

`git diff --check` passed; because these are new files, the final scope check also
checks the Markdown/Python additions directly for trailing whitespace and final
newlines, parses the Python AST, and verifies every status path against the
assigned writable scope. No workspace-wide formatting was run.
