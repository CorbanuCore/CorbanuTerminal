# owner-visibility-68 — shared coordinator visibility; Slack scope blocker

Allocation digest: `41e9c5b600c479e4b29512f74461d1e9d284c8d46f6500be0ae4c6526ee71ea4`.
Claim: `4e1181f7-30cf-475e-8cb5-0f2fce7778e1`.
Verified frozen brief SHA-256:
`c5e66d9af21d539ed01a3755d437da76a918a8fa51e9b670ee342e3365d41f46`.
Initial HEAD and assigned base: `c0e4fe8a9063b3ca631008df3d70ccddc8a22de6`.
Worker: gpt-6-astra / high.
Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/owner-recurrence-20260917`.

## Classification and authority

Bounded visibility correction within existing management-bootstrap PF-80-S01,
`in_progress`, under active `initiative-delivery-control`. Product heading:
**Internal delivery control — TO BUILD**, “durable event dispatch,
acknowledgments and watchdog”; “initialize and rehearse all three workstreams
before enabling recurring operation”. No admission rule, watchdog behavior,
credential access, persistent schema or external action changed.

The frozen assignment supplies this worktree and newer base; the plan's registered
worktree base remains `08db99fff46fd22c582fbea0240fa379a42242e7`. Shared plan/sprint
records remain manager-owned. Sprint checker: 115 current, 127 archived, exit 0.
This is an implementation return, not independent functional acceptance or release.

## Implemented visibility

`owner_daemon.py --activation-status --config CONFIG` now adds a `coordinator`
object containing observation time (Unix seconds), coordinator revision/enabled
state, in-flight actions, overdue actions, and any manager run. Every action
includes its exact ID, status, deadline (Unix seconds), overdue/stall flags,
whether watchdog will report it on the observed state, and the resulting status.

In-flight means claimed but nonterminal: dispatching, dispatched, running,
dispatch_uncertain and returned. Returned/uncertain actions remain visible but
are explicitly marked as not subject to another watchdog report. Prepared and
terminal actions are excluded. Overdue uses the watchdog's strict
`deadline < observed_at`; equality is not overdue. Previously reported stalls
remain visible but are not described as new watchdog mutations.

The warning states that armed fixture-only operation is not read-only against
the shared coordinator, including when dispatch is paused or a manager owns it.
It spells out stall reporting, dispatching → dispatch_uncertain, manager stalls
and fixture events. The result is a snapshot, not a reservation: manual changes
and deadline expiry can occur between inspection and the next admitted tick.

The arm command's help directs the operator to inspect status before arming.
The successful arm response also carries the impact observed before the activation
write. That response arrives after arming; it is not a new interactive confirmation.
For a pre-commit inspection the operator must run `--activation-status` first.
Foreign claims do not cause a new refusal. Existing database recovery restrictions
remain; status uses a read-only coordinator connection and never runs watchdog.

[Actual synthetic status output](owner-visibility-68-status.json) shows
`manual` with deadline 1060, observed at 1200, overdue=true,
watchdog_will_report=true and watchdog_status=dispatch_uncertain. This is disposable
test state, not a claim about the live coordinator. Another test arms with that
manual claim present, proves arming itself leaves the coordinator unchanged,
then proves a paused admitted fixture tick performs the disclosed watchdog mutation.

## Slack defect: reproduced, implementation blocked by writable scope

The explicit writable list permits only activate.py, owner_daemon.py,
test_owner_daemon.py and this evidence directory. It excludes
`scripts/initiative_control/decision_feed.py` and
`scripts/initiative_control/test_decision_feed.py`. The broad brief's requested
Slack implementation cannot override “only these paths”. No workaround was
inserted into activate.py or owner_daemon.py to modify the feed indirectly.

The two requested desired-behavior cases are retained in
[owner_visibility_68_feed_repro.py](owner_visibility_68_feed_repro.py):
20 decisions/53 revisions must project within the bound, and latest open plus
acknowledged questions must remain selected after 80 closed decisions. They
currently fail in the production size validator and are explicitly NOT passing
acceptance evidence. The reproduction file is outside the ordinary test discovery
directory and is run separately so its unresolved failures cannot be mistaken for
a passing repository suite.

A third diagnostic proves the current failure behavior: all 53 revision rows
reach validation; their canonical projection is 16,850 bytes in this synthetic
OFF-state example; validation raises before atomic_json; the existing cache
retains exactly its old bytes. A changed feed makes that old cache invalid via
its digest binding.

**Plain answer:** this inspected projection code fails closed on overflow. It does
not silently truncate or drop individual decisions. The diagnostic proves the
oversized path preserves the prior cache rather than replacing it with a partial
projection. No live historical audit was performed, so this is not a claim about
every previously deployed version or every past live publication.

### Proposed retention rule for the feed owner (not implemented)

Retain the latest revision of every decision, including every open or acknowledged
decision. Additionally retain older revisions whose alert still carries an unanswered
question, undrained ingress, an unacknowledged answer, or unresolved delivery
(pending/sending/uncertain/failed). A sent alert with no answer is still unanswered;
zero pending events is not evidence of an answer or explicit dismissal.
Deduplicate by decision ID and revision. Resolved historical rows without
outstanding work may be omitted; preserve the source feed and alert/reply journals.

Do not merely slice at 100 rows or 16,384 bytes: even latest-only cannot guarantee
all open questions at that fixed limit. The source feed has a 1 MiB byte bound but
no 100-decision bound. A proper fix must either page the projection with complete
mandatory coverage or derive a larger explicit projection bound from the admitted
feed and worst-case row size, and align writer, validator and reader limits.
A bound that silently evicts mandatory rows is unacceptable. If optional closed
history is trimmed, expose its omission count; continue failing closed on malformed
data. The new implementation must also test more than 100 open decisions and
older unanswered-alert retention, not just the observed 53-revision case.

This design is a follow-up recommendation, not a deployed bounded projection.
The manager needs to extend the allocation to the two feed files before that
implementation can be completed.

## Brief discrepancies and unavailable inputs

- “The rest of that review's findings” is not enumerated or linked in the frozen
  brief. The prior round-65 record is present, but the underlying review report
  was not supplied or located in the scoped evidence. No claim is made to have
  fixed undisclosed findings.
- The watchdog does not rewrite every overdue action: it only changes an
  unreported overdue dispatching action to dispatch_uncertain. Dispatched/running
  statuses remain unchanged; stall_reported and stall events change. It also
  reports overdue manager runs.
- “Every admitted tick ... before any readiness gate” overstates ordering:
  fixture ticks inspect readiness and replay before watchdog; exceptions can
  prevent reaching it. The material issue is still real: paused dispatch and
  manager ownership do not suppress the watchdog.
- About thirty seconds is the normal installed interval, not a guaranteed
  relabeling deadline: schedule HOLDs, locks, process failure or earlier errors
  can prevent an admitted tick.
- The stated live 20/53 count and healthy transport were not verified against
  live state. Synthetic 20/53 data reproduces the size failure. Row size is
  content-dependent; 339 bytes per row is not a universal constant.
- The brief asks for a Slack fix outside its explicit writable scope.

## Test campaign

Read `docs/development/test-isolation.md` before tests. A disposable venv was
built under `env -i` at `/private/tmp/owner-visibility-68.M5iai9/venv`, installing
only requirements.txt's markdown-it-py 3.0.0, mdurl 0.1.2 and slack-sdk 3.44.1.
Commands ran from the repository root with an empty inherited environment,
disposable HOME and all three profile aliases, PYTHONDONTWRITEBYTECODE=1,
TMPDIR=/private/tmp and checkout scripts/initiative_control on PYTHONPATH.

Commands:
- `python -m unittest -v test_owner_daemon.ActivationVisibilityTests`
- `python -m unittest -v test_owner_daemon test_decision_feed`
- `python -m unittest discover -s scripts/initiative_control -p 'test_*.py'`
- `python qa/initiative-control/management-bootstrap/owner_visibility_68_feed_repro.py`

Preserved attempts:
- owner-visibility-68-red.txt: 5 tests, 1 failure, 4 errors, exit 1. One error
  was an incomplete synthetic action (missing workstream); preserved as a
  fixture defect, not evidence against product behavior.
- owner-visibility-68-red-corrected.txt: same 5 tests, 1 failure and 4 errors,
  exit 1, now all attributable to absent visibility/recovery protection.
- owner-visibility-68-focused.txt: 131 tests, 2 failures, exit 1.
  `ActivationVisibilityTests.test_cli_status_discloses_manual_deadline_and_help_warns_before_arming`
  compared unnormalized wrapped help; corrected whitespace normalization.
  `ArmingTests.test_arm_tick_disarm_rearm_uses_real_entry_point` required the
  old exact response shape; it now verifies all prior fields and the added
  coordinator snapshot.
- owner-visibility-68-feed-red.txt: 3 errors in 3 tests, exit 1, fixture defects
  (missing lock and an acknowledged initial revision) preserved.
- owner-visibility-68-feed-red-corrected.txt: 3 tests, 1 pass, 2 errors, exit 1.
  Error names:
  `ProjectionRegressionTests.test_twenty_decisions_fifty_three_revisions_projects_within_bound`
  and `ProjectionRegressionTests.test_bound_never_drops_open_or_acknowledged_question_after_closed_history`.
  Both raise decisions.Invalid at decision_feed.py's size validator.
  The fail-closed diagnostic passes.

- owner-visibility-68-suite.txt: 785 tests in 448.230s, 4 failures, zero
  errors/skips, exit 1. It loaded tests before the two assertion corrections
  described above. Its other two failures were:
  `test_owner_tmux.TmuxTests.test_ack_start_return_resume_and_clean_shutdown`
  and `test_owner_tmux.TmuxTests.test_bracketed_paste_echo_is_not_ack_and_start_is_denied`,
  both fixture READY timeouts with blank panes. These are reported as observed
  failures, not relabeled as passes. ManagerTests and RealTmux reported no failure.
- owner-visibility-68-focused-final.txt: **131 tests passed**, zero
  failures/errors/skips, 60.460s, exit 0.

Final full-suite rerun, owner-visibility-68-suite-final.txt: **785 tests passed**
in 440.148s, zero failures/errors/skips, exit 0. ManagerTests, RealTmux and both
previously failing owner_tmux cases passed. This final run was started after the
assertion corrections; its tested source/test files match the final hashes below.
No native credential prompt was observed. The separate unresolved Slack regression
run remains 1 pass / 2 errors; the clean ordinary suite does not qualify that fix.

Source/test diff: owner_daemon.py +50/-4; test_owner_daemon.py +96/-1:
146 additions, 5 deletions, 151 changed lines. activate.py unchanged.
All added artifacts are inside the permitted management-bootstrap directory.
`git diff --check` passes. Source SHA-256:
- owner_daemon.py: `7a9f285966d40ca0e4d369dfe2b77a0f2cf135ba98539d0fc2a3de7bc25487d3`.
- test_owner_daemon.py: `ba51a57d230baf580550802a6d4b6994d95b5281165c201edaa641e95a399c74`.
Package digest: `b24f191ada2029197cb333d286a15e30484060d7ad60e225aed418c45c00e927`.

## Handoff limits

No live profile, auth file, token, live coordinator, Slack send, launchd activation,
push, release or broad formatter was used. activate.py is unchanged. Changes to
owner_daemon.py change the package digest; existing live pins cannot be silently
reused. Any integration/deployment must use the final package/config pins.

Independent code-blind functional acceptance was not run by this implementation
worker. Integrator acceptance of an internal-only N/A has not been recorded;
applicable independent design/execution/evidence gates remain with the manager
before an unqualified human-test or live recurrence handoff. No TUI, live-repository,
benchmark, human acceptance or release qualification is asserted.
