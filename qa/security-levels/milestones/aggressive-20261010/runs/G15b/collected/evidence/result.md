# G15b independent functional execution report

This is a functional execution report, not human acceptance.

## Coordinator isolation and candidate identity

- Coordinator preflight summary and both `preflight-*.json` files were read. Summary: `"ok": true`, `launcher_ok: true`, `tmux_child_ok: true`, `pty_input_echoed: true`, 85 probes, no failures.
- Executor preflight was run as required:
  - Command: `agms-probe all > "$AGMS_RUN/evidence/preflight-executor.json"; echo rc=$?`
  - Result: `rc=0`; JSON reported `"ok": true`.
- Candidate: `/opt/agms/pkg/bin/corbanu`; SHA-256 observed and expected matched `181ad0299d238accbf936512531c90988e8d15cb02a50ea8241f8566b2372a4a`; visible launch/version evidence reported Corbanu Terminal 0.1.48.
- Frozen cases SHA-256 observed and expected matched `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`.

## AGG-25 (blocker)

### Requested versus actual actions

1. **Requested:** fresh profile launched with an unmanaged secret in config, using the E1 condition.  
   **Actual:** `agms-tui start -c 'cli_auth_credentials_store="file"'` at 2026-10-11T04:51:08Z; launch completed by 2026-10-11T04:51:10Z. The live trust dialog offered `1. Yes, continue` and `2. No, quit`; Enter on the displayed Yes control was used. The app then showed Corbanu Terminal 0.1.48 in the workspace (`AGG25-launch`).
2. **Requested:** `/security`, Down Down, Enter.  
   **Actual:** At 2026-10-11T04:51:24Z, `/security` was typed as text without Enter (`AGG25-security-command-typed`), then Enter was pressed (`AGG25-security-screen`). At 2026-10-11T04:51:30Z, Down Down was pressed (`AGG25-aggressive-selected`); Aggressive was highlighted and Permissive remained active. At 2026-10-11T04:51:32Z, Enter was pressed (`AGG25-after-aggressive-enter`).
3. **Requested:** scroll down, press Enter.  
   **Actual interpretation and evidence:** the displayed picker used Up/Down movement rather than a separate scroll operation; Down Down moved the highlight from Permissive to Aggressive. The subsequent Enter opened the confirmation/preflight screen. No separate scroll control was shown.
4. **Requested:** fix the cause as instructed, relaunch, retry.  
   **Actual:** the blocker instructed moving the config literal into the vault with a reference or removing it. The replay instruction defined the required fix as starting again without that argument. The first session was stopped at 2026-10-11T04:52:55Z with `agms-tui stop`; relaunch without the argument began at 2026-10-11T04:52:56Z and completed by 2026-10-11T04:52:58Z (`AGG25-relaunch`). Then `/security` was typed and Enter pressed, Down Down was pressed, and Enter was pressed at 2026-10-11T04:53:12Z (`AGG25-retry-after-enter`).

Verification action beyond the requested sequence: at 2026-10-11T04:51:51Z, an extra labeled Enter on the blocked screen produced `Not saved: resolve the blockers above first. Nothing changed.` (`AGG25-enter-on-blocked`). This was used to verify the no-save expected result and did not change the case wording.

### Positive visible checkpoints and times

- 2026-10-11T04:51:08Z: fixture before recorded as `G15b-AGG25-before`. Relevant contents: `outside-notes.txt` contained only synthetic outside-note text; `workspace/.env` contained only the comment `# synthetic test value, not a real credential` and `NOTES_SYNC_TOKEN=agms-canary-7f3a9c-synthetic-token`; `workspace/.gitignore` contained `.env`.
- 2026-10-11T04:51:10Z: launch showed Corbanu Terminal 0.1.48 (`AGG25-launch`).
- 2026-10-11T04:51:24Z: `/security` showed `Active in this session: Permissive`, `Permissive  (active)`, `Moderate  not available yet`, and `Aggressive` (`AGG25-security-screen`).
- 2026-10-11T04:51:32Z: Aggressive confirmation showed `Preflight blocked: Aggressive cannot be saved until these are resolved.` and the specific fix: `✗ launch flags cli_auth_credentials_store: unmanaged secret (config literal); move it into the vault and use a reference, or remove it [config-literal-46d03d4a1d3b]` (`AGG25-after-aggressive-enter`). It also showed `Controls ready: 6 of 6` and `Your config.toml is not modified.`
- 2026-10-11T04:51:52Z: Escape showed `Cancelled. Nothing changed.`, `Active in this session: Permissive`, `Permissive  (active)`, and Aggressive highlighted (`AGG25-after-escape`).
- 2026-10-11T04:53:12Z: retry without the argument showed `Preflight passed: no known raw-secret route stays open to agents or the model.`, `Controls ready: 6 of 6`, and `Your config.toml is not modified.` (`AGG25-retry-after-enter`).
- 2026-10-11T04:53:17Z: Enter on the passing confirmation showed `Saved: Aggressive (nested agents: refuse)`, `Restart Corbanu Terminal to activate Aggressive. Until then this session stays Permissive.`, and `Core's level is Aggressive now in this session, and from every start.` It offered `r restart now · enter or esc close · i inspect` (`AGG25-after-save-enter`).
- 2026-10-11T04:53:29Z: fixture after save recorded as `G15b-AGG25-after-save`; relevant fixture contents matched the before snapshot. The product session was then stopped.

### Security level / profile state at transitions

- First launch: no security level visible on the initial screen.
- `/security`: active Permissive; picker active item Permissive.
- After selecting Aggressive: active Permissive, Aggressive highlighted.
- Blocked confirmation: no-save state; picker afterward still active Permissive.
- Retry without the argument: active Permissive before selection; Aggressive highlighted.
- Passing confirmation: preflight passed; confirmation said `Your config.toml is not modified.` before confirm.
- After save: `Saved: Aggressive`; restart was required to activate Aggressive, while the current session remained Permissive.

### Cancellation, recovery and continuation paths

- Initial launch offered trust continuation and quit choices.
- Blocked screen showed `esc back, nothing changes`; Escape did cancel and no-save.
- Blocked screen showed `i inspect`; not exercised because it was not required.
- Passing screen showed `n refuse/pass · enter confirm and save · esc back, nothing changes · i inspect`.
- Saved screen offered `r restart now` and `enter or esc close`. Neither restart-now nor close was used beyond `agms-tui stop` after evidence capture.

### Fixture before/after

The relevant files were unchanged between before and after save. `.env` remained the synthetic file shown above and `.gitignore` continued to ignore `.env`. The outside file remained outside the project and unchanged. Other listed fixture files also matched before/after.

### Expected results and outcomes

- `'Preflight blocked' is shown with a specific fix` — **passed**. Both exact visible phrases and the specific config-literal fix were captured.
- `Enter does not save and the picker still shows Permissive` — **passed**. The extra Enter produced `Not saved: resolve the blockers above first. Nothing changed.`; Escape then showed `Cancelled. Nothing changed.` with Permissive active.
- `After the fix the preflight passes and saving works` — **passed**. Relaunch without the argument produced `Preflight passed`, and Enter produced `Saved: Aggressive`.

### Other observations

- No product-model refusal or decline occurred; no model prompt was submitted.
- No credential prompt, system password prompt, unexpected profile access, missing required control, lost observation, or budget exhaustion occurred.
- The “Down Down” wording maps to moving through a picker; there was no separately labelled scroll control before Enter. This was not treated as an ambiguity requiring blocking.
- Functional result only; no human acceptance is claimed.

```text
VERDICT AGG-25: passed
```
