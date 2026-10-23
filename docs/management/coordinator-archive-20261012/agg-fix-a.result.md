All four issues are fixed and merged, and #434, #440, #441 and #442 are closed with evidence. One test on main still fails, though: it belongs to #443's worker, and is listed under "Open" below.

## Shipped
- **#454 → #434 (vault value shown verbatim).** Before the fix, both cases leaked: a value added mid-session and a value stored before a restart.
  - Stored vault credentials are now read and registered with the output gate at the start of every task, before it produces output.
  - Values added through `/vault` in the same session are protected at once.
  - Values added from another terminal are protected from the next turn, and the inspector says so.
  - Custody material (seed phrases, private keys, keystores) and the non-secret parts of provider accounts are not loaded.
- **#458 → #441 and #440.**
  - **#441:** under Aggressive with nested agents set to refuse, `corbanu exec`, `e` and `review` are now refused before any approval prompt, with the reason. The rule matches exactly, so commit messages that merely mention it aren't refused. The launch-time check is unchanged and still catches forms the rule can't see, like `env corbanu exec`.
  - **#440:** behaviour is unchanged (the agent environment is still allowlisted); the `/security` review and inspector now say so.
- **#464 → #442, as you decided.** Protected reads are never granted under Aggressive.
  - "Grant once" now lifts only write protections (start-up files, git hooks and config, the project's `.codex` and `.agents`).
  - No grant is offered for a command that names a protected path, and the prompt says why.
  - A denied read like `cat .env` gets a plain warning naming Aggressive, saying it can never be granted, and pointing to Permissive in `/security`. The review and inspector say the same.
  - I removed the old "protected read offers Grant once" work and its demo. I also corrected #439's notice and model instructions, which still said protected files open with "Grant once".
- **#470:** follow-up fixing one core test that still expected the old grant wording. It failed on main after #464 merged, because PR CI doesn't run that test.

## Gate evidence
- **Regression tests:** they reproduce the milestone observations and fail before each fix.
- **`just test`:** green with and without developer-accounting. The one exception is `security_confirm_level_file_edited_by_an_agent_is_caught`, which fails locally on every branch; I believe that's a line-wrap issue from my long temp path, not these changes.
- **Linux clippy `-D warnings` on glitch:** clean for all three PRs, both feature sets.
- **Review:** an independent Opus 5.5 High read-only review ran two rounds per PR. Every round requested changes, and all major findings were fixed. Round two found no remaining way for a grant or approval to reach a protected read.
- **CI:** all green before each merge, merged with `--match-head-commit`.
- **tmux functional checks:** GLM 5.3 Flash, disposable home with `CORBANU_TEST_NO_NATIVE_KEYRING=1`, before (main b78c0111) and after. Before-run frames are in `.codex-work/targets/aggp1-evidence`.
- **Videos (after):**
  - #434: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-434-issue434-vault-value-gated-7400022d9978-2026-10-11.mp4
  - #441: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-441-issue441-nested-refused-before-prompt-abf6c06251ca-2026-10-11.mp4
  - #440: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-440-issue440-review-says-allowlist-abf6c06251ca-2026-10-11.mp4
  - #442: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-442-issue442-protected-read-explained-880e5030fc82-2026-10-11.mp4
- **Cleanup:** all worktrees, build targets, binaries and glitch target directories are removed. Small logs and evidence are kept.
- The Anthropic content filter did not interrupt any of this.

## Open
- **Failing test on main, owned by #443's worker:** `suite::provenance::pf_30_s03_vault_command_after_untrusted_content_needs_the_human`. After #453 merged, the vault command under Aggressive is refused before the prompt this test expects. That looks like intended #443 behaviour, so the test needs updating. I commented on #453 rather than change it.
- **Superseded demo specs:** the three PF-25 grant demo specs (`pf25s01-grant-*.toml`) now show behaviour #442 replaced; I marked them as superseded.

## For you to decide
1. **#434:** under Aggressive with the output gate on, Core now decrypts the vault itself at a session's first task and whenever the vault changes. This is the "cover both cases" option from the brief. The alternative is to drop that read: values stored before a session would then only be protected once revealed, and the inspector would have to say so. I'd keep it as is.
2. **#440:** I kept the allowlist and fixed the wording. Letting every variable that isn't secret-like pass would weaken the accepted PF-27-S02 design. I'd keep it.
3. **#442:** a `.env` is only protected if the preflight found it at startup; one created after an Aggressive session starts is readable. This predates my change, and the UI now says ".env files found at start". Covering new `.env` files would need a sandbox change.
4. **#442:** "Grant once" can still lift write protection on the project's `.codex` and `.agents`, and a file written there could run code later, outside the sandbox. The grant review now names both and warns about that; making them always read-only would be stricter. My recommendation is to make them always read-only.