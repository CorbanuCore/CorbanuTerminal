# RETURN

Allocation: `acct-bindings-69`; Astra High.
Claim: `5800cd5e-de56-483c-80b3-588aeea5677a`.
Allocation digest:
`83c9ebc5fb8d306ad70fe59234331097b1e3466be9a9b7442d07c71a47fbe387`.
Brief SHA-256 verified before other work:
`cc5f40194d2bf9a9ef75264331767246343af8742a353aeae387e3d2bb3c6a47`.
Clean launch HEAD matched `a06048a8a4cd1422c9ef80dcef5503ee297d8d4c`.

**Distinctness retracted.** The round-66 claim “They remain distinct
captures/attempt populations” is explicitly retracted in its RETURN.
The two selected JSON files are byte-identical, SHA-256
`cb01c7c7fdc7e4872d760318224460f6a79120b9c2c2e8409dfdb23f9c347465`.
Both viewport gzip files are 1249 bytes and byte-identical, SHA-256
`e40aa441137f8f956d17b726e035d3f0ea510a0b3dc5f847511868bfb2a51c7e`.
The pages are indistinguishable, including read time and admission interval,
and establish neither identical nor distinct populations or resolved roots.
Separation rests on store read-backs alone.
[Corrected note](../acct-receipt-66/capture-bindings.json) and its generator
now say this explicitly; [capture recheck](capture-recheck.json) verifies the
actual bytes and hashes the store read-back sources.
The [scope recheck](scope-recheck.json) retains round 62's four priced attempts
across three roots at USD 0.00071 each, USD 0.00284 total. That amount is not
imported into round 61.

**Content binding extended to all 24 selected pages.**
[Auditor](../acct-readers-61/audit_readers.py) now requires the full
[24-page digest contract](../acct-readers-61/page-content-digests.json), alongside
the existing arithmetic/coverage checks. The 15 subtotal pages are 9 priced,
4 unknown-cost and 2 zero-recorded pages. Expected content was checked against
the unchanged captures in the assigned base before mutation testing.

[Final 75 new controls](priced-controls-03/results.json) all pass the frozen
round-66 auditor and fail the revised auditor before it writes a success receipt.
Each subtotal page receives five injected forms: dollar total, euro total,
cents, symbol-free estimate and leading-decimal dollars.
The exact requested counterexample, `extra-total-historical`, appends
`Total: $9.99` to `historical-selected.json`: old exit 0 with receipt;
new exit 1, no receipt, `unexpected content on subtotal page`.
The baseline passes with all 24 content bindings.
The [74 existing controls](page-controls-03/results.json) and
[28 amount-removal controls](removal-controls-02/results.json) also reject every
mutant on the final checker. These are auditor tests, not new product captures.

**Timeout verdict: load-sensitive timeout, inferred from timings.**
The exact test
`suite::accounting_responses_ws_recovery::accounting_responses_ws_native_auxiliary_scope_and_event_parity`
passes alone with developer-accounting in **31.743 s** and in the full feature
lane in **31.335 s**, with no retries. It also passes in the default lane in
**31.650 s**. Round 66's default result was 58.790 s; its feature attempts timed
out at 60.013 and 60.012 s. The Rust Git tree is identical across both rounds.
[Attribution](timeout-attribution.md) includes the broader prior-lane slowdown,
exact logs and limits: no specific resource contention was measured, and an
intermittent defect cannot be excluded. No reproducible correctness defect,
timeout relaxation or test exemption is claimed. Historical failures remain.

Final guarded gates, shared dedicated target, prerequisites built first and
`NEXTEST_TEST_THREADS=4`:

| Lane | Passed/run | Skipped | Suite seconds |
| --- | ---: | ---: | ---: |
| Core accounting default | 124/124 | 3545 | 46.287 |
| Core accounting developer-accounting | 127/127 | 3545 | 45.654 |
| TUI usage | 91/91 | 4078 | 1.240 |
| Isolated feature replay | 1/1 | 3671 | 31.766 |

All four runs: zero failed, timed out, flaky or leaky tests; failure names: none.
Required lanes total **342/342 executions**, with one additional isolated
execution. Default/feature counts overlap; this is not 342 unique tests.
[Machine summary](test-results.json), commands, environments, uncompressed hashes
and lossless logs in `gates-02/` preserve the results. No native credential
prompt or live-profile access was observed.

Worker errors are retained in [intermediate checks](intermediate-checks.json):
the first prerequisite command repeated `--bins` and was rejected, and an
intermediate validation-order change tripped a harness error-message assertion.
The final command/expectation corrections passed in fresh output directories.
The isolated run's original JSON lacked derived counts due to singular “test”;
the summary reparses its hash-checked log without changing the original record.

**Changed lines.** Existing tracked files: **+19/−11**, all in QA. New authored
Python, prose, digest contract, copied earlier auditors and generated evidence
are enumerated separately in [scope inventory](scope.json); build outputs and
regenerable mutation copies are ignored. Production and Rust test changes:
**zero**. No broad formatting, commit or push. Final whitespace and scope checks
pass.

**Brief correction:** “15 priced pages” means 15 subtotal pages: nine are priced,
four have unknown cost and two are zero-recorded. The byte-identity finding and
reported unrecognized-dollar-total hole were both reproduced; no other factual
brief error was found.

This is routine internal QA maintenance under the active PF-60 plan and
in-progress PF-60-S03. Product context: **Product measurement**, excerpt
“No commercial performance numbers have been supplied.”
No functional/human-test or release acceptance is declared. The integrator's
later independent code-blind, live-repository/platform and human-acceptance gates
remain open; this worker does not grant their N/A or approval.
[Reproduction instructions](REPRODUCE.md).
