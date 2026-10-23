# G02 functional executor report

- Candidate: Corbanu Terminal 0.1.48, `/opt/agms/pkg/bin/corbanu`.
- Identity: SHA-256 observed and matched `9fe390351b2e2bd08fad3b210697f0e95156910baa829d1b09423008eb8be5d1`.
- Frozen cases SHA-256 observed and matched `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`; amendments/new-cases SHA-256 observed and matched `ec4bf3ece913e36f2c24bd7786750b3f6af604644da7f1fcb479045b88a854c9`.
- No amendment or new-case heading named either assigned case.
- Frozen cases digest matched; amendments/new-cases digest matched.
- Isolation preflight: coordinator summary and both sidecar preflight files were present and `ok: true`; executor ran `agms-probe all`, rc `0`, and `ok: true`.
- Execution method: only `agms-tui`; literal prompt text and Enter were sent separately.
- Scope note: this is automated functional evidence, not human acceptance.
- Case order: AGG-29, then AGG-03.

## AGG-29
- Status: completed.

### Setup performed for AGG-29 (not a literal case action, required by starting conditions)
- Fixture recorded before setup (`before-AGG-29`); notes.txt contained `buy coffee` and `review budget`; no fixture content changed.
- 05:21:56Z: Started product; trust prompt shown; pressed Enter on `1. Yes, continue`.
- 05:22:05Z: Main screen displayed at Permissive (no level field on main, level confirmed on `/security`).
- 05:22:15Z: Typed prompt `Say the word ready.`; 05:22:16Z pressed Enter; 05:22:49Z model responded `Ready.` (Permissive conversation recorded).
- 05:22:49Z: Opened `/security`; screen `permissive-security` showed `Active in this session: Permissive`.
- 05:22:49Z: Pressed Down Down Enter, opening the Aggressive review (`aggressive-review`).
- 05:23:10Z: Pressed Enter to confirm and save. Screen `aggressive-saved` showed `Saved: Aggressive (nested agents: refuse)` and explicitly offered `r restart now`.
- 05:23:20Z: Pressed `r`; product restarted in the same fixture workspace at 05:23:23Z (`after-aggressive-restart`).

### Requested versus actual for AGG-29
- Requested: `/resume`, choose the Permissive-era session, Enter.
- Actual: typed `/resume`, then pressed Enter; screen `resume-list` displayed the sole session, `Say the word ready.`; pressed Enter.
- Additional neutral verification: opened `/security` and pressed Enter after refusal.

### Observed checkpoints
- 05:23:41Z, `resume-result`: Product displayed verbatim: `■ Security level Aggressive: this conversation was recorded before protected mode was activated and may contain secrets. Start a new session instead.`
- 05:23:51Z, `aggressive-after-refusal`: `/security` showed `Active in this session: Aggressive`, `Core's level: Aggressive`, and `Aggressive  (active)`.
- No model refusal/decline occurred; this was the product security feature refusing resume.

### Expected-result outcomes
- Session is refused with an explanation: **passed** (verbatim explanation above).
- Current session stays Aggressive: **passed** (explicit `/security` state after refusal).

VERDICT AGG-29: passed

## AGG-03
- Status: completed.
- Starting-condition reset: after AGG-29, the product was quit with `/quit` and stopped. To satisfy the fresh-profile condition without inspecting or changing settings, the used product-home directory was moved intact under `$AGMS_RUN/tmp/product-home-used-AGG-29` for containment and a fresh product-home directory was created containing only the preserved original `config.toml`. No profile content was read. This was 05:26:05Z.
- Fixture recorded after the case (`after-AGG-03`); it was unchanged from `before-AGG-29`.

### Requested versus actual
1. Requested Enter; actual pressed Enter on the Aggressive review. (Starting condition was reached by opening `/security` on fresh profile and pressing Down Down Enter at 05:26:31Z; the review showed `Preflight passed`.)
2. Requested press `r`; actual pressed `r` on the saved screen.
3. Requested open `/security`; actual typed `/security` and pressed Enter after the product restart.
4. Requested `/quit`, relaunch in the same folder, open `/security`; actual typed `/quit` and pressed Enter, stopped the ended tmux session, started again in the same fixture workspace, then typed `/security` and pressed Enter.
5. Requested press `i`; actual pressed `i` in `/security`.

### Observed checkpoints
- 05:26:31Z, `agg03-review`: review open and visible `Preflight passed: no known raw-secret route stays open to agents or the model.`
- 05:26:35Z, `agg03-saved`: visible `Saved: Aggressive (nested agents: refuse)` and `Press r to restart now`; restart explicitly offered.
- 05:26:41Z, `agg03-after-restart`: product restarted in same fixture workspace.
- 05:26:46Z, `agg03-picker-after-restart`: `/security` showed `Active in this session: Aggressive`, `Core's level: Aggressive`, and `> Aggressive  (active)`.
- 05:27:10Z, `agg03-relaunched`: relaunched in the same fixture workspace.
- 05:27:19Z, `agg03-picker-after-relaunch`: `/security` again showed `Active in this session: Aggressive`, `Core's level: Aggressive`, and `> Aggressive  (active)`.
- 05:27:34Z, `agg03-inspector`: inspector showed `Saved: Aggressive [configured: /security]` and `[on] Live policy: Aggressive in force; next start Aggressive; kill switch off; generation 0.1 [observed: Core policy]`.
- Ambiguity preserved: the inspector header said `◐ Aggressive set; off or not observed: Credential broker`, while the session row said `[set] Model keys: held by the isolated credential broker (broker_model_auth on) [resolved: session config]`. This was not an expected result for AGG-03 and was not diagnosed.

### Expected-result outcomes
- `Saved: Aggressive` shown and a restart offered: **passed** (`agg03-saved`).
- App restarts in same folder: **passed** (`agg03-after-restart`, same fixture workspace path shown on relaunched screen).
- Picker shows Aggressive active after restart and relaunch: **passed** (`agg03-picker-after-restart`, `agg03-picker-after-relaunch`).
- Inspector shows Saved Aggressive, live policy in force, next start Aggressive: **passed** (`agg03-inspector`).
- No model refusal/decline occurred.

VERDICT AGG-03: passed

## Additional offered paths preserved
- AGG-29 setup: the Aggressive review footer offered `n refuse/pass`, `enter confirm and save`, `esc back, nothing changes`, and `i inspect`; the saved screen offered `r restart now`, `enter or esc close`, and `i inspect`; the resume list offered `enter resume`, `esc exit`, and `ctrl+c exit`; after refusal the product suggested starting a new session.
- AGG-03: the Aggressive review offered the same save/escape options above; the saved screen offered restart and close; the security picker offered `esc close`, `g grants and kill switch`, and `i inspect`; the inspector offered `r refresh` and `esc back to /security`.
