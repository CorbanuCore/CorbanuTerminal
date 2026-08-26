---
sprint_id: "PF-27-S01"
title: "Curated GLM-5.3-Flash recipe and qualified endpoint"
status: completed
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
- Acceptance advanced: the B300 preset reached READY, completed the mixed-context sweep, and terminated with provider confirmation inside approved limits.

## Code boundaries

- Recipe and launch lifecycle: `codex-rs/gpu-market/src/glm_recipes.rs`; `codex-rs/gpu-market/src/recipe_tests.rs`
- Catalog/TUI evidence: `codex-rs/tui/src/chatwidget/gpu_menu_tests.rs`; reviewed snapshots; forced true-tmux slash dispatch
- Benchmark and results: `qa/gpu-rentals/benchmarks/glm53-b300/`

## Preconditions

- [x] Read root, Rust, TUI, repository development, and true-TUI instructions.
- [x] Worktree, branch, and base commit matched the active plan.
- [x] Official model/runtime artifacts plus Hopper and Blackwell constraints were pinned.
- [x] Product owner authorized $16/hour, $125 total, and 480 minutes before the billable request.

## Done

- [x] Added immutable authenticated vLLM TP4/H200 and TP2/B300 presets with hardware, topology, context, and spend/lifecycle bounds.
- [x] Corrected unsupported runtime flags, selected text-only checkpoint initialization, and supervised vLLM through authenticated phase publication and shutdown.
- [x] Added catalog, manifest, launch, credential, snapshot, stream-schema, and forced true-tmux regressions.
- [x] Added the deterministic mixed-context 4–256 stream harness and content-free incremental artifacts.
- [x] Rented exact 2×B300/NVLink capacity through `/gpu`, reached READY, and passed the full authenticated endpoint contract.
- [x] Completed 1,016/1,016 requests with zero failures and recorded throughput/latency at every level.
- [x] Terminated through `/gpu`; Corbanu and Vast confirmed absence, the tunnel exited, and the rental token was removed.

## Remaining

- None. Product-owner review of the completed initiative remains at the plan level and does not leave sprint implementation work open.

## Verification

- [x] `cd codex-rs && just fix -p codex-gpu-market && just fix -p codex-tui && just fmt` passed.
- [x] `cd codex-rs && just test -p codex-gpu-market` passed 78 tests.
- [x] `cd codex-rs && just test -p codex-tui gpu_menu` passed 9 tests with the reviewed `r4` snapshot.
- [x] Forced true-tmux `/gpu` catalog/cancel test passed with zero retries.
- [x] Benchmark unit tests and seven-level workload validation passed.
- [x] Live 4–256 sweep passed with zero failures; 256 streams produced 2,662.88 aggregate and 10.40 per-stream output tok/s.
- [x] Governance checks passed after archival.

## Exit evidence

- [x] Implementation and correction commits are recorded in [PF-27-S01 evidence](../../../../qa/gpu-rentals/sprints/PF-27-S01/evidence.md).
- [x] Content-free benchmark summaries and per-request timing evidence are committed under `qa/gpu-rentals/benchmarks/glm53-b300/results/20260826-vast-48809614/`.
- [x] Exact limits, offer/resource, READY contract, cost, and provider-confirmed cleanup are recorded without secrets.
- [x] Ledgers reflect reality and this completed record is archived.
