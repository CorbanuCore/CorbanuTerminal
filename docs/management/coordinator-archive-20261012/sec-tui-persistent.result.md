# #187 follow-up 2: persistent approval options need the arrow keys

PR #193 is merged to main as `5ddb70adb9`. It references #174.

## What shipped
- No letter, digit or chord chooses an option that lasts beyond the request. The prompt explains why, and Enter waits until the user moves the highlight with the arrow keys. One-shot options work as before (key, then Enter).
- Lasting options:
  - Exec: prefix "don't ask again" and session scope.
  - Network host: "for this conversation", "allow in the future" and "block in the future".
  - Patch: "don't ask again for these files".
  - Permissions: "for this session".
  - MCP form: "Allow for this session" and "Always allow".
- One-shot options: approve once, decline, and permissions "for this turn" (including with strict auto review).
- The prompt adds a line: "Options that last beyond this request: choose with ↑/↓, then enter." Lasting rows no longer show a shortcut.
- The first highlighted row is always a one-shot option. In MCP approval lists, `j`/`k` no longer move the selection.

## Gate
- **Tests:** approval, MCP and keymap tests pass, 281/281.
  - Full `just test -p codex-tui`: 4214/4242 passed. All 28 failures are environmental or already fail on main.
  - Library tests on the final code: 4144/4155 passed; the 11 failures are the same environmental set.
  - CI is green, and `just fix` and `just fmt` are clean.
- **GLM 5.2 tmux run:** `.codex-work/tui-persist-20261006/logs/tmux-run{1,2}-final.txt`.
- **Opus 5.5 High review:** approved; I applied its should-fix items. Output is in `.codex-work/tui-persist-20261006/review/output.txt`.
- **Video:** https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-174-approval-persistent-needs-arrows-3003d6e0aee0-2026-10-06.mp4

## Open
- **#192 (security):** in the MCP tool approval form, a number key for a one-shot option still submits at once, with no Enter.
- **Confirm with Travis:** permissions "for this turn" is treated as one-shot.
- **Not changed:** the Windows-only "Continue and don't warn again" world-writable warning is a settings confirmation, not an approval prompt.
