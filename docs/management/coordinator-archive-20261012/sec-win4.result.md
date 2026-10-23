The PF-27-S07 gate passed on the real Windows machine, and every check passed. The docs-only PR #316 is merged (`f4da7b26`), and the sprint record is now `ready`. I didn't archive it; that's left to the P1 integration owner.

**Commit tested:** `origin/main` at `265172beed3d`, built on the Windows host. That build includes the PF-27-S07 changes (#281, #282, #284) and the earlier Windows fixes (#298, #302). All runs were in a normal (medium-integrity) session with no debug privilege, started the same way sec-win3 did. Both PF-27-S07 flags were on (off where a check needs that), GLM 5.2 on `zai`, and a fresh disposable Corbanu home for each run.

**What passed:**
- **Core and its threads can't be opened.**
  - Another process of the same user gets only the two harmless rights the design allows (`QUERY_LIMITED`, `SYNCHRONIZE`). It couldn't open any of Core's 49 threads with any right tried (0 of 245 attempts).
  - The sandbox user gets no access at all.
- **New threads are protected.** A same-user watcher ran during an agent turn and tried to open each new thread the moment it appeared. It saw 6 new Core threads and 2 new broker threads and opened none.
- **The broker can't be opened.** A same-user process gets only the same two harmless rights and couldn't open any of the broker's 8 threads (0 of 40 attempts); the sandbox user gets nothing.
- **The config-folder deny is removed when the flag is off.**
  - A flag-on run adds the deny entry to the folder, and the files in it pick it up.
  - A later flag-off process removes it. The folder's and an existing file's permissions (SDDL) then match the state before the flag-on run exactly. A second flag-off run changes nothing.
  - The deny stays in place while a protected session is still running.
- **Regressions still fixed:** agent commands work in a normal session (#295), and the vault is denied under `workspace-write` (#294).
- **Test suites on the machine:** all pass when elevated, which is how CI runs them. In a normal session, two core tests fail because they need admin rights:
  - one needs a one-time admin setup that waits for a Windows admin prompt (error 1223);
  - the other needs the right to create symbolic links (error 1314).
  
  CI also runs these two only elevated. I don't count them as product failures, so I filed no issues.

**Videos** (the recorder's leak scan plus a second scan on the Mac, including the downloaded release files; no key found). All under `https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/` and indexed in `qa/demos/index/PF-27-S07.md`:
- `pf-27-s07-pf27s07-win-core-threads-unopenable-265172beed3d-2026-10-08.mp4`
- `pf-27-s07-pf27s07-win-broker-unopenable-265172beed3d-2026-10-08.mp4`
- `pf-27-s07-pf27s07-win-codex-home-deny-flag-off-265172beed3d-2026-10-08.mp4`
- `pf-27-s07-pf27s07-win-codex-home-deny-kept-while-armed-265172beed3d-2026-10-08.mp4`
- `pf-27-s07-pf27s06-win-normal-session-works-265172beed3d-2026-10-08.mp4`
- `pf-27-s07-pf27s06-win-vault-denied-265172beed3d-2026-10-08.mp4`

The new "Real Windows gate run" section in the PF-27-S07 sprint evidence README says what CI covers and what it doesn't. In short:
- CI proves new threads and a just-started broker can't be opened. It runs elevated, though, and never tests the real running product, a normal session or the TUI.
- This run covers those, but the watcher can't prove it caught a thread in its first instant.
- Neither covers a separate non-admin account.

**For Travis:**
- **Missing log line.** The "removed the CODEX_HOME new-file deny" message never reaches `codex-tui.log`, because the config is loaded before the file logger starts. It's minor; I didn't file an issue.
- **Seed script bug in earlier videos.** The script that reuses the admin setup for normal-session runs silently failed one permission step, in the PF-27-S06 runs too. It's fixed in this run's scripts; the product's own setup wasn't affected.
- **Broker rights.** The sprint text says "every process right" is denied on the broker. It means the rights its tests check; the two harmless rights above are allowed by design.
- **The flag-off video's starting state is set up off screen.** Before the TUI starts, a script runs one flag-on session that adds the deny, and the video prints what it left behind. I noted this in the evidence README.
- **Still open on the sprint:** your acceptance of the known limits listed in the sprint record.

**Cleanup:**
- **Windows:** I removed the Z.AI key, the temporary local user `pf27s07logon` and its password file, all my scheduled tasks, the run folders and the sandbox seed files. I also removed `C:\CorbanuQA\clippy1`, a leftover clippy build folder from an earlier worker. I kept `C:\CorbanuQA\target`, the toolchain and the source clone. A scan of the scripts, logs and shims found no key-shaped strings.
- **Mac:** the worktree and branch are removed. My scripts, probe source and logs are in `.codex-work/workers-20261002/win-gate-s07/`.