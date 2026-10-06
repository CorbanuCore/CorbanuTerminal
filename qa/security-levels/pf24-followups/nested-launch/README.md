# Nested agent launches under Aggressive

Before this change, an agent command under Aggressive could start
`corbanu exec` (or `resume`, `app-server`, `mcp-server`, a new session). The
child loaded the user's normal config, not Aggressive; only the outer OS
sandbox still applied.

## Fix (Travis's decision, 2026-10-06)

**Setting.** `nested_agents` in `$CODEX_HOME/security_level.toml`:
- `refuse` (default);
- `pass`: the nested agent runs with Aggressive enforced.

Only a person can change it: in `/security`, open the Aggressive review and
press `n`. It has the same protection as the level: agent commands cannot
write the file, and no `-c`, profile or project file reads it. Saving
Permissive resets it to `refuse`. An unknown value enforces Aggressive and
refuses.

**What counts as nested.** A Corbanu home is the origin of a nested launch
when it enforces Aggressive and the launching `corbanu` can neither write it
nor read its vault store. That is exactly what Aggressive's profile does to
agent commands. A person's own launch, or a Permissive sandbox, can read the
vault store; Aggressive launches now create its (empty) folder so the denial
is observable.

The candidate homes are:
- the one in `CORBANU_SECURITY_ORIGIN`, which Aggressive now sets in every
  agent command's environment and verifies at launch;
- the launching process's own home;
- the account's default homes, found through the account database, not
  `$HOME`;
- every home registered by an Aggressive launch, under
  `~/Library/Application Support/Corbanu/aggressive-homes` (macOS) or
  `~/.local/state/corbanu/aggressive-homes`.

Dropping the variable or pointing `CODEX_HOME` elsewhere therefore does not
help. A home the agent wrote itself is writable, so it is never an origin. If
several origins apply, `refuse` wins. An Aggressive session that saved
Permissive keeps its rule file until it restarts, and still counts.

**Per subcommand:**

| Subcommand | `refuse` | `pass` |
| --- | --- | --- |
| `exec`, `review` | refused | Aggressive enforced and verified |
| new session, `resume`, `fork` | refused | refused: the agent that started it would answer its approval prompts |
| `app-server`, `mcp-server`, `debug app-server`, `remote-control`, `app`, `exec-server`, `cloud`, `telegram`, `claude-pane-*` | refused | refused: the client chooses each agent's sandbox and approvals, or the agent runs outside Corbanu's sandbox |
| `vault`, `tasknode`, `internal-claude-oauth-token`, `internal-claude-login-health`, `internal-gpu-endpoint-token`, `internal-gpu-controller` | refused | refused: these read stored credentials |

`pass` for `exec` and `review`:
- adds the Aggressive overrides, protecting both the origin and the child's
  home;
- forces `untrusted` approvals; exec cannot ask anyone, so commands that need
  approval do not run;
- ignores `--dangerously-bypass-approvals-and-sandbox`, `--sandbox`,
  `--add-dir`, `--ignore-rules` and `--dangerously-bypass-hook-trust`, and
  says so;
- verifies every Aggressive row before the run starts.

`--dangerously-bypass-hook-trust` is now also ignored by a normal Aggressive
launch.

**Limits:**
- **`pass` cannot do useful work today.** Inside a real Aggressive agent
  command, the Corbanu home is read-only and the network is off. A nested
  `corbanu exec` starts with Aggressive enforced, then stops at
  "Operation not permitted" (tmux run, step 3). It could only work if the
  Aggressive profile let it write a home and reach the model. That is the
  sandbox design, which is Travis's call.
- No account database (some containers) and Windows: only the variable and
  the process's own home are checked.
- Only `corbanu` builds with this change check. The standalone `codex-exec`
  and `codex-tui` binaries do not.

## Gate evidence

- **Tests:** after `just fmt` and `just fix`:
  - `just test -p codex-cli --test nested_launch` (5): both modes, forged
    homes, a person's own launch, and a nested `exec` with `pass` against a
    mock model. In that run, its vault read fails in the sandbox and its
    network and outside-write commands are never run.
  - `just test -p codex-tui security` (54): detection (writable, Permissive
    sandbox, Aggressive sandbox, forged pass home, saved Permissive), the
    registry, the setting, the picker, and a nested child config that cannot
    read either vault store, write outside or use the network.
  - `just test -p codex-exec` (130).
- **tmux run on GLM 5.2** (`tmux-run/`, real nesting: GLM ran the commands
  under Aggressive):
  1. `refuse`: `corbanu exec` and, with the variable removed,
     `env -u CORBANU_SECURITY_ORIGIN corbanu exec`, `corbanu mcp-server` and a
     new session were all refused.
  2. `/security`, `n`, save: the file reads `nested_agents = "pass"`.
  3. `pass`: `corbanu exec -s danger-full-access` reported that Aggressive was
     enforced and `--sandbox` ignored, then failed on the read-only home.
  4. A child started outside a sandbox but pointed at a `pass` origin (an
     earlier build in which nested sessions could pass): its approved
     commands could not read either vault store, reach example.com or write
     `../outside`, and `/permissions` was refused. GLM first declined to run
     the probes (`model-refusal*.txt`), so they ran through a script.
- **Review (Opus 5.5 High):** see the PR's `review/disposition.md`.
