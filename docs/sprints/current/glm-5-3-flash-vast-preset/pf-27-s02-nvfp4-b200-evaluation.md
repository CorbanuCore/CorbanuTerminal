---
sprint_id: "PF-27-S02"
title: "NVFP4 B200 matched evaluation"
status: in_progress
plan_file: "docs/plans/active/glm-5-3-flash-vast-preset.md"
plan_feature: "PF-27"
execution_order: 2
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-glm53-flash"
branch: "feat/glm-5-3-flash-vast-preset"
base_commit: "1bdc515bff48a4d9048dae7d06c6214e884265bc"
depends_on: "PF-27-S01"
created: 2026-08-27
updated: 2026-08-27
---

# PF-27-S02 — NVFP4 B200 matched evaluation

## Execution mandate

- Deliver: an immutable experimental two-B200 NVFP4 route and matched GLM evaluation evidence.
- Excludes: quality-equivalence claims, automatic promotion, unbounded spend, SGLang, and changes to the qualified FP8 presets.

## Plan linkage

- Plan: [GLM-5.3-Flash Vast GPU preset](../../../plans/active/glm-5-3-flash-vast-preset.md)
- Feature: `PF-27`
- Acceptance advanced: the newly published NVFP4 checkpoint can be evaluated through the existing bounded authenticated rental lifecycle.

## Code boundaries

- Existing: `codex-rs/gpu-market/src/{glm_recipes,recipe,recipe_tests}.rs`
- Planned: `benchmarks/coding/configs/glm53-flash-nvfp4-gpu.json`
- Evidence: `qa/gpu-rentals/benchmarks/glm53-nvfp4-b200/`; `benchmarks/{coding,website-builder}/runs/`
- TUI: existing `/gpu` catalog and billing confirmation; no new interaction primitive

## Preconditions

- [x] Plan is active and PF-27-S01 is completed and archived.
- [x] Worktree, branch, and base commit are exact and match the plan.
- [x] Model revision, 181.291 GiB checkpoint size, dedicated image digest, and engine instructions were revalidated.
- [x] Read-only Vast search found a funded account and a compatible verified two-B200 NVLink offer; no rental was created.
- [ ] Product owner confirms fresh exact hourly, total-spend, and duration caps before final billable confirmation.

## Done

- [x] Sprint record created and linked to one plan feature.
- [x] Evaluation scope freezes the existing EventForge, LogTriage, QueueCraft, website-builder, and mixed-context workloads.
- [x] NVFP4 remains experimental until live evidence is reviewed.
- [x] Added the immutable two-B200 recipe with exact checkpoint/image, CUDA 13, topology, authentication, parser, context, and capacity bounds.
- [x] Added focused recipe/catalog regressions and a reviewed GPU-menu snapshot.
- [x] Reused the content-free mixed-context harness under an NVFP4-specific evidence path.
- [x] Built `corbanu-debug`; the true-tmux test opened `/gpu` with separately sent text/Enter, observed the NVFP4 route, and cancelled with Escape.

## Remaining
- [ ] After fresh financial authorization, select an in-cap offer and complete the product's final billable confirmation.
- [ ] Reach READY and verify model, auth rejection, chat, stream, cancellation, and forced tool-call contracts.
- [ ] Run EventForge, LogTriage, QueueCraft, website-builder, and the matched mixed-context sweep without cross-workload contention.
- [ ] Terminate through `/gpu` and confirm provider absence, token cleanup, endpoint removal, and final spend.

## Verification

- [x] Fix: `cd codex-rs && just fix -p codex-gpu-market`.
- [x] Format: `cd codex-rs && just fmt`; inspected the final diff.
- [x] Focused and full tests: 15 recipe tests and all 80 `codex-gpu-market` tests passed.
- [x] Harness tests and validate-only mode passed for the copied mixed workload.
- [x] GPU-menu unit/snapshot/true-tmux selection-cancel group: 9 passed.
- [ ] TUI applicability: required; actual selection, cancel, billable confirm, status, and termination keys/checkpoints recorded.
- [ ] Governance and `git diff --check` pass on the final tree.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Model, image, offer, hardware, READY, eval, timing, and spend evidence linked without prompts, outputs, endpoints, or secrets.
- [ ] Provider-confirmed termination and no remaining billable resource are recorded.
- [ ] Done and Remaining ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/glm-5-3-flash-vast-preset/`.
