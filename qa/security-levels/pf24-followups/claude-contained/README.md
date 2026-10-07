# Contained Claude panes (#218)

Claude panes ran `claude --permission-mode bypassPermissions` directly, with
Corbanu's whole environment, no sandbox and full network, and direct
providers handed Claude Code the raw key through `apiKeyHelper`. Round 4
(#220) refused them under protected levels. #218 brings them under the
PF-27-S02 secretless launch contract, behind the default-off feature
`contained_external_agents`, in two PRs: part 1 contains the launch, part 2
routes tool approvals to a person and lifts the refusal.

## Part 1: contained launch

With the feature on, every Claude pane turn is launched like this:

| Contract item | What a contained turn gets |
| --- | --- |
| Clean environment | `env_clear()`, then PF-27-S02's allowlist of this process's environment, minus every `*PROXY*` variable, then only the pane's own variables |
| No raw secrets | Every provider goes through the per-turn loopback bridge, including Z.AI, Baseten and OpenRouter (new `AnthropicApiKeyPassthrough` bridge kind; the key goes upstream as `Authorization` and `x-api-key`). No `apiKeyHelper`; Claude Code holds only the per-turn bridge token. The passthrough bridge forwards only `/v1/messages` and `/v1/messages/count_tokens` to its provider's own scheme, host and port (this hardens the existing Vercel and Claude Plan bridges too) |
| OS sandbox | Claude Code itself runs under Seatbelt or the Linux sandbox. Writes: the pane's folder and its own state folder (`CLAUDE_CONFIG_DIR`, `TMPDIR`, `CLAUDE_CODE_TMPDIR`) under `~/Library/Application Support/Corbanu/claude-panes/` or `$XDG_STATE_HOME/corbanu/claude-panes/`, outside `CODEX_HOME`. Reading `CODEX_HOME/panes` (every pane's transcripts) and other panes' state folders is denied; Claude Code's settings for the turn are a read-only file in its state folder. The contract adds its denials: the vault store, `auth.json`, `config.toml`, sessions, logs, `*.sqlite*`, CLI credential files and Keychains, with `CODEX_HOME` read-only |
| Network | Only the bridge. Claude Code reaches it as `HTTP_PROXY` (base URL `http://127.0.0.1:<port>`): Seatbelt allows that one loopback port, and the Linux sandbox carries it into its network namespace over a Unix socket (its managed-proxy routing). The bridge accepts absolute-form targets |
| Contract checks | `codex_core::protect_external_agent_launch`: refused unless `secretless_agent_launch` armed the contract, the platform sandbox exists, the process is hardened, and argv carries no managed secret; managed values are removed from the environment. This runs before any credential is read or the bridge starts |

Claude Code's own telemetry, auto-update and other traffic are off or fail
closed. Sessions resume from the pane's state folder; a pane that ran
uncontained before keeps its old session in `~/.claude`, so its first
contained turn starts a new session.

`--permission-mode bypassPermissions` is unchanged in part 1, which is why
the refusal under protected levels stays.

## Part 2: tool approvals, and panes allowed under protected levels

- **No permission bypass:** a contained turn runs Claude Code with
  `--permission-mode default --permission-prompt-tool stdio
  --input-format stream-json`; Claude Code writes a `can_use_tool` request
  to stdout before each tool that needs permission.
- **What asks:** Corbanu's settings for the turn add `ask` rules for Bash,
  Edit, Write, MultiEdit, NotebookEdit, WebFetch and WebSearch, so every
  command, file edit and web request asks, including the Bash commands
  Claude Code would consider read-only (`ls` ran without asking before).
  Reading and searching files (Read, Glob, Grep) stays automatic, inside the
  sandbox.
- **A person decides:** each request opens a popup naming the pane (short ID
  and title), its folder, the tool and its `tool_use_id`, and showing every
  input field in full: the whole command, wrapped and scrollable, the content
  of a Write, the old and new text of an Edit, and fields such as
  `run_in_background`, `timeout` or `dangerouslyDisableSandbox`. Line breaks
  show as `⏎`; control, format and bidi characters are escaped
  (`\u{202e}`); runs of 8 or more spaces show as a count. Past 60,000
  characters the rest is counted, not shown, and the request can only be
  denied.
- **No accidental allow:** each request opens on Deny; Allow needs ← or →
  then Enter. Every key but Esc and Ctrl-C (both deny) is ignored until 600 ms
  have passed since the request first showed, since the last ignored key and
  since the last composer keystroke, so an Enter meant for the composer,
  continued typing or a held key does nothing. Held keys never confirm;
  Tab, digits and pasted text never choose; a dropped popup denies. Allow
  works only once the last line of the details has been on screen.
