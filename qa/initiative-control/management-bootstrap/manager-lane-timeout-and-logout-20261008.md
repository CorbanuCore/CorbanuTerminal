# Manager lane: pre-cycle timeouts and the logout dependency (2026-10-08)

Travis, 2026-10-08: manager stays on standby (no allocations, no mode change); fix the logout dependency;
fold in the pre-cycle timeout fix.

## Pre-cycle timeout is an error, not a hold
On Oct 6 06:48Z a host-wide slowdown made one 2-5 s check time out before any cycle started, and the
manager lane latched `TimeoutExpired` until a manual `--recover` a day later (the owner lane counted the same
timeout as an error and kept ticking). Now, in `owner_daemon.scheduled_tick`:
- a `subprocess.TimeoutExpired` with the manager cycle counter unchanged (read before the pins check, which
  runs the pinned interpreter with a 5 s timeout, and again afterwards) is counted as an error and logged as
  `transient_error` in `manager-cycles.jsonl`;
- the third consecutive one latches `manager_pre_cycle_timeouts` (`refusal: TimeoutExpired x3`); BUSY does not
  reset the streak, a normal pass does, `--recover` does;
- a timeout after a cycle started, an unreadable counter and every other error still latch at once;
- `main()` reports a `--recover` that times out instead of crashing.

Reviews: Opus 5.5 High, round 1 CHANGES REQUIRED (the fix missed the only reachable pre-cycle timeout, the pins
check; tests injected where production never times out), round 2 APPROVE with three P3s, all applied.

Not live: the pinned owner package changes, so it needs the PF-83 requalification before a repin. That waits
for the logout decision below so one requalification covers both.

## Logout dependency: blocked on a credential decision
The manager lane is a `gui/501` job because a manager cycle reads the Claude login from the Corbanu vault at
`auth_vault_home`, and that vault's key is in the login keychain (`local.age`, no `keyring-fallback`). From
`user/501` (session `Background`, its own audit session) the read fails with "User interaction is not allowed"
(round-7 probe), and after logout the login keychain is locked in any domain, including a root LaunchDaemon.
So moving the job to `user/501` alone would only turn every cycle into a `vault_auth_unavailable` hold. The
job can run without the GUI session only with a manager credential that does not depend on the login keychain,
which reverses Travis's Oct 3 decision B1 ("from the vault at use time, never from a file") in part.

Also found: both jobs are bootstrapped from `~/Library/Application Support/corbanu-owner-live/*/owner.plist`,
not `~/Library/LaunchAgents`, so neither is reloaded after a reboot. Whether `user/501` itself outlives a full
logout was never tested.
