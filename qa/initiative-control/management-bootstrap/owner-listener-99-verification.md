# owner-listener-99 verification

Read `docs/development/test-isolation.md` before tests. The venv was created
under `env -i` at `/private/tmp/owner-listener-99.lUTMJD/venv`, with an empty
disposable HOME. Installed exactly `scripts/initiative_control/requirements.txt`:
markdown-it-py 3.0.0, mdurl 0.1.2, slack-sdk 3.44.1. Commands ran from the assigned
repository root. No Rust tests, workspace formatter, fix tool, native credential
prompt, live profile, live Slack connection or live-store operation.

Every test used `env -i` with:

```text
HOME=/private/tmp/owner-listener-99.lUTMJD/home
CODEX_HOME=/private/tmp/owner-listener-99.lUTMJD/home
CORBANU_HOME=/private/tmp/owner-listener-99.lUTMJD/home
PFTERMINAL_HOME=/private/tmp/owner-listener-99.lUTMJD/home
CORBANU_TEST_NO_NATIVE_KEYRING=1
TMPDIR=/private/tmp
PATH=/opt/homebrew/bin:/usr/bin:/bin
PYTHONPATH=scripts/initiative_control:qa/initiative-control/management-bootstrap
PYTHONDONTWRITEBYTECODE=1
```

Use `/private/tmp/owner-listener-99.lUTMJD/venv/bin/python -B` for `python`
below. The tests use synthetic credentials and disposable stores. Sources for
the four production/test modules were fixed before the final full discovery and
remained unchanged during that run. The separate QA reproduction harness was
strengthened from write-failure injection to actual flock contention and rerun;
it is not imported by discovery.

## Campaign results

| Check | Result | Evidence |
| --- | --- | --- |
| First targeted regressions | 7 tests, 7 passed, 7.707s, exit 0 | [raw](owner-listener-99-regression-first.txt) |
| Exploratory affected modules | 157 tests, six failure records across five methods, 232.559s, exit 1 | [raw](owner-listener-99-affected-first.txt) |
| Final targeted regressions | 11 tests, 11 passed, 11.311s, exit 0 | [raw](owner-listener-99-regression-final.txt) |
| Final full discovery | **839 tests, 839 passed**, 451.554s, exit 0, no failures/errors/skips | [raw](owner-listener-99-suite.txt) |
| Owner/feed/renderer/promotion checks | **257 tests, 257 passed**, 103.066s, exit 0, no failures/errors/skips | [raw](owner-listener-99-focused.txt) |
| Facilities JavaScript renderer | One regression program passed, exit 0 | [raw](owner-listener-99-renderer.txt) |
| Base/revised reproduction, injected write failure | Four child scenarios, expected results, harness exit 0 | [first](owner-listener-99-reproduction.jsonl), [final-source replay](owner-listener-99-reproduction-final.jsonl) |
| Base/revised reproduction, actual flock contention | Four child scenarios per run, expected results, both harnesses exit 0 | [first flock run](owner-listener-99-reproduction-lock.jsonl), [final measured callback replay](owner-listener-99-reproduction-observed.jsonl) |

Full discovery command:

```sh
python -m unittest discover -v -s scripts/initiative_control -p 'test_*.py'
```

Owner/feed/renderer/promotion command:

```sh
python -m unittest -v test_owner_daemon test_owner_tmux test_decision_feed test_attention test_owner_promotion_94 test_owner_preflight_98
```

The Node command was `node scripts/initiative_control/test_facilities_js.cjs`.
The reproduction command was
`python qa/initiative-control/management-bootstrap/owner_listener_99_reproduce.py`.
Final targeted command names are retained verbatim by unittest in its raw log.

Counts overlap; focused runs are not additional unique tests. Focused breakdown:
133 owner daemon, 52 owner TMUX, 38 feed, 22 attention/renderer, 8 round-94
promotion and 4 round-98 preparation cases. The full suite includes eight new
regression methods relative to the 831-test base.

## Preserved failed exploratory run

No failure was overwritten or relabeled as passed. Exact failure names:

