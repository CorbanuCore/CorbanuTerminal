# Independent functional execution report — group G15

- Run UTC start: 2026-10-11T02:36:48Z
- Timezone: America/Los_Angeles
- Assigned cases, in order: AGG-25, AGG-34
- Frozen cases digest: matched `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`
- Candidate digest: matched `181ad0299d238accbf936512531c90988e8d15cb02a50ea8241f8566b2372a4a`
- Candidate version observed by probe: `corbanu 0.1.48`

## Isolation preflight

- Coordinator `preflight-summary.json`: `"ok": true`; the launcher and tmux-child reports beside it also report `"ok": true`.
- Executor command `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?` returned `rc=0`; the saved executor report also reports `"ok": true`.
- Candidate and frozen packet digests were independently verified before execution.

## Execution constraints and state

- Product actions were driven only through `agms-tui`; prompt text and Enter were sent separately.
- Fixture contents were recorded with `agms-tui fixture` as `before-agg25`, `before-agg34`, and `after-agg34`; the visible workspace fixture did not change during these cases.
- The product home `config.toml` was altered only in attempts to establish AGG-25's required unmanaged-secret condition, then restored to the packaged baseline after execution. The product profile itself retains state changes from navigation/saving attempts; this is noted under AGG-25.
- No live account operation was attempted. When the product later showed an API-key onboarding screen, the session was stopped rather than entering a key.

## AGG-25 (blocker)

### Requested versus actual

Requested starting condition:

- Fresh profile launched with an unmanaged secret in config, producing the screen E1 preflight-blocked condition.

Actual starting-condition attempts:

1. Added an unrecognized top-level `cli_auth_credentials_store` value. Product did not launch. Observed terminal error: `Error loading config.toml: unknown variant \`synthetic-unmanaged-secret\`, expected one of \`file\`, \`keyring\`, \`auto\`, \`ephemeral\` in \`cli_auth_credentials_store\``.
2. Tried a table form for the same key. Product did not launch. Observed terminal error: `Error loading config.toml: expected table, found string in \`cli_auth_credentials_store\``.
3. Tried synthetic provider/credential entries under a `[credentials]` table and a top-level token name. The product launched, but the Aggressive review showed `Preflight passed`, not the required `Preflight blocked`.
4. No supplied fixture or case instruction identified the exact config literal required for the E1 condition. Because the required starting condition could not be established, the case was stopped as blocked.

Observations made during an aborted attempt:

- At `02:46:36Z`, `/security` opened the picker showing `Active in this session: Permissive`.
- After Down Down and Enter, the review showed `Preflight passed`, not `Preflight blocked`.
- Escape returned to the picker, which still showed `Active in this session: Permissive` and `Cancelled. Nothing changed.` This is a partial cancellation checkpoint, but not the required blocked-state checkpoint.
- After AGG-34, a later probe attempt found the profile was no longer fresh because AGG-34 had saved Aggressive. It was returned to Permissive through the UI and restarted, but the API-key onboarding screen then appeared. Execution was stopped rather than supplying a key.

### Expected results

- `'Preflight blocked' is shown with a specific fix` — **blocked**: the required unmanaged-secret starting condition was not available and could not be established from the supplied fixtures/instructions.
- `Enter does not save and the picker still shows Permissive` — **inconclusive**: a cancellation checkpoint was observed in an aborted attempt, but not from the required blocked preflight state.
- `After the fix the preflight passes and saving works` — **not exercised**: no blocked state or specified fix existed.

### Screens and times

- `02-security-picker-with-config-secret` — 2026-10-11T02:47:34Z — Permissive picker.
- `03-aggressive-review-with-config-secret` — 2026-10-11T02:47:35Z — review showed `Preflight passed` (condition mismatch).
- `25-permissive-saved` — 2026-10-11T02:58:51Z — later profile reset saved Permissive.
- `25-github-token-after-permissive-reset` — 2026-10-11T03:00:00Z — API-key onboarding screen; session stopped.

**VERDICT AGG-25: blocked**

## AGG-34 (advisory)

### Requested versus actual

Requested:

1. Fresh profile; Aggressive review open.
2. Press `n`, Enter, `r`.
3. `/security`, `i`.

Actual:

- Fresh start reached trust prompt; Enter continued to the main screen.
- `/security` text and Enter were separate.
- Down Down then Enter opened the Aggressive review at `2026-10-11T02:53:15Z`.
- Pressed `n`, then Enter. The saved screen showed `Saved: Aggressive (nested agents: pass)` at `2026-10-11T02:53:26Z`.
- Pressed `r`; the product restarted and returned to its main screen.
- After restart, `/security` text and Enter were separate; the picker showed `Active in this session: Aggressive` and `Core's level: Aggressive`.
- Pressed `i`, then scrolled the inspector to the Agents section.

### Positive checkpoints

- `34-01-fresh-start` — 2026-10-11T02:53:08Z — fresh product main screen.
- `34-03-aggressive-review-open` — 2026-10-11T02:53:15Z — Aggressive review open.
- `34-04-after-n-enter` — 2026-10-11T02:53:26Z — `Saved: Aggressive (nested agents: pass)`.
- `34-06-security-after-save` — 2026-10-11T02:54:13Z — active and core level Aggressive.
- `34-10-inspector-scrolled-end` — 2026-10-11T02:55:09Z — inspector showed `Nested agents: pass` with `corbanu exec` and `review` running under Aggressive enforcement.

### Refusals or model declines

- None observed. No model completion was requested.

### Expected results

- `The saved line shows 'nested agents: pass' and the inspector agrees` — **passed**: both visible checkpoints showed Aggressive saved and nested agents set to pass.
- `The inspector's list of refused nested commands matches the review's list` — **failed**: the captured post-toggle review text named `interactive sessions`, `app-server`, and `mcp-server` as still refused, while the visible inspector Agents row showed the pass behavior for `corbanu exec` and `review` but no matching refused-command list. The case expectation was therefore not positively verified.

### Screens and times

- `34-00-trust` — 2026-10-11T02:53:08Z
- `34-02-security-picker` — 2026-10-11T02:53:14Z — Permissive.
- `34-03-aggressive-review-open` — 2026-10-11T02:53:15Z — Permissive review state.
- `34-04-after-n-enter` — 2026-10-11T02:53:26Z — saved Aggressive/pass.
- `34-06-security-after-save` — 2026-10-11T02:54:13Z — Aggressive active.
- `34-07-inspector` — 2026-10-11T02:54:13Z — inspector top.
- `34-10-inspector-scrolled-end` — 2026-10-11T02:55:09Z — Agents section.

**VERDICT AGG-34: failed**

## Final verdicts

```text
VERDICT AGG-25: blocked
VERDICT AGG-34: failed
```

This report records automated functional observations only and does not claim human acceptance.
