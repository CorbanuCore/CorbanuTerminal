# owner-recover-101 verification

Read `docs/development/test-isolation.md` before testing. The disposable venv
was built under `env -i` at
`/private/tmp/owner-recover-101.1v5mru/venv` from exactly
`scripts/initiative_control/requirements.txt` (markdown-it-py 3.0.0,
mdurl 0.1.2, slack-sdk 3.44.1). All commands ran from the assigned repository root.

Every Python/Node verification command used an empty environment plus:

```text
HOME=/private/tmp/owner-recover-101.1v5mru/home
CODEX_HOME=/private/tmp/owner-recover-101.1v5mru/home
CORBANU_HOME=/private/tmp/owner-recover-101.1v5mru/home
PFTERMINAL_HOME=/private/tmp/owner-recover-101.1v5mru/home
CORBANU_TEST_NO_NATIVE_KEYRING=1
TMPDIR=/private/tmp
PATH=/opt/homebrew/bin:/usr/bin:/bin
PYTHONPATH=scripts/initiative_control:qa/initiative-control/management-bootstrap
PYTHONDONTWRITEBYTECODE=1
```

`python` below means
`/private/tmp/owner-recover-101.1v5mru/venv/bin/python -B`.
Stores, profiles, child listeners and SDK credentials were disposable fixtures.
There were no Rust tests, live store/profile operations, native credential
prompts, workspace format/fix runs, commits or pushes.

## Results

| Check | Result | Raw evidence |
| --- | --- | --- |
| First targeted recovery regressions | 12 passed, 15.516s, exit 0 | [targeted](owner-recover-101-targeted-first.txt) |
| Final full discovery | **850 passed**, 469.030s, exit 0; no failures/errors/skips | [full suite](owner-recover-101-suite.txt) |
| Final owner/feed/renderer/promotion checks | 257 passed, 99.336s, exit 0 | [focused](owner-recover-101-focused.txt) |
| Facilities JavaScript renderer | One regression program passed, exit 0 | [renderer](owner-recover-101-renderer.txt) |
| Actual flock reproduction | Four scenarios, harness exit 0; two expected exit-1 children | [reproduction](owner-recover-101-reproduction.jsonl) |
| Source/patch checks | Four Python ASTs valid; git diff --check and unapplied renderer git apply --check passed | Commands below |

Full discovery:

```sh
python -m unittest discover -v -s scripts/initiative_control -p 'test_*.py'
```

Owner/feed/renderer/promotion:

```sh
python -m unittest -v test_owner_daemon test_owner_tmux test_decision_feed test_attention test_owner_promotion_94 test_owner_preflight_98
node scripts/initiative_control/test_facilities_js.cjs
```

Reproduction:

```sh
python qa/initiative-control/management-bootstrap/owner_listener_99_reproduce.py
```

Read-only source/patch checks:

```sh
git diff --check
git apply --check qa/initiative-control/management-bootstrap/owner-recover-101-renderer-followup.patch
git status --short
```

Python AST checks parsed only the four changed source/test modules. No formatter
rewrote other paths. Source edits finished before final discovery and the focused
campaign; sources stayed fixed during those runs. The earlier targeted run
preceded the last validation hardening and two additional regression cases;
its evidence is retained, not presented as the final-tree gate.

Focused counts overlap discovery: 133 owner-daemon, 52 owner-TMUX, 38 feed,
22 attention/renderer, 8 promotion and 4 preflight. They are not additional
unique tests. The unapplied HTML follow-up is not part of the tested tree.

## Nonzero accounting

**Zero nonzero top-level test/harness commands; zero unittest failures, errors
or skips; failure names: none.** Full discovery passed 850 tests, including 11
new regression methods (8 transport, 3 manager) relative to the 839-test base.
The focused campaign passed 257 tests; the earlier targeted run passed 12. The reproduction's base/revised
contention children intentionally exited 1 (two expected nonzero child exits);
its poison cases exited 0. Other negative-path child exits in unit tests are
asserted by their enclosing cases, not top-level verification failures.

Three read-only inspection commands returned nonzero: one absent renderer
filename glob returned 2; one no-match JavaScript-field search returned 1;
one no-match reply-validation function search returned 1. One orchestration
script was rejected for JavaScript syntax before execution and made no edits;
it was corrected using valid JavaScript string literals. No failed experiment
or command was overwritten, relabeled as passed, or used as acceptance evidence.

## Changed source lines and identity

Relative to starting HEAD
`181b2d72fa02e4208d5472d39f55924e7bc9a34c`:

| File | Added | Deleted | SHA-256 |
| --- | ---: | ---: | --- |
| scripts/initiative_control/decision_manager.py | 50 | 6 | `b12ddf6047544cc243608708b5ab316d1a445b3d8a0912a501a546a70319b8b6` |
| scripts/initiative_control/slack_transport.py | 128 | 26 | `b273de47e628d2d8893c560d8ec2e2d47698483aa97c4c0645461120b5a2bcae` |
| scripts/initiative_control/test_decision_manager.py | 57 | 1 | `0b11c476ac889f89cb8c925af2e29773a99f3b3bc96a7e6d76f8c4ace816489c` |
| scripts/initiative_control/test_slack_transport.py | 122 | 2 | `a413992a6695c262b8fcab6a4ca736c09980c1ed3975d9576edf717d2d33e879` |
| Total source/tests | 357 | 35 | |

QA additions (all under `qa/initiative-control/management-bootstrap/`):

| Artifact | Added lines |
| --- | ---: |
| owner-recover-101-focused.txt | 289 |
| owner-recover-101-renderer-followup.patch | 16 |
| owner-recover-101-renderer.txt | 1 |
| owner-recover-101-reproduction.jsonl | 4 |
| owner-recover-101-suite.txt | 903 |
| owner-recover-101-targeted-first.txt | 17 |
| owner-recover-101.md | 157 |
| owner-recover-101-verification.md | 122 |

QA total: +1,509 lines. All-files total: **+1,866 / -35**. Twelve changed/new files, all
within the writable allocation. The renderer patch remains an inert review
artifact; `attention.py` is unchanged.
