**Verdict: APPROVE.** R3-1 is fixed, and I found nothing else that blocks. Two notes on the PR description remain: one should be fixed and one is a nit. I reviewed the code read-only and didn't build or run anything.

### Blocking
None.

### Should-fix (PR description only)

**S-1. The before/after table is out of date.**
- It says "all 3 fail" on `origin/main`, but the PR now adds five unit tests (`bwrap.rs` test module).
- `writable_link_to_protected_git_keeps_it_read_only` very likely passes on `origin/main`.
  - Old code ordered roots by how the path is spelled. The link `<tmp>/l` (depth 3) was bound before `<tmp>/p/q/w` (depth 5).
  - The workspace pass then applied `--ro-bind .git` again after the link's bind, so `.git` was already read-only.
  - So this test guards against a regression from the new ordering. It isn't a failure that `main` already had.
- Please list which tests fail on `origin/main` by name, the same way the "re-applying only strictly-below paths" row does.

**S-2. A stricter behaviour change isn't fully disclosed.**
- The body says a writable link to `.git` stays read-only "as when the link was bound first". That undersells the change.
- Now a writable link never reopens a path that another root made read-only (`bwrap.rs:653-662`), however deeply the link is spelled. That covers the link's own target, from this round, and paths below it, from round 1.
- Before this PR, a link spelled deeper than the root did reopen the path. Examples:
  - writable `/w` and writable `/a/b/c/l -> /w/.git`: `.git` used to be writable, now read-only;
  - `:root = write`, `read /opt/ro` and writable `/x/l -> /opt/ro`: `/opt/ro` used to be writable, now read-only.
- This is stricter, never weaker, so it meets the hard requirement. It should still sit next to the gitdir exception.
- I believe macOS Seatbelt still allows writes through such a link: it allows each root's own bound path, with the read-only paths excluded per root, at `sandboxing/src/seatbelt.rs:488-541`. So the two platforms now differ for this edge case. I checked this only briefly; it's worth one sentence in the body.

### Nit
- For a link to a protected `.git`, the link's own metadata targets (`<target>/.git`, `/.codex` and so on) are still recorded as protected create targets. They are computed at `:646` before the extension.
- Their mounts are now skipped, because they sit under the read-only target (`:719-729`).
- This is harmless: they can't be created, and recording them only adds cleanup work. No change needed.

### Answers

**1. B1, B2, S1, N1 and R3-1 are resolved.**
- `:657` now re-applies a read-only path equal to the bound path for link passes, and still excludes it for ordinary passes.
- In the round-3 example (writable `/p/q/w` plus writable link `/x/l -> /p/q/w/.git`):
  - the link pass now re-applies `.git` read-only;
  - it is placed first, since it is the shallowest after the depth sort at `:663`, so deeper protections stack on top of it;
  - it comes after the link's `--bind`, so `.git` stays read-only.
- Denials were already handled for the equal case: the #499 target filter (`starts_with`, which includes the equal path) and the `earlier_unreadable_roots` re-mask both cover it.
- I found no other case where a later bind covers an earlier pass's masks or read-only paths.

**2. The re-application doesn't make anything wrongly read-only or cause new errors.**
- **Ordinary roots:** the condition reduces to the old `path != mount_root` when `symlink_target` is `None`, so ordinary argv is unchanged apart from the disclosed gitdir exception.
- **No path the policy grants writable directly is made read-only.**
  - An earlier read-only path equal to a link target can't also be a direct writable entry: the protocol drops explicit write grants from the default read-only list, and `read` carve-outs are non-writable.
  - The only ambiguity is a write granted through a link spelling, which is S-2.
- **No new errors:**
  - The canonical target contains no symlinks.
  - `append_read_only_subpath_args` already ran on the same path, with the same `allowed_write_paths`, in the earlier pass. So `first_writable_symlink_component_in_path` can't newly fail.
- **Mount handling:**
  - Missing metadata or denials under a target that is now read-only correctly go down the #473 skip/snapshot path (`:705-729`), instead of asking bubblewrap to mkdir inside a read-only mount.
  - An empty `.git` is handled the same way as in the earlier pass: an empty-directory tmpfs and a synthetic target, deduplicated per pass.

**3. Interaction with #499 and #473 is consistent.**
- Target-spelled denials, re-masks and the equal-path read-only re-apply all go through the same sorting, snapshot and placeholder logic as before.
- Synthetic targets are still taken and restored per pass (`:594`, `:762-768`).

**4. Tests and PR description.**
- **The new test is sound** (`writable_link_to_protected_git_keeps_it_read_only`):
  - the `rposition` of the last `--ro-bind .git` must come after the link's `--bind .git`;
  - the non-empty `.git` stops the transient-empty-metadata path from applying.
- **Without the `:657` change:**
  - the last `--ro-bind .git` would be the workspace pass's, before the link's bind, so the test would fail as the table claims;
  - this test doesn't separate the fix from the old spelling order, which is S-1.
- **PR description:**
  - The R3-1 resolution is described, and the review history is accurate.
  - `GATE_COUNTS` and `GATE_REVIEW` are placeholders, filled at merge time as noted.
  - S-1 and S-2 remain.