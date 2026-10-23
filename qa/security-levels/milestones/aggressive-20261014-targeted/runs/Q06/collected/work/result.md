# AGMS Q06 executor run — AGG-25

## Run identity and preflight

- Run started (UTC): 2026-10-14T09:17:27Z
- Executor preflight command: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`
- Result: rc 0; `agms-probe all` reports `"ok": true`.
- Coordinator isolation preflight: `$AGMS_RUN/evidence/preflight-summary.json`, `preflight-launcher.json`, and `preflight-tmux-child.json` were read; all report `"ok": true`, and no failed probes.
- Candidate: `/opt/agms/pkg/bin/corbanu`
- Candidate SHA-256: `dc61bd6403b99a892b3db4ed59fef22f30a4ff65f1cd2a9ee165e994c2a50483` (matches required value).
- Frozen cases SHA-256: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4` (matches).
- Amendments/new cases SHA-256: `072a8c3fc4910e819fbcf569fd359bfff7521f7b02587f2e272936d311da7523` (matches).
- Product version observed by preflight and live UI: `corbanu 0.1.48`.
- AGG-25 is judged against the 2026-10-14 amendment (part 5); where unmentioned, the frozen case remains. Design-time screens `new-screens-4/U10-E1-flag-preflight-today.txt` and `new-screens-4/U11-unmanaged-secret-preflight-blocked.txt` were used for current wording.
- The executor did not read product source, prior findings, credentials, or hidden history, and did not claim human acceptance.

## Starting state and profile

- The assigned fixture was the supplied `ledger-notes` project with `README.md`, `src/notes.py`, `notes.txt`, `.env`, `.gitignore`, `.git/`, and `outside-notes.txt` immediately outside the project.
- At 2026-10-14T09:19:03Z, `agms-tui fixture pre-AGG-25` showed no user modifications. The later post-case fixture capture was byte-for-byte identical to this capture; compare `evidence/fixture-pre-AGG-25.json` and `evidence/fixture-post-AGG-25.json` (same SHA-256: `4a89f346d3a644b12c68c1e01a4202b7f56b5a6e32b8c4ca464595321f07f1d8`).
- Product home contained the supplied `config.toml` at run setup. No other product-home files were observed before the first launch. The executor changed starting conditions only by the exact case launch flags.
- No model turn was made, so there were no model refusals or declines to quote.
- No credential or system-password prompt appeared, and no permissions were bypassed.

## Case AGG-25 — requested versus actual

### Part A: store-mode flag

- Requested: launch fresh profile with the E1 flag `-c 'cli_auth_credentials_store="file"'`; `/security`, Down Down, Enter; scroll to the bottom and record preflight line and footer; press Esc without saving; quit.
- Actual: performed exactly through `agms-tui`; prompt text and Enter were separate actions; accepted the displayed trust option with Enter; used the displayed picker choices and review footer controls; pressed Esc without save and then used `/quit`.

### Part B: unmanaged secret

- Requested: after Part A quit, relaunch with `-c 'shell_environment_policy.set.SERVICE_TOKEN="synthetic-e1b-token"'`; `/security`, Down Down, Enter; scroll down, press Enter; fix the cause as instructed by removing the flag and relaunching; retry and save.
- Actual: performed exactly. On the blocked review, scrolled to the bottom and pressed Enter; the UI said `Not saved: resolve the blockers above first. Nothing changed.` Then quit, launched again with no extra launch flag, repeated `/security`, Down Down, Enter, scrolled to the bottom, and pressed Enter. The UI reported saved. Prompt text and Enter remained separate; no text was typed into an approval prompt.

## Case AGG-25 — Part A observations

| UTC time | Visible checkpoint / transition | Security level shown |
|---|---|---|
| 2026-10-14T09:19:50Z | Launched with `-c 'cli_auth_credentials_store="file"'`; trust-folder prompt showed highlighted `1. Yes, continue`. | Not shown at this transition |
| 2026-10-14T09:20:02Z | Enter accepted trust and reached `Corbanu Terminal (v0.1.48)`, model `glm-5.3-flash`, route `zai`, directory `fixture/workspace` (`A01-initial`). | Not shown on main screen |
| 2026-10-14T09:20:16Z | `/security` picker showed `Active in this session: Permissive`, with `Permissive (active)` (`A03-security-picker`). | Permissive |
| 2026-10-14T09:20:20Z | Down Down highlighted `Aggressive`; picker still showed active Permissive (`A04-aggressive-highlight`). | Permissive (still active) |
| 2026-10-14T09:20:24Z | Enter showed review with `Preflight passed: no known raw-secret route stays open to agents or the model.`, `Controls ready: 6 of 6`, and footer `enter confirm and save` (`A05-review-top`). | Review for switch; current session not yet Aggressive |
| 2026-10-14T09:20:34–09:20:43Z | Scrolled to bottom; same preflight passed line and footer confirmed (`A06-review-scrolled`, `A07-review-bottom`). Current wording also said Core protections apply as soon as confirm and approval for every command from each pane's next turn. | Review for switch; current session not yet Aggressive |
| 2026-10-14T09:20:56Z | Esc returned to picker with `Cancelled. Nothing changed.` and `Active in this session: Permissive`; `Permissive (active)` remained visible (`A08-picker-after-esc`). | Permissive |
| 2026-10-14T09:21:06Z | `/quit` hint was shown; Enter exited product normally (`A09-quit-typed`). | No saved level shown; no save was performed |

