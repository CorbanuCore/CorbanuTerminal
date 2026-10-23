You are a bug-fix worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding; Linux
work on glitch, `ssh glitch`, max -j10) and bugs-473.result.md in that folder, then issues #493 and #494
(`gh issue view N`). One PR per issue, off origin/main, new worktrees under /Volumes/CorbanuDrive/Corbanu/worktrees/.

Fix:
- #493 (Linux sandbox, security-relevant): under a full-write profile with protected paths (Moderate/Aggressive with
  danger-full-access), the `--bind / /` covers `--dev /dev` and is mounted nodev, so every write to /dev/null fails.
  Restore a working minimal /dev after the root bind (and before any explicit /dev writable binds) without exposing the
  host's /dev more than today and without weakening any denial. Regression test (real bwrap on glitch) that writes to
  /dev/null under a full-write profile with a deny entry, failing before / passing after, and a test that the denied path is
  still unreadable. Check Seatbelt/Windows for an equivalent and note the evidence.
- #494 (exec): early `?` returns after the in-process app-server client starts skip `client.shutdown()`, so the plugin
  sync `git` can outlive exec on error exits. Make shutdown always run once the client exists, and add an exec test where
  turn/start fails, checking the fake git from `suite::plugin_sync_shutdown` does not outlive exec.
- Also close the gap from #492's review: a real TUI quit check in tmux (fresh debug binary, disposable home,
  CORBANU_TEST_NO_NATIVE_KEYRING=1) while a plugin sync is in flight: quitting finishes within ~3 s and leaves no git
  process. Record timing in the PR.

Gate: focused tests; Linux clippy -D warnings on glitch; macOS clippy; one independent Opus 5.5 High read-only review per
PR via installed `corbanu exec` (iterate to APPROVE); for #493 a functional check on glitch with the fresh debug binary in a
disposable home at Moderate and Aggressive running `echo hi > /dev/null; echo rc=$?` and a `cat` of a protected file
before/after, using GLM 5.3 Flash (`-m glm-5.3-flash -c model_provider="zai"`; key via
`ZAI_API_KEY="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper provider/zai_api_key)"`
on the Mac, piped to glitch over ssh stdin into a file you delete afterwards; never print it).
Merge only when ALL checks are green, never with checks pending (`gh pr merge N --merge --match-head-commit <sha>`;
never force-push; never rm -f; never pkill broadly). Close issues with evidence. Remove build targets when done. Report as
in sec-common.md.
