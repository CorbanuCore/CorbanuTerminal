---
sprint_id: "PF-79-S01"
title: "Desktop beta channel and public manual-test contract"
status: draft
plan_file: "docs/plans/active/initiative-delivery-control.md"
plan_feature: "PF-79"
execution_order: 2
owner: "UNALLOCATED — Task Node integration owner assigns before ready"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-80-S01"
created: 2026-09-11
updated: 2026-09-11
---

# PF-79-S01 — Desktop beta channel and public manual-test contract

## Execution mandate

- Deliver: one isolated Desktop beta candidate/channel and versioned test-card/evidence contract.
- Excludes: public assignments, recruitment, a new Desktop app, reward commitments and stable release.
- Estimate: 3–5 working days after prerequisites; stop/re-slice if existing packaging is missing.

## Plan linkage

- [Workstream 3](../../../plans/active/initiative-delivery-control.md), feature PF-79.
- [Authoritative detailed contract](../../../plans/tasknode-beta-program.md), S01 and public test card v1.
- Product citation: **Internal delivery control — TO BUILD**, “ongoing public manual-testing beta program”.

## Code boundaries

- Existing: `codex-rs/cli/src/tasknode_cmd.rs`; `codex-rs/features/src/lib.rs`; `scripts/initiative_control/` read contracts only in S01.
- Planned: `docs/research/beta-program/contract.md`, candidate schema and fixtures under `qa/beta-program/`.
- Desktop package/update/signing paths: unresolved; enumerate exact repository-relative paths before ready, never substitute CLI artifacts.
- Native public path reference: Task Node `scripts/bm/writes.mjs::taskCreate` and `server/hive-routes.js`; do not change the remote board manager in this sprint.

## Preconditions

- [ ] PF-80-S01 completed/archived, including explicit historical PF-76 event/source migration; main's unrelated PF-76 remains unchanged.
- [ ] Travis confirms Desktop repo, branch name, pilot OS, release owner and existing packaging paths.
- [ ] Resolve supported public task/card visibility, board authority, budget semantics and safe evidence destination from pinned current source.
- [ ] Allocate exact owner, worktree/base, disjoint paths and receiving integration tests; recheck PF-79 ID.

## Done

- [x] Created two draft sprints inside workstream 3, not a fourth initiative or running worker.
- [x] Recorded candidate, public visibility, privacy, rollback and human-approval requirements.

## Remaining

- [ ] Pin approved main base and create only the approved beta branch; review channel config separately from stable.
- [ ] Configure distinct app/data/auth identity and prerelease update feed using existing packaging; implement native default-OFF beta boundary.
- [ ] Produce signed immutable manifest: SHA/version/digest/platform/flags and install/update/rollback instructions.
- [ ] Implement test-card/candidate schema validation and valid/invalid synthetic fixtures; reject missing build/expected-results/cleanup/publication fields.
- [ ] Freeze three public-safe cards: install/update/rollback, expired-auth recovery, and flag-OFF/cancel/restart; each fits 10–20 minutes.
- [ ] Verify full-card hosting, private escalation/consent/retention rules and public projection truncation behavior; no public publish yet.
- [ ] Record narrow public publisher contract and explicit supported zero-reward or approved-budget gate; no credentials or rewards provisioned.
- [ ] Hand off manifest/schema version, exact candidate and source/publication decisions to S02.

## Verification

- [ ] Run schema/fixture checks including missing candidate, secret-like attachment, unknown OS and unsafe cleanup.
- [ ] Record exact Desktop packaging/channel test commands after path allocation; verify beta cannot update stable or read its auth/config.
- [ ] Real Desktop GUI install/update/failure/rollback/uninstall plus version checks; retain screenshots and exact artifact identity.
- [ ] True-TUI `/tasknode` success/expiry/cancel/relink if exposed by the Desktop candidate; literal keys and checkpoints, no helper-only repair.
- [ ] Exercise OFF/ON/OFF and recovery; flag-OFF hides discovery, denies new actions and preserves supported recovery/data.
- [ ] Independent reviewer reproduces a seeded broken recovery case; Travis reviews public card and candidate handoff.
- [ ] Run plan/sprint checks and affected focused tests on final tree; test evidence names the allocated commands, not generic promises.

## Exit evidence

- [ ] Implementation commit, signed candidate digest, contract versions and scope audit recorded.
- [ ] Final-tree evidence and independent review linked; unresolved packaging/publication prerequisites block completion.
- [ ] Named human acceptance recorded; no beta task published and no stable release inferred.
- [ ] Update Done/Remaining and archive only after all required gates; S02 stays draft until then.
