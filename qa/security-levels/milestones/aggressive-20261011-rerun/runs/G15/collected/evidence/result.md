# Group G15 functional execution report

Executor: independent functional executor (not implementer/reviewer; no human acceptance claimed).  
Candidate identity: `corbanu 0.1.48`; binary SHA-256 `9fe390351b2e2bd08fad3b210697f0e95156910baa829d1b09423008eb8be5d1` (verified).  
Frozen cases SHA-256: verified. Amendments/new cases SHA-256: verified.  
Coordinator preflight: launcher/tmux-child summaries report `ok: true`; executor preflight (`preflight-executor.json`) reported overall `"ok": true` and rc=0.  
Assigned cases and order: AGG-25, AGG-34. No amendment headings were found for these case IDs.

All times are UTC unless otherwise noted.

## AGG-25

Requested starting condition: fresh profile launched with an unmanaged secret in config, using the E1 launch argument `-c 'cli_auth_credentials_store="file"'` after the agms-tui options.  
Actual setup: fixture recorded as `AGG25-before`; launch at 08:31:11Z with `agms-tui start -c 'cli_auth_credentials_store="file"'`. The trust prompt appeared; I selected the displayed “Yes, continue” with Enter.

Requested actions:
1. `/security`, Down, Down, Enter.
2. Scroll down, press Enter.
3. Fix the cause as instructed, relaunch, retry.

Actual actions:
- 08:31:33Z: typed `/security` and pressed Enter separately. Picker showed `Active in this session: Permissive` (`AGG25-security-before-downs`).
- 08:31:59Z: pressed Down twice; picker highlighted Aggressive (`AGG25-aggressive-highlight`); pressed Enter.
- 08:31:59Z: Aggressive review appeared, but contrary to E1 it showed `Preflight passed: no known raw-secret route stays open to agents or the model.` It did not show the E1 `✗ launch flags cli_auth_credentials_store: unmanaged secret` line (`AGG25-E1-live`).
- 08:33:16Z: pressed Down to scroll, pressed Enter. The product showed `Saved: Aggressive (nested agents: refuse)` and stated `Core's level is Aggressive now in this session, and from every start` (`AGG25-after-save`).
- 08:33:29Z: pressed `r` (offered restart path). Product returned to the main prompt (`AGG25-after-restart`).
- Retry: typed `/security` and pressed Enter separately. Picker showed `Active in this session: Aggressive` and `Core's level: Aggressive`, with `> Aggressive (active)` (`AGG25-retry-security`).

Positive visible checkpoints:
- Initial picker: `Active in this session: Permissive` (`AGG25-security-before-downs`, 08:31:33Z).
- Aggressive review controls ready `6 of 6`; but preflight passed, not blocked (`AGG25-E1-live`, 08:31:59Z).
- Save succeeded: `Saved: Aggressive (nested agents: refuse)` (`AGG25-after-save`, 08:33:16Z).
- After restart/retry: `Active in this session: Aggressive`, `Core's level: Aggressive` (`AGG25-retry-security`, 08:33:37Z approx.).

Security level shown at transitions: Permissive before choosing Aggressive; Aggressive immediately after save in the current session; Aggressive after restart and retry. No E1-style blocked state was observed.

Fixture: `AGG25-before` captured at 08:31:11Z. `AGG25-after` captured at 08:34:39Z; contents matched the before capture exactly.  

Product/model refusals or declines: none observed.  
Cancellation/recovery/continuation paths observed: review offered `↑/↓ scroll · n refuse/pass · enter confirm and save · esc back, nothing changes · i inspect`; save result offered `r restart now · enter or esc close · i inspect`. No credential or system password prompt appeared.

Expected-result outcomes:
- “Preflight blocked is shown with a specific fix”: **failed** — the specified launch condition did not produce preflight blocking; visible preflight passed, with no specific fix shown.
- “Enter does not save and the picker still shows Permissive”: **not exercised** — preflight did not block, and Enter saved Aggressive.
- “After the fix the preflight passes and saving works”: **inconclusive** — preflight passing and save/restart/retry succeeded visually, but the E1 cause was never displayed and no instructed cause-specific fix was available.

