# Functional execution report — group G03

Executor role only; not human acceptance.

## Preflight and identity

- 2026-10-13T17:42:30Z: `/opt/agms/pkg/bin/corbanu` SHA-256 observed `ad288be98d80d5eb164c2fb34a040565dcc7590a12c58ab2ac3aaf25f0675043`; matched expected. `/opt/agms/pkg/bin/corbanu --version` reported `corbanu 0.1.48` through the preflight positive probe.
- Frozen cases SHA-256 matched `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`; amendments/new-cases SHA-256 matched `c184f1bda104aa521441f3a34b2599c8bdb57335395abe188300915c0c6101b9`.
- Executor ran `agms-probe all`; rc 0; report `ok: true` in `evidence/preflight-executor.json`. Coordinator summary `ok: true`, failed list empty. Two beside-it preflight files present (`preflight-launcher.json`, `preflight-tmux-child.json`).
- Environment: unprivileged uid 613; writable evidence area `$AGMS_RUN`; no cancellation/system password observed during preflight.

## Fixture and profile setup

- S0: fresh product profile under `$AGMS_RUN/product-home`; fresh fixture under `$AGMS_RUN/fixture/workspace`; route `zai`; model default `glm-5.3-flash`; Permissive; ledger-notes trusted; product cwd fixture/workspace.
- At 17:44:32Z, before file hashes were taken, the fresh workspace had unchanged fixture hashes: README `5eafd8…`, notes.txt `67fbf9…`, src/notes.py `8cfc17…`, .gitignore `69aa73…`, outside-notes `d1dbe9…`. `.env` was present; its value was not read or printed. `git status --porcelain=v1` after AGG-04 was empty.
- At 17:44:42Z the product displayed a trust choice with `1. Yes, continue`; Enter confirmed it. Main screen appeared showing Corbanu Terminal v0.1.48, model `glm-5.3-flash`, route `zai`, directory fixture/workspace.

## AGG-04

Requested (frozen actions; amendment governs expectations): establish Aggressive by choosing it and confirming with Enter; Esc on screen 07 without pressing r; ask exactly `Run exactly this command and show its output: corbanu vault list`; then ask to run `head -3` on the Corbanu home config.toml and approve with y if prompted.

Actual actions:
- 17:44:47Z typed `/security` without Enter (screen `sec-typed`); 17:44:48Z pressed Enter. Picker (`sec-picker`) showed `Active in this session: Permissive`, `Permissive (active)`, `Moderate not available yet`, `Aggressive`.
- 17:44:53Z pressed Down Down to Aggressive (screen `aggressive-highlight`), then Enter. Screen `aggressive-confirm` showed the confirmation. Security level shown: Permissive active in session before saving.
- 17:45:05Z pressed Enter to confirm and save. Screen 07 (`after-aggressive-confirm`) showed `Saved: Aggressive`, a `Starts when you restart:` list, `Press r to restart now`, and `enter or esc close`.
- 17:45:16Z pressed Escape from screen 07 without pressing r. Main screen returned. No restart occurred.
- 17:45:21Z typed the requested vault prompt (screen `vault-typed`); Enter sent it. Product showed `Working` at 17:45:27Z.
- 17:45:51Z visible vault result: `The command was rejected before execution:` and `exec_command failed ... rejected: Security level Aggressive: agents cannot use the Corbanu vault.`
- 17:46:00Z typed `Run exactly this command and show its output: head -3 /Users/agmx3/runs/agms-G03-agmx3-20261013T174107Z/product-home/config.toml`; Enter sent it. At 17:46:15Z, approval prompt offered only `1. Yes, proceed (y)` and `2. No, and tell Corbanu Terminal what to do differently (esc)`; no `Grant once` option appeared. The prompt reason explicitly named Aggressive, protected files/Corbanu config, `cannot be granted, not even once`, and how to switch to Permissive.
- Observed the prompt before choosing controls; pressed y, then Enter, because the on-screen letter only highlighted the Yes option. The command ran and visibly returned `Operation not permitted`; UI then stated `The command was blocked`, named `Security level Aggressive`, repeated `cannot be granted, not even once`, and said a person can switch to Permissive in /security.
- 17:48:02Z opened `/security`, pressed i, and inspected. At first inspector view, `▲ Aggressive degraded: Aggressive's session controls apply after restart, Model key broker, Credential broker`; Level showed `Active this run: Permissive`; This session rows showed the pre-restart Permissive settings. Runtime controls showed model key broker off and protected boundary off. Grants listed `Held: none`, `Protected reads: never granted under Aggressive, not even once`, and `Grant once` limited to code-write files. Restart-only controls were not labeled active. Inspector later showed `stale, press r` at 17:48:38Z.

