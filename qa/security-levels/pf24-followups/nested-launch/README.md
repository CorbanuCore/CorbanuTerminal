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
help. An agent can make a home of its own look protected, but real origins are
checked first and `refuse` wins when several apply. An Aggressive session that
saved Permissive keeps its rule file until it restarts, and still counts.
Probe errors other than "allowed" (for example no free file descriptors)
count as denied. If the account lookup fails while the variable is set, the
launch is refused.

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
- **No account database** (some containers) **and Windows:** only the
  variable and the process's own home are checked.
- **Workspace is your home folder:** an agent can then delete registry
  entries. Aggressive already warns about this workspace at launch. Closing
  it means making the registry read-only in the Aggressive profile, which is
  the sandbox design (Travis's call).
- **A second, Permissive launch of the same home** while an Aggressive
  session still runs removes the rule file and registry entry. This was
  already open from round 3 and fits PF-24-S02.
- **Under `pass`,** the nested run's hooks, MCP servers and trust records come
  from whichever home the agent chose. Only the outer sandbox contains them.
- `corbanu` checks since #219; the standalone `codex-exec`, `codex-tui`,
  `codex-app-server` and `codex-mcp-server` check too since
  [standalone-nested](../standalone-nested/README.md).
- **Older builds** read a file with `nested_agents = "pass"` as unreadable.
  They then enforce Aggressive and show a warning.
- **Debug builds** honour `CORBANU_TEST_ACCOUNT_HOME` in place of the account
  database, so tests never read the operator's profile.

## Gate evidence

- **Tests:** after `just fmt` and `just fix`:
  - `just test -p codex-cli --test nested_launch` (6): both modes, forged
    homes, the registry with the variable and home changed, a person's own
    launch, and a nested `exec` with `pass` against a
    mock model. In that run, its vault read fails in the sandbox and its
    network and outside-write commands are never run.
  - `just test -p codex-tui security` (54): detection (writable, Permissive
    sandbox, Aggressive sandbox, forged pass home, saved Permissive), the
    registry, the setting, the picker, and a nested child config that cannot
    read either vault store, write outside or use the network.
  - `just test -p codex-exec` (130).
- **tmux run on GLM 5.2** (`tmux-run/`; real nesting: GLM ran each command
  under Aggressive and a person approved it):
  1. `refuse` (`1-refuse.txt`): refused, one count each:
     - `corbanu exec`;
     - `env -u CORBANU_SECURITY_ORIGIN corbanu exec`;
     - `CODEX_HOME=$PWD corbanu exec`;
     - `corbanu mcp-server`;
     - a new session;
     - `corbanu tasknode status`.
  2. Variable removed and both `CODEX_HOME` and `HOME` pointed at the
     workspace: still refused, found through the registry
     (`2-marker-and-home-dropped.txt`).
  3. `/security`, `n`, save: the file reads `nested_agents = "pass"`
     (`3-*`).
  4. `pass` (`4-*`): `corbanu exec -s danger-full-access` reported that
     Aggressive was enforced and `--sandbox` ignored, then failed on the
     read-only home. A new session was still refused.
  5. Earlier build only (`5-earlier-build-*`), where nested sessions could
     pass. A session started outside a sandbox but pointed at a `pass` origin
     could not, even with approval:
     - read either vault store;
     - reach example.com;
     - write `../outside`.

     `/permissions` was refused. GLM first declined to run the probes
     (`model-refusal*`), so they ran through a script.
- **Reviews (Opus 5.5 High):**
  - first: request changes, 12 findings. All are fixed, or recorded above
    (finding 3: the rule written into an agent-made child home fails
    closed).
  - second: approve with fixes. Fixed:
    - real origins are checked before the variable;
    - indeterminate probes count as denied;
    - a failed account lookup refuses when the variable is set;
    - `archive`, `delete` and `unarchive` are hosts;
    - stale registry entries are pruned;
    - a failed registration shows a startup warning;
    - a test seam for the account home, plus a registry end-to-end test.

    Recorded above: N1 (registry under a home-folder workspace), N6 (second
    Permissive launch), N7.
