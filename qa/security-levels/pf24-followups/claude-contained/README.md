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
  --input-format stream-json`. Claude Code still allows its own read-only
  tools; for anything else it writes a `can_use_tool` request to stdout.
- **A person decides:** each request opens a popup naming the pane, the tool
  and what it would do (the command, path or URL, redacted and shortened):
  "Allow once" or "Deny"; Esc denies, and digits do nothing. The answer goes
  back on Claude Code's stdin. With nobody to ask (the smoke commands), every
  request is denied. Other control requests are refused.
- **Prompt on stdin:** the prompt is Claude Code's first stream-json input,
  not an argv entry; stdin closes once the turn's result arrives.
- **Nothing a pane writes can allow tools:** contained panes load no setting
  sources (`--setting-sources ""`, so no project hooks or allow rules from
  the pane's folder) and no MCP servers (`--strict-mcp-config`); only
  Corbanu's read-only settings file applies. Without this, a hook the pane
  wrote into `.claude/settings.json` ran on the next turn without approval
  (checked with Claude Code 2.1.292).
- **Allowed under Aggressive:** with `contained_external_agents` on and the
  secretless launch contract armed, Claude panes are no longer refused under
  protected levels. Otherwise the refusal stays, and its message names the
  two features. The `/security` review's Child agents row says so.

### Limits (part 2)

- A popup still open when its turn ends (interrupt) stays on screen;
  answering it does nothing.
- Projects' own Claude Code settings, hooks and MCP servers are not used by
  contained panes.

## Limits

- Both features take effect when Corbanu Terminal starts.
- Other panes' state folders are denied as they exist at launch; one created
  during a turn is not denied to that turn.
- `claude-pane-smoke` and the workflow suite stay uncontained and refused
  under protected levels.
- Contained panes need Claude Code with `--bare`,
  `--exclude-dynamic-system-prompt-sections` and stdio permission prompts
  (tested with 2.1.292).
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
