---
sprint_id: "PF-29-S01"
title: "Source envelopes and untrusted ingress"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-29"
execution_order: 17
owner: "Jim Ricketts"
lane: "content"
write_scope: "codex-rs/core/src/security/ingress, codex-rs/core/src/mcp_tool_call.rs, codex-rs/core/src/session/inject.rs, codex-rs/core/src/tools/handlers/read_file.rs"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-27-S01, PF-26-S01"
created: 2026-08-27
updated: 2026-08-27
---

# PF-29-S01 — Source envelopes and untrusted ingress

## Execution mandate

- Deliver: Attach non-forgeable host provenance to supported external inputs before model ingestion.
- Excludes: Browser process containment, derived lifecycle propagation, classifier training, and protected-action dispatch.

## Plan linkage

- Upstream: [plan touch record](../../../plans/active/p0-security-levels.md#upstream-touch-record); resolve this sprint's adapter rows.
- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md).
- Feature: `PF-29`; see the plan's adopted contract and requirement traceability.
- Acceptance advanced: Attach non-forgeable host provenance to supported external inputs before model ingestion.

## Code boundaries

- Existing (paths below `codex-rs/`): `core/src/mcp_tool_call.rs`; `core/src/session/inject.rs`; `core/src/tools/handlers/read_file.rs`.
- Planned: `core/src/security/ingress/` source adapters and normalized-content tests.
- Tests: sibling unit/integration regressions and the PF-26 fixture matrix.

## Preconditions

- [ ] Plan upstream baseline, adapter ownership, and exact contract tests are resolved before readiness.
- [ ] Listed dependencies are completed and archived.
- [ ] Read applicable root and nested AGENTS instructions; plan remains active.
- [ ] Allocate exact owner/worktree/branch/base and literal write scope in the plan; check lane/slot conflicts.

## Done

- [x] Approved feature contract decomposed into this single-feature draft.

## Remaining

- [ ] Map repository/files, web/browser/document extraction, tool/MCP/connector/email output, prior memory, and delegated results to host-issued source envelopes.
- [ ] Publish every concrete ingress adapter's owner, supported/denied state, fixture ID, and result; fail coverage checks for unclassified paths.
- [ ] Record pinned OpenClaw source/license and adapted wrapper/normalization fixtures; implement natively and keep typed host provenance authoritative over marker text.
- [ ] Record unsupported ingress paths and disable their stronger-mode use until a local adapter can enforce the contract.
- [ ] Normalize hostile encodings, executable/hidden markup, and model-control tokens; reject forged source delimiters without promoting sanitized text to trusted instructions.
- [ ] Retain origin, digest, trust class, and derivation identifiers in bounded model context and secret-free audit metadata.
- [ ] Expose the normalized-content handoff for PF-30 and sticky-taint input for S02; publish health/degradation through PF-27.
- [ ] Test each supported source, spoofed markers, parser failures, oversized content, and recovery using PF-26 fixtures; preserve Permissive.

## Verification

- [ ] Record applicable upstream adapter evidence or justified non-applicability; structural checks alone are not qualification.
- [ ] Run `just fix -p <affected-crate>` and `just fmt` in `codex-rs` before final tests.
- [ ] Run focused and affected integration tests with `just test -p <affected-crate>`; never direct `cargo test`.
- [ ] Record exact dependency commits, candidate, commands, artifacts, and missing coverage; no pass by code presence.
- [ ] Run source-coverage fixtures with hostile text and forced detector misses; no classifier is the trust boundary.
- [ ] Exercise benign quotations and non-secret task-hijacking cases; distinguish normalization failures, task corruption, and authority violations.
- [ ] TUI applicability: true-PTY hostile-file/MCP ingestion, visible rejection, and retry with separate prompt/Enter actions.

## Exit evidence

- [ ] Final candidate and dependency commits, commands, platform results, and artifact paths recorded.
- [ ] Applicable actual-key proof is linked; no missing test is relabeled a pass.
- [ ] Ledgers reflect reality; completed record is archived and plan evidence linked.
