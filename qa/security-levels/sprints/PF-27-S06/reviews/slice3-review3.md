**VERDICT: APPROVE**

I reviewed only the code in `git diff 6a45591a3e HEAD -- codex-rs`. I didn't build or run anything, and I didn't check CI job 113179399269 myself.

**H1 is fixed.** New files in CODEX_HOME are now denied to the sandbox.
- **The new rule:** `acl.rs:~649-672` adds a deny-read entry to CODEX_HOME for the sandbox's users group, `CodexSandboxUsers`. It applies only to files created directly in CODEX_HOME, so subdirectories such as `skills/` stay readable.
- **It checks its own work:** it re-reads the directory's permissions to confirm the entry is there. The existing-entry check also matches the exact flags, so the call is safe to repeat.
- **Launches fail closed:** `launch_contract.rs:~395-416` refuses the launch when the group doesn't exist or the entry is missing afterwards.
- **Both launch paths are covered:** `sandboxing.rs:495-501` applies it to every elevated Windows launch. `apply_patch.rs:112-115` uses the protected profile only after it succeeds; otherwise the patch gets no file access.
- **Real token refresh is covered:** `login/src/auth/storage.rs:202-218` rewrites `auth.json` in place. That keeps the per-file deny. Logout followed by login creates a new file in CODEX_HOME, which gets the new deny.
- **The test now asserts the result:** it requires DENIED for LATER, REPLACED and WAL (`launch_contract_windows_tests.rs:~228-233`), and the readable control moved into `skills\`. Points 1–4 of my earlier fix are done.

**Medium**
- **M1. A file moved into CODEX_HOME can keep its own permissions.**
  - **Why:** on NTFS, a file renamed in from another directory on the same volume isn't given the inherited deny. The test's rename keeps the temp file inside CODEX_HOME, so the gap only appears if a future writer creates its temp file elsewhere (for example under `%TEMP%`) and then renames it in.
  - **Fix:** add a doc comment saying protected writers must create their temp files inside CODEX_HOME. Add a test assertion, or have writers set the deny explicitly after the rename.

**Low**
- **L1. The "already applied" flag ignores which directory it applied to.**
  - **Where:** `launch_contract.rs:~397`. `APPLIED` is one flag for the whole process.
  - **Why:** a second contract with a different CODEX_HOME in the same process, as in tests or app-server, would skip protection.
  - **Fix:** key the cache by the canonical `codex_home`, using a `Mutex<HashSet<PathBuf>>`.
- **L2. The flag isn't refreshed if setup rewrites the directory's permissions.**
  - **Where:** same cache.
  - **Why:** if the elevated setup is re-run later in the process and resets CODEX_HOME's permissions, the cached `true` stays and the deny isn't re-applied.
  - **Fix:** clear the cache after setup or refresh, or re-check the permissions on each launch (it's cheap).
- **L3. The CODEX_HOME ≠ workspace check from my earlier fix wasn't added.**
  - **Why it's low:** with that layout the deny would only block reading the workspace's top-level files, not writing them. It's a self-denial, not a leak.
  - **Fix:** in `protect_launch`, refuse when `cwd` is CODEX_HOME or inside it, or record why that isn't needed.
- **L4. The sleep-based timing in the during-run test is unchanged.** It's recorded as a follow-up; replace the ~5 s ping / 2 s sleep with a file the test creates as the signal.
- **L5. The extra credential-file denies are recorded as a follow-up.** Those are `%USERPROFILE%\.git-credentials`, `.ssh`, `.npmrc` and `.config\gh`; that's fine.

**Follow-up**
- **One-way permission change:** the deny entry stays on CODEX_HOME permanently. Mention this in the user docs and the uninstall or reset steps.
