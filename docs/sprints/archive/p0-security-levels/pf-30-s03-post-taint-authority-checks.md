---
sprint_id: "PF-30-S03"
title: "Post-taint authority checks"
status: completed
plan_file: "docs/plans/completed/main-2026-10-08-p0-security-levels.md"
plan_feature: "PF-30"
execution_order: 39
owner: "untrusted-content lane"
parallel_lane: "untrusted-content"
write_scope: "codex-rs/core/src/security/tainted_action.rs, codex-rs/core/src/security/tainted_action/, codex-rs/core/src/security/tainted_action_tests.rs, codex-rs/core/src/tools/runtimes/apply_patch_tests.rs, codex-rs/core/src/session/tests.rs, codex-rs/core/Cargo.toml, codex-rs/Cargo.lock, codex-rs/core/src/security/ingress/native.rs, codex-rs/core/src/security/ingress/pf_30_s02_tests.rs, codex-rs/core/src/client.rs, codex-rs/core/src/session/mod.rs, codex-rs/core/src/tools/orchestrator.rs, codex-rs/core/src/tools/approvals.rs, codex-rs/core/src/tools/runtimes/shell.rs, codex-rs/core/src/tools/runtimes/unified_exec.rs, codex-rs/core/tests/suite/provenance.rs, qa/security-levels/sprints/PF-30-S03/, qa/demos/specs/, qa/demos/index/PF-30-S03.md, docs/sprints/current/p0-security-levels/pf-30-s03-post-taint-authority-checks.md"
integration_gate: "Per-sprint gate of 2026-10-06: focused tests, GLM 5.2 tmux demos, one independent Opus 5.5 High review; merge behind source_envelopes. Shared with PF-27-S02 and serialized by the integration owner, not reserved here: the one-line module registration in codex-rs/core/src/security/mod.rs, the fresh_human_authority field and cache bypass in codex-rs/core/src/tools/sandboxing.rs, and the preapproval bypass in codex-rs/core/src/tools/runtimes/apply_patch.rs."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf30-s03b-20261006"
branch: "feat/pf-30-s03-finish"
base_commit: "38516a5b22"
depends_on: "PF-30-S02, PF-13-S05"
created: 2026-08-28
updated: 2026-10-06
---

# PF-30-S03 — Post-taint authority checks

## Execution mandate

- Deliver: A classifier false negative or tainted summary cannot mint or widen protected authority.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/completed/main-2026-10-08-p0-security-levels.md#pf-30).
- Feature: `PF-30`.
- Product citation: **Non-negotiable controls** — “Classify instruction intent and provenance before external content can influence tools or financial actions.”
- Acceptance advanced: A classifier false negative or tainted summary cannot mint or widen protected authority.
- Sources and archive disposition: [PF-30 reconciliation](../../../plans/security-source-reconciliation.md#pf-30).

## Code boundaries

- OpenClaw adoption reference: [OC-4](../../../plans/openclaw-source-review-2026-08-28.md#oc-4), [OC-5](../../../plans/openclaw-source-review-2026-08-28.md#oc-5), [OC-11](../../../plans/openclaw-source-review-2026-08-28.md#oc-11) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; reference tests are not candidate evidence.
- Taint generation: `security/ingress/native.rs`, `client.rs`, `session/mod.rs`.
- Classification: `security/tainted_action.rs` and `security/tainted_action/{shell,paths,invocation,indirect}.rs`.
- Gate at the shared approval seam: `tools/orchestrator.rs`, `tools/approvals.rs`, `tools/sandboxing.rs`,
  shell/unified-exec/apply-patch runtimes.
- Tests: `pf_30_s03` unit and suite tests; fixtures use synthetic content and fake providers only.

## Preconditions

- [x] Active plan; PF-30-S02 and PF-13-S05 completed and archived. Decisions of 2026-10-06 apply (flagged merge).

## Done

- [x] Slice 1 (PR #204, merge `38516a5b22`); slice 2 (PR #212): taint generation; with `source_envelopes` and Moderate/Aggressive,
  once the session is tainted a shell, exec or patch action that reaches the vault, credential stores or security
  policy needs a fresh human approval of that exact action. Cached session approvals, hook allows, the automatic
  reviewer and patch preapproval cannot stand in; `approval_policy = never` refuses.
- [x] Slice 2: the approval is bound to the taint generation and the effective policy (epoch, revocations, kill
  switch, level, lineage). A change while the prompt is open, at the first prompt or the escalation retry,
  refuses; the kill switch refuses before asking.
- [x] Slice 2: the classifier follows indirect routes: shell quoting and expansion forms, variables, folder
  changes, symlinks and canonical spellings, inline interpreter code, encoded payloads, script files, patched
  scripts and per-interpreter options. Code it cannot read before it runs (piped downloads, unseen variables or
  substitutions, unreadable scripts, any limit hit) is a fourth kind, "running code the host cannot read first",
  and is gated the same way.
- [x] Tests for the slice-1 review gaps: taint arriving while a prompt is open (orchestrator probe), a hook's
  allow ignored, apply-patch preapproval ignored, the escalation retry asking again; end-to-end recalled memory
  and a tricked child agent; PF-26 research-workflow counts.
- [x] PF-26: each decision logs kind, taint generation, outcome and the human's wait; classification logs its
  time. A 25-command research workflow plus 5 protected actions gives exactly 5 prompts, about 1 ms per
  classification (debug build).

## Remaining

Nothing. The milestone VM run and human sign-off happen when Moderate ships (program milestone). These items were
moved to [PF-23-S01](pf-23-s01-moderate-ingress-and-disclosure-enforcement.md), which owns typed protected
surfaces:

- MCP tool calls, `write_stdin` into running processes and code mode, which do not pass the shared approval seam.
- What a command-text classifier cannot see: strings built at run time, build tools that run arbitrary code, hard
  links, and the remaining lexical gaps the review listed. Typed resources and sandbox-level denial of home reads
  replace the net.
- Outbound disclosure requests and value transfer proposed from tainted content.

## Verification

- [x] `just fmt`; `just fix -p codex-core`; Bazel lock unchanged.
- [x] Focused: `just test -p codex-core pf_30_s03` (27 pass); full crate with only the three known baselines
  failing.
- [x] Each slice: GLM 5.2 tmux demos and an independent Opus 5.5 High review until APPROVE.

## Exit evidence

- [x] Slice 1 gate: [post-taint-gate.md](../../../../qa/security-levels/sprints/PF-30-S03/post-taint-gate.md).
- [x] Slice 2 gate: [slice-2-gate.md](../../../../qa/security-levels/sprints/PF-30-S03/slice-2-gate.md).
- [x] Videos: [qa/demos/index/PF-30-S03.md](../../../../qa/demos/index/PF-30-S03.md).
- [x] Both slices merged to main behind `source_envelopes`; record archived.
