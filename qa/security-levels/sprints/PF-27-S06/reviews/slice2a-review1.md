VERDICT: CHANGES REQUESTED

I reviewed the diff and the token code it depends on, but did not run the tests.

**High**

1. **A sandboxed command can still open the pipe read-only, so claim 1 is false** (`pipe.rs:5-8`, `pipe.rs:118-140`, `pipe_tests.rs:105-115`, `pipe_tests.rs:214-230`).
   - The real unelevated token is built with `WRITE_RESTRICTED` (`windows-sandbox-rs/src/token.rs:479-481`). That flag only checks the restricting SIDs for write access.
   - Read access is checked against the token's normal SIDs, which include the user SID. The pipe's DACL grants that SID `GA`.
   - So a sandboxed process calling `CreateFileW(name, GENERIC_READ, …)` on either pipe should connect. The test only tries read+write, so its `open=denied` result can't catch this.
   - What it gets: the pid check in `accept` still rejects it before anything is read or written, so nothing leaks today. But the sandbox can now open broker pipes, which breaks the sprint's "cannot open the broker" mandate, and it causes the denial of service in finding 2.
   - Fix:
     - Correct the module doc and PR claim: the pid check is the guarantee, not the DACL.
     - Add a restricted-token test that opens `GENERIC_READ` only and asserts the client is dropped with zero bytes.
     - Look for a DACL that also blocks reads. Note that a deny ACE on the logon SID won't work, because the real user's token has that SID too.
     - The test also needs the pipe name. The sandbox can only learn it through what 2b exposes (environment, handles, `NtQueryDirectoryFile` on `\\.\pipe\`). Pipe names can be listed, so the random name is not a secret. Say so in the doc.

2. **Anyone who can connect can block Core and make the broker leak instances** (`pipe.rs:104-115`, `pipe.rs:214-233`).
   - There is only one listening instance. A read-only sandbox client (finding 1) or any same-user process can connect in a tight loop and keep taking it.
   - Each foreign connection makes the broker create another instance. Core's `connect_data` keeps hitting `PIPE_BUSY` and fails after 5 s, and `connect_control` fails on the first `PIPE_BUSY` because it has no retry.
   - Fix:
     - Keep a small pool of listening instances. For the control pipe, accept exactly once and then stop listening.
     - Add a backoff or limit on rejected clients.
     - Give `connect_control` the same busy-retry as `connect_data`.
     - Add a test with a concurrent flooding client.

**Medium**

3. **The broker and Core trust raw pids with no process handle held** (`pipe.rs:109`, `pipe.rs:209`, `pipe.rs:234`).
   - If the controller exits, its pid can be reused by a sandbox-spawned process, which would then pass `accept`. The same applies to Core's `broker_pid` if the broker dies.
   - Fix (enforce in 2b, note here):
     - The broker should hold a handle to the controller process and exit when that handle is signalled.
     - Core should check the broker's process handle with `WaitForSingleObject(h, 0)` after connecting, or compare process creation times.
     - Record this as a 2b requirement in the sprint.

**Low**

4. **`accept` error path** (`pipe.rs:106-107`). If `create_instance` fails after `connect`, `?` leaves the connected, unchecked client in `self.listening`. The next `connect()` returns success (`ERROR_PIPE_CONNECTED`), and only the pid check stops it. It's safe, but fragile. Fix: on error, disconnect `self.listening` before returning.

5. **The inheritance test may miss handles** (`pipe_tests.rs:234-248`). The child only scans handle values up to `0x10000`, so it can report a clean result even if handles exist above that. Fix: use `NtQuerySystemInformation(SystemExtendedHandleInformation)` filtered to the child's own pid, or raise the bound and assert the positive control was found.

6. **Test timeouts** (`pipe_tests.rs:387`, `pipe_tests.rs:325-335`). The restricted child waits 60 s, and the same-user child (`Command::output`) has no timeout. If `served=no` regressed into a server that never writes, the child's `read` would block until the server is dropped. Fix: add an explicit timeout on the child's read and on the parent's wait for the child.

7. **DACL grants `GA` to the same user** (`pipe.rs:177`). This includes `FILE_CREATE_PIPE_INSTANCE`, so same-user processes can add server instances. Core's server-pid check handles that (they are already trusted at the same-user level), but it should be written down as an assumption.

**Checked, no issue found**

- **Naming:** a 128-bit random nonce; first-instance creation means the name can't be squatted or joined. `valid_pipe_name` matches the prefix, length and hex exactly, so look-alike names are refused.
- **Remote clients:** `PIPE_REJECT_REMOTE_CLIENTS` is set.
- **Impersonation:** clients connect at identification level, so a pipe server can't act with Core's token. std and tokio both add `SECURITY_SQOS_PRESENT` when QoS flags are set.
- **Restricted callers can't create server instances:** creating an instance needs `FILE_CREATE_PIPE_INSTANCE`, which is a write right that write-restricted tokens fail.
- **No race between connect and the pid check:** the pid belongs to the connecting process and can't be swapped without same-user handle duplication.
- **Inheritance:** handles have `bInheritHandle=0`, and std and tokio client opens are non-inheritable.
- **FFI:** the security descriptor outlives `CreateNamedPipe` and is freed with `LocalFree`; the out-pointers are valid.
- **CI:** the new probe step runs with `--test-threads=1`, which avoids the concurrent-spawn inheritance race noted in the test's own comment.
