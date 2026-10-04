# owner-surface-110 verification

Assigned base and starting HEAD:
`932d2e4be7f2eb85cad69357ba2c411f80e3f72f`.
Brief SHA-256 verified before all other file reads:
`b6160c29ce10b6c2ad241cc2eb2a78c1e219dc24c603ff3eabef90762d0cdb63`.
Read `docs/development/test-isolation.md` before any tests.

The venv was created under `env -i` using
`/opt/homebrew/bin/python3 -m venv /private/tmp/owner-surface-110.5dukCx/venv`.
Requirements were installed under `env -i` from
`scripts/initiative_control/requirements.txt`; see
[installation log](owner-surface-110-venv.txt). Both commands received only
`HOME=/private/tmp/owner-surface-110.5dukCx/home`,
`PATH=/opt/homebrew/bin:/usr/bin:/bin`, and `TMPDIR=/private/tmp`.

Every Python test/evidence command ran from the repository root with
`env -i` and only these variables:

```text
HOME=/private/tmp/owner-surface-110.5dukCx/home
CODEX_HOME=/private/tmp/owner-surface-110.5dukCx/home
CORBANU_HOME=/private/tmp/owner-surface-110.5dukCx/home
PFTERMINAL_HOME=/private/tmp/owner-surface-110.5dukCx/home
CORBANU_TEST_NO_NATIVE_KEYRING=1
TMPDIR=/private/tmp
PATH=/opt/homebrew/bin:/usr/bin:/bin
PYTHONPATH=scripts/initiative_control:qa/initiative-control/management-bootstrap
PYTHONDONTWRITEBYTECODE=1
```

Below, `python` means
`/private/tmp/owner-surface-110.5dukCx/venv/bin/python -B`.

```sh
python -m unittest -v \
  test_decision_manager.ManagerTests.test_expired_history_is_not_outstanding_and_review_does_not_discard_held \
  test_slack_transport.TransportTests.test_off_missing_scopes_wrong_identity_and_stale_binding \
  test_slack_transport.TransportTests.test_qualification_restores_prior_hold_after_interrupt_auth_and_final_write_failure \
  test_slack_transport.TransportTests.test_qualification_single_flight_across_instances_and_processes \
  test_slack_transport.TransportTests.test_failed_qualification_preserves_newer_fault_hold \
  test_slack_transport.TransportTests.test_initial_qualification_failures_retry_after_real_reload_without_reset \
  test_slack_transport.TransportTests.test_review_rechecks_undrained_events_after_auth \
  test_decision_feed.FeedTests.test_saved_verified_snapshot_downgrades_without_reprojection_or_mutation \
  test_attention.SlackNoticeTests \
  test_decision_manager.ManagerTests.test_supervisor_uncertainty_downgrades_fresh_projection_and_rendering
python -m unittest discover -v -s scripts/initiative_control -p 'test_*.py'
python -m unittest -v test_owner_daemon test_owner_tmux test_decision_feed test_attention test_control test_owner_promotion_94 test_owner_preflight_98
node scripts/initiative_control/test_facilities_js.cjs
python qa/initiative-control/management-bootstrap/owner_surface_110_observe.py
python qa/initiative-control/management-bootstrap/owner_surface_110_interrupt.py
git diff --check
git status --short
```

The Node program used the same empty-inherited environment. Full discovery and
the focused suite ran concurrently, each test using its own disposable stores.
No production source was edited after targeted run 2 began. Two old test
expectations were corrected after full discovery attempt 1; no production/test
source was edited after final targeted run 3 began.
There was no workspace formatter, Rust test, live profile, credential-store
access, native prompt, live listener/journal/coordinator access, commit or push.

## Results

Targeted run 2: **9 passed**, 10.976s, exit 0.
Final targeted run 3: **11 passed**, 12.177s, exit 0.
JavaScript renderer: **1 program passed**, exit 0.
Rendered observations: **7 captured**, exit 0 (six requested situations and
the saved-snapshot aging regression).
Interrupted qualification replay: **1 passed** (null → qualifying → null;
reopened journal unchanged; successful retry), exit 0.

Focused owner/feed/attention/control/promotion/preflight: **299 passed**,
102.898s, exit 0. Counts by module: owner-daemon 133, owner-TMUX 52,
feed 39, attention 24, control 39, promotion 8, preflight 4.
These runs overlap discovery; they are executions, not additional unique tests.
Full discovery attempt 2: **872 passed**, 492.412s, exit 0, **0 failures /
0 errors / 0 skips**. Raw final gate: [owner-surface-110-suite-2.txt](owner-surface-110-suite-2.txt).
Final targeted, focused, renderer and evidence commands also exited 0.
Final nonzero accounting remains **3 developmental test commands**, **1
read-only shell command**, and **1 no-op structured-edit rejection**; final
failure names: **none**.

