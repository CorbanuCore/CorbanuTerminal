# RETURN — owner-cutover-85

Allocation `owner-cutover-85`; digest
`48e3f4b4eb58005a2f60de45781dec9eb8a609122cce207a6853146a00a54235`;
claim `91880e90-1949-4813-94a5-dcfd95bdeda9`.
Worker: gpt-6-astra / high.
The first file read was the frozen brief, SHA-256 verified as
`76e178d5ac84d356a92a5bae109a7ed4535dd10fa7b2388637f82e3eedb34472`.
HEAD equals frozen base `08f42237c6462a2cb2015a9ee5dd1307858aa479`.

Bounded reliability/disclosure corrections to the authorized owner cutover.
Exact product heading **Internal delivery control — TO BUILD**:
“durable event dispatch, acknowledgments and watchdog” and “initialize
and rehearse all three workstreams before enabling recurring operation”.
The existing active plan is `initiative-delivery-control`, feature PF-80,
sprint `PF-80-S01 / in_progress`. The explicit frozen manager allocation
authorizes this worktree and file scope. Historical plan coordinates retain
base `08db99fff46fd22c582fbea0240fa379a42242e7`; shared plan/sprint changes
remain manager-owned. No broader product outcome or live cutover is claimed.

## Computed watchdog coverage and deliberate choice

The scheduled owner **does not watchdog hand-owned actions** in tmux-workers
mode. The manager owns stall detection for those actions and every future
default action. `--hand-run` runs a manual pass; it is not a recurring hand
watchdog. This preserves the accepted ownership boundary and prevents the
scheduled owner from changing an external hand claim to dispatch_uncertain.
The alternative global watchdog would change hand state from the owner lane;
this allocation explicitly allowed retaining separate responsibility.

`coordinator.watchdog_coverage` is computed from the same
`Coordinator.watchdog_covers` predicate used by the watchdog's snapshot and
transactional recheck. It lists covered/excluded action IDs, the new-action
default owner and its coverage, manager coverage, the admission condition,
the manager's responsibility and a computed summary. Coverage is routing,
not a promise that a terminal/prepared/future/already-reported action produces
a stall now. Each in-flight row separately computes `watchdog_covered`,
`watchdog_will_report` and resulting `watchdog_status`.
The report describes an admitted tick reaching the watchdog; OFF, refusal or
an earlier failure does not execute it. Fixture-only ticks retain all-action
coverage. Manager stalls remain covered in both lanes. All reads are advisory.

## Remaining review findings corrected at their claims

The full Round 80 review was located through
`/private/tmp/fmgr.Q1SIYZ/ownhand80-review.path`, pointing to
`/private/tmp/frev.u1IOyR/review.txt`: one P2 and three P3s.

- Hidden holds: activation status now returns every unresolved hold with op ID,
  action, reason, timestamps, current dispatch owner and whether it is excluded
  from the owner lane. Tick output also includes unresolved holds. If the owner
  read is unavailable, the field stays unknown (`null`), not an empty success.
  A hand hold remains visible while owner work can proceed. This does not
  resolve any hold; outstanding holds still block reconfiguration. The promotion
  runbook now says so and does not present schedule recovery as hold resolution.
- Reconfiguration with an installed schedule: `--reconfigure` now requires
  `--schedule`. Under the same installation lock used by install/repin it
  requires a recorded uninstalled receipt bound to the exact config path and
  absent services in both domains (an absent sibling GUI domain is allowed).
  It then takes the owner locks in the installer's order and applies the
  existing OFF/settled/quiescent checks. No receipt, no implicit unscheduled
  repin: the old two-argument invocation now refuses. This intentional stricter
  precondition is included in the corrected recipe.
- Synthetic-worker proof: the promotion recipe now explicitly names the missing
  real Corbanu rollout/auth-link/model-latency/ACK/START/RETURN/pane-death evidence.
  Synthetic tmux success cannot qualify those claims. A real-binary,
  disposable-root qualification (or explicitly limited first promoted-action
  evidence) remains manager-owned. No live credentials or model call was used
  to manufacture that evidence.

## Corrected promotion recipe

Use [the corrected exact command recipe](owner-handoff-80-promotion.md).
It reads the received candidate's status before disarming, so an older installed
status implementation cannot silently omit the new coverage fields.
Order: inspect → disarm → uninstall → copy received runtime → reconfigure with
`--schedule` → freeze/commit handoff → inspect coverage and holds → repin/install
OFF → observe owner_off → arm → explicit schedule recovery → inspect again.

