You are a bug-fix worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding; Linux
work on glitch, `ssh glitch`, max -j10) and agg-fix-b.result.md in that folder, then issues #466, #467, #468 and #446
(`gh issue view N`).
Fix each in its own small PR, highest impact first:
- #467: a workspace opened through a symlinked path (e.g. /tmp on macOS -> /private/tmp) reports `.env` as not isolated;
  the same mismatch makes some security unit tests fail when TMPDIR is /tmp. Canonicalize consistently (security-relevant:
  make sure the fix never loosens isolation — compare canonical paths on both sides).
- #466: a sandbox-denied command can stay shown as "Running".
- #468: in /spawn, choosing Max reasoning for a Troll saves the main model instead of spawning.
- #446: flaky CI test sec_win_323_a_linked_or_foreign_owned_registry_is_refused in the pf_27_s06 Windows probe
  (medium-integrity run: SetNamedSecurityInfoW fails setting the owner of a temp dir). Make it robust without weakening what
  it proves; you can't run Windows locally (the Windows box is not available) — reason from the API docs and CI logs, and
  verify via the PR's windows-security-probes job (rerun it a few times to show it's stable).
Regression tests for each. Gate: focused tests; Linux clippy -D warnings on glitch; one independent Opus 5.5 High read-only
review via installed `corbanu exec` per PR; tmux functional checks for #466/#468 with GLM 5.3 Flash
(`-m glm-5.3-flash -c model_provider="zai"`,
`ZAI_API_KEY="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper provider/zai_api_key)"`)
in disposable homes (CORBANU_TEST_NO_NATIVE_KEYRING=1). Never pkill broadly. New worktrees off origin/main under
/Volumes/CorbanuDrive/Corbanu/worktrees/. Merge only when ALL checks are green, never with checks pending
(`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f). Close issues with evidence. Remove build
targets when done. Report as in sec-common.md.
