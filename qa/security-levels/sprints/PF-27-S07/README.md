# PF-27-S07 evidence: Windows hardening follow-ups

These are two of the four limits PF-27-S06 documented, which Travis approved fixing on 2026-10-08. The other two are planned as [PF-27-S08](../../../../docs/sprints/current/p1-security-hardening/pf-27-s08-windows-broker-restricted-token.md) (the broker's own token) and [PF-27-S09](../../../../docs/sprints/current/p1-security-hardening/pf-27-s09-windows-model-client-auth.md) (Windows model auth).

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
- **Still not done.** The GLM 5.2 tmux run and the SOP videos need a real Windows machine, as for PF-27-S06.

## Reviews (Opus 5.5 High)

| Slice | Rounds |
| --- | --- |
| 1 | Changes requested (a lock a sandboxed process could hold, fail-open without the lock, cheap check), then approve with lows (an explicit deny on the lock file, retry, hard links), then approve with lows. Waiting on the launch path removed; error message and removal-order test added. |
| 2a | Changes requested (fail-safe ordering, self-rights, positive controls), then approve. Page protection is now restored in reverse order. |
| 2b | Changes requested (`=C:` variables), then approve with lows. All fixed: full denial asserted after the broker's hardening, plus a comment on the control and a missing SAFETY comment. |

Texts: [reviews/](reviews/).
