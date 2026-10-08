**VERDICT: CHANGES REQUESTED**

The redirect itself is sound. Two failure paths now switch off slice-1 protection, and one kernel behaviour that everything relies on has no recorded proof. I could not see a windows-2022 run for this branch.

**Medium**

1. **`broker_containment.rs:322-324`: a broker failure skips the old protection.** The `&&` short-circuits. If `protect_new_objects_by_default()` fails (for example, the token can't be opened with `TOKEN_ADJUST_DEFAULT`), `restrict_current_process_access()` never runs. The broker then has neither the slice-1 process DACL nor the thread DACL.
   *Fix:* call both every time. Report `dacl` when the restrict call succeeds and `default-dacl` as a separate mechanism.

2. **`windows_process_access.rs:158`: a redirect error stops all hardening.** `redirect_thread_creation()?` returns before the thread snapshot and before the process DACL is set. A failed `VirtualProtect` (EDR, odd section layout) or a missing import now leaves Core with no hardening, not just without the new feature. It can also leave some import slots patched and others not.
   *Fix:* always run the snapshot and set the process DACL, then return the redirect error at the end. Also check every slot can be made writable before writing any of them.

3. **Rights a protected thread has on itself are unproven.** When Windows creates a thread, it checks the new thread's descriptor and records the rights the thread gets through its own pseudo-handle (`GetCurrentThread()`). The protected DACL gives the user only `QUERY_LIMITED|SYNCHRONIZE`. The `OWNER RIGHTS` entry only grants `READ_CONTROL`, which removes the owner's implicit `WRITE_DAC`. So unless Windows adds a broad baseline:
   - The TLS callback (`on_thread_event`, line 200) calls `NtSetSecurityObject(GetCurrentThread())`, which needs `WRITE_DAC`. It would fail and increment `THREAD_PROTECT_FAILURES` for every redirected thread.
   - `ImpersonateSelf` and impersonation reverts need `THREAD_SET_THREAD_TOKEN`. This includes ntdll thread-pool workers in the broker, which now get this DACL too.
   - Thread names would silently disappear: std and tokio call `SetThreadDescription`, which needs `THREAD_SET_LIMITED_INFORMATION`, and ignore its result.

   The target's failure-count assert and `ImpersonateSelf` check should catch the first two, but only a green windows-2022 run proves it.
   *Fix:* record that run in the sprint evidence. Extend `assert_threads_keep_rights_to_themselves` (tests:385) to cover `SetThreadDescription` and `GetThreadDescription` on itself. Run it in both target modes, and assert that `thread_protection_failures()` is still 0 afterwards. If the callback does fail on already-protected threads, have it skip them rather than count a failure.

**Low**

4. **`windows_thread_creation.rs:67-68`: possible call through a null pointer.** The wrapper converts `ORIGINAL_CREATE_THREAD` to a function without checking it. On ARM64, another thread could in theory see the new slot value before it sees the stored original. The original also comes from the first slot only, which is fine as long as every slot resolves to the same function.
   *Fix:* if the value is 0, return null with `SetLastError(ERROR_NOT_READY)`, or resolve `CreateThread` with `GetProcAddress`. Never call back through the import table.

5. **Remaining gaps are wider than the docs say.** Only this image's import table is patched. Threads started by any other DLL still wait for the TLS callback: RPC, ws2/DNS thread pools, combase, injected AV DLLs. In the Windows targets the CRT is linked statically (`+crt-static`), so `_beginthreadex` in V8 and zstd is covered; keep that build setting. Loader worker threads skip the TLS callback and keep a full-access DACL for their whole life in Core. A same-user process could use `SET_CONTEXT` on one of them to run code in Core.
   *Fix:* say "threads started by any other module" in the module docs. Track the loader workers as an open hole, either with a periodic re-sweep in Core or by moving Core to the default-DACL approach once its child pipes use explicit descriptors.

6. **`windows_process_access_tests.rs:173`: the restricted-token denial may be vacuous.** Nothing asserts the restricted token can open the unprotected (`direct`) thread, so "denied" may come from the default DACL, not the fix.
   *Fix:* assert the `direct` thread is granted for the restricted probe, or explain why it isn't expected.

7. **Workflow line 55-58: the new step tests nothing.** `-p codex-windows-sandbox --lib pf_27_s07` matches no tests on this branch, so it passes with zero tests.
   *Fix:* remove it from slice 2a, or fail the step when zero tests run.

**Checked and OK**
- **PE parsing:** the offsets are correct for PE32+ (`NumberOfRvaAndSizes` at +108, data directories at +112, descriptor fields at 0/12/16). PE32 images and imports by ordinal or without a name table are safely skipped. Parsing the process's own mapped image without bounds checks is acceptable.
- **Patching:** `VirtualProtect` is restored after each slot, under a lock. Each slot is written with one aligned pointer-sized store.
- **Linker and Control Flow Guard:** CFG is not enabled. Incremental-link thunks are consistent because every use of the wrapper's address goes through the same expression.
- **Delay-load:** no delay-loaded imports are used.
- **Wrapper semantics:** it preserves the caller's inherit flag and passes explicit caller descriptors through unchanged. It makes no API calls before calling the original, so error codes are preserved. Creator handles still get full requested access, so `JoinHandle` works.
- **Broker default DACL:** the pipe instances use an explicit descriptor. The job, events and sockets are used through the creator's handles. The broker starts no children, so nothing inherits the token DACL.
- **Test window:** creating the threads suspended keeps the TLS callback from running. The same-user `GET/SET_CONTEXT` granted on the `CreateRemoteThread` thread is a valid positive control, and the "denied" assertions are exact.
