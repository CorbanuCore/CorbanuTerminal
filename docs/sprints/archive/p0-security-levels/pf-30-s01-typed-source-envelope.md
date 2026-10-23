---
sprint_id: "PF-30-S01"
title: "Typed source envelope and trusted ingress"
status: completed
plan_file: "docs/plans/completed/main-2026-10-08-p0-security-levels.md"
plan_feature: "PF-30"
execution_order: 37
owner: "untrusted-content lane"
parallel_lane: "untrusted-content"
write_scope: "codex-rs/core/config.schema.json, codex-rs/core/src/client.rs, codex-rs/core/src/client_common.rs, codex-rs/core/src/client_tests.rs, codex-rs/core/src/context_manager/mod.rs, codex-rs/core/src/context_manager/normalize.rs, codex-rs/core/src/hook_runtime.rs, codex-rs/core/src/security/ingress/, codex-rs/core/src/session/mod.rs, codex-rs/core/src/session/rollout_budget.rs, codex-rs/core/src/session/session.rs, codex-rs/core/src/session/time_reminder.rs, codex-rs/core/src/session/token_budget.rs, codex-rs/core/src/stream_events_utils.rs, codex-rs/core/src/tasks/mod.rs, codex-rs/core/src/tasks/user_shell.rs, codex-rs/core/tests/suite/provenance.rs, codex-rs/features/src/lib.rs, qa/security-levels/sprints/PF-30-S01-typed-source-envelope/, qa/demos/specs/, qa/demos/index/PF-30-S01.md, docs/sprints/current/p0-security-levels/pf-30-s01-typed-source-envelope.md"
integration_gate: "Per-sprint gate of 2026-10-06: focused tests, GLM 5.2 tmux demos, one independent Opus 5.5 High review; merge behind source_envelopes."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf30-s01-20261006"
branch: "feat/pf-30-s01-source-envelope"
base_commit: "cb78550a31cdeec2a0c37cd0fc5608fba9b6cb6c"
depends_on: "PF-22-S02"
created: 2026-08-28
updated: 2026-10-06
---

# PF-30-S01 — Typed source envelope and trusted ingress

## Execution mandate

- Deliver: Content cannot assign its own authority or impersonate a human approval through source labels.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/completed/main-2026-10-08-p0-security-levels.md#pf-30).
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

- [x] Active plan; PF-22-S02 archived. Security decisions of 2026-10-06 apply (PF-35 moved to P1; lighter
  per-sprint gate; merge behind a feature flag).

## Done

- [x] Round five (2026-09-04, frozen handoff `2a4fb5857`/`e890ae4a9`, integrated `0266c2db9`, all on main):
  immutable envelopes, Core-only admission, complete-input segmentation, three provider projections and
  fail-closed Moderate/Aggressive. Evidence in QA (`qualification.md`, `segmentation-qualification.md`).
- [x] 2026-10-06: behind the default-off `source_envelopes` feature, Moderate/Aggressive no longer fail closed.
  A deterministic host producer (`core/src/security/ingress/structural.rs`) runs each external text through the
  existing screening contract and sends it as labelled untrusted data (`<corbanu_untrusted_data>`,
  `source=<kind> id sha256 authority=none`). It neutralizes wrapper closes, role tags, chat special tokens,
  look-alike brackets/letters and invisible/bidi/tag/variation characters, and withholds oversize text.
  Permissive and flag-off requests are unchanged.
- [x] Standing comes only from Core recording seams: the human prompt, host context/reminders/world state, and
  the provider stream (assistant text and exact model structure). Tool/MCP/hook/user-shell/agent output and
  anything unrecorded (injected, restored, forked) is labelled; unrecorded calls become labelled data with their
  outputs; unrecorded reasoning and unknown variants are withheld. Quoted approvals never become grants.
- [x] Resume/fork keeps restored messages labelled and reinjects host context once on the next protected turn.
- [x] Gate: `pf_30_s01` Core 39/39; full Core 3,740 pass with 2 failures that reproduce on main; features,
  protocol and content-security 349/349; GLM 5.2 real-TUI demos; Opus 5.5 High review, final APPROVE.
  Evidence: [labelled-ingress-gate.md](../../../../qa/security-levels/sprints/PF-30-S01-typed-source-envelope/labelled-ingress-gate.md),
  videos: [qa/demos/index/PF-30-S01.md](../../../../qa/demos/index/PF-30-S01.md).

- [x] Merged as PR #178 (merge commit `b96b23344b`); record archived 2026-10-06.

## Remaining

- Moved out: persisted per-message origins and lineage (PF-30-S02); classifier producer (PF-35, P1).
- Known gaps (QA file): images, audio and encrypted agent content unwrapped; MCP tool-search descriptions;
  realtime and memory summarisation still fail closed; parent→child messages labelled `child_agent`.

## Verification

- [x] `just fmt`; `just fix -p codex-core -p codex-features`.
- [x] Focused: `just test -p codex-core pf_30_s01` (39 ran); `just test -p codex-protocol -p codex-features -p codex-content-security`.
- [x] Integration: `just test -p codex-core` (baseline failures recorded). No manifest changes.
- [x] Real TUI: three GLM 5.2 tmux demos at `61b75ec43a`.
- Milestone (not this sprint): the full isolated code-blind VM run and human sign-off happen when Moderate ships.

## Exit evidence

- [x] PR #178 merged to main behind `source_envelopes`; merge commit `b96b23344b` recorded in the QA file.
- [x] Archived with status `completed`; plan and index links point here.
