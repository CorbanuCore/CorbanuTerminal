**Verdict: REQUEST CHANGES.** B1, S1 and the read-only half of B2 are fixed. But the new ordering lets a deny-read rule be silently dropped in a case the old code protected.

### Blocking

**N1. A denial spelled through a link can be dropped when the link is spelled deeper than its target.**

Example policy:
- `T/a/b/c/l -> T/x`, writable
- `T/x/sub`, writable
- `deny T/a/b/c/l/sub/secret`

Denials keep the link spelling. `normalize_effective_absolute_path` keeps the logical path for links at depth 2 or more.

- **Old order** sorted by spelling: `T/x/sub` (depth d+2) ran before `l` (d+4). The `l` pass remapped the denial to `T/x/sub/secret` and masked it last, so it was protected.
- **New order** sorts by target (`bwrap.rs:542`): `l` (target `T/x`, d+1) runs first and masks `T/x/sub/secret`. Then the `T/x/sub` pass binds `T/x/sub` writable over that mask.
  - Its denial list (`:667-680`) only takes paths that start with `T/x/sub` (or with its own target, which it doesn't have). The link-spelled path matches neither, so the mask is not re-applied.
  - The rootless pass (`:755`) and the ancestor pass both skip the path, because it is under the writable `T/a/b/c/l`.
  - Result: `secret` becomes readable.

The same thing happens on a tie: link `T/a/b/l -> T/x` listed before a writable `T/x`. The old order was `T/x` then `l`.

`earlier_read_only_subpaths` (`:650-662`) fixes exactly this for read-only paths. Nothing does the same for denials. #499 only adds denials spelled by **this pass's own** target.

**Fix:** keep the bound (remapped) form of each mask an earlier pass applied, and re-apply the ones strictly below a later pass's bound path, alongside its own denials. Alternatively, include any denial whose remapped form (through any symlinked writable root) falls under the current bound path. Add a unit test for the deep-spelled link with a nested ordinary root, plus the tie case.

### Answers

1. **Round-1 findings and remaining coverage gaps.**
   - **B1 is fixed.** The test now asserts there is no `--bind <link>` and that the ancestor mask comes after the `/` bind. With #499, the ancestor gets `111` plus a harmless `--dir` chain.
   - **B2 is fixed for read-only paths**, both for a target nested inside another root and for two roots with the same target (`:650-656`). It is **not fixed for denials** (N1).
   - **S1 is fixed.** `:root` and a link to `/` both have depth 0. Whichever pass runs second re-applies the first pass's read-only paths and, via #499, every denial.
   - **Nit (B2 leftover):** the bind target is now resolved once (`:532-548`), so the order and the binds agree.
2. **Can the re-application go wrong?**
   - **Writable paths.** A writable root is never made read-only: paths equal to the bound path are excluded. Explicit `read` entries only reach a root's read-only list when the path is not writable anywhere in the policy (`protocol/src/permissions.rs:1076-1077`), so an explicit writable `.git` or a more specific writable root stays writable.
   - **Symlink errors.** No new `first_writable_symlink_component_in_path` errors. Every re-applied path was already checked in its original pass in the same form, and a crossing would have failed there. A path skipped there as "missing below read-only" can't cross a writable symlink either (`:1251`).
   - **Mount targets.** A missing re-applied path gets its placeholder below the current bound path, on the live host folder, unless a read-only parent in the merged list makes it skip. Registering the same placeholder again in a later pass matches the existing #473 design (deduplicated per pass).
   - **Protected create targets.** Unchanged: they are computed before the extension (`:643-649`).
   - **Nit, contradicts the PR's "same argv" claim:** git worktree gitdirs can change argv for ordinary, non-symlinked roots. Take a worktree `/w` whose gitdir is `/p/main/.git/worktrees/w`, plus a writable `/p/main`.
     - The `/p/main` pass now adds a redundant `--ro-bind` for the gitdir.
     - If `/p/main/.git` is explicitly writable, the gitdir now stays read-only regardless of depth order. Before, that depended on which root was shallower.
     - This is stricter, never weaker.
3. **Interaction with #499 and #473.**
   - **#499:** consistent for denials spelled under this pass's root or target. N1 is the gap #499 doesn't cover.
   - **#473:** re-applied read-only paths are added before the snapshot/missing-component checks (`:650-662`, then the denial loop). Denials missing below a re-applied read-only folder correctly get a snapshot instead of a placeholder that bubblewrap couldn't create in a read-only mount.
   - **Denied-ancestor `--dir` check:** switching it to the bound path is correct.
4. **Tests and PR description.**
   - All three unit tests and the bwrap test target their claims and would fail on the old code. The bwrap test does exercise the re-application (`ro=1`).
   - No test covers a denial spelled through a link reaching a deeper or same-target root (N1).
   - The PR body says "ordinary roots keep the same argv"; that's not quite true (the gitdir nit).
   - It also says denials are handled by #499; that's incomplete because of N1.

I couldn't reach `gh pr view 511`, so I checked the description against `/tmp/pr503.md`.