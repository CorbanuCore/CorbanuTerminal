You are an accounting bug-fix worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
(binding: keychain rule, disk hygiene, per-fix gate, report format) and acct-fix1.result.md in that folder (how #299
made model requests tolerate a clock behind the ledger checkpoint).
Fix GitHub issue #308 (`gh issue view 308`): in developer-accounting builds, `corbanu delete --force <session>` with the
process clock behind the accounting ledger's last checkpoint fails with "negative or backward checkpoint", but only after
it has already removed the session's rollout file(s). Required end state:
- Deleting a session is all-or-nothing from the user's point of view: if accounting can't be updated, the session is not
  half-deleted (do the accounting step first or make the order safe), and the error says what to do in plain words.
- Deletion tolerates a clock behind the checkpoint the same way #299 made model requests tolerate it (consistent policy,
  reuse the same helper; don't invent a second notion of skew).
- Spend from a deleted session is still attributed as "deleted conversations" (#303 behaviour) after the fixed delete.
- Regression tests: delete with the clock 58 s and 6 days behind succeeds and leaves consistent state; an injected
  accounting failure leaves the rollout intact. Use the existing test clock injection rather than libfaketime where
  possible.
Gate: focused tests (codex-state, codex-core / app-server as touched) with and without `developer-accounting`, Linux
clippy on the RTX box (ssh ambient@100.99.88.49) with -D warnings, one independent Opus 5.5 High review via installed
`corbanu exec` (read-only). Another worker (acct-fix2, PR #318) is changing /usage and /cost wording in codex-tui;
avoid those files and merge origin/main often. Work in a new worktree off origin/main under
/Volumes/CorbanuDrive/Corbanu/worktrees/. PR to main, merge when green (`gh pr merge N --merge --match-head-commit <sha>`;
never force-push; never rm -f). Close #308 with evidence. Remove your build target when done. Report as in sec-common.md.
