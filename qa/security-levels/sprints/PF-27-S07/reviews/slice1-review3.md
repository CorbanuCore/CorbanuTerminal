**VERDICT: APPROVE**

All four round-2 findings are fixed. I found nothing new that lets the sandbox read a protected file. Every new error path I traced either refuses the protected launch or leaves the CODEX_HOME deny in place. I reviewed the code only; I did not build it or run the tests.

**How each fix checks out**

- **L1 (lock file's own deny): fixed.**
  - `dacl_has_explicit_read_deny_for_sid` (`windows-sandbox-rs/src/acl.rs:357-389`) skips entries marked `INHERITED_ACE` or `INHERIT_ONLY_ACE`. It also requires a deny-type entry, an equal SID and the full `FILE_GENERIC_READ` mask.
  - `ReadExplicit` sets inheritance to 0, so the new entry applies only to the lock file itself.
  - `ensure_explicit_deny_read_ace` reads the DACL again after writing it. A failed `SetEntriesInAclW` or `SetNamedSecurityInfoW` therefore comes back as `Ok(false)`, and `deny_armed_lock` turns that into an error (`launch_contract.rs:755-766`).
  - In `release_new_file_deny` (`:833-837`) the lock file gets its own deny before the CODEX_HOME entry is removed. If that step fails, removal is aborted and the deny stays.
  - When the remover creates the lock file, the entry is still present, so the new file inherits the deny. That path has no window.
  - The only window left is when an armed process creates the file before the group exists or before its deny is added. That matches the recorded limit.
  - The ACL calls use the file's path rather than its handle. The path cannot be swapped while they run, because the file is held open without `FILE_SHARE_DELETE` and was checked not to be a reparse point.
- **L2 (lock never retried): fixed.**
  - `ArmedLock` sits behind a `Mutex`, and `ensure_locked` retries for up to 5 s on each protected launch.
  - A poisoned mutex or an I/O error refuses the launch, which is fail-closed.
  - The lock check runs before the group is resolved, so a contract that never got the lock is refused even on hosts where the group is missing.
- **L3 (hard links): fixed.**
  - The link count is read from the open handle (`file_link_count`), so the check cannot be raced. Anything other than exactly one link is rejected before any lock or deny is applied.
  - Someone adding a second link to the real lock file afterwards changes nothing that matters.
- **L4 (error texts): fixed.** "Protecting the lock file" and "removing the entry" now have separate messages, and the warning no longer says the entry "could not be removed".

**Low findings**

1. **A protected launch can block a Tokio worker for up to 5 s** (`launch_contract.rs:695-711`, called from `tools/sandboxing.rs:498` and `apply_patch.rs:319`).
   - `ensure_locked` uses `std::thread::sleep` while holding the `Mutex`, so concurrent launches queue behind it.
   - If something holds the lock exclusively for a long time (the fail-closed denial of service from round 2), every protected launch now waits 5 s before it is refused.
   - Fix: once the lock is missing, do one non-blocking attempt per launch and back off between attempts (for example, record the last failed attempt time). Or run the wait in `spawn_blocking`/`block_in_place`.

2. **The lock file keeps an extra explicit entry permanently** (`launch_contract.rs:833`).
   - After removal, `.secretless-launch.lock` keeps its own read deny for CodexSandboxUsers.
   - That is harmless, but it means "every other entry stays" is not quite true for the tree. Each protected launch also now costs two more `GetNamedSecurityInfoW` calls (cheap).
   - Fix: note this in the S07 limits and in the user docs. No code change needed.

3. **A pre-existing hard link at the lock path refuses protected launches until it is deleted.** Only the same user can create one, and the result is fail-closed. Consider naming the hard-link cause in the error rather than the generic "is not a regular file".

**Tests**

- `pf_27_s07_lock_file_gets_its_own_deny` measures what it claims:
  - It first asserts that the inherited copy alone does not count, then that the explicit entry survives removal.
  - `SANDBOX_GROUP` is a synthetic domain SID, so the SDDL shows the raw SID and the `fields[5]` comparison is valid.
  - The `ID` substring can only come from the inherited flag, because no other flag combination produces it.
- **Gap:** that test calls the `windows-sandbox-rs` helpers directly. Nothing tests the order inside core's `release_new_file_deny` (lock-file deny before removal). `release_new_file_deny` takes a raw SID, so a core test could:
  - add the new-file deny for a synthetic SID;
  - create the lock file with `open_armed_lock(dir, None)` and drop it;
  - call `release_new_file_deny(dir, synthetic)`;
  - assert the lock file has an explicit, non-inherited deny.
- `pf_27_s07_contract_without_the_lock_refuses_protected_launches` is valid:
  - Windows byte-range locks belong to handles, so a same-process exclusive lock blocks the contract's shared lock.
  - The second assertion only checks that the refusal is not the lock refusal, so it also passes on CI where the group is missing (`WindowsSandboxNotSetUp`).
  - The `armed_lock: true` check confirms the retry took the lock.
  - It costs 5 s of wall time, which is acceptable.
- The hard-link test is correct. Like the symlink test, it fails rather than skips on runners that cannot create links.

**Confirmed unchanged**

- The removal check still uses the exact-match test, and the cheap no-SID check still runs before the group lookup.
- `open_armed_lock` rejects a bad file before the lock is taken.
- Both callers still refuse on any `LaunchDenied`.
