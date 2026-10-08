**VERDICT: CHANGES REQUESTED**

The process DACL is correct in itself. It does not meet the mandate yet, because the threads it leaves open let another process take over the hardened process. I reviewed the diff only; I did not build or run anything.

## High

**1. Threads keep the default DACL, so a same-user process can hijack the hardened one.** `windows_process_access.rs:44-69`
- Only the process object is changed. Existing and future thread objects still get the token's default DACL, which gives the user full access.
- A same-user process outside the sandbox (an MCP server or a hook) can call `OpenThread(THREAD_SUSPEND_RESUME|THREAD_GET_CONTEXT|THREAD_SET_CONTEXT)`. With `SuspendThread` and `GetThreadContext`, it can read the registers, which may hold pointers or pieces of a secret.
- With `SetThreadContext`, it can point the instruction pointer at existing code (a ROP chain or a `LoadLibrary` gadget). That is code execution inside Core or the broker, which defeats the whole protection without ever needing `PROCESS_VM_READ`.
- Whether the sandbox's restricted token can do the same depends on whether the default DACL's logon SID passes the restricted-SID write check. That needs measuring, not assuming.
- **Fix:**
  - Walk the current threads with `CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD)`. Apply the same protected DACL to each with `SetSecurityInfo(SE_KERNEL_OBJECT)`, using thread-specific rights such as `THREAD_QUERY_LIMITED_INFORMATION|SYNCHRONIZE`.
  - Make new threads secure too: set `TokenDefaultDacl` on the process token with `SetTokenInformation`. Because that also changes the DACL of every object the process creates, including child processes, children (the agents) must be spawned with explicit security attributes or a separate token.
  - Add probes that the hardened target denies `THREAD_SET_CONTEXT`, `THREAD_GET_CONTEXT` and `THREAD_SUSPEND_RESUME` on every thread, including one created after hardening.

## Medium

**2. The probes skip some routes and have no positive controls for the same-user and handle-duplication cases.** `windows_process_access_tests.rs:55-83`
- **Same-user probe:** only the restricted-token probe has an unhardened control. Add one where a plain same-user probe reads an unhardened target, so a broken same-user probe can't pass by accident.
- **Duplication and DACL rights:** `dup_handle` and `write_dac` are never shown to be granted on an unhardened target. Assert both are `Granted` in the control.
- **Rights not probed:** add checks that `PROCESS_VM_WRITE`, `PROCESS_VM_OPERATION`, `PROCESS_CREATE_THREAD`, `PROCESS_SUSPEND_RESUME`, `WRITE_OWNER` and `PROCESS_SET_INFORMATION` are all denied. `CREATE_THREAD` combined with `VM_WRITE` is the classic injection route; the DACL denies it today, but nothing tests that.
- **Leftover probe on timeout:** `WaitForSingleObject(..., 60_000)` ignores its return value. A hung probe is left running, and its stdout may still be shared with later tests. Call `TerminateProcess` on `WAIT_TIMEOUT`.
- **Target readiness can hang:** `Target::spawn` (`lines.any(...)`) has no deadline. A target that blocks before printing `ready` stalls the job until the 120-minute timeout. Read on a thread and use `recv_timeout`.

**3. Window before hardening, and handles opened before hardening survive.** This affects later slices, but it should be stated in this function's contract.
- The environment block exists from process creation, and the DACL is only replaced afterwards. Any secret placed in the environment, or loaded before `restrict_current_process_access()` runs, is readable during that window.
- Any process that opened Core during the window keeps its handle.
- **Fix:** document that the call must come first in `main`, before any secret is loaded and before any thread or child is spawned. Later slices must deliver secrets only after hardening, never through the environment. Add a test that a handle opened before hardening still works, to pin this behavior down.

## Low

**4. What the remaining user rights still reveal.**
- `PROCESS_QUERY_LIMITED_INFORMATION` still allows reading the command line (`ProcessCommandLineInformation`, Windows 8.1+) and the image path. That's fine, since the mandate already bans secrets on the command line, but write it down.
- SYSTEM and debug-privileged administrators bypass the DACL by design; the doc comment already says so.
- On elevated tokens the owner is `BUILTIN\Administrators`. The `OW` ACE then removes that group's implicit `WRITE_DAC`. The user ACE doesn't grant `WRITE_DAC`, so this is correct.
- **Fix:** add an elevated-runner probe, since GitHub's `windows-2022` runners run elevated. Confirm it doesn't hold `SeDebugPrivilege` by default; if it does, the "denied" results there are misleading or wrong.

**5. The PEB offsets assume a 64-bit target.** `windows_process_access_tests.rs:403-405`
- The offsets are only right for 64-bit builds (x64 and ARM64). On a 32-bit build the positive control would fail rather than falsely pass.
- **Fix:** gate with `#[cfg(target_pointer_width = "64")]`.

**6. FFI is sound.**
- The `TOKEN_USER` buffer is 8-byte aligned and large enough.
- Each `LocalFree` frees what the matching API allocated.
- The token handle is closed exactly once.
- The DACL pointer stays alive for the duration of the call.
- **Optional:** check the `GetTokenInformation` size query for `ERROR_INSUFFICIENT_BUFFER` explicitly.

**7. Workflow triggers are mostly right.**
- The paths cover the crate, the sandbox crate and the lock file.
- Also add `codex-rs/Cargo.toml` (workspace dependency changes) and `.github/actions/setup-ci/**`.
- `cancel-in-progress` on pushes to `main` can cancel evidence runs on main. Limit it to PRs: `cancel-in-progress: ${{ github.event_name == 'pull_request' }}`.

**8. Test interference is acknowledged but not removed.**
- Spawning children concurrently through Rust std (which inherits all inheritable handles) and through the sandbox's inheritable pipes can delay end-of-file on a pipe. Reading only the report line works around that.
- **Fix:** run the `pf_27_s06` tests with `--test-threads=1` in the workflow, or serialize them with a mutex. That way a passing run doesn't depend on the workaround.
