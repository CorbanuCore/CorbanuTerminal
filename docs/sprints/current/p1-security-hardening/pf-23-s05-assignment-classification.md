---
sprint_id: "PF-23-S05"
title: "Shell assignment provenance in protected-action classification"
status: in_progress
plan_file: "docs/plans/active/p1-security-hardening.md"
plan_feature: "PF-23"
execution_order: 78
owner: "Contributor integration coordinator"
parallel_lane: "broker"
write_scope: "codex-rs/core/src/security/tainted_action.rs, codex-rs/core/src/security/tainted_action/shell.rs, codex-rs/core/src/security/tainted_action_post_taint_tests.rs, codex-rs/core/src/security/tainted_action_reading_tests.rs, codex-rs/core/tests/suite/pf_23_s01.rs, codex-rs/core/tests/suite/pf_23_s05_assignment_tests.rs, docs/plans/active/p1-security-hardening.md, docs/sprints/current/p1-security-hardening/pf-23-s04-chroot-classification.md, docs/sprints/current/p1-security-hardening/pf-23-s05-assignment-classification.md, docs/sprints/current/p1-security-hardening/index.md, qa/security-levels/sprints/PF-23-S05/, qa/release/unreleased/notes/2133-assignment-provenance.md"
integration_gate: "Coordinator serializes classifier/lexer and plan edits; preserve current chroot, finder, brace and depth-limit checks; exact independent security review, isolated full classifier and targeted integration tests, blob/performance gates and all green CI before main integration. Functional execution/demo remain separately required and held for Opus recharge."
worktree: "/Volumes/CorbanuDrive/Corbanu/.codex-work/contrib-1112-2133"
branch: "maint/contrib-2133-1112"
base_commit: "02a3f70862ae62d555f08cff741cbec2f17bb894"
depends_on: "PF-23-S01"
created: 2026-11-12
updated: 2026-11-14
---

# PF-23-S05 — Assignment provenance

## Execution mandate

Travis authorized full review, revision and incorporation of @araupooo's #2133.
Under PF-23, prevent conditional/non-parent assignments from converting unknown
arguments into trusted values. Preserve conservative prompts and ordinary controls.
No new grants, level changes, credential handling or shell execution capability.

## Plan linkage

- Active [P1 plan](../../../plans/active/p1-security-hardening.md), feature PF-23.
- Product **P0 /security levels → Release rule**: “Moderate and Aggressive controls
  must be deterministic policy around the model, not prompt text.”
- PF-23-S01 is archived; this does not depend on unfinished S04 functional work.
- S04 returned to draft with evidence handoff; no invented parallel owner.

## Code boundaries

Literal front-matter paths. Reuse lexer and classifier; sibling tests only.
Review pipeline/function omissions and nested shell forms explicitly, not by
silently declaring complete shell analysis. Track any out-of-scope limitations.

## Preconditions

- [x] Active plan, owner authorization, archived PF-23-S01 and serialized handoff recorded.
- [x] Exact worktree/branch/base and literal write scope allocated before edits.

## Done

- [x] Owner authorization, exact allocation and prior evidence handoff recorded.
- [x] Contributor ancestry preserved; current main already included.
- [x] Four initial Astra High lenses; code-blind design pass5 frozen, repair pass6 granted.
- [x] Plan/sprint checkers passed before product edits; scoped Clippy/fmt passed.
- [x] [Original independent design](../../../../qa/security-levels/sprints/PF-23-S05/codeblind-design.md) frozen; execution held.

- [x] Reproduced and repaired actionable source-review findings with regression controls.
- [x] Covered conditional/group/subshell/background/substitution and identified inherited gaps.
- [x] Scoped fix/fmt, then 414 classifier and 16 mock-session tests passed on the combined tree.
- [x] Work-counter linearity checks passed; retained 5,397 cases/10,794 verdicts unchanged.
- [x] Exact-head source security approval and all 46 final CI checks passed/skipped.
- [x] Source merged as `23ac70e1f16561df98a4d198b5df9d531ca9b683`; @araupooo thanked and credited.
- [x] [Source evidence and retained limitations](../../../../qa/security-levels/sprints/PF-23-S05/source-integration.md) recorded.

- [x] 19 frozen cases run code-blind in Tart guests on `1dea8b3b03` (2026-11-14):
  [functional record](../../../../qa/security-levels/sprints/PF-23-S05/functional-20261114.md); 18 pass after
  evidence review and local-model replays, case 11 inconclusive; findings outside S05: #2166, #2167;
  [reel](../../../../qa/demos/index/pf23-s05-reel.md).

## Remaining

- [ ] Independent functional evidence review, true-TUI/demo and applicable live-repository evidence.
- [ ] Complete remaining functional evidence; archive only after all gates pass.

## Verification

- [x] `just test -p codex-core --lib --test-threads 4 -E 'test(security::tainted_action)' --locked --offline`.
- [x] New assignment approval-flow integration tests via isolated `just test` (16 passed).
- [x] Changed files below 512,000 bytes; diff/key-pattern scan, scoped Clippy and platform CI passed.
- [x] Permissive control passed; no runtime protected operation in unit fixtures.

- [ ] Frozen functional cases executed with enforced isolation and independent evidence review.

## Exit evidence

- [x] Exact reviewed source tree, raw failed attempts and fresh replay logs recorded.
- [x] Separate implementation, functional, demo and release statuses recorded.
- [x] No mock/unit pass described as independent acceptance or release.

- [ ] Independent sealed functional execution, evidence review, true-TUI/live-repository and demo artifacts.

## Review budget

Initial four code lenses each count once. Fifth pass reserved for independent
code-blind design. Further repair/evidence passes need a recorded coordinator
grant before dispatch; preserve prior #1974 history, never reset usage.
