---
sprint_id: "PF-35-S03"
title: "Calibrated detector and ingress enforcement"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-35"
execution_order: 50
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-35-S02, PF-34-S01, PF-30-S03, PF-23-S01"
created: 2026-08-28
updated: 2026-08-28
---

# PF-35-S03 — Calibrated detector and ingress enforcement

## Execution mandate

- Deliver: Protected-mode ingestion is screened and fails closed, while detector misses do not defeat deterministic authority.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-35).
- Feature: `PF-35`.
- Product citation: **Non-negotiable controls** — “Classify instruction intent and provenance before external content can influence tools or financial actions.”
- Acceptance advanced: Protected-mode ingestion is screened and fails closed, while detector misses do not defeat deterministic authority.
- Sources and archive disposition: [PF-35 reconciliation](../../../plans/security-source-reconciliation.md#pf-35).

## Code boundaries

- OpenClaw adoption reference: [OC-4](../../../plans/openclaw-source-review-2026-08-28.md#oc-4), [OC-5](../../../plans/openclaw-source-review-2026-08-28.md#oc-5), [OC-10](../../../plans/openclaw-source-review-2026-08-28.md#oc-10) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: PF-34 sanitizer; PF-35 evaluator.
- Planned: codex-rs/content-security/src/{screen,calibration}.rs; qa/security-levels/classifier/qualification.json.
- Tests: planned colocated Rust test modules prefixed `pf_35_s03`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] All dependencies in front matter are completed and archived; plan remains active.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Integrate the actual PF-34 sanitizer, PF-30 post-taint checks and PF-23 Moderate dispatcher. Force allow/error/timeout on hostile source/memory/child fixtures and prove no unapproved effect; unavailable screening pauses ingestion.

- [ ] Force classifier allow/error/timeout on hostile wrappers, metadata, cache and child/memory content; required screening failure must pause/quarantine, while an allow verdict cannot erase provenance or authorize an effect.

- [ ] Calibrate Moderate and stricter Aggressive thresholds on development data only; freeze the exact artifact, tokenizer and signed threshold identities before any blind evaluation.
- [ ] Have Travis run the untouched blind evaluator under independent custody and return only the allowlisted aggregate plus detached signature envelope, bound to the development report, artifact and threshold identities; no blind rows, labels or row-level errors enter training, review or Git.
- [ ] Screen every external/tool/search/child segment and reassembled multi-chunk content before model ingestion; prevent streaming prefixes from reaching the model before a decision.
- [ ] Route allow, bounded sanitize-and-rescan, quarantine and reject outcomes; retries cannot bypass limits, and missing/unavailable classifier pauses external ingestion in both protected modes.
- [ ] Shadow evaluation is an isolated test harness, not a shipping protected mode or silent change to Permissive; no threshold adjustment from the blind holdout.
- [ ] Force benign decisions on hostile fixtures to prove independent broker/policy denial; failed quality/resource targets leave qualification incomplete.
- [ ] Add named `pf_35_s03` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-content-security pf_35_s03`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-35-S03/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