The one field to read before and after is
`coordinator.watchdog_coverage.summary`:

- BEFORE (fixture-only): `Owner watchdog on admitted ticks: covered=N, excluded=0; new default actions=covered; hand stall detection=covered by this watchdog.`
- AFTER (tmux-workers): `Owner watchdog on admitted ticks: covered=N, excluded=M; new default actions=excluded; hand stall detection=manager responsibility (--hand-run is manual; no scheduled hand watchdog).`

N/M and exact covered/excluded IDs come from the current state. The recipe
prints BEFORE, BEFORE ARM and AFTER and asserts the corresponding conditions.
Review the excluded set before arming and arrange manual stall detection.
Disarm revokes admission; it is not rollback of history or unresolved holds.

## Evidence boundaries

No live owner root or live coordinator was read or mutated. No auth/token
contents, native credential store, real inference, Slack, push or release.
All test profiles are disposable under env -i. No Rust tests or workspace-wide
formatter. Independent code-blind acceptance, named-human acceptance,
live-repository qualification and release are not claimed by this revise-worker
return. This is an internal implementation/evidence return to the manager;
independent functional gates remain before any unqualified human-test handoff.

## Tests and retained nonzero attempts

Disposable venv `/private/tmp/owner-cutover-85.WdNw7u/venv` was created under
`env -i`, using only `scripts/initiative_control/requirements.txt`:
markdown-it-py 3.0.0, mdurl 0.1.2, slack-sdk 3.44.1.
All test commands run from the repository root with HOME and all three profile
aliases set to `/private/tmp/owner-cutover-85.WdNw7u`,
TMPDIR=/private/tmp, CORBANU_TEST_NO_NATIVE_KEYRING=1,
PATH=/opt/homebrew/bin:/usr/bin:/bin,
PYTHONPATH=scripts/initiative_control and PYTHONDONTWRITEBYTECODE=1.

- [First focused attempt](owner-cutover-85-focused-first.txt): 49 tests,
  2.970s, 1 failure and 1 error, exit 1. Failure:
  `ActivationVisibilityTests.test_status_lists_in_flight_and_overdue_with_exact_watchdog_effects`
  (case-sensitive “fixture-only” text). Error:
  `unittest.loader._FailedTest.OwnershipTests` (my command used a nonexistent
  class; the actual class is HandoffTests).
- [Second focused attempt](owner-cutover-85-focused-second.txt): 200 tests,
  20.561s, 0 failures and 1 error, exit 1:
  `HandoffTests.test_reconfigure_requires_off_and_preserves_old_history`
  (the new recorded-schedule fixture lacked installation.lock).
  Corrected the fixture. Neither prior log is overwritten.

[Final affected regression](owner-cutover-85-regression.txt): **236 passed**
in **101.454s**, zero failures/errors/skips, exit 0:
owner daemon 128, owner tmux 48, decision feed 38, attention/renderer 22.

[Concurrent rehearsal](owner-cutover-85-concurrent-first.jsonl): **PASS**, exit 0,
disposable root `/private/tmp/oc85-s_02a3sl`. Two dispatcher processes and two
real tmux/PTY synthetic workers, handoff revisions **10, 13, 14**, **22 applied
effects**, **zero holds**, unchanged worker claims, one prompt and one START
per worker, two completed work artifacts. **20 dispatcher observations** include
**7 BUSY** skips; skips are not counted as admitted successes.
Owner claim `b1c02270-cfe9-4b15-bdaf-d1d59564fee8`;
hand claim `f6db561b-9089-4c3c-824c-da9a8b5c6e79`.

The same run then stopped only the hand dispatcher and created a new external
hand claim with **no dispatch_owner key**, so ownership genuinely comes from
the post-cutover default. An explicit fixture mutation shortened its deadline
to two seconds; the run recorded status before it expired, waited for actual
wall-clock expiry, and recorded status plus an admitted owner pass afterward.
The owner recurrence process remained alive throughout. Coverage was
`covered_actions=["owner-work"]`,
`excluded_actions=["hand-work","overdue-hand"]`, default `hand` excluded.
For `overdue-hand`, `watchdog_covered=false` and
`watchdog_will_report=false` before and after expiry; the owner left its exact
action state unchanged, emitted **zero** stalls and performed **zero** effects
for it. One manual hand pass reported **one**
`stall:overdue-hand:0`, set `stall_reported=true` and
`status=dispatch_uncertain`, preserving claim
`73df6b3c-1c16-450f-a40c-abcbcf32f7e2`.
No third worker launched; operation count remained 22 and holds remained zero.
[Compact computed coverage and matching effects](owner-cutover-85-coverage.json)
is extracted from that raw transcript. Final disarm refusal was exactly
`owner_off`; the private tmux servers and dispatcher children were cleaned up.

