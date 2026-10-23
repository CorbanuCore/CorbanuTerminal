The Windows broker and secretless launch now work on Windows and are measured on the `windows-2022` CI runners. Everything is merged to main, but the sprint's gate isn't complete: the GLM 5.2 tmux run and the videos still need a real Windows machine.

## What shipped
All five PRs merged with every check green (merge commits, `--match-head-commit`):

| PR | What it does |
| --- | --- |
| #267 (slice 1) | Core and the broker now lock other processes out of their memory: no process can open them with `PROCESS_VM_READ`, read their environment, duplicate their handles or take over their threads. Also adds the `windows-security-probes` CI job. |
| #269 (slice 2a) | Named-pipe transport for the broker: random pipe names nobody else can claim, a DACL that grants only the user, remote clients refused, no inheritable handles, and the process id checked on both ends before any data moves. |
| #270 (slice 2b) | The broker now runs on Windows over those pipes. It can't start other processes or touch the desktop or clipboard, and it exits when its controller dies. The existing broker test suite (24 tests) passes on Windows. |
| #272 (slice 3) | Windows launches are no longer refused outright. They pass under the elevated sandbox (agent commands run as a separate user) and are refused with a stated reason under the unelevated one. |
| #277 | Sprint record and evidence in `qa/security-levels/sprints/PF-27-S06/`, including all 10 review texts. |

## Gate evidence
- **Windows tests:** the CI job runs the `pf_27_s06` tests and the broker suite on every PR touching this code. On the final merged head (job 113190329753), 38 tests pass.
- **Measured results:**
  - **Files:** under the contract the agent can't read the vault, `auth.json`, `config.toml` or the SQLite files. The same command reads all of them without the contract, and an unprotected file stays readable.
  - **Files that appear during a run:** a new file in `secrets/`, an `auth.json` replaced by rename, and a new SQLite WAL file are all denied.
  - **Memory:** a process under the unelevated sandbox's token, and an ordinary process of the same user, can open an unhardened process but not a hardened one.
  - **Broker pipes:** a pipe client from another process is dropped before any data is sent.
- **Linux:** clippy with `-D warnings` is clean on the RTX box for every slice, and the Linux and macOS suites pass.
- **Reviews (Opus 5.5 High):** every slice ended at APPROVE after two or three rounds.
- **Not done:** the GLM 5.2 tmux run and the SOP videos.

## Bugs found by the Windows runs (all fixed)
- The CI runners have `SeDebugPrivilege` enabled, which opens any process regardless of its DACL. The probes now drop their privileges first, as an ordinary process would have them.
- The broker suite hung because the controller's first message waited behind its own pending read on the same pipe handle. The control pipe is now overlapped.
- An `auth.json` replaced while an agent command was running was readable by that command. Protected launches now add a deny on `CODEX_HOME` that every new file in it inherits.

## Needs Travis
1. **Accept the documented limits:**
   - A brand-new Core thread can be opened by an unsandboxed same-user process for a moment before it is protected.
   - The deny on `CODEX_HOME` stays after the flag is turned off.
   - The broker on Windows still isn't confined by its own restricted token.
   - Windows model auth (PF-27-S05) stays Unix-only.
2. **The P1 plan has to be activated before this record can leave `draft`.** That needs a plan slot (3/3 are used) and its worktree coordinates added to the plan. Until then the lifecycle checker rejects any other status.
3. **A Windows machine for the tmux run and videos:**
   - Windows 11 Pro/Enterprise 23H2 or later (or Server 2022/2025 with Desktop Experience), x64, as a real machine or full VM. Windows Sandbox or a container won't work because the elevated sandbox creates local users.
   - A local admin account for the one-time sandbox setup, then a normal session as that user.
   - At least 8 cores, 16 GB RAM and 60 GB free.
   - OpenSSH Server and RDP.
   - Git, VS 2022 Build Tools (C++ workload and Windows SDK), Rust 1.95 MSVC, Python 3.12+, MSYS2 with tmux, asciinema 3.x, agg, ffmpeg and gh.
   - Network access to GitHub, crates.io and Z.AI.
   - A Z.AI key reachable on that machine.
   - The video recorder has only ever run on macOS and Linux, so the first step is checking it works with tmux under MSYS2. The fallback is Windows Terminal plus ffmpeg screen capture.

My worktrees, build directories, Docker images and volumes, and the RTX scratch folders are removed. Nothing I removed was referenced by the owner settings. The short result is in `.codex-work/workers-20261002/sec-win1.result.md`.