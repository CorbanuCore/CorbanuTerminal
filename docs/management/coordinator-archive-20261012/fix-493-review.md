You are an independent, read-only code reviewer (do not edit files, do not run builds, tests or the TUI).

Repository worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/fix-493-20261011 (branch fix/493-bwrap-dev-null, PR #496).
Review `git -C /Volumes/CorbanuDrive/Corbanu/worktrees/fix-493-20261011 diff origin/main...HEAD` (codex-rs/linux-sandbox/src/bwrap.rs, linux-sandbox/tests/suite/landlock.rs, sandboxing/src/seatbelt_tests.rs). Read `gh issue view 493` context: under a full-write profile with protected paths (Moderate/Aggressive with danger-full-access => `:root = write` plus deny/read entries, see core/src/security/protected_surface/read_denials.rs), bwrap gets `--ro-bind / / --dev /dev ... --bind / /`; the `/` bind covers the minimal /dev with the host /dev mounted nodev, so /dev/null writes fail.

Fix: in create_filesystem_args, right after `--bind <mount_root> <mount_root>` for a writable root that is a strict ancestor of /dev (i.e. `/`), push `--dev /dev`, before that pass's read-only/deny masks and before deeper writable roots (sorted by depth), e.g. /dev/shm.

Hard requirements: no denial may be weakened, and the host /dev must not be exposed more than before. Check with file:line evidence and rank findings (blocking / should-fix / nit):
1. Ordering: can the fresh /dev hide or undo any read-only or deny mask (ones applied before the `/` bind: unreadable ancestors of writable roots; masks inside the pass; rootless masks after; the synthetic-mount registry ro-bind; symlink-remapped roots)? Any denied/protected path under /dev in real profiles?
2. Writable roots under /dev (/dev, /dev/shm, /dev/pts...), roots that are symlinks to `/`, restricted-read (tmpfs /) policies with a writable `/`, and the no-denial `create_bwrap_flags_full_filesystem` path (intentionally unchanged; agree?).
3. Behavior changes worth flagging (private /dev/shm and devpts vs host ones; GPU/device access) and whether anything fails closed or open unexpectedly.
4. Tests: do they prove the claims and fail without the fix; the Seatbelt/Windows "no equivalent" claims in the PR body (`gh pr view 496`).
Report findings concisely with a verdict APPROVE / REQUEST CHANGES.
