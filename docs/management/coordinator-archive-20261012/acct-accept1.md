You are the INDEPENDENT functional acceptance executor for PF-60-S03 (accounting inspector, ranges, /cost screens).
Travis chose option 2 of decision acct-s03-acceptance-20260917. You must be code-blind: do NOT read the S03 implementation
source, the lane's briefs, or prior review files. You may read only: the acceptance request
qa/portfolio/agent-cost-accounting/pf-60-s03/acct-acceptance-75/acceptance.md, the sprint record's acceptance criteria,
user-facing docs/help, and sec-common.md (binding: disposable homes, CORBANU_TEST_NO_NATIVE_KEYRING=1, never the real keychain).
Build the CURRENT origin/main (record the commit) with developer accounting enabled as documented. Then, against REAL
repositories (clone 2-3 small public repos into a disposable dir) and real model calls (GLM 5.2, provider zai), on macOS
and on the RTX box (Linux; ssh ambient@100.99.88.49; sudo only via the RTX-machine vault rule if truly needed):
- run real sessions incl. subagents and concurrent `corbanu exec` runs; then inspect /cost and the CLI/JSON outputs;
- independently recompute totals from provider-reported usage and published prices you look up yourself; compare;
- exercise day/range views across a boundary, empty vs unavailable, missing price, no usage, deleted history.
Record exact commands, outputs, discrepancies, and a verdict per acceptance criterion (PASS / FAIL / NOT VERIFIABLE) in
qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/ (redact any keys/paths). Then get ONE
separate reviewer (installed `corbanu exec`, claude-plan, Opus 5.5 high, read-only) who also hasn't seen the source to
audit your evidence and verdicts. Docs-only PR, merge when green (`gh pr merge N --merge --match-head-commit <sha>`).
Do not fix code; file GitHub issues for real defects. Another worker (acct-s03a) is updating S03 code/docs concurrently;
avoid its files. Report verdicts and anything for Travis.
