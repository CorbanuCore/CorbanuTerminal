---
sprint_id: "PF-27-S01"
title: "Curated GLM-5.3-Flash recipe and qualified endpoint"
status: in_progress
plan_file: "docs/plans/active/glm-5-3-flash-vast-preset.md"
plan_feature: "PF-27"
execution_order: 1
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-glm53-flash"
branch: "feat/glm-5-3-flash-vast-preset"
base_commit: "1bdc515bff48a4d9048dae7d06c6214e884265bc"
depends_on: "none"
created: 2026-08-26
updated: 2026-08-26
---

# PF-27-S01 — Curated GLM-5.3-Flash recipe and qualified endpoint

## Execution mandate

- Deliver: authenticated GLM-5.3-Flash vLLM presets for four connected H200s and two connected B300s, plus a reproducible B300 4–256 stream qualification.
- Excludes: unauthenticated ingress, provider-create bypass, spend-confirmation bypass, SGLang, NVFP4 artifacts, and changes to unrelated recipes.

## Plan linkage

- Plan: [GLM-5.3-Flash Vast GPU preset](../../../plans/active/glm-5-3-flash-vast-preset.md)
- Feature: `PF-27`
- Acceptance advanced: select the B300 preset, approve bounded spend, reach READY, run the mixed-context 4–256 stream sweep, and terminate without an orphan charge.

## Code boundaries

- Existing: `gpu-market/src/recipe.rs::RecipeCatalog`; `tui/src/chatwidget/gpu_menu.rs`
- Planned: B300 GLM-5.3 recipe constructor, mixed-traffic benchmark artifact, and catalog/snapshot regressions
- Tests: `gpu-market/src/recipe_tests.rs`; `tui/src/chatwidget/gpu_menu_tests.rs`; true-tmux slash dispatch; reviewed snapshots

## Preconditions

- [x] Read root, Rust, TUI, repository development, and true-TUI instructions.
- [x] Worktree, branch, and base commit match the active plan.
- [x] Official model/runtime artifacts plus Hopper and Blackwell constraints are pinned.

## Done

- [x] Sprint record created and linked to one active plan feature.
- [x] Added a verified vLLM recipe pinned to the GLM-5.3 checkpoint and dedicated serving-image digests.
- [x] Required TP4 H200/NVLink topology, Hopper-safe KV behavior, bounded capacity/deadlines, and scoped endpoint authentication.
- [x] Added catalog, immutable-manifest, hardware/launch, credential, snapshot, and true-tmux regressions.
- [x] Built and qualified the real `/gpu` selection/cancel flow without a provider search or create.

## Remaining

- [ ] Add the experimental native-FP8 TP2/B300 recipe with CUDA 13, FP8 KV cache, 131,072-token context, and 256-request scheduler bound.
- [ ] Add deterministic mixed-context benchmark inputs and result aggregation for 4, 8, 16, 32, 64, 128, and 256 closed-loop streams.
- [ ] Update and review catalog, manifest, launch, and true-tmux regressions.
- [ ] With exact user-approved limits, provision through `/gpu`, wait for READY, run the authenticated benchmark sweep, and terminate with provider confirmation.

## Verification

- [x] Fix: `cd codex-rs && just fix -p codex-gpu-market && just fix -p codex-tui`.
- [x] Format: `cd codex-rs && just fmt`; inspected the final diff.
- [x] Focused: `cd codex-rs && just test -p codex-gpu-market recipe` — 11 passed.
- [x] Integration: `cd codex-rs && just test -p codex-gpu-market` — 76 passed.
- [x] Snapshot: `cd codex-rs && just test -p codex-tui gpu_menu` — 9 passed; intended snapshot reviewed.
- [x] Governance: `python3 docs/plans/check.py && python3 docs/sprints/check.py`.
- [x] TUI: actual `/gpu` keys showed the preset and cancelled without a provider create.
- [ ] Live: bounded Vast B300 rental reaches READY, completes the secret-free 4–256 stream matrix, and terminates with provider confirmation.

## Exit evidence

- [x] Implementation commit `c0f2e02e4a` and changed paths recorded.
- [x] Final-tree commands and reviewed snapshot recorded under `qa/gpu-rentals/sprints/PF-27-S01/`.
- [ ] Live rental records endpoint readiness, per-stream/aggregate throughput and latency, and provider-confirmed termination without secrets.
- [ ] Ledgers reflect reality and completed record moves to `docs/sprints/archive/glm-5-3-flash-vast-preset/`.
