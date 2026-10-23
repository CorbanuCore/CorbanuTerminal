# G21 independent functional execution — AGG-32

## Preflight and identity

- Executor UTC start: 2026-10-11T05:20:12Z; launcher preflight `ok=true`; tmux-child preflight `ok=true`; PTY input echo `true`; summary `ok=true` (`preflight-summary.json`).
- Executor preflight command run verbatim: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`; observed `rc=0`, parsed `ok=true` at 2026-10-11T05:22:13Z.
- Candidate SHA-256 observed: `181ad0299d238accbf936512531c90988e8d15cb02a50ea8241f8566b2372a4a` (expected same).
- Frozen cases file SHA-256 observed: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4` (expected same).
- Product package/version source: `/opt/agms/pkg/bin/corbanu`, packaged Corbanu Terminal 0.1.48.
- Coordinator fixture check: root-owned `/etc/codex/config.toml` was present and readable, containing `openai_base_url = "http://127.0.0.1:18443/v1"` and `cli_auth_credentials_store = "file"`.
- Product home initially contained only `config.toml`, enabling feature flags and starting security at Permissive as supplied.
- Neutral navigation help was run before product launch (`agms-tui help`).

## AGG-32 (advisory)

Requested case actions:
1. Restart.
2. Open `/security`, press `i`.

Setup to reach requested starting condition: established saved Aggressive from the fresh supplied profile (Permissive), then used the product's saved-state screen's offered `r restart now` as the case's Restart action.

### Setup observations

- 2026-10-11T05:23:20Z, requested `agms-tui start`; actual executed launch arguments were `/opt/agms/bin/agms-product zai glm-5.3-flash`. The workspace trust prompt appeared (`SETUP-00-trust`), offering `1. Yes, continue`, `2. No, quit`, and Enter. At 05:23:41 pressed Enter; main screen appeared (`SETUP-01-main`).
- 2026-10-11T05:23:49, typed `/security` as a separate action; then pressed Enter at 05:23:49. Security picker (`SETUP-03-security-picker`) showed `Active in this session: Permissive`, Permissive marked active, Moderate unavailable, and Aggressive offered.
- 2026-10-11T05:23:55, pressed Down twice; Aggressive was highlighted (`SETUP-04-aggressive-highlight`). Pressed Enter at 05:23:56; Aggressive review appeared (`SETUP-05-aggressive-review`) with escape/save choices and `i inspect`.
- 2026-10-11T05:24:03, pressed Enter to confirm and save. Saved screen (`SETUP-06-aggressive-saved`) showed: `Saved: Aggressive (nested agents: refuse)`, that Aggressive required restart, that the session stayed Permissive until then, Core's level was Aggressive, and offered `r restart now`, `enter or esc close`, and `i inspect`.

### Requested versus actual

1. **Requested: Restart.** Actual: on the offered saved-state screen, pressed `r` at 2026-10-11T05:24:13. At 05:24:15 the product visibly showed `Restarting Corbanu Terminal to apply the saved security level…` and then the warning plus main UI (`CASE-01-restart`).
2. **Requested: Open `/security`, press `i`.** Actual: at 2026-10-11T05:24:21 typed `/security` separately; pressed Enter at 05:24:21. Picker (`CASE-03-security-picker`) showed `Active in this session: Aggressive` and `Core's level: Aggressive`, with Aggressive active. At 05:24:22 pressed `i`; inspector appeared (`CASE-04-inspector`). Scrolled with Down to reveal the boundary row (`CASE-05-inspector-scrolled`, `CASE-06-inspector-boundary`).

Additional navigation-only actions after the requested `i`: five Down presses by 05:24:31, then six Down presses by 05:24:36, to view the inspector's Protected boundary row; `r refresh` and `esc back to /security` were offered but not needed.

### Positive visible checkpoints

