NOTE: you are a RELAUNCH. The previous run of this brief died from a transient provider 429 after doing real work. Recover its state first from /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/acct-rerun3.log.died-429 (use tail/rg; it is long), its worktree under /Volumes/CorbanuDrive/Corbanu/worktrees/, any scratch dirs and open PRs (gh pr list --author @me); continue from there, don't redo finished work.

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

This is the FINAL confirming re-run before Travis decides on PF-60-S03 acceptance. Earlier runs: the first independent
run (PR #292), rerun-20261008 (PR #311) and rerun2-20261008 (PR #315). Since then these merged: #318 (fixes the #289
residuals R1, R1b, R2, R3 and criterion 13b) and #322 (fixes #308: delete with the clock behind the ledger checkpoint).
Wait until `gh pr view 322 --json state` is MERGED (poll every 5 minutes, at most 2 hours; then build whatever main is and
say whether #322 was included). Build CURRENT origin/main (record the commit) with developer accounting exactly as before
(tools/build-mac.sh; tools/build-rtx.sh on the RTX box for the libfaketime cases), plus a DEFAULT (no developer-accounting)
macOS build, each with a NEW scratch home. Re-run ONLY these, with real GLM 5.2 calls (provider zai), code-blind:
1. #289 R1/R1b/13b in the DEFAULT build: `/usage requests`, `/usage requests <past day>`, `<future day>`, and the `/usage`
   menu. Record exactly what each says.
2. #289 R2 (week and month buckets reaching today, and a week after today) and R3 (retention floor wording agrees between
   the range header and day pages) in the developer build.
3. #308: `corbanu delete --force <session>` with the clock 58 s and 6 days behind the last checkpoint (libfaketime, as in
   rerun-20261008 criterion 15): it must either succeed with consistent state or fail leaving the session fully intact
   (rollout present, resumable); then the day page's "deleted conversations" disclosure.
4. A short regression pass of the previously PASSED criteria 8a, 9 and 15 (one case each).
Verdict per item (PASS / FAIL / NOT VERIFIABLE) with exact commands and captures (redact keys/paths), totals independently
recomputed, in qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/rerun3-<YYYYMMDD>/ (README.md
+ captures/, data/). Then ONE separate code-blind reviewer (installed `corbanu exec`, `-m claude-opus-5-5-plan
-c model_provider=claude-plan -c model_reasoning_effort=high -s read-only`) audits it; save REVIEW.md there. Own worktree
off origin/main under /Volumes/CorbanuDrive/Corbanu/worktrees/; docs-only PR; merge when green
(`gh pr merge N --merge --match-head-commit <sha>`; never force-push). Scan for key material before committing. Do not
fix code; comment on/file issues for any defect. Final message: per-item verdicts with one-line evidence, the PR number,
and a one-paragraph recommendation for Travis on accepting PF-60-S03 (list anything still failing).
Clean up: remove your scratch target dirs and scratch homes (`rm -r <dir> < /dev/null`, never rm -f) when done.
