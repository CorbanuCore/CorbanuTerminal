VERDICT: APPROVE

I reviewed the diff from `fcbb605fd7` to `HEAD` in `codex-rs/network-proxy` and `.github`, read only. I did not build anything or run the tests. The windows-2022 CI results you quoted (job 113155298127, 7/7 pass) are taken as given.

**Fixes from the first review**

- **H1 (sandboxed command can connect read-only): fixed.**
  - The module doc (`pipe.rs:12-21`) now says the peer check is the guarantee, that a write-restricted token can still connect read-only, and that pipe names can be listed.
  - `pf_27_s06_restricted_token_cannot_use_broker_pipe` (`pipe_tests.rs:109-125`) checks both cases: read+write is `open=denied`, and read-only is `open=granted,served=no`.
  - No byte reaches a foreign client.
- **H2 (connection flood blocks Core): fixed as scoped.**
  - `connect_control` (`pipe.rs:217-234`) now retries on `PIPE_BUSY` for 5 s, the same way `connect_data` does.
  - A flood from a same-user process is documented as an availability risk only, which is acceptable because that process is already trusted at the same-user level.
- **L4 (unchecked client left connected): fixed.** If `create_instance` fails, `pipe.rs:114-121` disconnects the unchecked client before returning.
- **L5 (inheritance scan may miss handles): fixed.**
  - The scan now goes up to `0x100000` and uses `GetHandleInformation` to find valid handles.
  - Each pipe-name query runs on its own thread with a 200 ms deadline.
  - The positive control is asserted as `(1, 0)` at `pipe_tests.rs:206`.
  - Threads stuck on a hung handle are left behind, but only in a short-lived child process, so that's fine.
- **L6 (test timeouts): fixed.**
  - The same-user child is read through the shared `collect_report`, which stops after 60 s, then killed.
  - A restricted child that times out is terminated.
- **L7 (DACL grants the user everything): fixed.** The assumption is now documented at `pipe.rs:16-17`.

**Low (non-blocking)**

1. **The module doc describes protection this slice doesn't have yet** (`pipe.rs:19-21`).
   - It says the broker "holds a handle to its controller and exits with it". That is M3 behaviour from slice 2b, and nothing in 2a does it.
   - If 2a merges first, the doc claims a protection against reused process ids that doesn't exist.
   - Fix: add "(enforced by the broker launcher, PF-27-S06 slice 2b)". Or move the sentence into 2b, and make sure 2b's sprint checklist has a test that the broker exits when its controller exits.
2. **Hard-coded timeout constant** (`pipe_tests.rs:453`). The literal `0x102` should use `windows_sys::Win32::Foundation::WAIT_TIMEOUT`. This is cosmetic.
3. **M3 itself is deferred to slice 2b** (`pipe.rs:109`, `pipe.rs:234`). Before 2b is accepted, confirm three things there:
   - the broker opens its controller with `SYNCHRONIZE|QUERY_LIMITED` before binding the pipes;
   - the broker refuses a parent that was created after itself;
   - Core checks the broker's process handle is still alive after `server_pid` matches.

**Re-checked, no regressions**

- The busy-retry still checks the server's process id after it connects, so retrying doesn't weaken who Core will talk to.
- The disconnect-on-error path can't hand back a connection that hasn't passed the check.
- The CI probe step still runs with `--test-threads=1`.
