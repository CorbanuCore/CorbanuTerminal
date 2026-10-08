**VERDICT: CHANGES REQUESTED**

I reviewed the diff only (`7d7ab75da4..HEAD`). I did not build or run anything, and I took the CI result (run 37726895482) as reported.

## First-review findings

| # | Status |
|---|---|
| 1 Thread hijack | **Mostly fixed.** Existing threads, and threads started later through the normal loader path, get the protected DACL. Gaps below. |
| 2 Probes and controls | **Fixed.** All the listed rights are probed, the positive controls are in place, waits are bounded and a hung restricted probe is terminated. |
| 3 Hardening window | **Fixed.** It is documented in the module comment. A test that a handle opened before hardening keeps working is still missing (optional). |
| 4 Remaining rights, elevated runner | **Fixed.** Command-line and image-path reads are documented, and the probe disables all privileges first. |
| 5 64-bit offsets | **Fixed.** |
| 7 Workflow | **Fixed.** |
| 8 Test interference | **Fixed:** the tests now run with `--test-threads=1`. |

## Medium

**A. A new thread is open to hijack until its TLS callback runs.** `windows_process_access.rs:183-203`
- A new thread's object is created with the token's default DACL, which gives the user full access. The callback only replaces that DACL later, at `DLL_THREAD_ATTACH`.
- During that gap, a same-user process outside the sandbox can call `OpenThread(THREAD_SET_CONTEXT|THREAD_SUSPEND_RESUME)` on the new thread ID and keep the handle afterwards. Changing the DACL later does not take back handles already open.
- Tokio's blocking pool creates threads all the time, so an attacker that keeps scanning for new threads gets many tries.
- **Fix, preferred:** set `TokenDefaultDacl` to a DACL that gives the user `THREAD_QUERY_LIMITED_INFORMATION|PROCESS_QUERY_LIMITED_INFORMATION|SYNCHRONIZE`, plus GA for SYSTEM and RC for OWNER RIGHTS.
  - Your reason for skipping this (Core must still open and kill its own children) mostly doesn't hold: `CreateProcess` returns full-access handles to the creator whatever the DACL says. Only reopening a child by PID breaks.
  - The real cost is that children copy the token's default DACL. Give children a duplicated token whose default DACL is restored; the sandbox already builds its own restricted token for agent commands.
- **Fix, minimum:** document this race as a known limit in the module comment, and get the security owner to accept it.

**B. Some threads never run TLS callbacks.**
- Threads created with `THREAD_CREATE_FLAGS_SKIP_THREAD_ATTACH` never reach `DLL_THREAD_ATTACH`, so they keep the default DACL. This includes the ntdll loader's worker threads that start after hardening, and any thread a third-party DLL creates this way.
- **Fix:** the default-DACL change in A covers these too. Otherwise, document the gap.

**C. Nothing makes sure the callback is linked in.** `windows_process_access.rs:208-211`
- `#[used]` only keeps the symbol in its own object file. Whether the MSVC linker keeps `.CRT$XLC` in the final Core binary depends on that object being pulled from the rlib and on `/OPT:REF` not removing the section. `codex-process-hardening` is a library linked into a different binary, so this is not guaranteed.
- CI proves it only for the test binary, where the module is compiled into the test crate.
- **Fix:** reference the callback from `restrict_current_process_access`, for example `std::ptr::read_volatile(&THREAD_ATTACH_CALLBACK)`.
- **Fix:** add a debug assertion, or a check on the release binary, that its TLS directory lists the callback.
- **Fix:** probe a thread started after hardening in the real Core binary, not only in the test harness.

## Low

**D. The callback ignores failure.** `:197`
- If `NtSetSecurityObject` fails in the callback, the thread stays unprotected and nothing records it.
- **Fix:** count failures in an atomic counter and report the count, for example from a debug or status call.

**E. The callback is otherwise sound.**
- It is safe under the loader lock: it does one atomic load and one ntdll call, with no allocation and no advapi32.
- The descriptor is published before the snapshot is taken, so a thread can't be missed between the snapshot and the publish.
- The descriptor is leaked on purpose, so the callback never reads freed memory.
- Thread-pool and std threads go through the loader, so they get the callback.

**F. The positive control can pass without checking every right.** `windows_process_access_tests.rs:113`
- The same-user control loops over whatever keys the probe reported. If keys are missing, it still passes.
- **Fix:** assert the report length, as `assert_all_denied` already does.

**G. The test can't catch A or B.**
- **Fix:** add a probe that scans for new thread IDs while the target keeps spawning threads, and record its results. If the window is accepted rather than closed, this at least measures it.

**H. Minor robustness.**
- `open_threads` returns `granted` when any one thread opens. That is the right severity semantics, but report which thread ID opened, to make failures easier to diagnose.
