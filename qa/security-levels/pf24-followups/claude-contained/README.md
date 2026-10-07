# Contained Claude panes (#218)

Claude panes ran `claude --permission-mode bypassPermissions` directly, with
Corbanu's whole environment, no sandbox and full network, and direct
providers handed Claude Code the raw key through `apiKeyHelper`. Round 4
(#220) refused them under protected levels. #218 brings them under the
PF-27-S02 secretless launch contract, behind the default-off feature
`contained_external_agents`, in two PRs. Panes stay refused under protected
levels until the second lands.

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
