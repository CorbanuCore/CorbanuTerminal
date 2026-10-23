You are the broker lane worker (round 2). Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
first (binding), then the previous worker's report:
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-broker.result.md (PF-27-S04 merged as PR #180, flag
`isolated_credential_broker`, evidence in qa/security-levels/sprints/PF-27-S04/isolated-broker-20261006/).

Coordinator decision (2026-10-06, following the worker's recommendation; Travis may revisit):
- Moving Core's own model-client auth into the broker becomes its own sprint, placed after PF-27-S02 in the broker
  lane. Create that sprint record (next free PF-27-Sxx id) with scope and dependencies.
- The PF-27-S04 revocation trigger moves to PF-23-S03 (record it there as an explicit item).
- With those two moved, close PF-27-S04 per the archive rules if its per-sprint gate is otherwise met; say plainly
  what is carried forward.

Then implement PF-27-S02 "Secretless agent launch and bypass containment"
(docs/sprints/current/p0-security-levels/pf-27-s02-secretless-agent-launch.md), including the open items the S04 worker
handed over: broker OS sandbox / separate containment, the macOS setup-pipe inheritance window, the broker socket in
an agent-writable directory, and the design conflict (stripping secret-looking env vars in Moderate/Aggressive must
exempt or run after the broker's dummy substitution). Three OSes: if Windows can't be done in this sprint, ship
macOS+Linux behind the flag and record Windows as an explicit follow-up rather than blocking.

Per-sprint gate per sec-common.md (focused tests, GLM 5.2 tmux run, one independent Opus 5.5 High review, demo
videos via scripts/demo_video.py). PR to main; merge when green. Update sprint files and index rows.
Report as in sec-common.md.