1. `test_slack_transport.TransportTests.test_session_generation_duplicate_conflict_and_unbound_thread_hold`
2. `test_decision_manager.ManagerTests.test_dedicated_exec_rejects_wrong_root_and_inherited_descriptor_before_ready`, subcase `wrong='root'`
3. The same method, subcase `wrong='descriptor'`
4. `test_decision_manager.ManagerTests.test_failure_before_handshake_survives_failed_start_cleanup`
5. `test_decision_manager.ManagerTests.test_real_child_failure_record_survives_reap_and_projects_to_dashboard`
6. `test_decision_manager.ManagerTests.test_unbound_follower_reply_is_never_attributed_to_parent`

The two unbound-thread tests expected the old intake shutdown/no-ACK behavior.
Their replacements assert durable quarantine/ACK/continued intake, retain the
hold, and still prove that no reply is attributed to the wrong decision.
The two dedicated-exec subcases expected empty stdout; they now assert the
fixed failure frame and unchanged transport state.

The two new frame tests failed in a **mixed-source exploratory run**: while it
was running, fallback counters were corrected from misleading zero to null for
unknown counts. Its already-imported parent rejected the new child's null
schema and fell back to child-unreported. Those failure records remain intact.
A fresh final targeted run passed all eleven affected regressions. The final
full suite uses one fixed source version; this is also why coordinated
supervisor/child adoption matters.

## Nonzero accounting

- **One nonzero top-level test command**: the exploratory affected run,
  exit 1, six failure records, zero errors. **Zero nonzero final gate commands**:
  final targeted, full discovery, owner/feed/renderer/promotion and Node all
  exited 0, with no unittest failures/errors/skips. No failed final gate was retried.
- Each of the four reproduction invocations deliberately produced two exit-1
  children (base/revised “three” cases): eight expected child exits in total,
  distinct from top-level harness failures. All four harness commands exited 0.
  Other unit-suite negative-path subprocesses remain asserted by their tests;
  they are not reclassified as verification command failures.
- Three nonzero top-level read-only inspection commands, plus one embedded
  nonzero search: two nested-AGENTS searches found no files, one attention.py
  listener-field search found no match, and an interim failure-marker search
  found no failures. These four individual searches returned 1 and changed
  nothing.
- Zero native credential prompts, live listener restarts/stops, live journal
  edits, live coordinator mutations, real messages, new approvals, commits or
  pushes.

## Source identity and changed lines

| File | Added | Deleted | Final SHA-256 |
| --- | ---: | ---: | --- |
| scripts/initiative_control/decision_manager.py | 62 | 8 | `aea5afa32253ce7b4444fc1f245f80f4bd3484165371aefb2376056c18640c41` |
| scripts/initiative_control/slack_transport.py | 128 | 9 | `d4034bbc4f61b46c8ca6352350419248c95c6dd852bbe24711b651b1cef3a83a` |
| scripts/initiative_control/test_decision_manager.py | 89 | 4 | `1aef3691529a4701bafc5d1f73730c019bf05023dec26ff96cee8fb676ebfff9` |
| scripts/initiative_control/test_slack_transport.py | 92 | 6 | `ab6417e2255c3e51aaa5fa105e94f0bd1eed9d5820409b93b3ad05c5e8343485` |
| **Tracked total** | **371** | **27** | |

Reproduction harness SHA-256:
`f5e68185d166c3a89069f03727300b487251f55d7e22c6eecd4dd2349245dcfc`.

All new files are under `qa/initiative-control/management-bootstrap/`:

| New file | Added lines |
| --- | ---: |
| owner-listener-99.md | 244 |
| owner_listener_99_reproduce.py | 111 |
| owner-listener-99-affected-first.txt | 256 |
| owner-listener-99-focused.txt | 289 |
| owner-listener-99-regression-final.txt | 16 |
| owner-listener-99-regression-first.txt | 12 |
| owner-listener-99-renderer.txt | 1 |
| owner-listener-99-reproduction.jsonl | 4 |
| owner-listener-99-reproduction-final.jsonl | 4 |
| owner-listener-99-reproduction-lock.jsonl | 4 |
| owner-listener-99-reproduction-observed.jsonl | 4 |
| owner-listener-99-suite.txt | 894 |

This verification record adds 144 lines; QA additions total 1,983 lines.
All changes total **+2,354 / -27** across 17 files. Final source AST,
whitespace/newline, hash and scope checks passed: four modified Python files
and thirteen new QA files, all in scope. No workspace formatting ran.
