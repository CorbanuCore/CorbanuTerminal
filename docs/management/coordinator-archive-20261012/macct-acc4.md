You are the INDEPENDENT functional acceptance executor for sprint PF-84-S04 (account management in /providers and onboarding; duplicate-row fix) — background: PF-84-S02 (named
accounts per provider: registry, storage, resolution; behind the default-off `named_accounts` flag). Be code-blind: do NOT
read implementation source (codex-rs/**, scripts/**), PR diffs, lane briefs, worker logs/results or review files under
.codex-work. You may read only: the S01/S02 sprint records and acceptance criteria
(docs/sprints/current/unified-provider-auth/pf-84-s04-*.md (the S01-S03 records are archived under docs/sprints/archive/unified-provider-auth/) on origin/main), the plan's PF-84 section
(docs/plans/active/unified-provider-auth.md), qa/provider-auth/pf-84/s04-gate.md, the previous independent acceptances qa/provider-auth/pf-84/independent-acceptance-20261010/ (README + REVIEW, for its method and the failed criteria only), qa/demos/index/PF-84-S0*.md,
user-facing docs (docs/authentication.md and any account docs), `corbanu account --help` / `--help` output, and
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding: disposable homes,
CORBANU_TEST_NO_NATIVE_KEYRING=1, never the real keychain; keys only via
`X="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME /Users/Neo/.local/bin/corbanu.bak-20261010 vault auth-helper <label>)" <cmd>`
on the consuming command; never print a key).
Build CURRENT origin/main (record the commit) and verify EVERY acceptance criterion of S04 from the outside, through the TUI (/providers and onboarding) via the tmux harness, as a
skeptical tester, through the CLI and the TUI via the repo's tmux harness (docs/tmuxHarness.md), with real providers:
GLM 5.3 Flash on zai (`provider/zai_api_key`; owner policy: glm-5.3-flash), Kimi k3 on kimi-code (`provider/kimi_api_key`),
OpenAI gpt-5.4 (`openai-api-key`) and Claude plan (`claude-plan-test-token`) where criteria apply. Prove isolation with
fake keys in named accounts (expected 401s) and per-account canary values; check the flag-off behaviour is unchanged; check
an existing single-account home carries over untouched; check that a missing/wrong account fails and never falls back;
for S04 also prove: add, rename, remove and switch accounts in /providers and onboarding behave as described and persist across restart; Esc changes nothing; no duplicate provider rows; the spawn line and /status show the active account; a fake-key account gets a 401 naming it; never touch the real ~/.local/bin/corbanu or the real home (use
CORBANU_LAUNCHER_WORK_DIR / CORBANU_LAUNCHER_WORKSPACE_DIR overrides; never touch the real ~/.local/bin/corbanu or the real
home). Verdict per criterion (PASS / FAIL / NOT VERIFIABLE) with commands and redacted captures in
qa/provider-auth/pf-84/independent-acceptance-s04-<YYYYMMDD>/ (README.md + captures/). Then ONE separate code-blind reviewer
(installed `corbanu exec`, `-m claude-opus-5-5-plan -c model_provider="claude-plan" -c model_reasoning_effort="high" -s
read-only`) audits it; save REVIEW.md there. Own worktree off origin/main under /Volumes/CorbanuDrive/Corbanu/worktrees/;
docs-only PR; merge when ALL checks are green (never with checks pending; `gh pr merge N --merge --match-head-commit
<sha>`; never force-push). Key scan before committing. Do not fix code; file issues for defects. Final message:
per-criterion verdicts with one-line evidence, PR number, and a short plain-language recommendation for Travis on accepting
S04. Clean up scratch targets and homes (`rm -r <dir> < /dev/null`, never rm -f).
