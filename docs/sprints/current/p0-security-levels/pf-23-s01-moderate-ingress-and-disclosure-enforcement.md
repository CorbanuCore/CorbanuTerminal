---
sprint_id: "PF-23-S01"
title: "Moderate ingress and disclosure enforcement"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-23"
execution_order: 40
owner: "untrusted-content lane"
parallel_lane: "untrusted-content"
write_scope: "codex-rs/core/Cargo.toml, codex-rs/core/src/security/protected_surface.rs, codex-rs/core/src/security/protected_surface/, codex-rs/core/src/security/protected_surface_tests.rs, codex-rs/core/src/security/tainted_action.rs, codex-rs/core/src/security/tainted_action/, codex-rs/core/src/security/tainted_action_tests.rs, codex-rs/core/src/security/ingress/native.rs, codex-rs/core/src/client.rs, codex-rs/core/src/mcp_tool_call.rs, codex-rs/core/src/tools/registry.rs, codex-rs/core/src/tools/code_mode/mod.rs, codex-rs/core/src/tools/handlers/dynamic.rs, codex-rs/core/src/tools/handlers/extension_tools.rs, codex-rs/core/src/tools/handlers/mcp.rs, codex-rs/core/src/tools/handlers/unified_exec/write_stdin.rs, codex-rs/core/tests/suite/pf_23_s01.rs, codex-rs/core/tests/suite/mod.rs, codex-rs/core/src/memory_stage_one.rs, codex-rs/core/src/memory_stage_one_tests.rs, codex-rs/core/src/accounting_tests.rs, codex-rs/core/src/tools/orchestrator.rs, codex-rs/memories/write/src/phase1.rs, codex-rs/memories/write/src/runtime.rs, codex-rs/memories/write/src/start.rs, codex-rs/memories/write/src/startup_tests.rs, codex-rs/memories/write/src/prompts.rs, codex-rs/core/src/tools/handlers/apply_patch.rs, codex-rs/core/src/tools/handlers/structured_edit.rs, codex-rs/core/src/tools/handlers/view_image.rs, codex-rs/core/src/session/mod.rs, qa/security-levels/sprints/PF-23-S01/, qa/demos/specs/, qa/demos/index/PF-23-S01.md, docs/sprints/current/p0-security-levels/pf-23-s01-moderate-ingress-and-disclosure-enforcement.md"
integration_gate: "Per-sprint gate of 2026-10-06: focused tests, GLM 5.2 tmux demos, one independent Opus 5.5 High review; merge behind source_envelopes. Not reserved here and serialized by the integration owner: codex-rs/core/src/security/mod.rs (PF-27-S02), codex-rs/core/src/tools/sandboxing.rs, codex-rs/core/src/unified_exec/process_manager.rs and codex-rs/Cargo.lock."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf23-s01-20261006"
branch: "feat/pf-23-s01-slice2-3"
base_commit: "8cf46179f569050bf066cc3367f593057023e178"
depends_on: "PF-13-S05, PF-22-S02, PF-30-S03"
created: 2026-08-24
updated: 2026-10-06
---

# PF-23-S01 — Moderate ingress and disclosure enforcement

## Execution mandate

