All three items are done and merged to main, behind the `security_levels` flag:

- **#258:** PF-24-S02's GLM 5.2 pass and its archive.
- **#260:** PF-25-S01, temporary grants. I merged this one at its head with every check green.
- **#261:** PF-25-S02, revocation and kill switch. You merged this one, and #258.

All three sprint records are archived with status `completed`. The full report is in `.codex-work/workers-20261002/sec-tui8.result.md`.

**PF-24-S02 GLM pass (#258, docs only)**
- Four GLM 5.2 videos on main `e4d17dbdc6`, each ending with GLM reading a file in the hidden Corbanu home:
  - [Confirm](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s02-pf24s02-glm-confirm-e4d17dbdc6f0-2026-10-07.mp4): after confirming Aggressive, the same read gets "Operation not permitted" with no restart.
  - [Cancel](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s02-pf24s02-glm-cancel-e4d17dbdc6f0-2026-10-07.mp4): Esc changes nothing and the read still works.
  - [Downgrade](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s02-pf24s02-glm-downgrade-e4d17dbdc6f0-2026-10-07.mp4): the review lists the protections removed; Aggressive stays until `r`, then the read works.
  - [Restart now](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s02-pf24s02-glm-restart-now-e4d17dbdc6f0-2026-10-07.mp4): same options, the initial prompt isn't sent again, and the approved read is denied.
- One take was redone because GLM rewrote the command; the gate record notes it.

**PF-25-S01, temporary grants (#260)**
- **What it does:** under Aggressive, a command approval has a "Grant once" option (`g`). It opens a review of the exact command, folder, agent, session, limit and expiry, and what stays denied. The review starts on "Back"; ↓ then Enter grants.
  - "1 run" lifts the protected-path rules for that one run only.
  - `u` switches to any run of the same command for 10 minutes, listed in `/security`.
- **Safety:** the grant applies only if that same approval is then approved. Esc, a declined approval or any security-state change grants nothing.
- **Tests:**
  - Focused: Core 62/62, TUI 164/164. A Core test also runs the whole grant flow end to end.
  - The wider TUI run, 4414 of 4416, was on the round-1 code only, not the final code. The two failures are the existing command-menu snapshot and kitty pet image tests.
- **Linux clippy** on the RTX box: clean, including after your merge of PF-41-S01 into the branch.
- **Review:** Opus 5.5 High, four rounds, final verdict APPROVE.
- **GLM tmux run:** it caught a real bug, fixed in this PR: Esc in the review declined the whole command instead of going back to the approval.
- **Videos** (`aee150e46a`):
  - [grant one run](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-25-s01-pf25s01-grant-once-aee150e46ae7-2026-10-07.mp4)
  - [Esc grants nothing](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-25-s01-pf25s01-grant-esc-aee150e46ae7-2026-10-07.mp4)
  - [grant until expiry, listed in /security](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-25-s01-pf25s01-grant-until-expiry-aee150e46ae7-2026-10-07.mp4)

**PF-25-S02, revocation and kill switch (#261)**
- **What it does:** `/security` then `g` opens "Grants and kill switch". It lists the grants held now, "Revoke all active authority", and the kill switch, and each choice has its own review.
  - Revoke all and the kill switch apply at once to every session of the process on the same Corbanu home, and are saved, so they hold after a restart.
  - Turning the kill switch off starts on "Back", keeps the security level, and also turns it off in other sessions that have the same switch on.
- **Tests:**
  - Focused: Core 66/66, TUI 190/190.
  - Full TUI run: 4411 of 4413, with the same two existing failures.
  - Core security set: 278/278.
- **Linux clippy** on the RTX box: clean.
- **Review:** Opus 5.5 High, three rounds, final verdict APPROVE.
- **CI:** the macOS credential canary failed once on its network-proxy check, a crate this PR doesn't touch. It passed locally (54/54) and on re-run.
- **Videos** (`be5d33660c`):
  - [revoke one grant](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-25-s02-pf25s02-revoke-grant-be5d33660c7e-2026-10-07.mp4)
  - [kill switch on, still on after restart](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-25-s02-pf25s02-kill-switch-restart-be5d33660c7e-2026-10-07.mp4)
  - [kill switch off keeps the level](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-25-s02-pf25s02-kill-switch-off-be5d33660c7e-2026-10-07.mp4)
- **Core fix:** a kill-switch change made in the same second as the previous one was silently ignored. Core now dates it a second later.

**Keychain and cleanup:** every run used a throwaway home with `CORBANU_TEST_NO_NATIVE_KEYRING=1`, and nothing touched the real login keychain. My worktrees, build folders, RTX checkouts and temporary files are removed.

**Decisions for you**
1. **Turning off a saved kill switch with the flag off:** there's no `/security` picker then, so it can't be turned off from the TUI. The options are to show the grants and kill-switch view whenever a kill switch is saved, even with the flag off, or to leave it until the flag is removed. I recommend showing it.
2. **Kill switch under Permissive:** with no grants and the switch off, `g` isn't offered, so there's no emergency stop there. The options are to always offer it when the picker is on, or to keep it hidden since Permissive has little for it to stop. I'd keep it hidden.

**Known limits** (in the gate records):
- Grants live only in memory, so a restart ends them. The sprint text asked for a saved grant record; that's deliberately not done, matching Core's existing design.
- Only shell commands get the grant option. Patches don't, and typing into a process started without the protections is still refused under Aggressive.
- `/security` lists grants from every session in the process, including ended ones, until they expire.
- Nothing in the code stops other parts of the app from calling Core's grant-confirm function. A test fails if anything other than the grant review calls it.
- The end-to-end grant test runs on macOS only.