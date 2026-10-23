You are the lane owner for PF-84 (named accounts per provider: several API keys and subscription logins per provider,
for ANY provider, selectable per session, per `corbanu exec`, and per spawned worker/tmux worker). Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding), multiacct1.result.md,
multiacct1.log (full investigation; file:line refs at 63ea3d0cbd) and multiacct-plan1.result.md in that folder, the
corbanu-terminal-development skill, and on origin/main docs/plans/proposed/unified-provider-auth.md (PF-84 section) and
docs/sprints/current/unified-provider-auth/pf-84-s0{1..5}-*.md.
Travis, 2026-10-10: "Yes, let's hit multi-account." Earlier requirements: provider-generic (not Anthropic-only); alternate
API keys, not just different plans; workers must be able to run on a different account than the coordinator. Decisions
D1-D3 were not explicitly answered; use the plan's recommendations (D1 one vault per home with account-specific labels;
D2 no automatic import from extra homes; D3 spawned agents may only name already-configured accounts, shown at spawn,
approval under Aggressive) and record them as "recommended defaults, Travis may revisit" in the plan. Reconcile with
PF-76-S01 (provider sign-ins per named profile) as the plan notes.
1. Activation: the WIP limit is full until the PF-60 plan completes (worker acct-archive3 is completing it now). Poll
   origin/main every 10 min (up to 4 h) until docs/plans/active/ no longer contains portfolio-agent-cost-accounting.md,
   then activate the PF-84 plan per procedure (move/approve, allocate worktree/branch/40-char base, literal write
   scopes and nonempty test commands for S01 and S02) in a docs PR. Meanwhile you may prepare in your own worktree.
   Merge when green.
2. Implement in order, S01 and S02 can overlap if write scopes don't, then S03 -> S04 -> S05, each in PR slices. Per
   slice gate (sec-common decision 5): focused `just test` with and without developer-accounting; Linux clippy
   -D warnings on RTX (ssh ambient@100.99.88.49); one independent Opus 5.5 High read-only review via installed
   `corbanu exec`. Per sprint: a tmux run following docs/tmuxHarness.md and a GLM run on GLM 5.3 Flash
   (`-m glm-5.3-flash -c model_provider="zai"`; owner policy 2026-10-10, not glm-5.2) with the real default Z.AI key
   (`ZAI_API_KEY="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper provider/zai_api_key)"`)
   plus a named account holding a fake key (its 401 proves the right account was used); also prove the feature on a
   second provider (OpenAI `openai-api-key`, Kimi `provider/kimi_api_key`, Claude plan `claude-plan-test-token`), and
   short SOP demo videos (qa/demos/README.md, leak-scanned). Disposable homes with CORBANU_TEST_NO_NATIVE_KEYRING=1;
   never the real keychain; never print keys; no new keychain items per account.
   Merge only when ALL checks are green, never with checks pending (`gh pr merge N --merge --match-head-commit <sha>`;
   merge origin/main on conflicts; never force-push; never rm -f).
3. Set each sprint to `ready` when its gate passes; do NOT archive (independent code-blind acceptance, then Travis).
   Stop and report after S03 is ready if context is getting large, so a fresh worker can continue S04/S05.
Keep your context lean (don't dump big logs). Remove build targets when done. Report as in sec-common.md, listing
anything Travis must decide.
