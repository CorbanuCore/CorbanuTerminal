You are an independent, read-only security reviewer. Do not modify any file. Review branch
fix/398-sandbox-cmdlog-redact in this checkout (`git diff origin/main...HEAD`; `gh issue view 398` if available).

Issue #398: the Windows sandbox writes its own daily command log (`CODEX_HOME/.sandbox/sandbox.<date>.log`), outside
codex-log-guard (#387), so a sandboxed command line carrying a credential (`curl -H "Authorization: Bearer ..."`,
`--password=...`, `X-Api-Key: ...`, `?api_key=...`) was written verbatim; `/feedback` attaches that log.

Required end state: every sandbox/exec command log on Windows, Linux and macOS (including any approval/command history
the app persists) passes command text through the same redaction layer before writing; rollout files excepted only if
they are the user's own transcript by design (documented). Plus, if feasible, a Bazel-compatible enforcement that every
tracing subscriber install goes through codex-log-guard, without filesystem source scans.

The change (summary in qa/security-levels/pf24-followups/issue-398/README.md):
- windows-sandbox-rs/src/logging.rs: `log_writer` returns a line-buffering writer that redacts each whole line;
  `preview` redacts before truncating to 200 bytes; `debug_log` stderr redacted.
- log-guard/src/redact.rs: new shapes (credential command-line options, curl -u, env assignments, URL passwords, more
  key formats); `token`, `password`, `secret`, `api_key` and similar names added; a quoted part continues an unquoted
  value only after `=`.
- state/src/log_scrub.rs: one-time in-place masking of existing sandbox logs, called at TUI and app-server start
  (Windows only).
- core/src/exec_policy.rs: no "don't ask again" amendment proposal for credential-bearing commands; a client-sent one
  applies for the session only and is not written to rules/default.rules.
- clippy.toml: disallows global subscriber installation except `codex_log_guard::guard(s).{init,try_init}()`
  (inherent methods in log-guard/src/guard.rs that shadow SubscriberInitExt); tracing-test modules allowed.
- Tests: log-guard unit tests, windows-sandbox logging + real sandboxed-command tests (legacy, elevated, capture),
  state scrub test, exec_policy tests, core/tests/suite/log_redaction.rs sandboxed command test (Seatbelt/Linux).

Check, with evidence (file:line), and say which findings are blocking:
1. Windows sandbox log: is there ANY remaining write path into the sandbox log dir (or other sandbox-owned files:
   setup helper, command runner, elevated runner, setup_error.json, stderr) that bypasses redaction? Is the
   line-buffering writer correct (partial lines, Drop, flush, rotation, multiple lines per write)?
2. Other command logs on Linux/macOS/Windows: is there any persisted log or history of commands (sandbox helpers,
   exec-server, shell-escalation, approvals, analytics, rollout-trace, state DB, hooks, Corbanu-specific stores) not
   covered and not legitimately "the user's own transcript"? Search with rg yourself. Is the documented exception list
   right?
3. Redaction patterns: false negatives for common credential command shapes on PowerShell/cmd/bash (e.g. `-H
   'Authorization: token x'`, `--header=Authorization:Bearer x`, `$env:X="y"`, `set "X=y"`, `export X="a b"`,
   `mysql -pSECRET`, `wget --http-password=`, `git -c http.extraHeader="Authorization: ..."`, AWS keys, Debug/JSON argv
   forms), and harmful false positives in normal tracing logs (token counters, `token_type`, paths, `has_*` flags).
   Any regex performance concern (the patterns run on every log event)? Any span that could cut a UTF-8 char?
4. exec policy: is skipping persistence safe and correct (in-memory policy still updated, no error to the client,
   proposals filtered at every derivation site)? Could a client still persist a credential another way?
5. Enforcement: does clippy's disallowed_methods actually catch `SubscriberInitExt::try_init` via the blanket impl,
   `fmt().init()`, `dispatcher::set_global_default`? Does Bazel clippy use the same clippy.toml? Gaps (e.g.
   `tracing::Dispatch` + `dispatcher::set_default`, `log::set_logger`, `env_logger`)?
6. The old-log scrub: races, permissions on the Windows sandbox dir, marker semantics, cost.
7. Test quality: would the tests fail without the fix? Deterministic?

Output: a verdict (approve / approve with fixes / request changes), then numbered findings with severity
(blocking / should-fix / nit), file:line, and a concrete suggested fix. Be concise.
