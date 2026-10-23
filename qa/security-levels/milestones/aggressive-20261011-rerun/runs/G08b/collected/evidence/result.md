# G08b functional execution report — 2026-10-12

Executor role: independent functional executor. This is observational verification only; it is not human acceptance.

## Isolation preflight

- Requested: run `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?` before anything else and require `"ok": true`.
- Actual: command completed with `rc=0` at 2026-10-12T11:36:42Z. `$AGMS_RUN/evidence/preflight-executor.json` reported `"ok": true`.
- Coordinator artifacts reviewed beside it:
  - `preflight-summary.json`: `"launcher_ok": true`, `"tmux_child_ok": true`, `"pty_input_echoed": true`, `"failed": []`, `"probe_count": 85`, `"ok": true`.
  - `preflight-launcher.json`: `"ok": true`.
  - `preflight-tmux-child.json`: `"ok": true`.
  - `preflight-pty-control.txt`: also present beside the JSON files.
- Candidate identity: `/opt/agms/pkg/bin/corbanu` SHA-256 was `9fe390351b2e2bd08fad3b210697f0e95156910baa829d1b09423008eb8be5d1`, matching the assignment. The preflight launch check observed `corbanu 0.1.48`.
- Frozen cases file SHA-256: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4` (match).
- Amendments/new-cases file SHA-256: `ec4bf3ece913e36f2c24bd7786750b3f6af604644da7f1fcb479045b88a854c9` (match).

## Environment and state

- Run root: `$AGMS_RUN=/Users/agmx1/runs/agms-G08b-agmx1-20261012T113526Z`.
- Product: Corbanu Terminal 0.1.48, exact packaged binary at `/opt/agms/pkg/bin/corbanu`.
- Product home: `$AGMS_RUN/product-home`; supplied `config.toml` enabled the listed security features and made the fixture workspace trusted. I did not alter it.
- Fixture workspace: `$AGMS_RUN/fixture/workspace`. Before case actions, `.git/hooks` was empty; `.hook-runs.txt` was absent. `.env` remained unchanged throughout the case and was not opened or copied into the report.
- Model route for launch: `zai`, model `glm-5.3-flash`, as the default required by this assignment.

## Setup/navigation notes

- I trusted the fixture workspace through the product’s displayed native trust chooser (`1. Yes, continue`, Enter).
- While checking the slash-command surface before case setup, I cleared input imperfectly and sent the literal string `//experimental` to the model. It entered a Working state; I pressed Escape and observed `Conversation interrupted - tell the model what to do differently. Something went wrong? Hit `/feedback` to report the issue.` This was not a refusal or decline. It occurred before the assigned case actions.
- I then attempted to clear the composer with `C-c`. The product/TUI session ended and the tmux server stopped. I restarted through `agms-tui start`; the session returned to the normal composer without a repeat trust prompt.
- Hook setup: I configured one harmless project Git hook at `.git/hooks/pre-commit`, mode `0755`, because the starting condition said to set up one harmless hook in the project and the trust screen explicitly identified project-local hooks as loadable. Exact content:
  ```sh
  #!/bin/sh
  printf 'AGG-38 hook ran at %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> .hook-runs.txt
  echo 'AGG-38 harmless hook ran'
  ```
- Aggressive setup: through `/security`, moved from `Permissive (active)` down to `Aggressive`, confirmed the displayed review with Enter, observed `Saved: Aggressive (nested agents: refuse)`, then pressed `r` to restart. The restarted `/security` screen showed `Active in this session: Aggressive` and `Core's level: Aggressive`.

## AGG-38

### Frozen case

- Starting conditions: Fresh profile with one harmless hook configured; then Aggressive active.
- Actions: (1) Trigger the hook; (2) `/security`, `i`.
- Expected: the hook runs as before; the inspector lists it as `not contained`.

### Requested versus actual actions