The table consistency check passed: all six report rows exactly match captured
rendered strings, and there are no duplicate pairs even with timestamps removed.
See `owner-surface-110-table-check.json`.

## Preserved nonzero attempts

- `owner-surface-110-targeted-1.txt`: **69 tests**, exit 1,
  **1 failure / 2 subtest errors**, 57.737s.
- `owner-surface-110-diagnostic-1.txt`: **2 tests**, exit 1,
  **1 failure / 2 subtest errors**, 2.619s.

Those two runs have the same exact failure names:

- `test_slack_transport.TransportTests.test_qualification_restores_prior_hold_after_interrupt_auth_and_final_write_failure`,
  subtests `prior=None, failure='final-write'` and
  `prior='ingress-held', failure='final-write'`: the fixture initially expected
  raw OSError, but Store.lock intentionally converts that error to
  `decisions.Invalid`. The accepted exception was corrected; the test still
  verifies exact journal restoration after the failed final write.
- `test_slack_transport.TransportTests.test_review_rechecks_undrained_events_after_auth`:
  the old assertion expected a pinned non-null hold. It now checks restoration
  of the prior hold and separately verifies that the new undrained event and
  ingress evidence survive, with no gap-review or verification advancement.

Full discovery attempt 1 (`owner-surface-110-suite.txt`): **872 tests**,
492.477s, exit 1, **2 failures / 0 errors**. Exact names and resolutions:

- `test_decision_manager.ManagerTests.test_expired_history_is_not_outstanding_and_review_does_not_discard_held`:
  its old assertion required omission of expiry history. It now asserts the
  precise projection: expired=1 with outstanding count=0 and held=0.
- `test_slack_transport.TransportTests.test_off_missing_scopes_wrong_identity_and_stale_binding`:
  its old assertion expected a failed renewal to pin the previously qualified
  journal. It now requires exact unchanged journal state, including no new
  verification, and an unchanged prior hold. The rotated-binding refusal and
  OFF credential guard remain. Existing qualification retains its original
  expiration; failed renewal does not grant a new one.

Total nonzero test commands: **3**. Across these preserved runs: **4 failure
occurrences and 4 subtest error occurrences**; final results are separate.

The initial 69-test run is developmental evidence only: minor freshness-label
edits landed while it ran. Targeted run 3 and full discovery attempt 2 use the
final production/test tree. The passing focused suite used identical production
code; only assertions in two modules outside its selection changed afterward.
The diagnostic and original attempts remain unmodified.

Other nonzero operations: **1 read-only shell command** exited 1 when
`rg --files -g AGENTS.md scripts qa` found no nested instructions;
**1 structured-edit rejection** for an accidental identical old/new string
(no file change). No approval rejection occurred.

## Changed source/test lines

Against the assigned base:

| File | Added | Deleted |
| --- | ---: | ---: |
| attention.py | 42 | 4 |
| decision_feed.py | 18 | 4 |
| decision_manager.py | 9 | 3 |
| slack_transport.py | 24 | 2 |
| test_attention.py | 26 | 0 |
| test_decision_feed.py | 42 | 1 |
| test_decision_manager.py | 3 | 2 |
| test_slack_transport.py | 87 | 4 |
| Total | 251 | 20 |

All paths above are under `scripts/initiative_control/`; all new artifacts are
under the allowed `qa/initiative-control/management-bootstrap/` directory.
Control and owner-daemon source needed no edits: their focused tests cover the
shared saved-feed and renderer paths.

SHA-256 of the final source/test tree:

```text
7a79a53c2a1adab7e186d17fc26733c4c17457a05141a07ec0ba01d54f2e0af8  attention.py
b04b6e60a75ae5cf2c90b11bbfdc82e030dfc1949a972a307544ccfcf422db97  decision_feed.py
8c490cd590ddc66c1e74fbb1745d42af473d25c6b55568bd506389cefaa316a6  decision_manager.py
d2a68e13a0a99ff5f687961645db69c889d25b9c307c292b6660a4b459b0bbd6  slack_transport.py
68168fbd0f62f567d9b3e2bca63deafd848d06163747375a5de06f585c03fb39  test_attention.py
d3eab57599b0d07b9cad54baa3388d91fa173751c0a0c2886a28c10bd4757ca1  test_decision_feed.py
e7f49793f4f5225b3347f002f8ade52221f7229861c82c484af04dbf46b344c6  test_decision_manager.py
d42b034bcffa4418643d2e2deb91844b62abf8379f8d585b1ecdb11e3eef73c0  test_slack_transport.py
```
