**VERDICT: CHANGES REQUESTED**

This was a code review only of 6a45591a3e against 67fbf94d36. I didn't build or run anything. The CI result (5/5 on windows-2022, job 113168396384) is as you reported it; I didn't check it myself. Every earlier finding is closed except H1.

**High (still open)**

**H1. A refreshed `auth.json` is readable by a running sandboxed command.**
- **Where:** `launch_contract.rs:~433-447`, which adds deny entries only for files that exist at launch. Your measurement is in `launch_contract_windows_tests.rs:~185-213`.
- **Why it blocks:** your own run confirms it (REPLACED-READ). Token refresh writes a temp file and renames it over `auth.json`, so any long-running ConPTY or unified-exec session started before a refresh or login can read the live credential.
- **Same gap for other root files:** new root-level `*.sqlite`, `-wal`/`-shm` and `state_N.sqlite` files created after launch are also unprotected.
- **Not a scope boundary:** this isn't an edge case to document and move on from. It's the main secret the slice protects, and it's exposed on a routine code path. A "known limit" in the sprint record doesn't close it.
- **The test doesn't guard it:** it asserts only `LATER-DENIED`. The REPLACED result is just printed, so neither the gap nor a future fix is tracked.

**Fix (in this PR, or as a blocking dependency before slice 3 ships):**
1. Implement the follow-up you proposed now: at elevated setup, put an inherit-only, files-only, non-propagating deny-read entry (`OI|IO|NP`) on CODEX_HOME for the sandbox user. New root files get the deny; `skills/` and `.sandbox-bin/` stay readable.
2. Keep the per-file denies for files that already exist, because inherited entries don't apply to them retroactively.
3. Change the test to `assert!(later.contains("REPLACED-DENIED"))`.
4. Add a case that creates a new `state_9.sqlite-wal` during the run and asserts it is denied.
5. Check that the `OI|IO|NP` entry doesn't block the sandbox user's own writes in the workspace. CODEX_HOME should never be the workspace, but assert that in the elevated setup.

If product authority explicitly accepts shipping with this limit, record that acceptance and add a test asserting `REPLACED-READ`, so a fix shows up as an expected change. Without that, this blocks approval.

**Closed**
- **H2:** `apply_patch.rs:105-116` now gets the protected profile on Windows only under the elevated backend; otherwise it gets no file access. That's correct, and patches don't go through the proxy, so `proxy_enforced=false` is fine.
- **M3:** the base-profile run now asserts all five READ, and the protected profile asserts `NOTES-READ`. The control is no longer vacuous.
- **M4:** the probe runs from a copy in the workspace, and the sandboxed probe of the unhardened stand-in is recorded as denied. The statement that the separate user is the boundary is acceptable, with the process DACL measured in slice 1.
- **M5:** `pf_27_s06_env_for_requires_the_elevated_windows_sandbox` covers RestrictedToken (refused) and Elevated (accepted). The proxy case relies on the existing backend tests, which is acceptable.
- **L6:** exec-server is refused on every platform, and the doc comment now says so.
- **L7:** `%APPDATA%\gcloud`, `CARGO_HOME\credentials(.toml)` and `hosts.yml` are now protected. Minor follow-up, not blocking: `%USERPROFILE%\.git-credentials`, `.ssh`, `.npmrc` and `.config\gh` still aren't on the deny list. That only matters if the elevated setup ever grants profile reads, and slice 3 currently grants only the platform, the workspace and CODEX_HOME. Add them anyway as defense in depth.
- **L8:** in the sprint evidence; I accept it.

**Low**
- The timing in the during-run test (ping for about 5 s against a 2 s sleep) could be flaky on slow runners. Use a file the test creates as the signal instead of a fixed sleep.