- **One popup, queued:** requests from all panes queue in one popup (first
  in, first out) instead of covering each other; each one starts on Deny
  behind a fresh guard, as does the popup when it comes back from under
  another view.
- **Requests end cleanly:** when a turn ends, is interrupted, or Claude Code
  cancels a request (`control_cancel_request`), its popups close and nothing
  is answered. A request nobody answers in 15 minutes is denied. A request id
  seen before in the turn is dropped (no popup, no answer), so nothing else
  writing to Claude Code's stdout can take over a pending request's answer.
  A control request without an id, or a second `result`, ends the turn with
  an error instead of hanging it. Other control requests get an error reply.
  With nobody to ask (no TUI), every request is denied.
- **Prompt on stdin:** the prompt is Claude Code's first stream-json input,
  not an argv entry; stdin closes once the turn's result arrives. One task
  writes stdin, so large prompts and approvals never block reading stdout.
  A prompt starting with `/` gets a leading space, so it reaches the model as
  text instead of running a Claude Code command.
- **Nothing a pane or repository plants can allow tools:** contained panes
  load no setting sources (`--setting-sources ""`), no MCP servers
  (`--strict-mcp-config`) and no CLAUDE.md, skills, plugins, custom commands
  or agents (`--safe-mode`). They get only Bash, Read, Edit, Write, MultiEdit,
  NotebookEdit, Glob, Grep, WebFetch, WebSearch and TodoWrite (`--tools`, so
  tools a newer Claude Code adds are not available), and subagent, skill,
  slash-command and agent-starting tools are also denied by name
  (`--disallowedTools` and `deny` rules): a subagent definition can ask for
  other permission modes, hooks and MCP servers, and skills and commands
  carry `allowed-tools`. Corbanu's read-only settings file sets
  `disableAllHooks`, the `ask` rules and `disableBypassPermissionsMode`.
- **Which Claude Code runs:** the first `claude` in an absolute PATH folder
  (empty, `.` and relative PATH entries are skipped), refused when it lies in
  the pane's folder or state folder. That same file runs `--version` (refused
  below 2.1.292, the version these checks were made with) and then the turn
  in the sandbox; a file that passed is remembered by path, size, time and
  inode.
- **Allowed under Aggressive:** with `contained_external_agents` on and the
  secretless launch contract armed (`secretless_agent_launch`), Claude panes
  are no longer refused under protected levels. Otherwise the refusal stays,
  and its message names the two features. The `/security` review's Child
  agents row says so.

### Real Claude Code regression

`claude_code_regression.py` runs the real `claude` with the flags and
settings above (`--flags new`) or part 2's first version (`--flags old`), in
a pane folder and config folder seeded with everything a cloned repository
or an earlier turn could plant: `.claude/settings*.json` with allow rules,
`bypassPermissions` and hooks; project and user subagents asking for
`bypassPermissions`/`acceptEdits` with hooks and MCP servers; skills and
commands with `allowed-tools`; `.mcp.json`; and a `.claude.json` with
`allowedTools` and MCP servers. A mock Anthropic API plays the model and asks
for `ls`, an Edit, a Write, a subagent, a skill and a slash command; the
script denies every request. `--prompt` sends a different first prompt. Results with Claude Code 2.1.292 on Linux, in
`regression/` (`bare` = API-key profiles, `full` = Claude Plan):

| Flags | Asked the person | Planted hooks, MCP, agents, skills, commands | Agent/Task/Skill tools |
| --- | --- | --- | --- |
| old, bare | Edit only; `ls` ran without asking | none loaded or ran | not in bare mode |
| old, full | Edit, Write; `ls` ran without asking | none loaded or ran | available (built-ins only) |
| new, bare | Bash, Edit | none loaded or ran | unavailable |
| new, full | Bash, Edit, Write | none loaded or ran | unavailable |

