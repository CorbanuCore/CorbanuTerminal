You are an independent, read-only code reviewer (do not edit files, do not run builds, tests or the TUI).

Repository worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/fix-473-20261011 (branch fix/473-bwrap-missing-protected, PR #491).
Review `git -C /Volumes/CorbanuDrive/Corbanu/worktrees/fix-473-20261011 diff origin/main...HEAD` (codex-rs/linux-sandbox/src/bwrap.rs and linux-sandbox/tests/suite/landlock.rs). Issue #473: on Linux, under a full-write profile with protected paths (core/src/security/protected_surface/read_denials.rs builds it: `~/.claude` read-only, `~/.claude/.credentials.json` and `~/.config/gh|gcloud|solana` denied with skip-missing), bubblewrap failed every command when `~/.claude/.credentials.json` was missing (cannot create the mask target in the read-only bind) or `~/.config` was missing (two `--ro-bind-data` on the same first-missing component; the second fails with ENOENT).

Fix: (a) a denied path missing below an existing read-only subpath -> the missing component's parent becomes a read-only tmpfs snapshot (binds of its entries at launch via --ro-bind-try, symlinks recreated via --symlink, the missing components excluded) so nothing created after launch (e.g. a credentials file the host writes mid-command) is visible; (b) dedupe synthetic placeholders at the same path; (c) skip a missing read-only subpath whose first missing component's parent is under a read-only subpath.

Hard requirement: the fix must never weaken protection of denied paths that exist, or that could be created later during the command (TOCTOU). Please check with file:line evidence and rank findings (blocking / should-fix / nit):
1. Mount ordering: can the snapshot (applied after all read-only subpaths of the writable root, before nested deny masks) ever re-expose a denied path that was masked earlier, or undo a read-only protection? Deeper read-only subpaths under the snapshot dir, nested snapshots, writable roots nested under the snapshot dir, symlink-remapped roots, the registry ro-bind.
2. Races between arg construction and bwrap run: entries appearing/vanishing; the excluded (hidden) components appearing before read_dir; existing denied children of the snapshot dir (still masked over the child binds?).
3. Dedupe: can skipping a second placeholder at the same path ever drop protection (different kinds: empty file vs empty dir for protected metadata names; read-only vs deny)?
4. The read-only skip (c): is it always safe (parent really bound read-only in this sandbox)?
5. Anything that now fails closed where it should not, or fails open; symlink fail-closed checks preserved; non-UTF-8 names; tests: do they prove the claims and fail without the fix?
Report findings concisely with a verdict APPROVE / REQUEST CHANGES.
