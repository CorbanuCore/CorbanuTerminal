---
sprint_id: "PF-33-S02"
title: "Connection pinning and alternate-egress denial"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-33"
execution_order: 33
owner: "network-lane worker round 2 (codex, 2026-10-06)"
parallel_lane: "tui"
write_scope: "codex-rs/network-proxy/src/connect_policy.rs, codex-rs/network-proxy/src/upstream.rs, codex-rs/network-proxy/src/upstream_tests.rs, codex-rs/network-proxy/src/mitm.rs, codex-rs/network-proxy/src/mitm_tests.rs, codex-rs/network-proxy/src/destination.rs, codex-rs/network-proxy/src/destination_tests.rs, codex-rs/network-proxy/src/proxy.rs, docs/sprints/check.py, docs/sprints/tests/test_check.py, qa/security-levels/sprints/PF-33-S02/, qa/demos/index/PF-33-S02.md, docs/sprints/current/p0-security-levels/pf-33-s02-connection-pinning-and-bypass.md"
integration_gate: "PR to main under the per-sprint gate (sec-common decision 5): focused tests, GLM 5.2 tmux run, one Opus 5.5 High review, SOP videos; merged behind the existing url_destination_policy flag (default off). No Cargo, lockfile, lib.rs, config.rs or runtime.rs change. Shared-record hunks serialized by the integration owner: plan worktree coordinates, the PF-33-S01 and PF-28-S02 notes (coordinator decision), the sprint index row, and new demo specs qa/demos/specs/pf33s02-*.toml under the directory PF-30-S03 reserves (new files only)."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-33-s02-20261006"
branch: "pf-33-s02-20261006"
base_commit: "39c1f06213da3f15ec41cdd673a09a98cbc7d027"
depends_on: "PF-33-S01"
created: 2026-08-28
updated: 2026-10-06
---

# PF-33-S02 — Connection pinning and alternate-egress denial

## Execution mandate

- Deliver: An agent cannot bypass policy by changing transport, resolution, proxy or local socket.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-33).
- Feature: `PF-33`.
- Product citation: **Non-negotiable controls** — “Default to no secret export, arbitrary egress, clipboard exposure, or sensitive logging.”
- Acceptance advanced: An agent cannot bypass policy by changing transport, resolution, proxy or local socket.
- Sources and archive disposition: [PF-33 reconciliation](../../../plans/security-source-reconciliation.md#pf-33).

## Code boundaries

- OpenClaw adoption reference: [OC-2](../../../plans/openclaw-source-review-2026-08-28.md#oc-2), [OC-9](../../../plans/openclaw-source-review-2026-08-28.md#oc-9) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/network-proxy/src/policy.rs; codex-rs/sandboxing/src/{manager,spawn}.rs.
- Planned: codex-rs/network-proxy/src/connection_policy.rs; codex-rs/secret-broker/tests/egress.rs.
- Tests: planned colocated Rust test modules prefixed `pf_33_s02`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [x] Active plan. PF-33-S01 merged behind `url_destination_policy` (#210, #213) with gate evidence but is not
  archived; the coordinator let S02 start anyway (2026-10-06, PF-28-S01 precedent). The checker accepts a current
  dependency with `merged_behind_flag` and an existing `gate_evidence` file (`docs/sprints/check.py`, tested).
- [x] Read root and nearest implementation-path AGENTS.md; plan/worktree coordinates recorded; both checkers pass.
- [x] Reuses `url_destination_policy` (same feature, same proxy). Pinning lives in `connect_policy.rs` instead of a new
  `connection_policy.rs` so `lib.rs` (PF-27-S02 scope) is untouched. `secret-broker` is broker-lane scope: no
  `secret-broker/tests/egress.rs` here.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Use real transport fixtures to prove the checked DNS peer is used: mocks may skip DNS. Attack trusted-env/explicit/managed proxies, pinDns=false equivalents, NO_PROXY and stale pooled connections; abort/release one request without breaking an authorized sibling.

- [ ] Pin approved resolution to the actual connection; reject DNS rebinding, mismatched peer/TLS identity and reused pooled connections under changed authority.
- [ ] Enforce outbound-only broker/retriever routes using the OS backend; deny direct sockets, UDP/QUIC bypass, alternate proxies, Unix/domain sockets, host networking and metadata APIs.
- [ ] Distinguish authenticated local broker IPC from forbidden local network destinations; a caller cannot turn the loopback exemption into general SSRF.
- [ ] Route self-hosted SearXNG only through a separately human-configured exact service endpoint and narrow adapter; never widen public-fetch private-IP policy.
- [ ] Test NO_PROXY/env changes, malicious proxy config, CONNECT/blind tunnel, pinned-TLS failure, stale grants and platform backend failure; record supported-platform capability matrix.
- [ ] Add named `pf_33_s02` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-network-proxy pf_33_s02 && just test -p codex-secret-broker pf_33_s02 && just test -p codex-sandboxing pf_33_s02`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-33-S02/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