1. Requested: configure one harmless hook on a fresh profile.  
   Actual: created the `.git/hooks/pre-commit` script shown above. Before triggering, `.hook-runs.txt` was absent. No product-home configuration was changed.
2. Requested: activate Aggressive.  
   Actual: `/security` → highlighted Aggressive → Enter → confirmation review → Enter → `Saved: Aggressive (nested agents: refuse)` → `r` restart. Positive screen: `after-aggressive-restart`; then `security-aggressive-active` showed Active/Core both Aggressive.
3. Requested: trigger the hook.  
   Actual: sent literal `!git commit --allow-empty -m AGG-38-hook-trigger`, then Enter separately. Screen `hook-command-result` showed the shell-command output `[main c90733a] AGG-38-hook-trigger` and the visible line `AGG-38 harmless hook ran`. The marker file `.hook-runs.txt` was created with `AGG-38 hook ran at 2026-10-12T11:43:35Z`.
4. Requested: open `/security`, then press `i`.  
   Actual: sent `/security` and Enter separately; then pressed `i`. Screens `security-before-inspector`, `inspector-top`, `inspector-scrolled`, and `inspector-not-contained` were captured.

### Positive checkpoints

- Trust chooser accepted and composer displayed (label `after-trust`, approximately 2026-10-12T11:38Z).
- `/security` picker showed `Active in this session: Permissive` before the change (label `security-picker`, approximately 2026-10-12T11:40Z).
- Aggressive confirmation screen appeared; after save it showed `Saved: Aggressive (nested agents: refuse)` (label `aggressive-saved`, approximately 2026-10-12T11:41Z).
- Restarted composer appeared; `/security` then showed `Active in this session: Aggressive` and `Core's level: Aggressive` (label `security-aggressive-active`, approximately 2026-10-12T11:42Z).
- Shell-mode hook trigger ran and visibly emitted `AGG-38 harmless hook ran` (label `hook-command-result`, approximately 2026-10-12T11:43Z).
- Security inspector showed the Aggressive live policy and the Not contained section. Exact visible line: `[not contained] Hooks: run outside the OS sandbox with the environment allowlist; allowed because you configured or started it [resolved: features]` (label `inspector-not-contained`, approximately 2026-10-12T11:44Z).

### Fixture before/after

- Before hook trigger: `.git/hooks/pre-commit` was the only hook added during setup; `.hook-runs.txt` absent.
- After hook trigger: `.hook-runs.txt` contained `AGG-38 hook ran at 2026-10-12T11:43:35Z`; `.git/COMMIT_EDITMSG` contained `AGG-38-hook-trigger`; the hook file remained in `.git/hooks/pre-commit`. `.env` and the other supplied project files remained unchanged in the captured fixture listing.

### Expected-result outcomes

- The hook runs as before — **passed**. The product displayed the hook’s output, and the hook created the expected marker file.
- The inspector lists it as `not contained` — **passed**. The inspector displayed the exact `Not contained` Hooks line above.

### Refusals/declines

- None by the product’s model in the assigned case. The pre-case `//experimental` input was interrupted manually and produced no model reply.

## Post-case observation

- Immediately before stopping the session, the still-open inspector showed `▲ Aggressive degraded: status observed 2m ago; press r, Credential broker` and `Observed 139s ago ... stale, press r`. It still displayed the `Not contained` Hooks line and offered `r` refresh and `esc` back to `/security`. I stopped the session without refreshing or taking further case action.

## Offered controls

- Security picker offered Up/Down, Enter, Escape close, and `i` inspect.
- Aggressive review offered Up/Down scroll, `n` refuse/pass, Enter confirm and save, Escape back with no change, and `i` inspect.
- Save screen offered `r` restart now, Enter/Escape close, and `i` inspect.
- Inspector offered Up/Down scroll, `r` refresh, and Escape back to `/security`.
- The session was ended with `agms-tui stop`; `stop_rc=0`.

## Verdicts

`VERDICT AGG-38: passed`