- 05:23:20 / `SETUP-00-trust`: trust prompt displayed with cancellation choice `No, quit`.
- 05:23:51 / `SETUP-03-security-picker`: Permissive was active in the fresh profile.
- 05:23:57 / `SETUP-05-aggressive-review`: Aggressive review displayed its save/escape/inspect controls.
- 05:24:04 / `SETUP-06-aggressive-saved`: Aggressive was saved, with the exact visible text `Saved: Aggressive (nested agents: refuse)`.
- 05:24:15 / `CASE-01-restart`: startup warning showed one blocker and named `/etc/codex/config.toml`.
- 05:24:22 / `CASE-03-security-picker`: `Active in this session: Aggressive` and `Core's level: Aggressive`.
- 05:24:24 / `CASE-04-inspector`: inspector header showed `▲ Aggressive degraded: Protected boundary, Credential broker`.
- 05:24:37 / `CASE-06-inspector-boundary`: inspector row showed `[degraded] Protected boundary: protected boundary not clean: 1 blocker; details in the Aggressive review [resolved: launch preflight]`.

### Security level at transitions

- Initial fresh launch: security picker showed `Active in this session: Permissive`.
- Immediately after saving Aggressive: saved screen said this session stayed Permissive until restart, while Core's level was Aggressive.
- After restart and on `/security`: `Active in this session: Aggressive`, `Core's level: Aggressive`.
- Inspector: `Saved: Aggressive [configured: /security]`; `Core config: Permissive [configured: config layers]`; `[set] Active this run: Aggressive, every row verified at launch`; `[on] Live policy: Aggressive in force; next start Aggressive; kill switch off; generation 0.1`.

### Warning verbatim

From `CASE-01-restart` at 05:24:15:

> ⚠ Security level Aggressive is enforced, but the protected boundary is not clean (1 blocker). Earlier conversations cannot be resumed. See /security:
>     Isolation: agent commands can still read /etc/codex/config.toml; it was not found when Corbanu Terminal started (another folder or config layer,
>   or a new file). Remove it, or restart from the folder that holds it

### Inspector boundary observation verbatim

From `CASE-06-inspector-boundary` at 05:24:37:

> [degraded] Protected boundary: protected boundary not clean: 1 blocker; details in the Aggressive review [resolved: launch preflight]

### Fixture contents

- Pre-case: recorded as `pre-AGG-32`. `workspace/.env` contained only `# synthetic test value, not a real credential` and `NOTES_SYNC_TOKEN=agms-canary-7f3a9c-synthetic-token`. `outside-notes.txt` contained `outside-the-project: synthetic file`. Other supplied workspace/git files were as provided.
- Post-case: recorded as `post-AGG-32`; comparison of the two recorded fixture maps showed them identical.
- Product profile: after setup/case, `product-home/config.toml` retained the supplied feature flags and additionally showed the workspace as `trust_level = "trusted"`; Aggressive/profile artifacts appeared under product-home (including `rules/corbanu-security-aggressive.rules` and `security_level.toml`), matching the product's saved-level behavior. No fixture file changed.

### Refusals and declines

- None observed. No credential or system password prompt appeared; no model prompt was submitted in this case, so there was no model decline.

### Expected result dispositions

- **A warning names the specific path and the fix — passed.** Startup warning explicitly named `/etc/codex/config.toml` and offered the fix: `Remove it, or restart from the folder that holds it`.
- **The inspector shows the boundary as not clean — passed.** Inspector showed `Aggressive degraded` and explicitly identified the Protected boundary as `protected boundary not clean: 1 blocker`.

### Notes and ambiguities

- The startup warning says `/etc/codex/config.toml` "was not found when Corbanu Terminal started (another folder or config layer, or a new file)"; the file was present as the coordinator's supplied root-owned layer. This wording is preserved as displayed and was not interpreted further.
- The inspector reports `Core config: Permissive [configured: config layers]` while the picker reports `Core's level: Aggressive`; both observations are preserved without reconciling internal behavior.
- Evidence artifacts include tool-generated `actions.jsonl`, `screens.jsonl`, `pty.raw`, and labeled fixture records. No human acceptance is claimed.

VERDICT AGG-32: passed
