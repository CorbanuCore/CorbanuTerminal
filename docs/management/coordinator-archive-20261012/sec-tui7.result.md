PF-24-S02 is merged in PR #253 as `e4d17dbdc6`. Travis merged it after two docs-only merges of main into the branch, and every check passed on the final head. PF-25-S01 (temporary grant TUI) was not started.

**What shipped** (behind `security_levels`)
- **Confirm:** pressing Enter on the `/security` review is the only way to change the level. It saves `security_level.toml` and commits PF-23-S03's Core transition through the current session.
  - The save runs off the UI thread and only goes through if nothing changed since the review; otherwise the person is asked to review again.
  - No model, tool or app-server request can reach it.
- **Aggressive:** Core's protections apply to running sessions straight away. The sandbox, approvals, network and vault settings still take effect at restart. Core is only raised after the PF-29 preflight passes; that preflight now also requires `source_envelopes`, because Core refuses unlabelled external content at protected levels.
- **Permissive:** the review lists every protection that will be removed before you confirm. The change applies at the next start, and this session's grants end immediately. It warns when a project, profile, `-c` or managed config keeps a stricter level.
- **Esc** changes nothing. **A failed save** puts the previous files back and keeps the view open so you can try again.
- **Picker follow-ups:**
  - **Tamper check:** a level file weaker than Core's confirmed Aggressive record reads as invalid, so Aggressive stays enforced and `/security` says why.
  - **"Restart now":** pressing `r` restarts with the same arguments without resending the initial prompt or images.
  - **Second Permissive launch:** it no longer removes the vault rule file while an Aggressive session on the same home is still running.

**Gate evidence** (`qa/security-levels/sprints/PF-24-S02/`)
- **Tests:**
  - Core and security-level focused sets: 111/111. TUI focused sets: 273/273.
  - The wider tui + cli + security-level run was 5920/5923, on the first-round code rather than the final code. The three failures are in code this change doesn't touch: a command-menu snapshot, a kitty pet image test, and a nested-launch exec test that reports a missing test registry. I didn't run that last one on main to compare.
- **Linux clippy** (`-D warnings`) on the RTX box: clean up to `9fda6ceb5e`. Later commits only change messages, comments and argument comments, and Windows CI's argument-comment lint passed on them.
- **tmux runs:** GLM 5.2 is out of Z.AI credit, so all runs used a mock model with no model turn. That's recorded in the gate record; a GLM run can be added later. They covered:
  - confirm, then restart: options kept, prompt not resent;
  - downgrade review, cancel, then confirm;
  - a second Permissive launch while an Aggressive one runs;
  - a blocked save;
  - a level file rewritten by a shell command.
- **Review:** Opus 5.5 High, four rounds. Rounds 1 and 2 asked for changes, round 3 approved with fixes, round 4 approved. All findings are fixed or listed in `review/disposition.md`.
- **Videos** (mock model, commit `9fda6ceb5e02`):
  - [confirm Aggressive, then restart](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s02-pf24s02-confirm-aggressive-restart-9fda6ceb5e02-2026-10-07.mp4)
  - [downgrade shows removed protections](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s02-pf24s02-downgrade-shows-removed-9fda6ceb5e02-2026-10-07.mp4)
  - [failed save stays open](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s02-pf24s02-save-failure-stays-open-9fda6ceb5e02-2026-10-07.mp4)
  - [level-file tamper caught](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s02-pf24s02-level-file-tamper-9fda6ceb5e02-2026-10-07.mp4)
- **Keychain:** every run used a throwaway home with `CORBANU_TEST_NO_NATIVE_KEYRING=1`; nothing touched the real login keychain.

**Cleanup:** my worktree, its build target, the RTX box checkout and target, and all temporary run folders are removed.

**Still open**
- A GLM 5.2 run once there's Z.AI credit.
- The sprint record is merged but not archived: it's in `draft` so it passes the sprint-format check. Archiving needs a follow-up docs change by whoever owns the plan.
- **Known limits:**
  - Editing or deleting both level files together isn't detected (that needs the PF-20 external anchor).
  - On Windows, the old process doesn't ignore Ctrl-C while it waits for the restarted one.
  - A downgrade doesn't rewrite a profile-v2 config path; this errs on the strict side.
  - Startup warnings didn't appear in the mock runs. An unrelated theme warning didn't appear either, so the cause is in the existing display path; `/security` shows the tamper and unreadable-level cases.