[Exact promotion recipe replay](owner-cutover-85-promotion.jsonl): **PASS**,
exit 0, disposable root `/private/tmp/op80-vv908vnp`.
Generation **1 fixture-only → 2 OFF → 3 tmux-workers**; a real launchd interval
completed synthetic `promotion-work`, `hold=null`, `firing=interval`.
History remained, disarm/uninstall completed, final service state **absent**.
The exact observed status lines were:

```text
BEFORE: Owner watchdog on admitted ticks: covered=1, excluded=0; new default actions=covered; hand stall detection=covered by this watchdog.
AFTER: Owner watchdog on admitted ticks: covered=1, excluded=0; new default actions=excluded; hand stall detection=manager responsibility (--hand-run is manual; no scheduled hand watchdog).
```

This replay had only owner-assigned current work, hence excluded=0; it still
correctly discloses exclusion of newly created default actions. The concurrent
rehearsal separately proves the nonempty excluded set and overdue behavior.

## Candidate and changed lines

[Candidate manifest](owner-cutover-85-candidate.json) pins 13 source, test,
requirements and recipe inputs. Package digest:
`b930703e643c59c103e57fa8d43e760f7f060387b47be3aeeba90e857daa5946`.

| Existing file | Added | Deleted |
| --- | ---: | ---: |
| scripts/initiative_control/coordinator.py | 7 | 2 |
| scripts/initiative_control/owner_daemon.py | 88 | 24 |
| scripts/initiative_control/test_owner_daemon.py | 117 | 6 |
| qa/.../owner-handoff-80-promotion.md | 51 | 4 |
| qa/.../owner-handoff-80.md | 7 | 2 |

Production **95 additions / 26 deletions**; tests **117 / 6**.
Existing tracked total **270 / 38** (**308 changed lines**).
New executable rehearsal: **294 lines**. New return/manifest/coverage/log
artifacts are additional QA evidence. No edits to activate.py, owner_tmux.py,
test_owner_tmux.py or test_coordinator.py.

`git diff --check`, source AST parsing and all **5** promotion shell blocks
(`bash -n`) pass. Sprint checker: **115 current / 127 archived**, passes.
All changed paths are inside the frozen writable scope.

[Final full discovery gate](owner-cutover-85-suite.txt):
**822 tests passed in 432.653s**, **0 failures, 0 errors, 0 skips**, exit **0**.
Exact invocation in the isolated environment above:

```sh
/private/tmp/owner-cutover-85.WdNw7u/venv/bin/python -B -m unittest discover -s scripts/initiative_control -p 'test_*.py'
```

Final affected invocation was `-m unittest -v test_owner_daemon test_owner_tmux
test_decision_feed test_attention`. Both rehearsals used that same interpreter
and isolated environment. Final verification found all **13 manifest files**
and the package digest unchanged. Scope validation found **15 changed/untracked
entries**, all allowed; HEAD remains the frozen base. No commit or push.

Final nonzero accounting: **2 unexpected nonzero test commands**, containing
**1 assertion failure and 2 errors** across the preliminary attempts above;
**0 failed rehearsals**. The promotion script intentionally observed one
subcommand exit **2 / owner_off** before arming; its outer run passed. Those
expected refusal semantics are not a failed rehearsal. Separately, two
read-only exploratory shell commands exited nonzero (a wrong sprint-file glob,
and misplaced rg options); both were corrected and neither changed files.
No native prompt or contaminated test run occurred.

## Findings about the brief

The P2 is confirmed. Its wording “every newly created action” means the default
ownership at creation: explicit subsequent handoff can transfer selected work
to the owner. The other three findings were omitted from the frozen brief's
text but recoverable from the named prior review; all are addressed above.
The claimed live-state context was not verified because live inspection is
prohibited. No contradiction in that context is asserted.
