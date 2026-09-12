---
sprint_id: "PF-30-S01"
title: "Typed source envelope and trusted ingress"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-30"
execution_order: 37
owner: "/root/provenance"
parallel_lane: "source-envelope"
write_scope: "codex-rs/protocol/src/provenance.rs, codex-rs/protocol/src/provenance_tests.rs, codex-rs/core/src/security/ingress/, codex-rs/core/src/context/provenance.rs, codex-rs/core/src/session/session.rs, codex-rs/core/src/client_tests.rs, codex-rs/core/src/session/turn.rs, codex-rs/core/src/client_common.rs, codex-rs/core/src/realtime_conversation.rs, codex-rs/core/src/realtime_conversation_tests.rs, codex-rs/core/src/tools/router.rs, codex-rs/core/src/mcp_tool_call.rs, codex-rs/core/tests/suite/provenance.rs, qa/security-levels/sprints/PF-30-S01-typed-source-envelope/, docs/sprints/current/p0-security-levels/pf-30-s01-typed-source-envelope.md"
integration_gate: "Codex /root owns protocol/context/test exports and shared Cargo/Bazel/lock registration, audits native ingress coverage without changing Permissive, reruns protocol/Core/governance plus actual TMUX on RTX and Astra High/Fable 5.1 High reviews (maximum five per lane). Contract-only evidence cannot complete this sprint."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/security-round5-provenance"
branch: "feat/security-round5-provenance"
base_commit: "07791288b6feeccfaee5a57c12452359cc666957"
depends_on: "PF-22-S02"
created: 2026-08-28
updated: 2026-09-04
---

# PF-30-S01 — Typed source envelope and trusted ingress

## Execution mandate

- Deliver: Content cannot assign its own authority or impersonate a human approval through source labels.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-30).
- Feature: `PF-30`.
- Product citation: **Non-negotiable controls** — “Classify instruction intent and provenance before external content can influence tools or financial actions.”
- Acceptance advanced: Content cannot assign its own authority or impersonate a human approval through source labels.
- Sources and archive disposition: [PF-30 reconciliation](../../../plans/security-source-reconciliation.md#pf-30).

## Code boundaries

- OpenClaw adoption reference: [OC-4](../../../plans/openclaw-source-review-2026-08-28.md#oc-4), [OC-10](../../../plans/openclaw-source-review-2026-08-28.md#oc-10) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/protocol/src/models.rs; codex-rs/core/src/tools/router.rs; codex-rs/core/src/mcp_tool_call.rs.
- Planned: codex-rs/protocol/src/provenance.rs; codex-rs/core/src/security/ingress.rs.
- Tests: planned colocated Rust test modules prefixed `pf_30_s01`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [x] Active plan; PF-22-S02 completed and archived at the exact recorded base.
- [x] Round-five allocation records a recoverable worktree, branch, base, owner, literal scope, and integration gate.
- [x] Read root/Rust/Core policies, product citation and the recorded OC-4/OC-10 source review; inspected native session/MCP/tool and all three provider request paths. The lost pinned source checkout was not represented as a fresh source inspection. Protected screening readiness remains unavailable and fail closed.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.
- [x] On 2026-09-02, product authority confirmed the recovered repository contains no surviving PF-30 branch, implementation commit, or handoff artifact; the absent `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-source-envelope` reservation was therefore released to draft.
- [x] Round-five envelope/admission, native sidecar/hooks, inherited policy floor and realtime guard reached tested source `e592cf75a`; the exact 22-Core/285-protocol evidence, baseline full-Core failures and three reviews are retained in QA. This is a qualified staged boundary, not completed production screening or a clean overall review.
- [x] Define immutable source ID/type, origin, actor, retrieval time, untrusted authority, digest, transformations and taint lineage separately from content. Descriptive protocol deserialization cannot mint Core admission.
- [x] Add named source/role/metadata and missing-source regressions, including a synthetic unknown route/tool variant; connect exact admitted input to the three real provider adapters with unchanged Permissive shaping.
- [x] Rolling continuation implements complete-input screening transport segmentation within the existing 2,048-byte input bound: chunks at most 512 bytes, one exact source/digest/count, full reassembly and atomic admission. No prefix release or per-segment authority. RTX after scoped fix/full formatting: 27 Core provenance tests and 22 content-security contract tests pass, including split Unicode escapes, partial/duplicate/cross-source/swapped-content chunks and exact provider-wire replay stability. Source checkpoint `078342d85` plus synchronized remote formatting; final candidate/TMUX/review ledger is recorded in QA.

## Remaining

- [ ] Connect a real trusted screening producer to the completed candidate/segment handoff. `codex-content-security` currently provides the contract/reassembler, not a production classifier; the positive tests use an explicitly synthetic engine. A production verdict cannot be fabricated and PF-35 qualification cannot be bypassed. Core client.rs and session/mod.rs belong exclusively to PF-30-S04; request any additional integration hooks from the coordinator.

- [ ] Port forged metadata/role/model-token and Unicode wrapper fixtures, including complete markers before a clipped marker. Host-generated authorization notices use a separate typed constructor; external labels and unsafe-hook switches cannot mint trust or bypass required screening.

- [ ] Assign envelopes only at trusted ingress for web/search, files, transcripts, social/trollbox/email, MCP/tool/plugin/hook output and child messages; unknown origin is untrusted, never human authority.
- [ ] Complete finer native producer identity and unsupported hosted/opaque ingress coverage. Generic tool/transcript origins remain conservative; producer names or source text cannot substitute for an observed host adapter. The three provider projections and escaping are implemented, but do not prove missing native source coverage.
- [ ] Keep user-issued authority on an authenticated human event channel; quoted text inside a human message is not automatically a grant.
- [ ] Test forged system/human markers, missing envelope, mixed-source chunks, Unicode confusables and provider round trips.
- [ ] Finish whole-feature native/production qualification and record final-tree review/TMUX evidence; no archive based on fixture screening alone. Shared registrations remain coordinator-owned.

## Verification

Round-five staged checkpoint: validated envelopes, native host observation/pending
screening handoff and three wire projections are implemented and tested, without
claiming production screening or complete source coverage. RTX focused protocol:
285/285; latest combined Core provenance/realtime: 88/88 (22 provenance tests).
Full Core: 3,455 passed, five request-permissions failures reproduced on the
allocation baseline, eight skips. Astra review 1 fixes and Fable review 2 realtime
remediation are tracked in the QA ledger. Final realtime actual-key TMUX passed;
review 3 verified those fixes and identified a separate unbound stage-one memory
worker outside the allocation. That finding is retained and escalated; no overall
clean review or complete memory-path protection is claimed.
2026-09-04: returned incomplete to draft, releasing its slot for PF-20-S03.
Coordinates/scope are historical, not current write authority. Frozen handoff:
source `2a4fb5857`, evidence `e890ae4a9`, integrated `0266c2db9`;
envelope/admission/segmentation contracts unchanged. Core 27/27, content-security
22/22, actual-key TMUX and clean Astra/Fable reviews; budget 5/5. Earlier baseline
failures and all remaining production/coverage gates persist; no dependency unlocks.

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-protocol pf_30_s01 && just test -p codex-core pf_30_s01`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: no feature UI; run the required supporting real-TMUX `/status`/clean-exit smoke, while PF-26-S02 retains final interaction qualification.
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-30-S01-typed-source-envelope/`; preserve the superseded browser-isolation artifacts already stored under `PF-30-S01/` unchanged.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
