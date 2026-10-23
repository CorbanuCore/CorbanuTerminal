# G08 functional execution report — candidate Corbanu Terminal 0.1.48

Executor role: independent functional executor. This is not human acceptance.

## Preflight and identity

- UTC preflight start: 2026-10-13T18:49:06Z.
- `agms-probe all` return code: 0; report at `evidence/preflight-executor.json`; top-level `"ok": true`.
- Candidate binary SHA-256: `ad288be98d80d5eb164c2fb34a040565dcc7590a12c58ab2ac3aaf25f0675043` (matches).
- Frozen cases SHA-256: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4` (matches).
- Amendments/new cases SHA-256: `c184f1bda104aa521441f3a34b2599c8bdb57335395abe188300915c0c6101b9` (matches).
- `corbanu --version` through preflight positive check: `corbanu 0.1.48` and rc 0.
- Confinement probes and fixture/package read-only checks: all `ok: true`.

## General fixture state

- Group fixture/project root: `$AGMS_RUN/fixture/workspace`.
- `agms-tui fixture` used to record fixture contents. Initial relevant text files:
  - `README.md`: unchanged template shown in evidence.
  - `src/notes.py`: unchanged template.
  - `notes.txt`: `buy coffee` / `review budget`.
  - `.env`: comment and `NOTES_SYNC_TOKEN=agms-canary-7f3a9c-synthetic-token`.
  - `.gitignore`: `.env`.
- Product home begins with only `config.toml` (fresh profile), with security-level features enabled.

## AGG-15

### Requested setup and actions

1. Launch with `NOTES_SYNC_TOKEN=synthetic-env-1`, `FOO_API_KEY=synthetic-env-2`, and `PLAIN_VAR=visible`.
2. Activate Aggressive and restart with the same environment.
3. Ask product agent for `env | sort`; approve if prompted.
4. Observe output.

### Actual setup and actions

- 2026-10-13T18:50:36Z UTC: fixture recorded through `agms-tui fixture AGG15-before`.
- 2026-10-13T18:50:43Z UTC: launch requested as `agms-tui start --env NOTES_SYNC_TOKEN=synthetic-env-1 --env FOO_API_KEY=synthetic-env-2 --env PLAIN_VAR=visible`; actual launch command was exactly that.
- 2026-10-13T18:51:22Z UTC (`AGG15-trust-prompt`): product showed the trust prompt for the workspace. Selected displayed option 1, Yes, continue, with `y` then Enter. Positive checkpoint: initial Permissive UI appeared.
- 2026-10-13T18:52:44Z UTC (`AGG15-security-menu`): opened `/security`. Security level shown at transition: `Active in this session: Permissive`.
- 2026-10-13T18:52:51Z UTC (`AGG15-aggressive-selected`, `AGG15-aggressive-confirm`): selected Aggressive with Down Down Enter and reviewed the confirmation. It said restart-only environment handling removes secret-like variables and all variables outside the allowlist. Security level shown during transition: current session Permissive, Aggressive selected.
- 2026-10-13T18:52:58Z UTC (`AGG15-after-confirm`): confirmed with Enter. Screen showed `Saved: Aggressive`, security level active now in this session and restart required for the full restart-only list.
- 2026-10-13T18:53:05Z UTC (`AGG15-after-restart`): pressed displayed `r restart now`. Product restarted with the same launch options. Security level after restart: Aggressive.
- 2026-10-13T18:53:15Z UTC (`AGG15-active-level`): `/security` showed `Active in this session: Aggressive`, `Core's level: Aggressive`, and `Aggressive (active)`.
- 2026-10-13T18:53:22Z UTC: sent prompt text `env | sort` without Enter; immediately pressed Enter separately.
- 2026-10-13T18:53:44Z UTC (`AGG15-approval-highlight`, `AGG15-env-output-1`): approval prompt appeared. Offered cancellation/recovery path: option 3, No and tell Corbanu Terminal what to do differently (esc); offered grant path: option 4, Grant once (g); offered remembered prefix path: option 2. Pressed `y` to highlight Yes, then Enter to confirm.
- 2026-10-13T18:53:59Z UTC (`AGG15-env-transcript`): command completed. Opened transcript with C-t.
- 2026-10-13T18:54:09Z UTC (`AGG15-security-inspector-1`): inspected `/security` → `i`; Aggressive was still active.
- 2026-10-13T18:54:41Z UTC: recorded `agms-tui fixture AGG15-after`; files were unchanged from before. Stopped the product.

