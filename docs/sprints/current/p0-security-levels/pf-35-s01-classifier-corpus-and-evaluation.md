---
sprint_id: "PF-35-S01"
title: "Classifier corpus and leakage-free evaluation"
status: draft
external_qualification_state: "Independent external campaign pending; engineering reservation released, not completed"
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
allocation_commit: "e0c23fe95165636d621dae8c16a5366c4f7250ac"
depends_on: "PF-34-S04"
created: 2026-08-28
updated: 2026-09-04
---

# PF-35-S01 — Classifier corpus and leakage-free evaluation

## Execution mandate

On 2026-09-04 the user removed PF-35 from the engineering main path.
This record releases its engineering reservation to draft under the existing
sprint process; the independent external corpus campaign is not stopped.
Previously merged foundation and all remaining blind/signature/N100 gates
are preserved. Reallocate before repository implementation resumes; PF-35
consumers still require honest completion and archival. Handoff:
`qa/security-levels/planning/parallel-handoffs-2026-09-04-round-5/README.md`.

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

- [x] All dependencies in front matter are completed and archived; plan remains active.
- [x] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [x] Confirm source pins and declared crate/module paths.
- [x] Reconciled the plan-mandated immutable dispatch base (`9d08b15f…`) with the later coordination/allocation commit (`e0c23fe95…`) from which this worktree was created; both coordinates are explicit.
- [ ] Obtain external RTX, private-custodian, signing and qualifying-N100 availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.
- [x] Added public, commercially usable source metadata with immutable hashes, attribution, customer/protected-data exclusion, synthetic-campaign metadata, and human-owned blind custody.
- [x] Added grouped train/development/blind targets and mandatory unseen-source/language/topic/position/adaptive holdouts, with no corpus or blind records committed.
- [x] Added exact-allowlist, bounded-manifest development and aggregate-only blind evaluators. Ground truth drives metrics; Wilson confidence bounds, a conservative hard-negative difference bound, sample floors, and the approximately 150k blind-total tolerance drive statistical gates; manifest, model, artifact, and threshold identities are bound.
- [x] Pinned canonical group-tuple fingerprinting, emitted the development fingerprint in the manifest-bound report, deduplicated same-label sibling records for the group set, and rejected contradictory-label tuples.
- [x] Bound each snapshotted regular-file predictions/aggregate/development-report input by evaluator ID/report-schema version, portable path, kind, byte count and SHA-256; blind evaluation derives the expected fingerprint from the prior report, while artifact/threshold declarations remain explicitly unverified.
- [x] Kept N100 latency/RSS, model size, signed release identity, private custody, and a custodian-side train/development/blind group-tuple overlap audit fail closed as external requirements; unequal fingerprint declarations alone do not prove disjointness.
- [x] Added evaluator regression coverage plus a meaningful PF-35 Rust regression proving altered model-artifact and threshold digests cannot authorize release.
- [x] Integration owner wired `scripts/test_security_classifier_eval.py` and a shipped-CLI smoke into recurring CI without expanding this lane's literal write scope.

## Remaining

- [ ] Acquire, license-review, group and freeze the approximately 250k/25k/150k train/development/blind records on the external RTX/custodian lanes; no such run or private corpus is claimed here.
- [ ] Obtain the custodian's aggregate blind report and signed train/development/blind group-tuple overlap audit without disclosing record-level blind output; unequal fingerprints alone are declarations and do not prove disjointness.
- [ ] Measure the signed production artifact on the qualifying Intel N100/16 GiB/x86-64 Linux floor at 2,048 tokens; prove model ≤300 MiB, p95 ≤50 ms and peak RSS ≤512 MiB.

## Verification

- [x] Run `cd codex-rs && just fix -p codex-content-security`, then `just fmt`; inspect the final diff.
- [x] Focused: `cd codex-rs && just test -p codex-content-security pf_35_s01`; one named test ran and passed.
- [x] Integration: `cd codex-rs && just test -p codex-content-security`; 22/22 tests passed. No Cargo/Bazel/lock change was required.
- [x] Supporting TUI smoke: exact candidate Corbanu binary launched in a private TMUX server with trace logging; `/status` and clean `/exit` were verified using separate text/Enter sends.
- [x] Record commands, outcomes and safe artifact digests under `qa/security-levels/sprints/PF-35-S01/`; no production credentials, blind data, weights or funds.
- [ ] Run final external corpus, custodian, signed-artifact and qualifying-N100 evidence gates; the current preparation candidate cannot prove them.

## Exit evidence

- [x] Implementation commits and deterministic final-tree outputs are recorded under `qa/security-levels/sprints/PF-35-S01/`; external qualification outputs remain pending.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
