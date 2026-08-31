---
sprint_id: "PF-35-S02"
title: "Reproducible local CPU detector artifact"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-35"
execution_order: 21
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-35-S01"
created: 2026-08-28
updated: 2026-08-28
---

# PF-35-S02 — Reproducible local CPU detector artifact

## Execution mandate

- Deliver: The local detector runs offline within an explicit resource envelope and identifies its exact artifact.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-35).
- Feature: `PF-35`.
- Product citation: **Non-negotiable controls** — “Classify instruction intent and provenance before external content can influence tools or financial actions.”
- Acceptance advanced: The local detector runs offline within an explicit resource envelope and identifies its exact artifact.
- Sources and archive disposition: [PF-35 reconciliation](../../../plans/security-source-reconciliation.md#pf-35).

## Code boundaries

- OpenClaw adoption reference: [OC-4](../../../plans/openclaw-source-review-2026-08-28.md#oc-4) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: PF-35 adapter and corpus manifests.
- Planned: codex-rs/content-security/src/classifier.rs; tools/security-classifier/{train,export,manifest}.py.
- Tests: planned colocated Rust test modules prefixed `pf_35_s02`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-35-S01 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Record CPU baseline, artifact size/RSS, reproducible build and license/security pins early; do not interpret fixture quality as final live-ingress acceptance. Reserve blind holdouts for the independent evaluator.

- [ ] Record and verify the separately selected CPU model/license/artifact/runtime; OpenClaw's regex signals are not that model and supply no quality or resource qualification. Keep deterministic policy independent of the detector.

- [ ] Compare untuned and small fine-tuned candidates using the frozen training/validation split; retain model selection rationale, licenses and reproducible seeds/config.
- [ ] Package selected model/tokenizer and ONNX-compatible CPU runtime with pinned hashes, dependency inventory, signature and deterministic artifact verification.
- [ ] Have Alex measure the complete 2,048-token tokenization-plus-inference path on Intel N100/16 GiB/x86-64 Linux; require model ≤300 MiB, p95 ≤50 ms and peak RSS ≤512 MiB, preserving failed evidence without retargeting the floor.
- [ ] Enforce memory/latency/segment limits and offline operation; no external telemetry, protected-data training, hidden downloads or runtime model replacement.
- [ ] Return typed unavailable on missing/corrupt artifact, unsupported CPU, timeout and resource exhaustion; never silently return benign.
- [ ] Test repeatability, signature/hash failure, tokenizer/version mismatch, truncated/multi-chunk inputs and shutdown cleanup.
- [ ] Add named `pf_35_s02` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-content-security pf_35_s02`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-35-S02/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
