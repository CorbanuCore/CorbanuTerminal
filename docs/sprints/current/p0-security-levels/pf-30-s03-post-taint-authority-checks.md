---
sprint_id: "PF-30-S03"
title: "Post-taint authority checks"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-30"
execution_order: 39
owner: "untrusted-content lane"
parallel_lane: "untrusted-content"
write_scope: "codex-rs/core/src/security/tainted_action.rs, codex-rs/core/src/security/tainted_action_tests.rs, codex-rs/core/src/security/ingress/native.rs, codex-rs/core/src/security/ingress/pf_30_s02_tests.rs, codex-rs/core/src/client.rs, codex-rs/core/src/session/mod.rs, codex-rs/core/src/tools/orchestrator.rs, codex-rs/core/src/tools/approvals.rs, codex-rs/core/src/tools/runtimes/shell.rs, codex-rs/core/src/tools/runtimes/unified_exec.rs, codex-rs/core/tests/suite/provenance.rs, qa/security-levels/sprints/PF-30-S03/, qa/demos/specs/, qa/demos/index/PF-30-S03.md, docs/sprints/current/p0-security-levels/pf-30-s03-post-taint-authority-checks.md"
integration_gate: "Per-sprint gate of 2026-10-06: focused tests, GLM 5.2 tmux demos, one independent Opus 5.5 High review; merge behind source_envelopes. Shared with PF-27-S02 and serialized by the integration owner, not reserved here: the one-line module registration in codex-rs/core/src/security/mod.rs, the fresh_human_authority field and cache bypass in codex-rs/core/src/tools/sandboxing.rs, and the preapproval bypass in codex-rs/core/src/tools/runtimes/apply_patch.rs."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf30-s03-20261006"
branch: "feat/pf-30-s03-post-taint"
base_commit: "743a7c22dabba6a0368e8390dcd9dd7b2ecdd9e8"
depends_on: "PF-30-S02, PF-13-S05"
created: 2026-08-28
updated: 2026-10-06
---

# PF-30-S03 — Post-taint authority checks

## Execution mandate

- Deliver: A classifier false negative or tainted summary cannot mint or widen protected authority.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-30).
- Feature: `PF-30`.
- Product citation: **Non-negotiable controls** — “Classify instruction intent and provenance before external content can influence tools or financial actions.”
- Acceptance advanced: A classifier false negative or tainted summary cannot mint or widen protected authority.
- Sources and archive disposition: [PF-30 reconciliation](../../../plans/security-source-reconciliation.md#pf-30).

## Code boundaries

- OpenClaw adoption reference: [OC-4](../../../plans/openclaw-source-review-2026-08-28.md#oc-4), [OC-5](../../../plans/openclaw-source-review-2026-08-28.md#oc-5), [OC-11](../../../plans/openclaw-source-review-2026-08-28.md#oc-11) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; reference tests are not candidate evidence.
- Slice 1: taint generation (`security/ingress/native.rs`, `client.rs`, `session/mod.rs`), action
  classification (`security/tainted_action.rs`), and the gate at the shared approval seam
  (`tools/orchestrator.rs`, `tools/approvals.rs`, `tools/sandboxing.rs`, shell/unified-exec/apply-patch runtimes).
- Tests: `pf_30_s03` unit and suite tests; fixtures use synthetic content and fake providers only.

## Preconditions

- [x] Active plan; PF-30-S02 and PF-13-S05 completed and archived. Decisions of 2026-10-06 apply (flagged merge).

## Done

- [x] Allocated 2026-10-06 to the untrusted-content lane after PF-30-S02 closed.
- [x] Taint generation: each recorded batch with content lacking standing (tool, MCP, agent, memory,
  unattributed, restored without a record) raises a per-session counter that never goes down.
- [x] With `source_envelopes` and Moderate/Aggressive, once the session is tainted, a shell, exec or patch
  action that reaches the vault, credential stores or security policy (Corbanu config, rules, hooks,
  login state, anything under `CODEX_HOME`) needs a fresh human approval of that exact action. A cached
  session approval, a permission hook's allow, the automatic reviewer and preapproved patch scope cannot
  stand in; a "for this session" answer counts once. With `approval_policy = never` it is refused, not run.
- [x] The prompt says why; the decision is re-checked after the prompt: taint that arrived while it was
  open refuses the action. Each check logs its kind and taint generation.
- [x] Classification resolves `~`/`$HOME`/`$CODEX_HOME`, `.`/`..`, `cd`, quotes and globs, finds CLI and
  credential commands behind wrappers, and treats agent worktrees and ordinary project files as ordinary.
- [x] Tests: classification (positive, negative, evasion, custom home), taint counting, approval required
  (no "don't ask again" rule), approvals off, untainted/Permissive/ordinary unchanged, session approval not
  reused, automatic reviewer bypassed.

## Remaining

- [ ] MCP tool calls, `write_stdin` into running processes, code-mode and other dispatch routes that do not
  pass the shared approval seam; typed protected resources replace the lexical net in PF-23-S01.
- [ ] Bind the policy epoch and lineage to approvals (stale on grant change or revocation), not only the
  taint generation; outbound disclosure requests.
- [ ] Memory-recall → protected-action and child confused-deputy end-to-end tests; quoted malicious trades.
- [ ] Record research-workflow approval counts and latency for PF-26.
- [ ] Slice 1 gaps from review: tests for taint arriving while a prompt is open, a hook's allow, apply-patch
  preapproval and the escalation retry; folder tracking beyond a leading `cd`; ANSI-C quoting.

## Verification

- [x] `just fmt`; `just fix -p codex-core`.
- [x] Focused: `just test -p codex-core pf_30_s0` (84 pass, 9 `pf_30_s03`).
- [x] Slice 1: GLM 5.2 tmux demos; independent Opus 5.5 High review, four rounds, final APPROVE.
- [ ] Later slices get the same gate; milestone VM run and sign-off when Moderate ships.

## Exit evidence

- [x] Slice 1 gate: [post-taint-gate.md](../../../../qa/security-levels/sprints/PF-30-S03/post-taint-gate.md);
  videos in [qa/demos/index/PF-30-S03.md](../../../../qa/demos/index/PF-30-S03.md).
- [ ] Slices merged to main behind `source_envelopes`; record archived.
