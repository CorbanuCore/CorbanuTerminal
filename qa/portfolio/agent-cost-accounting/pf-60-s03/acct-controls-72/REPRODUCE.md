# Reproduce allocation acct-controls-72

Run from the assigned checkout root. Read `docs/development/test-isolation.md`
first. Set `QA=qa/portfolio/agent-cost-accounting/pf-60-s03`; choose unused output
directories below this allocation. No live profile or product execution is
needed for the Python checks.

1. `python3 -B "$QA/acct-controls-72/controls.py" "$QA/acct-controls-72/NEW-CONTROLS"`
   runs two positive baselines, 159 semantic mutations covering all 147 named
   monetary check instances, 24 JSON integrity mutations and 45 viewport-only
   integrity mutations. Each viewport mutation also passes semantics-only:
   JSON/semantic checks alone would miss it. The independently enumerated plan
   must exactly match the auditor's check inventory, so deleting a check fails
   the campaign. Replacing its condition with true fails its named mutation.
2. `python3 -B "$QA/acct-scope-62/coverage_controls.py" "$QA/acct-controls-72/NEW-REMOVALS"`
   now rejects all 28 removals for exact-count/display-count semantic reasons.
   It no longer accepts a generic failure or a digest mismatch as proof.
3. `python3 -B "$QA/acct-bindings-69/priced_controls.py" "$QA/acct-controls-72/NEW-PRICED"`
   and `python3 -B "$QA/acct-inherited-64/page_controls.py" "$QA/acct-controls-72/NEW-PAGES"`
   preserve the earlier 75 and 74 integrity regression controls.
4. `python3 -B "$QA/acct-controls-72/readback_separation.py" "$QA/acct-controls-72/NEW-READBACK"`
   proves four saved attempts per dataset, disjoint attempt/request/thread ID
   sets, and exact round-62 before/during store equality. Four negative controls
   reject overlapping IDs or a changed during-reader store.
5. For Rust gates copy `gate.py` into a fresh allocation directory at this depth,
   or select a fresh output directory in a copy. Run `prerequisites`, inspect
   the log, then `core-default`, `core-feature`, `tui`, inspecting each before
   its successor. They share the existing dedicated target under this writable
   QA tree, use `NEXTEST_TEST_THREADS=4`, and invoke the checkout's guarded
   `just test` from `codex-rs`. Stop on any native prompt/live-profile access.
6. `python3 -B "$QA/acct-controls-72/summarize.py"` verifies the recorded gates,
   source-bound viewport digests and per-check mutation failures. It emits compact
   summaries and losslessly compresses large direct reports. Gzipped JSON/JSONL
   is raw original output, readable with Python's `gzip` module.

The default auditor verifies the original 24 JSON content hashes and all 45
decompressed viewport hashes. The expected viewport identities/hashes were
derived only after checking every gzip byte against the assigned base commit;
`viewport-provenance.json` repeats that check. JSON contracts were unchanged.
Decompressed hashes intentionally ignore gzip header metadata while binding
every visible byte. A viewport-only edit could hide/change displayed amounts,
warnings, clipping or startup state even if selected JSON stayed correct.

`--semantics-only` is a mutation diagnostic mode, explicitly reported with
`capture_qualified=false` and zero content bindings. It cannot qualify a
capture's integrity. Monetary failures are accumulated so overlapping narrow
and general amount checks all execute; no success receipt is written on failure.
Page identity, fixture consistency and nonmonetary checks still apply.

Final results: `controls-04/`, `removal-controls-02/`, `priced-controls-01/`,
`page-controls-01/`, `readback-01/`, `gates-01/`. The original failed and
intermediate attempts remain separate; see `intermediate-attempt.md`.
These are internal evidence checks, not new TUI captures or independent
functional acceptance. See [the sprint acceptance gap](acceptance-gap.md).
