You are a bug-fix worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding; Linux
work on glitch, `ssh glitch`, max -j10) and main-fix1.result.md in that folder, then issues #473 and #477
(`gh issue view N`).
Fix:
- #473 (product bug, security-relevant): on Linux, bubblewrap fails when a protected home path is missing under a
  read-only parent; in practice `~/.claude` without `.credentials.json` makes EVERY sandboxed command fail at Moderate and
  Aggressive. Fix so missing protected paths are handled (skip masking a nonexistent path, or mask the parent safely)
  without ever weakening protection of paths that do exist or could be created later during the session (think about
  TOCTOU: a file created after launch must still be protected — choose the approach that guarantees that, e.g. masking the
  parent dir or a tmpfs over it, and document it). Regression tests that fail before / pass after on Linux (glitch), plus the
  macOS Seatbelt and Windows equivalents if the same pattern exists there (check; fix or document).
- #477 (intermittent exec test since #469): a background plugin-sync `git clone` keeps running after `codex-exec` exits
  (`$CODEX_HOME/.tmp/plugins-clone-*/.git/shallow.lock`). Make exec wait for or cancel that clone on shutdown (bounded),
  and make the test deterministic.
Gate: focused tests; Linux clippy -D warnings on glitch; one independent Opus 5.5 High read-only review via installed
`corbanu exec` per PR; for #473 a functional check on glitch with the fresh debug binary in a disposable home
(CORBANU_TEST_NO_NATIVE_KEYRING=1) at Moderate/Aggressive with a `~/.claude` lacking `.credentials.json` (use a disposable
HOME), running one sandboxed command before/after with GLM 5.3 Flash (`-m glm-5.3-flash -c model_provider="zai"`; key via
`ZAI_API_KEY="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper provider/zai_api_key)"`
on the Mac, piped to glitch over ssh stdin into a file you delete afterwards; never print it). Never pkill broadly. New
worktrees off origin/main under /Volumes/CorbanuDrive/Corbanu/worktrees/. Merge only when ALL checks are green, never with
checks pending (`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f). Close issues with evidence.
Remove build targets when done. Report as in sec-common.md.
