# RETURN — slack-listener-34

Allocation digest: ef8ae8c7bce62c1f98cfda83bf2e410c7b63062cd1fb9fa55a0739621c3a0726.
Claim: 1539e3fc-9e02-400c-a010-15c64f4c8e4e.
Worker: gpt-6-astra, high. Base: 9c49184b5d06f37a2103c45e9cb95f47af32af12.
Frozen brief SHA-256 verified with `shasum -a 256`:
10c8db685f308e8469a2ee2b76e984da978a55710cfb017ffbcc7d703a3a806e.

Bounded reliability fix under **Internal delivery control — TO BUILD**:
“durable event dispatch, acknowledgments and watchdog”; “run logs and freshness”.
Existing initiative-delivery-control / PF-80-S01 context. Only allocated code and
this evidence directory are changed. This is an internal repair return to the
manager, not live-route qualification, human-test readiness, release or approval.
Independent functional execution/evidence acceptance remains with the integrator;
this return does not waive that gate. No product plan/sprint records were edited.

## Captured failure and scope of the diagnosis

Live read-only observation: ingress 79, fence size 82, watermark 5, epoch 28,
listener_exits 0. No live listener was started. Only named journal files and
lock/fence files were copied into private disposable directories. Copied runtime
and owner inode pins were relocated inside each copy; the binding was not changed.

Running the original base's real child with this shell's
`/opt/homebrew/bin/python3` returned the runtime-owned frame and then exited 1.
The actual caught exception was `ModuleNotFoundError`, propagated from
`slack_transport.py:719`, the `slack_sdk.socket_mode` import, into
`decision_manager.py:507`. Thus runtime ownership succeeded but SDK import did
not. This is a reproducible missing dependency in the launching interpreter;
it is not a fence-gap refusal. Use an interpreter containing the pinned
requirements for both the manager and its inherited child interpreter.

The frozen brief did not provide the original invocation's interpreter or an
operator credential-file path. This worker's environment has no Slack credential
injection. The newly built pinned venv gets past the SDK import and then exits 1
with `KeyError` at the environment credential callback, before network activity.
These are captured failures of the stated environments, not proof of which
interpreter/environment the manager used for the two historical attempts.
Successful operational startup remains unverified; no credentials were sought,
read or printed.

The diagnostic harness uses a child-local trace hook and emits only exception
class, source basename, function name and line number for the exception that
propagates into `listener_child`, plus a boolean comparison of the missing module
name against the fixed string `slack_sdk`. It never emits exception messages, arguments,
locals, source text, payloads or credential values. Production child stderr
remains DEVNULL and production CLI error text remains fixed. The final harness
also denies `socket.connect`.

An earlier synthetic-credential diagnostic reached `apps.connections.open`
and exited with `SlackApiError`; its shutdown changed **that disposable copy's**
fence. It did not post a Slack message or modify the live store. This attempt is
not a preservation pass and is not evidence about real credential validity.
The later base/fixed import-failure reproductions made no network call and
preserved all protected copy fields and the copied fence.

## Why the records were missing and what changed

Both explanations in the brief apply to different boundaries:

- Original one-shot `main(listen)` creates only `ManagedListener`; it never
  invokes increment D's `ListenerSupervisor`. The real base CLI reproduction
  exits through `Invalid` after the child import failure and a reopened copy
  still shows zero exits. A separate idle supervisor cannot observe an unrelated
  process it does not own. One-shot and supervised ownership remain different.
- Separately, a child that fails before its runtime-owned frame is reaped and
  cleared inside `ManagedListener.start()`; the old supervisor sets its options
  only after start returns. That loses even an explicitly supervised initial
  startup death. The same reaping boundary affects failed restart handshakes.

`ManagedListener` now retains the failed-start process handle across reaping.
The supervisor's shared `observe_exit` records the proved return code, preserves
pending observations on journal failure, publishes unhealthy status, and avoids
duplicate records for the same handle. It also captures reaped restart children.
A rejected start with no newly spawned child does not invent an exit or disable
supervision of the existing child.

