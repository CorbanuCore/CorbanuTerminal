**VERDICT: CHANGES REQUESTED**

One finding blocks approval: on Windows, the broker fails to start when Core inherits the `=C:` drive variables that `cmd.exe` sets. The protection itself looks correct: the process and first thread get their DACLs at creation, and the token change runs before any of the broker's own code. I only read the code; I didn't build it or run the tests.

### High

**H1 — `windows_protected_spawn.rs:343-345` rejects environment variables Windows treats as valid.**
- Processes started from `cmd.exe` (and anything they launch) carry hidden variables like `=C:=C:\work` and `=ExitCode=…`.
- Rust's `std::env::vars_os()` returns these with the name `=C:`, because it looks for the separator `=` only after the first character.
- `client.rs:1030-1036` copies the whole environment, and `environment_block` refuses any name containing `=`. So `spawn_protected` returns `InvalidInput` and the broker never starts.
- The test helper (`windows_process_access_tests.rs`, the `protected_spawn` branch) has the same problem, so CI launched from bash or pwsh hides it.
- `std::process::Command` passes these variables through, so this is a regression from slice 2a.
- **Fix:** reject `=` only after the first character, e.g. `name[1..].contains(&'=')`. Add a unit case for `("=C:", r"C:\x")`.

### Low

**L1 — `:152-155` A concurrent spawn can inherit the stdout write end.** The doc comment admits this. Core doesn't take std's spawn lock, so a sandboxed tool command that `std` spawns at the same moment could inherit the broker's stdout write end. It could then inject bootstrap lines or hold the pipe open.
- What limits it: `connect_control` checks that the pipe server's process ID is the broker's, and nothing secret travels over stdout. Worst case is a denial of service.
- **Fix:** drop stdout bootstrapping on Windows. Have Core create a named pipe with a random name and an explicit DACL, pass the name as an argument, check the client's process ID, and spawn with `bInheritHandles=FALSE`. If you keep the current design, document the accepted risk in the plan and make sure the bootstrap reader thread exits on timeout.

**L2 — `:305-310` NUL characters aren't rejected.** An argument or program path containing a NUL silently cuts off the command line or `lpApplicationName`. `std` returns `InvalidInput` instead. **Fix:** reject NUL the way the environment check does.

**L3 — `:355-359` Sort order doesn't match Windows.** `to_uppercase` applies full Unicode case mapping, which can expand one character into several, and lossy UTF-16 conversion can change names. Windows sorts the block with ordinal case-insensitive comparison, as `std` does via `CompareStringOrdinal(…, TRUE)`. The harm is limited to non-ASCII names. Relatedly, `client.rs` removes duplicates with `eq_ignore_ascii_case`. **Fix:** compare with `CompareStringOrdinal(…, TRUE)` or an ASCII uppercase comparison on the UTF-16 units, and remove duplicates in `environment_block`.

**L4 — `:198-201` Cleanup can hang.** On a failure after `CreateProcessW`, `child.wait()` waits forever even if `kill()` failed. **Fix:** wait only if `kill` succeeded, or use a short timeout.

**L5 — Test gaps (`windows_process_access_tests.rs` ~205-226).**
- **Not covered:** the window between `CreateProcessW` and `ResumeThread`. The probes run only after the target reports ready. The design looks correct (descriptors set at creation, and nothing can start a thread without a process handle), but "never openable" isn't measured for that window. Add a test hook that pauses before resuming, probe the suspended child, then resume.
- **Not covered:** a Windows-side check that provider keys and removed launcher variables (in any case) are absent from the broker's environment. The equivalent tests exist only for Unix.

### Checked, no issue
- **Command-line quoting:** follows the MSVC / `CommandLineToArgvW` rules. Backslashes are doubled only before a quote or the closing quote, and empty arguments are quoted. `lpApplicationName` is an absolute path (`current_exe` or the configured program), so there's no search-path ambiguity.
- **Handles:** the handle list names only the write end, and the attribute list and `handles` array outlive the call. Every error path drops the pipe ends and the thread handle. Leaving stdin/stderr as NULL handles makes Rust treat stdin as EOF and drop stderr writes, the same as `Stdio::null()` before.
- **Token default DACL:** `CreateProcessW` gives the child its own token copy and returns a full-access handle, so `OpenProcessToken(TOKEN_ADJUST_DEFAULT)` works without touching Core's token. Named pipes use explicit security attributes (`pipe.rs:154-171`). The child's first thread can't grant itself `WRITE_DAC`, so `protect_threads` skips it with no error. Setting the process DACL goes through the pseudo-handle. The job self-assignment uses the pseudo-handle too. Moving the default DACL from containment time to process start doesn't change what the broker's own code sees, since containment already set it in slice 2a.
- **`DETACHED_PROCESS`:** has the same effect as the old `CREATE_NO_WINDOW` for console control events, and the broker starts no children.
- **`kill`/`wait`:** match `std`. Exit code 1, `ERROR_ACCESS_DENIED` after exit counts as success, and the handle is kept open so the exit status stays readable.
- **The new test:** it does measure the steady state it claims. It covers every thread of an unhardened target, including the two threads held in their creation window, for both the same-user probe and the restricted-token probe.
