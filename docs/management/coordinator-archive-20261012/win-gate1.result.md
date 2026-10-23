**The PF-27-S06 gate failed on the real Windows machine: I found two security defects. PF-27-S06 stays `blocked` and is not archived.** The docs-only PR #293 is merged (`7f361233aa`), and all 5 videos are published.

I built from `origin/main` at `63ea3d0cbd0c`. sec-win2's later merges (#280, #282) are not in this build.

**Tool versions** (all under `C:\CorbanuQA`):
- Rust 1.95.0 MSVC.
- VS 2022 Build Tools 17.14.37710 (MSVC 14.44.35207, Windows SDK 10.0.26100). A second Build Tools install, 17.14.37314, was already in Program Files; I didn't touch it.
- MSYS2 runtime 3.6.10, with tmux 3.7c, Python 3.12.15 and Git 2.56.
- asciinema 2.4.0. Version 3 has no Windows build, so a small shim translates the recorder's version 3 flags.
- agg 1.9.0 and ffmpeg 9.0.2.
- Already present: Git for Windows 2.54.0, gh 2.93.0, Python 3.12.10.

**What passed:**
- The PF-27-S06 tests on the machine: 38 of 38.
- Under the unelevated sandbox, agent commands are refused and the reason is shown.
- Broker pipes and a foreign client: another process of the same user connects and is dropped with 0 bytes received; the sandbox user can't open the pipe at all.
- Under the contract, the agent can't read `auth.json`, `config.toml` or the state database, and can't write `config.toml`. With the flag off, the same probe reads them all (control).
- The recorder works under MSYS2 tmux, so the Windows Terminal fallback wasn't needed.

**What failed:**
1. **Normal (unelevated) session: every agent command fails.** With `secretless_agent_launch` on, commands fail with `CreateProcessWithLogonW failed: 5`. They work with the flag off, or when Corbanu runs elevated. The likely cause is the contract's process-memory protection blocking Windows from opening Core to launch the command. sec-win2's PF-27-S07 work (#282, #284) extends that same protection.
2. **The vault store is readable.** With the normal `workspace-write` settings, an agent command reads `CODEX_HOME\secrets\*`. The contract's per-path denies are never applied as file permissions. The other files are protected only by the deny on `CODEX_HOME` itself, which doesn't reach subfolders.

CI missed both because its Windows runners run elevated and its test uses narrower read permissions than the product. A branch `sec/pf-27-s06-d2-vault-deny` has since appeared, so someone has picked up the second defect.

**Videos** (prefix `https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/`; both the recorder's leak check and a second scan of the published files found no key):
- `pf-27-s06-pf27s06-win-protected-files-63ea3d0cbd0c-2026-10-08.mp4` (shows defect 2)
- `pf-27-s06-pf27s06-win-baseline-flag-off-63ea3d0cbd0c-2026-10-08.mp4` (control)
- `pf-27-s06-pf27s06-win-unelevated-refused-63ea3d0cbd0c-2026-10-08.mp4`
- `pf-27-s06-pf27s06-win-broker-pipe-foreign-client-63ea3d0cbd0c-2026-10-08.mp4`
- `pf-27-s06-pf27s06-win-normal-session-launch-fails-63ea3d0cbd0c-2026-10-08.mp4` (shows defect 1)

They're indexed in `qa/demos/index/PF-27-S06.md`. The gate section is in `qa/security-levels/sprints/PF-27-S06/README.md`, and the sprint record now says it's blocked on these two defects. Codespell flagged the .NET name `InOut` in one recorded spec, so I added it to `.codespellignore` rather than edit the spec after recording.

**For Travis:**
- **Both defects block PF-27-S06** and probably affect PF-27-S07. After the broker lane fixes them, the gate needs a rerun in a normal session.
- **Agent commands under the admin-set-up sandbox can't run from an SSH session.** They time out on a pipe connection with no useful error. I ran everything in the console session through scheduled tasks; normal-session runs were started via `explorer.exe` so they get the user's ordinary permissions.
- **The admin-set-up sandbox's one-time setup is stored per Corbanu home.** In a normal session it needs a UAC click, which a script can't give. I copied the result of an elevated setup into each normal-session run's disposable home. Defect 1 also occurs on a home the product set up itself, and outside tmux.
- **Changes left on the Windows machine:**
  - The product's setup created two sandbox users, a sandbox group and firewall rules. It also gave that group write access to the user's `%LOCALAPPDATA%\Temp`.
  - I set `git config --global core.longpaths true`.
  - During cleanup I force-killed every `powershell.exe` once to stop a hung script, which would also have closed any PowerShell windows open on that machine.

**Cleanup:**
- **Removed on Windows:** the 33 GB build directory, all run directories, the Z.AI key file, the copied sandbox setup files and my scheduled tasks.
- **Kept on Windows:** the toolchain and a source clone. The key-file scan found no key in the scripts or logs.
- **On the Mac:** my worktree and branch are removed. The rerun scripts are in `.codex-work/workers-20261002/win-gate1/`, the full write-up in `win-gate1.result.md`, and the defect details in `win-gate1-findings.md`.