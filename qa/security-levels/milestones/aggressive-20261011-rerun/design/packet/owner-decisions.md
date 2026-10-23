# Owner decisions (Travis, 2026-10-11), verbatim from the issues' decision comments

Extracted by the integrator; only comments that state an owner decision. Implementation and evidence comments are left out.


## #434 (2026-10-11T19:55:30Z): Aggressive (AGG-16): vault-managed secret shown verbatim in command output despite the output gate
https://github.com/CorbanuCore/CorbanuTerminal/issues/434#issuecomment (2026-10-11T19:55:30Z)

Owner decision (Travis, 2026-10-11): keep Core's vault read at a session's first task and whenever the vault changes (the "cover both cases" behaviour shipped in #454). No change.


## #436 (2026-10-11T11:55:30Z): Aggressive (AGG-26): returning to Permissive with "restart now" loses the environment provider key (onboarding shown)
https://github.com/CorbanuCore/CorbanuTerminal/issues/436 (only the decision paragraph of the closing comment)

Open product decision (Travis): restart now could bring the key back only if the broker handed the value back to Core; the broker protocol is one-way by design, so this keeps it one-way and asks the person to start again from the shell.


## #440 (2026-10-11T19:55:31Z): Aggressive (AGG-15): ordinary environment variables are removed too; review says only secret-like ones
https://github.com/CorbanuCore/CorbanuTerminal/issues/440#issuecomment (2026-10-11T19:55:31Z)

Owner decision (Travis, 2026-10-11): keep the agent environment allowlist (PF-27-S02 design; wording fixed in #458). No change.


## #442 (2026-10-11T08:17:28Z): Aggressive (AGG-09/17/18/23/33): "Grant once" appears unreachable for protected reads such as cat .env
https://github.com/CorbanuCore/CorbanuTerminal/issues/442#issuecomment (2026-10-11T08:17:28Z)

Owner decision (Travis, 2026-10-11): **No** — under Aggressive, reads of protected credential files (e.g. `.env`) are never grantable, not even once. The product must explain this plainly at the point of denial (and in the inspector/review), naming the security level and how a user can change it; 'Grant once' remains for other gated operations (e.g. protected writes), and the milestone's grant/revoke/expiry cases should be exercised on those.


## #442 (2026-10-11T16:28:20Z): Aggressive (AGG-09/17/18/23/33): "Grant once" appears unreachable for protected reads such as cat .env
https://github.com/CorbanuCore/CorbanuTerminal/issues/442#issuecomment (2026-10-11T16:28:20Z)

Owner decision (Travis, 2026-10-11): protected reads stay non-grantable under Aggressive (no "grant once"), and the denial must explain why where it failed. This matches what #464 shipped; no further change needed.


## #442 (2026-10-11T19:55:33Z): Aggressive (AGG-09/17/18/23/33): "Grant once" appears unreachable for protected reads such as cat .env
https://github.com/CorbanuCore/CorbanuTerminal/issues/442#issuecomment (2026-10-11T19:55:33Z)

Owner decisions (Travis, 2026-10-11) on the two open points from #464:
1. `.env` files (and the other protected credential file names the launch preflight uses) **created after** an Aggressive session starts must be protected too, in the sandbox on every platform (Seatbelt by name pattern, bwrap with a TOCTOU-safe mechanism or fail closed, Windows deny-ACE inheritance checked). The launch check, the denial explanation and the ".env files found at start" wording must follow.
2. The project's `.codex` and `.agents` are **always read-only** under Aggressive: "Grant once" no longer lifts write protection on them (it still lifts the other write protections). Grant review, inspector and /security text to be updated.
Implementation in progress; PR to follow.


## #443 (2026-10-11T07:40:37Z): Aggressive (AGG-04): product decision needed on what applies between confirming Aggressive and restarting
https://github.com/CorbanuCore/CorbanuTerminal/issues/443#issuecomment (2026-10-11T07:40:37Z)

Owner decision (Travis, 2026-10-11): protections apply **immediately** on confirming Aggressive (no wait for restart) — in particular vault refusal and denial of the Corbanu home must hold from the moment of confirmation; screen 07 must not claim 'this session stays Permissive'.


## #443 (2026-10-11T20:06:29Z): Aggressive (AGG-04): product decision needed on what applies between confirming Aggressive and restarting
https://github.com/CorbanuCore/CorbanuTerminal/issues/443#issuecomment (2026-10-11T20:06:29Z)

Owner decisions (Travis, 2026-10-11) on the follow-ups named here:
1. Restart-only protections stay applied at restart for this milestone; what applying them at confirmation would take is recorded in #505 (no code change).
2. Listing the Corbanu home is **denied** under Aggressive (not only reading protected files), from the moment of confirmation and after a restart, explained at the point of denial in the #464 style. Core itself must keep working. Tracked in a new issue (linked below).


## #445 (2026-10-11T07:40:39Z): Aggressive (AGG-22): with the kill switch on, model turns are still answered although the screen says the credential broker stays closed
https://github.com/CorbanuCore/CorbanuTerminal/issues/445#issuecomment (2026-10-11T07:40:39Z)

Owner decision (Travis, 2026-10-11): **yes** — with the kill switch on, model turns are refused; the message names the kill switch and how to turn it off.


## #445 (2026-10-11T20:06:54Z): Aggressive (AGG-22): with the kill switch on, model turns are still answered although the screen says the credential broker stays closed
https://github.com/CorbanuCore/CorbanuTerminal/issues/445#issuecomment (2026-10-11T20:06:54Z)

Owner decisions (Travis, 2026-10-11) on the questions left here: the kill switch also stops Claude panes; turning it on interrupts the current turn in every pane; agent messages keep queuing until the next turn after it is turned off (no change). Tracked in #507.


## #505 (2026-10-11T20:06:28Z): Aggressive: apply restart-only protections at confirmation
https://github.com/CorbanuCore/CorbanuTerminal/issues/505#issuecomment (2026-10-11T20:06:28Z)

Owner decision (Travis, 2026-10-11): restart-only protections (sandbox, approval for every command, network off, environment stripping, model-key broker, child-agent rules) **stay applied at restart** for the "Aggressive ships" milestone. No code change; this issue records what applying each one at confirmation would take.


## #506 (2026-10-11T20:06:52Z): Aggressive: deny listing the Corbanu home to agent commands
https://github.com/CorbanuCore/CorbanuTerminal/issues/506#issuecomment (2026-10-11T20:06:52Z)

Owner decision (Travis, 2026-10-11): listing the Corbanu home is DENIED under Aggressive (not only reading protected files), from the moment of confirmation and after restart, with the same explanation style as #464 at the point of denial. The product itself (Core, not agent commands) must keep working.


## #507 (2026-10-11T20:06:53Z): Kill switch: stop Claude panes and interrupt the current turn
https://github.com/CorbanuCore/CorbanuTerminal/issues/507#issuecomment (2026-10-11T20:06:53Z)

Owner decisions (Travis, 2026-10-11):
- (3) The kill switch STOPS Claude panes too; turning it on stops/refuses Claude panes the same way as Codex panes; the kill-switch text is updated.
- (4) Turning the kill switch on INTERRUPTS the current turn: a reply already streaming is cancelled and its pending/running commands do not run (or are stopped), in every pane, with a clear message.
- (5) Agent messages arriving while the switch is on keep queuing until the next turn after it is turned off (current behaviour, no change); the kill-switch text must say so.


## Coordinator brief from Travis (2026-10-11), restating the decisions for this re-run

"protected reads are never grantable under Aggressive, not even once, and the denial is explained at the point of
failure (#442/#464); protections apply immediately on confirmation except the restart-only list named on screen 07
(#443/#453, #505); the kill switch refuses model turns, interrupts the current turn in every pane incl. Claude panes,
and queued agent messages wait (#445/#456/#507); listing the Corbanu home is denied under Aggressive (#506); `.env`
files created after start are protected and `.codex`/`.agents` are always read-only (#520); the credential broker
stays one-way, so "restart now" is not offered when a key was handed to the broker (#436/#449)."
