You are a CI worker for CorbanuCore/CorbanuTerminal. Read the "Working rules" in
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (they apply; this is not a security sprint).

Travis's decisions (2026-10-07):
1. CI coverage option 2: add the Linux clippy/lint job to PR checks (fast, ~10-15 min target). Today PR checks don't
   run Linux clippy; only postmerge-ci (rust-ci-full "Lint/Build — ubuntu-24.04 x86_64-unknown-linux-gnu") does, so
   Linux-only lint errors (e.g. cfg(target_os = "linux") code, redundant_closure, expect_used) land on main. Reuse the
   existing rust-ci-full Lint/Build job definition (gnu only is enough; musl stays postmerge), wire it into the PR
   workflow (blocking-ci / rust-ci) so it's part of the required "CI results" aggregation, use the existing sccache /
   cache setup, and keep it within ~15 minutes. Don't add the Linux core test suite to PRs. Update docs/ci.md and any
   CI timeout-budget tests (.github/scripts/test_ci_timeout_budgets.py). Prove it on the PR itself (the new job must
   run and pass) and show its wall time in the PR body.
2. Record, in a separate small docs PR, that the Aggressive sandbox design is decision A (the `corbanu-aggressive`
   permission profile, as merged in #186), confirmed by Travis, in the PF-24 records/plan where the A/B/C choice is
   mentioned as open.
Merge each PR when all checks pass. Report: PR links, new job wall time, anything open.
