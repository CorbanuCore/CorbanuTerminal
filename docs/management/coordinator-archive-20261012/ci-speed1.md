You are a small CI worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding) and
the repo skill /Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home/skills/pushing-ci-changes/SKILL.md.
Problem: the "Ubuntu tmux smoke" job (.github/workflows/tmux-smoke.yml in CorbanuCore/CorbanuTerminal) now takes 45-52
minutes per run (it was ~33 min this morning) against a 60-minute timeout, so it gates every PR and will soon start
timing out. It has no Rust build cache at all; it cold-builds codex-wallet-daemon and codex-cli every run.
1. From recent run logs (`gh run list -w tmux-smoke.yml`, `gh run view --log`), measure where the time goes (each build
   step vs the test step; the step prints "tmux smoke runtime").
2. Add build caching the same way other workflows in this repo already do (sccache as in
   rust-ci-full-nextest-platform.yml, or Swatinem/rust-cache pinned to a commit SHA as the repo pins actions). Keep the
   test behaviour identical (same tests, --retries 0, CORBANU_TMUX_REQUIRED=1). If the test step itself is the slow part,
   find why (e.g. one slow scenario) and report it rather than weakening the test.
3. Show before/after timings from real runs on your PR (a warm-cache second run), and that the job still fails when a
   tmux test fails (reason from the workflow, no need to break a test).
PR to main, merge when green (`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f).
Report: timings before/after, what changed, anything for Travis.
