---
sprint_id: "PF-33-S02"
title: "Connection pinning and alternate-egress denial"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-33"
execution_order: 33
owner: "network-lane worker round 2 (codex, 2026-10-06)"
parallel_lane: "tui"
write_scope: "codex-rs/network-proxy/src/connect_policy.rs, codex-rs/network-proxy/src/upstream.rs, codex-rs/network-proxy/src/upstream_tests.rs, codex-rs/network-proxy/src/mitm.rs, codex-rs/network-proxy/src/mitm_tests.rs, codex-rs/network-proxy/src/destination.rs, codex-rs/network-proxy/src/destination_tests.rs, codex-rs/network-proxy/src/proxy.rs, codex-rs/network-proxy/src/http_proxy.rs, codex-rs/sandboxing/src/seatbelt_tests.rs, codex-rs/core/src/config/network_proxy_spec.rs, codex-rs/core/src/windows_sandbox_tests.rs, docs/sprints/check.py, docs/sprints/tests/test_check.py, qa/security-levels/sprints/PF-33-S02/, qa/demos/index/PF-33-S02.md, docs/sprints/current/p0-security-levels/pf-33-s02-connection-pinning-and-bypass.md"
integration_gate: "PR to main under the per-sprint gate (sec-common decision 5): focused tests, GLM 5.2 tmux run, one Opus 5.5 High review, SOP videos; merged behind the existing url_destination_policy flag (default off). No Cargo, lockfile, lib.rs, config.rs or runtime.rs change. Shared-record hunks serialized by the integration owner: plan worktree coordinates, the PF-33-S01 and PF-28-S02 notes (coordinator decision), the sprint index row, one clause in docs/sprints/index.md, and new demo specs qa/demos/specs/pf33s02-*.toml under the directory PF-30-S03 reserves (new files only)."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-33-s02-20261006"
branch: "pf-33-s02-20261006"
base_commit: "39c1f06213da3f15ec41cdd673a09a98cbc7d027"
depends_on: "PF-33-S01"
created: 2026-08-28
updated: 2026-10-06
---

# PF-33-S02 — Connection pinning and alternate-egress denial

**October 6:** merged (PR #215) behind `url_destination_policy` (default off; Permissive unchanged).
[Evidence, platform matrix and review](../../../../qa/security-levels/sprints/PF-33-S02/README.md).

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

- [x] New single-feature record reconciled with current ownership and archived design input.
- [x] Pinning (`connect_policy.rs`, `destination.rs`, `mitm.rs`): each guarded request carries the checked answers;
  the connector dials only those, for that exact host/port, never re-resolving; no pin or another authority is
  refused; each address still passes the peer check; TLS identity still verified; every request dials fresh.
- [x] Alternate egress: inherited upstream proxies refused (`upstream_proxy`); the proxy's `x-unix-socket` route
  refused (`unix_socket`); a host denied after CONNECT is refused on the open tunnel (`host_denied`).
- [x] OS backend: the guard drops `allow_local_binding` and Unix-socket grants (`proxy.rs`, Windows provisioning in
  `network_proxy_spec.rs`), so Seatbelt allows only the proxy ports (no loopback services, raw DNS, Unix sockets).
  Linux proxy-routed mode (netns, seccomp AF_UNIX deny) unchanged. Matrix in the evidence README.
- [x] Real transport fixtures: loopback TCP/TLS with unresolvable `.invalid` names prove the pinned peer is used;
  fallback stays inside the checked answers; TLS mismatch fails; aborting one request leaves its sibling working.
- [x] 15 `pf_33_s02` network-proxy tests, 1 sandboxing (Seatbelt), 1 core (Windows provisioning). No Cargo or
  lockfile change.

## Remaining

- [ ] Brokered credential routes (isolated broker) dial without the pin; the broker re-resolves with its own peer
  check. Carry `PinnedPeers` through the broker protocol (broker lane; coordinator to place).
- [ ] SearXNG: no adapter exists in the tree; route it through an exact private-service grant when one is added.
- [ ] Runtime-approved (decider) hosts are not revoked on an open tunnel (PF-25-S02). No live rebinding-resolver
  fixture; rebinding is covered by pinning tests.

## Verification

- [x] `just fix -p codex-network-proxy`, `-p codex-sandboxing`, `-p codex-core`; `just fmt`; final diff inspected.
- [x] Focused: `cargo test -p codex-network-proxy pf_33_s02` (15), `-p codex-sandboxing pf_33_s02` (1),
  `-p codex-core --lib pf_33_s02` (1). `codex-secret-broker` is broker scope: no test there.
- [x] Integration: `just test -p codex-network-proxy` (296 passed); `just test -p codex-sandboxing` (79 passed, 2
  failing identically on the base commit: temp-dir writable roots, unrelated); core `windows_sandbox`/`network_proxy`.
- [x] GLM 5.2 tmux runs as five SOP videos ([index](../../../../qa/demos/index/PF-33-S02.md)); Opus 5.5 High review
  APPROVE WITH NITS, dispositions in the [evidence README](../../../../qa/security-levels/sprints/PF-33-S02/README.md).
- [x] PR CI green; merged behind `url_destination_policy` as PR #215 (`a0d96aea4b`).
- [ ] PF-26 final-candidate requalification (milestone gate).

## Exit evidence

- [x] Commits, commands, outcomes and review record under `qa/security-levels/sprints/PF-33-S02/`.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Remaining items placed; completed record archived and plan/navigation updated.
