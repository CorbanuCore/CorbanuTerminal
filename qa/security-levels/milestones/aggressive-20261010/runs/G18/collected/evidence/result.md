# Functional execution report — group G18

- Executor role: independent functional executor; not implementer or reviewer; no human acceptance claimed.
- Scope executed: assigned case AGG-28 only, in assigned order.
- Isolation preflight:
  - Executor command run before case work: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`.
  - Exit code: `0`; JSON: `"ok": true`.
  - Coordinator `preflight-summary.json`: `"ok": true`, `"probe_count": 85`, `"failed": []`.
  - `preflight-launcher.json`: `"ok": true`; `preflight-tmux-child.json`: `"ok": true`.
- Candidate and packet identity:
  - `/opt/agms/pkg/bin/corbanu` observed SHA-256 `181ad0299d238accbf936512531c90988e8d15cb02a50ea8241f8566b2372a4a`; expected value matched.
  - Packaged launch version observed from preflight: `corbanu 0.1.48`.
  - `/opt/agms/packet/design-cases.md` observed SHA-256 `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`; expected value matched.
- State/profile observations:
  - Initial `/security` showed `Active in this session: Permissive`.
  - After save and restart, `/security` showed `Active in this session: Aggressive` and `Core's level: Aggressive`.
  - Product-home `security_level.toml` later observed: `level = "aggressive"`.
  - Product-home `security_state.json` later observed: `level: aggressive`, `kill_switch_active: false`.
  - Product-home `config.toml` included `isolated_credential_broker = true`; trust was `trusted` for the fixture workspace.
  - The security inspector showed `Model keys: held by the isolated credential broker (broker_model_auth on)`, `[unobserved] Model key broker: installed`, and `[unobserved] Credential broker: on in config`.
  - No macOS Keychain or system password prompt appeared. No unexpected profile access appeared.
- Fixture summary:
  - Before evidence: `evidence/agg28-fixture-before.json`, captured at `2026-10-11T03:26:10Z`.
  - After evidence: `evidence/agg28-fixture-after.json`, captured at `2026-10-11T03:34:03Z`.
  - Redacted comparison (`evidence/agg28-fixture-before.redacted.tsv` versus after) showed no file-key or serialized-length change.
  - `.env` remained present with a synthetic value; its value is redacted here and was not relied on as model output.

## Case AGG-28 — blocker

### Requested case

- Starting condition: Aggressive active with the broker prevented from starting by the harness.
- Action 1: Ask: `Reply with exactly: pong`.

Expected:

1. The request is refused and the relay shows no direct request.
2. The message names `broker_model_auth` and how to change it.

### Setup and requested-versus-actual actions

1. Started the packaged product through `agms-tui start`; route `zai`, model `glm-5.3-flash`.
   - The UI offered a directory-trust choice. Chose the displayed `1. Yes, continue` with `Enter`.
   - Actual: fresh main screen appeared.
2. Sent `/security` as text and pressed `Enter` separately.
   - Actual: Security picker showed `Active in this session: Permissive` (`S1-security-permissive`).
3. Pressed `Down Down`, moving the highlighted choice to `Aggressive`; pressed `Enter`.
   - Actual: `Switch to Aggressive?` review opened and showed `Preflight passed` and `Controls ready: 6 of 6` (`S1-aggressive-review-top`, `S1-picker-aggressive-highlight`).
   - Security level shown before confirmation: active session `Permissive`.
4. Pressed `Enter` to confirm and save.
   - Actual: `Saved: Aggressive (nested agents: refuse)` appeared. It explicitly said restart was required and the current session stayed Permissive (`S1-aggressive-saved`).
5. Pressed `r` to restart now.
   - Actual: restart banner appeared; the wait pattern did not match because the suggested prompt changed, but the next live screen showed the restarted product.
   - Opened `/security`: active session and Core level both `Aggressive` (`S1-security-aggressive-active`).
