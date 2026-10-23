I've merged PR #193 into main (merge commit `5ddb70adb9`, references #174). An approval option that lasts beyond the current request can now only be chosen with the arrow keys, then Enter. The full gate passed: tests, the GLM 5.2 tmux run, the review and the video.

**What users see:**
- Pressing the letter, digit or a custom shortcut for a lasting option ("don't ask again", session scope, allow or block a host in the future) chooses nothing. The prompt explains that the arrow keys choose it, and Enter does nothing until the user moves the highlight.
- "Approve once" and "decline" still work with their key, then Enter. Your decisions 1 and 3 are unchanged.
- Lasting options no longer show a shortcut. The prompt adds a line: "Options that last beyond this request: choose with ↑/↓, then enter."
- The highlight always starts on a one-shot option. If a key is bound to both kinds (`d`), it picks the one-shot "No".

**Prompt types covered:** exec (command prefix, session), network host (this conversation, allow or block in the future), patch (these files), permissions (this session), and the MCP tool approval form ("Allow for this session", "Always allow"). In the MCP form, `j`/`k` no longer move the list. MCP prompts in the main approval overlay have no lasting options.

**Gate evidence**
- **Regression tests:** these cover every prompt type:
  - The shortcut letter or digit for a lasting option does nothing, followed by Enter twice. This is also what a pasted letter plus newline looks like on a terminal without bracketed paste.
  - A bracketed paste of the letter plus a newline, then Enter, chooses nothing.
  - Arrows then Enter choose each lasting option.
  - Edge cases: a custom key combination (held or not), a queued request, a request dismissed after the key was pressed, a prompt with only lasting options, and a key shared by both kinds.
- **Test results:**
  - The approval, MCP and keymap tests pass, 281/281, and the snapshots are updated.
  - The full `just test -p codex-tui` ran 4242 tests: 4214 passed and 28 failed. 22 are the environmental failures already recorded for #187. The other 6 are provider-manager tmux tests; one of them fails the same way with main's code, so I'm treating that group as pre-existing.
  - On the final code, 11 library tests fail, all from the same environmental set.
  - CI is green, and `just fix` and `just fmt` are clean.
- **GLM 5.2 tmux run:** in a live session, these all did nothing: `p`, Enter, `p` and Enter sent together, `2` then Enter, and a bracketed paste of `p` plus a newline. Down then Enter saved the rule, and running the same command again didn't prompt. The patch prompt behaved the same way, and `y` then Enter still approved once. Logs are in `.codex-work/tui-persist-20261006/logs/`.
- **Opus 5.5 High review:** approved with no must-fix items. I applied its should-fix items and nits except one: lasting rows still show their row number. Output is in `.codex-work/tui-persist-20261006/review/output.txt`.
- **Video (38 s):** https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-174-approval-persistent-needs-arrows-3003d6e0aee0-2026-10-06.mp4

**Still open**
- **#192 (security, filed):** in the MCP tool approval form, one number key for a one-shot option ("Allow") still submits immediately, with no Enter. This is the #174 kind of bug in a form #187 didn't touch. The reviewer rated it high priority. The fix is to use the main prompt's key-then-Enter pattern there.
- **Decision needed:** should permissions "for this turn" count as one-shot? I treated it that way, so its `y`/`r` keys still work. It does cover every command for the rest of the turn, but it's the narrowest grant offered. I recommend keeping it one-shot.
- **Not changed:** the Windows-only "Continue and don't warn again" warning still accepts digit keys. It's a settings confirmation, not an agent approval, but it's worth checking later.
- I didn't touch the plan's #174 row, since the plan worker owns that text. The summary is in `.codex-work/workers-20261002/sec-tui-persistent.result.md`.