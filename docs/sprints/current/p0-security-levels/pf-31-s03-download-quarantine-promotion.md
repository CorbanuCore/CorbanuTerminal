---
sprint_id: "PF-31-S03"
title: "Download quarantine and human file promotion"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-31"
execution_order: 48
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-31-S02, PF-24-S01"
created: 2026-08-28
updated: 2026-08-28
---

# PF-31-S03 — Download quarantine and human file promotion

## Execution mandate

- Deliver: Downloaded content reaches a workspace only after bounded checks and exact human promotion.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-31).
- Feature: `PF-31`.
- Product citation: **Non-negotiable controls** — “Default to no secret export, arbitrary egress, clipboard exposure, or sensitive logging.”
- Acceptance advanced: Downloaded content reaches a workspace only after bounded checks and exact human promotion.
- Sources and archive disposition: [PF-31 reconciliation](../../../plans/security-source-reconciliation.md#pf-31).

## Code boundaries

- OpenClaw adoption reference: [OC-8](../../../plans/openclaw-source-review-2026-08-28.md#oc-8), [OC-10](../../../plans/openclaw-source-review-2026-08-28.md#oc-10) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/tui/src/bottom_pane/approval_overlay.rs; PF-31 retriever protocol.
- Planned: codex-rs/web-retriever/src/download.rs; codex-rs/tui/src/bottom_pane/download_review.rs.
- Tests: planned colocated Rust test modules prefixed `pf_31_s03`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-31-S02, PF-24-S01 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Cover overflow spill files as downloads: keep their raw bytes in sealed quarantine, preserve provenance on later reads, and prevent public-worker output from creating an ordinary workspace file that bypasses promotion.

- [ ] Store downloads as sealed bounded artifacts outside the agent workspace; record origin, MIME, size, digest and taint without executing or previewing active content.
- [ ] Reject archive bombs, traversal, symlinks, oversized/unsupported formats and MIME mismatches; re-check redirects and attachment filenames.
- [ ] Require trusted human review for an exact digest and destination before promoting a file; restrict path and permissions and preserve untrusted provenance after promotion.
- [ ] Prevent auto-open, autorun, shell execution, host-browser fallback and reuse of a promotion grant for changed bytes; revoke on cancel or expiry.
- [ ] Test safe promotion, rejection, tampered artifact, disk quota, cancellation and restart; human challenge handoff does not leak cookies into the public worker.
- [ ] Add named `pf_31_s03` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-web-retriever pf_31_s03 && just test -p codex-tui pf_31_s03`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: required: inspect download → Esc → approve exact destination → tamper/deny → restart; promotion never executes the file.
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-31-S03/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
