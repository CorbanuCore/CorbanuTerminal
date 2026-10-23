**Verdict: REQUEST CHANGES**

The `/` case is fixed in the common layout, but the new ordering removes read-only and deny protection in some policies that are protected on the base. The unit test will also fail once the branch is rebased onto the current `main`. I couldn't reach `gh` (no network access), so I didn't check the issue #503 or PR #511 text. I reviewed #499 from local `origin/main`, where it is **already merged** (`2e2e2f8d82`). This branch is still based on `b7fbc69430`, which is older than that merge.

### Blocking

**B1. The unit test fails after rebasing onto `main` (#499).**
- The test is at `bwrap.rs:2181`. With #499, the link-to-`/` pass picks up every denial, because every path starts with `/`.
- That pass now runs first and masks the denied ancestor `a`. The masking code at `bwrap.rs:1350-1366` sees the link spelling as a writable descendant. It marks `a` `111` instead of `000` and emits `--dir a/b/c/root-link`.
- So the assertion `!args.args.iter().any(|arg| arg == &link)` at `:2229` will fail.
- On this branch alone, `a` is never masked at all. It falls under the writable target `/`, so both the ancestor filter and the rootless filter skip it. #499 is what fixes that.
- Fix: rebase onto `main` and run CI again. Change the test to assert there is no `--bind <link>` and that `a` is masked after the `/` bind.

**B2. The new order regresses protections when two roots share a target or one target sits inside another root.**
- The sort at `bwrap.rs:535-538` breaks ties by the order in the policy. That alone does not cover the case where a link's target is deeper than its spelling and sits under another writable root.
- Each pass masks only paths spelled under its own root (`:640`, and the protocol's `read_only_subpaths`). Any pass that runs later and binds the same real directory covers those masks.

Two examples:
- **Nested target:** writable `/a/b`, a link `/l → /a/b/c/d`, and `read` on `/a/b/c/d/ro`.
  - Before: `/l` ran at depth 1, then `/a/b` at depth 2, which reapplied `ro`.
  - Now: `/a/b` runs at depth 2, then `/l` at depth 4. The `/l` pass binds `/a/b/c/d` writable, so `ro` becomes writable.
  - On this branch's base, a denial at `/a/b/c/d/secret` is also lost. #499 restores the denial but not the read-only rule.
- **Same target:** `/real` listed before a link `/l → /real` (link spelled shallower).
  - Before, the order was `/l` then `/real`.
  - Now they tie, `/l` runs last, and read-only rules spelled under `/real` are lost.

Fix: combine passes that bind the same directory. For each pass, also apply the read-only rules and denials from other roots that fall under its bound directory, the same way #499 does for denials. At minimum, add a second sort key so previously working orders don't regress.

### Should-fix

**S1. #503 is only partly fixed when `:root` is listed first.**
- `:root` gives `/` and the link to `/` both depth 1. With the entry order used in the PR's own integration test (`landlock.rs:1453-1459`), the link pass rebinds `/` after the `:root` pass.
- Read-only rules outside the workspace (e.g. full-write plus `read` on `~/.ssh`) are then lost. On this branch's base, denials outside the workspace are lost too.
- The test only checks protections inside the workspace, which a deeper pass puts back afterwards.
- The B2 fix covers this; ordering symlinked roots before non-symlinked ones at equal depth is a cheap partial fix.

### Answers to your questions
1. **Ordering:** Yes, later binds can cover earlier masks. Ties are resolved by policy order (`sort_by_cached_key` keeps equal elements in order), which is the B2/S1 problem. Different directories at the same depth are disjoint and safe.
2. **Masking root (`:581-588`):** Nothing is weakened. The `--dir` calls only create mount points, which already exist from the ancestor mask's writable-descendant setup or else fail closed. Dropping the useless chain under the link's spelling is correct.
3. **Full-write metadata filtering (`:613`, `:617`):**
   - For a link to `/`, it skips mount points at `/.git`, `/.codex`, `/.agents`, plus any missing explicit read-only path directly under the link's spelling. That is the same as for `/` itself and only applies when `full_write_root` is set.
   - Restricted policies without `:root` still fail closed. Nit: document that missing direct children are dropped.
4. **#499:** The code changes touch separate lines, so the merge should be clean, but I couldn't run a test merge in the read-only sandbox. #499 makes the denial side correct here. It also causes B1 and leaves the read-only side of B2 open.
5. **Tests:**
   - Both new tests fail on the old code: before the fix, the `/` bind came after the workspace bind, and `--dir` was created at the link spelling. They support the main claim.
   - They don't cover ties, a denial or read-only rule outside the workspace, `:root` listed after the link, or a target nested under another root.
   - I couldn't verify the PR description.

### Nits
- The bind target is resolved twice: once in the sort (`:535`) and again in the pass (`:573`). If the link changes in between, the order and the binds disagree. Resolve it once and store the result.