---
sprint_id: "PF-40-S02"
title: "Isolated advisory Agent Sweep reviewer"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-40"
execution_order: 69
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-40-S01, PF-36-S01"
created: 2026-08-28
updated: 2026-08-28
---

# PF-40-S02 — Isolated advisory Agent Sweep reviewer

## Execution mandate

- Deliver: An optional second model can advise on risk but cannot become a second privileged agent.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-40).
- Feature: `PF-40`.
- Product citation: **Non-negotiable controls** — “Support allowlists, denylists, rate limits, daily loss/notional/leverage caps, cooldowns, revocation, and a kill switch.”
- Acceptance advanced: An optional second model can advise on risk but cannot become a second privileged agent.
- Sources and archive disposition: [PF-40 reconciliation](../../../plans/security-source-reconciliation.md#pf-40).

## Code boundaries

- OpenClaw adoption reference: [OC-4](../../../plans/openclaw-source-review-2026-08-28.md#oc-4), [OC-8](../../../plans/openclaw-source-review-2026-08-28.md#oc-8) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: PF-40 sanitized event stream; codex-rs/core/src/agent/control.rs.
- Planned: codex-rs/core/src/security/sweep/reviewer.rs.
- Tests: planned colocated Rust test modules prefixed `pf_40_s02`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-40-S01, PF-36-S01 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Treat reviewer output as untrusted advisory content in a separately constrained worker; reference tool-name policies and sessions_spawn naming do not prove Codex-native subagent isolation or authorize reviewer effects.

- [ ] Define an optional local/approved-model advisory reviewer consuming only bounded sanitized events; no vault, credentials, filesystem, tools, financial actions or authority-grant API.
- [ ] Keep event content untrusted and outputs typed as reason/severity recommendations; reject attempts to set policy, resume work, execute actions or request protected data.
- [ ] Reuse PF-36's human consent component for separate reviewer data/cost disclosure; keep the reviewer disabled without a qualified model/runtime and explicit reviewer consent.
- [ ] Map advisory findings to deterministic bounded escalation only; reviewer timeout/corruption cannot disable base rules or grant access.
- [ ] Test injected event instructions, fabricated tool calls, denial-of-service recommendations, malformed output and reviewer unavailable while base rules continue.
- [ ] Add named `pf_40_s02` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-core pf_40_s02`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-40-S02/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