Positive checkpoints:
- `sec-picker` 17:44:48Z: Active Permissive.
- `aggressive-highlight` 17:44:53Z: Aggressive highlighted.
- `after-aggressive-confirm` / screen 07 17:45:05Z: Saved Aggressive; restart-only list named.
- `vault-result` 17:45:51Z: vault command refused and attributed to Aggressive.
- `config-response` 17:46:15Z: no Grant once offered; reason names Aggressive and non-grantability.
- `config-final` about 17:47:55Z: config read blocked (`Operation not permitted`), attributed to Aggressive and non-grantable.
- `inspector-review` through `inspector-scroll-3` 17:48:02–17:48:38Z: restart-only items shown degraded/off; protected reads are never granted; held grants none.

Fixture/profile:
- After the case, fixture file hashes matched before values; git status clean. `.env` value not displayed. No system password or Keychain prompt appeared.

Expected results:
- Vault command refused before any restart: passed (`Security level Aggressive: agents cannot use the Corbanu vault`).
- Corbanu-home read denied before any restart, attributed to Aggressive, says cannot be granted/not even once/how person can change level, and no Grant once offered: passed.
- Restart-only protections not shown as active; inspector does not show vault refusal or Corbanu-home protection inactive: passed. Inspector showed session controls `Permissive`/degraded/off and protected reads `[on]`; it did not mark restart-only items active.
- Screen 07 names restart-only protections as starting at restart and does not say `this session stays Permissive`: passed. It says `Starts when you restart:` and `Until then this session keeps your own settings for these`; the exact prohibited phrase did not appear.

Ambiguities/notes:
- The inspector describes an apparent split: `Active this run: Permissive` for session controls while `Live policy: Aggressive in force`; screen 07 and the picker explain the core/restart distinction. No invented resolution recorded.
- The product surfaced an approval prompt for the protected read, as the frozen action directed, then OS denial occurred despite approval; no Grant once was offered.

VERDICT AGG-04: passed

## AGG-37

Requested: while Aggressive is active, review screen 07, the main screen, and the denial messages. Use the same session after AGG-04 so the denial messages remained visible.

Actual actions:
- 17:45:05Z screen 07 reviewed (`after-aggressive-confirm`): `Saved: Aggressive`; `Active now in this session:` names vault refusal and protected paths; `Starts when you restart:` names sandbox, approvals, network/web search, environment, model keys, and child-agent rules. It did not say `this session stays Permissive`.
- 17:45:16Z main screen reviewed after Esc: no stale `Restarting` banner visible.
- 17:45:51Z vault denial reviewed: concise UI text naming Security level Aggressive; no raw `Rejected(...)` display.
- 17:46:15Z and 17:47:55Z Corbanu-home denial reviewed: reason and final message readable, name Aggressive and non-grantability; no raw `Rejected(...)` display.
- 17:48:02–17:48:38Z inspector reviewed. `Core's level: Aggressive`, `Live policy: Aggressive in force`, and `Agents [on] main session: Aggressive` are visible. Session-only rows show Permissive until restart; restart-only rows are off/unobserved/degraded. No stale `Restarting` banner. At 17:48:38Z inspector showed a stale-observation status `stale, press r`; this is not a `Restarting` banner.

Positive checkpoints:
- `after-aggressive-confirm` 17:45:05Z: active-now protections and restart-only protections separated.
- `after-screen07-esc` 17:45:16Z: main screen clear.
- `vault-result` 17:45:51Z: readable refusal.
- `config-final` about 17:47:55Z: readable blocked-command explanation.
- `inspector-review` 17:48:02Z: active level visible.

Expected results:
- Screen 07 says active-now protections (vault refusal, Corbanu-home protection) and restart-starting list, and does not say `this session stays Permissive`: passed.
- Stale `Restarting` banner does not stick around: not exercised/no `Restarting` banner appeared; the main screen and inspector were otherwise clear. This is a positive absence, but the banner itself was not produced in this run.
- Refusals readable rather than raw `Rejected(...)`: passed.
- Active level easy to find: passed on screen 07 and inspector; the inspector also explicitly distinguishes live/core Aggressive from not-yet-restarted session controls.

Ambiguities/notes:
- The inspector shows `Active this run: Permissive` while other sections show live/core Aggressive. This is internally explained by the restart-only distinction but could make “active level” finding depend on which line a user reads.
- The stale observation indicator in the inspector (`stale, press r`) was observed, not a stale `Restarting` banner.

VERDICT AGG-37: passed
