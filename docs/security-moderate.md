# Moderate security level

`/security` → Moderate protects you from instructions hidden in files, tool
output and web or MCP content, and keeps secrets away from agent commands,
while normal work continues. It keeps your own approval, sandbox, network and
web search settings and adds the protections below. The review in
`/security` lists the same rows before you confirm, and the inspector (`i`)
shows what is in force: its recent denials include what Moderate's sandbox
refused, and a model request of the session (or one of its agents) that the
credential broker refused, for any reason, shows the broker as degraded until
a later request of the same session goes through. A refused request outside
any session (the model list, the wallet) shows in every session's inspector.

Moderate needs the untrusted-content labels (the `source_envelopes` feature):
without them Corbanu refuses every model request under Moderate, so
`/security` doesn't save Moderate until the feature is on.

## What Moderate adds

- **Commands stay in the sandbox.** No agent command runs outside your
  sandbox, even when you approve a request to run outside it. "Retry without
  sandbox?" isn't offered. Commands you allowed with "don't ask again"
  (also under Permissive) skip the prompt but stay in the sandbox, with the
  protected-file rules below kept, also when they ask to run outside it.
  After untrusted content they don't count for protected actions.
  "Revoke all active authority" (`/security` → `g`) doesn't suspend them.
- **Protected files.** Agent commands can't read the credential files in your
  home folder (such as `.ssh`, `.aws`, `.gnupg`, `.kube`, `.docker`, `.netrc`,
  `.npmrc` and `~/Library/Keychains`) or Corbanu's sign-in, vault, config,
  history and state. They can't write files that run code later: your shell
  and git start-up files, git hooks and config, the project's `.codex` and
  `.agents`, and Corbanu's own files. Your fresh approval of one command
  lifts this for that command, except for Corbanu's runtime files, its
  security state and keys (such as `security_state.json` and
  `source-origin.key`) and what `secretless_agent_launch` keeps; a remembered
  or automatic approval never does. A project's `.env` files aren't covered;
  Aggressive covers them.
- **Untrusted content.** Tool, MCP, web, agent and memory output reaches the
  model labelled untrusted. After it, protected actions need your fresh
  approval: the vault, credentials (a project's `.env` files included),
  Corbanu's own files and security settings, disclosure, value transfer, code
  you haven't seen, MCP calls that change or send data, and tools Corbanu
  hasn't classified. Where approving can't unlock a file (Corbanu's sign-in,
  vault, config and history stay closed to agent commands under
  `secretless_agent_launch`), the prompt says so. Cached, remembered or
  automatic approvals don't count, and with approvals off these actions are
  refused.
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
and grants and "for session" approvals end. Other Corbanu Terminals running
on the same Corbanu home take them within about a second and say so, as they
do with the kill switch. The credential broker starts at the next start
(in each of them); press `r` to restart right away. Your `config.toml` isn't
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
- With full access in your own settings, commands can also write the
  project's `.git`, except its hooks and config (with a sandbox setting `.git`
  stays read-only). A command could leave something there that your own git
  later follows outside the sandbox (#874): a link such as
  `.git/COMMIT_EDITMSG` to a file elsewhere (or to `.git/config`), which your
  next `git commit` writes; a hard link; rebase or cherry-pick steps whose
  `exec` lines, or a merge strategy that isn't git's own, run when you
  continue; a remote in `.git/remotes`; config or a hooks folder where there
  was none; or a replaced `.git`. On macOS and Linux, Corbanu looks at `.git`
  before a session's first such command (and reports links, steps and hard
  links already there), after each one, before each later command and at
  the end of each turn, and warns you about anything new, with the command
  that removes it. It removes nothing itself. Below `objects`, `lfs` and
  `annex` it looks only at their own entries: git reads the files below and
  puts new ones in place, it doesn't write through them. Links inside a
  hooks folder are left alone (git runs hooks, it doesn't write through
  them). Until you remove what it found, and while it can't look at all of
  `.git` (a folder it can't read, or too many entries), approving a command
  in that project doesn't lift the protected-file rules (nor run it outside
  the sandbox), in any session of this Corbanu Terminal. Some setups of your
  own hold them too until you remove them: a layout made of links
  (`git new-workdir`, Android's `repo`), a `config.worktree` that
  `git sparse-checkout` adds, or a hooks folder a hook installer creates
  where there was none. It doesn't watch in between: a process a command
  left running can change `.git` after the last look, even while an approved
  command runs, and what it changes after a turn ends is found at the next
  command, not before you run git yourself. It watches the repositories of
  the folders agent commands run in: not a repository a command reaches
  elsewhere (`git -C`), and not one a command creates and fills in one go
  (nothing was there to compare with). New config and hooks are checked only
  in a repository's own git folder, not in its worktrees' or submodules'. A
  session that works in more than 16 repositories forgets the oldest; its
  next look there reports only links, steps and hard links.
- MCP servers, apps, hooks and `!` shell commands run outside the sandbox, as
  under every level.
- On Linux, where a command you approve can write a Corbanu home's folder
  (another Corbanu home such as `~/.codex`, or this one without
  `secretless_agent_launch`), it can create those of Corbanu's security files
  there that don't exist yet (such as `security_state.json` before a level is
  first confirmed); it can't read or change the ones that exist. On macOS it
  can't create them either.
- Nested agents are recognised on macOS and Linux only, and on Linux not when
  they run as root. Corbanu marks
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
- On Linux agent commands can't create user namespaces (#826): `unshare` and
  `clone` with a new user namespace fail with "Operation not permitted". In
  one of its own a command could open the files the sandbox hides and mount
  over paths, so a nested `corbanu exec` wouldn't be recognised. This covers
  every process in Corbanu's Linux sandbox, but not commands that run
  without it (your settings turn the sandbox off, or an external sandbox),
  contained Claude panes, or a remote Linux executor used from macOS or
  Windows. What needs a user namespace fails under Moderate and Aggressive:
  a sandbox of its own (so the commands of a nested `corbanu exec` don't
  run; Corbanu says why), rootless containers (Podman, Buildah, rootless
  Docker), `bwrap`, Flatpak, Nix's build sandbox, and Chrome or Electron
  with their sandbox on (Puppeteer's default; pass `--no-sandbox`, as
  Playwright does). 32-bit programs stop too. No approval lifts it. Choose
  Permissive to run them.
- A remote app server can't enforce a level chosen here; Corbanu refuses to
  start against one until you choose Permissive.

See [Aggressive security level: protected files](security-aggressive.md) for
the protected-file rules both levels share.
