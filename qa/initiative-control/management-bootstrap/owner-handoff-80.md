# RETURN — owner-handoff-80

Allocation: `owner-handoff-80`.
Digest: `a75e797b1bc0a4c10e0d9aa4b240b4a5fdfb0366509bbbb8900238661f878292`.
Claim: `55650af0-2b5b-4912-8ec8-23e619266f20`.
Worker: `gpt-6-astra / high`.
Frozen brief was the first read; verified SHA-256
`de1e38e6184efa1ea6b7355e5106c44943d3ac62b2bec58b3eba29fb9b8d9b14`.
Initial HEAD/base: `469f3209468c2d7e8bb60f661f0976d9b4501aee`.
Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/owner-recurrence-20260917`.

The P3 corrections restore the authorized preview/rehearsal behavior. The
ownership and OFF-only promotion mechanisms are a product-initiative increment
within active `initiative-delivery-control`, `PF-80-S01 / in_progress`, covered
by this exact manager allocation. Product heading:
**Internal delivery control — TO BUILD**, “durable event dispatch,
acknowledgments and watchdog”; “initialize and rehearse all three workstreams
before enabling recurring operation”.
The plan records this worktree/branch at historical base
`08db99fff46fd22c582fbea0240fa379a42242e7`; the frozen dispatch supplies the
newer exact base above. Shared plan/sprint changes remain manager-owned.

## Dry run: no owner locks, bounded SQLite backups

`--arm --dry-run` takes neither owner flock. It makes a read-only owner SQLite
snapshot into RAM, closes the source, then validates/simulates against RAM.
The coordinator backup also closes before simulation. Each backup proceeds in
32-page batches, waits zero on BUSY/LOCKED, and refuses once its monotonic
250ms budget is reached. SQLite still uses brief read locks; this is **not**
a claim of database-lock-free I/O or a hard real-time wall-clock bound on a
stalled filesystem/OS. The budget is checked after each SQLite backup step.
No exclusive owner lock spans a database copy or preview.

The returned `read_policy` discloses these limits. Snapshots are advisory and
independent; the real arm revalidates. This avoids making a long read rehearsal
hold off the actual owner. A regression attempts to acquire both owner locks
*during both backups*, verifies success and checks all persistent files unchanged.
Busy/budget refusals are separately exercised. Existing fixture-only preview
scope and unsettled-operation refusal remain; a transport preview is not claimed.

## Post-disarm refusal is pinned

Scheduled refusal output and `tick.json` now retain the exact kernel refusal
alongside the existing `owner_run_refused` latch. The rehearsal waits for
`refusal == "owner_off"` specifically. Its verifier asserts the same field;
old transcripts without it are not retroactively accepted.

The real disposable launchd rehearsal preserves activation/watchdog history,
has two admitted interval boots, then proves the next interval is refused
specifically for `owner_off`. The named test service is uninstalled afterward.
The first transcript remains separate from the final-tree replay.

## New mechanism

- Explicit `--handoff REQUEST --config CONFIG` commits a shared ownership
  partition. The request freezes coordinator revision and each selected
  action's prior owner, destination owner, claim ID, allocation digest and
  status. Initial cutover must name every pending action. The audit/evidence
  record and returned revision/time identify the exact commit instant.
- Both dispatchers and handoff use the same existing owner delivery/admission
  locks. A successful transfer cannot overlap a cooperating key delivery.
  Lock contention refuses handoff without mutation; recurrent dispatch reports
  BUSY/skipped without pretending a tick was admitted or latching a hold.
- Future actions default to **hand**. The daemon's launcher and watchdog skip
  hand actions. Action-local hand holds do not hold the owner lane; unknown
  and global holds remain conservative. Status exposes every action's owner,
  claim, allocation and status plus the cutover revision/time/evidence.
- Claim, dispatched, ACK, return and reconciliation mutations validate the
  dispatcher against the partition. Existing manual callers default to hand.
  A stale/manual caller cannot mutate an owner claim through these methods.
- `--hand-run` uses the *same* kernel and journal for hand-owned TMUX work.
  Active journal-backed claims can move owner→hand→owner with the same claim ID,
  process and completed deliveries. Missing effect receipts still prohibit retry.
  A legacy hand claim with no daemon journal stays external hand work: the owner
  neither launches it nor creates `unowned_claim`. Transfer into the owner is
  refused as `handoff_requires_receipted_claim`; finish/reconcile it through the
  existing protocol and transfer new prepared work instead.
- `--reconfigure REPLACEMENT` is explicit OFF-only config/package repinning,
  preserving generations and history. Pending/held operations and active owner
  claims block it. An activation intent fences an interrupted update; explicit
  disarm remains the recovery path.
- Installer `--repin` requires an uninstalled, absent job and OFF matching
  owner/config pins; label/domain/cadence/publication remain fixed. It preserves
  tick/recovery history rather than deleting a receipt to bypass pin checks.

## Existing mechanisms configured/reused

No new worker wire protocol: existing claim, exact ACK, START, rollout
correlation, RETURN, leases, uncertain-effect journal and resource reservations
remain. The transport's pinned binary, tmux, private runs directory, auth-link
path and frozen allocation runtime are existing configuration.

Both parties use **one canonical coordinator**, one owner journal and one
configured transport. This eliminates duplicate coordination *within the agreed
deployment*. It is not a global registry preventing an unrelated second
database or a same-account process from deliberately sending raw tmux keys.
Initial cutover requires legacy/raw key senders to stop, and cooperating hand
work then uses the mediated path. Separate coordinators with overlapping
allocations are not a supported configuration.

This increment supervises explicitly assigned, prepared worker actions.
It does not add automatic manager inference, invent allocations, default every
future proposal to owner control, accept worker results, or complete the sprint.

## Rehearsals and exact promotion

[Concurrent runner](owner_handoff_80_rehearsal.py) starts independent owner and
hand dispatcher processes against the same disposable coordinator, armed at
`tmux-workers`. It uses real private tmux servers and real prompt/paste/Enter
delivery into harmless synthetic worker processes. Both claims are active
together; an active owner claim transfers out and back while both dispatchers
remain alive, before ACK is released. Both claims return. Each has 11 applied
journal effects, exactly one prompt and one START, unchanged claim identity,
a synthetic performed-work artifact and zero holds. Raw pane captures and
dispatcher attempts, including BUSY, are preserved.

These are real process/PTY/coordination proofs with **synthetic rollout and
inference records**, not live-model or independent functional qualification.
No authentication contents are read.

[Exact live promotion commands and first-five-minute checks](owner-handoff-80-promotion.md)
include the transport input, selected prepared actions and actual decision
authority. They preserve the live databases/receipts and move fixture-only→OFF→
repinned transport→partitioned→installed OFF→armed tmux-workers→explicit recovery.
The live commands were **not executed**. The companion
[promotion rehearsal](owner_handoff_80_promotion_rehearsal.py) runs the exact
Markdown recipe on a disposable installation, not the named live root.

## Qualification boundary

No live owner root/coordinator, real profile, auth/token contents, Slack or
inference service accessed. No native credential prompt, push, commit, release,
workspace-wide formatter or out-of-scope edit. All tests run under `env -i`
with disposable HOME/profile aliases and a newly built disposable venv using
only `scripts/initiative_control/requirements.txt`.

Independent code-blind design/execution/evidence review, manager receipt,
human acceptance, default-live-repository qualification, benchmarks and release
remain manager-owned gates. This worker return is not an unqualified human-test
handoff, full sprint completion or release claim.

The first focused command used the misspelled denial flag
`CORBANU_TEST_DISABLE_NATIVE_KEYRING`; retained as an invalid guard setting,
not represented as guarded native qualification. It exercised Python synthetic
fixtures only, with all profile aliases disposable. Every successor command
uses `CORBANU_TEST_NO_NATIVE_KEYRING=1`. No prompt occurred.

## Findings about the brief

Both P3s were real: the original preview held both owner locks through backup,
and the original post-disarm rehearsal asserted only the generic refusal latch.
The original owner did also treat journal-less hand claims as unowned holds.

The brief's live-state history was provided context, not independently verified
here because it explicitly prohibits live inspection. No contradiction is
asserted. “Neither disturbs the other” means disjoint claim/effect ownership
with serial, nonblocking admission to the shared journal; it does not mean
simultaneous unsynchronized tmux writes or zero possible BUSY observations.

## Validation results

Candidate package digest:
`4efded21862ee45b0baaef09f7615c45859c5b56f6e36142ed09f4557521e957`.

Retained intermediate results:

- [First focused run](owner-handoff-80-first.txt): 185 tests in 14.717s,
  **3 failures, 1 error**, exit 1. Error:
  `ArmingTests.test_transport_arm_does_not_launch_or_read_auth` needed the new
  required handoff. Failures:
  `ActivationVisibilityTests.test_busy_owner_still_reports_coordinator_and_unknown_owner`
  (restored the existing status-lock semantics);
  `WorkerLifecycleTests.test_claim_commit_gap_holds_without_adopting_or_launching`
  and
  `WorkerLifecycleTests.test_receipt_after_domain_commit_gap_holds_without_duplicate_mutation`
  (updated crash injection signatures to forward dispatcher keywords).
- [Second focused run](owner-handoff-80-second.txt): 193 passed in 14.272s,
  zero failures/errors; exit 0.
- [Third focused run](owner-handoff-80-third.txt): 195 passed in 14.495s,
  zero failures/errors; exit 0.
- [Initial full gate](owner-handoff-80-suite.txt): **818 tests in 446.857s;
  0 failures, 7 errors**, exit 1. All seven were fixture setup rejection of
  macOS's symlinked `/tmp` after the isolated environment omitted TMPDIR:
  `test_decision_manager.ListenerStartupTests`:
  `test_cli_keeps_fixed_redaction_and_records_import_failure`,
  `test_one_shot_records_real_child_death_before_and_after_handshake`,
  `test_rejected_start_keeps_running_child_supervised`,
  `test_restart_child_death_before_handshake_is_not_lost_after_reap`,
  `test_spawn_failure_does_not_invent_child_exit`,
  `test_supervised_post_handshake_startup_death_is_observed`,
  `test_supervised_startup_death_retries_failed_record_without_duplicate`.
  These are **not** the load-sensitive ManagerTests failures anticipated by the
  brief; the intermediate progress message guessing that attribution was wrong.
- [Canonical-TMPDIR replay](owner-handoff-80-tempdir-replay.txt):
  all 7 passed in 0.759s with `TMPDIR=/private/tmp`, zero failures/errors; exit 0.
  No source change was needed. The final full gate uses that environment too.
- [First launchd arming replay](owner-handoff-80-arming-first.jsonl): PASS;
  [delta verification](owner-handoff-80-arming-verification.txt): 3 comparisons
  passed. Intermediate package recorded in the transcript.
- [Final-tree launchd replay](owner-handoff-80-arming-final.jsonl): PASS at
  `/private/tmp/owner-marks-78.handoff80/rehearsal-1789655384116543000`;
  [final verifier](owner-handoff-80-arming-final-verification.txt): all 3
  comparisons passed, and the explicit owner_off assertion passed.
- [First concurrent rehearsal](owner-handoff-80-concurrent-first.jsonl):
  PASS at `/private/tmp/oh80-jzvu_7by`; retained before the last scoped-HOLD
  hardening. The final-tree replay is separate.
- Promotion attempt [one](owner-handoff-80-promotion-first.jsonl): exit 1,
  installer subprocess exit 1 from `ModuleNotFoundError: markdown_it` under
  `-S`. Attempt [two](owner-handoff-80-promotion-second.jsonl): exit 1,
  installer subprocess exit 1 from the missing adjacent plist template.
  Both disposable services were cleaned up. These were **recipe defects**,
  corrected in the Markdown commands, not production fixes.
- [Promotion final replay](owner-handoff-80-promotion-final.jsonl): PASS at
  `/private/tmp/op80-q3h6e76n`. Exact Python command recipe succeeds, transitions
  generation 1 fixture-only → generation 2 OFF → generation 3 tmux-workers,
  then the actual launchd interval launches and completes synthetic worker
  `promotion-work`. Tick hold is null, firing is interval, action is returned,
  and old history remains. Service is disarmed/uninstalled and private tmux
  cleaned up. Exit 0. The outer shell now explicitly creates the same kind of
  pinned-requirements administrative venv used by this rehearsal.

[Final-tree concurrent replay](owner-handoff-80-concurrent-final.jsonl): PASS,
exit 0, `/private/tmp/oh80-1ry_aakk`. Cutover revisions 10, 13 and 14;
owner claim `4f2944d6-677b-4f7f-9d8f-f05d6b56ef96`,
hand claim `0faa6e9e-73ee-41df-9b1c-cd07bc38c587`.
22 applied effects, zero holds, two completed work artifacts.
15 dispatcher observations include **8 BUSY** passes, explicitly preserved rather
than counted as admitted successes. Both final ownership/claim identities match
the agreed partition. Post-disarm refusal is again exactly `owner_off`.

[Final owner/transport/feed/renderer regression](owner-handoff-80-regression.txt):
**232 tests passed**, zero failures/errors/skips, exit 0:
owner daemon 124, owner tmux 48, decision feed 38, attention/renderer 22.

Final regression duration: **113.331s**. Full-gate replay result follows below.

## Candidate and changed lines

[Candidate manifest](owner-handoff-80-candidate.json) pins all runtime inputs,
changed production/test files, the two new runners, the updated arming
runner/verifier and the promotion runbook.

| Existing file | Added | Deleted |
| --- | ---: | ---: |
| scripts/initiative_control/activate.py | 12 | 1 |
| scripts/initiative_control/coordinator.py | 55 | 8 |
| scripts/initiative_control/owner_daemon.py | 170 | 31 |
| scripts/initiative_control/test_owner_daemon.py | 215 | 5 |
| qa/.../owner_marks_78_rehearsal.py | 1 | 1 |
| qa/.../owner_marks_78_verify_rehearsal.py | 5 | 1 |

Production: **237 additions / 40 deletions**. Tests: **215 / 5**.
All existing tracked files: **458 / 47** (505 changed lines).
New executable QA runners: **229 + 110 = 339 lines**.
New runbook, return record, manifest and raw transcripts are additional QA
artifacts. `owner_tmux.py`, `test_owner_tmux.py` and `test_coordinator.py`
are unchanged. All changes are inside the frozen writable scope.

`git diff --check` passes. Promotion shell syntax: five blocks pass
`bash -n`. Source/harness Python AST parsing passes. Sprint checker:
**115 current / 127 archived**, passes. HEAD remains the frozen base; no commit
or push was made.

## Reproduction environment

Venv: `/private/tmp/owner-handoff-80-venv`, freshly built under `env -i`;
installed only `markdown-it-py==3.0.0`, `mdurl==0.1.2`,
`slack-sdk==3.44.1` from the repository requirements. From the repository root:

```sh
env -i HOME=/private/tmp/owner-handoff-80-home TMPDIR=/private/tmp \
  CODEX_HOME=/private/tmp/owner-handoff-80-home \
  CORBANU_HOME=/private/tmp/owner-handoff-80-home \
  PFTERMINAL_HOME=/private/tmp/owner-handoff-80-home \
  CORBANU_TEST_NO_NATIVE_KEYRING=1 \
  PATH=/opt/homebrew/bin:/usr/bin:/bin \
  PYTHONPATH=scripts/initiative_control PYTHONDONTWRITEBYTECODE=1 \
  /private/tmp/owner-handoff-80-venv/bin/python -B -m unittest discover \
  -s scripts/initiative_control -p 'test_*.py'
