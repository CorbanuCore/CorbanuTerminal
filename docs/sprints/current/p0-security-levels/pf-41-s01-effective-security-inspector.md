---
sprint_id: "PF-41-S01"
title: "Effective security inspector and degradation state"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-41"
execution_order: 71
owner: "convergence lane worker (2026-10-07)"
parallel_lane: "tui"
write_scope: "codex-rs/core/src/security/inspection.rs, codex-rs/core/src/security/inspection_tests.rs, codex-rs/core/src/security/effective_policy.rs, codex-rs/core/src/security/aggressive.rs, codex-rs/core/src/security/launch_contract.rs, codex-rs/core/src/security/protected_surface/gate.rs, codex-rs/core/src/security/mod.rs, codex-rs/core/src/tools/orchestrator.rs, codex-rs/core/src/client.rs, codex-rs/core/src/lib.rs, codex-rs/app-server-client/src/lib.rs, codex-rs/tui/src/security/inspector.rs, codex-rs/tui/src/security/inspector_tests.rs, codex-rs/tui/src/security/mod.rs, codex-rs/tui/src/bottom_pane/security_inspector.rs, codex-rs/tui/src/bottom_pane/security_inspector_tests.rs, codex-rs/tui/src/bottom_pane/security_view.rs, codex-rs/tui/src/bottom_pane/security_view_tests.rs, codex-rs/tui/src/bottom_pane/mod.rs, codex-rs/tui/src/bottom_pane/snapshots/, codex-rs/tui/src/chatwidget/slash_dispatch.rs, qa/security-levels/sprints/PF-41-S01/, qa/demos/specs/pf41-inspector-live-aggressive.toml, qa/demos/specs/pf41-inspector-taint-denial.toml, qa/demos/specs/pf41-inspector-failure-recovery.toml, qa/demos/index/PF-41-S01.md"
integration_gate: "Per-sprint gate of 2026-10-06 behind security_levels: focused tests, GLM 5.2 tmux run, one Opus 5.5 High review, SOP videos, Linux clippy on the RTX box. Not touched: bottom_pane/security_level_picker* (PF-25 lane)."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-41-s01-20261007"
branch: "pf-41-s01-inspector"
base_commit: "e4d17dbdc6f0d6a4c460d9005fd7cc73aebc2566"
depends_on: "PF-23-S03, PF-29-S02, PF-24-S02"
merged_behind_flag: "security_levels"
gate_evidence: "qa/security-levels/sprints/PF-41-S01/README.md"
created: 2026-08-28
updated: 2026-10-07
---

# PF-41-S01 — Effective security inspector and degradation state

## Execution mandate

