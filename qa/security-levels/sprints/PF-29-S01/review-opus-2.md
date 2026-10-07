VERDICT: APPROVE WITH NITS

I only read the code for this review. I didn't run cargo or any tests. The resume fix was checked only at the two chokepoints in the diff. I didn't trace every caller.

**Fixes I checked:**
- **P1, unsafe file reads:** fixed. `open_regular` (`core/src/security/inventory.rs:465-491`) now skips anything that isn't a regular file, opens with `O_NONBLOCK`, checks again on the open handle, and reads at most MAX+1 bytes before rejecting. The age-header probe (`:1336`) uses the same function now.
- **P1, resume refusal:** fixed for the TUI. The check is now in `resume_thread` and `fork_thread_at_with_presentation` (`tui/src/app_server_session.rs:911,1007`). Leaving exec and app-server clients outside it is fine, because Aggressive is TUI-only and the review screen says so.
- **P2, secrets in Debug output:** fixed. The new `Debug` for `InventorySources` shows only variable names. `ConfigLayerInput` leaves out the TOML contents, and `Preflight` leaves out the key.
- **P2, per-file isolation:** fixed. Isolation now covers whole folders, and directories go through `paths` into `isolation_paths`. Side effect: `~/.ssh/config`, `known_hosts` and `*.pub` are also blocked under Aggressive. That's acceptable, but please mention it in the docs.
- **Smaller items:** the new config-only message, millisecond timestamps, and deleting the receipt when the level save fails all look right.

**Findings**

- **P2: old receipts switch the resume gate off.** `tui/src/security/preflight.rs:33,104`. `activated_at` now counts milliseconds, but `RECEIPT_VERSION` is still 1. A receipt written before this change stores seconds (about 1.7e9). Read as milliseconds, that's a date in January 1970, so every thread looks newer than activation and none are refused. QA and dev profiles from the earlier runs would have these receipts.
  - Fix: change `RECEIPT_VERSION` to 2. The existing `version !=` check then treats old receipts as Unverified. Add a test that loads a version-1 receipt.
- **P3: unclear error when resume is refused at the chokepoint.** `app_server_session.rs:913,1009`. `bail!(message)` sends the refusal back as a generic resume/fork error. Some callers may wrap it in text like "failed to resume", which hides the reason.
  - Fix: return a typed error, or check that the refusal text appears unchanged on the startup `--resume` path.
- **P3: `O_NONBLOCK` is set only on Unix.** That's fine for now, because Windows has no FIFOs reachable through `metadata().is_file()`. A short comment saying so would help.