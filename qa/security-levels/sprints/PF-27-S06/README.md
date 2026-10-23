# PF-27-S06 evidence: Windows broker and secretless launch

Flags: `isolated_credential_broker` and `secretless_agent_launch` (both default off). Permissive is unchanged, and
nothing changes on macOS or Linux beyond a refactor of the broker's transport code (Linux and macOS suites rerun).

**The real-Windows gate run on 2026-10-08 failed with two defects; see
[the gate section](#real-windows-gate-run-2026-10-08-failed-two-defects).** The sections before it are the original
CI evidence. No Windows host was available then. Those results are measured on GitHub's `windows-2022` runners by the
`windows-security-probes` workflow, which runs on every PR that touches this code. The runners are elevated
administrators with `SeDebugPrivilege` enabled, so probes that stand for an ordinary process disable their own
privileges first (an enabled debug privilege opens any process whatever its DACL; that is a documented limit).

## What shipped

| Slice | PR | What |
| --- | --- | --- |
| 1 | #267 | `restrict_current_process_access()`: protected process DACL (user: query-limited + synchronize; SYSTEM; OWNER RIGHTS: read-control) and the same for every thread, existing ones at once and new ones from a TLS callback. Hardening fails closed if the callback is not in the image's TLS directory. |
| 2a | #269 | Named-pipe transport: random first-instance names, protected DACL for the user, remote clients refused, no inheritable handles, peer process id checked before any byte, identification-level impersonation, an overlapped control pipe for the controller. |
| 2b | #270 | The isolated broker on Windows over those pipes; the broker contains itself with the DACL plus a job (no child processes, no desktop/clipboard/atoms/other USER handles); it holds its controller's process handle and exits with it. |
| 3 | #272 | The secretless launch contract passes on Windows under the elevated sandbox (a separate sandbox user) and refuses the unelevated restricted-token sandbox with a stated reason. A protected launch first gives `CODEX_HOME` an inherit-only, files-only read deny for the sandbox's users, so files created or replaced there later stay denied. |

## Measured on windows-2022

| Probe | Result |
| --- | --- |
| Command under the unelevated sandbox's restricted token vs an unhardened process (positive control) | memory and environment canary read |
| Same token, and a same-user process without privileges, vs a hardened process | `PROCESS_VM_READ`, `QUERY_INFORMATION`, `DUP_HANDLE`, `WRITE_DAC`, `WRITE_OWNER`, `VM_WRITE`, `VM_OPERATION`, `CREATE_THREAD`, `SUSPEND_RESUME`, `SET_INFORMATION` all denied; every thread (one started after hardening) denied `GET_CONTEXT`, `SET_CONTEXT`, `SUSPEND_RESUME`; environment unreadable |
| Same-user process vs an unhardened process (positive control) | every right granted |
| Broker pipe: same-user foreign client | connects (the DACL grants the user) and is dropped before any byte |
| Broker pipe: restricted-token client | read+write open denied; read-only open connects and gets nothing |
| Pipe handles | none inheritable; a child that scans its own handle table finds no broker pipe (an inheritable duplicate is found: positive control) |
| Wrong server pid, look-alike name | refused by the controller |
| Contained broker | containment `dacl+job`; it cannot start a process; another same-user process cannot open it to read memory |
| Broker suite over named pipes | the PF-27-S04/PF-28-S02/PF-33-S02 suite, 24 tests, passes (substitution only inside the broker, wrong peer, forged/replayed frames, revocation, crash, bounds, scrubbing, pinning) |
| Controller killed outright | the broker exits within 10 s |
| Contract decisions | unelevated restricted token refused; elevated accepted (also through `env_for`) |
| Elevated sandbox, base profile (positive control) | vault, `auth.json`, `config.toml`, `*.sqlite` and an unprotected file all readable |
| Elevated sandbox, contract's profile | vault, `auth.json`, `config.toml`, `*.sqlite` denied; unprotected file readable; `CODEX_HOME` not writable |
| File created in a protected directory during a run | denied (inherited deny) |
| `auth.json` replaced by rename, and a new `state_9.sqlite-wal`, during a run | denied (the files-only deny on `CODEX_HOME`); before that deny existed the replaced file was measured readable |
| Unprotected file in `CODEX_HOME\skills` under the contract | readable (control) |
| Elevated sandbox probe vs hardened and unhardened Core stand-ins | both denied: the separate sandbox user is already the boundary |

CI jobs (`windows-security-probes`): merged slice 3 head 113190329753 (all 38 tests); slice 1 113152390863; slice 2a 113167492899; slice 2b 113167757386 (broker
suite); slice 3 113179399269 (new-file deny). The merged heads were rerun green before each merge.

## Found and fixed on the way (all measured)

- The CI runner's `SeDebugPrivilege` opened the hardened target in the first run; probes now drop privileges.
- A synchronous pipe handle serializes all I/O on its file object: the controller's first write waited behind its
  own reader thread and the broker never received Hello. The controller's control pipe is now overlapped.
- The elevated runner re-quotes argv, so quoted paths in `cmd.exe` probes became invalid; the base-profile control
  caught it.
- The handle scan hung on inherited CI pipes with pending synchronous I/O; each query now has a deadline.

## Known limits

- **`CODEX_HOME` ACL change (contract).** The files-only deny for `CodexSandboxUsers` stays on `CODEX_HOME` after
  the flag is turned off (sandboxed commands then cannot read files directly in `CODEX_HOME`); reset or uninstall
  notes should mention it. A file *moved* into `CODEX_HOME` from another directory keeps its own ACL, so protected
  writers must create temporary files inside `CODEX_HOME` (the `auth.json` storage does). Until the elevated
  sandbox's setup has created its users group, protected launches are refused.
- **Threads (DACL).** A new thread has the default DACL until its TLS callback runs, and threads created without
  loader notifications keep it; a same-user unsandboxed process could open such a thread in that window. Commands
  under the elevated sandbox are another user and are denied either way. Needs Travis's acceptance (review 3).
- **Same-user unsandboxed processes** (MCP servers, hooks) can connect to the broker pipes and be dropped (denial of
  service, not disclosure), as PF-27-S02 already declares them not contained.
- **Broker containment** on Windows: no file-write confinement, no protection of other processes from the broker,
  and requests through WMI, Task Scheduler or COM are possible. A restricted or AppContainer token for the broker is
  the follow-up.
- **Model auth** (PF-27-S05) stays Unix-only; Windows `broker_model_auth` refuses as before.
- In-process file tools (structured edits, image view) get no file access on Windows under the contract; patches
  work only under the elevated sandbox; external agents (Claude panes) stay refused on Windows.
- Debug-privileged administrators and `SYSTEM` bypass any DACL. Credential Manager and DPAPI blobs are bound to the
  real user and are not readable by the sandbox user.

## Reviews (Opus 5.5 High)

Slice 1: changes requested (threads), changes requested (TLS linkage, window), approve. Slice 2a: changes requested
(read-only restricted connect, busy retry, timeouts), approve. Slice 2b: approve with findings, approve (overlapped
pipe and parent handle; its low finding was fixed in 2a). Slice 3: changes requested (replaced files, apply_patch,
vacuous control), changes requested (replaced files still open), approve. Texts: [reviews/](reviews/).

## Linux and macOS

Linux clippy (`-D warnings`) on the RTX box is clean for every slice; Linux `credential_broker` 55, process-hardening
7, core `launch_contract` 12 and `pf_27_s0` 23 tests pass. macOS: the same suites pass.

## Real Windows gate run (2026-10-08): FAILED, two defects

Host: Windows 11 Pro 25H2 (build 26200), 16 cores, 31 GB, local admin `User`, logged on at the console. Candidate:
debug build of `origin/main` at `63ea3d0cbd0c` (built on the host). GLM 5.2 (`zai`) drove the real TUI in tmux on a
fresh `CODEX_HOME` per run. Toolchain: Rust 1.95.0 MSVC, VS 2022 Build Tools 17.14 (MSVC 14.44.35207, SDK
10.0.26100), MSYS2 (runtime 3.6.10) with tmux 3.7c and Python 3.12.15, asciinema 2.4.0, agg 1.9.0, ffmpeg 9.0.2,
Git 2.54.0, gh 2.93.0.

| Check | Result |
| --- | --- |
| `pf_27_s06` suites on the host (elevated session, like CI): process-hardening 9, broker over pipes 24, core 5 | pass |
| Unelevated sandbox + contract: agent command refused with the stated reason | pass |
| Broker pipes, same-user foreign process (the user's `!` command) | data pipe: connects, dropped with 0 bytes received; control pipe: no free instance |
| Broker pipes, agent command (sandbox user `CodexSandboxOffline`) | data pipe open denied; control pipe: no free instance |
| Elevated sandbox + contract, product `workspace-write` profile: `auth.json`, `config.toml`, `*state*.sqlite`, writing `config.toml` | denied |
| Same probe: vault store `CODEX_HOME\secrets\local.age` (created before start or during the run) | **READ: defect 2** |
| Control, contract off: vault, `auth.json`, `config.toml`, state database | all read |
| Elevated sandbox + contract in a normal (medium-integrity) session | **every command fails: defect 1** |

**Defect 1: unusable in a normal session.** Started from the user's shell (medium integrity) with
`[windows] sandbox = "elevated"` and `secretless_agent_launch` on, every agent command fails with
`windows sandbox: CreateProcessWithLogonW failed: 5`. The same profile works with the flag off (broker on or off)
and from an elevated session. The Security log has no logon by the sandbox user, so the secondary logon service
refuses before logging on. Likely cause: the contract arms `restrict_current_process_access()` (user keeps only
query-limited and synchronize on Core), and the service opens the caller's process while impersonating it.

**Defect 2: the vault store is readable.** Under the product's `workspace-write` profile the contract's deny
entries are never applied as ACLs (`.sandbox\deny_read_acl_state.json` stays empty; the sandbox log shows only
the `.git` deny). `auth.json`, `config.toml` and the databases are denied only by the files-only deny on
`CODEX_HOME`, which does not reach `secrets\`. The CI probe passes because it uses a restricted-read base profile.

CI missed both because the `windows-2022` runners are elevated and the probe does not use the product profile.

### Videos

[qa/demos/index/PF-27-S06.md](../../../demos/index/PF-27-S06.md); specs `qa/demos/specs/pf27s06-win-*.toml`. Leak
scan: no credential value or key-shaped string in any cast (script scan and a second scan of the published files),
and no redactions in private logs.

| Video | Session | Shows |
| --- | --- | --- |
| `pf27s06-win-protected-files` | elevated | defect 2 next to the denials |
| `pf27s06-win-baseline-flag-off` | elevated | control: everything readable without the contract |
| `pf27s06-win-unelevated-refused` | normal | refusal with the reason |
| `pf27s06-win-broker-pipe-foreign-client` | normal (contract off) | foreign clients dropped or denied |
| `pf27s06-win-normal-session-launch-fails` | normal | defect 1 |

### How it was run (for reruns)

- The recorder works under MSYS2 tmux; native console programs run inside tmux panes through ConPTY. asciinema 3
  has no Windows build, so a shim maps the script's asciinema 3 flags to asciinema 2.4.0 (MSYS2 Python) and shifts
  its events by asciinema's start-up delay so cuts and holds line up. Publishing ran on macOS from the copied run
  directories. Shims and job scripts: `.codex-work/workers-20261002/win-gate1/` (not in the repository).
- The elevated sandbox's runner does not start from an SSH session (`timed out ... connecting runner pipe-in`), so
  runs were started in the console session by scheduled tasks: elevated runs directly, normal-session runs through
  `explorer.exe` so they get the user's ordinary token. MSYS2's `TEMP` was pointed back at the user's temp folder.
- The elevated sandbox's admin setup is stored per `CODEX_HOME` and needs a UAC answer in a normal session. Normal-
  session runs were seeded with the marker and sandbox-user record of the latest elevated setup (same user, same
  DPAPI scope, the setup's deny on `.sandbox-secrets`). Defect 1 reproduces identically on a profile set up by the
  product itself, and in `corbanu exec` started from `cmd.exe` without MSYS2.
- Each run's directory names showed in the TUI's cwd; one GLM reply remarked on the folder name. No tool call or
  result was affected.

## Gate rerun after the fixes (2026-10-08): PASS

Same host. Candidate: debug build of `origin/main` at `661b5c6a48cd`, which includes #298 (fixes #294) and #302 (fixes
#295). Every run below was in a **normal session**: medium integrity, started through `explorer.exe`, and the
recorder printed `Medium Mandatory Level`.

| Failed item | Rerun |
| --- | --- |
| Defect 1: agent commands fail with `CreateProcessWithLogonW failed: 5` | **pass**: `whoami` runs as `CodexSandboxOffline`. The sandbox log records `runner started through the logon launcher (protected process, #295)` |
| Defect 2: vault store readable under `workspace-write` | **pass**: vault store, `auth.json`, `config.toml` and the state database are denied, and so is writing `config.toml`; the workspace control file is read |
| `pf_27_s06_d1` (#295 probe), normal session | pass: the fallback ran (`via launcher: true`) |
| `pf_27_s06_d2` (#294 probes, product profile through the tool path), normal session | pass: 2 tests. The vault is denied to a protected launch, to an unprotected launch from an armed process, and to a command still running across it |

Before the fixes, the same probes failed on this host: `pf_27_s06_d1` in a normal session, from an elevated session
through the CI script, and (with PF-27-S07 merged) even elevated; both `pf_27_s06_d2` tests in both sessions. CI
now covers this: `windows-security-probes` runs the probes again with the token UAC gives an administrator's normal
session (`.github/scripts/run-at-medium-integrity.ps1`). What CI does not cover: a separate non-administrator
account, and the elevated setup itself from a normal session (it needs a UAC answer, so the step reuses a setup
recorded just before).

Videos ([index](../../../demos/index/PF-27-S06.md); leak scan: no credential value in any cast, screen or log, checked
by the recorder and again on macOS):

| Video | Session | Shows |
| --- | --- | --- |
| `pf27s06-win-normal-session-works` | normal | agent command runs as the sandbox user (defect 1 fixed) |
| `pf27s06-win-vault-denied` | normal | vault read denied under `workspace-write`, next to the other denials (defect 2 fixed) |

Follow-ups filed while fixing: #300 (unelevated tool path doesn't enforce deny-read; needs a product decision),
#301 (a flag-off session on the same `CODEX_HOME`), #304 (deny ACEs are never revoked), #307 (launcher pipe
inheritance window). The rerun used the same harness as the first run. The normal-session runs were seeded from the
setup of an elevated run made just before, because each elevated setup resets the sandbox passwords.

## Windows machine needed for the remaining gate

To rerun the gate after the fixes, use a Windows 11 Pro or Enterprise 23H2+ machine with:

- a local admin logged on at the console,
- OpenSSH,
- the toolchain above,
- a Z.AI key in a file only that user can read.

A real machine or full VM is required: the elevated sandbox creates local users, so Windows Sandbox and containers won't work.
