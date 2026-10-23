You are the INDEPENDENT functional acceptance executor for a targeted re-run of PF-60-S05 (accounting collection
correctness). Be code-blind exactly as the first run was: do NOT read implementation source (codex-rs/**), PR diffs,
lane briefs, worker logs/results or review files under .codex-work. You may read only: the PF-60-S05 sprint record and
ACs; qa/portfolio/agent-cost-accounting/pf-60-s05/ (defaults table, options memo, real-provider checks, and the first
independent run independent-acceptance-20261009/ incl. its tools/ and method); user-facing docs/ and `/cost help`; and
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding: disposable homes,
CORBANU_TEST_NO_NATIVE_KEYRING=1, never the real keychain, never print a credential).
The first run (PR #355) left these NOT VERIFIABLE for lack of credentials. Travis has now added three to the vault:
- `provider/kimi_api_key`  — his real Kimi Code membership key (route `kimi-code`, model `k3`, env KIMI_API_KEY).
- `claude-plan-test-token` — a Claude subscription setup token (route `claude-plan`, env CLAUDE_CODE_OAUTH_TOKEN).
- `openai-api-key`         — an OpenAI project API key (built-in `openai` provider; in `exec` the key must be given as
  CODEX_API_KEY — OPENAI_API_KEY alone gets 401 there; note that as an observation).
The coordinator smoke-tested all three: each answers. IMPORTANT credential mechanics: resolve each key with the INSTALLED
wrapper and WITHOUT the test env vars, then pass it only on the consuming command, e.g.
  V(){ env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper "$1" < /dev/null; }
  KIMI_API_KEY="$(V provider/kimi_api_key)" <debug build> exec ...
Never echo, log or commit any value; key-scan everything before committing.
Build CURRENT origin/main (record the commit) with developer accounting, new scratch homes, and re-run ONLY the items that
were NOT VERIFIABLE because of these credentials, with real requests and independently recomputed totals:
- AC5 Kimi Code real: real k3 requests record as subscription, nothing counted as spent, overflow note shown (Blocker 1).
- AC10 / AC12 / AC2 parts for `kimi-code`, `claude-plan` and the OpenAI API-key route: each records with its declared basis
  (OpenAI API key = pay per use with an exact estimate you recompute from the published price).
- AC4: the Claude-plan/pane-bridge part if reachable with the token (Claude panes); priority tier if the OpenAI key allows
  `service_tier` priority on a supported model; image generation and realtime ONLY via the API-key route if supported.
- AC7: real web search and image generation return their result when accounting cannot record (same injected-failure
  method as the first run), using the OpenAI API-key route.
ChatGPT-login items stay NOT VERIFIABLE (Travis won't swap his ChatGPT account login); say so. Keep spend small (short
prompts; one image at the smallest size).
Record verdicts (PASS / FAIL / NOT VERIFIABLE) with exact commands and captures (redacted) in
qa/portfolio/agent-cost-accounting/pf-60-s05/independent-acceptance-20261009/rerun-<YYYYMMDD>/ (README.md + captures/,
data/), then ONE separate code-blind reviewer (installed `corbanu exec`, `-m claude-opus-5-5-plan -c
model_provider=claude-plan -c model_reasoning_effort=high -s read-only`) audits it; save REVIEW.md there. Own worktree off
origin/main under /Volumes/CorbanuDrive/Corbanu/worktrees/; docs-only PR; merge only when ALL checks are green
(`gh pr merge N --merge --match-head-commit <sha>`; never force-push). File issues for defects; don't fix code. Final message:
per-item verdicts with one-line evidence, total spend, PR number, and an updated recommendation for Travis on accepting
PF-60-S05. Clean up scratch targets/homes (`rm -r <dir> < /dev/null`, never rm -f).
