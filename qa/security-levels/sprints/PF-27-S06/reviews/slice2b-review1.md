VERDICT: APPROVE

This is a read-only review. I didn't build anything or run tests.

**Unix regressions: none found.** The move into `mod transport` keeps the behaviour the same:
- **Socket directory:** still created as `0o700`, with sockets at `0o600`.
- **Peer checks:** the uid and pid checks are shared through `peer_is` for both the controller and the data connections.
- **Control socket:** it's unlinked after the controller is accepted, as before.
- **Shutdown signals:** SIGTERM, SIGHUP and SIGINT are still handled, and the 1 s parent-death poll is now folded into `shutdown_requested`. The signal handlers are still installed after `Ready`, as before.
- **Cleanup:** dropping the `Endpoint` still removes the directory through `TempDir`.
- **Windows-only cleanup path:** `remove_socket_dir` is harmless for a pipe name, because the parent of `\\.\pipe\…` has no `file_name`, so nothing is deleted.

### Medium

**M1. The job only stops direct process creation, not other ways to run code** (`process-hardening/src/broker_containment.rs:~315-345`). A compromised broker can still:
- ask another process to start one (WMI `Win32_Process.Create`, Task Scheduler, out-of-process COM servers);
- use window messages, `SendInput` or the clipboard on the user's desktop;
- open other same-user processes, including Core and agents, with `PROCESS_VM_WRITE` or `CREATE_THREAD`, because the DACL only protects the broker itself.

The tag `dacl+job` suggests more than that, and Unix blocks cross-process inspection.

Fix:
- Add `JOBOBJECT_BASIC_UI_RESTRICTIONS` (`DESKTOP | HANDLES | READCLIPBOARD | WRITECLIPBOARD | GLOBALATOMS | SYSTEMPARAMETERS | EXITWINDOWS`).
- Write down what is still possible in the sprint record and the containment doc comment.
- Plan a lowered-integrity or AppContainer token for the broker as a follow-up.

The test `pf_27_s06_contained_broker_cannot_start_processes` only exercises `CreateProcess`, so it doesn't support any broader claim.

**M2. `controller_pid` in Hello is accepted without being checked** (`server.rs:276-278`). The broker trusts `parent_pid` (from the OS) to accept the controller, but serves data connections for whatever `controller_pid` Hello names. Today only the authenticated controller can send Hello. Still, on Windows that field becomes the only rule for who may open the data pipe, so a controller bug or a mixed-up launcher would hand data access to another pid.

Fix: `anyhow::ensure!(*controller_pid == parent_pid, …)` on both platforms, or drop the field and use `parent_pid`.

### Low

**L1. Parent pid could be reused** (`pipe.rs parent_pid`). `InheritedFromUniqueProcessId` isn't checked against the parent's lifetime. If the controller dies before the broker queries it and the pid is reused by a same-user process, that process could win the control-pipe accept.

The window is tiny and the bootstrap only goes to the original stdout. Even so, compare the parent's creation time (`GetProcessTimes`) with the broker's own and refuse if the parent is newer. Open the parent's `SYNCHRONIZE` handle before binding, not after `Ready`, so `process_exit` waits on the right process.

**L2. Windows always waits the full 2 s at teardown** (`client.rs:704-713`, `reap_after_grace`). The broker never sees EOF, so every session waits the whole grace period and then kills it. That is acceptable for security, because killing the process wipes its memory, but it's slow.

Fix: send an explicit `ControlRequest::Shutdown` and handle it in the broker loop, or `CancelIoEx` the reader thread's handle before dropping it. Keep the kill as the backstop.

**L3. Concurrent spawns can inherit the broker's stdio pipes** (`client.rs` spawn). Rust std makes the child's stdio ends inheritable. Another Core spawn running at the same time (an agent) can inherit the broker's stdout write end and inject fake bootstrap lines.

This is mitigated: `connect_control` checks the server pid, and secrets travel only over the non-inheritable pipes. Note it in the threat model, or spawn through `PROC_THREAD_ATTRIBUTE_HANDLE_LIST`.

**L4. The orphaning path on Windows is not tested.** Nothing checks that the broker exits after the controller is killed with `TerminateProcess` (no graceful close). Add a test that kills the controller and asserts the broker pid is gone within a few seconds.

### Checked, no issue
- **No fallback to raw injection:** the `Unsupported` arm is now dead on Windows. A launch failure goes through `IsolatedMode { client: None }`, which fails closed rather than falling back to `InProcess`.
- **Containment is required when armed:** when containment is armed, `containment_sufficient` requires both `dacl` and `job`.
- **Data endpoint is validated:** `Ready`'s data name goes through `valid_pipe_name(false)`, and every data connection checks the server pid against `broker_pid`.
- **Nested jobs:** with nested jobs (Windows 8 and later) `AssignProcessToJobObject` succeeds. No breakaway flag is set and the job handle can't be inherited, so the broker can't leave its job. If Core's own job allows silent breakaway, the broker leaves Core's job, but it still exits through the `process_exit` wait.
- **No secrets on Core-created handles:** secrets cross only the control and data pipes created by the broker. The client's `File` handles from `std::fs` can't be inherited, and nothing secret goes over stdin or stdout.
- **Model auth correctly stays off on Windows:** arg0 on Windows dispatches with no stored-key resolver, and the `cfg(unix)` gates keep PF-27-S05 model auth off Windows.
