---
sprint_id: "PF-33-S01"
title: "URL DNS and redirect policy"
status: completed
plan_file: "docs/plans/completed/main-2026-10-08-p0-security-levels.md"
plan_feature: "PF-33"
execution_order: 32
owner: "first-free lane worker (codex, 2026-10-06)"
parallel_lane: "tui"
write_scope: "codex-rs/network-proxy/src/destination.rs, codex-rs/network-proxy/src/destination_contract.rs, codex-rs/network-proxy/src/destination_tests.rs, codex-rs/network-proxy/src/http_proxy.rs, codex-rs/network-proxy/src/mitm.rs, codex-rs/network-proxy/src/socks5.rs, codex-rs/network-proxy/src/connect_policy.rs, codex-rs/core/src/network_policy_decision.rs, codex-rs/core/src/network_policy_decision_tests.rs, qa/security-levels/sprints/PF-33-S01/, qa/demos/index/PF-33-S01.md, docs/sprints/current/p0-security-levels/pf-33-s01-url-dns-and-redirect-policy.md"
integration_gate: "PR to main under the per-sprint gate (sec-common decision 5): focused tests, GLM 5.2 tmux run, one Opus 5.5 High review, SOP videos; merged behind url_destination_policy (default off). Shared files serialized by the integration owner, not reserved here: one module line in codex-rs/network-proxy/src/lib.rs and one Core-only field in codex-rs/network-proxy/src/config.rs, a flag accessor in runtime.rs and the removed cargo-shear exception in Cargo.toml (all listed by merged PF-27-S02), the flag registration in codex-rs/features/src/lib.rs and codex-rs/core/config.schema.json, one hunk in codex-rs/core/src/config/mod.rs, and new demo specs qa/demos/specs/pf33s01-*.toml under the directory PF-30-S03 reserves (new files only)."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-33-s01-20261006"
branch: "feat/pf-33-s01-url-dns-redirect"
base_commit: "a662c2ce357ee542fdac08ecaf083d27fd58391b"
depends_on: "PF-27-S02, PF-33-S03"
created: 2026-08-28
updated: 2026-10-06
---

# PF-33-S01 — URL DNS and redirect policy

## Closure — 2026-10-06

Completed under the per-sprint gate (sec-common decision 5): merged (PR #210) behind `url_destination_policy`
(default off; Permissive unchanged) in the managed network proxy. It waited only on PF-27-S02, archived the same day.
[Evidence, behaviour and known limits](../../../../qa/security-levels/sprints/PF-33-S01/README.md).

| Carried forward | To |
| --- | --- |
| Bind legacy per-host credential adapters to exact host, port, method and path (`credential_broker.rs`) | [PF-28-S02](../../archive/p0-security-levels/pf-28-s02-reflected-secret-response-scrubbing.md) (broker lane; coordinator, 2026-10-06) |
| Local upstream proxies, connection pinning to the checked answers, in-process MITM test seam | Done in [PF-33-S02](../../archive/p0-security-levels/pf-33-s02-connection-pinning-and-bypass.md) |
| POST answered 301/302 refused; more than 16 answers refused; timed-out lookups not cancelled; redirects to runtime-only approved hosts refused | Documented behaviour; not scheduled |

## Execution mandate

- Deliver: URL authorization remains valid through DNS and every redirect, not merely on the initial hostname.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/completed/main-2026-10-08-p0-security-levels.md#pf-33).
- Feature: `PF-33`.
- Product citation: **Non-negotiable controls** — “Default to no secret export, arbitrary egress, clipboard exposure, or sensitive logging.”
- Acceptance advanced: URL authorization remains valid through DNS and every redirect, not merely on the initial hostname.
- Sources and archive disposition: [PF-33 reconciliation](../../../plans/security-source-reconciliation.md#pf-33).

## Code boundaries

- OpenClaw adoption reference: [OC-2](../../../plans/openclaw-source-review-2026-08-28.md#oc-2), [OC-9](../../../plans/openclaw-source-review-2026-08-28.md#oc-9) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/network-proxy/src/policy.rs.
- Planned: codex-rs/network-proxy/src/{destination,destination_tests}.rs.
- Tests: planned colocated Rust test modules prefixed `pf_33_s01`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [x] PF-33-S03 completed and archived; PF-27-S02 merged behind its flag (#191) and archived on 2026-10-06.
- [x] Root and `codex-rs` AGENTS.md read; plan/worktree coordinates recorded; both checkers pass.
- [x] Reused the frozen `pf33-destination-policy/v1` contract unchanged except one visibility change.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input.
- [x] `network-proxy/src/destination.rs`: a guard over the contract. Public retrieval is HTTPS on 443 with standard
  methods; private networks need an exact private-service grant (`allow_local_binding` and literal local allowlist
  entries are not grants); reserved names never reach a resolver without one.
- [x] Every A/AAAA answer is checked (loopback, private, link-local, metadata, CGNAT, multicast, NAT64/6to4, mapped
  IPv6, mixed answers, empty, failure, more than 16); the connector refuses non-public peers under the guard.
- [x] URLs canonicalized by the contract (IDNA, case, trailing dot, numeric IPv4 forms, mapped IPv6); userinfo,
  fragments and ambiguous syntax refused; plain HTTP and other ports refused.
- [x] Every CONNECT and SOCKS TCP tunnel is checked and intercepted (UDP relays refused); each inner request and every 3xx is re-authorized
  before it is relayed: host allowlist before DNS, downgrade, method/body replay, private targets; `Location` is
  screened and rewritten to the checked absolute URL. Chains are cut at 10 hops or 120 s (late follow-ups refused);
  `Authorization` is stripped on cross-origin hops. Byte bound: `Location` ≤ 4096 bytes (contract).
- [x] Agent sees "blocked by the URL destination policy (<reason>)"; records and logs carry host, port and reason only.
- [x] 23 `pf_33_s01` network-proxy tests with synthetic DNS (no private endpoint contacted), plus a core test.

## Remaining

- [x] Every open item moved or recorded on 2026-10-06; see the closure table above.

## Verification

- [x] `just fix -p codex-network-proxy`, `-p codex-features`, `-p codex-core`; `just fmt`; final diff inspected.
- [x] Focused: `cargo test -p codex-network-proxy pf_33_s01` (23 passed); `cargo test -p codex-core pf_33_s01` (1).
- [x] Integration: `cargo test -p codex-network-proxy` (265 + 16 contract tests); core schema fixture test passes.
- [x] TUI: GLM 5.2 tmux runs through the real proxy against httpbin.org, recorded as SOP videos.
- [x] Independent Opus 5.5 High review and re-checks; findings dispositioned in the evidence README.
- [x] Linux and Bazel CI on the PR: all checks green; merged as PR #210 (`8dd531714a`).
- [x] PF-26 final-candidate requalification runs at the milestones, not per sprint (decision 5).

## Exit evidence

- [x] Commits, commands, outcomes and review records under `qa/security-levels/sprints/PF-33-S01/`.
- [x] PF-26 final qualification moved to the P1 hardening plan (decision 3); no release-complete claim here.
- [x] Done/Remaining reflect reality; record archived and the plan/index updated.