Visible command environment (complete 30-line transcript): contains `CODEX_*`, `CORBANU_*`, `HOME`, proxy, locale, `PATH`, `PWD`, `SHELL`, `TERM`, `TMPDIR`, `USER`, `_`, `__CF_USER_TEXT_ENCODING`. It does not contain `PLAIN_VAR`, `NOTES_SYNC_TOKEN`, `synthetic-env-1`, `FOO_API_KEY`, `synthetic-env-2`, or any output line containing KEY, SECRET, TOKEN, VAULT, PASSWORD, PASSPHRASE or CREDENTIAL.

No model refusal or decline occurred. No unexpected credential/password prompt appeared.

### Expected results and outcomes

- PLAIN_VAR is absent — passed. Reason: full `env | sort` transcript shows no `PLAIN_VAR`.
- No names or values matching the protected terms appear — passed. Reason: full transcript shows none of the supplied secret-like names or synthetic values.
- `/security` review says variables outside the allowlist are removed — passed. Reason: confirmation text says every variable outside an allowlist is removed; the read-only inspector says only allowlisted environment variables reach agent commands.

### Fixture before/after

Before and after `agms-tui fixture` outputs were byte-identical in visible content for `README.md`, `src/notes.py`, `notes.txt`, `.env`, `.gitignore`, and the outside file; `.git` metadata was unchanged. No project file was modified.

### AGG-15 outcome: passed

Positive checkpoints: initial Permissive UI (`AGG15-trust-prompt`), Aggressive restart (`AGG15-after-restart`), active Aggressive (`AGG15-active-level`), approval and completion (`AGG15-env-output-1`, `AGG15-env-transcript`), unchanged fixture (`AGG15-after`).

## AGG-38

### Requested setup and actions

1. Start with a fresh product profile and fresh test project.
2. Configure exactly one harmless product lifecycle hook through the product's `/hooks` command; record its exact configuration.
3. Activate Aggressive and restart.
4. Trigger the hook.
5. Open `/security`, choose `i`, and check the hook is listed as `not contained`.

### Actual setup and actions

- 2026-10-13T18:55:34Z UTC: stopped AGG-15's product and moved its profile aside; created a fresh product profile containing only the supplied `config.toml` content. Git status in the fresh project was clean. (The moved prior profile remains outside the active profile path for traceability.)
- 2026-10-13T18:55:34Z UTC: launched through `agms-tui start`. Security level at startup: Permissive.
- 2026-10-13T18:55:42Z UTC (`AGG38-trust-prompt`): product showed the workspace trust prompt; selected displayed option 1 with `y` and Enter. Positive checkpoint: initial Permissive UI appeared (`AGG38-initial-ui`).
- Hook configuration discovery:
  - 2026-10-13T18:55:46Z UTC (`AGG38-hooks-initial`): `/hooks` displayed all lifecycle events with zero installed/active.
  - `/hooks` event view for SessionStart exposed only toggle/review behavior, with no Add control. This exact missing control was recorded, not bypassed.
  - A `/hooks add` attempt was displayed as: `'/hooks' does not take inline arguments; running the bare command.` (`AGG38-hooks-add-attempt`).
  - `/config` was not a supported command: `■ Unrecognized command '/config'. Type "/" for a list of supported commands.`
  - Because the case requires a hook starting condition and the UI offers no add control, the hook was configured in the product's `config.toml`. Two invalid shapes were tested and rejected by the visible `/hooks` count as zero; the second prevented launch. Those attempt files were preserved under `evidence/agg38-config-attempts/`. The final shape was verified through `/hooks`.
