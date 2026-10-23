# RETURN — owner-derived-89

Allocation digest: `967f867ea3659bfc1e5a513720c4e33ab4ae012b36d6a99b85b7b1399bc7855d`.
Claim: `923e340e-e56b-449a-8733-58eeb3411649`.
Worker: GPT-6 Astra, high effort.
Base: `7b4fb5b07cd26cbb17f47f9da5ec71940854448a`.
Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/owner-recurrence-20260917`.
Branch: `bootstrap/owner-recurrence-20260917`.
Frozen brief SHA-256 verified:
`30337b6ae0cfa0051708614a036beb496f95714243b0e7516207673fdf205092`.

## Scope and authority

Bounded fix restoring truthful advisory reporting for the existing watchdog.
Product specification heading **Internal delivery control — TO BUILD**:
“durable event dispatch, acknowledgments and watchdog” and “One manager owns
plan records and publication; workers emit separate redacted reports.”
Context: active initiative-delivery-control plan, PF-80-S01 `in_progress`;
this frozen worker allocation supplies the exact worktree/base/scope.
No new ownership, activation or credential authority is added.

No live owner root or live coordinator was accessed. No real auth/token
contents, native credentials, inference, Slack, push or release were used.
Changes are confined to the allocated source files and this QA directory.
This is an internal implementation return, not an unqualified human-test or
release handoff. Independent functional acceptance and real-worker qualification
remain manager-owned; no new integrator N/A acceptance is invented here.
TensorCash/Isometric runs, human sign-off and release benchmarks are not claimed.

## Derived coverage

`manager_covered` calls
`Coordinator.watchdog_covers(snapshot, None, dispatcher)`.
The canonical predicate treats `action=None` as the unpartitioned manager.
`watchdog_manager_reportable` uses that same predicate plus the shared
`watchdog_due` deadline/unreported check. Both the watchdog's initial check
and its transactional recheck call it. Manager predictions and the summary
therefore follow actual execution, including a changed manager coverage policy.

Action counts first use `watchdog_action_reportable(action, observed_at)`,
the same eligibility predicate execution uses, then partition with
`watchdog_covers`. Counted actions must be dispatching/dispatched/running,
strictly overdue and not already reported. Prepared, returned, uncertain,
terminal, already-reported, future-deadline and exact-deadline actions do not
count. The `counted_actions` field and summary state this scope.
The detailed ownership map still contains actions outside those counts.

The expanded regression fixture contains 15 actions; exactly **1 covered**
(`owner-overdue`) and **2 excluded** (`default-hand`, `hand-overdue`) count.
Execution emits the covered action's stall and the manager stall. A subsequent
hand watchdog reports exactly the two excluded actions.

Hand-stall guidance also calls the shared lane predicate with a hand-owned
probe; it no longer restates the current ownership policy in summary code.
`manager_covered` describes lane policy even if no manager currently exists;
the separate manager row predicts whether a current manager would report.

## Stable hold rows

Every unresolved hold row has these eight keys, including failed coordinator
reads:

```text
op_id, reason_code, first_seen, last_seen, action_id,
dispatch_owner, excluded_from_owner_lane, ownership_status
```

`ownership_status` is `resolved`, `action_missing`, `unmapped`,
`coordinator_unavailable`, or `not_checked` (raw journal/tick reads).
Here “resolved” refers only to the ownership lookup: the hold is still unresolved.
Unknown ownership/exclusion values are `null`, never omitted or asserted false.
If the owner journal itself is unavailable, top-level `unresolved_holds`
remains `null` with an explicit component error; that is not an empty hold list.

Regression checks compare row key sets across successful and failed reads and
across missing and subsequently found actions.

## Scratch predicate control

[Control runner](owner_derived_89_control.py) copies modules into disposable
packages, changes only the coordinator lane predicate, and runs fresh child
interpreters against synthetic state. It compares summary action IDs and
manager prediction with actual watchdog events.

[Final control](owner-derived-89-control-final.json): PASS, three variants:

| Variant | Covered | Excluded | Manager covered | Actual emitted events |
| --- | ---: | ---: | --- | ---: |
| Baseline owner lane | 1 | 1 | true | 2 |
| Widen predicate to include hand actions | 2 | 0 | true | 3 |
| Exclude manager in predicate | 1 | 1 | false | 1 |

The widening also changes new-default coverage to true and hand-stall guidance
to “covered by this watchdog.” The manager variant changes the literal summary
to `manager=excluded` and emits no manager event.
Every package has the identical owner_daemon.py SHA-256:
`3812a2cca5e5cd70af836c795cf10a6a4971511a45ccf74fff95d72b5f80a36f`.
The control JSON records each different coordinator hash and scratch location.

Initial control failed before probing coverage: the synthetic action omitted
required `workstream`/ordering bookkeeping. Exit 1, baseline child exit 1,
`KeyError: 'workstream'` in `Coordinator._archive_actions`.
Original scratch package: `/private/tmp/od89-control-g8a5b1pk/baseline`.
Initial stdout is retained in owner-derived-89-control-first.stdout (empty); the error
was emitted on the tool's stderr. The final replay is a separate artifact.
This was a fixture defect, not evidence of a watchdog pass or failure.

## Promotion recipe and exact lines

The [updated executable recipe](owner-handoff-80-promotion.md) preserves the
existing installation/history and prints BEFORE, BEFORE ARM and AFTER from
computed status. It checks same-snapshot reported action IDs and exclusion
eligibility separately from the ownership map.

Exact synthetic control observations (not live measurements):

```text
BEFORE: Owner watchdog on admitted ticks (unreported overdue actions): covered=2, excluded=0; manager=covered; new default actions=covered; hand stall detection=covered by this watchdog.
AFTER: Owner watchdog on admitted ticks (unreported overdue actions): covered=1, excluded=1; manager=covered; new default actions=excluded; hand stall detection=manager responsibility (--hand-run is manual; no scheduled hand watchdog).
```

The [exact recipe replay](owner-derived-89-promotion.jsonl) also passed against
a disposable launchd installation with a real TMUX/PTY synthetic worker:
root `/private/tmp/op80-g9s9mk3f`, generation **3**, returned work artifact,
interval firing, `hold=null`, service **absent** after cleanup, outer exit **0**.
The intentional OFF observation returned exit **2 / owner_off** once, as expected.
Both printed action counts were **0 covered / 0 excluded** because the selected
work was prepared at handoff and was not overdue on the later read.
These are the exact replay lines:

```text
BEFORE: Owner watchdog on admitted ticks (unreported overdue actions): covered=0, excluded=0; manager=covered; new default actions=covered; hand stall detection=covered by this watchdog.
AFTER: Owner watchdog on admitted ticks (unreported overdue actions): covered=0, excluded=0; manager=covered; new default actions=excluded; hand stall detection=manager responsibility (--hand-run is manual; no scheduled hand watchdog).
```

Live counts cannot be supplied honestly without reading live state, which the
brief forbids. The recipe prints the actual integers; it does not substitute
the synthetic counts above. Counts can change between reads as actions expire,
report or finish; the intended policy change is fixture-wide to owner-lane
coverage.

Order: coordinate/quiesce new hand delivery → inspect → disarm → uninstall →
copy received runtime → OFF reconfigure with the existing schedule → freeze
and commit ownership handoff → inspect → repin/install OFF → observe owner_off →
arm with the real authorization → explicit schedule recovery → inspect again.
The exact commands and operator inputs are in the linked recipe.

On unexpected disagreement, stop. Before arm, leave admission OFF. After arm,
read current generation and disarm; retain both raw statuses, observation times,
revisions, pins and refusal output. Quiesce cooperating dispatchers, reconcile
individual statuses/deadlines/stall flags/owners, and re-read with the received
candidate. Do not edit SQLite or blindly rearm. Disarm does not kill existing
workers or undo ownership/history. Escalate unresolved differences to the
integration owner. A hand lane still requires manager-arranged stall detection;
manual `--hand-run` is not scheduled supervision.

## Brief corrections and limits

All three reported defects were confirmed. One additional defect in the prior
recipe became apparent: `covered_actions == selected` equated watchdog coverage
with selected prepared work. It is now replaced by comparisons against report
predictions, while selected ownership is checked against the ownership map.
No factual error was found in the brief. The exact lines above are explicitly
synthetic observations; live counts remain unknown under its no-live-read
boundary. The recipe prints live observations for the authorized manager.

Synthetic tests do not establish real Corbanu rollout provenance, auth-link
startup, model latency, ACK/START/RETURN or post-pane-death behavior. The prior
real-worker qualification limitation remains explicit in the promotion recipe.

## Verification

Disposable venv: `/private/tmp/owner-derived-89.DYYOlt/venv`, created under
`env -i` and installed only from
`scripts/initiative_control/requirements.txt` (markdown-it-py 3.0.0,
mdurl 0.1.2, slack-sdk 3.44.1). The test-isolation document was read first.
All test commands ran from the repository root with this clean environment:

```text
HOME=CODEX_HOME=CORBANU_HOME=PFTERMINAL_HOME=/private/tmp/owner-derived-89.DYYOlt
CORBANU_TEST_NO_NATIVE_KEYRING=1
TMPDIR=/private/tmp
PATH=/opt/homebrew/bin:/usr/bin:/bin
PYTHONPATH=scripts/initiative_control
PYTHONDONTWRITEBYTECODE=1
```

- [First focused attempt](owner-derived-89-focused.txt): **86 tests**, 3.010s,
  exit **1**, **0 failures / 1 error**:
  `ActivationVisibilityTests.test_hold_row_shape_for_missing_and_resolved_action`.
  The new fixture omitted `workstream`/ordering fields, causing
  `KeyError: 'workstream'` in coordinator archival.
- [First discovery](owner-derived-89-suite.txt): **824 tests**, 441.069s,
  exit **1**, **0 failures / 1 error**, the same named fixture defect.
  This process had already loaded the old test before the fixture was fixed;
  it is retained as a non-final attempt, not final-tree evidence.
- [Final focused](owner-derived-89-focused-final.txt):
  `python -B -m unittest -v test_owner_daemon.ActivationVisibilityTests test_coordinator`:
  **86 passed**, 4.705s, exit **0**, zero failures/errors/skips.
- [Final affected regression](owner-derived-89-regression.txt):
  `python -B -m unittest -v test_owner_daemon test_owner_tmux test_decision_feed test_attention`:
  **238 passed**, 103.438s, exit **0**, zero failures/errors/skips:
  **130 owner daemon + 48 owner TMUX + 38 feed + 22 attention/renderer**.
- [Final full discovery](owner-derived-89-suite-final.txt):
  `python -B -m unittest discover -s scripts/initiative_control -p 'test_*.py'`:
  **824 passed**, 442.779s, exit **0**, zero failures/errors/skips.

Unexpected top-level verification nonzero exits: **3** total, all retained:
the first focused run, the first full discovery, and the initial scratch-control
fixture failure. The two unittest runs named the same test; the scratch failure
was outside unittest. All three corrected replays pass. The recipe additionally
observed exactly **1 expected nonzero subcommand**, exit **2 / owner_off**;
the promotion harness itself exited **0**.

The [candidate manifest](owner-derived-89-candidate.json) records source, recipe
and control hashes and the package digest. All five hashes were rechecked and
remained unchanged after the final discovery, affected tests and recipe replay.
No formatter/fix tool ran.
Python AST validation passed for the four changed Python files; all **5**
promotion shell blocks passed `bash -n`; `git diff --check` passed.
The read-only sprint checker passed: **115 current / 127 archived**.

Tracked-file changed-line accounting (additions/deletions):

| File | Added | Deleted |
| --- | ---: | ---: |
| scripts/initiative_control/coordinator.py | 21 | 9 |
| scripts/initiative_control/owner_daemon.py | 26 | 13 |
| scripts/initiative_control/test_owner_daemon.py | 61 | 2 |
| qa/.../owner-handoff-80-promotion.md | 56 | 15 |

New QA files contain the **102-line** control runner, candidate manifest, this
return, raw logs and control/promotion artifacts. Source changes total **108
added / 24 deleted**, all within the frozen writable scope. Final
`git status --short` shows only the three allocated source files and this QA
directory. No commit or push was made.
