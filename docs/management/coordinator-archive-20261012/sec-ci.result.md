Both PRs are merged, with every check passing before merge.

**1. Linux clippy on PRs: [#240](https://github.com/CorbanuCore/CorbanuTerminal/pull/240)** (merge `45107a95db`)
- I moved the existing Lint/Build job from `rust-ci-full.yml` into a shared `rust-ci-lint-build.yml` without changing its steps. The sccache and cargo-home cache setup is the same, so PR runs reuse main's cache.
- `rust-ci.yml` now runs the gnu dev leg when `codex-rs/` or `.github/` changes, with a 20-minute timeout. A failure blocks `CI results (required)` and therefore `CI required`. The musl dev and musl release legs stay postmerge, and the Linux test suite is still not on PRs.
- I updated `docs/ci.md`, and added 4 tests to `.github/scripts/test_ci_timeout_budgets.py` (13/13 pass).
- **New job wall time on the PR: 7m09s.** The clippy step itself took 6m23s, with a 100% sccache hit rate. This is in the PR body.
- The first postmerge run after the merge passed all three Lint/Build legs through the shared workflow, in 6m48s (gnu), 9m19s (musl release) and 10m57s (musl).
- **Changed check names:** they now end in `/ cargo clippy`, for example `rust-ci / Lint/Build — ubuntu-24.04 - x86_64-unknown-linux-gnu / cargo clippy`. Only `CI required` needs to be in the main ruleset, so nothing there has to change.

**2. Aggressive sandbox decision A: [#241](https://github.com/CorbanuCore/CorbanuTerminal/pull/241)** (merge `0ed626fcc7`)
- The A/B/C choice wasn't written down as open in any repo record. It only appeared in the PF-24-S03 worker's report, plus two notes in the nested-launch follow-up that called the sandbox design "Travis's call."
- I recorded decision A (the `corbanu-aggressive` profile as merged in #186, confirmed by Travis on 2026-10-07) in four places:
  - a new row in the plan's decisions table
  - the archived PF-24-S03 sprint record
  - the PF-24-S03 evidence README
  - those two nested-launch notes

**Anything open**
- Main's postmerge Linux test shards were already failing before my changes (shards 1 and 2 on `9bac4915f3`). This work didn't touch them.
- `.github/workflows/README.md` still says `rust-ci.yml` is manual-only, which is out of date. I left it alone because it was outside this task's scope.
- Both worktrees are removed. No target directories were created.