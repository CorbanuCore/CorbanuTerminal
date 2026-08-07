# Checkout Policy

## One canonical checkout

Keep exactly one canonical PfTerminal checkout at the standard workspace path.
It stays on `main`, tracks `origin/main`, and is not used for feature work.
After updating it, these commands must report `main`, the same local and remote
commit, and no status entries:

```bash
git fetch origin --prune
git switch main
git pull --ff-only origin main
git status --short --branch
test "$(git rev-parse HEAD)" = "$(git rev-parse origin/main)"
```

Do not create long-lived sibling clones. They drift independently, duplicate
large build trees, hide stashes, and make it unclear which checkout is
canonical.

## Short-lived worktrees

Create every task on a named branch in a short-lived worktree:

```bash
git fetch origin --prune
git switch main
git pull --ff-only origin main
git switch -c <type>/<short-description>
git push -u origin HEAD
git worktree add ../worktrees/<short-description> <type>/<short-description>
```

If the branch does not exist yet, the shorter equivalent is:

```bash
git worktree add -b <type>/<short-description> \
  ../worktrees/<short-description> origin/main
```

Use conventional short-lived prefixes such as `fix/`, `feat/`, `docs/`,
`qa/`, `wip/`, and `release/`. Never check the same branch out in two
worktrees.

## Daily preservation

Before ending work each day:

1. Review `git status --short`.
2. Remove only known generated artifacts.
3. Scan the intended diff for credentials and private material.
4. Commit coherent WIP, even if it is not ready for review.
5. Push the branch and verify it with `git ls-remote --heads origin`.
6. Open or update a draft PR when review context would otherwise be lost.

A local stash is temporary transport, not durable storage. Do not leave the
only copy of work in a stash or an untracked file.

## Quarantine before cleanup

When provenance or intent is uncertain, preserve bytes before judging them.
Use:

```text
quarantine/<what>-YYYYMMDD
```

Commit the complete tracked and untracked worktree to that branch, scan it for
secret material, push it, and verify the exact commit with `git ls-remote`.
Only then may the local worktree be reset, cleaned, or removed. Never
force-push a quarantine branch, `main`, or `release/*`.

Ignored compiler caches and generated dependency directories may be discarded
after a dry-run of `git clean -fdxn`; they must never contain the only copy of
source or evidence.

## Removing a worktree

A worktree may be removed only after its branch is pushed or proven merged:

```bash
git status --short
git ls-remote --heads origin refs/heads/<branch>
git merge-base --is-ancestor <branch> origin/main  # when claiming merged
git worktree remove ../worktrees/<short-description>
git worktree prune
```

Keep unmerged PR worktrees only while active review requires them. After merge,
remove the worktree and delete the local merged branch with `git branch -d`.
Quarantine refs are retained.