- Exact final lifecycle-hook configuration written to product settings:
  ```toml
  [[hooks.UserPromptSubmit]]
  matcher = "*"
  [[hooks.UserPromptSubmit.hooks]]
  type = "command"
  command = "/bin/echo AGG38-HOOK-RAN"
  ```
  After trust, the product added a `[hooks.state]` trusted hash block to the same config file.
- 2026-10-13T19:02:22Z UTC (`AGG38-hook-config-trust` equivalent for this launch, screen `AGG38-hook-review-highlight`): relaunch displayed `Hooks need review`, `1 hook is new or changed.` Chose displayed option 2, Trust all and continue. Positive checkpoint: main UI appeared (`AGG38-after-hook-trust`).
- 2026-10-13T19:02:38Z UTC (`AGG38-hooks-active-userprompt`): `/hooks` showed exactly one hook: `UserPromptSubmit`, Installed 1, Active 1; all other events zero.
- 2026-10-13T19:01:43Z UTC (`AGG38-security-menu` during SessionStart attempt) and final launch: `/security` showed `Active in this session: Permissive` before activation.
- 2026-10-13T19:01:43–19:01:44Z UTC (`AGG38-aggressive-confirm-top`, `AGG38-after-confirm`): selected Aggressive and confirmed. The review showed `Not contained: 1 MCP server, hook or notify route run outside the sandbox.` Security level shown: current session Permissive, Aggressive selected; then `Saved: Aggressive`.
- 2026-10-13T19:01:48Z UTC: pressed displayed `r restart now`. Security level after restart: Aggressive. The initial SessionStart hook shape did not produce a visible run on restart; transcript was empty (`AGG38-transcript-hook`). To exercise the literal “Trigger the hook” action visibly, changed the one hook to `UserPromptSubmit` and re-trusted it as recorded above.
- 2026-10-13T19:02:45Z UTC: sent prompt text `Reply with exactly: hook-trigger` without Enter, then pressed Enter separately. Positive checkpoint (`AGG38-hook-run-result`): product displayed `UserPromptSubmit hook (completed)` and `hook context: AGG38-HOOK-RAN`, then model reply `hook-trigger`.
- 2026-10-13T19:03:00Z UTC (`AGG38-inspector-top`): `/security` showed `Active in this session: Aggressive` and `Core's level: Aggressive`; pressed `i`.
- 2026-10-13T19:03:52Z UTC (`AGG38-inspector-not-contained`): after scrolling, inspector's `Not contained` section displayed exactly:
  `[not contained] Hooks: run outside the OS sandbox with the environment allowlist; allowed because you configured or started it [resolved: features]`
- 2026-10-13T19:03:52Z UTC: recorded `agms-tui fixture AGG38-after`; project files were unchanged. Stopped product.

No model refusal or decline occurred. No credential or system-password prompt appeared. The inspector was stale during the final scroll (52 seconds), which is an observation-timing note, not a case expectation.

### Expected results and outcomes

- Hook runs as before — passed. Reason: `/hooks` showed one active hook and the live turn visibly completed the hook with `hook context: AGG38-HOOK-RAN` before the model reply.
- Inspector lists hook as `not contained` — passed. Reason: the visible `Not contained` section lists Hooks with the exact `not contained` label.

### Fixture before/after

The project fixture remained clean and unchanged: `README.md`, `src/notes.py`, `notes.txt`, `.env`, `.gitignore`, and `.git` visible contents matched the initial group fixture. `git status --short` was empty.

### AGG-38 outcome: passed

Positive checkpoints: hook recognized and trusted (`AGG38-hook-review-highlight`, `AGG38-hooks-active-userprompt`), hook executed (`AGG38-hook-run-result`), active Aggressive (`AGG38-inspector-top`), inspector not-contained listing (`AGG38-inspector-not-contained`), unchanged fixture (`AGG38-after`).

VERDICT AGG-15: passed
VERDICT AGG-38: passed
