#226, #230 and #231 are merged. #232 (#218 part 2) is still open: its CI is green, but the independent review was never finished. Running it now would mean touching your login keychain, which the new rule forbids, so I stopped there.

**PRs**
- **#226** (nested-launch check in the standalone binaries): merged after all checks passed (merge commit `80869748b2`).
- **#231** (Aggressive-homes registry read-only): merged (`da75ee8676`). After #226 landed I merged main into the branch. Two fixes were needed: point `origin_registry_dir` at the new `codex-security-level` crate, and resolve a `launch.rs` conflict with main's new preflight denials, keeping both. Then clippy and 252 TUI tests passed, CI was green and it merged.
- **#230** (#218 part 1, contained Claude panes): merged (`9bac4915f3`). I merged main in and resolved two conflicts, in `tui/Cargo.toml` and `core/src/lib.rs`. Local tests passed (core `launch_contract`, TUI `claude_panes` and `security::`), CI was green, and Linux clippy was clean on the RTX box. The refusal of Claude panes under protected levels stays until part 2 lands.
- **#232** (#218 part 2, tool approvals): open; CI green at `33bc217fd4`.
  - **Fixes this run:**
    - Linux clippy failed because a lock was held across an await point. Writes to Claude Code's stdin now go through a single task, and Linux clippy (`codex-tui` and `codex-core`) is clean.
    - The Windows argument-comment lint failed on three unnamed `true`/`false` arguments in `approval.rs`; they are now named.
  - **Gate so far:**
    - Tests: 162 TUI tests passed, including a fake Claude Code that asks for approvals.
    - Recorded GLM 5.2 run under Aggressive (macOS): "Allow once" created `approved.txt` and Esc denied `denied.txt`. Evidence is `qa/security-levels/pf24-followups/claude-contained/tmux-run/6-macos-aggressive-approvals.txt`.
    - Video: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/claude-contained-claude-contained-approvals-72a3988b59c5-2026-10-06.mp4

**Decision needed from Travis: the #232 review.** The earlier Opus 5.5 High review run was killed with the previous worker and left no verdict. Our reviews run through the installed `corbanu exec` with the Claude Plan provider, which reads Claude Code's login item from the macOS keychain. That may also be what caused the popups. Every other model provider here needs a `corbanu vault auth-helper` call, and the vault keeps its key in your login keychain too. Options:
1. Allow one review run through the installed `corbanu` binary (it reads the keychain).
2. You run the review yourself. The prompt is at `.codex-work/workers-20261002/sec-tui5b-review-232-prompt.md` and the checkout is `worktrees/sec-tui5-review-218a`, already on the #232 head.
3. Merge without an independent review. I don't recommend this.

I recommend option 2, or 1 if the popups are acceptable for one run. #232 should not merge until the review is in.

**Keychain items the earlier probes created** (I didn't delete them):
- A generic password with service `corbanu-sec-tui5-probe`, account `probe`, holding a made-up value. This is the only item a probe deliberately created.
- I found no other creations in the log. However, earlier runs read from your keychain:
  - Claude Code ran with the real `$HOME`.
  - A probe looked up `Claude Code-credentials`.
  - The reviews used the Claude Plan provider.
- Claude Code could have added its own `Claude Code-credentials-<hash>` items. I couldn't check without reading the keychain, so it's worth a look in Keychain Access.
- Leftover contained-pane state folders (not keychain), which you can delete: `~/Library/Application Support/Corbanu/claude-panes/1306335b994f5193/` and `.../3fbc5d1bbf187c9e/`.

**Cleanup:**
- Removed the 231 worktree and its target folder, plus the two finished review worktrees.
- Deleted the test vault stores (the `secrets` folders) from the throwaway homes in `/tmp`, the demo runs and on the RTX box.
- I kept the `sec-tui5-218-20261006` worktree, its target folder and `~/corbanu-rtx/sec-tui5*` on the RTX box for #232.

**Still open:**
- The #232 review.
- The parent-folder-rename limit on the registry is still your decision; it's described in the #231 PR body.
- A display bug that also affects part 1: the pane's final answer text shows up twice, once while streaming and again with the result.
- I made no new keychain evidence: no probes, no throwaway keychain and no runs touching your Claude login.