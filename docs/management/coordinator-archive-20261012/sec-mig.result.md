Both sprints are merged to main behind the new default-off flag `protected_mode_preflight` (it only has an effect with `security_levels` on), and both records are back to `draft` for their open items. With the flag off nothing changes, and a control video confirms that.

## What shipped
| PR | Sprint | Merge |
| --- | --- | --- |
| #228 | PF-29-S01 inventory and activation preflight | `b751254169` |
| #235 | PF-29-S02 credential migration and recovery | `65d42158d7` |
| #236 | docs: S02 record back to draft, `merged_behind_flag` set | `e72563e5f7` |

PF-28-S02 was in CI when I started, so I built on main and recorded its `merged_behind_flag` in its record, following the PF-28-S01 and PF-33-S02 precedent. PF-20-S02 was already archived.

- **S01:** choosing Aggressive in `/security` now runs a preflight. It lists where secrets live by location only and checks the required protections. Blockers keep Aggressive unsaved, and anything that changed since the review stops the save.
  - This covers PF-27-S02's known limit. While Aggressive is saved after a preflight, agents can't read `~/.ssh` (keys and the whole folder), cloud and CLI token files, browser profiles, or Corbanu's history and databases. Each denial is checked at launch.
  - Every launch re-checks: if the boundary is no longer clean, Corbanu says so in a startup warning and in `/status` instead of claiming it.
  - Conversations from before activation can't be resumed or forked.
- **S02:** in a blocked review, `m` previews moving shell-profile secrets into the vault, and Esc changes nothing. Enter stores each value in the vault and replaces the line with a `corbanu vault auth-helper` reference, leaving the file readable only by you, with no backup copy.
  - A crash at any stage keeps the current level and blocks Aggressive until `r` finishes the job. Finishing only moves forward, never puts plain text back, and never touches the saved level.
  - Afterwards it lists which credentials to rotate at their provider.

## Gate evidence
- **Focused tests:** S01 has 14 core and 8 TUI tests; S02 adds 7 core and 4 TUI tests. All pass.
- **Broader suites:**
  - Core security tests pass (161 on S01, 176 on S02).
  - The full TUI suite has the same 21 failures on both branches. 20 are Unix socket paths too long for this worktree's location and 1 is an old branding snapshot; none touch this code.
  - The app-server-client suite has one failure for the same socket-path reason.
  - Clippy is clean on the changed code.
- **Linux check:** I did not run Linux clippy in Docker or on the RTX box. The change has no Linux-only code, only Unix file handling.
- **GLM 5.2 tmux runs:** the five S01 videos show the block, the `~/.ssh` denial after a real restart (with the flag-off control showing it readable), the refused resume, and the launch re-check. The three S02 videos show cancel, migrate and save, and an injected crash followed by restart and recovery. Links are in `qa/demos/index/PF-29-S01.md` and `PF-29-S02.md`.
- **Opus 5.5 High reviews:**
  - S01: changes required, then approve with nits, then a third pass with no P0/P1.
  - S02: changes required, then approved.
  - Every finding was fixed or answered in the evidence READMEs.

The tmux runs also caught a real startup race: database side files appeared after launch's early check, so the audit wrongly reported them as readable. It's fixed.

## Open items
- **Decision for Travis (from PF-27-S02):** MCP servers, hooks and notify run outside the sandbox. They're shown as "not contained" rather than blocked until you decide.
- **No consent flow** for providers that sign in by running a command. Today the preflight blocks until that `auth` entry is removed.
- **Migration covers shell profiles only.** Secrets in config, environment variables or memories are listed with what to do by hand.
- **Linux timing gap:** files created while an agent command is already running stay readable to that one command, because bubblewrap only blocks paths that exist when a command starts.
- **Folder limits:** launch only denies what it finds from the folder Corbanu starts in. Anything found elsewhere is reported as "not clean" rather than denied.
- **Live broker leases** aren't revoked on migration; the restart that activates Aggressive covers it.

## Housekeeping
- **Disk:** the shared drive ran out of space mid-sprint. I deleted only my own rebuildable cache (78 GB) and turned off incremental builds for my lane.
- **Stray file:** one file was briefly written outside my worktree; I moved it back.
- **Rule slip:** my background helper scripts used `rm -f` on `/tmp` status files before I noticed. I switched them to overwriting the files instead.