You are an independent functional test designer. You have NOT seen any code, tests, implementation notes or results, and you must not ask for them. Propose test cases only; do not run anything.

## User intent (from the product owner)
Corbanu Terminal is an AI coding agent in the terminal. A user cannot easily answer "How locked down is my agent right now?". Behind a feature flag (`[features] security_levels = true` in config.toml), the user types `/security` and picks a level:
- Permissive: exactly today's behaviour; nothing added, removed or rewritten.
- Aggressive: sensitive access denied by default, built only from controls the product already has.
- Moderate: shown as "not available yet".

Essential product constraints:
- With the flag off, the product and `/security` behave and render exactly as before.
- A change shows its differences before confirmation; Esc changes nothing.
- Only a human can change or downgrade the level; no agent tool, prompt, config overlay or project file can.
- Aggressive means: agent commands write only in the current folder (not /tmp, not other folders) even when the human approves a command; every command that is not known read-only asks the human; agent commands have no network and web search is off; agent commands cannot use `corbanu vault ...`, cannot read the vault store, and secret-looking environment variables are removed; spawned child agents get the same settings.
- A saved level takes effect at the next start of Corbanu Terminal and survives restarts. An unknown or corrupt stored level fails visibly and never silently becomes Permissive.
- Returning to Permissive restores the user's own prior settings exactly.
- The UI never shows a protection as active when it is not.

## Screens (text captures of the real terminal; paths shortened to <run>)

### /security with the flag off
Security profiles — read only
Requested: Permissive (configuration only)
Effective protection: unverified. Live security status is unavailable.
Protected modes are blocked; required controls are not qualified.

> Permissive
  Moderate
  Aggressive

No additional security controls. Existing approvals, sandbox, vault, wallet, tool, network and agent policies remain unchanged.

### /security with the flag on (first screen)
Security level
Active in this session: Permissive

> Permissive  (active)
  Moderate  not available yet
  Aggressive

Today's behaviour. Your approval, sandbox, network, vault and agent settings
apply exactly as configured.
↑/↓ move · enter choose · esc close

### Enter on Aggressive (review screen)
Switch to Aggressive?
These settings replace yours for every session and child agent:
• Sandbox: write only in the current folder: no extra writable folders, no /tmp
or $TMPDIR; approved commands stay inside it too
• Approvals: untrusted: you approve every command that is not known read-only
(reviewer: you, never auto-review)
• Network: off for agent commands; web search off
• Vault: agent commands running `corbanu vault …` are refused; the vault store
is unreadable; secret-like environment variables (KEY, SECRET, TOKEN, VAULT,
PASSWORD, PASSPHRASE, CREDENTIAL) are removed
• Child agents: spawned agents get the same values
Unchanged: model and provider, MCP servers and apps, wallet scopes, and commands
you have already allowed permanently.
Takes effect when you restart Corbanu Terminal; this session keeps its current
settings until then. Your config.toml is not modified.
enter confirm and save · esc back, nothing changes

### After Esc on the review screen
Security level
Active in this session: Permissive

  Permissive  (active)
  Moderate  not available yet
> Aggressive

Sensitive access denied by default, built from existing sandbox, approval, network and exec-policy controls.
Cancelled. Nothing changed.

### After Enter on the review screen
Saved: Aggressive
Restart Corbanu Terminal to activate Aggressive. Until then this session stays Permissive. Resume this conversation with /resume
after restarting.

enter or esc close

### /status after restarting
│  Permissions:          Next turn: Profile corbanu-aggressive (workspace, untrusted)       │
│  Security:             Aggressive active (/security)                                      │

### /security after restarting with Aggressive saved
Security level
Active in this session: Aggressive
  Permissive
  Moderate  not available yet
> Aggressive  (active)
Sensitive access denied by default, built from existing sandbox, approval, network and exec-policy controls.
↑/↓ move · enter choose · esc close

### Return to Permissive review (while Aggressive is active)
Return to Permissive?
Your own approval, sandbox, network, web search and environment settings from
config apply again exactly as saved. The Aggressive vault rule file is removed.
Takes effect when you restart Corbanu Terminal; this session stays Aggressive
until then, and its pending approvals end with it.
enter confirm and save · esc back, nothing changes

### After confirming Permissive
Saved: Permissive
Restart Corbanu Terminal to activate Permissive. Until then this session stays
Aggressive. Resume this conversation with /resume after restarting.
enter or esc close

### /status after confirming Permissive, before restart
│  Permissions:          Next turn: Profile corbanu-aggressive (workspace, untrusted)       │
│  Security:             Aggressive active; Permissive saved for next start (/security)     │

### Startup with an unreadable stored level
⚠ Stored security level is unreadable (<run>/home/security_level.toml:
  unknown level `max`). Aggressive is enforced; choose a level in /security to repair it.

### /security then
Security level
Active in this session: Aggressive
Stored level is unreadable (<run>/home/security_level.toml: unknown level
`max`); Aggressive is enforced. Choose a level to repair it.
  Permissive
  Moderate  not available yet
> Aggressive  (active)
Sensitive access denied by default, built from existing sandbox, approval, network and exec-policy controls.
↑/↓ move · enter choose · esc close


## Your task
Write a prioritized list of functional test cases a human tester would run in the real terminal (real keys, the real AI agent, disposable profiles). For each case give: ID, priority (P0/P1/P2), starting state, exact actions, observable expected result. Cover success, cancel/failure, restart/recovery, downgrade, agent attempts to change the level, child agents, and flag-off. Then list ambiguities you noticed in the intent or screens. Output Markdown only.
