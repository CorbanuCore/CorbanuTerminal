# Group G17 independent functional execution report

- Session UTC start: 2026-10-12T08:58:39Z (executor preflight completion)
- Candidate: Corbanu Terminal 0.1.48 at `/opt/agms/pkg/bin/corbanu`
- Candidate SHA-256 observed: `9fe390351b2e2bd08fad3b210697f0e95156910baa829d1b09423008eb8be5d1`
- Frozen cases SHA-256 observed: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`
- Amendments/new cases SHA-256 observed: `ec4bf3ece913e36f2c24bd7786750b3f6af604644da7f1fcb479045b88a854c9`
- Coordinator preflight summary: `ok: true`; executor `agms-probe all` result: `ok: true` (rc=0). Files: `preflight-summary.json`, `preflight-launcher.json`, `preflight-tmux-child.json`, `preflight-executor.json`.
- Assigned order: AGG-27, then AGG-A01.
- Fixture `ledger-notes` before AGG-27 recorded via `agms-tui fixture BEFORE-G17` at 2026-10-12T08:59:57Z. Relevant contents: workspace `.env` contains only the synthetic `NOTES_SYNC_TOKEN`; outside file is `outside-the-project: synthetic file`.

## AGG-27

Requested actions (frozen case):

1. On route `zai`, send prompt `Reply with exactly: pong`.
2. Open `/model`, switch to `zai-anthropic` GLM 5.3 Flash, send the same prompt.
3. Restart and repeat on each route (`zai` and `zai-anthropic`, per the group note; `zai-plain-http` excluded).
   A restart means quitting the product in its own UI and starting it again on the applicable route.

Actual actions and observations:

- 2026-10-12T09:00:43Z launched the packaged product with `agms-tui start` (route `zai`, model `glm-5.3-flash`). Screen `AGG27-START` showed a directory-trust prompt offering `1. Yes, continue` and `2. No, quit`; pressed Enter to choose the displayed `Yes, continue`.
- 2026-10-12T09:00:51Z screen `AGG27-AFTER-TRUST` showed main UI: model `glm-5.3-flash`, provider `Z.AI GLM 5.3 Flash via zai default`. The main screen did not show a security level.
- To verify the required starting condition without modifying it, sent `/security` as text (09:01:13Z, screen `AGG27-SECURITY-PRETEXT`), then pressed Enter (09:01:17Z). Screen `AGG27-SECURITY-PRE` displayed `Active in this session: Permissive`, `Permissive (active)`, and `Aggressive` as a selectable option. This contradicts the required starting condition `Aggressive active`.
- No model prompt was sent. No route switch, restart, or security change was made. 2026-10-12T09:02:14Z pressed Escape (screen `AGG27-BLOCKED-MAIN`) and stopped the product session at 09:02:15Z.

Security level at observed transition: `Permissive` (required start was `Aggressive`).

Model/provider choices: `glm-5.3-flash` / `Z.AI GLM 5.3 Flash via zai default` before blocking. Later choices not exercised.

Fixture before/after: fixture before recorded as `BEFORE-G17`; no fixture-changing action was requested or performed. No post-state comparison is needed.

Expected results:

- Every turn completes with `pong`: **not exercised** — case was stopped before any model prompt because the required starting security level was absent.
- No broker or refusal error appears: **not exercised** — no model turn was run.
- Model and provider choices are unchanged by the level: **not exercised** — the required starting level and later route choices were not reached.

Outcome: **blocked**, because the visible `/security` screen showed the required start state was missing (`Permissive` instead of `Aggressive`).

## AGG-A01

Requested actions (new case):

1. On fresh Permissive profile on route `zai-anthropic` with model `glm-5.2`, send prompt `Reply with exactly: pong`.
2. Open `/security`, choose `Aggressive`, confirm with Enter, restart with `r`.
3. Send the same prompt.

Actual actions and observations:

- 2026-10-12T09:03:10Z prepared a fresh profile by moving the prior product home to `product-home-before-A01`, then copying only its prior `config.toml` into a new empty `product-home` (contents not printed). The first launch with the case command failed twice with launcher output `tmux error: no server running on .../tmux.sock` before `config.toml` was present. After the fresh profile contained only `config.toml`, the exact case launch command succeeded.
- 2026-10-12T09:04:11Z launched with `agms-tui start --route zai-anthropic --model glm-5.2`. Screen `A01-START` showed model `glm-5.2` and provider `Z.AI GLM 5.2 via zai-anthropic standard`. No trust prompt appeared. Fixture before A01 was recorded as `BEFORE-A01`.
- 2026-10-12T09:04:21Z used `/security` to verify starting state. Screen `A01-SECURITY-PRE` showed `Active in this session: Permissive` and `Permissive (active)`, matching the starting condition. Pressed Escape at 09:04:27Z.
- Turn 1: 2026-10-12T09:04:34Z typed `Reply with exactly: pong` (screen `A01-PROMPT1-TEXT`) and pressed Enter at 09:04:41Z. At 09:05:18Z, screen `A01-TURN1-30S` showed the completed assistant bullet `• pong`. Security level remained `Permissive` at the transition.
- Security change: opened `/security` again at 09:05:30Z (`A01-SECURITY-MENU2`), pressed Down twice at 09:05:38Z so screen `A01-SECURITY-AGGRESSIVE-SELECTED` highlighted `Aggressive`, and pressed Enter at 09:05:45Z. Screen `A01-SECURITY-CONFIRM` showed a detailed `Switch to Aggressive?` dialog and offered `enter confirm and save`, `esc back`, and `i inspect`. Pressed Enter at 09:05:52Z. Screen `A01-SECURITY-CONFIRMED` showed `Saved: Aggressive (nested agents: refuse)`, that full Aggressive settings start when restarted, and offered `r restart now`. Pressed `r` at 09:05:58Z.
- 2026-10-12T09:05:58Z screen `A01-RESTARTED` showed a new main screen on model `glm-5.2` and provider `Z.AI GLM 5.2 via zai-anthropic standard`. To verify transition state, `/security` at 09:06:08Z showed screen `A01-SECURITY-POST`: `Active in this session: Aggressive`, `Core's level: Aggressive`, and `Aggressive (active)`. Pressed Escape at 09:06:16Z.
- Turn 2: 2026-10-12T09:06:22Z typed the identical prompt (screen `A01-PROMPT2-TEXT`) and pressed Enter at 09:06:25Z. At 09:07:25Z, screen `A01-TURN2-60S` showed the completed assistant bullet `• pong`. Security level shown after restart was `Aggressive`.
- No refusal, decline, missing-output-limit error, or other error text was visible. Fixture after A01 was recorded as `AFTER-A01` at 2026-10-12T09:07:34Z and matched the before record. Product session then stopped at 09:07:34Z.

Security levels: `Permissive` before the change; `Aggressive` after confirming/restarting.

Fixture before/after: unchanged; `BEFORE-A01` and `AFTER-A01` show the same workspace and outside-notes contents, including only the synthetic `.env` token.

Expected results:

- Both turns complete with `pong`: **passed** — turn 1 visibly completed with `pong`, and turn 2 visibly completed with `pong`.
- No error about a missing output limit and no refused request: **passed** — no such error or refusal was visible on either completed turn or transition screens.

Outcome: **passed**.

## Final verdicts

- VERDICT AGG-27: blocked
- VERDICT AGG-A01: passed
