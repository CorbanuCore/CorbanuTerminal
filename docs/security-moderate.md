# Moderate security level

`/security` → Moderate protects you from instructions hidden in files, tool
output and web or MCP content, and keeps secrets away from agent commands,
while normal work continues. It keeps your own approval, sandbox, network and
web search settings and adds the protections below. The review in
`/security` lists the same rows before you confirm, and the inspector (`i`)
shows what is in force.

Moderate needs the untrusted-content labels (the `source_envelopes` feature):
without them Corbanu refuses every model request under Moderate, so
`/security` doesn't save Moderate until the feature is on.

## What Moderate adds

- **Commands stay in the sandbox.** No agent command runs outside your
  sandbox, even when you approve a request to run outside it. "Retry without
  sandbox?" isn't offered, and commands you allowed with "don't ask again"
  skip the prompt but stay in the sandbox.
- **Protected files.** Agent commands can't read the credential files in your
  home folder (such as `.ssh`, `.aws`, `.gnupg`, `.kube`, `.docker`, `.netrc`,
  `.npmrc` and `~/Library/Keychains`) or Corbanu's sign-in, vault, config,
  history and state. They can't write files that run code later: your shell
  and git start-up files, git hooks and config, the project's `.codex` and
  `.agents`, and Corbanu's own files. Your fresh approval of one command
  lifts this for that command, except for git hooks and config, Corbanu's
  runtime files and what `secretless_agent_launch` keeps; a remembered or
  automatic approval never does. A command you approve can also write the
  project's `.git`, so `git commit` works; its hooks and config, and the
  `commondir`, `modules` and `worktrees` entries through which git would
  read other ones, stay read-only even then. A project's `.env` files aren't covered; Aggressive covers them.
- **Untrusted content.** Tool, MCP, web, agent and memory output reaches the
  model labelled untrusted. After it, protected actions need your fresh
  approval: the vault, credentials, security settings, disclosure, value
  transfer, code you haven't seen, MCP calls that change or send data, and
  tools Corbanu hasn't classified. Cached, remembered or automatic approvals
  don't count, and with approvals off these actions are refused.
- **Model keys.** From the next start, provider API keys are held by the
  credential broker, as under Aggressive. `broker_model_auth = false` in your
  own config keeps it off.
- **Child agents.** Agents started in this Corbanu Terminal (`/spawn` and the
  agent tools) run with Moderate enforced. Claude panes are refused, because
  Claude Code runs outside the sandbox, unless `contained_external_agents` and
  `secretless_agent_launch` run them in it.
- **Nested agents.** `corbanu exec` and `review` started by an agent command
  run with Moderate enforced, also when pointed at another Corbanu home. With
  this Corbanu home, which they can't write inside the sandbox, they run in
  an empty Corbanu home of their own, without your config, sign-in or
  model keys: the agent passes the model and provider with `-m` and `-c`, and
  they reach the model only if agent commands have network access and a key.
  They run headless, nobody approves their commands, and a run without a
  sandbox is refused. Interactive sessions, `app-server`, `mcp-server` and
  credential commands (such as `corbanu vault`) an agent command starts are
  refused.

Not available yet: financial preview and approvals, isolated retrieval,
screened search, brokered browser login, quarantine, the untrusted-content
detectors and Agent Sweep.

## Choosing, restarting and going back

When you confirm Moderate, Core's rules (the sandbox, protected files and
untrusted content) apply at once in every session of this Corbanu Terminal,
and grants and "for session" approvals end. The credential broker starts at
the next start; press `r` to restart right away. Your `config.toml` isn't
changed.

Choosing Permissive or Aggressive from Moderate shows what changes first.
Back to Permissive takes effect when you restart; until then Moderate stays
on. Aggressive applies Core's stricter rules at once when its preflight
passes (the `protected_mode_preflight` feature), and the rest at the restart.
The kill switch (`/security` → `g`) works under every level.

## Limits

- If your own settings turn the sandbox off (full access), commands that the
  protected-file rules don't cover run without one. Choose a sandbox in your
  settings to keep every command in it.
- MCP servers, apps, hooks and `!` shell commands run outside the sandbox, as
  under every level.
- An approved command writes `.git` only where it is a folder in the
  project: not in a linked worktree or submodule (whose `.git` is a file) or
  through a link, so `git commit` doesn't work there under Moderate. On Linux,
  while such a command runs, `.git/commondir` holds an empty file if it was
  missing (removed when the command ends); other git runs on that repository
  fail until then.
- Nested agents are recognised on macOS and Linux only, and on Linux not when
  they run as root or in a user namespace of their own (#826). Corbanu marks
  Moderate by denying agent commands a file in the Corbanu home's `tmp`
  folder, and lists the homes Moderate runs on in the `moderate-homes` folder
  under `~/Library/Application Support/Corbanu` (macOS) or
  `~/.local/state/corbanu` (Linux); under Moderate agent commands can't change
  that list. A launch pointed at another Corbanu home finds its origin through
  that list or your default home (`~/.corbanu`); if the list can't be written
  or your account can't be looked up, such a launch runs at that home's level,
  as before. With a Corbanu home other than the default, a nested launch is
  refused, as before, while `~/.corbanu` or `~/.pfterminal` also stores a
  security level and a vault. A program started outside the sandbox (MCP
  servers, hooks, `!` commands) isn't an agent command and isn't held to
  Moderate. A nested run's own Corbanu home is removed when the run ends, but
  stays in the temporary folder if the run stops on an error (#766).
- A remote app server can't enforce a level chosen here; Corbanu refuses to
  start against one until you choose Permissive.

See [Aggressive security level: protected files](security-aggressive.md) for
the protected-file rules both levels share.
