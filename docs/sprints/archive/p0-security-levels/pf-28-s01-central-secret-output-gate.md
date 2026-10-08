---
sprint_id: "PF-28-S01"
title: "Central secret and protected-output gate"
status: completed
plan_file: "docs/plans/completed/main-2026-10-08-p0-security-levels.md"
plan_feature: "PF-28"
execution_order: 30
owner: "broker lane worker (codex, 2026-10-06)"
parallel_lane: "broker"
write_scope: "codex-rs/secret-broker/src/output_gate.rs, codex-rs/secret-broker/src/output_gate_tests.rs, codex-rs/secret-broker/src/lib.rs, codex-rs/secret-broker/Cargo.toml, codex-rs/core/src/security/disclosure_gate.rs, codex-rs/core/src/security/disclosure_gate_tests.rs, codex-rs/core/src/security/mod.rs, codex-rs/core/src/client.rs, codex-rs/core/src/exec.rs, codex-rs/core/src/session/mod.rs, codex-rs/core/src/session/session.rs, codex-rs/login/src/auth/storage_gate.rs, codex-rs/login/src/auth/storage.rs, codex-rs/login/Cargo.toml, codex-rs/vault/src/lib.rs, codex-rs/vault/src/tests.rs, codex-rs/otel/src/events/session_telemetry.rs, codex-rs/otel/Cargo.toml, codex-rs/feedback/, codex-rs/message-history/, codex-rs/state/src/log_db.rs, codex-rs/state/Cargo.toml, codex-rs/tui/src/gated_log_writer.rs, codex-rs/tui/src/lib.rs, codex-rs/tui/Cargo.toml, codex-rs/Cargo.lock, qa/security-levels/sprints/PF-28-S01/, qa/demos/specs/pf28s01-tool-output-gated.toml, qa/demos/specs/pf28s01-split-output.toml, qa/demos/specs/pf28s01-persistence-clean.toml, qa/demos/specs/pf28s01-baseline-flag-off.toml, qa/demos/index/PF-28-S01.md, docs/sprints/current/p0-security-levels/pf-28-s01-central-secret-output-gate.md"
integration_gate: "PR to main under the per-sprint gate (sec-common decision 5); merged behind secret_output_gate. Shared files kept to small hunks: the flag in codex-rs/features/src/lib.rs and codex-rs/core/config.schema.json, and one hunk in codex-rs/core/src/config/mod.rs (arm, snapshot off, warning)."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf28-s01-secret-gate-20261006"
branch: "feat/pf28-s01-secret-gate-20261006"
base_commit: "699bd4a82f78c5967de7daf4314d4d426b464260"
depends_on: "PF-27-S02"
created: 2026-08-28
updated: 2026-10-06
---

# PF-28-S01 — Central secret and protected-output gate

## Closure — 2026-10-06

Completed under the per-sprint gate (sec-common decision 5): merged as PR #208 behind `secret_output_gate`
(default off). It waited only on PF-27-S02, archived the same day. The open items moved; nothing below is claimed:

| Carried forward | To |
| --- | --- |
| Wrapped base64/hex and other decode-and-rescan cases; reflected values in responses | [PF-28-S02](../../archive/p0-security-levels/pf-28-s02-reflected-secret-response-scrubbing.md) |
| Register MCP OAuth tokens refreshed after start; withhold comma-separated or numbered seed phrases; scrub known text fields per type (no serde round trip); per-session stream state and fewer rescans under load | [PF-28-S02](../../archive/p0-security-levels/pf-28-s02-reflected-secret-response-scrubbing.md) (gate carry-overs) |
| Register a derived view for financial values | [PF-39-S01](../../current/p1-security-hardening/pf-39-s01-protected-financial-derived-views.md) (P1) |

## Execution mandate

- Deliver: Managed secret canaries never reach model, tool, persistence, diagnostic, or export sinks.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/completed/main-2026-10-08-p0-security-levels.md#pf-28).
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

- [x] Active plan; PF-27-S02 merged behind its flag (#191) and archived on 2026-10-06.
- [x] Read root and nearest implementation-path AGENTS.md; plan/worktree coordinates verified.
- [x] Source pins, crate/module paths and backend/API availability confirmed.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input.
- [x] Typed output classes and one registry of active values in `secret-broker/src/output_gate.rs`, inside Core's process. Agent processes never get raw values (PF-27-S02).
- [x] Exact values, JSON, percent, base64 (all alignments, both alphabets) and hex. Covers short values (whole-word, 3 to 5 bytes), overlapping and repeated values, chunk splits, rotation with leases, per-owner retirement, and more than 512 representations. Capacity exhaustion denies without evicting; oversized payloads are withheld.
- [x] Gated sinks: model requests (turns, compaction), recorded history, rollout transcript, client events (at `send_event` and again at delivery), errors, tool and prompt telemetry, TUI log, feedback, log database and prompt history. Rollout traces and shell snapshots are off while armed.
- [x] Sign-in tokens are registered on load and save (login, refresh, keyring), and vault values on reveal; a value that cannot be protected is not released. Seed phrases (also any three consecutive words) and private keys withhold the whole payload. Seeding skips ordinary settings.
- [x] Named `pf_28_s01` tests (38) in secret-broker, core, vault and login; Cargo lock updated.
- [x] GLM 5.2 TUI runs and four SOP videos; Opus 5.5 High review rounds dispositioned. [Evidence](../../../../qa/security-levels/sprints/PF-28-S01/README.md).

## Remaining

- [x] Every open item moved on 2026-10-06; see the closure table above.

## Verification

- [x] `just fix -p` on every touched crate, then `just fmt`; final diff inspected.
- [x] Focused: `just test -p codex-secret-broker -p codex-vault -p codex-login -p codex-otel -p codex-core -E 'test(pf_28_s01)'`: 38 passed. (`codex-secrets` gained no tests: its sanitizer is not used by the gate.)
- [x] Integration: affected crate suites; results in the evidence README.
- [x] TUI applicability: four GLM 5.2 runs recorded as SOP videos ([index](../../../../qa/demos/index/PF-28-S01.md)).
- [x] Candidate, commands and outcomes recorded; synthetic canaries only.
- [x] Milestone qualification (isolated code-blind VM run, human sign-off) runs at "Moderate ships", not per sprint (decision 5).

## Exit evidence

- [x] Implementation commits and outputs under `qa/security-levels/sprints/PF-28-S01/`.
- [x] One independent Opus 5.5 High review, run in rounds until approved, dispositioned.
- [x] PF-26 final qualification moved to the P1 hardening plan (decision 3); no release-complete claim here.
- [x] Done/Remaining reflect reality; record archived and the plan/index updated.