One-shot listen uses that same durable observation path, with restart held.
It never runs supervisor retry or pointer-delivery ticks. The fixed copy of the
same import failure reopens with listener_exits 1 and a child-exit record with
returncode 1, ingress_count 79, fence_count 82, fence_gap 3, epoch 28, restart held.

The original base's fresh idle supervisor observation was healthy. Reading the
same observation six seconds later produced unknown / observation-stale. The
brief's simultaneous fresh-timestamp/stale-health claim was not reproduced.

## Verification

Read `docs/development/test-isolation.md` before any test. No Rust tests,
workspace formatter, Corbanu credential profile, native credential access, push, Slack message
post or operational `qualify` was used. The explicitly requested existing suite
itself calls `qualify` on synthetic fixtures; it would be inaccurate to claim
that function was never invoked anywhere in the test process.

Disposable venv:
`/private/tmp/slack-listener-34.tsqYCF/venv`, created using
`python3 -m venv`, then that venv's `python -m pip install -r
scripts/initiative_control/requirements.txt`. No inherited venv/PYTHONPATH
site-packages. Freeze: markdown-it-py 3.0.0, mdurl 0.1.2, slack_sdk 3.44.1
(three application packages; pip is venv bootstrap tooling).

Command: unset CODEX_HOME, CORBANU_HOME, PFTERMINAL_HOME and both Slack token
variables; set PYTHONDONTWRITEBYTECODE=1 and
PYTHONPATH=scripts/initiative_control; use the above venv's python with
`-B -m unittest discover -s scripts/initiative_control -p '*test*.py'`.

Seven new cases in `ListenerStartupTests` cover actual child death on both sides
of the handshake, exact CLI redaction, retrying a failed durable write without
duplicates, supervised post-handshake death, invalid-start supervision retention,
pre-handshake restart death, and no invented exit on Popen failure. The existing
production-clock test now permits extra heartbeat clock reads while still
requiring the two child clock samples to advance by one hour.
The final eight-case targeted replay passed in 1.502s. Running three new
regression methods against the unmodified base produced four assertion failures
(two one-shot handshake subcases, missing unhealthy startup-observation status,
and zero durable exits after the actual CLI failure), in 0.571s.

Final-tree full suite: **737 tests passed in 464.342s**, exit 0; no skips.
Remaining failure names: **none**.

Earlier full attempt: 734 tests in 514.135s, 13 failures and one error, retained
in `focused-suite.log`; this was an intermediate tree, not final evidence.
Final result is in `final-suite.log`.

Separate unmodified-base `test_owner_tmux` attribution: 48 tests in 64.553s,
42 passed and six failed, retained in `base-tmux.log`. Failure names:

- `test_exit_at_pane_query_uses_a_post_exit_process_snapshot`
- `test_exit_between_pane_query_and_snapshot_is_reobserved_by_close`
- `test_fast_exit_before_launch_identity_collection_is_closeable`
- `test_inherited_session_wrong_worktree_and_extra_turn_hold`
- `test_malformed_rollout_structure_is_retained_as_uncertain`
- `test_missing_completion_response_and_malformed_return_hold`

Final source diff: decision_manager.py **41 added / 16 removed**;
test_decision_manager.py **157 added / 2 removed**. No slack_transport.py
or test_slack_transport.py changes. AST parsing and `git diff --check` passed.
Only those two allocated files and the two new evidence files are present in
`git status --short`. No workspace formatter was run.

Final read-only integrity check compared all nine named live files against
their pre-reproduction SHA-256 values: **all unchanged**. This includes the
entire transport journal (therefore ingress, watermark, binding and gap reviews)
and the independent fence file. No operational qualification was performed.

Private raw evidence: `/private/tmp/slack-listener-34.tsqYCF/`, including
`repro-base.log`, `repro-fixed.log`, `repro-fixed-confirmed.log`,
`fixed-durable-event.log`, `health-base.log`, per-copy exception-location
JSON and full test logs. The unmodified base scripts were extracted with
`git archive` into that directory for attribution. No history checkout was
modified. The retained reproduction harness is adjacent to this report.
