# Round 69 evidence correction

Classification: routine internal QA correction; no product or Rust source changes.
Context: PF-60-S03, in progress, under the active
`docs/plans/active/portfolio-agent-cost-accounting.md` plan.
Product heading: **Product measurement**; excerpt:
“No commercial performance numbers have been supplied.”
This is evidence maintenance under the frozen allocation, not new product scope
or a functional/human-test handoff. Later independent functional qualification,
live-repository/platform proof and human acceptance remain with the integrator.

Base: `a06048a8a4cd1422c9ef80dcef5503ee297d8d4c`.
Frozen brief SHA-256:
`cc5f40194d2bf9a9ef75264331767246343af8742a353aeae387e3d2bb3c6a47`.
All commands start at the assigned repository root unless noted.
Set `QA=qa/portfolio/agent-cost-accounting/pf-60-s03` in the shell.

1. Read `docs/development/test-isolation.md` before testing.
2. `python3 -B "$QA/acct-bindings-69/priced_controls.py" "$QA/acct-bindings-69/NEW-PRICED"`
   must pass the unchanged baseline, accept all 75 mutants with the frozen
   round-66 auditor, and reject all 75 with no success receipt using the current
   auditor. It verifies each of the 24 source captures against the assigned base
   before comparing the content digests. The 15 subtotal pages comprise 9 priced,
   4 unknown-cost and 2 zero-recorded pages. The exact reported counterexample is
   `extra-total-historical`: append `Total: $9.99` to the selected rows and
   corresponding case/viewport so consistency checks cannot mask the hole.
3. `python3 -B "$QA/acct-inherited-64/page_controls.py" "$QA/acct-bindings-69/NEW-PAGES"`
   must reject the original 74 controls. The earlier auditors still accept 70.
4. `PYTHONDONTWRITEBYTECODE=1 python3 -B "$QA/acct-scope-62/coverage_controls.py" "$QA/acct-bindings-69/NEW-REMOVALS"`
   must reject all 28 amount-removal controls.
5. `python3 -B "$QA/acct-scope-62/audit_scope.py" "$QA/acct-scope-62/scope-run-01" "$QA/acct-bindings-69/NEW-SCOPE.json"`
   rechecks the separate store read-backs. It does not bind the root resolved by
   either identical empty page.
6. `capture_recheck.py` exclusively creates `capture-recheck.json`; run a copy
   with a fresh output name for replay. It verifies both artifact hashes,
   byte equality and day/row consistency. It makes no population-distinctness
   inference. The corrected note lives in round 66's `capture-bindings.json`;
   the original erroneous claim is explicitly retracted in its `RETURN.md`.
7. For a fresh Rust campaign, copy `gate.py` to another allocation directory at
   the same depth or choose a new output directory in the script. Invoke
   `prerequisites`, inspect completion/log, then `alone-feature`,
   `core-feature`, `core-default`, and `tui`, inspecting every result before
   dispatching the next. All use the same dedicated target and four test threads.
   The prerequisite build selects CLI, rmcp-client and code-mode-host together
   with a single `--bins`; Cargo rejects repeating that flag.
   Test commands use the checkout's guarded `just test`.
   Do not continue after a native credential prompt or live-profile access.

The 24-page digest contract qualifies only the frozen reader-run-05 content.
It is not a general monetary parser or automatic acceptance of new UI wording.
The independent arithmetic, price, coverage and unknown-cost assertions remain.
Do not regenerate expected digests from mutants.

Raw failed attempts and intermediate successful runs remain alongside final
runs. `intermediate-checks.json` records the rejected prerequisite command and
an intermediate harness error. Mutation copies are regenerable and ignored;
consolidated results retain commands' exits, receipt presence and stderr.
Rust logs are preserved as lossless gzip with uncompressed SHA-256.
No timeout setting, retry count, product source or Rust test was altered.
