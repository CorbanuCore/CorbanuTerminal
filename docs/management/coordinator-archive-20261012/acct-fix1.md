You are an accounting-lane fix worker. Read sec-common.md (binding: disposable homes, CORBANU_TEST_NO_NATIVE_KEYRING=1,
never the real keychain) and acct-audit1.result.md in /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/, and the
independent acceptance evidence in PR #292 (qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/).
Fix these GitHub issues in CorbanuCore/CorbanuTerminal, one small PR each, with regression tests, Opus 5.5 High review,
Linux clippy on the RTX box, merged when green (`gh pr merge N --merge --match-head-commit <sha>`; merge origin/main on
conflicts, never force-push):
1. #287 HIGHEST PRIORITY: when the clock is behind the ledger checkpoint, every model request fails with "Native Anthropic
   accounting failed" (this is very likely the error Travis kept hitting). Accounting must never block a model request;
   handle clock skew safely and fail open for the request while recording an honest accounting gap.
2. #286: deleting a session must not make /cost claim "no other requests" that day; show the deleted spend honestly.
3. #289: /cost wording inconsistencies listed in the issue.
Do NOT touch #288 or the /cost label work in PR #291 (worker acct-s03a owns it) beyond rebasing. After your fixes merge,
re-run ONLY the failed acceptance criteria code-blind the same way #292 did (real GLM 5.2, macOS + RTX box) and add the
results next to #292's evidence in a docs PR. Report per issue.
