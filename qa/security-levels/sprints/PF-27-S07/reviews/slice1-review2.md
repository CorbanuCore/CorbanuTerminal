**VERDICT: APPROVE**

Fixes 1, 2, 4, 5 and 6 work as described, and fix 2 fails closed on every path I traced. The S06 checks remain the only gate on protected launches. I found nothing that lets the sandbox read a protected file. What remains is low severity: a sandboxed process could at worst block cleanup, which leaves the deny in place, or make arming fail, which refuses protected launches. I could not confirm fixes 3 and 7: the local copy of the S07 record (`0c3bcf1c56`) doesn't contain the "restart protected sessions before turning the flag off" or partial-propagation limits. Check that PR #280 has them.

**Low**

1. **The lock file's own deny is often never added, and a failure to add it is silent** (`core/src/security/launch_contract.rs:711,719-723,794`).
   - `add_deny_read_ace` checks for an existing entry with `dacl_has_read_deny_for_sid`, which also counts inherited entries.
   - When the lock file already carries an inherited copy of the new-file deny, `open_armed_lock` adds nothing. The commit message says an armed contract gives the file its own deny; in this case it doesn't.
   - The file only gets an explicit deny after a flag-off removal (line 794), and that happens *after* propagation has removed the inherited copy. For that short window, CodexSandboxUsers can open the file.
   - There is a second window: an armed contract creates the file before the CODEX_HOME entry exists (first arm, before the first launch), and the deny is added only after `CreateFile`.
   - Handles opened during either window survive later ACL changes. A sandboxed process from another flag-off session that keeps retrying the open can then hold the lock for good.
     - Holding it shared blocks cleanup, which is fail-safe.
     - Holding it exclusively makes every later arm time out, so protected launches are refused. That is a fail-closed denial of service.
   - `add_deny_ace` returns `Ok(false)` when `SetEntriesInAclW` or `SetNamedSecurityInfoW` fails, and `deny_armed_lock` throws that result away with `.map(|_| ())`.
   - Fix:
     - Make sure an explicit entry exists by checking with explicit scope and ignoring `INHERITED_ACE`.
     - In `release_new_file_deny`, call `deny_armed_lock` **before** `remove_deny_read_ace_for_new_files`.
     - Treat `Ok(false)` as an error unless an explicit deny is confirmed present.
     - Ideally, create the file with its security descriptor already set: `CreateFileW` with `SECURITY_ATTRIBUTES`, or a temporary file in CODEX_HOME renamed into place after the deny is applied.

2. **A timed-out lock is never retried, so protected launches stay refused until restart** (`launch_contract.rs:656-680`).
   - The lock is attempted only once, in `capture`.
   - `SetNamedSecurityInfoW` re-propagates down the whole CODEX_HOME tree (sessions, logs, plugins), even though the entry is NP. On a large home that can take longer than 5 s.
   - A process that arms while that is running loses the lock and refuses every protected launch until it restarts.
   - The wait also blocks a Tokio worker with `std::thread::sleep` during config load.
   - Fix: keep the `File` in a `Mutex<Option<File>>` and retry `try_lock_shared` lazily in `protect_new_codex_home_files` before refusing.

3. **Hard links pass the regular-file check** (`launch_contract.rs:703-709`).
   - A hard link at the lock path that points to `auth.json` or `config.toml` counts as a regular file. Corbanu then holds a whole-file shared byte-range lock on that file and adds a deny to it.
   - Only the same user can create one, since the sandbox has no write access to CODEX_HOME. Same trust level, cheap to close.
   - Fix: get `nNumberOfLinks` with `GetFileInformationByHandle` and reject anything other than 1.

4. **Misleading warning** (`launch_contract.rs:794`). If removal succeeded but `deny_armed_lock` fails, the error reaches the caller, which logs "could not be removed". Log the two failures separately.

**Tests**

- `pf_27_s07_keeps_protection_and_wider_denies` correctly checks that the protected DACL is kept (SDDL starts `D:P` and is unchanged) and that the wider same-flags deny is kept. The inheritance bits 0x1|0x4|0x8 are right.
- The links-and-deletion test is valid. Without `FILE_SHARE_DELETE`, deleting the held file fails with a sharing violation. With the open-reparse-point flag, `metadata().is_file()` is false for a symlink. However, it fails rather than skips when the runner cannot create symlinks.
- No test checks the lock file's deny, which is the core of fix 1. Call `open_armed_lock(dir, Some(synthetic_sid))` on a directory that already has the new-file deny, run `release_new_file_deny`, then assert the lock file's SDDL has an explicit deny without the inherited flag (`(D;…;<sid>)` without `ID`). As written today, that test would fail; see Low 1.
- The "blocked while armed" test still runs in one process. That is valid, since Windows locks belong to handles, and the test comment now says so. A child-process test would still be stronger.
- Nothing covers a timed-out lock being refused: hold an exclusive lock, call `capture`, and assert `CodexHomeLockUnavailable`.

**Confirmed OK**

- The armed lock lives in a static, so it is held for the life of the process. Test contracts release it when dropped.
- `protect_new_codex_home_files` refuses before resolving the group SID. The shell path (`sandboxing.rs:497`) and the patch path (`apply_patch.rs:319`, where a refused patch gets no file access) both honour the refusal.
- The pre-check for an exact entry with any SID is a single `GetNamedSecurityInfoW` call, so the name lookup no longer runs on every start. The removal pre-check now uses the same exact match as removal.
- The lock file's sharing modes are compatible between armed processes and the remover. The remover's exclusive `try_lock` still never blocks.
- The `CodexHomeLockUnavailable` refusal has a label in inspection and a message telling the user to restart Corbanu.
