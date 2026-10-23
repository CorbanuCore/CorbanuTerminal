# owner-marks-78 — omission marks and read-only arming rehearsal

Allocation digest: `624a9f2cb058f4789562e7bb2387a1cd46dfff10cabe83712d26d52e894f2601`.
Claim: `8423e87d-3fb3-4187-97dd-1356e558869e`.
Worker: `gpt-6-astra / high`. First read was the frozen brief; verified SHA-256:
`e8b74ee8737ddca19aa9a4357cd97e352aeb3a8b3a7dc7e72052ffe81a0d0650`.
Base/initial HEAD: `0760908894c9b4f730050103ae1f8e84655b7947`.
Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/owner-recurrence-20260917`.

## Scope and qualification boundary

The omission repair is a bounded fix. The arming preview is a scoped increment
within active initiative-delivery-control, PF-80-S01 (in_progress), under this
frozen management-bootstrap allocation. Product heading:
**Internal delivery control — TO BUILD**, “durable event dispatch,
acknowledgments and watchdog”; “initialize and rehearse all three workstreams
before enabling recurring operation”; “actual Slack reply/decision/agent acknowledgment”.
The plan lists this worktree/branch with historical base
`08db99fff46fd22c582fbea0240fa379a42242e7`; this dispatch supplies the exact newer base above.
Shared plan/sprint records and independent acceptance remain manager-owned.
Sprint checker passes: 115 current, 127 archived.

This is an implementation return plus operator rehearsal, not independent
code-blind functional acceptance, a qualified human-test handoff, release,
benchmark or default-live-repository qualification. The existing manager owns
the independent design/executor/evidence gate for the renderer/CLI changes.
No live owner root, live coordinator, real profile, authentication material,
Slack or inference service was accessed. No native credential prompt, push,
release or workspace-wide formatter occurred.

## Reconciled omissions

A card counts absent Slack projection rows by their own revision keys. Each
missing row now has exactly one “Slack projection row for revision N: omitted”
mark on revision N, even when that revision is resolved and its question-status
line correctly refers back to a different answered revision. The count wording
explicitly distinguishes these two concepts. Retained answered rows and existing
unknown/omitted question observations are unchanged.

The regression uses five revisions: resolved revision 2 answers 1, resolved
revision 5 answers 3; rows 1/3/5 are retained, rows 2/4 omitted. Both Slack states
show two own-revision marks and a retained question-1 observation.
Against the frozen base, this one test fails in both enabled/disabled subcases:
[base omission transcript](owner-marks-78-base-omission.txt). Current tree passes.

## Arming preview

Use the candidate's arming entry point, not the schedule installer:

```text
python -B owner_daemon.py --activation-status --config CONFIG
python -B owner_daemon.py --arm --dry-run --config CONFIG --authority DECISION
python -B owner_daemon.py --arm --config CONFIG --authority DECISION
python -B owner_daemon.py --schedule SCHEDULE --recover "actual inspection evidence"
python -B owner_daemon.py --disarm --config CONFIG --generation CURRENT
```

The first two commands inspect; only the third arms. All arm input/config/package/
schema/permission/generation/recovery checks and reads run through the same arm
path before the dry-run branch. Both existing owner locks are held. The owner
database opens with SQLite `mode=ro` and a read transaction. No intent, activation
file, metadata update, service installation or kernel tick is performed.
Use `-B` (as the installed launchd command does) to suppress Python import caches.

Output `WOULD_ARM` includes the exact before/after `meta(singleton=1)`, activation
file path/create-or-replace/content, crash-intent content and transient paths.
It also reports the first admitted fixture tick's coordinator field/row deltas,
owner row keys, artifact paths and watchdog events. The real coordinator logic
executes only against an in-memory SQLite backup, preserving event deduplication,
revision, audit, sequence and terminal-action archival behavior. Read-only source
transaction acquisition has a 250ms busy timeout; a revision change between
inspection and backup refuses with `preview_coordinator_changed`.

Preview is deliberately scoped to fixture-only configurations with no
`operations.phase IN ('intent','observed')`. Unsupported cases return exit 2 with
`preview_requires_fixture_only` or `preview_requires_settled_operations`, rather
than an incomplete green preview. These limits do not change actual arming rules.
Other validation failures retain their existing reason codes. `--dry-run` without
`--arm` is a usage error. The projection is advisory: inspect while coordinator
work is quiescent, recheck if state/deadlines change, and allow actual UUID/PID/
timestamp allocation and I/O failure at execution. It does not inspect the
separately installed service, schedule HOLD or schedule pins.

Changing owner_daemon.py changes the package digest. The final digest is
`84c09899ca50fd1acfe8843fe770d24ce00d40a8180fc3f5c2866a1dc3108625`.
The manager must use matching reviewed runtime/config/owner-state/installation
pins; this worker neither reads nor migrates the live installation.

## Real disposable rehearsal and reversal

[Runner](owner_marks_78_rehearsal.py),
[raw commands, responses and deltas](owner-marks-78-rehearsal-first.jsonl),
[delta verifier](owner_marks_78_verify_rehearsal.py),
[verification result](owner-marks-78-rehearsal-verification.txt).

Disposable root:
`/private/tmp/owner-marks-78.iJAPn2/rehearsal-1789651904280711000`.
It mirrors the installer-defined layout: corbanu-owner-live contains runtime,
config, decision, installation receipt, plist, tick state, locks, logs, recovery
and tick artifacts; coordinator and publish-state are separate private configured
directories. It uses synthetic data, not a copy or observation of live files.
The actual uniquely named test service ran in the user launchd domain with
Background session type and the production 30-second interval. No launchctl
or kernel mocking was used. Total rehearsal elapsed approximately 122.5 seconds.

1. Dry-run exit 0; all 36 existing files retained identical bytes, sizes and
   modification timestamps; no extra file appeared.
2. Initial installed OFF tick refused and latched `owner_run_refused` (ticks=1).
3. Status and real fixture-only arm returned exit 0 / generation 1.
4. Next real interval stayed HOLD, skipped=1, ticks=1; coordinator/owner stores
   remained identical to their armed snapshot. Arming did not recover anything.
5. Explicit recovery returned RECOVERED. Next interval admitted a READY fixture
   tick (ticks=2): three overdue synthetic actions plus an overdue hand-manager
   produced four stall events. Manager ownership suppressed the fixture effect:
   zero operations/observations/deliveries/processes; one boot row and watchdog
   health. The dispatching action became dispatch_uncertain.
6. Second admitted interval (ticks=3) added the second boot, refreshed health and
   left coordinator bytes unchanged: no repeated stall events.
7. Disarm generation 1 returned OFF / generation 2. Only owner meta mode and
   generation changed; activation.json and coordinator history remained intact.
8. Next real interval refused before adding a boot (ticks=4, latched HOLD).
   Final activation status was OFF/generation 2; total admitted boots remained 2.
9. Uninstall succeeded and verified the unique test service absent. Receipt phase
   is uninstalled. Disposable evidence is retained; nothing remains scheduled.

Three transcript comparisons pass: exact arm meta, every changed coordinator
table on first admission (only wall times normalized), and exact disarm row delta.
Coordinator tables changed: state, events, audit, sqlite_sequence.
Unchanged: action_history and evidence for this particular fixture.
The raw disarm command/response and subsequent refused interval are in the same
rehearsal transcript; reversal was executed, not inferred.

## First five minutes: exact write map

Let **S** be the installed schedule root, **C** the configured coordinator
directory, and **P** the installation receipt's publish_state directory.
No live IDs, current deadlines or timer phase were read. The dry-run names their
actual row keys when the manager executes it against quiescent state.

- **At arm:** C/owner.sqlite3, meta(singleton=1): requested_mode=armed,
  control_generation=next, activation_decision_id, activation_revision and
  activation_digest set from the validated decision. C/activation.json is
  created/replaced. Transient files are C/activation-transaction.json.pending,
  C/activation-transaction.json, C/activation.json.pending and
  C/owner.sqlite3-journal; successful completion removes them.
- **First firing after arm:** arming fires no tick. The next already-installed
  30-second interval is first, subject to launchd scheduling/delay; missed
  intervals are not replayed. With the specified latched HOLD, every firing
  returns HOLD before pin checks/kernel admission. S/tick.json changes only
  skipped (+1) and last_probe; ticks, completion and success stay unchanged.
  P/owner-recurrence.json refreshes observed service/status. Launchd appends the
  JSON result to S/logs/owner.stdout.log; stderr changes only on an error.
  Atomic replacements transiently use tick.json.pending and
  owner-recurrence.json.pending. No C database row/artifact changes. Over five
  minutes this is roughly ten skipped probes if healthy, not a guaranteed count.
- **Recovery is required before scheduled kernel work**, but not before the
  probes/publication above. Recovery creates S/recovery/<UUID>.json (and
  transient .pending), retaining the previous tick plus supplied evidence;
  rewrites S/tick.json, clearing hold, started_at, completed_at, last_success,
  previous_success; preserves counters/refusal history; then CLI publication
  rewrites P/owner-recurrence.json. Recovery itself runs no kernel. A manual
  --run --config can bypass the schedule latch; it was not used in this rehearsal.
- **First admitted interval after recovery:** validates pinned runtime/config,
  updates S/tick.json started_at/completed_at/firing, ticks and success/error
  fields; creates S/ticks/<UUID>.json and transient .pending; republishes P status
  and appends daemon stdout. C/owner.sqlite3 inserts boots(boot_id=<UUID>,
  owner_epoch=current generation, host_boot_id, pid, process_start,
  package_digest, started_at), then fills stopped_at and stop_reason.
  health(component=watchdog) is inserted/replaced with attempt/success times,
  failures=0, next_probe_at=NULL, status=ok, digest of the watchdog result.
- **Watchdog despite paused dispatch or hand-manager ownership:** if any new
  overdue eligible record exists, C/coordinator.sqlite3 updates state(id=1).body:
  revision increments once, eligible actions/manager gain stall_reported=true,
  dispatching actions become dispatch_uncertain. It inserts events keyed
  manager-stall:<manager-id> and stall:<action-id>:<dispatch-epoch> with
  meaningful=1, consumed=NULL (exact duplicate events are not inserted again).
  audit adds operation=watchdog, at and body.observed_at; SQLite sequence rows
  for events/audit advance on insertion. The mutation can also move excess
  terminal actions into action_history(id=<action-id>) and remove them from
  state; the RAM preview includes this if applicable. evidence is unchanged.
  Owner HOLDs are checked after watchdog. Existing intent/observed replay would
  run earlier and is therefore explicitly outside the dry-run supported scope.
- **Fixture event when enabled, unowned and unheld:** one operation per activation
  generation, keyed by the reported op_id. C/runs/<op_id>/request.json,
  receipt.json and observations/1.json are created, each through .pending,
  with required private directories. Owner operations goes intent → observed →
  applied; observations(observation_id=op_id) and deliveries(delivery_id=op_id,
  phase=applied) are inserted. Coordinator events adds owner-fixture:<identity>,
  state revision increments, events sequence advances; event() adds no audit.
  With a hand-manager owning the coordinator, none of these fixture effects run.
  The admitted tick still writes boots/health. No worker or manager model launches.
- **Later admitted ticks:** new boot row and watchdog health each time, plus
  schedule/status/log artifacts. No duplicate fixture operation within the
  generation and no already-reported stall repeats. Newly overdue manual work
  may cause further watchdog changes. SQLite transactions use transient
  C/owner.sqlite3-journal and C/coordinator.sqlite3-journal when mutated.
- **Disarm:** C/owner.sqlite3 meta(singleton=1) mode=off and generation=current+1.
  Activation identity/digest and C/activation.json remain as evidence.
  Any activation intent/.pending files are removed; owner journal is transient.
  Schedule/plist stay installed. Next unheld scheduled tick refuses before boot
  admission, latches owner_run_refused, records a tick artifact and publishes;
  later firings become skipped probes. Disarm neither stops hand workers nor
  resets scheduler HOLD, watchdog flags, coordinator revision, events or audit.

**WATCHDOG/EVENT/AUDIT HISTORY IS NOT UNDONE BY DISARM. GENERATIONS ARE CONSUMED,
NOT ROLLED BACK. THE FIRST ADMITTED TICK CAN DURABLY RECLASSIFY OVERDUE MANUAL
CLAIMS AS DISPATCH_UNCERTAIN. THERE IS NO DISARM COMMAND THAT UNDOES THAT.**
This is the concrete limit of “reversal”: admission is revoked; history is kept.
The rehearsal proves that distinction. No financial or external worker effect
exists in this fixture-only path.

## Tests and tree evidence

Read docs/development/test-isolation.md before testing. Disposable venv built
under env -i at /private/tmp/owner-marks-78.iJAPn2/venv from requirements.txt:
markdown-it-py 3.0.0, mdurl 0.1.2, slack-sdk 3.44.1. Test commands use env -i,
disposable HOME and all three profile aliases, checkout PYTHONPATH,
PYTHONDONTWRITEBYTECODE=1 and TMPDIR=/private/tmp.

- [Initial focused attempt](owner-marks-78-first.txt): 20 tests, 3 failure records,
  zero errors; exit 1. test_cli_preview_is_write_free_and_matches_arm found
  SQLite connection-lifetime mismatch in the RAM simulation (fixed by rolling
  back when a real per-call connection would close). Both enabled/disabled
  subcases of test_omission_count_matches_own_revision_marks_with_answered_indirection
  initially used an invalid reopened-answer lineage; fixture corrected to the
  valid five-revision sequence above. Original transcript retained.
- [Corrected focused](owner-marks-78-focused.txt): 20 passed, zero failures/errors,
  exit 0. Followed by final source busy-timeout/revision-stability hardening.
- [Final owner/feed/renderer regression](owner-marks-78-regression.txt):
  **173 passed in 62.015s**, zero failures/errors/skips, exit 0:
  owner 113, decision feed 38, attention/renderer 22.
- [Frozen-base omission proof](owner-marks-78-base-omission.txt): 1 test,
  2 expected subcase failures, zero errors; exit 1. Both fail because revision-2's
  own omitted row is not marked by the old renderer.
- [Full requested discovery](owner-marks-78-suite.txt): **807 passed in 447.948s**,
  zero failures/errors/skips, exit 0. Final failure names: none. Printed
  TimeoutExpired/ERROR JSON and HTTPError ResourceWarnings are fixture output,
  not unittest failures.
  Command: python -m unittest discover -s scripts/initiative_control -p 'test_*.py'.
- Native launchd rehearsal: 1 complete lifecycle, 2 admitted interval ticks,
  4 stall events, 0 duplicate stalls, 0 fixture operations while manager-owned,
  successful disarm and uninstall. Three additional full-delta comparisons pass.

Production/test diff: owner_daemon.py +136/-6; attention.py +4/-1;
test_owner_daemon.py +109/-0; test_attention.py +20/-0.
Total **276 changed lines** (269 additions, 7 deletions), excluding QA artifacts.
activate.py, decision_feed.py and test_decision_feed.py are unchanged.
Only assigned paths changed. git diff --check and the final sprint checker pass.
All ten evidence links resolve; the copied rehearsal owner_daemon.py is byte-for-byte
identical to final source. No commit or push.

SHA-256:
- owner_daemon.py: 3ed76dfe45898a9b3d3b2b6868f8a9ff13205c191d9017b41e1b44d7c2e31e78
- attention.py: 52678a338a53d9c5d0cdbcd8617bde7f3f1215aea8dc851bf170ee62c20a52be
- test_owner_daemon.py: 6d1a66601a6f94e76747da2f9e5442211f1b2264153a3db8d289d28a9c18e467
- test_attention.py: 38e9eb749db6ef34d539b5c898fc6ee43d9cce3af672029431937d0ca6d05c2a

## Corrections and limits in the brief

The P3 is real. Arming does not itself schedule/fire a tick or lift HOLD.
“Reversal” cannot mean rollback of watchdog/audit history or consumed generations.
Fixture-only is not read-only, and manager_enabled does not start a manager model.
The exact next live firing and live action-row identities cannot be asserted
without the forbidden live inspection; the preview/rehearsal and conditional
write map above deliberately distinguish observation from prediction.
The supplied prior review/approval is treated as manager-provided context, not a
new review or authority generated by this worker. No other brief fact was disproven.
