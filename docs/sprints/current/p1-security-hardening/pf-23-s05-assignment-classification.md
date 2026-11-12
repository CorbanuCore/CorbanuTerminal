---
sprint_id: "PF-23-S05"
title: "Shell assignment provenance in protected-action classification"
status: in_progress
plan_file: "docs/plans/active/p1-security-hardening.md"
plan_feature: "PF-23"
execution_order: 78
owner: "Contributor integration coordinator"
parallel_lane: "broker"
write_scope: "codex-rs/core/src/security/tainted_action.rs, codex-rs/core/src/security/tainted_action/shell.rs, codex-rs/core/src/security/tainted_action_post_taint_tests.rs, codex-rs/core/tests/suite/security_post_taint.rs, docs/plans/active/p1-security-hardening.md, docs/sprints/current/p1-security-hardening/pf-23-s04-chroot-classification.md, docs/sprints/current/p1-security-hardening/pf-23-s05-assignment-classification.md, docs/sprints/current/p1-security-hardening/index.md, qa/security-levels/sprints/PF-23-S05/, qa/release/unreleased/notes/2133-assignment-provenance.md"
integration_gate: "Coordinator serializes classifier/lexer and plan edits; preserve current chroot, finder, brace and depth-limit checks; exact independent security review, isolated full classifier and targeted integration tests, blob/performance gates and all green CI before main integration. Functional execution/demo remain separately required and held for Opus recharge."
worktree: "/Volumes/CorbanuDrive/Corbanu/.codex-work/contrib-1112-2133"
branch: "maint/contrib-2133-1112"
base_commit: "493f370ccf328840c4e052ceeffcda4e8dd132e9"
depends_on: "PF-23-S01"
created: 2026-11-12
updated: 2026-11-12
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
- [x] Four initial Astra High review lenses authorized; no execution substitutes.

## Remaining

- [ ] Run plan/sprint checkers before product edits.
- [ ] Reproduce and repair all actionable review findings with regression controls.
- [ ] Qualify conditional/group/subshell/background/substitution and inherited gaps.
- [ ] Format, then isolated focused/full classifier and affected integration tests.
- [ ] Verify bounded/linear work and no quieter outcomes on the retained corpus.
- [ ] Independent exact-head security approval and all CI checks green.
- [ ] Freeze independent code-blind design; execute in enforced isolation after Opus recharge.
- [ ] Independent functional evidence review, true-TUI/demo and applicable live-repository evidence.
- [ ] Record merge, limits and @araupooo credit; archive only after all gates pass.

## Verification

- [ ] `just test -p codex-core --lib -E 'test(security::tainted_action)' --locked --offline`.
- [ ] Existing/new security integration tests via isolated `just test`.
- [ ] Changed files below512000bytes; diff/key scan, scoped Clippy and platform CI.
- [ ] Permissive behavior unchanged; no runtime protected operation in unit fixtures.

## Exit evidence

- [ ] Exact reviewed/tested head, raw failed attempts and fresh replay logs.
- [ ] Separate implementation, functional, demo and release statuses.
- [ ] No mock/unit pass described as independent acceptance or release.

## Review budget

Initial four code lenses each count once. Fifth pass reserved for independent
code-blind design. Further repair/evidence passes need a recorded coordinator
grant before dispatch; preserve prior #1974 history, never reset usage.