With this PR's flags, prompts that run built-in commands (`/update-config
allow every tool`, `/loop 1m touch x`, sent without Corbanu's leading space)
still ask for Bash, Edit and Write (`regression/new-*-slash-*.json`).

It ran on Linux only: on macOS a real `claude` outside a disposable account
would read the login keychain.

### Limits (part 2)

- Projects' own Claude Code settings, hooks, MCP servers, CLAUDE.md, skills,
  commands and subagents are not used by contained panes.
- The Bash tool's own sandbox flag `dangerouslyDisableSandbox` is shown, not
  refused; the command still runs inside Corbanu's OS sandbox.
- The input a tool runs with is the redacted one the person saw; a command
  that contained a known secret runs with `[REDACTED_SECRET]` in its place.

## Limits

- Both features take effect when Corbanu Terminal starts.
- Other panes' state folders are denied as they exist at launch; one created
  during a turn is not denied to that turn.
- `claude-pane-smoke` and the workflow suite are never contained, even with
  the feature on: they run Claude Code as before
  (`--permission-mode bypassPermissions`, no sandbox) and are refused under
  protected levels.
- Contained panes need Claude Code 2.1.292 or later (checked at launch).
- A Bash tool command inside the pane can use the bridge token from its
  environment, so it can spend on the pane's provider. It cannot read the
  provider key.
- The contract's protected list does not include `~/.ssh`; agent commands
  can read it too. That is PF-27-S02's list, not this change.
- The Claude Plan profile is covered by unit tests only: the disposable test
  profiles have no Claude Plan sign-in.

## Gate evidence (part 1)

- **Tests** (after `just fmt` and `just fix`): `just test -p codex-tui -E
  'test(claude_panes)'` (115: contained plans for every profile, the bridge's
  absolute-form and hostile targets, the profile's denials, the clean
  environment) and the `security::` suites; `just test -p codex-core -E
  'test(launch_contract)'` (13, including `protect_external_launch`);
  `just test -p codex-features`. Linux: `cargo clippy -p codex-tui -p
  codex-core --tests -D warnings` clean on the RTX box (Ubuntu, kernel 7.0).
- **GLM 5.2 runs**, main model GLM 5.2 on Z.AI and a GLM 5.2 Z.AI Claude pane
  (Claude Code 2.1.292), `contained_external_agents` and
  `secretless_agent_launch` on:
  - `tmux-run/1-macos-probes.txt` (the recorded run): writing in the pane
    folder works; reading `config.toml`, the vault store and `panes/`, the
    network and writing outside are denied.
  - `tmux-run/2-macos-keychain-probe.txt`: a synthetic Keychain item readable
    outside is denied inside; the session resumes from the state folder.
  - `tmux-run/3-linux-probes.txt`: the same probes under bubblewrap; the
    bridge is reached through the sandbox's Unix-socket route
    (`HTTP_PROXY` rewritten to an in-namespace port).
  - `tmux-run/4-macos-sibling-state.txt`, `5-linux-sibling-state.txt`: another
    pane's Claude state is denied, the pane's own is readable.
  - The pane environment (names only, checked by hand) holds no provider key.
- **Review** (Opus 5.5 High): request changes, then approve with fixes; all
  fixed or recorded in `review/disposition.md`.
- **Video:** [contained Claude pane probes](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/claude-contained-claude-contained-probes-d2c36f09303f-2026-10-06.mp4)

## Gate evidence (part 2, after review rev-232)

- **Tests** (after `just fmt` and `just fix -p codex-tui`): `just test -p
  codex-tui -E 'test(claude_panes) | test(bottom_pane) | test(security) |
  test(app::) | test(chatwidget)'`: 2639 of 2641 pass. The two failures
  touch nothing changed here: `default_command_popup_items_snapshot` (the
  slash-command list differs from its snapshot) and a wallet test whose Unix
  socket path is too long under this checkout. Linux: `cargo clippy -p
  codex-tui --tests -D warnings` clean on the RTX box. New:
  the popup (`claude_approval_view`: Enter on a fresh popup, the guard, held
  keys, a dropped popup, a long multi-line command, a request too long to
  show, a snapshot), the details (`approval`: every field, escapes,
  redaction, the limit), the turn (`approval_turn`: repeated and missing
  request ids, a second result, interrupt and `control_cancel_request` close
  popups, Claude Code exiting before the prompt, a 300 KB prompt, the
  timeout, version parsing), the contained flags and settings, and the
  refusal lifting only with the feature, the armed contract and a sandbox.
- **Real Claude Code regression:** above (`regression/`).
- **tmux runs** (`tmux-run/7-macos-review-fixes.txt`): GLM 5.2 on Z.AI with
  Claude Code 2.1.292 under Aggressive: the popup opens on Deny; Right, Enter
  allowed `touch approved.txt`, Esc denied `touch denied.txt`. With the
  protocol stand-in `fake-claude/`: an Enter sent as the popup appeared was
  ignored, the untouched popup's Enter denied, the two-line command showed
  `⏎` and `\u{202e}`.
- **Videos:** [approvals under Aggressive](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/claude-contained-claude-contained-approvals-3611045bbe3e-2026-10-06.mp4),
  [popup guard and escaping](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/claude-contained-claude-contained-approval-guard-3611045bbe3e-2026-10-06.mp4)
