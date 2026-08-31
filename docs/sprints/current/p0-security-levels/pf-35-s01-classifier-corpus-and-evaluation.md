---
sprint_id: "PF-35-S01"
title: "Classifier corpus and leakage-free evaluation"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-35"
execution_order: 20
owner: "Codex — PF-35 external qualification lane"
parallel_lane: "classifier-corpus"
write_scope: "codex-rs/content-security/, scripts/security-classifier-eval, scripts/security_classifier_eval.py, scripts/test_security_classifier_eval.py, qa/security-levels/classifier/, qa/security-levels/sprints/PF-35-S01/, docs/sprints/current/p0-security-levels/pf-35-s01-classifier-corpus-and-evaluation.md"
integration_gate: "The Codex ingress/classifier integration lane receives PF-35-S01, audits the literal scope and private-blind-data exclusion, serializes any workspace/build registration, reruns the full content-security and evaluator suites plus governance checks, preserves measured blockers honestly, and archives the sprint only when every required corpus/evaluation claim has final-tree evidence."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-35-s01-external-qualification"
branch: "feat/pf-35-s01-external-qualification-20260830"
base_commit: "2bcaf8d0b70f039f48165d0e4a4f291101574a41"
depends_on: "PF-34-S04"
created: 2026-08-28
updated: 2026-08-31
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

- [x] All dependencies in front matter are completed and archived; plan remains active.
- [x] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [x] Confirm source pins and declared crate/module paths.
- [x] Reallocated from current `origin/main` (`2bcaf8d0…`) into the distinct recorded CorbanuDrive worktree and branch.
- [x] Confirmed Travis as blind custodian plus RTX/signing operator and Alex as the authorized N100 operator; credentials, keys and blind rows remain outside this lane.

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
- [x] Pinned and license-reviewed the exact Qwen 3.8 generator, tokenizer, vLLM environment, launch recipe, prompt schemas, seeds and sampling; the same-host benchmark and 128/128 structured bakeoff qualify it for the pilot.
- [x] Generated 11,767 hash-bound provisional pilot records from 12,000 candidates on the RTX host, with privacy/dedup rejections and disjoint human/Opus review queues; no record material entered Git.
- [x] Human semantic review rejected that first pilot for systemic attack-class didactic leakage; preserve it as quarantined failure evidence and prohibit adjudication, training, import, or scale-up from its records.
- [x] Replaced free Cartesian generation with a label/scope-compatible matrix, attacker-authentic attack contracts, inert hard-negative contracts, short/medium/long length buckets, deterministic semantic-leakage rejection, and hash-bound quarantine enforcement.
- [x] Ran three versioned 1,000-candidate canaries: quarantined the first two as semantic failures, then produced `canary-r3`, which retained 829 records with zero deterministic compatibility, accepted attack self-description, fragment-mechanism, or length-ceiling matches. The 82-record human packet and disjoint 32-record Opus audit remain pending.

## Remaining

- [ ] Complete and validate all 82 `canary-r3` human decisions plus the disjoint 32-record Opus audit, adjudicate the canary, and obtain integration-owner acceptance of its content quality.
- [ ] Generate and adjudicate a replacement pilot with at least 10,000 final accepted records, including complete human decisions for disagreement/uncertainty/suspicious rows and disjoint 1% human/Opus high-confidence audits; obtain integration-owner acceptance before scaling.
- [ ] Group and freeze approximately 250k training and 25k development records on the RTX lane; preserve hashes and aggregate evidence without committing record data or weights.
- [ ] Have Travis independently freeze/encrypt/sign the approximately 150k blind manifest and return a signed train/development/blind canonical group-tuple overlap audit without exposing blind rows, labels or row-level errors.

## Verification

- [x] Run `cd codex-rs && just fix -p codex-content-security`, then `just fmt`; inspect the final diff.
- [x] Focused: `cd codex-rs && just test -p codex-content-security pf_35_s01`; one named test ran and passed.
- [x] Integration: `cd codex-rs && just test -p codex-content-security`; 22/22 tests passed. No Cargo/Bazel/lock change was required.
- [x] Supporting TUI smoke: exact candidate Corbanu binary launched in a private TMUX server with trace logging; `/status` and clean `/exit` were verified using separate text/Enter sends.
- [x] Record commands, outcomes and safe artifact digests under `qa/security-levels/sprints/PF-35-S01/`; no production credentials, blind data, weights or funds.
- [ ] Run final S01 corpus/custody evidence gates; signed-artifact/N100 proof remains S02 and calibrated aggregate blind evaluation remains S03.

## Exit evidence

- [x] Implementation commits and deterministic final-tree outputs are recorded under `qa/security-levels/sprints/PF-35-S01/`; external qualification outputs remain pending.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
