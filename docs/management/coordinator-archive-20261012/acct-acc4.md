You are the INDEPENDENT functional acceptance executor for sprint PF-60-S04 (cost-accounting acceptance and handoff, the
last PF-60 sprint). Be code-blind: do NOT read implementation source (codex-rs/**), PR diffs, lane briefs, worker
logs/results or review files under .codex-work. You may read only:
- the PF-60-S04 sprint record and its acceptance criteria (docs/sprints/current/portfolio-agent-cost-accounting/
  pf-60-s04-*.md on origin/main), qa/portfolio/agent-cost-accounting/qualification.md and pf-60-s04/ evidence,
  qa/demos/index/PF-60-S04.md;
- the previous independent acceptance methods and tools for reuse: qa/portfolio/agent-cost-accounting/pf-60-s03/
  independent-acceptance-20261008/ and pf-60-s05/independent-acceptance-*/ (README, tools/);
- user-facing help (`/cost help`), docs/, and /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
  (binding: disposable homes, CORBANU_TEST_NO_NATIVE_KEYRING=1, never the real keychain; keys only via
  `X="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper <label>)" <cmd>`
  on the consuming command; never print a key).
Build CURRENT origin/main (record the commit) with developer accounting, new scratch homes, and verify EVERY acceptance
criterion of PF-60-S04 yourself with real provider calls through the TUI via the repo's tmux harness
(docs/tmuxHarness.md): parent/child journeys across providers — OpenAI gpt-5.4 (label openai-api-key, as OPENAI_API_KEY),
GLM 5.3 Flash on zai (provider/zai_api_key; owner policy 2026-10-10: use glm-5.3-flash, not glm-5.2), Kimi k3 on
kimi-code (provider/kimi_api_key), Claude plan via CLAUDE_CODE_OAUTH_TOKEN (claude-plan-test-token); /side conversations
and memory consolidation recorded under their conversation; the single "Price: none recorded" line for unpriced requests;
cancellation ("usage incomplete"/"at least"), missing price, duplicate retry (one request, linked attempts, tokens once),
restart/resume and historical inspection identical, /cost date bounds and refusal reasons. Independently recompute every
pay-per-use total from provider-reported usage at published prices; specifically re-check waiver (a) from PF-60-S05
(OpenAI exact estimate) so Travis can decide whether to lift it. List anything not verifiable (ChatGPT login, image,
realtime, TensorCash) as NOT VERIFIABLE with reason. Verdict per criterion (PASS / FAIL / NOT VERIFIABLE) with exact
commands and captures (redact keys/paths) in qa/portfolio/agent-cost-accounting/pf-60-s04/independent-acceptance-<YYYYMMDD>/
(README.md + captures/, data/, tools/). Then ONE separate code-blind reviewer (installed `corbanu exec`, `-m
claude-opus-5-5-plan -c model_provider="claude-plan" -c model_reasoning_effort="high" -s read-only`) audits it; save
REVIEW.md there. Own worktree off origin/main under /Volumes/CorbanuDrive/Corbanu/worktrees/; docs-only PR; merge when ALL
checks are green (never with checks pending; `gh pr merge N --merge --match-head-commit <sha>`; never force-push). Key scan
before committing. Do not fix code; file issues for defects. Keep spend small (<$1 OpenAI). Final message: per-criterion
verdicts with one-line evidence, PR number, and a short plain-language recommendation for Travis on accepting PF-60-S04
and lifting waiver (a). Clean up scratch target dirs and homes (`rm -r <dir> < /dev/null`, never rm -f).
