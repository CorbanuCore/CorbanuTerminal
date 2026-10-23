Both fixes are merged into main as separate PRs, and each passed the full per-fix gate with CI green. Claude panes are blocked under protected levels rather than brought under the contract; the full fix is filed as #218. Two points need your decision (below).

| Fix | PR | Status |
|---|---|---|
| 1. Agents started by agent commands under Aggressive (nested launches): `refuse` or `pass` | #219 | merged |
| 2. Claude panes refused under protected levels | #220 | merged; full fix is #218 |

**1. Nested launches (#219)**
- **Setting:** `nested_agents` in `security_level.toml`, `refuse` by default. Only a person can change it, in `/security` by pressing `n` in the Aggressive review. It has the same protection as the level file. Saving Permissive resets it, and an unknown value refuses.
- **Marker the child can't drop:** Aggressive now sets `CORBANU_SECURITY_ORIGIN` in every agent command's environment. A launching `corbanu` also treats itself as nested when it finds a home that enforces Aggressive but that it can neither write nor read the vault store of, which is what the Aggressive profile does to agent commands. It checks:
  - its own home;
  - the account's default homes, found through the account database rather than `$HOME`;
  - a registry of Aggressive homes, recorded at each Aggressive launch.

  Removing the variable, or pointing `CODEX_HOME` and `HOME` elsewhere, is still refused. If several homes match, `refuse` wins.
- **`pass`:** `corbanu exec` and `review` run with Aggressive enforced. Weakening flags are ignored and every Aggressive row is verified before the run starts.
- **Always refused when nested:** interactive sessions, `app-server`, `mcp-server` and other agent hosts, and commands that read credentials (`vault`, `tasknode`, the internal token helpers).
- **Side fix:** `--dangerously-bypass-hook-trust` is now ignored under Aggressive.

**2. Claude panes (#220)**
- **Why block:** each turn starts `claude --permission-mode bypassPermissions` directly. It gets the full environment, no sandbox and full network, and for direct providers Claude holds the raw key through `corbanu vault auth-helper`.
- **Why not this round:** bringing it under the contract needs four things:
  - a clean environment;
  - every provider behind the bridge;
  - a sandbox around `claude` that still reaches the bridge (on Linux that means a Unix-socket bridge);
  - a replacement for Claude's permission bypass.

  #218 has the details.
- **What the block does:** Claude panes are refused while Aggressive is active or saved, at every entry point: the pickers, pane creation, every kind of turn, whips and assignments, the process start, and the smoke commands. Permissive is unchanged.
- **Remaining risk:** a Claude turn already running when Aggressive is saved is not interrupted.

## Gate evidence
- **Tests:** focused `just test` runs after `just fmt` and `just fix`, all passing.
  - #219: 6 CLI tests (both modes, a forged home, the registry, a person's own launch, and `pass` against a mock model where the vault read fails in the sandbox), 163 TUI and 130 exec. Re-run after merging main.
  - #220: 265 TUI.
- **Linux:** clippy is clean for both on the RTX box. Main currently fails Linux clippy at `core/src/security/tainted_action.rs:394` (`expect_used`); that is outside these PRs, so I allowed that line for the local check only.
- **GLM 5.2 tmux runs, real nesting:**
  - Nested `corbanu exec` was refused, including with the variable removed and with `CODEX_HOME` and `HOME` changed. `mcp-server`, a new session and `tasknode` were refused too.
  - With `pass`, the nested run reported that Aggressive was enforced and `--sandbox` was ignored.
  - On an earlier build, a nested child could not read either vault store, reach the network or write outside the folder, even with approval.
  - Claude panes: Permissive opens them; Aggressive, active or only saved, refuses.
  - Captures are under `qa/security-levels/pf24-followups/{nested-launch,external-panes}/`.
- **Reviews (Opus 5.5 High):**
  - #219: request changes (12 findings), then approve with fixes.
  - #220: approve with fixes, twice.
  - Every finding is fixed or recorded in the PR's README.
- **Videos:**
  - [nested launch refused](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/nested-launch-nested-launch-refused-06f05a222b28-2026-10-06.mp4)
  - [nested launch with pass](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/nested-launch-nested-launch-pass-06f05a222b28-2026-10-06.mp4)
  - [Claude panes refused](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/external-panes-claude-panes-aggressive-refused-5361875df38e-2026-10-06.mp4)
- **CI:** green on both. #219's macOS credential canary failed once on a network-proxy test that #219 doesn't touch; it passes locally, and the canary passed on the next CI run.

## Decisions for Travis
1. **Interactive sessions never pass.** Your decision listed a bare prompt and `resume` as `pass` candidates. In a nested interactive session, the agent that started it would answer its approval prompts, so I refuse them in both modes.
   - **Recommended:** keep it that way.
   - **Alternative:** allow them and state that their approvals aren't a person's.
2. **`pass` can't do useful work with today's Aggressive sandbox.** Inside an Aggressive agent command, the Corbanu home is read-only and the network is off. A passed `corbanu exec` starts with Aggressive enforced, then stops at "Operation not permitted". For now `pass` only changes the error message. It would need the sandbox to let the nested run write a home and reach the model, which is your call.

## Still open
- If the workspace is your home folder, an agent can delete registry entries. Fixing that means making the registry read-only in the Aggressive sandbox, which I didn't change.
- A second Permissive launch of the same home removes the rule file and registry entry while an Aggressive session is still running. This was already open from round 3 and fits PF-24-S02.
- On Windows, and in containers with no account database, only the variable and the process's own home are checked.
- The standalone `codex-exec` and `codex-tui` binaries don't run the nested check.
- Older builds read a file with `nested_agents = "pass"` as unreadable. They then enforce Aggressive and show a warning.
- Debug builds honour `CORBANU_TEST_ACCOUNT_HOME` in place of the account database; the isolated test runner sets it so tests never touch real homes.