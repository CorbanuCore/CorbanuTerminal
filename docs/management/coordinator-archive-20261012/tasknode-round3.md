You are the Task Node integration worker for Corbanu Terminal (sprint PF-80-S01), round 3.

Read first: /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/tasknode-round2.result.md, the record
qa/initiative-control/management-bootstrap/tasknode-writeback-live-20261002.md and the map tasknode-task-map.md in
/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911, and the tasknode-usage skill
(/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home/skills/.system/tasknode-usage/).

## New state
Travis, 2026-10-03: "I have accepted the tasks." The five tasks created in round 2 should now be accepted:
task_9d4fc2d8…, task_512bafc2…, task_cf2cc91f…, task_80628c7a…, task_6d472c45…. Confirm each task's state read-only
first.

## Do
1. Point the PF-80-S01 mapping in the delivery-control `control.json` at the right accepted task(s), through
   whatever supported command or reviewed edit the records describe, instead of the Rewarded task_789a0f3…. Record
   the change.
2. For each accepted task, submit evidence of completion only for work that is genuinely done and verifiable:
   commits, QA records and receipts, per the task map. Each submission uses the vault-backed sender, a fixed
   idempotency key and an honest scope.
   - task_80628c7a is the live progress event itself: send it, then re-send with the same key and prove there is
     no duplicate.
   - If any task's work isn't complete, don't submit it. Say what remains.
   - task_512bafc2 (Slack alert path): its reply half is not proven; say so in the evidence, or don't submit.
3. Read back every submission's receipt and status. Never move stages or claim rewards; Task Node does that.
4. Update the task map, the record and the PF-80-S01 ledger, then commit and push fast-forward. Never stage the
   four uncommitted files in scripts/initiative_control/ that belong to another session.

## Never
Never print credentials. Never touch wallet keys. Never turn on batch posting or TUI recording. If something
needs Travis, stop and say exactly what.

## Report (your final message)
A table of task, state, evidence submitted (yes or no, and why), receipt and duplicate re-send result. Then the
commits, and anything still open.
