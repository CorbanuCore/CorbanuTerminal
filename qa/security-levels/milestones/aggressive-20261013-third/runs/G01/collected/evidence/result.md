# Group G01 functional execution report

## Identity and isolation preflight

- Candidate: `/opt/agms/pkg/bin/corbanu`; SHA-256 `ad288be98d80d5eb164c2fb34a040565dcc7590a12c58ab2ac3aaf25f0675043` (expected and observed). `--version` in preflight: `corbanu 0.1.48`.
- Frozen cases SHA-256: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4` (expected and observed).
- Amendments/new cases SHA-256: `c184f1bda104aa521441f3a34b2599c8bdb57335395abe188300915c0c6101b9` (expected and observed).
- Coordinator files checked: `preflight-summary.json`, `preflight-launcher.json`, `preflight-tmux-child.json`; all reported `ok: true` with no failed probes.
- Executor probe command (`agms-probe all > evidence/preflight-executor.json; echo rc=$?`) returned rc=0; executor file reports `ok: true`.
- No case action bypassed permission mechanisms. No source, history, old findings, or credentials were read.

## AGG-01

- Requested: S0; `/security`; Down Down Enter; Esc; Esc; quit and relaunch; `/security`; press `i`.
- Actual: exactly the requested actions were performed via `agms-tui text`/`key`. Launch produced the real trust prompt; Enter accepted it. No model prompt or command approval was involved.
- Security levels: S0 picker and inspector showed `Active in this session: Permissive`; after each Esc remained Permissive/nothing saved. After relaunch remained Permissive. No change was saved.
- Positive checkpoints:
  - `S0-initial`: trust prompt offered `1. Yes, continue` and `2. No, quit`; Enter selected the displayed Yes control.
  - `S0-after-trust`: product v0.1.48 loaded in `fixture/workspace`, model `glm-5.3-flash` via `zai`.
  - `AGG01-security-open` (~17:21Z): picker showed `Active in this session: Permissive`, `Permissive (active)`, `Moderate not available yet`, and Aggressive.
  - `AGG01-aggressive-review-before-esc` (~17:22Z): Aggressive review opened with a passed preflight and `Controls ready: 6 of 6`; no save occurred before Esc.
  - `AGG01-after-first-esc` (~17:23Z): returned to picker and said `Cancelled. Nothing changed.` while still Permissive.
  - `AGG01-after-second-esc` (~17:23Z): returned/closed to main screen.
  - Product was quit with `/quit` and relaunched (~17:23–17:24Z). `AGG01-relaunch-security-picker` showed Permissive.
  - `AGG01-relaunch-inspector` (17:24Z) showed `Saved: Permissive (nothing saved)`, `Active this run: Permissive`, and `Live policy: Permissive in force; next start Permissive`.
- Expected-result outcomes:
  - Picker initially showed Permissive and Moderate `not available yet`: **passed**.
  - Each Esc returned/closed without saving: **passed**.
  - After relaunch, Saved line was `Saved: Permissive (nothing saved)`: **passed**.
  - Session rows matched screen 03: **failed**. The Vault session row differed from screen 03. The live row says “only allowlisted environment variables reach agent commands (for example PATH, HOME, locale, terminal, proxy and tool locations; secretless_agent_launch); set others explicitly…” while screen 03 says “secret-like environment variables are passed through (KEY, SECRET, TOKEN, VAULT, PASSWORD, PASSPHRASE, CREDENTIAL)”. All other visible session rows were substantively the same (path differs only as expected for this run). No AGG-01 amendment changes this expectation, so the visible row is a literal mismatch.
- No product/model decline occurred. No Keychain/system password prompt appeared.
- Evidence labels: `S0-initial`, `S0-after-trust`, `AGG01-security-open`, `AGG01-aggressive-review-before-esc`, `AGG01-after-first-esc`, `AGG01-after-second-esc`, `AGG01-relaunch-security-picker`, `AGG01-relaunch-inspector`. Fixture before: unchanged fixture snapshot at 17:21:22Z; no fixture action was requested or performed.

## AGG-02

- Requested: fresh Permissive profile; `/security`, Down Down, Enter; scroll to the bottom with Down.
- Actual: performed exactly. Because AGG-01 left the read-only inspector open, first pressed Esc twice (17:24:54Z) as neutral navigation back to the main screen, then the requested actions. Down was pressed 35 times (17:25:08–17:25:14Z) until the bottom text and footer were stable.
- Security levels: picker showed `Active in this session: Permissive` before selection. The review was still unsaved. Product was not confirmed to Aggressive.
- Positive checkpoints:
  - `AGG02-before-actions` (17:24:54Z): main screen, Permissive.
  - `AGG02-review-top` (17:24:58Z): Aggressive review top.
  - `AGG02-review-bottom` (17:25:14Z): bottom reached. It showed:
    - `now` and `Aggressive` lines for sandbox, approvals, network, vault/model environment, model keys, and child agents; nested agents showed its fixed `refuse` rule (matching screen 05, which also has no `now` row for nested agents).
    - Protected-read language: credential files include “also ones created later”; project `.codex` and `.agents` remain read-only “even with Grant once”; reads/Corbanu state are “never granted, not even once”; Permissive is the way to allow it.
    - Environment language: only allowlisted variables reach agent commands in Permissive, and Aggressive removes secret-like/non-allowlisted variables.
    - `Preflight passed`; `Controls ready: 6 of 6` with the six names.
    - Explicit now/restart split: Core protected-path/vault-command denial and sensitive-surface behavior applies now; sandbox, approvals, network/web, environment, model-key broker, and child-agent rules start at restart.
    - Footer: `enter confirm and save · esc back, nothing changes · n refuse/pass · i inspect`.
- Expected-result outcomes:
  - Each displayed setting shows now/Aggressive where applicable (with screen-05’s nested-agent format): **passed**.
  - Preflight result and controls-ready count shown: **passed**.
  - Now/restart split shown and names the required parts: **passed**.
  - Footer offers Enter, Esc, n and i: **passed**.
- No model decline. No fixture changes. Profile remained Permissive because Enter was not pressed in this case. Evidence fixture before/after unchanged at 17:21:22Z.

## AGG-03

- Requested: fresh profile, Aggressive review open with preflight passed; Enter; r; `/security`; `/quit`, relaunch same folder, `/security`; press i.
- Actual: the AGG-02 review remained open with preflight passed on the fresh profile, so its state satisfied the starting condition; then performed exactly the requested actions. `/quit` was sent as text and Enter separately; `agms-tui stop` ended the launcher’s tmux after the product exited, then relaunched.
- Security levels: before Enter Permissive. At Enter, Core/session state changed to Aggressive. After immediate restart and later relaunch, picker and inspector showed Aggressive.
- Positive checkpoints:
  - `AGG03-after-enter` (17:26:00Z): `Saved: Aggressive (nested agents: refuse)`; `Press r to restart now`; Core Aggressive now; listed restart-only rules.
  - `AGG03-after-restart` (17:26:05Z): product restarted itself in `~/runs/agms-G01-agmx1-20261013T171809Z/fixture/workspace` with model loading then `glm-5.3-flash`.
  - `AGG03-picker-after-restart` (17:26:12Z): `Active in this session: Aggressive`, `Core's level: Aggressive`, `> Aggressive (active)`.
  - `AGG03-relaunch-main` (17:26:23Z): after quit and relaunch, same `fixture/workspace`.
  - `AGG03-relaunch-picker` (17:26:30Z): Aggressive active again.
  - `AGG03-relaunch-inspector` (17:26:30Z): `Saved: Aggressive`, `Active this run: Aggressive`, `Live policy: Aggressive in force; next start Aggressive`.
