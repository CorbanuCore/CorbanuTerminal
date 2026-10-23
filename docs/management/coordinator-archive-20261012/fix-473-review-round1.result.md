**Verdict: REQUEST CHANGES.** One blocking problem: the dedupe in fix (b) can leave a denied or read-only path writable when one writable folder sits inside another. The snapshot in (a) and the skip in (c) hold up.

## Blocking

**B1. The dedupe in (b) can turn a protected path writable when writable folders are nested.**
- **The bug:** `has_synthetic_mount_target` (bwrap.rs:1170) checks one list, `bwrap_args.synthetic_mount_targets`, that is shared by every writable-folder pass in the loop at bwrap.rs:563. But each pass starts with `--bind <root>` (bwrap.rs:578), which puts the live host folder back over everything mounted before it.
- **How it happens:**
  1. Take two writable folders, W1 and W2, with W2 inside W1. Nesting like this is supported; see `split_policy_reenables_nested_writable_subpaths_after_read_only_parent`. Take a denied path P inside W2 that does not exist yet.
  2. The W1 pass adds a placeholder at P. While mounting it, bubblewrap creates an empty file at P on the host, through W1's writable bind.
  3. The W2 pass binds the host's W2, which now contains that empty file. The pass then reaches `append_missing_empty_file_bind_data_args` for P, finds P already in the list and skips it (bwrap.rs:1156).
- **Result:** P is an ordinary writable host file inside the sandbox. The command can write it, or replace it with a folder and create the denied tree, and it can see a credential file the host writes during the run. If the command changed P, the cleanup will likely leave it on the host.
- **Before this PR:** the W2 pass mounted a second placeholder over the existing empty file, and that worked. So this is a fail-open regression.
- **Read-only too:** `append_missing_read_only_subpath_args` (bwrap.rs:1141) has the same problem, because a read-only entry inside W2 appears in both W1's and W2's read-only lists.
- **Fix:** only dedupe within one pass. For example, record `synthetic_mount_targets.len()` at the start of each pass and only search entries added after that, or keep a set per pass.
- **Test to add:** nested writable folders with a missing denied path inside the inner one. Assert that there is a `--ro-bind-data` at P after *each* `--bind`. The current dedupe test (bwrap.rs:2965) only covers a single pass.

## Answers to your questions

1. **Mount ordering is OK.**
   - Read-only subpaths deeper inside the snapshot folder get replaced by `--ro-bind-try` binds of the host entries. That is equivalent, because everything below the folder is read-only anyway. Their symlink fail-closed checks still run first, since `append_read_only_subpath_args` is called before the snapshot.
   - Deny masks for existing paths inside the folder come after the snapshot (bwrap.rs:683-688), so they cover the child binds.
   - Nested snapshots are applied shallowest first; a deeper snapshot folder always exists, so it is never one of the hidden entries.
   - A writable folder inside the snapshot folder is bound in its own later pass. That pass re-checks its own denied paths, because `snapshot_covered` is per pass (bwrap.rs:631).
   - Folders behind a symlink are consistent: denied paths are now remapped before classification.
   - The registry `--ro-bind` still comes after the snapshot.
   - Nit: `fs::create_dir_all(&registry_root)` (bwrap.rs:678) runs after the snapshot's `read_dir`. If the registry were ever inside a snapshot folder and missing at that moment, bubblewrap would fail. That is fail-closed and unlikely.
2. **Races are OK.**
   - Hidden names are excluded by exact path (bwrap.rs:1225), so they stay hidden even if they appear before `read_dir`.
   - Entries that vanish are skipped by `--ro-bind-try`. Entries that appear later are not visible.
   - If an existing denied child vanishes before launch, its mask target is missing in the read-only tmpfs and bubblewrap fails. That fails closed, as it did before.
   - A dangling symlink at the denied path still fails closed: `first_writable_symlink_component_in_path` checks the last component too (bwrap.rs:1183+).
3. **Dedupe within one pass is safe.** Read-only placeholders always come before deny masks. An empty read-only file or folder is equally protective for a path that does not exist yet. Across passes it is not safe; see B1.
4. **Skip (c) is safe.**
   - The parent must exist as a folder on the host, so the covering read-only path R also exists.
   - R is then either bind-mounted read-only, mounted as an empty read-only placeholder, or the build fails on a writable symlink.
   - If a writable folder is nested between R and the parent, its own pass re-applies its read-only entries, just as before.
5. **Other points:**
   - Should-fix: there is no test proving an existing denied sibling, such as `~/.claude/secret`, stays masked over the snapshot's child bind. That ordering is the key safety property here.
   - Nit: entries with non-UTF-8 names, or symlinks to non-UTF-8 targets, quietly disappear inside the snapshot folder. This is fail-closed and documented, but it may surprise users.
   - Nit: the snapshot folder's mode becomes 555 with tmpfs ownership, and new top-level entries the host creates during the run are invisible.
   - Nit: one bwrap argument per entry, passed on the command line, could get very long for a big folder. Consider a limit.
   - Tests: the three unit tests and the integration race test (landlock.rs:1265) would each fail without the fix and do prove the stated claims. They miss B1 and the existing-sibling case.