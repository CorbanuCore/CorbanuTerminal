---
sprint_id: "PF-30-S01"
title: "Typed source envelope and trusted ingress"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-30"
execution_order: 37
owner: "/root/provenance"
parallel_lane: "source-envelope"
write_scope: "codex-rs/protocol/src/provenance.rs, codex-rs/protocol/src/provenance_tests.rs, codex-rs/core/src/security/ingress/, codex-rs/core/src/context/provenance.rs, codex-rs/core/src/session/session.rs, codex-rs/core/src/client_tests.rs, codex-rs/core/src/session/mod.rs, codex-rs/core/src/session/turn.rs, codex-rs/core/src/client_common.rs, codex-rs/core/src/client.rs, codex-rs/core/src/tools/router.rs, codex-rs/core/src/mcp_tool_call.rs, codex-rs/core/tests/suite/provenance.rs, qa/security-levels/sprints/PF-30-S01-typed-source-envelope/, docs/sprints/current/p0-security-levels/pf-30-s01-typed-source-envelope.md"
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
- [x] Round-five recovery checkpoints `9c53c0a03`, `abb97e254`, `e6b7b602a`, `ac34a8456`, and `c34da0a48` preserve envelope/admission, native sidecar/hooks, inherited policy floor and regression source. These are unqualified checkpoints; tests/reviews remain unchecked below pending serialized exports/build registration.

## Remaining

- [ ] Add a synthetic newly introduced provider/tool/ingress variant absent from the registry and prove rejection or conservative untrusted handling before model admission. Missing registration, malformed envelopes and allow verdicts cannot manufacture authority.

- [ ] Port forged metadata/role/model-token and Unicode wrapper fixtures, including complete markers before a clipped marker. Host-generated authorization notices use a separate typed constructor; external labels and unsafe-hook switches cannot mint trust or bypass required screening.

- [ ] Define immutable source ID/type, origin, actor, retrieval time, trust/authority, digest, transformations and taint lineage outside content-controlled fields.
- [ ] Assign envelopes only at trusted ingress for web/search, files, transcripts, social/trollbox/email, MCP/tool/plugin/hook output and child messages; unknown origin is untrusted, never human authority.
- [ ] Serialize separate data envelopes into each provider context adapter; normalize forged role delimiters, special tokens, bidi/zero-width markers and unsafe display metadata without promoting content.
- [ ] Keep user-issued authority on an authenticated human event channel; quoted text inside a human message is not automatically a grant.
- [ ] Test forged system/human markers, missing envelope, mixed-source chunks, Unicode confusables and provider round trips.
- [ ] Add named `pf_30_s01` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

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
