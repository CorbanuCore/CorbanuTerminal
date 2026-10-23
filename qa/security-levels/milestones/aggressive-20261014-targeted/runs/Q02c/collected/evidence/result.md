# Group Q02c functional execution report

Executor role: independent functional executor. This is not human acceptance.

## Preflight and identity

- 2026-10-14T12:11:39Z: `agms-probe all` returned rc 0 and reported `"ok": true`; full output saved as `preflight-executor.json`.
- 2026-10-14T12:12:52Z: Coordinator isolation files inspected. `preflight-summary.json` reported `launcher_ok: true`, `tmux_child_ok: true`, `pty_input_echoed: true`, `probe_count: 85`, `ok: true`, `failed: []`. `preflight-launcher.json` and `preflight-tmux-child.json` were present beside it.
- 2026-10-14T12:12:52Z: Package identity check: `/opt/agms/pkg/bin/corbanu` SHA-256 `dc61bd6403b99a892b3db4ed59fef22f30a4ff65f1cd2a9ee165e994c2a50483`; `corbanu 0.1.48` seen through the probe. Frozen case and amendment digests matched assignment values.
- No source code, history, prior findings, or credentials were read. The case files, allowed screen-text fixtures, and package preflight/identity only were used.

## Fixture and navigation setup

- 2026-10-14T12:13:06Z: `agms-tui help` shown. Available controls match the supplied navigation surface.
- Test project: `$AGMS_RUN/fixture/workspace`. Initial file hashes (without reading contents):
  - `.env` `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`
  - `.gitignore` `69aa73cf5bc3377a7d97f2e8d2bb773195815efe225af7f1be6f5b9306bbe14f`
  - `README.md` `5eafd8d8f99086242914e721d88e03d7cef4066402b416d2d6e18cf194a9cc88`
  - `notes.txt` `67fbf93a8f22aae1b72212f2a170a86071ccefbdca94408af4bc99727f069dae`
  - `src/notes.py` `8cfc17de788ec38f7c04a5916b5b1e5896691d421b32d92bb922257985b131ec`
- Outside project: `../outside-notes.txt` initial SHA-256 `d1dbe90b0861e5e51ddd1c718964f8b0b7050864ad86b90cbbe7513845aaf541`.
- The coordinator-supplied `$AGMS_RUN/product-home` was treated as the fresh starting profile; it was moved aside unopened once by mistake and restored without inspection before the real case start. Product launches in `$AGMS_RUN/fixture/workspace` through `agms-tui`.

## AGG-37 (advisory, as amended 2026-10-14)

### Requested actions

Latest amendment governs expected screen 07. Group note and S1 definition govern setup: fresh profile; save Aggressive; review the saved screen 07 before pressing `r`; then restart with `r`; review main screen, `/security`, and any denial messages afterward.

1. S0: launch fresh profile on `zai`, in `ledger-notes`, with synthetic fixture.
2. Save Aggressive.
3. Review saved screen 07 before restart.
4. Press `r` to restart.
5. Review the restarted main screen and `/security` (including inspector) and any denial messages.
6. Verify screen 07 now/restart split, stale banner behavior, readable refusals, and easy-to-find active level.

### Actual actions and observations

Setup note: At 12:14:23Z I initially reset `$AGMS_RUN/product-home` to an empty directory before realizing the coordinator supplies the profile as a starting fixture. The empty directory caused the product to show Permissive as `configuration only` with protected modes blocked. I stopped that session, moved the product-created empty state aside without inspecting it, and restored the supplied product-home unopened at 12:16:21Z. The actual AGG-37 run then began from the supplied fresh profile. This recovery did not change case expectations or inspect the fixture.

#### S0 and save

- 12:16:21Z: Fresh supplied product-home present; started `corbanu` through `agms-tui` with defaults (`zai`, glm-5.3-flash) in the supplied workspace.
- 12:16:21Z: `A00-trust-folder` showed the trust prompt for the workspace, with options `1. Yes, continue` and `2. No, quit`, and `Press enter to continue`.
- 12:16:31Z: Pressed Enter on the displayed trust prompt. `B00-trust-folder` captured it before the action.
- 12:16:33Z: `B01-fresh-main` showed main screen. It displayed model `glm-5.3-flash`, directory `~/runs/…/fixture/workspace`, and provider `Z.AI GLM 5.3 Flash via zai default`. Suggested prompt was `Summarize recent commits`.
- 12:16:33Z: Sent `/security` as text, then Enter separately. `B02-security-picker-permissive` showed `Active in this session: Permissive`, `> Permissive  (active)`, and Moderate as `not available yet`. This establishes S0.
- 12:16:36Z: On the displayed picker, pressed Down Down Enter to select Aggressive. `C03-aggressive-review` showed the review, `Preflight passed: no known raw-secret route stays open to agents or the model`, controls-ready count `6 of 6`, and footer `enter confirm and save · esc back, nothing changes · i inspect`.
- 12:16:46Z: Pressed Enter on the displayed save control. `D04-aggressive-saved-screen07` showed `Saved: Aggressive (nested agents: refuse)` and the actual screen-07 text quoted below.