## Case AGG-25 — Part B observations

| UTC time | Visible checkpoint / transition | Security level shown |
|---|---|---|
| 2026-10-14T09:22:00Z | Relaunched after Part A with `-c 'shell_environment_policy.set.SERVICE_TOKEN="synthetic-e1b-token"'`; product started in fixture workspace without an additional trust prompt. | Not shown at start |
| 2026-10-14T09:22:15Z | `/security` picker showed `Active in this session: Permissive` and `Permissive (active)` (`B02-security-picker`). | Permissive |
| 2026-10-14T09:22:23Z | Down Down highlighted `Aggressive`; picker still showed active Permissive (`B03-aggressive-highlight`). | Permissive (still active) |
| 2026-10-14T09:22:27Z | Enter showed review with `Preflight blocked: Aggressive cannot be saved until these are resolved.` and the specific fix: `✗ launch flags shell_environment_policy.set.SERVICE_TOKEN: unmanaged secret (config literal); move it into the vault and use a reference, or remove it [config-literal-9352ebfab0e5]` (`B04-blocked-review-top`). | Review for switch; current session not yet Aggressive |
| 2026-10-14T09:22:32Z | Scrolled to bottom; footer had no `enter confirm and save` option: `↑/↓ scroll · esc back, nothing changes · i inspect` (`B05-blocked-review-bottom`). | Review for switch; current session not yet Aggressive |
| 2026-10-14T09:22:43Z | Pressed Enter at the bottom. The review remained with `Not saved: resolve the blockers above first. Nothing changed.` (`B06-after-enter-at-bottom`). | Review for switch; no save occurred |
| 2026-10-14T09:22:53Z | Esc returned to picker with `Active in this session: Permissive`, `Permissive (active)`, and `Cancelled. Nothing changed.` (`B07-picker-after-blocked-enter`). | Permissive |
| 2026-10-14T09:23:01Z | Esc closed picker, `/quit` hint shown, Enter exited product normally (`B08-quit-typed`). | No save performed |
| 2026-10-14T09:23:19Z | Relaunched with no extra launch flag, as the review's fix directed. | Not shown at start |
| 2026-10-14T09:23:38Z | Repeated `/security`, Down Down, Enter. Review showed `Preflight passed: no known raw-secret route stays open to agents or the model.` and footer `enter confirm and save` (`C03-fixed-review-top`). | Review for switch; current session not yet Aggressive |
| 2026-10-14T09:23:43Z | Scrolled to bottom; passed review and saving footer visible (`C04-fixed-review-bottom`). | Review for switch; current session not yet Aggressive |
| 2026-10-14T09:23:54Z | Enter confirmed and saved. Screen showed `Saved: Aggressive (nested agents: refuse)`, described current Core protections and approval-for-every-command, and said the sandbox/network/environment/model-key/child-agent rows start at restart (`C05-after-confirm-save`). | Saved Aggressive; Core Aggressive now; non-Core rows pending restart |
| 2026-10-14T09:24:21Z | Esc closed the saved screen and returned to the main prompt; `/quit` exited normally (`C06-picker-after-save`, `C07-quit-typed`). | Last visible saved state: Aggressive |

## Expected-result outcomes — AGG-25

- Part A, preflight for store-mode flag: **passed** — `Preflight passed` was visible for `-c 'cli_auth_credentials_store="file"'`.
- Part A, review offers confirm/save: **passed** — footer displayed `enter confirm and save`.
- Part A, Esc saves nothing and picker remains Permissive: **passed** — `Cancelled. Nothing changed.` and `Active in this session: Permissive` / `Permissive (active)` were visible.
- Part A, restart after save: **not exercised** — Part A intentionally did not save.
- Part B, blocked preflight with specific fix: **passed** — review named the exact launch flag path, described it as an unmanaged secret config literal, and said to move it into the vault with a reference or remove it.
- Part B, Enter does not save: **passed** — Enter produced `Not saved: resolve the blockers above first. Nothing changed.`
- Part B, picker still shows Permissive: **passed** — after Enter, Esc returned to `Active in this session: Permissive` and `Permissive (active)`.
- Part B, after fix preflight passes: **passed** — relaunch without the flag showed `Preflight passed`.
- Part B, after fix saving works: **passed** — Enter showed `Saved: Aggressive (nested agents: refuse)`.
- No expected result was failed, blocked, inconclusive, or left otherwise unobserved.

## Case result

VERDICT AGG-25: passed
