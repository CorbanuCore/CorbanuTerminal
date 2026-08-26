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

- Deliver: a preconfigured authenticated GLM-5.3-Flash vLLM preset for four connected H200s and a qualified Vast endpoint.
- Excludes: unauthenticated ingress, direct provider scripts, spend-confirmation bypass, SGLang, and changes to other recipes.

## Plan linkage

- Plan: [GLM-5.3-Flash Vast GPU preset](../../../plans/active/glm-5-3-flash-vast-preset.md)
- Feature: `PF-27`
- Acceptance advanced: select one preset, approve bounded spend, reach READY, call the endpoint, and terminate without an orphan charge.

## Code boundaries

- Existing: `gpu-market/src/recipe.rs::RecipeCatalog`; `tui/src/chatwidget/gpu_menu.rs`
- Planned: GLM-5.3 recipe constructor and catalog/snapshot regressions
- Tests: `gpu-market/src/recipe_tests.rs`; `tui/src/chatwidget/gpu_menu_tests.rs`; reviewed snapshots

## Preconditions

- [ ] Read root, Rust, TUI, repository development, and true-TUI instructions.
- [ ] Worktree, branch, and base commit match the active plan.
- [ ] Official model/runtime artifacts and Hopper constraints are pinned.

## Done

- [x] Sprint record created and linked to one active plan feature.

## Remaining

- [ ] Add a verified vLLM recipe pinned to the GLM-5.3 checkpoint and dedicated serving-image digests.
- [ ] Require TP4 H200/NVLink topology, BF16 KV behavior, bounded context/concurrency, capacity, deadlines, and scoped endpoint authentication.
- [ ] Add generalized regressions for catalog order, immutable manifest, TP/GPU sizing, Hopper launch flags, and absence of embedded credentials.
- [ ] Update and review the `/gpu` catalog snapshot.
- [ ] Build the final candidate and exercise select/cancel through the true tmux TUI harness.
- [ ] With exact user-approved limits, provision through `/gpu`, wait for READY, call the authenticated endpoint, and terminate with provider confirmation.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-gpu-market && just fix -p codex-tui`.
- [ ] Format: `cd codex-rs && just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-gpu-market recipe`.
- [ ] Integration: `cd codex-rs && just test -p codex-gpu-market`.
- [ ] Snapshot: `cd codex-rs && just test -p codex-tui gpu_menu`; review intended snapshot changes.
- [ ] Governance: `python3 docs/plans/check.py && python3 docs/sprints/check.py`.
- [ ] TUI: actual `/gpu` keys show the preset and cancel without a provider create.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree commands and reviewed snapshot recorded under `qa/gpu-rentals/sprints/PF-27-S01/`.
- [ ] Live rental records endpoint readiness and provider-confirmed termination without secrets.
- [ ] Ledgers reflect reality and completed record moves to `docs/sprints/archive/glm-5-3-flash-vast-preset/`.