- Expected-result outcomes:
  - `Saved: Aggressive` and restart offered: **passed**.
  - App restarted in same folder: **passed**.
  - Picker shows Aggressive active after restart and relaunch: **passed**.
  - Inspector shows Saved Aggressive, live policy in force, next start Aggressive: **passed**.
- Observation ambiguity: inspector header said `◐ Aggressive set; off or not observed: Credential broker`, while the Model keys row said keys are held by the isolated credential broker. This was not part of the case’s expected results and is preserved as an observed ambiguity, not interpreted.
- No model decline. No fixture changes. Evidence fixture before/after unchanged at 17:21:22Z.

## AGG-35

- Requested: fresh Permissive profile; `/security`, Down to Moderate, Enter.
- Actual: performed exactly after neutral setup. Product session from AGG-03 was stopped. The prior product profile was moved aside inside `$AGMS_RUN/tmp`; a new `product-home/config.toml` was written with the supplied feature switches only. The test project itself was unchanged (`git status --short` produced no output). Launch then showed the real trust prompt and Enter accepted it.
- Security levels: before action, `Active in this session: Permissive`; after selecting Moderate, still `Active in this session: Permissive` and `Permissive (active)`.
- Positive checkpoints:
  - `AGG35-initial` (17:28:03Z): fresh-profile trust prompt.
  - `AGG35-after-trust` (17:28:09Z): main screen in `fixture/workspace`.
  - `AGG35-picker` (17:28:09Z): `Active in this session: Permissive`; `Moderate not available yet`.
  - `AGG35-moderate-enter` (17:28:16Z): after Down/Enter, `Moderate not available yet` remained highlighted; descriptive text said `Not available yet.` and `Moderate is not available yet.`; Permissive remained active.
  - Neutral verification: Escape then reopening `/security` at `AGG35-after-close-reopen` (17:28:27Z) still showed Permissive active and no saved Moderate.
- Expected-result outcomes:
  - Nothing is saved and the screen says not available yet: **passed**.
- Fixture before/after: no relevant change; `AGG35-after` (17:28:33Z) captured the same visible project/outside files as the initial fixture, and `git status --short` was clean. The evidence capture contains `.env` contents as a fixture record, but this report does not reproduce the token value. No case granted the token.
- No model decline. No Keychain/system password prompt appeared.

## Run-level observations

- No case asked for or executed a model prompt/command approval; therefore no product-model decline occurred.
- All actual navigation was performed through `agms-tui` keys/text with text and Enter separate. Approval prompts were not present except the initial trust choice, where Enter selected the displayed Yes option; no text was typed into a prompt.
- No system password or macOS Keychain prompt appeared.
- The only security-level save was AGG-03's requested Aggressive save. AGG-35 was run afterward on a newly reset profile.

## Final verdicts

VERDICT AGG-01: failed
VERDICT AGG-02: passed
VERDICT AGG-03: passed
VERDICT AGG-35: passed
