You are the lane owner for PF-84 (named accounts per provider), continuing after previous lane workers. Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding; "Linux box" section: Linux work on
glitch, `ssh glitch`, max -j10), macct-lane.md (original brief; rules and gates still apply), macct-lane2.result.md and
macct-lane3.result.md (S03 and S04 shipped; known limits), the corbanu-terminal-development skill, and on origin/main the
PF-84 plan section (incl. D4/S06), the S05 record and qa/provider-auth/pf-84/s0{3,4}-gate.md.
Do S05: usage and rate limits recorded per account; /status and /cost views grouped by provider and account (PF-60
accounting is archived; follow its ledger conventions — the per-account dimension must not break older ledgers and must
reconcile exactly with provider-reported usage). First a small docs PR to allocate S05's worktree/branch/base/write scope.
Per-slice gate: focused `just test` with and without developer-accounting; Linux clippy -D warnings on glitch; one
independent Opus 5.5 High read-only review via installed `corbanu exec`. Per sprint: tmux run (docs/tmuxHarness.md), GLM 5.3
Flash run (`-m glm-5.3-flash -c model_provider="zai"`) with the real default Z.AI key plus a second named account (a real
second key is not available: use the same real key under a second account name to show usage splits by account, plus a
fake-key account whose 401 records a failure and no spend), a second provider (Kimi `provider/kimi_api_key` or OpenAI
`openai-api-key`), SOP videos (leak-scanned). Keys only via
`X="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper <label>)" <cmd>`;
disposable homes with CORBANU_TEST_NO_NATIVE_KEYRING=1; never print keys. Never `pkill` broadly (e.g. `pkill -f "nextest
run"` kills other workers' tests); kill only PIDs you started. Merge only when ALL checks are green, never with checks
pending (`gh pr merge N --merge --match-head-commit <sha>`; merge origin/main on conflicts; never force-push; never rm -f).
Set S05 `ready` when its gate passes; do NOT archive. Keep context lean. Report as in sec-common.md, listing anything Travis
must decide. Other workers (agg-fix-a/b/c) are fixing Aggressive-level issues concurrently; stay out of their scope.
