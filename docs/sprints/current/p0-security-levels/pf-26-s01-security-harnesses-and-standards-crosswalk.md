---
sprint_id: "PF-26-S01"
title: "Security harnesses and standards crosswalk"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-26"
execution_order: 21
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-13-S05, PF-21-S01, PF-23-S03, PF-25-S02"
created: 2026-08-24
updated: 2026-08-25
---

# PF-26-S01 — Security harnesses and standards crosswalk

## Execution mandate

- Deliver: reproducible compatibility/adversarial harnesses and a checked standards-to-evidence manifest.
- Excludes: true-TUI/live-repository runs, human acceptance, feature docs, and release decision.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-26`
- Acceptance advanced: every adopted control and applicable agentic risk maps to code, a prevention control, and a passing test.

## Code boundaries

- Planned: `scripts/{security-level-compat,security-level-adversarial,security-level-standards-check}`
- Planned: `qa/release/<version>/security/standards-crosswalk.yaml`
- Tests: Python checker tests plus final Rust workspace security suites

## Preconditions

- [ ] PF-13-S05, PF-21-S01, PF-23-S03, and PF-25-S02 are completed and archived.
- [ ] Candidate version and commit are fixed for evidence collection.
- [ ] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record is linked only to PF-26.

## Remaining

- [ ] Implement deterministic CLI schemas, nonzero failure exits, candidate/baseline identity, and artifact manifests for all three harnesses.
- [ ] Cover applicable OWASP agentic risks plus the AuthZEN, RAR/token-exchange, CAEP, and AP2 semantics adopted by the plan.
- [ ] Map each row to exact code boundary, automated/adversarial case, expected result, actual result, and artifact.
- [ ] Run against the final formatted candidate; fail on missing rows, missing artifacts, open P0 findings, or baseline drift.
- [ ] Add harness self-tests for malformed manifests, missing binaries, failed commands, stale commits, and incomplete coverage.

## Verification

- [ ] Rust fix/format precedes final affected tests; inspect the exact candidate diff.
- [ ] Harness tests: `python3 -m unittest discover -s scripts/tests -p 'test_security_level_*.py'`.
- [ ] Final affected Rust suites use `cd codex-rs && just test -p <affected-project>`; no direct `cargo test`.
- [ ] Crosswalk checker passes against the versioned release manifest.
- [ ] TUI applicability: none; PF-26-S02 owns interactive evidence.

## Exit evidence

- [ ] Candidate commit, commands, and artifact digests recorded.
- [ ] Crosswalk has no missing applicable control or risk.
- [ ] Output linked under the versioned release security directory.
- [ ] Ledgers reflect reality and the completed record is archived.
