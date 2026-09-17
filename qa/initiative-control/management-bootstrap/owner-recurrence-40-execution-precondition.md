# owner-recurrence-40 — execution precondition unresolved

RETURN: implementation has not started. This is not a qualified candidate.

Allocation `owner-recurrence-40`; claim `117d411c-cb46-4331-81fc-60ad3dcb5c5f`.
Allocation digest: `24bc3b43bb6b782d01abb548d7638619227155544b960fe4aed5b2b8439be32c`.
Brief SHA-256 verified with `shasum -a 256`:
`4c7042c4f59babc169e8affa84adcea62b621b09ef8afdddf6efc6bb00e221fa`.
Worker: `gpt-6-astra`, high effort.
Actual worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/owner-recurrence-20260917`.
Actual branch: `bootstrap/owner-recurrence-20260917`.
Actual HEAD equals assigned base: `02c3ede6a3e8a1480772ee91409ac176547c664d`.

## Required manager reconciliation

Classification: product initiative, PF-80-S01 (`in_progress`), active plan
`docs/plans/active/initiative-delivery-control.md`. Product heading:
**Internal delivery control — TO BUILD**; excerpts: “durable event dispatch,
acknowledgments and watchdog” and “Show blockers, rendered sprints, human test
plans, machines, run logs and freshness”.

The current sprint records worktree `management-workstreams-20260911`, branch
`integrate/management-workstreams-20260911`, and base
`eb01bf006eacbeef5be07174a3f37ad124abc74c`. The active plan does not contain this
worker's worktree, branch or base. The same missing allocation was disclosed in
`owner-recurrence-35-design-stop.md`; expanding the source-file scope did not
repair these records. Root AGENTS.md requires executable sprint coordinates to
agree with the active plan before implementation; `docs/sprints/index.md`
requires worker coordinates in the active plan before parallel allocation.
Plan/sprint changes are manager-owned and outside this worker's writable scope.
The next action belongs to the Fable manager: reconcile the exact allocation and
scope in those records, validate them, then redispatch. No human product decision
or expansion of product scope is requested.

## Design and unexecuted evidence

The accepted design remains 30-second recurrence: three seconds conflicts with
the retained throttle; 300 seconds delays ACK, RETURN and watchdog observation.
Overruns do not overlap or catch up; 90 seconds without completion is stalled;
ExitTimeOut is shutdown grace. Global refusal latches HOLD until explicit
recovery, with later status-only probes; per-action holds retain isolation.
Running requires fresh service and tick evidence; stalled exposes age and
hold/error; never-installed requires verified absence and no installation
receipt; unavailable observations remain unknown with a reason.

The prior design's explicit pinned interpreter, four resolved placeholders,
private-path checks, idempotent installation and receipt-preserving uninstall
remain proposed, not implemented. No disposable label or store was created;
no real scheduling, double-install or reversal qualification was performed.
This worker did not enable the live job or inspect its current state, read
credentials, post Slack, modify coordinator state, or push.

Read `docs/development/test-isolation.md`. No test campaign or venv was created;
tests executed: **0**. Remaining test failure names: none observed (unrun).
`python3 docs/sprints/check.py` passed: current 115; archived 127. That checker
validates recorded relationships, not correspondence with this worker checkout.
Only this evidence record changed; implementation/test changed lines: **0/0**.
The new dashboard allocation resolves the previous source-scope blocker and
supersedes the brief's retained decision_manager prohibition. It does not resolve
the execution-record mismatch. Independent functional acceptance, tests and
human-test readiness remain unclaimed.
