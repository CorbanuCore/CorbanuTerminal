---
sprint_id: "PF-78-S01"
title: "Campaign Tracker capture, persistence and permissioned replay"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-78"
execution_order: 80
owner: "Campaign Tracker integration"
parallel_lane: "campaign-tracker"
write_scope: "codex-rs/tasknode-session/, codex-rs/Cargo.lock, codex-rs/tui/src/chatwidget/, codex-rs/tui/src/chatwidget.rs, codex-rs/tui/src/app.rs, codex-rs/tui/src/app/event_dispatch.rs, codex-rs/tui/src/app/thread_goal_actions.rs, codex-rs/tui/src/app_event.rs, qa/campaign-tracker/"
integration_gate: "Sequential local integration; preserved existing dirty edits; changed-crate tests and actual PTY workflows passed."
worktree: "/home/pfrpc/repos/CorbanuTerminal"
branch: "fix/tasknode-profile-isolation"
base_commit: "ec549c0c687f50a682487e9d68289c05557ff579"
depends_on: "none"
created: 2026-09-06
updated: 2026-09-10
---

# PF-78-S01 — Campaign Tracker

Renumbered from P0 PF-45-S01; see the [identity reconciliation](../../identity-reconciliation-2026-09-10.md). Historical evidence and completion state are retained.

## Execution mandate

Implement the local Campaign Tracker candidate in the Task Node tab. Public deployment, employee enrollment and production sharing are separate actions.

## Plan linkage

[Active P0 integration plan](../../../plans/active/p0-security-levels.md), PF-78.
Product: **Campaign Tracker — LOCAL PILOT CANDIDATE**, “preserve attributable user prompts and compact GLM 5.3 Flash summaries in a bounded database”.

## Code boundaries

Corbanu tracker/session module, encrypted outbox, typed TUI hooks and Task Node views; Task Node tracker contracts/repositories/routes/migration/fixtures in the adjacent tasknodeofficial checkout. Large artifacts remain on the data volume.

## Preconditions

User authority, active-plan linkage, checkout coordinates and mounted storage were verified before implementation. Existing dirty changes and global checker conflicts were recorded.

## Done

- [x] User implementation authority, worktree/base, specification and prior scoring recorded.
- [x] Implemented authenticated encrypted activity records, enrollment, handle grants, authorization and replay.
- [x] Implemented bounded subscription-backed Flash summaries and conservative structured task suggestions.
- [x] Implemented encrypted atomic outbox, restart recovery, source expiry and visible recording controls.
- [x] Implemented campaign metadata, activity totals, reviews, task corrections, search and access audit.
- [x] Passed 38 actual PostgreSQL checks and the model/regex-boundary contract suite.
- [x] Passed just fix, just fmt and 33 focused Rust tests; built on mounted data storage.
- [x] Exercised corbanu-debug --yolo through real PTY keys: recording, Unicode prompt, replay, provider outage, restart/deduplication, grant/revoke, goal, prompt review, campaign creation and pause.
- [x] Passed a separate live pinned GLM 5.3 Flash subscription call.
- [x] Recorded executable identity, disk usage, scope limits and rollout requirements.
- [x] Preserved unrelated working-tree edits and captured pre-existing global sprint checker conflicts.

## Remaining

None for this local integration and qualification sprint. Production rollout, backup restoration, advanced campaign/export UI and pilot calibration remain in PF-78's product follow-up; they are explicitly described in the qualification record and are not claimed complete.

## Verification

33 focused Rust tests, 38 PostgreSQL checks, the model contract checks, live Flash inference and actual PTY workflows passed. Plan validation passed; unrelated global sprint conflicts remain.

## Exit evidence

[Qualification record](../../../../qa/campaign-tracker/2026-09-06/README.md).
Final executable SHA-256: 8e6b66e67c1156f6faa2f0d1c366c03bfb6ff601bf9d709b7169ded8b67cb333.
The candidate is built from the recorded dirty checkout; no commit or release was created. Server migration 140 and summary schema v1 are implemented. Fixture inference and the separate live call are identified distinctly.
