**VERDICT: APPROVE**

All round-1 findings are fixed or accepted with a documented reason. The new ERROR_ACCESS_DENIED fallback in `restrict_current_process_access` is narrow enough to be safe, and I found nothing blocking. I only read the code; I didn't build it or run the tests.

### Round-1 fixes
- **H1 is fixed.** `windows_protected_spawn.rs:371` now rejects `=` only after the first character. A name that is only `=` comes from `vars_os` for an entry like `==x` and is passed through unchanged, which is fine. The unit case for `=C:` is included.
- **L2 is fixed.** `command_line` checks the finished line, so a NUL in the program, which is also `lpApplicationName`, is caught too. There is a unit case.
- **L3 is fixed.** Sorting is ASCII case-insensitive on UTF-16 units, the sort is stable, and later duplicates win (`:362-367`, `:404-417`). Non-ASCII names that differ only in case are still not merged, because Windows compares the full Unicode uppercase table. This is informational only; no Corbanu variable is non-ASCII.
- **L4 is fixed.** `kill_unstarted` (`:215-219`) waits only after a successful kill. If `TerminateProcess` fails on Core's full-access handle, the suspended child is orphaned, and only SYSTEM or an administrator could end it. That is acceptable: it holds no secret and never runs.
- **L5 is mostly covered.** `spawn_protected_suspended` probes after the process DACL, the thread DACL and the token default DACL are all in place, but before the child runs any instruction. That is the window that matters: the token DACL only affects objects the child creates, and it can't create any while suspended.
  - The probe fails on `no_threads`, so the first thread really is checked.
  - The std `CREATE_SUSPENDED` control proves the same-user probe can see a granted right in the suspended state.
- **L1:** accepted risk, documented in the doc comment. I agree it is a denial of service only.

### The new DACL-rewrite fallback (`windows_process_access.rs:119-205`)
**Correct.** The OWNER RIGHTS (`OW`) ACE grants only `RC`, which removes the owner's implicit `WRITE_DAC`. So the process's own pseudo-handle, whose access is computed at creation, can't rewrite its DACL. That matches what the windows-2022 run showed.

The fallback only accepts ERROR_ACCESS_DENIED, and only after it re-reads the DACL and finds it at least as strict as the target:
- A NULL DACL counts as unprotected.
- Callback and object ACEs, and any other unknown ACE type, count as unprotected.
- Deny ACEs are skipped, which is safe because they can only reduce access.
- Inherit-only allow ACEs are counted, which errs on the strict side.
- Generic bits in the mask are caught by `mask & !allowed`.
- The SID offset (header plus mask) is correct.

Core itself is unaffected: it starts with a full self-handle, so `SetSecurityInfo` succeeds there and the fallback never runs. If `GetSecurityInfo` fails, the function returns false and the broker refuses to start, so it fails closed.

The fallback doesn't require the DACL to match the broker's own user SID exactly. That's fine, because the check is a ceiling on access, not an exact match.

### Low

**N1. The hardening-path test doesn't check the process and first thread (`windows_process_access_tests.rs:228-241`).** `pf_27_s07_protected_spawn_then_broker_hardening_succeeds` checks only the `new_*` keys, and only for the same-user probe. The round-2 note says the target "stays fully denied", but the process rights and the original thread aren't asserted here. That is the exact path where `SetSecurityInfo` now fails and the code relies on the DACL set at creation.
- **Fix:** copy the end of `pf_27_s07_protected_spawn_is_never_openable`:
  1. Loop over both `probe_as_same_user` and `probe_with_restricted_token`.
  2. Filter out the `new_*` keys and call `assert_all_denied` on the rest.

**N2. The suspended-window control covers only the same-user probe (`:296-300`).** Nothing shows the restricted-token probe can see a granted right on a suspended target, so its "denied" result there could be caused by the restricted token itself.
- **Fix:** either run the restricted-token probe against the std control too and record what it returns, or reference the existing restricted-probe control at `:179` in a comment.

**N3. Two `unsafe` blocks have no `// SAFETY:` comment.** They are `GetAclInformation` (`windows_process_access.rs:~160`) and `LocalFree` reached from the early-return path. The second is fine as written, since it returns before allocating. This is style only: add the comment to match repository conventions.

### Checked, no issue
- **Error paths:** `spawn_protected` and `spawn_protected_suspended` split cleanly. A failed `ResumeThread` still kills the child. The thread handle returned to the test is dropped after `Target` cleanup, and killing a suspended process is fine.
- **Test cleanup:** `Target::drop` kills and waits on the suspended protected child. If an assertion panics, the child is still terminated.
- **Inheritance and token setup:** unchanged from round 1. The handle list names only the stdout write end, `bInheritHandles=1`, and the token is opened with `TOKEN_ADJUST_DEFAULT | TOKEN_QUERY` through the creator's full handle.
- **Broker behaviour:** `protect_threads` still runs first and skips the first thread, which was created protected, without error. It returns its result after the process-DACL step, as before.