- Deliver: The user can inspect the protection actually enforced, including degradation and recent denials.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-41).
- Feature: `PF-41`.
- Product citation: **Non-negotiable controls** — “Record tamper-evident policy decisions, tool calls, approvals, signatures, and transaction or order IDs without secrets.”
- Acceptance advanced: The user can inspect the protection actually enforced, including degradation and recent denials.
- Sources and archive disposition: [PF-41 reconciliation](../../../plans/security-source-reconciliation.md#pf-41).

## Code boundaries

- OpenClaw adoption reference: [OC-6](../../../plans/openclaw-source-review-2026-08-28.md#oc-6), [OC-7](../../../plans/openclaw-source-review-2026-08-28.md#oc-7), [OC-8](../../../plans/openclaw-source-review-2026-08-28.md#oc-8) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/tui/src/slash_command.rs; PF-22 runtime policy; PF-24 security view.
- Planned: codex-rs/core/src/security/inspection.rs; codex-rs/tui/src/bottom_pane/security_inspector.rs.
- Tests: planned colocated Rust test modules prefixed `pf_41_s01`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [x] Active plan; PF-23-S03 archived, PF-29-S02 (#235) and PF-24-S02 (#253) merged behind their flags. Decision 3 (2026-10-06) dropped PF-32-S06, PF-37-S02 and PF-40-S03; those controls belong to the P1 hardening plan.
- [x] Read root and nearest implementation-path AGENTS.md; worktree coordinates above.
- [x] Source pins, crate/module paths and APIs confirmed on `e4d17dbdc6`.

## Done

Gate: [qa/security-levels/sprints/PF-41-S01/README.md](../../../../qa/security-levels/sprints/PF-41-S01/README.md).

- [x] New single-feature record reconciled with current ownership and archived design input.
- [x] `i` in `/security` (picker only; flag-off view unchanged) opens a read-only inspector. Esc returns, `r` reads
  again, arrows scroll under a pinned header; no key changes level, grants, revocations or a stop.
- [x] Configured (saved level, Core config layers), resolved (launch check, session config, platform backend) and
  observed (Core's live policy with its generation, runtime controls, taint, grants, denials) facts are shown
  separately, each with its source. Observation age is shown and redrawn every second; after 30 s it is stale.
- [x] Badge: green only when nothing is degraded, every required control was observed or checked at launch, and the
  facts are fresh. Config-only levels, no live policy, an unobserved broker, Core behind the launch level, a saved
  level waiting for restart, stale facts, a missing sandbox, an unclean boundary or a degraded control all prevent it.
  Kill switch, unreadable state or a stopped session read Blocked.
- [x] Core snapshot (`core/src/security/inspection.rs`): live tree, every agent with its level, stricter-than-session
  and stopped flags; held grants (surface, expiry, uses; never ids or operations); taint from the session's registry;
  launch contract and hardening, output gate, model key broker; recent denials (post-taint refusals and declines,
  including interrupted ones, and launch-contract refusals) as fixed text only.
- [x] MCP servers, hooks, `!` commands and app-server `command/exec` read "not contained" (with or without the
  environment allowlist); PF-32, PF-37, PF-34 and PF-40 read "not available". Nested agents shows refuse or pass.
- [x] Tests: conflicting config, unsupported platform, stale health, broker crash, expired grant, tainted session,
  inherited stricter child, kill switch and unreadable state, read-only keys; 6 core and 14 TUI `pf_41_s01` tests.

## Remaining

- [ ] Full `just test -p codex-core` / `-p codex-tui` suites run in post-merge CI (only focused and security filters
  ran here); one pre-existing TMPDIR-length flake in a picker test is noted in the gate record.
- [ ] Code-blind functional design/execution and human sign-off are milestone work (flag removal), not this gate.
- [ ] Not observed here, so never green: the network credential broker's health and the model key broker's health.
  Launch-contract denials carry no thread (shown as "this process").

## Verification

- [x] `just fix -p codex-core -p codex-tui -p codex-app-server-client`, then `just fmt`; diff inspected.
- [x] Focused: `just test -p codex-core pf_41_s01` (6/6) and `just test -p codex-tui pf_41_s01` (14/14) on the final tree.
- [x] Security filters: core `security tainted orchestrator` and `taint aggressive launch protected pf_23 pf_30`;
  TUI `security slash_command` (see the gate record). No manifest or lock changes.
- [x] TUI: GLM 5.2 tmux run and three videos: open `/security` inspector → inspect backend, taint, grants and denials →
  inject failure (`security_state.json` overwritten) → Blocked → confirm a level → restart → recovered.
- [x] Commits, commands, outcomes and video links recorded in the gate record; no production credentials.
- [x] Linux clippy (`-D warnings`; core, tui, app-server-client) on the RTX box.
- [ ] Full crate suites in post-merge CI; code-blind run at the flag-removal milestone.

## Exit evidence

- [x] Implementation commits and final-tree outputs under `qa/security-levels/sprints/PF-41-S01/`.
- [x] Acceptance assertions proven in tests and true-TUI keys after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Record stays current until the milestone gate (flag removal) closes the remaining items above.
