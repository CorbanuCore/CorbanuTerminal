You are a bug-fix worker for PF-84 (named accounts per provider). Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding) and macct-lane.result.md in that folder,
then on origin/main qa/provider-auth/pf-84/independent-acceptance-20261010/README.md and REVIEW.md (once PR #420 merges; or
read them from the PR branch) and issues #414-#419 (`gh issue view N`).
Fix all six, in one or two PRs, highest severity first:
- #414 (medium, security-relevant): a named Claude Plan account silently falls back to the default account when an older
  `corbanu` is on PATH. The invariant is "a missing or failed account is refused, never falls back to another account" —
  make that hold regardless of which helper binary is found on PATH (e.g. resolve the helper relative to the running
  binary, or verify version/capability and refuse with a clear error).
- #415: with `named_accounts` off, the internal Claude OAuth token command must not serve named accounts.
- #416: a 401 on a named account must name that account in the recovery hint, not the default credential.
- #417: TUI onboarding must accept a home whose [provider_accounts] selects an enrolled named account (no demand for a
  default key).
- #418: `corbanu-debug` must honour a caller-set CORBANU_HOME like the release wrapper/launcher (S01).
- #419: the minor polish items.
Another worker (macct-lane2) is implementing PF-84-S03 (account selection per session/exec/spawn) concurrently in its own
worktree; keep your changes scoped to these issues, merge origin/main before pushing, and note any overlap in the PR.
Regression tests for each (fail before, pass after). Gate: focused tests with and without developer-accounting; Linux
clippy -D warnings (the RTX box ssh ambient@100.99.88.49 is OFFLINE — try once; otherwise rely on the PR's Ubuntu clippy
job and say so); one independent Opus 5.5 High read-only review via installed `corbanu exec`; a tmux functional check with
the fresh debug binary in disposable homes (CORBANU_TEST_NO_NATIVE_KEYRING=1) reproducing #414 before/after with a real
Claude plan token (`CLAUDE_CODE_OAUTH_TOKEN="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper claude-plan-test-token)"`)
and a fake named account, plus a GLM 5.3 Flash run (`-m glm-5.3-flash -c model_provider="zai"`, `provider/zai_api_key`)
for #416. Never print keys. New worktree off origin/main under /Volumes/CorbanuDrive/Corbanu/worktrees/. PR to main; merge
only when ALL checks are green, never with checks pending (`gh pr merge N --merge --match-head-commit <sha>`; never
force-push; never rm -f). Close the issues with evidence. Remove build targets when done. Report as in sec-common.md.
