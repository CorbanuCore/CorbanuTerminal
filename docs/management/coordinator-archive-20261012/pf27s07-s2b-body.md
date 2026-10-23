Slice 2b of PF-27-S07, stacked on slice 2a (#282); the base becomes `main` once #282 merges.

Before this, Core started the broker with `std::process::Command`, so its process and first thread got Core's default DACL. A same-user process could open either one before the broker hardened itself and keep the handle.

**`spawn_protected` (process-hardening)**
- The process and its first thread are created with the protected DACLs (`CreateProcessW` security attributes).
- The process starts suspended. Its token's default DACL is set to the protected thread DACL (`TOKEN_ADJUST_DEFAULT` on the child's token), then it resumes. Every thread it ever starts is protected at creation, including the loader's and the thread pool's.
- `DETACHED_PROCESS`: no console, so no conhost child and no console control events from the TUI.
- Stdout is a pipe and is the only inherited handle (`PROC_THREAD_ATTRIBUTE_HANDLE_LIST`). Stdin and stderr are closed.
- The child gets exactly the environment it is given.
- `ProtectedChild` mirrors the parts of `std::process::Child` that the broker client uses: `id`, `kill`, `wait`, `try_wait`.
- Limit: the stdout write end is inheritable for the duration of the `CreateProcessW` call, so a process `std` spawns concurrently could inherit it. That pipe carries only the broker's public pipe name, and Core checks the server process ID.

**Broker client (network-proxy)**
- On Windows the broker is started with `spawn_protected`.
- The Unix start is unchanged, only moved into `start_broker_process`. The environment rules are the same: provider keys removed, launcher removals, then launcher variables.

**Tests (Windows CI)**
- `pf_27_s07_protected_spawn_is_never_openable`: a target started with `spawn_protected` that never hardens itself is fully denied to a same-user process without privileges and to the unelevated sandbox's restricted token. That covers process rights, the environment canary, every thread, and two brand-new threads held in their creation window: one through the `CreateThread` import, one through `CreateRemoteThread`.
- The PF-27-S04/S06 broker suite (`credential_broker::isolated::`) now starts every broker through `spawn_protected`.
- Unit tests for command-line quoting and the environment block.

macOS: the network-proxy and process-hardening suites pass.

Sprint record: `docs/sprints/current/p1-security-hardening/pf-27-s07-windows-hardening-follow-ups.md` (#280).
