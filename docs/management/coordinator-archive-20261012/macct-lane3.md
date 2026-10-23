You are the lane owner for PF-84 (named accounts per provider), continuing after two previous lane workers. Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding; note the "Linux box" section: Linux
work runs on glitch, `ssh glitch`, max -j10, at most one heavy build per worker), macct-lane.md (original brief; its rules
and gates still apply), macct-lane.result.md and macct-lane2.result.md (S01-S03 shipped; S04 allocated by #423 to
worktrees/pf84-s04-providers, branch feat/pf-84-s04-provider-accounts-ui, base 0558c52cc3; S03 known limits; the
duplicate-row bug trace), the corbanu-terminal-development skill, and on origin/main the PF-84 plan section and the
S04/S05 records and qa/provider-auth/pf-84/s0{1,2,3}-gate.md.
Another worker (macct-fix1) has PR #422 open fixing #414-#419 on branch fix/pf-84-acceptance-bugs; merge origin/main
often and coordinate by keeping your write scope to S04/S05.
Do: S04 (`/providers` account management: add, rename, remove, switch accounts, in /providers and onboarding; fix the
duplicate provider rows bug; show the active account in the spawn line and in /status as S03 deferred), then S05 (usage and
rate limits recorded per account; /status and /cost views grouped by provider and account). Named ChatGPT sign-ins and AWS
profile accounts are NOT in S04 — they're a pending decision for Travis (proposed PF-84-S06); leave the refusal in place.
Include the S03 follow-up tests the review asked for (level raised mid-session, TUI-started worker inheritance, running-thread
mismatch). Per-slice gate: focused `just test` with and without developer-accounting; Linux clippy -D warnings on glitch
(workspace checkout ~/corbanu-rtx/integration or your own worktree under ~/corbanu-rtx/; delete your target after); one
independent Opus 5.5 High review via installed `corbanu exec` (read-only). Per sprint: tmux run (docs/tmuxHarness.md), GLM
5.3 Flash run (`-m glm-5.3-flash -c model_provider="zai"`) with the real default Z.AI key plus a fake named account (401),
a second provider (Kimi `provider/kimi_api_key` or OpenAI `openai-api-key`), SOP videos (leak-scanned). Keys only via
`X="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper <label>)" <cmd>`;
disposable homes with CORBANU_TEST_NO_NATIVE_KEYRING=1; never print keys. Merge only when ALL checks are green, never with
checks pending (`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f). Set each sprint `ready`
when its gate passes; do NOT archive. Stop and report after S04 is ready if context gets large. Keep context lean. Report
as in sec-common.md, listing anything Travis must decide.
