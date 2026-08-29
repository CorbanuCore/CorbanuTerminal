---
sprint_id: "PF-28-S01"
title: "Central secret and protected-output gate"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-28"
execution_order: 30
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-27-S02"
created: 2026-08-28
updated: 2026-08-28
---

# PF-28-S01 — Central secret and protected-output gate

## Execution mandate

- Deliver: Managed secret canaries never reach model, tool, persistence, diagnostic, or export sinks.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-28).
- Feature: `PF-28`.
- Product citation: **Non-negotiable controls** — “Default to no secret export, arbitrary egress, clipboard exposure, or sensitive logging.”
- Acceptance advanced: Managed secret canaries never reach model, tool, persistence, diagnostic, or export sinks.
- Sources and archive disposition: [PF-28 reconciliation](../../../plans/security-source-reconciliation.md#pf-28).

## Code boundaries

- OpenClaw adoption reference: [OC-3](../../../plans/openclaw-source-review-2026-08-28.md#oc-3) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/secrets/src/{lib,sanitizer}.rs; codex-rs/core/src/context_manager/mod.rs.
- Planned: codex-rs/secret-broker/src/output_gate.rs; codex-rs/core/src/security/disclosure_gate.rs.
- Tests: planned colocated Rust test modules prefixed `pf_28_s01`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-27-S02 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Trace every diagnostic/request-capture adapter to its final sink; test short secrets, capacity exhaustion and concurrent rotation. Retain all active managed values through their response lifetime or deny safely; registry eviction cannot make an in-flight secret unprotected.

- [ ] Add short values below six characters, more than 512 distinct representations, repeated/encoded values and split-output chunks to canary tests. Protected sinks remove the whole credential, not a diagnostic prefix/suffix; capacity exhaustion must fail safely without evicting live protection.

- [ ] Define typed output classes and a single broker-side registry for active secret values; never copy raw registry values into the agent process.
- [ ] Cover exact values, URI/JSON/base64 encodings and bounded pattern matches; include short credentials, overlapping matches, chunk boundaries and rotation; capacity exhaustion denies rather than evicting live protection.
- [ ] Gate model requests/responses, tool results, transcript, traces, errors, audit, snapshots, exports and diagnostic artifacts before persistence or presentation; attach provenance to safe output.
- [ ] Keep operational credentials/seeds/private keys permanently non-disclosable in protected modes; protected financial values require a separately authorized derived view, not string redaction alone.
- [ ] Add sentinel canaries for every sink, structured/streamed/error output and concurrency; unknown encoding or oversized payload receives bounded denial, not a claimed universal detector guarantee.
- [ ] Add named `pf_28_s01` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-secret-broker pf_28_s01 && just test -p codex-core pf_28_s01 && just test -p codex-secrets pf_28_s01`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-28-S01/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
