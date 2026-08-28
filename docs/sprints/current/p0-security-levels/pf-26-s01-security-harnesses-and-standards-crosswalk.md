---
sprint_id: "PF-26-S01"
title: "Security harnesses and standards crosswalk"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-26"
execution_order: 71
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-13-S05, PF-21-S02, PF-23-S03, PF-25-S02, PF-36-S02, PF-41-S02"
created: 2026-08-24
updated: 2026-08-28
---

# PF-26-S01 — Security harnesses and standards crosswalk

## Execution mandate

- Deliver: reproducible compatibility/adversarial harnesses and a checked standards-to-evidence manifest.
- Excludes: true-TUI/live-repository runs, human acceptance, feature docs, and release decision.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-26`
- Reconciliation: [source decisions and archive mapping](../../../plans/security-source-reconciliation.md).
- Product citation: **P0 `/security` levels** — “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.”
- Acceptance advanced: every adopted control and applicable agentic risk maps to code, a prevention control, and a passing test.

## Code boundaries

- OpenClaw adoption reference: [OC-1](../../../plans/openclaw-source-review-2026-08-28.md#oc-1), [OC-2](../../../plans/openclaw-source-review-2026-08-28.md#oc-2), [OC-3](../../../plans/openclaw-source-review-2026-08-28.md#oc-3), [OC-4](../../../plans/openclaw-source-review-2026-08-28.md#oc-4), [OC-5](../../../plans/openclaw-source-review-2026-08-28.md#oc-5), [OC-6](../../../plans/openclaw-source-review-2026-08-28.md#oc-6), [OC-7](../../../plans/openclaw-source-review-2026-08-28.md#oc-7), [OC-8](../../../plans/openclaw-source-review-2026-08-28.md#oc-8), [OC-9](../../../plans/openclaw-source-review-2026-08-28.md#oc-9), [OC-10](../../../plans/openclaw-source-review-2026-08-28.md#oc-10), [OC-11](../../../plans/openclaw-source-review-2026-08-28.md#oc-11) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing compatibility harness: `scripts/{security-level-compat,security_level_compat.py}`; planned adversarial/standards harnesses and deterministic capture-proxy/canary fixtures for PF-26-S02.
- Planned: `qa/release/<version>/security/standards-crosswalk.yaml`
- Tests: Python checker tests plus final Rust workspace security suites

## Preconditions

- [ ] PF-13-S05, PF-21-S02, PF-23-S03, PF-25-S02, PF-36-S02, PF-41-S02 are completed and archived.
- [ ] Candidate version and commit are fixed for evidence collection.
- [ ] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record is linked only to PF-26.

## Remaining

- [ ] Map every accepted architecture recommendation to its owning sprint, final test and artifact; reject missing platform/state-tamper, stale-TLS, capture-sink, closed-world route, control-flow-taint and uncertain-financial-kill coverage.
- [ ] Audit actual parallel diffs and combined-tree commits against declared scopes; rerun Codex seam tests, upstream-drift controls and the independent Permissive baseline. Source evidence and fixture-only contract passes remain separate from qualification.

- [ ] Add an OC-1–11 adoption matrix: pinned source/function and test, Corbanu owner/hook, deliberate difference, candidate case and artifact. Track inspected-only/open-channel concerns separately; the reference's 87 helper tests and 10 observation probes cannot be relabeled as Corbanu passes.

- [ ] Cover every row in the source reconciliation and every PF-13/PF-15–41 contract, including optional lanes' explicit disabled disposition; generate a missing-control report, not just standards coverage.
- [ ] Build hostile web/file/transcript/social/MCP/plugin/child/memory fixtures, canary credentials, fake login/venue and forced-classifier-allow cases; measure end-to-end unauthorized disclosure/action denial separately from detector recall.
- [ ] Attack process/env/helper bypasses, reflected provider responses, DNS/redirect/pinning, download promotion, provenance laundering, migration crashes, financial replay, Sweep and audit/inspector degradation across supported platforms.
- [ ] Re-run local/hosted detector quality/resource/privacy evidence on pinned artifacts; an unavailable optional vendor is visibly disabled, while required local-classifier qualification cannot be skipped.

- [ ] Supply PF-26-S02 with a deterministic local capture proxy and canary scanner: exactly one authorized transport-only bearer occurrence, zero forbidden-surface occurrences, no live provider; test failures and secret-free artifact cleanup.
- [ ] Implement deterministic CLI schemas, nonzero failure exits, candidate/baseline identity, and artifact manifests for all three harnesses.
- [ ] Cover applicable OWASP agentic risks plus the AuthZEN, RAR/token-exchange, CAEP, and AP2 semantics adopted by the plan.
- [ ] Map each row to exact code boundary, automated/adversarial case, expected result, actual result, and artifact.
- [ ] Run against the final formatted candidate; fail on missing rows, missing artifacts, open P0 findings, or baseline drift.
- [ ] Add harness self-tests for malformed manifests, missing binaries, failed commands, stale commits, and incomplete coverage.

## Verification

- [ ] Rust fix/format precedes final affected tests; inspect the exact candidate diff.
- [ ] Harness tests: `python3 -m unittest scripts.test_security_level_compat` plus `python3 -m unittest discover -s scripts/tests -p 'test_security_level_*.py'` for new harnesses.
- [ ] Final affected Rust suites use `cd codex-rs && just test -p <affected-project>`; no direct `cargo test`.
- [ ] Crosswalk checker passes against the versioned release manifest.
- [ ] TUI applicability: none; PF-26-S02 owns interactive evidence.

## Exit evidence

- [ ] Candidate commit, commands, and artifact digests recorded.
- [ ] Crosswalk has no missing applicable control or risk.
- [ ] Output linked under the versioned release security directory.
- [ ] Ledgers reflect reality and the completed record is archived.
