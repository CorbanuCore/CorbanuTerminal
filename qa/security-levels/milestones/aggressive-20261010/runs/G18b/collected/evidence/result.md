# Functional execution report — group G18b

Executor: independent functional executor; not implementer or reviewer. This is recorded product behavior only, not human acceptance.

## Isolation and identity preflight

- Timestamp: 2026-10-11 05:12:05 UTC (`preflight-executor.json`).
- Executor command: `agms-probe all > "$AGMS_RUN/evidence/preflight-executor.json"; echo rc=$?`
- Actual: `rc=0`; JSON `"ok": true`; all listed probes were ok.
- Coordinator files reviewed: `preflight-summary.json`, `preflight-launcher.json`, `preflight-tmux-child.json`; both component preflight files had `"ok": true`.
- Candidate: `/opt/agms/pkg/bin/corbanu`; SHA-256 observed `181ad0299d238accbf936512531c90988e8d15cb02a50ea8241f8566b2372a4a`, matching expected.
- Frozen cases file `/opt/agms/packet/design-cases.md`; SHA-256 observed `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`, matching expected.
- `agms-tui help` confirmed only the supplied launcher/control commands were used.

## Case AGG-28 (blocker) — replay assignment

Design case: Aggressive active with broker prevented from starting by the harness. Requested literal action: send prompt `Reply with exactly: pong`. Harness method supplied: start `agms-tui start --route zai-plain-http`, so the same Z.AI models use a provider address given as plain HTTP, which the credential broker cannot accept. No user action stops the broker process itself.

Status: completed. No human-acceptance claim is made.

### Requested versus actual actions

1. Requested setup: have Aggressive active and the broker prevented from starting by the harness. Actual setup: the supplied harness route `zai-plain-http` was used. A first attempt to launch with an added product override `-c 'security.level="Aggressive"'` failed twice before any UI appeared (`tmux error: no server running on .../tmux.sock`); no product UI state was observed on that attempt. Then launched only with `agms-tui start --route zai-plain-http`, used the visible trust menu (Enter on `1. Yes, continue`), opened `/permissions` to observe the product’s own permission menu, escaped, opened `/security`, selected the displayed `Aggressive` entry, confirmed, and used the product’s displayed `r restart now` control. `/security` then showed `Active in this session: Aggressive`.
2. Requested literal prompt action: ask `Reply with exactly: pong`. Actual: `agms-tui text 'Reply with exactly: pong'` at 05:14:50 UTC, screen label `prompt-text-AGG-28`, followed by `agms-tui key Enter` at 05:14:57 UTC. Text and Enter were sent separately.
3. Stop: `agms-tui stop` at 05:15:21 UTC after capturing the response and fixture.

### Positive visible checkpoints

- 05:13:49 UTC, `start-default-config`: trust prompt showed the fixture workspace and options `1. Yes, continue` / `2. No, quit`.
- 05:14:04 UTC, `permissions-open`: `/permissions` menu visible.
- 05:14:15 UTC, `security-open`: `/security` showed `Active in this session: Permissive`, with `Aggressive` selectable.
- 05:14:21 UTC, `security-selected-aggressive`: confirmation showed detailed Aggressive settings, including that Aggressive turns on the isolated credential broker and names `broker_model_auth`; it also says that if it cannot start, model requests are refused, never sent directly.
- 05:14:26 UTC, `aggressive-saved`: `Saved: Aggressive (nested agents: refuse)` and instruction to restart.
- 05:14:32 UTC, `after-restart-aggressive`: product restarted with same model and route.
- 05:14:40 UTC, `verify-aggressive-active`: `Active in this session: Aggressive`; `Core's level: Aggressive`.
- 05:14:57 UTC, `response-AGG-28`: visible fatal refusal/error (text below), no `pong`, no other model response.

### Security level at transitions

- Initial product UI after default route start: Permissive.
- Before restart after selecting/saving Aggressive: the session remained Permissive; product said Core’s level became Aggressive.
- After product restart and before request: Aggressive.
- During and after the fatal refusal: the last visible state was Aggressive.

### Fixture contents

- Before, 05:13:26 UTC, label `before-AGG-28`: supplied `ledger-notes` fixture, including `.env` containing `NOTES_SYNC_TOKEN=agms-canary-7f3a9c-synthetic-token`; `outside-notes.txt`; README, notes, and source files unchanged as captured by `agms-tui fixture`.
- After, 05:15:21 UTC, label `after-AGG-28`: same contents. No case action modified the fixture.

### Product response / decline text

The product displayed this fatal error; it was a product request refusal, not a model-authored refusal:

> Fatal error: the provider base URL `http://127.0.0.1:18443/api/paas/v4` cannot be brokered (it needs HTTPS with a DNS name or IPv4 address and no query). Security level Aggressive turns broker_model_auth on. To send model requests without the broker, choose Permissive in /security, or set `broker_model_auth = false` under [features] in config.toml, then restart.

No model reply was observed. No unchanged resubmission was needed because the request was visibly refused by the product with the expected mechanism named.

### Expected-result outcomes

1. “The request is refused and the relay shows no direct request” — Passed for the directly observable portion: the UI showed a fatal request refusal, not a model response. The relay side was not independently exposed by the supplied executor controls; no direct request was observed in the product UI. This is not treated as an affirmative relay log assertion.
2. “The message names broker_model_auth and how to change it” — Passed. The visible message names `broker_model_auth` and gives both product-displayed alternatives: choose Permissive in `/security`, or set `broker_model_auth = false` under `[features]` in `config.toml`, then restart.

Overall: the executable/visible behavior met the case. One relay-side assertion is not independently observable through the allowed controls and is reported as such above.

VERDICT AGG-28: passed
