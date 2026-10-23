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

The previous run FAILED three criteria; fixes have merged since (PRs #299, #303; #306 for the #289 wording is still in CI:
prepare first, and before building poll `gh pr view 306 --json state` until it is MERGED, or until 2 hours have passed, then
build whatever origin/main is and say whether #306 was included). Build the CURRENT origin/main (record the commit) with
developer accounting exactly as the previous run did (tools/build-mac.sh, tools/build-rtx.sh), on macOS and on the RTX box
(ssh ambient@100.99.88.49; reuse ~/corbanu-rtx/pf60-indep-20261008 incl. its libfaketime build; use a NEW scratch home).
Re-run ONLY these, with real GLM 5.2 calls (provider zai) on real public repos, as code-blind as before:
1. Criterion 15 / #287 (store robustness): turns with the process clock behind the ledger checkpoint (58 s and 6 days, via
   libfaketime with FAKETIME_DONT_FAKE_MONOTONIC=1, chronologically after a normal turn) must complete; check whether they
   are recorded or reported as an accounting gap, and that the error never names Anthropic. Also one turn with the clock
   >90 days behind. macOS and Linux if libfaketime is available on macOS; otherwise Linux, and say so.
2. Criterion 9 / #286 (deleted history): reproduce the previous steps (sessions with a subagent on two days, delete one with
   `corbanu delete --force`, view the day from another conversation). The day page must not claim "No other conversation
   recorded requests on this day" for a day with deleted spend; record exactly what it says instead. macOS and Linux.
3. Criterion 8 / #288 (no-usage): the same zainousage/zaiproxy proxy setup as before (tools/usage_proxy.py); does the
   no-usage conversation's own page give a next step for its own requests, and is missing usage told apart from a missing
   price? (#288 was handled by a different worker; just verify.)
4. #289 observations (not a criterion): fresh-home /cost wording before the first request; component lines vs costs in
   request technical details for a GLM 5.2 request; a week and a month bucket reaching today; request numbering order;
   and in a DEFAULT (no developer-accounting) Linux build, `/usage requests <yesterday>` and `/usage requests <tomorrow>`
   typed in the TUI the same way as rtx-52 was captured.
Independently recompute any totals you report, as before. Record exact commands, captures (redact keys/paths), and a
verdict per re-run criterion (PASS / FAIL / NOT VERIFIABLE) in a new directory NEXT TO the previous evidence:
qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/rerun-<YYYYMMDD>/README.md (+ captures/, data/).
Then get ONE separate reviewer (installed `corbanu exec`, `-m claude-opus-5-5-plan -c model_provider=claude-plan
-c model_reasoning_effort=high -s read-only`, also code-blind) to audit the evidence; save its report as REVIEW.md there.
Work in your own worktree off origin/main under /Volumes/CorbanuDrive/Corbanu/worktrees/. Open a docs-only PR and merge it
when checks are green (`gh pr merge N --merge --match-head-commit <sha>`; never force-push). Scan for key material before
committing. Do not fix code; if a defect remains, comment on its issue (or file a new one) with the evidence.
Final message: per-criterion verdicts with one-line evidence each, the PR number, and anything for Travis.
Clean up: remove your own scratch target dirs and scratch homes (`rm -r <dir> < /dev/null`, never rm -f) when done.
