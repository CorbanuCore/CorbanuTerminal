---
sprint_id: "PF-23-S04"
title: "Chroot protected-action classification"
status: draft
plan_file: "docs/plans/active/p1-security-hardening.md"
plan_feature: "PF-23"
execution_order: 19
owner: "Contributor integration coordinator"
parallel_lane: "broker"
write_scope: "codex-rs/core/src/security/tainted_action.rs, codex-rs/core/src/security/tainted_action/indirect.rs, codex-rs/core/src/security/tainted_action_chroot_tests.rs, codex-rs/core/src/security/tainted_action_tests.rs, docs/plans/active/p1-security-hardening.md, docs/sprints/current/p1-security-hardening/pf-23-s04-chroot-classification.md, docs/sprints/current/p1-security-hardening/index.md, qa/security-levels/sprints/PF-23-S04/, qa/release/unreleased/notes/1837-chroot-classification.md"
integration_gate: "Coordinator serializes #1837 against #1393 and #1055; preserve finder/sudo, brace readings and depth cap; exact-head Astra High security review, complete green CI, final combined classifier tests and blob-size check. Functional qualification and demo remain explicit unfinished gates, not inferred from unit tests."
worktree: "/Volumes/CorbanuDrive/Corbanu/.codex-work/contrib-1111-1837"
branch: "maint/contrib-1837-1111"
base_commit: "2cfd034860f792d0c3605b40437a3e21e29b361e"
depends_on: "PF-23-S01"
created: 2026-11-11
updated: 2026-11-11
---

# PF-23-S04 — Chroot protected-action classification

## Serialized handoff — 2026-11-12
Return this reservation to draft while the same coordinator implements PF-23-S05.
#1837 merged at493f370ccf328840c4e052ceeffcda4e8dd132e9:59green/skipped checks,
classifier407passed, test blob479784bytes. Eleven frozen cases/demo await Opus.
Evidence and review usage remain; source integration is not completion.

## Execution mandate

Deliver the retained #1837 contribution by @citadelculture after main reconciliation.
Travis approved the contributor assessment and asked to revise and incorporate it;
#2106 is the integration ledger. This is a product initiative because protected-action
classification changes, not a routine lint repair.

Excludes new grants, security-level changes, filesystem containment promises, arbitrary
finder/exec launcher coverage, authentication recovery and other contributor features.

## Plan linkage

- Plan: [P1 security hardening](../../../plans/active/p1-security-hardening.md), PF-23.
- Product specification: **P0 `/security` levels → Release rule**:
  “Moderate and Aggressive controls must be deterministic policy around the model,
  not prompt text.” The archived PF-23-S01 protected-action boundary is retained.
- Owner allocation: coordinator reuses the free broker lane for this serialized
  security repair. No other worker owns this checkout. #1055 execution has finished;
  #1393 and this change require a combined-tree test before integration.

## Code boundaries

- Existing classifier in `tainted_action.rs`; indirect command discovery in
  `tainted_action/indirect.rs`.
- Contributor chroot test file plus its existing test-module registration.
- Plan/sprint bookkeeping and one unreleased note; no other code paths.

## Preconditions

- [x] P1 plan active; owner approved this contribution's revision and incorporation.
- [x] PF-23-S01 completed and archived.
- [x] Exact coordinates recorded in plan and sprint.
- [x] Broker lane allocated; shared classifier merge boundary serialized.

## Done

- [x] Bounded mandate and allocation recorded before implementation.
- [x] Preserved contributor ancestry while reconciling finder/sudo and brace checks.
- [x] Frozen independent [code-blind design](../../../../qa/security-levels/sprints/PF-23-S04/codeblind-design.md) before functional execution; all eleven cases remain unexecuted.
- [x] Initial integrated classifier: 404/404 passed; independent review nevertheless found inherited script-input loss. New regression reproduced it; repair and final qualification remain in progress.
- [x] Closed inherited script-input finding with red/green tests and independent repair review; pre-reconciliation classifier406/406 passed. Recorded corpus:5397 cases/10794 verdicts,62 new asks,0 quieter outcomes,0 kind changes or panics. Combined current-main replay remains required below.

## Remaining

- [ ] Format and run final-tree focused chroot and full classifier tests.
- [ ] Verify no verdict becomes quieter on the recorded comparison corpus.
- [ ] Independent exact-head security review and all CI complete green.
- [ ] Execute applicable isolated, code-blind TUI acceptance and record demo reel.
- [ ] Record integrated commit, user-visible limits and actual evidence; archive only when all gates pass.

## Verification

- [ ] `just test -p codex-core --lib -E 'test(security::tainted_action)' --locked --offline`.
- [ ] Final merged blob below 512000 bytes and overlap changes requalified.
- [ ] True-TUI tests use synthetic state, no host chroot execution or live credentials.
- [ ] Functional evidence independently reviewed; no mock/unit run represented as TUI acceptance.
- [ ] Release/live-repository and human acceptance remain separate gates.

## Exit evidence

- [ ] Contributor credit names @citadelculture; Task Node provenance factual, no invented task id.
- [ ] Done/Remaining reflects final state; completed record moved to archive.

## Review budget

Prior PR rounds and the comparative assessment remain in their original ledger.
Coordinator authorizes two additional bounded review passes for this integration:
one fresh code-blind test design and one exact-head code/security review after
main reconciliation. No self-authorized retries or erased findings. A later
functional evidence review needs its own recorded allocation.

[Prior repair review and red/green evidence](https://github.com/CorbanuCore/CorbanuTerminal/blob/493f370ccf328840c4e052ceeffcda4e8dd132e9/docs/sprints/current/p1-security-hardening/pf-23-s04-chroot-classification.md#review-budget)
remain preserved; no functional/TUI pass is claimed.
