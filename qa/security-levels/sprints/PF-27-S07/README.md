# PF-27-S07 evidence: Windows hardening follow-ups

These are two of the four limits PF-27-S06 documented, which Travis approved fixing on 2026-10-08. The other two are planned as [PF-27-S08](../../../../docs/sprints/archive/p1-security-hardening/pf-27-s08-windows-broker-restricted-token.md) (the broker's own token) and [PF-27-S09](../../../../docs/sprints/current/p1-security-hardening/pf-27-s09-windows-model-client-auth.md) (Windows model auth).

Everything here sits behind the same default-off flags: `isolated_credential_broker` and `secretless_agent_launch`. Permissive mode, macOS and Linux are unchanged; on Unix the broker start was only moved into a function.

All Windows results come from GitHub's `windows-2022` runners, measured by the `windows-security-probes` workflow. These runners are elevated administrators with `SeDebugPrivilege` enabled, so any probe that stands in for an ordinary process disables its privileges first, as in PF-27-S06.

## What shipped

| Slice | PR | What |
| --- | --- | --- |
| 1 | #281 | **`CODEX_HOME` deny removed when the flag is off.** The first config load of a process that has `secretless_agent_launch` off and no armed contract removes exactly the entry `add_deny_read_ace_for_new_files` adds: an explicit deny for `CodexSandboxUsers`, flags OI\|IO\|NP, with a mask that maps to file read. The copies that files directly in `CODEX_HOME` inherited go with it. Every other entry stays, and so does the DACL's protection. Each contract holds `CODEX_HOME\.secretless-launch.lock` shared. A removal takes it exclusively without waiting, so it never runs under an armed process. A contract without the lock refuses protected launches (`CodexHomeLockUnavailable`). The lock file must be a regular file with one link; it cannot be deleted while open and gets its own explicit deny for the sandbox's users. |
| 2a | #282 | **New threads protected from creation.** In Core and the broker, `restrict_current_process_access` redirects this image's `CreateThread` imports to a wrapper that passes the protected thread descriptor, so every Rust thread (std, tokio) is protected at creation. The broker also sets its token's default DACL to the protected thread DACL, which covers every thread whoever starts it. |
| 2b | #284 | **The broker is never openable.** Core starts the broker with `spawn_protected`: process and first thread are created with the protected DACLs, suspended, and the token's default DACL is set before the broker runs. It has no console and inherits only its stdout pipe. |

## Measured on windows-2022

| Probe | Result |
| --- | --- |
| Brand-new thread created through the `CreateThread` import, held in its creation window (created suspended, so its TLS callback has not run), opened by a same-user process without privileges and by the unelevated sandbox's restricted token | `GET_CONTEXT`, `SET_CONTEXT` and `SUSPEND_RESUME` denied to both |
| Same, but the thread is created without the import (`CreateRemoteThread` on itself) in a Core-style target (positive control) | same-user: all granted; restricted token: `GET_CONTEXT` granted. The probe does reach the window. |
| Both kinds of new thread in a broker-style target (default DACL protected) | denied to both probes |
| A thread created protected, acting on itself | can set its priority and set and read its own description; the TLS callback records no failure |
| A process started by `spawn_protected` that never hardens itself | every process right, the environment canary, every thread and both new threads denied to both probes |
| The same process while still suspended, before its first instruction | process and first thread denied to both probes. Control: the same process started suspended by `std` is readable (`PROCESS_VM_READ`, `GET_CONTEXT` granted). |
| A protected-spawned process that then runs the broker's hardening | hardening succeeds. It cannot rewrite DACLs it was created with and need not, since they are checked to be as strict. Everything stays denied. |
| Broker suite over pipes, the broker started with `spawn_protected` | the PF-27-S04/PF-28-S02/PF-33-S02/PF-27-S06 suite (24 tests) passes |
| Flag-off removal: SDDL of `CODEX_HOME` and of an existing file | identical before the add and after the removal. A file created while the deny was present ends up with the same DACL as a fresh one. A second removal is a no-op. |
| A protected DACL; a wider deny with the same group and flags; another SID's new-file deny | all kept |
| Removal while a contract holds the lock | skipped; done once the contract is dropped |
| Contract that could not take the lock | refuses protected launches, then takes the lock once it is free |
| Lock file | deletion refused while held; a symbolic link or a hard link at the lock path refused; keeps its own explicit deny after the removal |

CI jobs (`windows-security-probes`), all green on the merged heads:

| Slice | Job | Tests passed |
| --- | --- | --- |
| 1 | 113293258039 | windows-sandbox 4, core 9 (`pf_27_s06`/`s07`), broker 24 |
| 2a | 113280283309 | process-hardening 12, broker 24, core 5 |
| 2b | 113292935422 | process-hardening 18, broker 24, core 5 |

## Found and fixed on the way

- **The TLS callback was failing on protected threads.** A thread created protected has, on its own handle, only what Windows computes from its DACL plus a baseline, so it has no `WRITE_DAC`. The callback now treats access denied as "already protected". `ImpersonateSelf` also fails on such a thread; Core and the broker never impersonate (limit below).
- **A protected-spawned broker could not harden itself.** Its process DACL and its threads' DACLs were set at creation, and it cannot rewrite them; a debug privilege still opens the thread, but the write is denied. The broker suite failed on CI until hardening accepted DACLs it reads back as being at least as strict.
- **The ACL test's per-file deny was silently skipped.** S06's `add_deny_read_ace` counts an inherited write deny as a read deny (the masks overlap in `READ_CONTROL` and `SYNCHRONIZE`). The new explicit-deny helper does not.
- **A `cfg(windows)` statement broke the Linux build.** Linux clippy on the RTX box caught it before merge.

## Known limits

- **Core's threads started by other modules.** Windows thread pools, RPC and injected DLLs still get the protected DACL only in their TLS callback. Loader workers skip that callback and keep the default DACL. Core cannot change its token's default DACL: its child pipes, created without a descriptor and then reopened, and its child processes would inherit it. The broker has no such gap.
- **Protected threads cannot change their own DACL or impersonate.** Nothing in Core or the broker does either.
- **Broker stdout inheritance.** The broker's stdout write end is inheritable only for the duration of the `CreateProcessW` call, so a process `std` spawns at the same moment could inherit it. That pipe carries only the public pipe name, and Core checks the server's process ID, so the worst case is a denial of service.
- **Mixed versions.** A PF-27-S06 build holds no lock, so a PF-27-S07 build with the flag off can remove the deny under it. Restart protected sessions before turning the flag off.
- **Interrupted removal.** If propagation fails partway, some root files keep an inherited copy. This fails safe: those files stay denied to the sandbox, and nothing retries.
- **Lock-file creation window.** If an armed contract creates the lock file before the sandbox group exists, or before its deny is applied, a sandboxed command could open it in that moment. Holding it shared only blocks cleanup (fails safe). Holding it exclusively makes protected launches refuse (fails closed).
- **The lock file stays.** It keeps its own explicit deny after the removal.
- **Lock path occupied.** A hard link or a symbolic link at the lock path makes protected launches refuse until the link is deleted.
- **Real-Windows gate.** Done on 2026-10-08; see [Real Windows gate run](#real-windows-gate-run-2026-10-08-pass).

## Reviews (Opus 5.5 High)

| Slice | Rounds |
| --- | --- |
| 1 | Changes requested (a lock a sandboxed process could hold, fail-open without the lock, cheap check), then approve with lows (an explicit deny on the lock file, retry, hard links), then approve with lows. Waiting on the launch path removed; error message and removal-order test added. |
| 2a | Changes requested (fail-safe ordering, self-rights, positive controls), then approve. Page protection is now restored in reverse order. |
| 2b | Changes requested (`=C:` variables), then approve with lows. All fixed: full denial asserted after the broker's hardening, plus a comment on the control and a missing SAFETY comment. |

Texts: [reviews/](reviews/).

## Real Windows gate run (2026-10-08): PASS

Host: the PF-27-S06 machine (Windows 11 Pro 25H2, build 26200; local admin `User` at the console; same toolchain).
Candidate: debug build of `origin/main` at `265172beed3d`, which has #281, #282 and #284 and the PF-27-S06 fixes
#298 and #302. Every run below was in a **normal session**: medium integrity, no `SeDebugPrivilege`, started
through `explorer.exe`. Config: `[windows] sandbox = "elevated"`, `workspace-write`, `isolated_credential_broker` and
`secretless_agent_launch` on (off where stated). GLM 5.2 (`zai`) drove the TUI in tmux on a fresh `CODEX_HOME` per run.

The probe (`threadprobe`, C# source in the specs) only opens handles. It tries 11 process rights and 5 thread rights
(`GET_CONTEXT`, `SET_CONTEXT`, `SUSPEND_RESUME`, `TERMINATE`, `QUERY_INFORMATION`) on every thread. Its control is a
child it starts itself, which it can always open.

| Check | Result |
| --- | --- |
| Core, same-user process (the user's `!` command) | **pass**: only `QUERY_LIMITED` and `SYNCHRONIZE` (the protected DACL's user entry); 0 of 245 thread opens (49 threads) |
| Core, agent command (sandbox user `CodexSandboxOffline`) | **pass**: no process right; 0 of 255 thread opens |
| New threads: a same-user watcher polls Core and the broker during an agent turn and opens each new thread as soon as it appears | **pass**: 6 new Core threads and 2 new broker threads, none opened |
| Broker (started with `spawn_protected`), same-user process | **pass**: only `QUERY_LIMITED` and `SYNCHRONIZE`; 0 of 40 thread opens |
| Broker, agent command | **pass**: no process right; 0 of 40 thread opens |
| `CODEX_HOME` deny, SDDL (script): after a flag-off run, then a flag-on run, then a flag-off `corbanu features list` | **pass**: the flag-on run adds `(D;OINPIO;0x80120089;;;<CodexSandboxUsers>)` to `CODEX_HOME`, and files in it (one created before, one after) get `(D;ID;FR;...)`. After the flag-off process, the SDDL of `CODEX_HOME` and of the earlier file match the baseline exactly. A second flag-off process changes nothing |
| Same in the TUI: flag-off session after a protected `corbanu exec` | **pass**: the entry is gone from `CODEX_HOME` and `config.toml` |
| A flag-off process while a protected session holds the lock | **pass**: the deny stays |
| #295 regression: agent commands in a normal session | **pass**: `whoami` runs as `CodexSandboxOffline` |
| #294 regression: vault under `workspace-write` | **pass**: the vault store, `auth.json`, `config.toml` and the state database are denied, and so is writing `config.toml`; the workspace control file is read |
| Probe suites, elevated (as CI): process-hardening 18 + 1 (`pf_27_s06_d1`), windows-sandbox 4, broker 24, core 12 | pass |
| Same suites in a normal session | pass, except 2 core tests that CI also runs only elevated (below) |

The two core tests that need an elevated session:
- `pf_27_s06_elevated_launch_cannot_read_protected_files_or_core_memory` runs the elevated sandbox's admin setup,
  which needs a UAC answer (`ShellExecuteExW ... 1223`).
- `pf_27_s07_armed_lock_refuses_links_and_deletion` can't create its symbolic link without
  `SeCreateSymbolicLinkPrivilege` (error 1314). The same limit stops a normal-session process from planting the link,
  unless Developer Mode is on.

Both pass elevated. Neither is a product failure.

**What CI covers and what it doesn't.** `windows-security-probes` (on `windows-2022`) proves two things
deterministically:
- a thread can't be opened while it is held in its creation window;
- the broker can't be opened while it is still suspended.

CI runs all `pf_27_s07` tests elevated, with privileges disabled in the probe. Of the PF-27-S07 work, it runs only the
#294/#295 probes at medium integrity. It does not cover:
- the running product's real Core and broker;
- a medium-integrity session (outside those two probes);
- the TUI.

This run covers those, but its watcher can't prove it reached the creation window. The deny-removal SDDL check and
the lock are covered by both. Neither covers a separate non-admin account, or the admin setup from a normal session.

**Videos** ([index](../../../demos/index/PF-27-S07.md); specs `qa/demos/specs/pf27s07-win-*.toml` and the PF-27-S06
regression specs). Leak scan: no key value and no key-shaped string in any cast, frame, log or published file (the
recorder's scan, and a second scan on macOS of the run folders and the downloaded release assets).

| Video | Shows |
| --- | --- |
| `pf27s07-win-core-threads-unopenable` | Core and its new threads unopenable by the user's other processes and by the sandbox |
| `pf27s07-win-broker-unopenable` | the same for the broker |
| `pf27s07-win-codex-home-deny-flag-off` | a flag-off session removes the deny an earlier protected run left |
| `pf27s07-win-codex-home-deny-kept-while-armed` | a flag-off process leaves it while a protected session runs |
| `pf27s06-win-normal-session-works` | #295 regression |
| `pf27s06-win-vault-denied` | #294 regression |

**How it was run.** Harness as for PF-27-S06 (scripts in `.codex-work/workers-20261002/win-gate-s07/`, not in the
repository). Normal-session runs reuse the result of one elevated setup run just before. The
`codex-home-deny-flag-off` launch wrapper runs one protected `corbanu exec` in the run's `CODEX_HOME` before the TUI
starts, and writes what it left to `before.txt`, which the video prints.

Notes:
- The removal's `info` log line never reaches `codex-tui.log`, because config loads before the file logger starts.
  The SDDL is the evidence.
- The seed wrapper's `.sandbox-secrets` deny had silently failed under MSYS2 path conversion, also in the PF-27-S06
  runs. It's fixed in this harness. The product's own setup was unaffected.
- `QUERY_LIMITED` and `SYNCHRONIZE` on the broker for the same user are by design (`process_dacl_sddl`); the sprint's
  "every process right" means the rights its tests probe (`PROCESS_RIGHTS`), which don't include those two.
