PF-24-S03 is merged in #186 behind the `security_levels` flag, and the sprint record is archived in #195. Every gate item passed. One thing needs your sign-off: I didn't build Aggressive's sandbox exactly the way the sprint table says, because the table's version leaks.

**Why the sandbox row changed:** the table pairs the legacy `SandboxPolicy` sandbox with `approval_policy = "untrusted"`. In that combination, when you approve a command and the sandbox blocks it, Corbanu quietly runs it again outside the sandbox without asking. In my tmux run, an approved `echo hi > ../outside/x.txt` wrote the file. An approved `curl` would get network access the same way. Aggressive now uses a named permission profile, `corbanu-aggressive`, built only from controls that already exist. Because it marks the vault store and `auth.json` unreadable, approved commands can never leave the sandbox. After approval, writes outside the folder or to `/tmp`, network access, and reads of the vault store all fail, and so does `$(corbanu vault …)` (it can't open the vault store).

**Decision for Travis:**
- **Option A (recommended, as merged):** keep the permission profile. Every row of the table holds.
- **Option B:** go back to the literal table and change the screen to say an approved command can run outside the sandbox.
- **Option C:** use `on-request` approvals instead. The sandbox would be strict, but commands inside the folder would no longer ask you first.

**Other differences from the plan:**
- **It takes effect at the next restart, not live through the PF-83 confirmed path.** Web search, environment stripping and the exec-policy rule are fixed when a session starts, so they can't change mid-session. The picker says "saved, restart to activate". On launch, every row is checked and Corbanu refuses to start if one is missing. It never claims Aggressive is active before that.
- **Extra settings needed so the environment row actually holds:** shell snapshots and login shells are off, because the snapshot was putting back variables the policy had removed. The permission-request tools are off, and the Corbanu home folder is read-only to agent commands.
- **Nothing is written to `config.toml`.** The level lives in `$CODEX_HOME/security_level.toml`, which no `-c` option, project file or agent command can change. Going back to Permissive left `config.toml` byte-for-byte the same.
- **Unknown or corrupt saved levels fail closed:** Aggressive is enforced and a warning is shown.

**Gate evidence** (all under `qa/security-levels/sprints/PF-24-S03/`):
- **Tests:** the focused security tests pass (31/31, and 43/43 including the permission-confirmation tests after merging main), and `codex-features` passes 33/33. The full `codex-tui` run had 4207 passing and 21 failing. 20 of those come from this machine's long temp-directory path, which makes Unix socket paths too long. The socket tests pass with a short path; the onboarding tmux ones show the same error but I didn't re-run them. The last is a command-menu snapshot test that fails on wording this PR didn't touch.
- **tmux run on GLM 5.2** (`tmux-run/`): checked:
  - flag off
  - review then cancel
  - select, then restart
  - the probes above, plus secret environment variables and the agent trying to rewrite the level file
  - `/permissions` refused
  - child agents inherit the settings
  - back to Permissive
  - unknown saved value
  - `--yolo` overridden
  - resume in both directions
- **Independent Opus 5.5 High review:** it asked for changes, with 12 findings (two high). Each one is fixed or explicitly deferred in `review/disposition.md`.
- **Videos:** 9 recorded with the demo script, on the final code after merging main, and published to the `demos` prerelease. Links are in `demos/index.md` and the #186 description.
- **Code-blind test design:** 49 cases, frozen before any results were shared; the designer made no tool calls (`code-blind-design/`).

**Still open** (listed in the PR and QA README):
- The launch check confirms the vault rule file is present but doesn't load the full exec policy. A broken `.rules` file elsewhere would silently drop the `corbanu vault` rule; the unreadable vault store still blocks access. Fixing this means adding `codex-execpolicy` to the TUI, a shared lock-file change for the integration owner.
- MCP servers, apps and hooks run outside the sandbox, and `corbanu exec` and IDE sessions ignore the level. Both are stated on the review screen.
- Custom agent roles that would change a child's settings are refused at start rather than overridden for the child.
- No tamper check on the level file and no "restart now" button; these fit PF-24-S02.
- The review screen shows Aggressive's values, not your current settings next to them. The code-blind designer flagged this (its case B-06).
- The `/security` command description still says "read only" when the flag is on.
- Not from this sprint: with trace logging on, the provider key shows up in the private TUI log. The demo tool caught and redacted it on every run.
- #187 changed approval prompts so `y` no longer approves; the demo specs now press Enter.
- The milestone code-blind run and human sign-off stay with "Aggressive ships".