Outcome summary: failed  
Reason: literal E1 precondition was not reproduced through the exact supplied launch argument; the product showed preflight passed and saved immediately.

## AGG-34

Setup for the requested fresh profile: after stopping AGG-25, I reset `$AGMS_RUN/product-home` by preserving only the supplied `config.toml`, removing other generated profile state, and restoring that same `config.toml`. This is an executor environment reset, not a product action. The fixture was then captured and the product launched at 08:35:45Z without the AGG-25 launch argument.

Requested starting condition: fresh profile; Aggressive review open.  
Requested actions:
1. Press n, Enter, r.
2. `/security`, i.

Actual actions:
- 08:35:45Z: launched fresh-profile product. It opened directly at the main prompt (no trust prompt was displayed on this launch). Initial fixture captured as `AGG34-before`.
- Navigated with `/security` text + separate Enter, Down, Down, Enter. Picker showed `Active in this session: Permissive`; Aggressive review opened (`AGG34-security-picker`, `AGG34-aggressive-highlight`, `AGG34-review-open`).
- 08:36:12Z: pressed `n`; review changed from `Nested agents: refuse. …` to `Nested agents: pass. … interactive sessions, app-server and mcp-server stay refused.` (`AGG34-after-n`).
- 08:36:12Z: pressed Enter; product showed `Saved: Aggressive (nested agents: pass)` (`AGG34-saved-pass`).
- 08:36:19Z: pressed `r`; product restarted to the main prompt (`AGG34-after-restart`).
- Typed `/security` and pressed Enter separately. Picker showed `Active in this session: Aggressive`, `Core's level: Aggressive`, `> Aggressive (active)` (`AGG34-security-picker-restart`).
- Pressed `i`; Security inspector opened read-only (`AGG34-inspector`). Scrolled with Down through the inspector; at the Agents section it showed `Saved: Aggressive [configured: /security]` earlier and `Nested agents: pass: ... interactive sessions, app-server and mcp-server stay refused [configured: /security]` (`AGG34-inspector`, through `AGG34-inspector-down23`).

Positive visible checkpoints:
- Review state after `n`: `Nested agents: pass.` and the same refused remainder listed above (`AGG34-after-n`, 08:36:12Z).
- Save confirmation: `Saved: Aggressive (nested agents: pass)` (`AGG34-saved-pass`, 08:36:12Z).
- Restart/retry picker: `Active in this session: Aggressive`, `Core's level: Aggressive` (`AGG34-security-picker-restart`, 08:36:25Z approx.).
- Inspector: `Saved: Aggressive [configured: /security]` (`AGG34-inspector`, 08:36:28Z approx.).
- Inspector nested setting: `[set] Nested agents: pass: ... interactive sessions, app-server and mcp-server stay refused [configured: /security]` (`AGG34-inspector-down22`, 08:37:47Z approx.).

Security level shown at transitions: Permissive before selection; Aggressive after save; Aggressive after restart and in inspector.

Fixture: `AGG34-before` and `AGG34-after` captured; contents matched exactly.  
Product/model refusals or declines: none observed.  
Cancellation/recovery/continuation paths observed: review offered `↑/↓ scroll · n refuse/pass · enter confirm and save · esc back, nothing changes · i inspect`; save result offered `r restart now · enter or esc close · i inspect`; inspector offered `↑/↓ scroll · r refresh · esc back to /security`. No credential or system password prompt appeared. The inspector later displayed an age-related degraded/stale banner and offered `r` refresh; it remained read-only. This was recorded as a visible state observation but was not a requested model decline.

Expected-result outcomes:
- “The saved line shows ‘nested agents: pass’ and the inspector agrees”: **passed** — the save line and inspector both explicitly showed `nested agents: pass` / `Nested agents: pass`.
- “The inspector’s list of refused nested commands matches the review’s list”: **passed** — both listed `interactive sessions`, `app-server`, and `mcp-server` as remaining refused under nested agents pass.

VERDICT AGG-34: passed  
Reason: both expected results were visibly satisfied in the live UI.

## Final verdicts

VERDICT AGG-25: failed  
VERDICT AGG-34: passed

No human acceptance is claimed.
