---
sprint_id: "PF-14-S02"
title: "Review packet and secret gate"
status: draft
plan_file: "docs/plans/proposed/arbitrary-model-autoreview.md"
plan_feature: "PF-14"
execution_order: 2
owner: "Jim Ricketts"
lane: "autoreview"
write_scope: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-14-S01, PF-28-S01, PF-29-S01"
created: 2026-08-24
updated: 2026-08-27
---

# PF-14-S02 — Review packet and secret gate

## Execution mandate

- Deliver: one canonical, complete, scanned packet per bounded reviewer call.
- Excludes: provider routing, reviewer execution, and TUI presentation.

## Plan linkage

- Upstream: [plan touch record](../../../plans/proposed/arbitrary-model-autoreview.md#native-lifecycle-and-upstream-touch-record); resolve this sprint's adapter rows.
- Plan: [Arbitrary-model Autoreview](../../../plans/proposed/arbitrary-model-autoreview.md)
- Feature: `PF-14`
- Acceptance advanced: a credential in any outgoing section refuses the send with zero provider calls.

## Code boundaries

- Existing: `benchmarks/scan_exact_keys.py`; upstream `scripts/autoreview::scan_outgoing_review_pack`
- Planned: `codex-rs/core/src/autoreview/bundle.rs`; `codex-rs/core/src/autoreview/secret_scan.rs`
- Tests: `codex-rs/core/tests/suite/arbitrary_model_autoreview.rs::packet_*`

## Preconditions

- [ ] Allocate literal implementation/test/registration scopes and check cross-plan collisions before readiness.
- [ ] Plan upstream baseline, adapter ownership, and exact contract tests are resolved before readiness.
- [ ] Plan is active.
- [ ] Dependencies are completed.
- [ ] Worktree, branch, and base commit are exact and match the plan.

## Done

- [x] Sprint record created and linked to PF-14.

## Remaining

- [ ] Resolve `local`, `branch`, and `commit` targets without shell interpolation and reject ambiguous or missing git state.
- [ ] Include metadata, prompt/datasets, tracked diff, deleted lines, renames, binary markers, and bounded untracked file snapshots in one canonical byte representation.
- [ ] Write each exact outgoing packet under an owner-only temporary directory and attest file permissions before scanning.
- [ ] Require a pinned compatible TruffleHog; scan `verified,unknown`; fail on missing binary, scanner error, timeout, or finding without redacting and forwarding.
- [ ] Add the shared exact-key canary scan and map findings back to repository paths without exposing matched secret bytes.
- [ ] Consume PF-28 confidentiality and PF-29 source-envelope contracts; preserve packet provenance and use shared secret classification without redacting and forwarding failed scans.
- [ ] Partition only at complete section or file boundaries, cap at eight packets, never truncate, and refuse larger input with a reduction action.
- [ ] Delete private temporary artifacts on success, refusal, cancellation, and crash recovery.
- [ ] Cover added, deleted, prompt, dataset, untracked, binary, oversized, symlink, invalid UTF-8, and scanner-failure cases.

## Verification

- [ ] Run fix/format before final tests; execute Rust commands below from `codex-rs`.
- [ ] Record applicable upstream adapter evidence or justified non-applicability; structural checks alone are not qualification.
- [ ] Focused test: `just test -p codex-core autoreview_packet`
- [ ] Adversarial test: `just test -p codex-core autoreview_secret_gate`
- [ ] Integration test: provider mock asserts zero calls on every scan refusal.
- [ ] If this sprint changes an interactive path, complete its actual-key success/failure/recovery proof before completion; otherwise record why internal-only. S07 repeats integrated proof.

## Exit evidence

- [ ] Implementation commit recorded.
- [ ] Final-tree test output linked.
- [ ] Exact outgoing-packet fixture hashes linked.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/arbitrary-model-autoreview/`.
