# Group G18 functional execution report

## Run preflight and identity

- Preflight executed before case work at 2026-10-13T20:59:08Z: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?` returned rc=0 and JSON `"ok": true`.
- Coordinator preflight files reviewed: `$AGMS_RUN/evidence/preflight-summary.json`, `preflight-launcher.json`, and `preflight-tmux-child.json`. All report `"ok": true`; summary says 85 probes, `failed: []`.
- Package `/opt/agms/pkg/bin/corbanu` SHA-256 matched the required `ad288be98d80d5eb164c2fb34a040565dcc7590a12c58ab2ac3aaf25f0675043`; probe reported `corbanu 0.1.48`.
- Frozen cases SHA-256 matched required value.
- Amendments/new-cases SHA-256 matched required value.

## AGG-28

### Starting conditions and setup

- A fresh product profile and the supplied fixture were used. The product was launched from `$AGMS_RUN/fixture/workspace` via `agms-tui start`.
- At 2026-10-13T21:00:29Z, the real trust screen was observed offering `1. Yes, continue` highlighted; Enter was pressed. At 21:00:30Z the main session appeared on zai with GLM 5.3 Flash. Evidence: screen `00-startup-trust.txt`, `01-after-trust.txt`.
- `/security` was sent as text, then Enter separately. The screen showed `Active in this session: Permissive` and Aggressive was selected with Down Down. Evidence: `02-before-security-enter.txt`, `03-security-initial.txt`, `04-security-aggressive-selected.txt`.
- Enter at 21:00:42Z opened the Switch to Aggressive review. A second Enter at 21:00:47Z saved it; the screen said `Saved: Aggressive (nested agents: refuse)`, described some protections as active now and others as starting at restart, and offered `r restart now`. Evidence: `05-security-aggressive-confirm.txt`, `06-security-aggressive-saved.txt`.
- Pressed `r` as displayed. At 21:00:54Z a fresh product session appeared. `/security` showed `Active in this session: Aggressive`, `Core's level: Aggressive`, and `Aggressive  (active)`. Evidence: `07-after-restart-aggressive.txt`, `08-security-verify-aggressive.txt` at 21:01:00Z.
- The product was stopped through `agms-tui stop` at 21:01:08Z, then launched through `agms-tui start --route zai-plain-http`, the harness route specified for preventing the credential broker from accepting the provider address. At 21:01:11Z the main screen showed `Z.AI GLM 5.3 Flash via zaihttp default`. `/security` again showed Aggressive active at 21:01:15Z. Evidence: `09-broker-blocked-start.txt`, `10-broker-blocked-security.txt`.

### Requested versus actual actions

| Requested | Actual |
|---|---|
| Aggressive active with broker prevented by harness | Verified Aggressive in the UI, then restarted on `zai-plain-http`; UI route label was `zaihttp` |
| Ask: `Reply with exactly: pong` | Sent exactly that text with `agms-tui text`, then pressed Enter separately at 21:01:27Z |

### Positive visible checkpoints

- 21:00:30Z — initial main session appears after trust.
- 21:00:48Z — Aggressive save message visible.
- 21:01:00Z — security screen confirms `Active in this session: Aggressive`.
- 21:01:11Z — fresh session starts on the required plain-HTTP route; status line shows `zaihttp`.
- 21:01:15Z — Aggressive remains active on that route.
- 21:01:27Z — the sent request is refused with a fatal error naming `broker_model_auth`; evidence: `12-typed-pong-before-enter.txt`, `13-pong-result.txt`.

### Security levels shown

- Initial product setup: Permissive.
- Before the restart: security screen showed Aggressive selected and saved; active Aggressive was confirmed after restart.
- On the broker-prevention start: `Active in this session: Aggressive`; `Core's level: Aggressive`.

### Fixture contents

- Fixture snapshots taken before and after using `agms-tui fixture`; files: `fixture-AGG-28-before.json` and `fixture-AGG-28-after.json`.
- Snapshots compare identical (`cmp` rc=0; both SHA-256 `4a89f346d3a644b12c68c1e01a4202b7f56b5a6e32b8c4ca464595321f07f1d8`). The synthetic `.env` remained present and unchanged; no fixture content was altered.
- The report intentionally does not reproduce the synthetic token value.

### Refusals / declines

- The product displayed a fatal refusal rather than a model response. Verbatim visible text:
  > Fatal error: the provider base URL `http://127.0.0.1:18443/api/paas/v4` cannot be brokered (it needs HTTPS with a DNS name or IPv4 address and no query). Security level Aggressive turns broker_model_auth on. To send model requests without the broker, choose Permissive in /security, or set `broker_model_auth = false` under [features] in config.toml, then restart.

  This is a product/harness refusal, not the model's own decline.

### Expected-result evaluation

1. Expected: The request is refused and the relay shows no direct request.
   - **Passed for the visible request completion/refusal:** the UI showed a fatal refusal instead of `pong` or model output.
   - **Relay component:** relay logs are not visible to the executor. No direct request was visibly initiated or completed in the transcript, and the product explicitly refused before model output. Relay confirmation is reserved for the independent evidence check.
2. Expected: The message names `broker_model_auth` and how to change it.
   - **Passed.** The visible fatal-error message names `broker_model_auth` and gives two change paths: choose Permissive in `/security`, or set `broker_model_auth = false` under `[features]` in config.toml and restart.

### Timing summary

- Request text sent at 21:01:26Z; Enter at 21:01:27Z; refusal visible immediately when checked at 21:01:37Z; post-check time 21:01:37Z.

## Summary

- AGG-28 visible execution completed successfully; no timeout occurred.
- Relay-side no-direct-request determination is not visible to this executor and depends on the independent evidence check.

VERDICT AGG-28: passed
