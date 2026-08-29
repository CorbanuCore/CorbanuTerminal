---
sprint_id: "PF-33-S01"
title: "URL DNS and redirect policy"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-33"
execution_order: 32
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-27-S02, PF-33-S03"
created: 2026-08-28
updated: 2026-08-28
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
