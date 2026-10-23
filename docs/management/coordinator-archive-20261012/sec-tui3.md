You are the TUI lane worker (round 3). Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md first
(binding), then /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-tui2.result.md and sec-picker.result.md in the same
folder. PF-24-S02 is still blocked on PF-23-S03 and PF-29-S02, so this round does bounded follow-ups, one small PR each,
under the per-fix gate.

1. #197: make exec-policy rule parse errors fatal (for new threads too) when Aggressive is active, so a `.rules` file
   broken after launch can't silently drop the `corbanu vault` rule.
2. Credential leftovers from #183/#201:
   a. Response-header logging (`set-cookie` and similar) must redact values.
   b. User-visible error messages that include the configured provider URL must redact query values / userinfo.
   c. Old leaked rows in users' existing log databases: add a one-time, idempotent scrub on startup (or a documented
      `corbanu` maintenance command if a startup scrub is unsafe) that redacts secret-shaped values from existing rows
      in the logs DB and old `codex-tui.log` files. Keep it bounded and fast; test it on a fixture DB.
3. Prefix rules not matching `./corbanu` or `env corbanu`: make the Aggressive `corbanu vault` rule also catch those
   forms (resolved program path / `env` wrapper), with tests.
Do not change the Aggressive sandbox design (waiting on Travis). Gate per fix: focused `just test`, GLM 5.2 tmux run
where user-visible, one independent Opus 5.5 High review, short demo video where user-visible. PR to main; merge when
green (stale CodeQL checks on PRs can be ignored). Report as in sec-common.md.