6. Pressed `i` in the security picker to inspect.
   - Actual: security inspector showed Aggressive, `broker_model_auth on`, model key broker installed, credential broker configured/unobserved (`S1-security-inspector-aggressive`, `S1-security-inspector-aggressive-scrolled`).
7. Attempted to establish the required harness condition.
   - `agms-tui help` documented only a generic `--env NAME=synthetic...` facility; it did not name a broker-sabotage option.
   - Stop/restart attempt 1: `agms-tui start --env AGMS_TEST_DISABLE_CREDENTIAL_BROKER=synthetic-1`.
     - `/security` again showed Aggressive active (`AGG28-security-before-prompt`).
     - Sent `Reply with exactly: pong` as text, then `Enter`, at `2026-10-11T03:30:33Z`.
     - Actual result at `2026-10-11T03:31:20Z`: the model completed with visible response `pong` (`AGG28-prompt-result-after-30s`). The requested precondition was not established.
   - Stop/restart attempt 2: `agms-tui start --env AGMS_TEST_BROKER_DISABLE=synthetic-1`.
     - `/security` again showed Aggressive active (`AGG28-second-security`).
     - Sent the same literal prompt text, then `Enter`, at `2026-10-11T03:32:24Z`.
     - Actual result by `2026-10-11T03:32:54Z`: the model completed with visible response `pong` (`AGG28-second-prompt-result`). The requested precondition was still not established.
8. Stopped the case and the product session after the second attempted setup; fixture after evidence captured at `2026-10-11T03:34:03Z`.

### Positive visible checkpoints

- `S1-fresh-before-security` — packaged UI v0.1.48, model `glm-5.3-flash`, route `zai`, correct workspace; observed after trust at `2026-10-11T03:26:20Z`.
- `S1-security-permissive` — initial active security level Permissive; at `2026-10-11T03:26:58Z`.
- `S1-picker-aggressive-highlight` — Aggressive highlighted; at `2026-10-11T03:27:04Z`.
- `S1-aggressive-review-top` — Aggressive review, preflight passed, controls ready 6/6; at `2026-10-11T03:27:04Z`.
- `S1-aggressive-saved` — Aggressive saved and restart offered; at `2026-10-11T03:27:11Z`.
- `S1-security-aggressive-active` — after restart, active session Aggressive and Core level Aggressive; at `2026-10-11T03:28:25Z`.
- `S1-security-inspector-aggressive` — Aggressive live policy, `broker_model_auth on`, broker installed/configured; at `2026-10-11T03:29:49Z`.
- `AGG28-prompt-result-after-30s` — first attempted-condition prompt returned `pong`; at `2026-10-11T03:31:20Z`.
- `AGG28-second-prompt-result` — second attempted-condition prompt returned `pong`; at `2026-10-11T03:32:54Z`.

### Security level at transitions

- Initial picker: `Active in this session: Permissive`.
- Immediately after Aggressive save: screen said current session remained Permissive while Core's level became Aggressive.
- After product restart via `r`: `Active in this session: Aggressive`, `Core's level: Aggressive`.
- Both later restart attempts for attempted broker sabotage: `Active in this session: Aggressive`, `Core's level: Aggressive`.

### Model refusals / declines

- None observed. Both prompt turns completed with `pong`; no broker refusal or `broker_model_auth` refusal was displayed.

### Ambiguity / stopping reason

- The exact harness control for “broker prevented from starting” was not supplied in the case text or `agms-tui help`. Only generic synthetic environment variables were available through the launcher.
- Two neutral `--env` attempts did not prevent broker-backed model completion. Therefore the required starting condition could not be established, and no valid assessment of refusal or relay behavior was possible.
- The live UI itself did not expose a direct relay request/non-request observation. Relay evidence would have depended on the expected refusal screen.

### Expected-result outcomes

- Expected 1 — request refused and relay shows no direct request: **blocked**. Required broker-failure precondition was not established; the two observed attempts instead completed with `pong`.
- Expected 2 — message names `broker_model_auth` and how to change it: **blocked**. No refusal was displayed because the required broker-failure precondition was not established.

### Case verdict

VERDICT AGG-28: blocked
