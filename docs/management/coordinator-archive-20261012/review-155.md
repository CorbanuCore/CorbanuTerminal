# Worker brief: independent security review of PR #155's linux-sandbox change

You are an Opus 5.5 High reviewer. Work autonomously; do not ask questions. Review only; do not push code.

## Context
- PR https://github.com/CorbanuCore/CorbanuTerminal/pull/155 (branch `codex/fix-rust-ci-full-20261004`) makes main's
  post-merge full suite pass. Most of it is test/CI hygiene. Two commits change security-sensitive, upstream-owned
  sandbox code: `73d809c176` and `bb6a4894c1` ("a small cleanup process in its own session that, once bubblewrap
  exits, runs the existing cleanup"), fixing a real bug: when unified exec SIGKILLs a sandboxed command,
  `codex-linux-sandbox` leaves empty `.git`, `.codex`, `.agents` directories in `/tmp` and `$TMPDIR`.
- The worker's report is in `/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/postmerge-full.result.md`.

## Do
1. Read those two commits in full plus the surrounding `codex-rs/linux-sandbox` code and its upstream history
   (`git log` on the touched files). Check: can the cleanup process ever delete anything other than the empty
   placeholder dirs the sandbox itself created (symlink races, TOCTOU, non-empty dirs, a user's real `.git`)? Does it
   leak privileges, file descriptors or capabilities, outlive the session indefinitely, or change exit codes/signal
   propagation of the sandboxed command? Does it hold up when the parent is SIGKILLed, when bwrap fails to start, and
   on non-Linux builds?
2. Also skim the rest of the PR (`gh pr diff 155 -R CorbanuCore/CorbanuTerminal`) for anything risky:
   `remote_env: false`, the opt-level profile change, the serialized tests, the cherry-picked dead-code commits.
3. Post your review as a PR comment (`gh pr comment 155 -R CorbanuCore/CorbanuTerminal --body-file ...`) with a
   verdict: APPROVE, APPROVE WITH NITS, or CHANGES REQUIRED, with concrete findings (file:line).

## Never
Never print secrets. Don't push, merge or approve via the GitHub review API.

## Final report (your last message)
Verdict, findings by severity, and the comment URL.
