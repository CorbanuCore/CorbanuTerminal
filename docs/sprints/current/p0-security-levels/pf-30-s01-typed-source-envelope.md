---
sprint_id: "PF-30-S01"
title: "Typed source envelope and trusted ingress"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-30"
execution_order: 37
owner: "/root/pf30_source_envelope"
parallel_lane: "source-envelope"
write_scope: "codex-rs/security-policy/src/provenance.rs, codex-rs/security-policy/src/provenance_tests.rs, codex-rs/protocol/src/provenance.rs, codex-rs/core/src/security/ingress/, codex-rs/core/src/mcp_tool_call.rs, codex-rs/core/src/mcp_tool_call_tests.rs, codex-rs/core/src/hook_runtime.rs, codex-rs/core/src/hook_runtime_tests.rs, codex-rs/core/src/client.rs, codex-rs/core/src/client_tests.rs, codex-rs/core/src/tools/router.rs, codex-rs/core/src/tools/router_tests.rs, codex-rs/core/src/agent/control/spawn.rs, codex-rs/core/src/agent/control_tests.rs, qa/security-levels/sprints/PF-30-S01-typed-source-envelope/, docs/sprints/current/p0-security-levels/pf-30-s01-typed-source-envelope.md"
integration_gate: "The round-four integration owner audits the literal PF-30 diff against PF-27 and the independent Claude-auth lane, alone registers shared protocol/Core modules and Cargo/Bazel/lock/schema edges, reruns PF-21 compatibility and all affected suites on the combined tree, completes a real-TMUX smoke and read-only Corbanu/Claude Opus 5 Max closure, and archives only after deterministic cache-stable provider serialization is proven."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-source-envelope"
branch: "feat/p0-security-source-envelope"
base_commit: "b457249aa29c912cb2d5404f0939e8b2386f4e5e"
depends_on: "PF-22-S02"
created: 2026-08-28
updated: 2026-08-31
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
- [x] Exact CorbanuDrive worktree/branch/base and a PF-27-disjoint worker scope are allocated; shared registrations remain integration-owner-only.
- [ ] Read root and nearest implementation-path AGENTS.md; confirm source pins, provider request paths and fail-closed backend/API availability before code changes.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

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
