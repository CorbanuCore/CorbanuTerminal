You are the INDEPENDENT functional acceptance executor for sprint PF-60-S05 (accounting collection correctness:
declared billing basis per route/credential with option B, versioned ledger format, best-effort compaction/search/image,
guardian and ephemeral sessions). Be code-blind: do NOT read implementation source (codex-rs/**), PR diffs, lane briefs,
worker logs/results or review files under .codex-work. You may read only:
- the PF-60-S05 sprint record and its acceptance criteria, qa/portfolio/agent-cost-accounting/pf-60-s05/ (billing-basis
  defaults, both-behaviour options memo with the option B decision, real-provider-checks-20261009.md) and
  qa/demos/index/PF-60-S05.md;
- the previous S03 independent acceptance method and tools for reuse:
  qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/ (README, tools/);
- the body review that motivated S05: qa/portfolio/agent-cost-accounting/pf-60-s03/body-review-20261008/review.md;
- user-facing help (`/cost help`), docs/, and /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
  (binding: disposable homes, CORBANU_TEST_NO_NATIVE_KEYRING=1, never the real keychain; keys only via
  `X="$(corbanu vault auth-helper <label>)" <cmd>` with the INSTALLED corbanu, on the consuming command; never print it).
First wait until PR #346 is merged (poll `gh pr view 346 --json state` every 5 min, up to 2 h). Then build CURRENT
origin/main (record the commit) with developer accounting (and a default build where a criterion needs it), new scratch
homes, and test EVERY acceptance criterion of PF-60-S05 with real provider calls where the criterion calls for it
(GLM 5.2 on zai and the Z.AI coding-plan route at minimum; any other route whose credential exists in the vault under
provider/*; list missing ones as NOT VERIFIABLE). Independently recompute every total you report from provider-reported
usage. Specifically verify the review's findings are gone: subscription work (e.g. a coding-plan route) never counted as
money spent; plan work without a catalogue price not shown as pay per use; an older ledger still readable; compaction /
web search / image generation return their result when accounting can't record; guardian reviews and ephemeral sessions
accounted or explicitly reported. Verdict per criterion (PASS / FAIL / NOT VERIFIABLE) with exact commands and captures
(redact keys/paths) in qa/portfolio/agent-cost-accounting/pf-60-s05/independent-acceptance-<YYYYMMDD>/ (README.md +
captures/, data/, tools/). Then ONE separate code-blind reviewer (installed `corbanu exec`, `-m claude-opus-5-5-plan
-c model_provider=claude-plan -c model_reasoning_effort=high -s read-only`) audits it; save REVIEW.md there. Own worktree
off origin/main under /Volumes/CorbanuDrive/Corbanu/worktrees/; docs-only PR; merge when ALL checks are green (never with
checks pending; `gh pr merge N --merge --match-head-commit <sha>`; never force-push). Key scan before committing. Do not
fix code; file issues for defects. Final message: per-criterion verdicts with one-line evidence, PR number, and a short
plain-language recommendation for Travis on accepting PF-60-S05.
Clean up scratch target dirs and homes (`rm -r <dir> < /dev/null`, never rm -f).
