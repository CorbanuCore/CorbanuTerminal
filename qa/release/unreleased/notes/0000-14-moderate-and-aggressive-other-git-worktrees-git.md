## Moderate and Aggressive: other git worktrees' `.git` files stay read-only

- **Agent commands can no longer rewrite another linked worktree's own `.git`
  file** (#675). The protected-path rules already kept every linked worktree's
  folder in the shared repository read-only (#650). Each worktree's own `.git`
  file, which says where its git folder is, now stays read-only too, so it
  can't be pointed at a planted git folder whose config would run code in your
  later git commands there. Git in the current worktree and in the main
  repository works as before. `git worktree repair` of another worktree now
  fails for agent commands as well, as `git worktree move` and `remove` already
  did. Permissive is unchanged.
- **The same holds for a submodule's linked worktrees** (#1468): the `gitdir`
  in `.git/modules/<module>/worktrees/<name>` and the `.git` file it names stay
  read-only too, and a `gitdir` spelled `.GIT` counts where the volume ignores
  case.
