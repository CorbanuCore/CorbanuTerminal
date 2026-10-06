# Claude panes under Aggressive

Claude panes are outside PF-27-S02's secretless launch and sandbox contract.
Each turn starts `claude -p … --permission-mode bypassPermissions` directly:
- it inherits Corbanu's whole environment, minus a few Claude credential
  variables;
- it runs outside the OS sandbox, with full network;
- direct providers get `corbanu vault auth-helper <label>` as Claude's
  `apiKeyHelper`, so Claude holds the raw key.

Bringing them under the contract needs a clean environment, every provider
behind the loopback bridge, an OS sandbox around `claude` that still reaches
the bridge (on Linux that means a Unix-socket bridge), and an answer for
`bypassPermissions`. That is more than one round, so this change blocks them
and [#218](https://github.com/CorbanuCore/CorbanuTerminal/issues/218) tracks
the full fix.

## Fix

While Aggressive is active or saved, Claude panes are refused with:

> Claude panes are off under security level Aggressive: Claude Code would run
> outside Corbanu's sandbox with your environment and network. Choose
> Permissive in /security and restart to use them; /panes switches back to
> Main.

The check sits at every entry point:
- the new-pane and spawn-worker provider pickers;
- pane creation (all panes, including spawned workers);
- each turn: typed into a pane, sent as a spawn task, or fired by a whip or
  assignment (refused before the fire is counted);
- the process start itself;
- `corbanu claude-pane-smoke` and `claude-pane-workflow-suite`, which read the
  stored level because they have no launch context.

"Saved" counts too: a pane in a session that saved Aggressive could otherwise
rewrite the level file before the restart. Claude Code sign-in and status
checks still run; they take no prompt and run no agent. Permissive is
unchanged.

## Gate evidence

- **Tests:** `just test -p codex-tui security claude_panes orchestrate spawn`
  after `just fmt` and `just fix`. New tests:
  - the refusal applies when active or saved and only then;
  - smoke runs read the stored level;
  - refused creation leaves no pane folder;
  - a refused turn leaves the pane idle at the same turn number;
  - a refused process start never runs the command or writes an audit file.
- **tmux run on GLM 5.2** (`tmux-run/`):
  - Permissive: `/panes` → `+ Claude Pane` opens the provider list, and a turn
    starts `claude` (it then fails on its own `--bare` flag in this
    environment; `A-*`).
  - Aggressive after a restart: `+ Claude Pane` shows the message above
    (`B-aggressive-new-pane.txt`).
- **Review (Opus 5.5 High):** approve with fixes. Findings 1-7 are fixed
  (saved level counts, whips refused before firing, smoke commands, any
  protected level, earlier refusal in the spawn picker and task path, comment,
  gate tests); 8 is this README and the PR.
