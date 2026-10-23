You are the accounting lane worker for sprint PF-60-S04 (cost-accounting acceptance and handoff), the last sprint of
PF-60. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding: keychain rule, disk
hygiene, per-sprint gate, secrets, SOP videos) first, then acct-s05.result.md, acct-archive2.result.md,
acct-acc5b.result.md and acct-price1.result.md in that folder, the corbanu-terminal-development skill, and on origin/main:
docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s04-cost-accounting-acceptance-and-handoff.md, the plan
docs/plans/active/portfolio-agent-cost-accounting.md, and the archived S01/S02/S03/S05 records (S05 accepted by Travis
with waivers (a) AC10 OpenAI exact estimate until #361 — now fixed by #369 — and (b) AC11 isolation).
Authority: Travis asked to finish accounting sprint by sprint with demo videos; S03 and S05 are archived, so S04's
dependencies are met. You own S04.
1. Small docs PR first: activate S04 per procedure (allocate worktree/branch/40-char base, literal write scope, nonempty
   test commands, owner = coordinator/Travis as receiving owner), and fold in the known gaps that must be closed or
   explicitly decided here: (i) `/side` conversations and memory consolidation are paid but not recorded in the ledger
   (S05 follow-up); (ii) the "Price source" line shown for a request with a basis but no price; (iii) the AC10 waiver
   re-check now that #369 landed; (iv) #368 price follow-ups only as documented limitations unless trivial. Merge when green.
2. Implement fixes for (i) and (ii) in PR slices, with regression tests. Each slice: focused tests with and without
   `developer-accounting`; Linux clippy -D warnings on RTX (ssh ambient@100.99.88.49); one independent Opus 5.5 High
   read-only review via installed `corbanu exec`. Merge when ALL checks are green, never with checks pending
   (`gh pr merge N --merge --match-head-commit <sha>`; merge origin/main on conflicts; never force-push; never rm -f).
3. Qualification on ONE recorded debug binary built from the final main commit: three-provider parent/child journeys
   (Claude plan via CLAUDE_CODE_OAUTH_TOKEN from `claude-plan-test-token`, GLM 5.2 on zai via `provider/zai_api_key`,
   Kimi k3 on kimi-code via `provider/kimi_api_key`, OpenAI gpt-5.4 via `openai-api-key` passed as CODEX_API_KEY unless
   #381 has merged), cancellation, missing price, duplicate retry, restart/resume, historical inspection, `/cost` date
   bounds, reconciliation against golden totals. Keys only as `X="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper <label>)" <cmd>`
   in disposable homes with CORBANU_TEST_NO_NATIVE_KEYRING=1; never print keys; never the real keychain. Drive the TUI
   through the repo's tmux harness (docs/tmuxHarness.md) and run the sprint's literal verification commands. Write
   qa/portfolio/agent-cost-accounting/qualification.md with expected vs actual, counterexamples, limitations, handoff.
4. Short SOP demo videos of each S04 flow (leak-scanned, uploaded to the `demos` release, indexed under qa/demos/index/),
   then a single narrated reel per the earlier accounting reel (.codex-work/cf-acct-reel, acct-reel1.result.md) — build
   it but do not publish; the coordinator will publish after Travis reviews.
Set the sprint to `ready` when the gate passes; do NOT archive (independent code-blind acceptance, then Travis's named
acceptance). Report as in sec-common.md, listing exactly what Travis must decide or accept.
