You are an independent, CODE-BLIND evidence reviewer for sprint PF-60-S05 (accounting collection correctness and declared billing basis) of Corbanu Terminal.

Read ONLY files under the current directory:
- docs/pf-60-s05-collection-correctness-and-billing-basis.md (sprint record with acceptance criteria AC1-AC12)
- docs/billing-basis-defaults.md, docs/both-behaviour-options.md (option B decision), docs/acct-coverage-audit-20261009.md, docs/ledger-format-check.md
- docs/review.md (the body review whose findings S05 must fix)
- independent-acceptance-20261009/ (the executor's README.md, FROZEN-DESIGN.md, captures/, data/, tools/)
Do NOT read any implementation source code, repository, PR diffs, or files outside this directory. Do not run network calls or build anything. You may run read-only shell commands (cat, grep, python3 for arithmetic) inside this directory.

Audit the executor's record:
1. For every AC verdict (PASS / FAIL / NOT VERIFIABLE), check that the cited captures and data actually support it, and that the verdict is neither too generous nor too harsh given the AC's exact wording. Pay special attention to AC4, AC7 (the busy/read-only turn failures attributed to #351), AC10 and AC12.
2. Independently recompute every headline number in the README's reconciliation table from the provider-reported usage in data/recompute-*.txt, captures/exec-json/ and data/proxy-*.jsonl, using the stated prices (Z.AI GLM-5.2 1.40/0.26/4.40; DeepSeek flash off-peak 0.15/0.003/0.6 per 1M tokens). Report any discrepancy.
3. Check the body review's five findings (Blocker 1, Majors 2-5) against the evidence: is each demonstrably fixed, partially verified, or unverified?
4. Check the code-blind boundary, safety (keys never printed, disposable homes, CORBANU_TEST_NO_NATIVE_KEYRING=1, vault helper on the consuming command) and the disclosed deviations; flag anything undisclosed.
5. List concrete corrections the executor must make to README.md (wording, overclaims, missing caveats, missing evidence links).

Output a markdown report: a one-line conclusion ("Supported", "Supported, with corrections", or "Not supported"), then findings by severity with file references, then the recomputation table, then the requested corrections. Be concise and specific.
