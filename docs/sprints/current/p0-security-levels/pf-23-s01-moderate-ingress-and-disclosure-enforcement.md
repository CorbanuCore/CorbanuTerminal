---
sprint_id: "PF-23-S01"
title: "Moderate ingress and disclosure enforcement"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-23"
execution_order: 40
owner: "untrusted-content lane"
parallel_lane: "untrusted-content"
write_scope: "codex-rs/core/src/security/protected_surface.rs, codex-rs/core/src/security/protected_surface/, codex-rs/core/src/security/protected_surface_tests.rs, codex-rs/core/src/security/tainted_action.rs, codex-rs/core/src/security/tainted_action/, codex-rs/core/src/security/tainted_action_tests.rs, codex-rs/core/src/security/ingress/native.rs, codex-rs/core/src/client.rs, codex-rs/core/src/mcp_tool_call.rs, codex-rs/core/src/tools/registry.rs, codex-rs/core/src/tools/code_mode/mod.rs, codex-rs/core/src/tools/handlers/dynamic.rs, codex-rs/core/src/tools/handlers/extension_tools.rs, codex-rs/core/src/tools/handlers/mcp.rs, codex-rs/core/src/tools/handlers/unified_exec/write_stdin.rs, codex-rs/core/tests/suite/pf_23_s01.rs, codex-rs/core/tests/suite/mod.rs, qa/security-levels/sprints/PF-23-S01/, qa/demos/specs/, qa/demos/index/PF-23-S01.md, docs/sprints/current/p0-security-levels/pf-23-s01-moderate-ingress-and-disclosure-enforcement.md"
integration_gate: "Per-sprint gate of 2026-10-06: focused tests, GLM 5.2 tmux demos, one independent Opus 5.5 High review; merge behind source_envelopes. Not reserved here and serialized by the integration owner: codex-rs/core/src/security/mod.rs (PF-27-S02), codex-rs/core/src/tools/sandboxing.rs, codex-rs/core/src/unified_exec/process_manager.rs and codex-rs/Cargo.lock."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf23-s01-20261006"
branch: "feat/pf-23-s01-moderate-ingress"
base_commit: "55339d5b24d955cf245efa7b8af9243eac66254a"
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

- Route matrix and post-taint check outside the approval seam: `core/src/security/protected_surface.rs`,
  `protected_surface/{gate,typed}.rs`; dispatch-boundary call in `core/src/tools/registry.rs`
  (`CoreToolRuntime::tool_origin`, overridden by MCP, dynamic and extension tools).
- Routes: `core/src/mcp_tool_call.rs`, `tools/handlers/unified_exec/write_stdin.rs`, `tools/code_mode/mod.rs`
  (nested results raise the taint generation through `client.rs` / `security/ingress/native.rs`).
- Shell classifier kinds `Disclosure` and `ValueTransfer`: `core/src/security/tainted_action/outbound.rs`.
- Tests: `protected_surface_tests.rs`, `tainted_action_tests.rs`, `core/tests/suite/pf_23_s01.rs`.

## Preconditions

- [x] PF-13-S05, PF-22-S02, PF-30-S03 are completed and archived.
- [x] Read root, `codex-rs/AGENTS.md`, and `codex-rs/core/AGENTS.md`.
- [x] Exact worktree coordinates match the active plan (stale PF-30-S03 slice-1 entry replaced, 2026-10-06).

## Done

- [x] Sprint record is linked only to PF-23.
- [x] Slice 1 ([gate](../../../../qa/security-levels/sprints/PF-23-S01/slice-1-gate.md)), behind `source_envelopes`
  with Moderate/Aggressive; flag off, Permissive and untainted sessions unchanged:
  - Typed route matrix at the Core dispatch boundary. Shell/exec/patch use the PF-30-S03 seam; MCP calls and
    `write_stdin` have their own check; code mode only reaches tools through dispatch; child-agent, read and
    report tools reach nothing protected. Client (dynamic) tools, unlisted extension tools and unknown built-ins
    are unclassified and need a fresh human answer after untrusted content, refused under `never`;
    `request_permissions` needs it when an automatic reviewer would answer, `request_plugin_install` always.
  - MCP calls: protected when the tool may change or send data (its annotations can only add protection), when
    its name moves value (read verbs excepted), or when its arguments reach a protected path or command. The
    check runs before remembered approvals, hooks, auto-approve and the automatic reviewer; the question shows
    the arguments and uses an id the delegate's reviewer never answers.
  - `write_stdin`: the text typed into a process since untrusted content is judged whole (split commands),
    as input to the program it goes to; line editing, history expansion and over 16 KiB are unreadable; writes
    to one process are serialized.
  - Code mode: each nested result raises the taint generation as the cell reads it.
  - Disclosure (local file, stdin or unseen text sent to another machine; literal bodies and loopback are quiet)
    and value transfer (wallet CLIs, MCP tools), through exec-style wrappers.
  - Decisions log `route`, kind, taint generation, outcome and wait time; never arguments.

## Remaining

- [ ] Slice 2, moved from PF-30-S02: positive protected memory extraction. Stage one still denies under
  Moderate/Aggressive (PF-30-S04); allowing it needs labelled, lineage-bound rollout input and the
  [stage-one handoff](../../../../qa/security-levels/sprints/PF-30-S01-typed-source-envelope/memory-stage-one-follow-up.md) matrix.
- [ ] Slice 3, moved from PF-30-S03: what a command-text net cannot see (run-time strings, build tools, hard
  links, unknown exec wrappers, reads of a home through an unclassified route). Typed resources plus
  sandbox-level denial of home and credential reads replace the lexical net. Lexical gaps from the PF-30-S03
  review: `cd` in a substitution or subshell, positional parameters/functions/`set --`, `su -c`/`runuser -c`/
  `script -c`/`ssh host cmd`, `${!x}`, `cd -P`/`||`, loop stdin from a process substitution, `awk system()`/
  `sed e`, automount symlink hops.
- [ ] Action/profile usability matrix with conservative data and control-flow ancestry; no runtime Moderate
  activation until all required subsystems qualify (full plan readiness matrix).
- [ ] Slice-1 known limits, listed in its [gate](../../../../qa/security-levels/sprints/PF-23-S01/slice-1-gate.md#known-limits).
- [ ] Product finding (not this sprint): Chat Completions and Anthropic wires drop namespace (MCP) tools.

## Verification

- [ ] Per slice: `just fix -p codex-core`, `just fmt`, `just test -p codex-core pf_23_s01` and `pf_30_s0`, full
  `just test -p codex-core`, Linux clippy on the RTX box, GLM 5.2 tmux demos, one Opus 5.5 High review
  (slice 1 done: [gate](../../../../qa/security-levels/sprints/PF-23-S01/slice-1-gate.md)).

## Exit evidence

- [ ] Commit, typed surface matrix, and changed paths recorded.
- [ ] Test output linked under `qa/security-levels/sprints/PF-23-S01/`.
- [ ] Ledgers reflect reality and the completed record is archived.
