---
sprint_id: "PF-28-S02"
title: "Reflected-secret response scrubbing"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-28"
execution_order: 31
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-28-S01"
created: 2026-08-28
updated: 2026-08-28
---

# PF-28-S02 — Reflected-secret response scrubbing

## Execution mandate

- Deliver: Even an allowed provider reflecting its credential cannot disclose it through Corbanu's protected-mode output path.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-28).
- Feature: `PF-28`.
- Product citation: **Non-negotiable controls** — “Default to no secret export, arbitrary egress, clipboard exposure, or sensitive logging.”
- Acceptance advanced: Even an allowed provider reflecting its credential cannot disclose it through Corbanu's protected-mode output path.
- Sources and archive disposition: [PF-28 reconciliation](../../../plans/security-source-reconciliation.md#pf-28).

## Code boundaries

- OpenClaw adoption reference: [OC-2](../../../plans/openclaw-source-review-2026-08-28.md#oc-2), [OC-3](../../../plans/openclaw-source-review-2026-08-28.md#oc-3) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/network-proxy/src/credential_broker.rs; PF-27 broker transport.
- Planned: codex-rs/secret-broker/src/response_gate.rs; codex-rs/secret-broker/tests/reflection.rs.
- Tests: planned colocated Rust test modules prefixed `pf_28_s02`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-28-S01 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Publish a protocol/encoding latency and buffer matrix with product-approved time-to-first-safe-output targets; bounded carry requires split/encoded/short/rotating-secret fixtures. Unknown codecs/resource exhaustion deny, and response streaming never waives whole-ingress screening.

- [ ] Exercise a permitted origin reflecting credentials in response headers, bodies, errors and split/encoded streams before model, log or persistence handoff; neither request substitution nor diagnostic masking satisfies this gate.

- [ ] Keep resolved request credentials registered until response, redirects, trailers, stream completion, cancellation and retries finish; scrub before crossing back into agent-readable memory.
- [ ] Bound decompression, buffering and streaming carry; handle split/encoded echoes in body, headers, SSE, error pages and upstream debug messages.
- [ ] Reject unsupported content encodings, redirects carrying auth, malformed/oversized frames, and TLS-pinned blind tunnels requiring secrets; do not return raw response bytes on failure.
- [ ] Test a permitted-host fake provider deliberately reflecting each active credential; scan downstream model payloads, logs, artifacts and reconnect output.
- [ ] Prove timeout/revoke/rotation clears buffers and kills old streams without reusing stale auth; run Permissive compatibility regressions.
- [ ] Add named `pf_28_s02` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-secret-broker pf_28_s02 && just test -p codex-network-proxy pf_28_s02`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-28-S02/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
