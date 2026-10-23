Both PRs are merged with every check green, and issues #509 and #510 are closed with evidence comments.

- **PR #568 (fixes #510):** Z.AI's retry on HTTP 429 now waits out a `Retry-After` of 5 seconds or less. A longer or unreadable one stops the retries and leaves the wait to the throttle. It was already merged before the crash.
- **PR #569 (fixes #509, plus the key-fingerprint decision):** merged now as `8511356e78`.
  - Named accounts are throttled by a salted fingerprint of the key instead of by name. Accounts from before S05 have no fingerprint and fall back to their name; the docs say so.
  - Active cooldowns and leases carry over from the old name-keyed rows.
  - Compaction, the turn-completion check and memory extraction now go through the throttle, with one test each.

**Gate:**
- **Tests and clippy:** before merging #569 I checked its final head, which includes main's #585. The focused tests pass on glitch with and without developer-accounting, and Linux clippy `-D warnings` is clean.
- **macOS clippy:** clean on the head before that merge. I didn't rerun it afterwards because the Mac is saturated; Linux clippy covered that head.
- **Review:** the Opus 5.5 High review approved every round, including one on how the #585 conflict was resolved.
- **tmux check (mock 429 server):** after one account got the 429, a second account on the same key was blocked locally for about 120 seconds. A third account on a different key still answered.

**Cleanup:** I don't own anything that's uncommitted or unmerged.
- Both worktrees are removed.
- On glitch, the 127 GB build target, the built binaries and both checkouts are gone. Only small log and test-evidence folders remain.

**Two follow-ups left open:**
- A test that the throttle key follows a mid-session account switch (a minor review finding).
- One shared `Retry-After` parser for `codex-client` and `api_bridge`.

The full report is in `.codex-work/workers-20261002/throttle-fp.result.md`.