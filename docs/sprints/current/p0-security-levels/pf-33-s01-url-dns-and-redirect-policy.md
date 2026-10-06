---
sprint_id: "PF-33-S01"
title: "URL DNS and redirect policy"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-33"
execution_order: 32
owner: "first-free lane worker (codex, 2026-10-06)"
parallel_lane: "tui"
write_scope: "codex-rs/network-proxy/src/destination.rs, codex-rs/network-proxy/src/destination_contract.rs, codex-rs/network-proxy/src/destination_tests.rs, codex-rs/network-proxy/src/http_proxy.rs, codex-rs/network-proxy/src/mitm.rs, codex-rs/network-proxy/src/socks5.rs, codex-rs/network-proxy/src/connect_policy.rs, codex-rs/core/src/network_policy_decision.rs, codex-rs/core/src/network_policy_decision_tests.rs, qa/security-levels/sprints/PF-33-S01/, qa/demos/index/PF-33-S01.md, docs/sprints/current/p0-security-levels/pf-33-s01-url-dns-and-redirect-policy.md"
integration_gate: "PR to main under the per-sprint gate (sec-common decision 5): focused tests, GLM 5.2 tmux run, one Opus 5.5 High review, SOP videos; merged behind url_destination_policy (default off). Shared files serialized by the integration owner, not reserved here: one module line in codex-rs/network-proxy/src/lib.rs and one Core-only field in codex-rs/network-proxy/src/config.rs (both listed by merged PF-27-S02), the flag registration in codex-rs/features/src/lib.rs and codex-rs/core/config.schema.json, one hunk in codex-rs/core/src/config/mod.rs, and new demo specs qa/demos/specs/pf33s01-*.toml under the directory PF-30-S03 reserves (new files only)."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-33-s01-20261006"
branch: "feat/pf-33-s01-url-dns-redirect"
base_commit: "a662c2ce357ee542fdac08ecaf083d27fd58391b"
depends_on: "PF-27-S02, PF-33-S03"
created: 2026-08-28
updated: 2026-10-06
---

# PF-33-S01 — URL DNS and redirect policy

## Execution mandate

- Deliver: URL authorization remains valid through DNS and every redirect, not merely on the initial hostname.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-33).
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

- [ ] All dependencies in front matter are completed and archived; plan remains active.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Port mixed/private DNS, mapped IPv6 and per-hop redirect cases; distinguish restricting host allowlists from private-network trust grants. Explicitly authorize scheme/port/method/path and redirect body/credential replay; reference hostname binding alone is insufficient.

- [ ] Canonicalize scheme, IDNA hostname, port, userinfo, literal IP and unusual numeric forms; public retrieval permits HTTPS only and rejects ambiguous/credential-bearing URLs.
- [ ] Validate every A/AAAA answer and connected peer; deny loopback/private/link-local/metadata/reserved/multicast and IPv4-mapped variants, mixed public/private answers and DNS failures.
- [ ] Re-authorize every redirect and retry with hop/time/byte limits; drop credentials across origins, reject downgrade and auth-host confusion.
- [ ] Bind credential adapters to exact normalized host, port, method and supported path; this is stricter than a hostname allowlist.
- [ ] Test redirect chains, dual stack, alternate IP encodings, trailing dots, suffix confusion, CNAME chains and synthetic DNS fixtures without contacting real private endpoints.
- [ ] Add named `pf_33_s01` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-network-proxy pf_33_s01`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-33-S01/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
