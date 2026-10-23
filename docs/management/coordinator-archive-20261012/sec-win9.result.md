Both fixes are merged, and both issues are closed with an evidence comment on each. Each PR passed an independent Opus 5.5 High review (approved in round 3), real-machine testing and fully green CI before merging.

## What shipped
**#354, fixes #345 (merged as `2f3e04201f`)**
- **Who gets access:** over SSH, window-station and desktop access now goes to the runner's own logon, not the sandbox user. Other Windows users' sandboxed commands can't use it.
- **What commands keep:** sandboxed commands lose that access once the runner reports they've started.
- **How long it lasts:** a small helper process removes the access when the runner exits, even if Corbanu has already exited.
- **The lock:** edits to that access are serialized by a per-user lock file. Its folder now grants only the user and SYSTEM, and a folder that is a link or owned by another user isn't used.
- **`sandbox_private_desktop = false` over SSH:** works now with the non-admin sandbox.

**#356, fixes #323 (merged as `5d283fde18`)**
- **The registry:** every session that relies on deny-read entries registers in `%ProgramData%\CorbanuTerminalSandbox\deny-read-sessions` while it runs. A sync removes entries only when no other session is registered.
- **Ownership:** the elevated setup creates that folder, owned by Administrators. A folder that is a link or owned by anyone else is never trusted.
- **Session files:** a live session's file can't be deleted or renamed.
- **One process, several rule sets:** once a process has used more than one set of deny rules (no rules counts as one), it removes nothing until it exits.
- **Upgrade cost:** the setup version is now 6, so updating asks for Administrator approval once more. While the registry is missing or untrusted, no entry is ever removed.

## Evidence (real Windows 11 machine)
- **Tests on #345:** the `sec_win_` tests pass 21/21 over SSH and in the console session, at both high and medium integrity.
- **Lock test on #345:** a sandboxed command can't open the lock but can open a control file next to it. A first version of this test always passed, so I added the control file to make it meaningful.
- **Tests on #323:** `sec_win_` passed 25/25 on its own, and 31/31 after merging in #345, over SSH and in the console session, high and medium.
- **Failing before the fix:** with the earlier behaviour patched back in, the new tests fail.
- **Cross-user check:** I ran a check as a standard local user, with no admin rights. They couldn't delete or rename another session's file, create sub-folders, delete the sync lock, rename the folders or change the folder's ACL.
- **End to end:** a two-home `corbanu sandbox` run kept the protection while the other home's command ran, and removed it afterwards.
- **Clippy:** Windows clippy `-D warnings` is clean for every crate I touched.
- **Linux clippy:** the RTX box (100.99.88.49) was unreachable throughout, so I relied on PR CI's Linux clippy, which passed. All changes are Windows-only code.

## Known gaps (both fail safe, both documented)
- **#323:** a local user who creates that ProgramData folder before the first setup, and keeps it open, can keep protected launches refused. No protection is ever removed this way.
- **#345:** a command in the non-admin sandbox runs as the user, so it can still hold the lock file. The only effect is that access edits go ahead unlocked, and those are logged.
- **An unrelated test that fails locally:** `pf_27_s07_armed_lock_refuses_links_and_deletion` fails in a normal session because that session can't create symlinks. It fails the same way on main, and CI runs it elevated only.

## Cleanup
- **Windows:** I removed my test user, my 8 scheduled tasks, `C:\CorbanuQA\win9` and my temporary probe folders. `C:\CorbanuQA\target` and the product's own registry and lock files are kept.
- **Mac:** both worktrees are removed; no target directories were created.

The full report is in `.codex-work/workers-20261002/sec-win9.result.md`.