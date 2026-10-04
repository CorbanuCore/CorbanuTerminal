# RETURN

Allocation `acct-acceptance-75`; `gpt-6-astra`, effort `high`.
Claim `7f14ecca-67e4-4bb7-b039-cb099553a564`.
Allocation digest `ba60cf28b745b9543b8f98acc69783c914e1f59b3542a7ff32b7e80c001864df`.
Brief read first; SHA-256 matched
`7b0f39e48f37be7aae39526c3ce32176146e29706657b1194b94187a983b7926`.
Clean launch HEAD matched `49e7d1b8cf6d762c433360e54efccd50a0840f83`.

Three corrected records:

1. [Corrective commit message](commit-message.txt) explicitly retracts the
   qualifying-mode independence claim in `49e7d1b8c`. A new corrective commit
   places this in Git history without rewriting that already-received commit
   or its merge `0850cf8654f03183ef698a64576e6d2d09267b5a`. Qualifying mode runs
   two arithmetic checks, then digest assertions, then remaining monetary checks;
   only diagnostic `--semantics-only` bypasses digests. The old receipt links
   this correction at the claim's evidence source.
2. [Round-72 return](../acct-controls-72/RETURN.md) now specifies 15 named subtotal
   pages: 9 priced, 4 unknown-cost, 2 zero-recorded. Only the first 13 require a
   displayed subtotal; the two zero-recorded pages require its absence, and their
   display-value assertion is vacuous. No all-page, arbitrary-money, viewport
   semantic or billed-dollar coverage is claimed.
3. [summarize.py](../acct-controls-72/summarize.py) derives executions from lane
   counts checked against hashed raw log summaries, checks mutation rows against
   the saved plan, and derives mutation/check/viewport totals. Reproduction
   preserves the original receipts byte-for-byte: 342 historical executions,
   147 named checks, 159 mutations, 45 viewports. [Recheck](summary-recheck.json)
   validates retained evidence; it is not a fresh mutation campaign.

[Acceptance document](acceptance.md) gives proven evidence, substantive limits,
first slices, responsible actors and incremental effort estimates for independent
code-blind design; isolated independent execution and evidence review;
live repositories; platforms/profiles; and named receiving/human acceptance.
Fable's existing authority covers QA allocation and review extensions. Dollar
cost is unquoted because rates, host availability and spend caps are unconfirmed.
No acceptance, new product permission or release authorization is fabricated.

Judgement: **fit for a developer to inspect recorded-request estimates today,
within disclosed scope/coverage. Not trustworthy as a complete bill for a run.**

Fresh guarded gates, prerequisites first, shared dedicated target,
`NEXTEST_TEST_THREADS=4`, commands run from `codex-rs`:

| Lane | Passed/run | Skipped | Failed / timed out / flaky / leaky |
| --- | ---: | ---: | --- |
| `just test -p codex-core accounting` | 124/124 | 3545 | 0 / 0 / 0 / 0 |
| Same with `--features codex-core/developer-accounting` | 127/127 | 3545 | 0 / 0 / 0 / 0 |
| `just test -p codex-tui usage` | 91/91 | 4078 | 0 / 0 / 0 / 0 |

**342/342 executions**, overlapping core test sets, not unique tests. Failure
names: **none**. Both core lanes mark
`suite::accounting_responses_ws_recovery::accounting_responses_ws_native_auxiliary_scope_and_event_parity`
slow and passed (30.982s default, 31.283s feature). [Receipt](test-results.json)
records commands, environments, base, timings and log hashes; lossless logs are
in `gates-01/`. No native credential prompt or live-profile access was observed.
`finalize.py` validates those logs and reruns the repaired historical summarizer.
Plan checker: 3 active/3 limit. Sprint checker: 115 current/127 archived.
Whitespace check passes. No Rust/product edits or workspace formatter; no push.

Brief precision: the claimed round-72 merge exists and contains the assigned
commit. Its introductory claim that digest/semantic layers are independent is
over-broad in qualifying mode, exactly as task 1 reports. "Only acceptance"
is not evidence that receiving-tree checks, independent functional execution
or inherited sprint dispositions are complete; those remain real work. No other
material factual error was found. No new live TUI or functional acceptance was
performed: these are routine internal evidence changes, not new user behavior.

Changed lines: the three pre-existing files total **+36/−11**. New helpers,
documents and generated receipts are enumerated in [scope.json](scope.json);
its self-entry is excluded to avoid a self-referential digest. Final Git diff
statistics include those new artifacts. All changed paths remain inside the
assigned `qa/portfolio/agent-cost-accounting/pf-60-s03/` scope.