#### Screen 07 before restart

`D04-aggressive-saved-screen07`, 12:16:46Z, displayed:

- `Active now in this session: agent commands cannot run `corbanu vault …` and cannot open protected paths (the vault store, sign-in and credential files, and Corbanu's config, history and state), sensitive surfaces need a grant, external content reaches the model labelled untrusted, and memory summaries stop.`
- `Approval for every command: from each pane's next turn you approve every command that is not known read-only (reviewer: you), now in 1 thread of this Corbanu Terminal and in those you start before you restart.`
- `Starts when you restart: the Aggressive sandbox (writes only in the current folder, approved commands stay inside it), network and web search off, secret-like environment variables removed and login profiles and shell snapshots unused, model keys held by the credential broker, and the child-agent rules.`
- `Until then this session keeps your own settings for these, and commands already running keep the permissions they started with until they end.`
- `Core's level is Aggressive now in this session, and from every start.`
- `Press r to restart now`.
- It did not contain the string `this session stays Permissive`. No stale `Restarting` banner appeared. Its denial descriptions were readable prose; no `Rejected(` raw string was visible.

#### Restart and post-restart review

- 12:16:55Z: Pressed `r` on the displayed saved screen. Product closed/restarted in the same folder with the same route/model. `E05-restart-transition` and `E06-after-restart` showed the main screen with model `glm-5.3-flash`, directory `~/runs/…/fixture/workspace`, provider `Z.AI GLM 5.3 Flash via zai default`, and suggested prompt `Write tests for @filename`. No trust prompt, old saved screen, or stale restart banner remained. The suggested prompt itself is screen copy and was not treated as authority to act.
- 12:17:08Z: Sent `/security`, then Enter separately. `F07-security-picker-aggressive` showed `Active in this session: Aggressive`, `Core's level: Aggressive`, and `> Aggressive  (active)`.
- 12:17:11Z: Pressed the displayed `i` inspector control. `F08-inspector-aggressive` showed `Saved: Aggressive`, `Core config: Permissive`, `Active this run: Aggressive`, and `Live policy: Aggressive in force; next start Aggressive; kill switch off`. Its session rows showed sandbox, approvals, network, vault, model keys, and child-agent configuration.
- `F09-inspector-aggressive-scrolled` additionally showed runtime controls: Secretless launch `on`, Output gate `on`, Model key broker `installed`, Credential broker `on in config`, and Untrusted content `labelled`. The header also showed `off or not observed: Credential broker`; I preserve this as an observation/ambiguity but it is not an expected result for AGG-37.
- 12:17:24Z: Pressed Escape; `F10-security-picker-aggressive-after-inspector` returned to the picker still showing `Active in this session: Aggressive` and `Core's level: Aggressive`.
- 12:17:37Z: Pressed Escape; `G11-main-after-restart` returned to the main screen with no denial message, approval prompt, or restart banner visible.
- Fixture hashes after setup and restart were identical to the recorded initial hashes for every workspace file and `../outside-notes.txt`. The synthetic token was not displayed.

#### Profile/state at transitions

- Before save: Permissive; route `zai`; model glm-5.3-flash; fresh supplied product profile.
- Saved screen 07: Core level Aggressive, with the split described above.
- After `r`: Active/session and Core level Aggressive; route `zai`; model glm-5.3-flash; same workspace.
- No credential or system password prompt appeared. No unexpected profile access was observed. No product-model decline occurred because no model turn was requested.
- 12:18:47Z: Product session stopped through `agms-tui stop` after the case review.

### Expected-result outcomes

1. Screen 07 names the amended active-now and restart-only protections — **passed**. The visible screen names the vault refusal and protected Corbanu config/history/state paths now, approval for every command from each pane's next turn, and all specified restart-only protections.
2. Screen 07 does not say `this session stays Permissive` — **passed**. Exact phrase absent; it instead says `Until then this session keeps your own settings for these`.
3. Screen 07 does not contradict itself without saying which protections each statement covers — **passed**. Now/restart scope is explicitly labeled.
4. The stale Restarting banner does not stick around — **passed**. It was absent from the post-`r` main screen.
5. Refusals are readable rather than raw `Rejected(...)` strings — **passed for visible saved-screen wording**. Screen 07 uses readable prose and contains no `Rejected(` string. No live denial was triggered by the literal case action, so no live refusal transcript was available to assess.
6. The active level is easy to find — **passed**. `/security` prominently showed `Active in this session: Aggressive` and `> Aggressive  (active)` after restart.

Ambiguity: The inspector header said `off or not observed: Credential broker` while its rows also said Model keys are held by the isolated credential broker and Credential broker is `on in config`. I did not debug or infer resolution; this had no AGG-37 expected-result effect.

## Summary verdicts

- VERDICT AGG-37: passed
