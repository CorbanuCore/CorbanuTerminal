---
sprint_id: "PF-38-S01"
title: "Typed financial executor and deterministic limits"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-38"
execution_order: 63
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-30-S03, PF-27-S02, PF-18-S01"
created: 2026-08-28
updated: 2026-08-28
---

# PF-38-S01 — Typed financial executor and deterministic limits

## Execution mandate

- Deliver: Financial proposals are typed and limited outside the model before they can consume authority.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-38).
- Feature: `PF-38`.
- Product citation: **Non-negotiable controls** — “Simulate and display the complete expected effect before signing.”
- Acceptance advanced: Financial proposals are typed and limited outside the model before they can consume authority.
- Sources and archive disposition: [PF-38 reconciliation](../../../plans/security-source-reconciliation.md#pf-38).

## Code boundaries

- OpenClaw adoption reference: [OC-1](../../../plans/openclaw-source-review-2026-08-28.md#oc-1), [OC-2](../../../plans/openclaw-source-review-2026-08-28.md#oc-2) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/wallet/src/{envelope,balance,payment,lib}.rs; PF-16–19 authorization primitives.
- Planned: codex-rs/secret-broker/src/financial/{request,policy}.rs.
- Tests: planned colocated Rust test modules prefixed `pf_38_s01`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-30-S03, PF-27-S02, PF-18-S01 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Document which opaque-reference/typed-handoff patterns are reused and which financial checks are Corbanu-only; a generic secret-substitution proxy cannot stand in for the narrow financial executor.

- [ ] Define narrow schemas for existing wallet/account-read/order-proposal/sign/broadcast capabilities and a fake venue; no new commercial venue integration or generic shell/transaction execution.
- [ ] Canonicalize venue/account/destination/asset/size/leverage/price/slippage/time/notional/loss fields and bind identity/task/purpose/expiry; reject unknown or unsupported fields.
- [ ] Enforce allow/deny lists, rate/daily notional/loss/leverage caps and cooldowns outside the model with atomic budget reservation; missing reliable state denies.
- [ ] Separate read, construct, approve, sign and broadcast permissions using existing grants/mandates; seeds/private keys never become generic credential capabilities or agent-readable data.
- [ ] Test unauthorized venue/asset, numeric/rounding/overflow ambiguity, stale market/limit state, concurrent caps and no-grant requests against fake balances and custody.
- [ ] Add named `pf_38_s01` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-secret-broker pf_38_s01 && just test -p codex-wallet pf_38_s01`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-38-S01/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
