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

- **Focused tests (final tree):**
  - `just test -p codex-core -p codex-security-level` (`security_transition`, `security_recovery`,
    `security_confirm`, `pf_29`, `pf_23`): 111/111.
  - `just test -p codex-tui` (`security`, `pf_24`, `pf_29`, `slash_command`): 273/273.
  - New: `core/src/security/level_change_tests.rs` (11), `security-level/src/level_tests.rs` (5),
    `tui/src/security/confirm_tests.rs` (6), picker `pf_24_s02` (7), and the launch and restart tests.
- **Wider run** (`just test -p codex-tui -p codex-security-level -p codex-cli`, round-1 tree): 5920/5923. The three
  failures are in code this sprint doesn't touch:
  - the command-menu snapshot (known drift);
  - a kitty pet image test;
  - `nested_launch::pass_mode_runs_exec_with_aggressive_enforced`, where the exec path reports the test's
    account-home registry missing. It wasn't compared on main.
- **Linux clippy** (`-D warnings`; core, tui, cli, security-level, app-server-client) on the RTX box: clean at
  `9b776a204b`, `ff0e076d90`, `3e1ba16e13` and `9fda6ceb5e`; later commits only change messages, comments and argument comments.
- **tmux runs (mock model; `.codex-work/workers-20261002/sec-tui7.log`):**
  - Confirm Aggressive: Core is Aggressive now in this session. `r` restarts with the `-c` options kept and doesn't send
    the initial prompt again. `/status` then shows `corbanu-aggressive` and "protected boundary checked at launch".
  - Downgrade: review, Esc, then confirm.
  - A second Permissive launch while an Aggressive one runs: the rule file stays.
  - A blocked lock on an upgrade: Core applies now and reports that the save failed.
  - The level file rewritten by `!printf`: caught.
  - Startup warnings weren't shown in these mock runs. An unrelated theme warning wasn't shown either, so this is the
    existing display path; the warnings are covered by tests and `/security`.
- **Review (Opus 5.5 High, installed `corbanu exec`, read-only):**
  - Round 1: REQUEST CHANGES (10 findings).
  - Round 2: REQUEST CHANGES (5).
  - Round 3: APPROVE WITH FIXES (4).
  - Round 4: **APPROVE**.
  - Each finding is fixed or recorded in `review/disposition.md`; the rounds are in `review/round1..4.md`.
- **Videos (mock model, commit `9fda6ceb5e02`):** four, listed in `qa/demos/index/PF-24-S02.md`:
  - confirm and restart;
  - downgrade shows removed protections;
  - a failed save stays open;
  - level-file tamper.

## Known limits

- Same-user edits of both `security_level.toml` and `security_state.json`, or deleting both, aren't detected
  (PF-20 anchor).
- Another process sees a commit at its next session start (as in PF-23-S03).
- A downgrade's revocation ends grants in the confirming session only.
- On Windows the parent doesn't ignore Ctrl-C while it waits for the restarted child.
- A profile-v2 user config path isn't rewritten by a downgrade. This errs strict.
- Startup warnings depend on the existing display path (see above).
