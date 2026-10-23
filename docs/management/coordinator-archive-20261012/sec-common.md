# Security program: shared rules (Travis's decisions, 2026-10-06)

Repo: CorbanuCore/CorbanuTerminal. Main is the base. Plan: docs/plans/active/p0-security-levels.md,
sprints: docs/sprints/current/p0-security-levels/.

## Decisions (binding; they override older plan prose)
1. PF-83-S01 closes now with its real results recorded (pass/fail/blocked per case). Open defects it found move
   to the TUI lane as bugs; they do not keep PF-83 open.
2. Ship the `/security` picker early behind a feature flag. Permissive = exactly today's behaviour.
   Aggressive = built from controls that already exist (read-only or workspace sandbox, approvals always on,
   no network, no vault access). Moderate is added as its protections land.
3. Core program is 19 sprints: PF-27-S04, PF-27-S02, PF-28-S01, PF-28-S02, PF-33-S01, PF-33-S02, PF-29-S01,
   PF-29-S02, PF-30-S01, PF-30-S02, PF-30-S03, PF-23-S01, PF-23-S02, PF-23-S03, PF-24-S02, PF-25-S01, PF-25-S02,
   PF-41-S01, PF-13-S07, plus the new picker sprint. Everything else (PF-32, PF-35, PF-36, PF-37, PF-38, PF-39,
   PF-40, PF-41-S02) moves to a separate "P1 security hardening" plan. PF-41-S01 drops its deps on PF-32-S06,
   PF-37-S02 and PF-40-S03 (the inspector shows those controls as "not available").
4. Up to 3 security sprints run in parallel when their write scopes don't overlap. Lanes:
   - broker: PF-27-S04 -> PF-27-S02 -> PF-28-S01 -> PF-28-S02
   - untrusted content: PF-30-S01 -> PF-30-S02 -> PF-30-S03 -> PF-23-S01..S03
   - TUI: picker sprint -> PF-24-S02 -> PF-25-S01 -> PF-25-S02
5. Per-sprint gate (lighter): focused tests via `just test`, a tmux functional test driven by GLM 5.3 Flash
   (`-m glm-5.3-flash -c model_provider="zai"`; Travis 2026-10-10: GLM 5.2 burns Z.AI credit, use it only with a recorded reason), one independent review (Opus 5.5 High subagent), and SHORT VIDEOS
   demonstrating every core feature of the sprint end to end, recorded with the demo-video SOP: qa/demos/README.md
   (`python3 scripts/demo_video.py record qa/demos/specs/<demo>.toml --bin <corbanu> --sprint <ID> --publish`).
   Merge behind the feature flag when that gate passes. The full isolated code-blind VM run and human sign-off
   happen only at milestones: Aggressive ships, Moderate ships, flag removal.

## Working rules
- Work in your own new worktree under /Volumes/CorbanuDrive/Corbanu/worktrees/ off origin/main. Never use or stage
  anything in worktrees/management-workstreams-20260911.
- Rust: pinned toolchain from .codex-work/integration-build-20260922a/dev-build.sh (copy its PATH/CARGO_HOME), your own
  CARGO_TARGET_DIR per worktree (never shared: a shared target produced binaries with another worktree's code)
  under /Volumes/CorbanuDrive/Corbanu/.codex-work/targets/<worktree-name>, and `-j8` (other lanes build
  at the same time). Run `just fmt` and `just fix -p <crate>` before the final tests.
- Open a PR to main per sprint (or per bounded bug fix). Merge commits only: when all checks pass, merge with
  `gh pr merge N --merge --match-head-commit <sha>`. Never force-push; never `rm -f`. Never print secrets
  (vault: `X="$(corbanu vault auth-helper <label>)" cmd`). Scan the diff for key material before committing.
- Keep docs short. Update only your sprint's file and its row in the index; the plan worker owns the plan text.
- Use the spawn tool for subagents if available; otherwise a separate `corbanu exec` session.
- If you need a product decision from Travis, stop and say exactly what, with options and a recommendation.
- Final message: what shipped (PRs, merged or not), gate evidence (tests, tmux run, review verdict, video paths),
  and anything open.
- CodeQL no longer runs on pull requests (Travis, 2026-10-06); it runs on main and nightly. Ignore any stale
  cancelled or pending "Analyze (...)" / "CodeQL" check on a PR when deciding whether checks pass.
- PR CI doesn't run Linux clippy or the full Linux test suite (only postmerge does), so cross-PR and Linux-only
  breakage lands on main. For Linux-only code (Landlock, seccomp, cfg(target_os = "linux")), check it compiles cleanly
  with clippy in Docker or on the RTX box before merging.
- Disk hygiene (CorbanuDrive filled up on 2026-10-06): when your PR is merged and you are done with a worktree,
  remove your own CARGO_TARGET_DIR with `rm -r <dir> < /dev/null` (never `rm -f`) and remove the worktree with
  `git worktree remove`. Don't create extra target dirs per review round; reuse one per worktree. On the RTX box,
  same rule for anything under ~/corbanu-rtx/ you created.
- NEVER touch the user's real macOS login keychain (OS keyring). On 2026-10-06 a demo run with a freshly built binary
  made the vault store its key in the real login keychain and Travis got repeated keychain password popups. Every
  run you start (tests, tmux, demos, app-server helpers) must set CORBANU_TEST_NO_NATIVE_KEYRING=1 and use a disposable
  CODEX_HOME/CORBANU_HOME/PFTERMINAL_HOME. If a code path ignores that variable and would reach the OS keyring, stop
  and fix the code path (or skip the live run) instead of running it.
- Clarification of the keychain rule: running the INSTALLED signed binary (`$HOME/.local/bin/corbanu`, e.g.
  `corbanu exec` reviewers on claude-plan, `corbanu vault auth-helper` for real provider keys) is fine: that's how every
  worker runs and it doesn't prompt. What's forbidden is freshly built debug/test binaries (or anything they start)
  reaching the real keychain, and creating/probing keychain items.

## Disk hygiene addendum (2026-10-08)
Never delete a worktree or directory referenced by the live owner settings (initiative owner / manager config,
e.g. worktrees/owner-loop-*). Check the live owner settings before any cleanup.

## Linux box (glitch; Travis 2026-10-12: glitch is the Corbanu Linux box)
Linux work runs on **glitch** only: always `ssh glitch` (ambient@100.83.137.68). Never use 100.99.88.49 (the RTX box) for
Corbanu work; the old ~/.ssh/config redirect from it to glitch was removed on 2026-10-12, so an older brief that says
`ambient@100.99.88.49` means `glitch`. Same layout: ~/corbanu-rtx/
(integration checkout, build.sh, bin/). glitch is smaller (16 cores, 30 GB RAM + 32 GB swap) and shares the machine with
an Ambient vLLM service: use at most `-j10` for cargo, run at most one heavy build/test per worker, and never more than
two workers building there at once. Rust 1.95.0 is installed via rustup (`. ~/.cargo/env`); the system /usr/bin/cargo is
1.75 — always source ~/.cargo/env first. Delete your target dirs when done; disk has ~700 GB free, keep it that way.
Do not touch anything outside ~/corbanu-rtx/ on glitch.

## Merging main into a PR branch (2026-10-12, CI is the bottleneck)
`main` has no "branch must be up to date" rule. Do NOT merge origin/main into an open PR just because main moved: every push
restarts ~40 CI jobs. Merge origin/main only when the PR is CONFLICTING/DIRTY, when your change depends on something newly
merged, or when a check failed because of something main has since fixed. Otherwise wait for green and merge.
