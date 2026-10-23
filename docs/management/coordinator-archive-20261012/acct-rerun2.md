You are the INDEPENDENT functional acceptance executor for a targeted re-run of PF-60-S03 (accounting /cost screens).
Be code-blind: do NOT read implementation source (codex-rs/**), PR diffs, lane briefs or review files under .codex-work.
You may read only:
- the previous independent run: qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/ (README.md,
  REVIEW.md, captures/, data/, tools/ — reuse its tools and method);
- qa/portfolio/agent-cost-accounting/pf-60-s03/acct-acceptance-75/acceptance.md and the sprint record's acceptance criteria;
- the GitHub issue texts #286, #287, #288, #289 (`gh issue view N`), which describe the defects in user terms;
- user-facing help (`/cost help`), docs/integrations/zai-glm-52.md, and
  /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding: disposable homes,
  CORBANU_TEST_NO_NATIVE_KEYRING=1, never the real keychain; key only via `ZAI_API_KEY="$(corbanu vault auth-helper
  provider/zai_api_key)" <cmd>` with the INSTALLED corbanu, substituted directly on the consuming command; never print it).

This is a SECOND targeted re-run. The first re-run (rerun-20261008/, PR #311) built origin/main b294864916, which did NOT
include PR #305 (the fix for #288, merged afterwards). Build the CURRENT origin/main (record the commit and confirm with
`git merge-base --is-ancestor <#305 merge commit> HEAD`) with developer accounting exactly as before (tools/build-mac.sh;
tools/build-rtx.sh only if needed), using a NEW scratch home. Re-run ONLY these, with real GLM 5.2 calls (provider zai) on
real public repos, as code-blind as before:
1. Criterion 8 / #288 (no-usage): the same zainousage/zaiproxy proxy setup (tools/usage_proxy.py). Does the no-usage
   conversation's own pages give a next step for its own requests that names its OWN provider, and is missing usage told
   apart from a missing price? Also check a request priced only in part, if the setup allows.
2. #289 residuals listed in rerun-20261008/README.md (and the criterion-13 question): re-check each on the new build.
Independently recompute any totals you report. Record exact commands, captures (redact keys/paths) and a verdict per item
(PASS / FAIL / NOT VERIFIABLE) in qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/rerun2-<YYYYMMDD>/
(README.md + captures/, data/). Then get ONE separate reviewer (installed `corbanu exec`, `-m claude-opus-5-5-plan
-c model_provider=claude-plan -c model_reasoning_effort=high -s read-only`, also code-blind) to audit the evidence; save its
report as REVIEW.md there. Work in your own worktree off origin/main under /Volumes/CorbanuDrive/Corbanu/worktrees/. Open a
docs-only PR and merge it when checks are green (`gh pr merge N --merge --match-head-commit <sha>`; never force-push). Scan
for key material before committing. Do not fix code; if a defect remains, comment on #288/#289 (or file a new issue) with
the evidence; if #288 passes, close it with a link to the evidence.
Final message: per-item verdicts with one-line evidence each, the PR number, and anything for Travis.
Clean up: remove your own scratch target dirs and scratch homes (`rm -r <dir> < /dev/null`, never rm -f) when done.
