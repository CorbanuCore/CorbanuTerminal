PF-41-S01 is shipped: PR #259 merged into main as `8b3ac213c4`, behind the `security_levels` flag. All 36 CI checks passed. The Linux credential canary hung for about two hours installing dependencies, so I cancelled it and re-ran it, and it passed.

**What it does**
- Press `i` in `/security` to open a read-only inspector. Esc goes back, `r` refreshes, and the arrow keys scroll. It is only offered alongside the picker, so the flag-off view is unchanged.
- It keeps three kinds of fact apart:
  - **configured:** the saved level and Core's config layers;
  - **resolved:** the launch check, the session's config and the platform sandbox;
  - **observed:** Core's live policy, each agent's level (including stricter child agents), grants, taint, runtime controls and recent denials.
- The status badge only turns green when nothing is degraded, every required control was observed or checked at launch, and the facts are under 30 seconds old. Otherwise it reads Partial, Degraded or Blocked.
- MCP servers, hooks, `!` commands and app-server `command/exec` show as "not contained". PF-32, PF-37, PF-34 and PF-40 show as "not available". The nested-agents setting shows refuse (the default) or pass.
- Nothing secret is shown: no grant ids, and no paths or commands from denials.

**Gate evidence**
- **Focused tests:** `just test -p codex-core pf_41_s01` 6/6 and `just test -p codex-tui pf_41_s01` 14/14.
- **Wider security tests:** core 324/324, TUI 286/287, app-server-client 37/38. Both failures are in tests this sprint didn't touch. This machine's temp-folder path is very long: one picker test checks for a phrase that gets wrapped across lines, and one socket path goes over the OS length limit. Both pass with `TMPDIR=/tmp`.
- **Linux clippy** (`-D warnings`, core, tui, app-server-client) on the RTX box: clean at all three code commits, including the final `99b5308309`.
- **tmux run on GLM 5.2:** full inspector after Aggressive and a restart, taint plus a declined git-hook write, and a corrupted security-state file showing Blocked, then recovered after confirming a level and restarting. No model refusals. The run found three bugs, now fixed:
  - pressing Esc on an approval interrupted the turn before the decline was recorded, so it never appeared as a denial;
  - a tall inspector lost its header off the top of the screen;
  - a Core level raised before restart wrongly read as "enforced".
- **Review (Opus 5.5 High):**
  - Round 1: REQUEST CHANGES, with two blocking issues, both fixed: the badge could claim Aggressive while Core enforced less, and stale facts could stay green because nothing redrew the screen.
  - Round 2: APPROVE WITH FIXES, all five applied.
  - Both rounds are in `qa/security-levels/sprints/PF-41-S01/review/`.
- **Videos** (GLM 5.2, commit `99b530830976`), listed in `qa/demos/index/PF-41-S01.md`:
  - [live-aggressive](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-41-s01-pf41-inspector-live-aggressive-99b530830976-2026-10-07.mp4)
  - [taint-denial](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-41-s01-pf41-inspector-taint-denial-99b530830976-2026-10-07.mp4)
  - [failure-recovery](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-41-s01-pf41-inspector-failure-recovery-99b530830976-2026-10-07.mp4)
- **Housekeeping:** I removed my worktree, target folder and temporary files, and my worktree and target on the RTX box. I didn't edit any `bottom_pane/security_level_picker*` file.

**Open items**
- **Possible bug for the TUI lane:** in a fresh home under Aggressive, the existing session "Vault" row says the vault store is readable to agent commands. That row comes from PF-24 code I didn't change.
- **Merge conflict for sec-tui7:** their PF-25-S01 branch adds `held()` in `aggressive.rs`. Mine is named `held_for_inspector` and placed in a different spot to avoid a clash. They will still get small conflicts on one-line edits in `core/lib.rs`, `security/mod.rs`, `app-server-client/lib.rs`, and possibly `orchestrator.rs`.
- **Not run here:** the full core and TUI suites (they run after merge), the code-blind run, and human sign-off, which belongs to the flag-removal milestone. The sprint record stays in current as merged behind the flag, with these items listed.
- **Limits:** broker health (network credential broker and model key broker) isn't probed, so those rows are never green. Launch-contract denials show as "this process" rather than a session. A permission-hook refusal reads "declined (by you or a permission hook)".