**VERDICT: APPROVE.** This depends on the pending windows-2022 `windows-security-probes` run passing on this head. I couldn't see that run, so finding 3 is not proven yet.

All seven round-1 findings are fixed. The new batched write step adds one small regression: below Low severity (finding 8).

**Round-1 fixes**

1. **Fixed.** `broker_containment.rs:323-325` now runs both calls every time, and `dacl` requires both to succeed. If only the redirect fails, Core refuses the broker even though the default DACL already protects its threads. That's a cautious choice, and I accept it.
2. **Fixed.** In `windows_process_access.rs`, the snapshot always runs and the error is returned at the end (`:201`). The process DACL is applied before the thread error is returned (`:118-121`). In Core, `launch_contract.rs:664` still reports any error as "not hardened" even when the DACL was applied. That's conservative and fine.
3. **The code is fixed; the proof is still pending.** The reasoning holds: in the WRK and later kernels, the thread's pseudo-handle uses `Thread->GrantedAccess`, which is the access check plus `TERMINATE|SET_INFORMATION|QUERY_INFORMATION`. On Vista and later, those two information rights also imply the matching LIMITED rights, so `SetThreadDescription` should work. The test now checks priority and setting/reading the thread name, then checks that the failure count is 0, in both target modes (`tests:393-432`). Treating `STATUS_ACCESS_DENIED` as "already protected" is reasonable, but see finding 9.
4. **Fixed.** The wrapper falls back to `GetProcAddress`, returns `ERROR_NOT_READY` if that also fails, and never calls back through the import. Calling `GetProcAddress` on kernel32 is fine even if the slot was bound through an API-set DLL, because both resolve to the same function. The Vec allocated on the fallback path is harmless.
5. **Fixed.** The docs now name the other modules, and the loader-worker gap is recorded in the sprint record.
6. **Fixed.** `tests:175-180`: the restricted probe now has a positive control.
7. **Accepted** as shared with #281. Merge #281 first, or the step passes without running any tests.

**New findings**

**Low**

8. **`windows_thread_creation.rs:151-172`: the import table can be left writable.** `VirtualProtect` works on whole pages. If two `CreateThread` import slots share a page, the second call saves the protection as `PAGE_READWRITE`, because the first call just set it. `restore_protection` then works forward: it restores the page to read-only, then sets it back to read-write. The import table page stays writable for the life of the process, which makes later import-table hijacking easier. The old code restored each slot straight after writing it, so it didn't have this problem. The same happens on the failure path (`:157`). It only applies when there are two or more slots, for example if raw-dylib produces several kernel32 descriptors.
   *Fix:* restore in reverse order (`for (slot, previous) in entries.iter().rev()`). Optionally, add a test assertion that the slot's page keeps its original protection after `redirect_thread_creation()`.

**Informational**

9. **`windows_process_access.rs:230`: ignoring access-denied can hide a real gap.** Ignoring `STATUS_ACCESS_DENIED` also hides threads that another module created with its own restrictive DACL that lacks `WRITE_DAC` but still grants `SET_CONTEXT` to the user. Before this change, those counted as failures. The risk is low because other modules almost never do this.
   *Fix (optional):* on access-denied, call `GetSecurityInfo` on the pseudo-handle and compare the result with the protected DACL. `READ_CONTROL` is granted through `OWNER RIGHTS`. Count a failure if they differ.
10. **`windows_thread_creation.rs:147-148`: the saved original depends on `slots[0]`.** After the `!= wrapper` filter, `slots[0]` is always an original target. If an EDR hooked one slot, the saved original becomes the EDR's trampoline, which still behaves correctly. No action needed.

**Checked and OK in the new code**

- The idempotent path: when every slot is already redirected, the call returns `Ok` through `thread_creation_protected()`.
- No slot is written until every slot has been made writable.
- The lock still serializes concurrent callers.
- `result.and(redirected)` reports the snapshot error first. Either way, the caller gets an error.
- `run_target` checks the failure count after threads are created protected, so the access-denied handling is covered by tests.

**Remaining windows (unchanged and documented)**

- In Core, threads started by other modules are exposed until their TLS callback runs.
- In Core, loader worker threads stay unprotected for their whole life.
- Both are gone in the broker because of the default DACL.
