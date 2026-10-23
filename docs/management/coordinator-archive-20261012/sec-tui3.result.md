All five follow-ups are merged into main, one PR each. Every PR passed its focused tests, had a GLM 5.2 tmux run, got at least one Opus 5.5 High review, and had CI green before merge. One item needs your decision (below); the Aggressive sandbox design is unchanged.

| Item | PR | Status |
|---|---|---|
| 1. A rules file broken after launch stops new threads (#197) | #203 | merged; closes #197 |
| 2a. Response-header values redacted in logs | #205 | merged |
| 2b. Provider URL credentials redacted in error messages | #206 | merged |
| 2c. Startup scrub of old leaked log data | #207 | merged |
| 3. Vault rule refuses wrapped forms | #209 | merged |

**1. Broken rules file (#203).**
- New core setting `strict_rules`, which Aggressive turns on and custom roles can't turn off. With it on, a `.rules` file that doesn't parse makes `/new`, resume, fork and child threads fail with the file and line, instead of starting without the vault rule.
- **Fix from the review:** choosing Permissive in `/security` used to delete the vault rule file while the session was still Aggressive. It now stays until the restart.
- **Before/after:** on main, `/new` started a thread and `corbanu vault list` got the normal approval prompt. Now `/new` is refused; after fixing the file, the command is refused by the vault rule.

**2a. Response headers (#205).** Logged header values are now shown only for a list of harmless diagnostic headers; every other value, `set-cookie` included, appears as `REDACTED`. The review also found tungstenite's websocket handshake (including `Authorization`) reaching the `/feedback` buffer; that is now blocked too. With a real Z.AI turn, `set-cookie` values in the log and logs DB went from 1 to 0. No video, since nothing on screen changes.

**2b. Error messages (#206).** A 401 from a provider whose URL holds a password or key now shows `http://REDACTED:REDACTED@…?key=REDACTED`. The same redaction covers MCP startup errors, the `/model` base URL warning and LM Studio errors.

**2c. Log scrub (#207).**
- **What it does:** runs once per database in the background after startup, so startup isn't delayed. It redacts leaked values in existing rows, then rewrites the database file so deleted rows can't hold them either. `codex-tui.log` and `codex-login.log` are masked in place. I chose a startup scrub over a maintenance command because a process that exits early just continues on the next start.
- **On a copy of a real 51 MB logs DB:** 499 `set-cookie` values before and 0 after; the file shrank to 0.8 MB.
- **Speed:** the worst case, 55 MB where every row leaks, takes 0.2 s to scan in a release build.
- **Fixture DB test:** covers leaked rows, a non-UTF-8 row and deleted rows.
- **Not covered:** a value split across 1 MiB line pieces, lines an older build still running writes afterwards, old free pages in the state DB, and daemon stderr logs.

**3. Wrapped vault commands (#209).**
- Under Aggressive, forbidden rules of two or more words now also match through wrappers, `sh -c` scripts, heredocs, `eval`, `env -S` strings, quote tricks and options before the subcommand. It can only refuse, never allow. The review caught several bypasses in my first two versions, so this took three review passes.
- **Before/after:** on main, `env FOO=1 corbanu vault list`, `sh -c 'corbanu vault list' 2>&1` and `corbanu -c x=1 vault list` all reached the approval prompt. Now all three are refused by the vault rule.
- **Known trade-off:** it over-matches text, so `git commit -m "… corbanu vault …"` is refused too. Names built at run time (`c=corbanu; $c vault`) are out of reach; the unreadable vault store still covers those.

## Gate evidence
- **Tests:** focused `just test` runs after `just fmt` and `just fix`, all passing:
  - #203: 110 core, 29 TUI, 45 app-server;
  - #205: 80, 10, 3 and 241;
  - #206: 80, 41, 241, plus the MCP request-error tests (`codex-exec-server`);
  - #207: 382 `codex-state`;
  - #209: 112 and 21 core, 29 TUI.
- **Reviews (Opus 5.5 High):**
  - #203, #205, #206: approve with fixes;
  - #207: request changes, then approve with fixes;
  - #209: approve with fixes, request changes, then approve with fixes.

  Every finding is fixed or recorded in each PR's `review/disposition.md`.
- **Evidence and videos:** before/after tmux captures are under `qa/security-levels/pf24-followups/` (`issue-197`, `response-headers`, `provider-url-errors`, `log-scrub`, `vault-rule-wrappers`). Videos are on the `demos` prerelease and linked in each PR:
  - `issue-197-aggressive-broken-rules-new-thread`
  - `url-error-redaction-provider-url-error-redacted`
  - `log-scrub-log-scrub-old-secrets`
  - `aggressive-vault-wrapped-aggressive-vault-wrapped`

## Decision for Travis
An agent under Aggressive can start a nested `corbanu exec`, and that run loads your normal config, not Aggressive. The outer sandbox still keeps the vault store unreadable. I didn't confirm whether a nested process can reach the native credential store. This touches the sandbox design you're deciding, so I didn't change it.
- **Option A (recommended):** under Aggressive, refuse agent-launching subcommands of the vault programs (`exec`, `resume`, `app-server`, `mcp-server`, a bare prompt).
- **Option B:** pass Aggressive to child processes through a marker the CLI honours.
- **Option C:** leave it as is and rely on the sandbox.

## Still open
- A second Permissive instance started while an Aggressive one is running removes the vault rule file at its launch. Fits PF-24-S02.
- Not fixed in #205, filed as follow-ups: the rmcp OAuth code logged at DEBUG, and the raw MCP URL in `rmcp-client`'s `auth_status.rs`.
- Rollout files from before #206 may still hold provider URLs in saved errors. Rotate any provider key that was set through `query_params` or the URL.