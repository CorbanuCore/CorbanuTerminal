---
sprint_id: "PF-33-S03"
title: "Pure destination-policy contract"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-33"
execution_order: 18
owner: "Codex browser/retrieval lane"
parallel_lane: "browser-retrieval"
write_scope: "codex-rs/network-proxy/src/destination_contract.rs, codex-rs/network-proxy/tests/, codex-rs/network-proxy/BUILD.bazel, qa/security-levels/sprints/PF-33-S03/, docs/sprints/current/p0-security-levels/pf-33-s03-destination-policy-contract.md"
integration_gate: "Jim Ricketts receives the PF-33-S03 candidate, audits the literal scope, runs fix/format/tests and governance on the combined tree, then archives the sprint and returns the slot."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-browser-retrieval"
branch: "feat/p0-security-browser-retrieval-pf33"
base_commit: "67cdfaf0fce7cdb4e19036306fb5ca3129192968"
depends_on: "none"
created: 2026-08-28
updated: 2026-08-30
---

# PF-33-S03 — Pure destination-policy contract

## Execution mandate

- Deliver: Freeze pure URL, address-set and redirect decisions for later connection enforcement.
- Excludes: protected-mode activation, adjacent feature implementation and Permissive behavior changes.

## Plan linkage

- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md#pf-33).
- Feature: `PF-33`.
- Product citation: **Reconciled security scope — TO BUILD** — “Unknown or unsupported protected paths fail visibly rather than falling back to raw secrets or unscreened execution.”
- Acceptance advanced: [accepted architecture refinements](../../../plans/security-architecture-refinements-2026-08-28.md).
- Source input: [OpenClaw source review](../../../plans/openclaw-source-review-2026-08-28.md) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; reference behavior is not candidate acceptance.

## Code boundaries

- Planned: codex-rs/network-proxy/src/destination_contract.rs; codex-rs/network-proxy/tests/destination_contract.rs
- Existing integration paths are read-only until the named consumer sprint; shared manifests/lockfiles require serialized ownership.

## Preconditions

- [x] Plan active; dependencies in front matter are `none`, and PF-31-S04 completed and archived before this slot rotation.
- [x] Named owner and exact plan-matching worktree/branch/base assigned; both governance checkers pass before implementation.
- [x] Root and `codex-rs/AGENTS.md` read; disjoint write scope and receiving integration gate reserved.

## Done

- [x] Bounded preparation/foundation mandate created from the accepted review; no implementation or platform acceptance claimed.
- [x] Frozen `pf33-destination-policy/v1` pure normalization, policy-polarity, DNS-answer, private-service and redirect/replay decisions without runtime registration or socket access.
- [x] Added standalone deterministic table/property coverage for URL ambiguity, IDNA, suffix/trailing-dot/path boundaries, unusual IPv4, mapped/translation/tunnel IPv6, reserved names, answer-set polarity and malformed policy.
- [x] Recorded a hashed versioned representative fixture and the isolated 239/239 network-proxy result under `qa/security-levels/sprints/PF-33-S03/`; the suite executes every frozen fixture case.
- [x] Completed Claude Opus 5 Max Computer Use review (`PASS`) and Codex GPT-5.5 Autoreview (clean, 0.86 confidence); resolved accepted findings in scope and retained immutable evidence.
- [x] Integration owner received the candidate, audited the literal scope, reran the ordered Rust and governance gates on the combined tree, archived the sprint, updated plan/navigation and returned the active lane slot.
- [x] Reopened after a trace-backed tmux/Corbanu Terminal/Claude Opus 5 Max review found a Bazel compile-data break and two permissive type-construction gaps.
- [x] Replaced the ambiguous optional public rules with explicit `PublicScope`, made normalized destinations and decisions non-forgeable, and retained read-only accessors.
- [x] Declared the source as Bazel test compile data, added a package-local executable fixture, froze its hash and retained a byte-identical QA evidence copy.
- [x] Final same-session Opus 5 Max follow-up returned `CLEAN`; all three original PF-33 findings and the follow-up ledger contradiction are closed.
- [x] Remediation commit `80a2469e401066ebaf04d95ba603ab68cb341854` passed the final integration-owner gates and the sprint was re-archived.

## Remaining

- None.

## Verification

- [x] Ran `just fix -p codex-network-proxy`, `just fmt`, then `just test -p codex-network-proxy` on the isolated and canonical trees: 239 passed, zero skipped across three binaries; sixteen destination-contract tests.
- [x] Ran standalone deterministic table/property tests with synthetic address sets; proved empty/absent/wildcard/private polarity and bounded normalization.
- [x] TUI applicability: none for this pure preparation/foundation boundary; user-facing consumer sprints retain true-TUI proof.
- [x] Verified the source has no socket/client call and remains absent from `src/lib.rs`, the module tree and the runtime graph; the Bazel integration target declares it only as test compile data, and fixture-only preparation exposes no route or profile.
- [x] Receiving integration owner reran plan/sprint governance and whitespace checks on the completed combined archive tree.
- [x] Final-tree ordered `just fix`, `just fmt`, `just test -p codex-network-proxy` sequence passed with all 239 network-proxy tests, including sixteen destination-contract regressions.
- [x] Focused Bazel target `//codex-rs/network-proxy:network-proxy-destination_contract-test` passed with declared source and fixture compile data.
- [x] Final tmux/Corbanu Terminal/Claude Opus 5 Max corrected-candidate verdict was `CLEAN` with no actionable P0-P3 finding.

## Exit evidence

- [x] Reviewed source commit, source/test/fixture hashes, owner reviews and final-tree outputs recorded under `qa/security-levels/sprints/PF-33-S03/`.
- [x] No SSRF-prevention claim; PF-33-S01/S02 retain real resolver, connected-peer, pool/proxy and alternate-egress qualification.
- [x] Scope audit, integration handoff, combined-tree reruns and completed archive transition recorded; all ledgers updated.
- [x] Exact remediation commit, post-archive transition and final-tree hashes are recorded under `qa/security-levels/sprints/PF-33-S03/`.
