# PF-24-S02: per-sprint gate (2026-10-07)

- **Branch:** `feat/pf-24-s02-security-confirm` off main at `4f09d7af99` (after PF-23-S03, #246–#249), with main
  merged in again before review round 3. Behind `security_levels`. Core's level is raised only after the
  `protected_mode_preflight` preflight passed, so with that flag off a save behaves as in PF-24-S03 and never
  touches `security_state.json`.
- **Model:** GLM 5.2 has no Z.AI balance, so every TUI run and video uses a mock provider (`mock-model`, no
  network, no model turn). A GLM run can be added later; nothing in this sprint needs a model turn.

## What a confirmation does

| Choice | Order | Core | Failure |
| --- | --- | --- | --- |
| Aggressive (preflight passed and rechecked) | receipt, `security_level.toml`, Core | stricter: applies now to this session's tree and the others of the process on the home | Core refusal or "changed": files restored, "Nothing changed" |
| Aggressive, preflight flag off | receipt-less save | not raised (said on the review) | as PF-24-S03 |
| Permissive | `security_level.toml`, Core | downgrade: saved now, applies at the next start; this session's grants and "for session" approvals end now | files restored, view stays open, Enter reviews again |
| Unchanged | as above | saves the level; raises other sessions below it (their sinks are told) | — |

Every step runs on its own thread, under `security_confirm.lock`, and only while Core's state (level, record, tree
epoch) is still what the review showed. Otherwise the result is "review it again".

At the next start, `security_level.toml` is checked against Core's `security_state.json`. If the level file is weaker
than an Aggressive record, or the record is corrupt, it reads as invalid: Aggressive is enforced, nested launches are
refused, and `/security` says the file "was changed outside /security". An Aggressive save whose Core step didn't
finish is reported at start. A Permissive launch leaves the rule file and registry entry alone while another
Aggressive process on the same home holds `security_level.lock`.

"Restart now" (`r` after a save, or on the list when a saved level is waiting) shuts down like `/quit`. It then starts
the same program again with the same arguments, folder and environment, plus `CORBANU_RESTARTED_FOR_SECURITY=1`, so
the initial prompt and images aren't sent a second time.

Keys: Enter confirms, Esc cancels (nothing written), `r` restarts, and Enter on a failure reviews again. Saving shows
"Saving…", and Ctrl-C can't close the view until the save finishes.

## Results

- **Focused tests:**
  - `just test -p codex-core -p codex-security-level` (`security_transition`, `security_recovery`,
    `security_confirm`, `pf_29`): see the final run below.
  - `just test -p codex-tui` (`security`, `pf_24`, `pf_29`, `slash_command`): 273/273 at round 1.
- **Wider run:** `just test -p codex-tui -p codex-security-level -p codex-cli` at round 1. Three tests failed, none in
  code this sprint touches:
  - the command-menu snapshot (known drift);
  - a kitty pet image test;
  - `nested_launch::pass_mode_runs_exec_with_aggressive_enforced`, where the exec path finds the test's
    account-home registry missing. It wasn't compared on main.
- **Linux clippy** (`-D warnings`; core, tui, cli, security-level, app-server-client) on the RTX box: clean at
  `9b776a204b` and `ff0e076d90`; see below for the final tree.
- **tmux runs (mock model):** in `.codex-work/workers-20261002/sec-tui7.log`:
  - confirm Aggressive: Core is Aggressive now in the session, and `r` restarts with `-c` options kept and the
    initial prompt not sent again. `/status` then shows `corbanu-aggressive` and "protected boundary checked at
    launch".
  - downgrade review, then Esc, then confirm;
  - a second Permissive launch while an Aggressive one runs: the rule file stays;
  - a blocked lock on an upgrade: Core applies now and reports it wasn't saved;
  - the level file rewritten by `!printf`.
  - Startup warnings weren't shown in these mock runs; an unrelated theme warning wasn't shown either, so this is a
    pre-existing display path. The warnings are covered by tests and `/security`.
- **Review (Opus 5.5 High):** round 1 and round 2 asked for changes; every finding is fixed or recorded in
  `review/disposition.md`.
- **Videos:** see `qa/demos/index/PF-24-S02.md`.

## Known limits

- Same-user edits of both `security_level.toml` and `security_state.json`, or deleting both, aren't detected
  (PF-20 anchor).
- Another process sees a commit at its next session start (as in PF-23-S03).
- A downgrade's revocation ends grants in the confirming session only.
- On Windows the parent doesn't ignore Ctrl-C while it waits for the restarted child.
- Agent commands inherit the restart marker.
- A profile-v2 user config path isn't rewritten by a downgrade. This errs strict.
- Startup warnings depend on the existing display path (see above).