- Deliver: Moderate deterministically blocks untrusted requests for secrets, protected financial data, policy changes, or protected actions.
- Excludes: Aggressive defaults, financial signing implementation, classifier training, TUI, and browser isolation.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-23`
- Reconciliation: [source decisions and archive mapping](../../../plans/security-source-reconciliation.md).
- Product citation: **P0 `/security` levels** — “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.”
- Acceptance advanced: normal analysis continues while hostile instructions cannot gain authority or protected data.

## Code boundaries

- Slice 1: `core/src/security/protected_surface.rs`, `protected_surface/{gate,typed}.rs`, dispatch call in
  `tools/registry.rs`; routes in `mcp_tool_call.rs`, `handlers/unified_exec/write_stdin.rs`, `code_mode/mod.rs`;
  shell kinds `Disclosure`/`ValueTransfer` in `security/tainted_action/outbound.rs`.
- Slice 2: `core/src/memory_stage_one.rs` (`label_rollout`), `memories/write/src/{phase1,runtime,start}.rs`.
- Slice 3: `core/src/security/protected_surface/read_denials.rs`, applied in `core/src/tools/orchestrator.rs`.
- Tests: `protected_surface_tests.rs`, `tainted_action_tests.rs`, `read_denials_tests.rs`,
  `memory_stage_one_tests.rs`, `memories/write/src/startup_tests.rs`, `core/tests/suite/pf_23_s01.rs`.

## Split with PF-29

Slice 3 is the runtime net: after untrusted content under Moderate/Aggressive, Core adds read denials to each
agent command's sandbox (Corbanu home, other Corbanu homes, fixed `$HOME` credentials such as `~/.ssh`). PF-29
owns launch and user data: S01 inventory and launch isolation (every credential file found, from launch), S02
migration of secrets out of shell profiles and config. Slice 3 never denies shell profiles; no shared files.

## Preconditions

- [x] PF-13-S05, PF-22-S02, PF-30-S03 are completed and archived.
- [x] Read root, `codex-rs/AGENTS.md`, and `codex-rs/core/AGENTS.md`; coordinates match the active plan.

## Done

All behind `source_envelopes` with Moderate/Aggressive; flag off, Permissive and untainted sessions unchanged.

- [x] Slice 1, #223 ([gate](../../../../qa/security-levels/sprints/PF-23-S01/slice-1-gate.md)): route matrix at the
  dispatch boundary (unclassified tools need a fresh human answer after taint); MCP calls that may change/send
  data, move value or name a protected path ask before remembered approvals, hooks and auto-review; typing into a
  process is judged as one command since taint; code-mode nested results raise the taint generation;
  Disclosure and value-transfer kinds; decisions logged without arguments.
- [x] Slice 2: stage one under Moderate. Core reads the claimed session's rollout (its opening record must be
  that session's), redacts each item, restores only origin records this home signed, labels the rest, drops whole
  items from the middle to fit the budget, and builds the message; the host gives only its redaction function and
  prompt template. The request must be exactly that message or it is refused before dispatch, also mid-job.
  Aggressive, and Moderate without the flag, deny; consolidation is skipped above Permissive.
- [x] Slice 3: after taint, agent command sandboxes, in-process file tools (patch pre-check, edits, image view,
  extension tools) and Codex Apps uploads deny reads of the Corbanu home (all but `tmp`, `shell_snapshots`, skills,
  plugins, packages, worktrees, `AGENTS.md`; fixed stores and `*.sqlite*` even before they exist), other Corbanu
  homes and `$HOME` credentials. Only the turn's folder, workspace and writable roots stay readable, never a
  command's own folder. Full access gets a sandbox that only denies those reads; no unsandboxed retries. Under
  Moderate a fresh human approval of the exact protected shell/exec command lifts them for that run (never for
  file tools). [Readiness matrix](../../../../qa/security-levels/sprints/PF-23-S01/activation-readiness.md).

## Remaining

- [ ] Slices 2-3 gate (tests, Linux clippy, GLM 5.2 videos, Opus review) and merge; then archive.

Moved: Aggressive grants lifting denials, write/action gaps of the command-text net and processes started before
untrusted content (sandbox fixed at spawn; typing still judged) to PF-23-S02; consolidation, Aggressive stage one
and the level a source session ran under to PF-23-S03. Known limits: external sandboxes and remote exec-server
environments take no extra rules (paths are this host's); where no sandbox can start (Linux without bubblewrap,
Windows unelevated) tainted full-access commands and file tools fail closed; extra permissions a human grants a
command can still open a path inside a denial (Aggressive turns those grants off); pre-existing hard links; MCP servers, hooks and notify run outside the
sandbox; the shell snapshot stays readable. Findings: Chat Completions and Anthropic wires drop namespace (MCP)
tools; GLM 5.2 wraps stage-one JSON in a code fence, so extraction fails at every level.

## Verification

- [ ] Per slice: `just fix -p codex-core -p codex-memories-write`, `just fmt`, `just test -p codex-core pf_23_s01`,
  `pf_30_s0`, `just test -p codex-memories-write`, full `just test -p codex-core`, Linux clippy on the RTX box,
  GLM 5.2 videos, one Opus 5.5 High review.

## Exit evidence

- [ ] Gates under `qa/security-levels/sprints/PF-23-S01/`, videos in `qa/demos/index/PF-23-S01.md`; archived.
