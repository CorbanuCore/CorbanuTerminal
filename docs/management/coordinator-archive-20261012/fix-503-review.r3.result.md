**Verdict: REQUEST CHANGES.** N1, B1, B2 and S1 are fixed, and ordinary roots now produce the same argv apart from the disclosed gitdir exception. One gap remains: a later link bind can still drop an earlier read-only path. It comes from the round-1 ordering change, and I missed it in rounds 1 and 2.

### Blocking (under the "never weaker for any policy" requirement)

**R3-1. An earlier read-only path that equals a later link's target is no longer re-applied, so it becomes writable where it used to be read-only.**

- **Where:** the re-apply filter skips paths equal to the bound path (`bwrap.rs:655`, `path.as_path() != mount_root`). Passes are now ordered by target depth (`:542-548`).
- **Example policy:**
  - writable `/p/q/w`, which has a `.git` directory, so `/p/q/w/.git` is in its read-only list;
  - writable link `/x/l -> /p/q/w/.git`, kept as a logical link because it sits at depth 2 or more.
- **Old order (by spelling):** `/x/l` (depth 2) ran before `/p/q/w` (depth 3). The `/p/q/w` pass then ro-bound `/p/q/w/.git` over the link's bind, so `.git` was read-only.
- **New order:** `/p/q/w` (depth 3) runs first. The link pass (target depth 4) binds `.git` writable, and the equality exclusion stops the read-only path being re-applied. `.git` ends up writable.
- **Why the protocol doesn't settle it:** the metadata-write checks are lexical (`protocol/src/permissions.rs:780`, `:1786`, `:1875`). The link spelling doesn't count as an explicit `.git` write grant, so `/p/q/w/.git` stays in the read-only list.
- **Before this PR the outcome depended on order.** If the link was spelled deeper than `/p/q/w`, `.git` was already writable. So the old read-only result was partly accidental, but it is still a weakening for this policy.
- **Fix:** only exclude the equal path for ordinary passes:
  `path.starts_with(mount_root) && (symlink_target.is_some() || path.as_path() != mount_root)`
  - For ordinary roots the equal case can't happen anyway: the protocol drops literal explicit entries from the default list (`:1066`), and `read` carve-outs must be non-writable. So ordinary argv stays the same.
  - Add a unit test: the link is spelled shallower than the root and points at the root's `.git`, and the last `--ro-bind <root>/.git` comes after the `--bind <root>/.git`.
  - The other option is a recorded product decision that an explicit link write into `.git` wins, disclosed in the PR body.

### Answers

1. **Earlier findings and remaining coverage gaps**
   - **N1 is fixed** (`:684-696`). Each pass re-masks the denials earlier passes masked at or below its bound path, in the form they were masked (remapped to the target).
     - Including the bound path itself is consistent with existing behavior. Both the root filter and #499's target filter use `starts_with`, which already includes the equal path (`:673-676`).
     - Deeper writable children inside a denial still reopen it. A child is always deeper than the denial, so it runs after any re-mask, and `D.starts_with(child)` is false.
   - **B1, B2 and S1 stay fixed.**
   - **Other masks a later bind could cover:**
     - `--dev`, the registry ro-bind and snapshots are re-derived on every pass.
     - Denied-ancestor masks only cover paths outside every allowed write path, link targets included. A link to `/` sends every denial to that pass's own list.
     - The rootless pass runs last.
     - **The only remaining gap is R3-1.**

2. **Can the re-application go wrong?**
   - **Writable paths:** nothing writable is wrongly made read-only, and the equality exclusion protects the root itself.
   - **Symlink errors:** no new `first_writable_symlink_component_in_path` errors.
     - Every re-applied denial was checked earlier in the same form against the same `allowed_write_paths`, either by `append_unreadable_root_args` or by `missing_component_below_read_only_path`, which returns `None` when a writable symlink is crossed (`:1266`).
     - The same holds for read-only paths.
   - **Snapshot vs. placeholder for missing denials:**
     - If the read-only folder lies strictly inside the later bind, it is re-applied and gets a snapshot again.
     - If it encloses or equals the later bound path, that path is live and writable, so a placeholder is the correct choice.
   - **Placeholders and protected create targets:**
     - Placeholders are deduplicated per pass, which matches #473.
     - Protected create targets are still computed before the extension (`:646-652`).
   - **Ordinary argv:** for ordinary-only policies, every earlier denial under a later root is already in that root's own list, and `sort`/`dedup` before the depth sort keeps the order the same.

3. **Interaction with #499 and #473:** consistent.
   - #499's target-spelled denials and the new re-masks are merged and deduplicated before the snapshot and mask logic.
   - The #473 snapshot and placeholder handling applies to the re-masked entries unchanged.

4. **Tests and PR description**
   - The new unit test covers both N1 cases: a deep-spelled link with a nested root, and the same-directory tie with the link first (also the lexicographic order).
   - The bwrap test still covers `ro=1`, `git=1` and `cat=1`.
   - No test covers R3-1.
   - **Nit:** the PR body's argv claim is now accurate ("almost always", plus the gitdir exception). It should also mention R3-1, or its resolution.
   - **Nit:** `/tmp/pr503.md` still contains `GATE_PLACEHOLDER`.

I reviewed the diff read-only and didn't run the build or tests. I checked the PR description against `/tmp/pr503.md`.