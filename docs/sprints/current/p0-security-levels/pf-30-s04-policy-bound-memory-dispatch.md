---
sprint_id: "PF-30-S04"
title: "Policy-bound stage-one memory dispatch"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-30"
execution_order: 77
owner: "/root/security_ui"
parallel_lane: "memory-dispatch"
write_scope: "codex-rs/core/src/memory_stage_one.rs, codex-rs/core/src/memory_stage_one_tests.rs, codex-rs/core/src/session/memory_stage_one.rs, codex-rs/core/src/client.rs, codex-rs/core/src/session/mod.rs, codex-rs/memories/write/src/runtime.rs, codex-rs/memories/write/src/phase1.rs, codex-rs/memories/write/src/start.rs, codex-rs/memories/write/src/startup_tests.rs, codex-rs/core/tests/suite/memory_stage_one_policy.rs, codex-rs/tui/tests/suite/memory_stage_one_policy.rs, qa/security-levels/sprints/PF-30-S04/, docs/sprints/current/p0-security-levels/pf-30-s04-policy-bound-memory-dispatch.md"
integration_gate: "Codex /root owns Core lib/codex_thread factory exports and shared test/module/Cargo/Bazel registration, verifies no overlap with PF-30-S01, and reruns affected Core/memories/TMUX tests on RTX. Deny-only protected memory must not claim lineage/screening readiness. Astra High and Fable 5.1 High through Corbanu TMUX, maximum five reviews for this new track."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/security-memory-dispatch"
branch: "feat/security-memory-dispatch"
base_commit: "526926934fa650b8eb6c4e6887d1f7461c26f38f"
depends_on: "PF-22-S02"
created: 2026-09-04
updated: 2026-09-04
---

# PF-30-S04 — Policy-bound stage-one memory dispatch

## Execution mandate

- Deliver: the existing stage-one worker obtains an opaque client from its owning live session; unsupported protected, unavailable or wrong-owner dispatch fails closed, including retries.
- Excludes: positive protected inference, persistent provenance/taint, phase-two redesign, new policy settings, memory activation, OS setup and PF-35 qualification.

## Plan linkage

- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md), feature PF-30.
- Product citation: **Non-negotiable controls** — “Classify instruction intent and provenance before external content can influence tools or financial actions.”
- This independent denial boundary consumes the completed PF-22-S02 runtime-policy contract, not unfinished PF-30-S01 screening or PF-30-S02 persistence.
- [Design and exact API proposal](../../../../qa/security-levels/sprints/PF-30-S04/memory-policy-binding-design.md) is adopted with this mandate as authority; its original preparation-only status is retained as historical provenance.
- PF-30-S02 remains draft with its existing dependencies. Execution order 77 is a topological identifier, not a requirement to wait for every lower-numbered unrelated sprint.

## Code boundaries

- Worker-owned paths are literal in front matter; shared exports and suite registries remain coordinator-owned.
- Use a CodexThread-derived opaque factory and dedicated stage-one facade, never a public policy setter or arbitrary authority selector.
- client.rs and session/mod.rs ownership is transferred from PF-30-S01 before dispatch. Neither lane edits the other's files.

## Preconditions

- [x] Active plan and completed/archived PF-22-S02 runtime contract.
- [x] User authorized rolling replacement lanes; coordinator records this genuine single-feature contract split without enabling protected memory.
- [x] Exact branch/worktree/base and non-overlapping scope allocated.
- [ ] Read root/Rust/Core/TUI policies and verify source/API assumptions before implementation.

## Done

- [x] Existing detached construction, lower-level retry risk and historical-source/runtime-owner distinction documented.

## Remaining

- [ ] Implement one host-owned opaque stage-one client factory; expected identity/provider are assertions, never authority selectors. Unknown/uninitialized/terminated/mismatched bindings deny.
- [ ] Preserve maximum configured/current/inherited policy floor; fail closed for all protected raw-rollout dispatch independently of admission/screening readiness.
- [ ] Check at actual HTTP retry dispatch and post-connect WebSocket send, then stream completion and before worker success persistence. Preserve Permissive payload, proxy, auth and cache shaping.
- [ ] Remove detached worker construction; use current per-job context, typed bounded denial and finite existing retry/backoff. Do not mark denied/EOF/cancelled work successful.
- [ ] Preserve feature-disabled, ephemeral, non-root and unavailable-DB skips; no new model traffic or memory activation.
- [ ] Cover real worker canary paths, wrong owner, live increase barriers, provider replacement, HTTP/WS retry, restart/cancel and legacy/mixed/forged sources. Historical rollout IDs remain data, not runtime owner IDs.

## Verification

- [ ] RTX only: scoped just fix, full just fmt, affected Core and full memories-write suites; relevant memories-read regression suite.
- [ ] Fake providers and synthetic fixtures only; prove zero canary-bearing requests under protected/unavailable policy and unchanged Permissive success.
- [ ] Actual-key TMUX startup/turn/cancel/restart with memory-enabled isolated home and recorded fake-provider/DB outcomes; text and Enter separately.
- [ ] Astra High and Fable 5.1 High review via Corbanu/private TMUX with exact frozen source; maximum five invocations, no redundant reviews.
- [ ] Coordinator exports/registries, lock parity if needed, combined-tree and governance checks.

## Exit evidence

- [ ] Commit, immutable RTX binary hash, tests, keys/captures and review dispositions recorded under the new QA directory.
- [ ] Denial-only boundary and remaining persistence/positive-screening limitations explicit; no human, cross-platform or release pass inferred.
- [ ] Done/Remaining reflect reality; archive only after all required final-tree evidence.
