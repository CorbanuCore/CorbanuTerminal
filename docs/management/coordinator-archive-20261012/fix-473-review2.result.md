**Verdict: APPROVE.** B1 is fixed, and I found no new blocking or should-fix problems. This was a read-only review: I didn't run any builds, tests or the TUI.

## B1 is fixed
- **Each pass starts with an empty list.** At the top of every writable-folder pass, the list of placeholders is set aside (`bwrap.rs:566-570`), before the pass's `--bind` at `:579-582`. Both placeholder helpers check for duplicates through `has_synthetic_mount_target` (`:1153`, `:1168`, `:1183`). So they now only see placeholders made in the current pass.
- **The nested case now gets its mask.** For W1 ⊃ W2 with a missing denied path P inside W2, the W2 pass no longer finds P in its list. It adds a new `--ro-bind-data` after `--bind W2`.
- **Read-only paths are fixed too.** The read-only case from round 1 (`append_missing_read_only_subpath_args`) uses the same check, so it gets the same fix.
- **Duplicates within one pass are still merged.** The original #473 case (`~/.config/gh` and `~/.config/gcloud` sharing one placeholder) relies on that, so it still works.
- **The new test would fail without the fix.** `missing_denied_path_is_masked_again_after_a_nested_writable_bind` (`bwrap.rs:3030-3060`) requires exactly two masks, one on each side of the inner `--bind`. Before the fix there would be only one.
- **The existing-sibling gap from round 1 is covered.** The unit test (`bwrap.rs:2966-2977`) checks that the `--perms 000 --ro-bind-data` mask for `secret.json` comes after the folder snapshot's `--remount-ro`. The real-bubblewrap test checks `secret=1` (`landlock.rs:1332`) and that the secret's contents don't appear in the output.

## Setting aside and merging back the list
- **Nothing is lost or reordered.** At the end of a pass (`bwrap.rs:695-701`), the earlier placeholders go back first, then this pass's are appended. That is exactly the order in which they were created. The loop has no outer `continue` or `break`; every `continue` is in an inner loop (`:654`, `:688`). So every pass that finishes reaches the merge.
- **Early `?` returns are harmless.** Several calls can fail partway through a pass (`:656`, `:661`, `:676`, `:689`); at that point the earlier placeholders are held in a local variable. But the error leaves `create_filesystem_args`, and `create_bwrap_flags` passes it on (`:310-317`). So the partly built arguments are dropped and never registered. Bubblewrap never runs, so no host files exist yet that would need cleanup.
- **Repeated paths are handled safely.** The same missing path can now appear once per nested pass, which is what `origin/main` did before this PR added the duplicate check (`main` has no `has_synthetic_mount_target`). With duplicates in `linux_run_main`:
  - `register_synthetic_mount_targets` (`linux_run_main.rs:875`) writes the same per-process marker file twice, which does no harm.
  - Both entries are `missing`, so the logic that converts an existing placeholder to a synthetic one never runs.
  - `cleanup_synthetic_mount_targets` (`:1026`) treats the second marker removal's `NotFound` as success.
  - `remove_synthetic_mount_target` returns early on `NotFound` (`:1184`), and `remove_dir` also allows `NotFound`.
- **Mounting over the leftover file works.** The second `--ro-bind-data` is mounted over the empty file the first pass left on the host. That is the same arrangement `main` already ran.

## Nits (not blocking)
1. **No real-bubblewrap test for nested folders.** B1 is proven by the unit test only. A small `landlock.rs` case would show the command really can't write P and that cleanup removes the file. Optional, since the mount order already matches `main`.
2. **The read-only variant isn't tested.** Nothing exercises a missing protected metadata path such as `W2/.codex` across nested passes. The code path is shared with the deny case, so the risk is low.
3. **Possible older issue, not caused by this PR, not verified.** Take a denied path `W2/.codex/x` inside both W1 and W2. The W1 pass may leave an empty *file* at `W2/.codex`. The W2 pass then mounts a read-only *folder* placeholder (`--tmpfs`) there. Bubblewrap would probably refuse to start, which fails safe. `main` behaves the same way, so it belongs in a follow-up if anyone can reproduce it.

Round-1 nits still open, none blocking: entries with non-UTF-8 names are dropped from the read-only snapshot folder, and very large folders could make the bubblewrap command line very long.