```

Same environment for focused tests:
`-m unittest test_owner_daemon test_owner_tmux test_decision_feed test_attention`.
Same environment for the two checked-in rehearsal runners. Arming runner uses
`/private/tmp/owner-marks-78.handoff80` as its explicitly disposable base.
Raw attempts are separate files; no earlier result was overwritten.

## Final full gate

[Final full-suite log](owner-handoff-80-suite-final.txt):
**818 passed in 459.787s; 0 failures, 0 errors, 0 skips; exit 0**.
This includes the load-sensitive `test_decision_manager.ManagerTests`,
`test_fable_launcher.RealTmux` and all 48 `test_owner_tmux.TmuxTests`:
**zero failures/errors in those lanes**, in both full-suite attempts and in the
applicable final focused regression.

There were exactly **four nonzero test/rehearsal command exits** during this
assignment: first focused run (3 failures/1 error), initial full gate (7 fixture
setup errors), first promotion recipe (dependency missing), second promotion
recipe (template missing). Every raw attempt remains linked above. Later
successes are separate replays, not overwritten or relabeled earlier attempts.

Final source/package and all 12 manifest entries verified unchanged.
No required implementation work from this allocation is left pending.
Manager review/receipt and independent functional qualification remain separate;
nothing live was armed or migrated.
