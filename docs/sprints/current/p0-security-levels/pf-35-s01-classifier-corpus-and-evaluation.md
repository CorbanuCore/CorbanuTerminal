---
sprint_id: "PF-35-S01"
title: "Classifier corpus and leakage-free evaluation"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-35"
execution_order: 20
owner: "Raman — classifier corpus lane"
parallel_lane: "classifier-corpus"
write_scope: "codex-rs/content-security/, scripts/security-classifier-eval, scripts/security_classifier_eval.py, scripts/test_security_classifier_eval.py, qa/security-levels/classifier/, qa/security-levels/sprints/PF-35-S01/, docs/sprints/current/p0-security-levels/pf-35-s01-classifier-corpus-and-evaluation.md"
integration_gate: "The Codex ingress/classifier integration lane receives PF-35-S01, audits the literal scope and private-blind-data exclusion, serializes any workspace/build registration, reruns the full content-security and evaluator suites plus governance checks, preserves measured blockers honestly, and archives the sprint only when every required corpus/evaluation claim has final-tree evidence."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-classifier-corpus"
branch: "feat/p0-security-classifier-corpus"
base_commit: "9d08b15fa94676c1383ee1605b77e7cc7218dcc4"
depends_on: "PF-34-S04"
created: 2026-08-28
updated: 2026-08-30
---

# PF-35-S01 — Classifier corpus and leakage-free evaluation

## Execution mandate

- Deliver: Detector quality is measured on reproducible, licensed, leakage-free blind evidence, not handpicked prompts.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-35).
- Feature: `PF-35`.
- Product citation: **Non-negotiable controls** — “Classify instruction intent and provenance before external content can influence tools or financial actions.”
- Acceptance advanced: Detector quality is measured on reproducible, licensed, leakage-free blind evidence, not handpicked prompts.
- Sources and archive disposition: [PF-35 reconciliation](../../../plans/security-source-reconciliation.md#pf-35).

## Code boundaries

- OpenClaw adoption reference: [OC-4](../../../plans/openclaw-source-review-2026-08-28.md#oc-4) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: PF-34-S04 frozen segment/verdict contract; docs/plans/active/p0-security-levels.md qualification targets.
- Planned: qa/security-levels/classifier/{corpus-manifest.json,split-manifest.json}; scripts/security-classifier-eval.
- Tests: planned colocated Rust test modules prefixed `pf_35_s01`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] All dependencies in front matter are completed and archived; plan remains active.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Build against the completed PF-34-S04 segment/verdict contract and frozen fixtures, without waiting for live browser/sanitizer integration. Separate licensed acquisition and blind evaluator ownership from training; publish measured CPU feasibility before costly tuning.

- [ ] Use upstream heuristic/wrapper adversarial cases only as labeled regression seeds, not a training/test leak or a validated detector. Add benign lookalikes and independently held-out attacks; record no upstream classifier-quality evidence was established here.

- [ ] Consume PF-34-S04 versioned allow/suspicious/hostile/unavailable results with model/version/threshold IDs and safe diagnostics; the evaluator cannot redefine trust or authorize actions.
- [ ] Inventory licensed source data, allowed use, hashes and synthetic finance/repository/web/tool/child attacks; exclude customer secrets and identifying financial records.
- [ ] Group splits by original source, template, attack family and semantic near-duplicates; retain independent unseen-source/language/topic/position/adaptive holdouts.
- [ ] Include benign security research, legitimate human trading instructions and trigger-token hard negatives; freeze evaluator-owned blind data before training.
- [ ] Implement metrics with sample counts/confidence intervals, family recall, FPR, CPU latency/RSS and artifact identity; pin the weakest supported CPU before qualification.
- [ ] Add named `pf_35_s01` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-content-security pf_35_s01`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-35-S01/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
