**VERDICT: CHANGES REQUESTED**

The ACL rebuild looks correct and memory-safe. The rebuilt list is no larger than the original, keeps the original revision and size, and copies each entry whole, so object entries are copied unchanged. Entries are checked for type before they are read as access-denied entries. The exact match removes only an explicit entry with flags OI|IO|NP, the group SID, and a mask equal to `FILE_GENERIC_READ` after generic mapping. That is the entry PF-27-S06 adds, so it cannot remove a looser entry someone else added. The lock rule itself holds: when a flag-off process holds the exclusive lock, an arming process waits. The remaining problems are in how the lock is held.

**Medium**

1. **Arming can hang forever if a sandboxed process holds the lock** (`core/src/security/launch_contract.rs:641`).
   - `CODEX_HOME` is a sandbox read root: `.codex` is not in `USERPROFILE_ROOT_EXCLUSIONS` (`setup.rs:55`).
   - After the flag-off cleanup, the lock file loses its inherited deny, so anyone in CodexSandboxUsers can open it for reading. A read handle is enough to take an exclusive lock with LockFileEx.
   - A lingering sandboxed process started from a flag-off session can therefore keep an exclusive lock. When the user turns the flag on, `arm` waits in `lock_shared()` inside `ACTIVE.get_or_init` during config load, and startup hangs.
   - Fix:
     - Give the lock file its own explicit read/write deny for the group every time it is opened or created. `add_deny_read_ace` on the file works, and the removal code leaves explicit entries on files alone.
     - Replace the blocking wait in `arm` with `try_lock_shared` retried for a bounded time.

2. **If the shared lock fails, the process stays armed with no protection** (`launch_contract.rs:633-647`).
   - If opening or locking fails, the code only logs a warning. A flag-off process can then remove the deny while this process's sandboxed command is running. Files written to `CODEX_HOME` during that window, such as an `auth.json` replaced by rename, are readable by the sandbox.
   - "Re-adds before every launch" does not cover commands that are already running.
   - Fix: record whether the lock is held, and have `protect_new_codex_home_files` return `LaunchDenied` (fail closed) when it is not.

3. **Mixed versions defeat the lock.**
   - An armed PF-27-S06 build holds no lock, so a newly started S07 build with the flag off will remove the deny underneath it.
   - Fix: at minimum, record this as an upgrade limit in the sprint or release notes ("restart all protected sessions before turning the flag off"). Optionally, skip removal while a session marker newer than the lock file exists.

**Low**

4. **The "is it present" check and the removal disagree** (`windows-sandbox-rs/src/acl.rs:378,385` vs `acl.rs:693`).
   - `has_deny_read_ace_for_new_files` accepts flags that merely include OI|IO|NP and any overlapping mask, but removal requires an exact match.
   - An entry that is present but not exact (for example, one SetEntriesInAclW merged with another deny for the same group, or one with the INHERITED flag) means:
     - every start creates or locks the lock file and rereads the DACL, and removes nothing;
     - S06's `add` treats the entry as already present, so it may never write the exact entry.
   - Fix: share one exact check (the new `is_new_file_read_deny`) between `has` and `add`.

5. **Every Windows start pays for a name lookup** (`launch_contract.rs:664`).
   - The flag is off by default, so most users run this, and most never ran setup. For them, `LookupAccountNameW("CodexSandboxUsers")` fails locally and, on a domain-joined machine, falls back to a domain lookup. With an unreachable domain controller, that can take seconds.
   - Fix: first read the `CODEX_HOME` DACL and resolve the SID only if some explicit deny with exactly OI|IO|NP exists. Alternatively, look up the machine-qualified name.

6. **The lock path follows symlinks and can be deleted while held** (`launch_contract.rs:635,690`).
   - Rust's default Windows share mode includes `FILE_SHARE_DELETE`. Someone with delete rights can unlink or rename the held file, and the next remover then locks a different file.
   - A symlink at the lock path would also put a mandatory byte-range lock on its target, such as `config.toml`, for the life of the process.
   - Setup excludes `CODEX_HOME` from sandbox write roots, so only same-user processes or misconfigured setups can do this.
   - Fix: `share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)`, `FILE_FLAG_OPEN_REPARSE_POINT`, and reject anything that is not a regular file.

7. **A partly failed propagation is never retried** (`acl.rs:801-812`).
   - If SetNamedSecurityInfoW updates the directory but fails partway through its children, it returns an error. The next start sees the entry as absent and never retries, so inherited copies stay on some root files.
   - This fails safe (files stay more restricted), but it should be recorded. Optionally, check the root files for leftover copies.
   - Propagation walks the whole `CODEX_HOME` tree synchronously during config load. That is a one-time cost, the same as the add path, and acceptable.

**Tests**

- `acl_tests.rs` measures what it claims for an unprotected DACL with inherited entries (a `%TEMP%` directory). It does not cover:
  - a protected DACL, which is the `PROTECTED_DACL_SECURITY_INFORMATION` branch at `acl.rs:796`;
  - an entry for the same SID and flags with a broader mask, which must be kept;
  - an object-type entry being copied unchanged.
- `launch_contract_windows_tests.rs` tests `release_new_file_deny` against a lock taken in the same process. On Windows that is valid because locks belong to handles. It does not test:
  - that `arm` actually takes the lock (`hold_armed_lock`);
  - that `release_codex_home_when_unarmed` skips while this process is armed;
  - a real second process.

  Add a test that starts a child process holding the shared lock, plus the protected-DACL case.

**Confirmed OK**

- If the group SID cannot be resolved, cleanup returns silently. An orphaned entry would deny nobody.
- The `active()` check plus `Once` correctly stops the process from removing its own deny.
- A process that arms during removal waits, then re-adds the deny before its first launch.
- The CI filter syntax (`-- pf_27_s06 pf_27_s07`) is valid.
