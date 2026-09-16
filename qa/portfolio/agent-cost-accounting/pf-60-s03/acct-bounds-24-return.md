# RETURN — acct-bounds-24
Action acct-bounds-24; claim ea898b3d-1592-441c-979a-c7f3def6f729; runtime gpt-6-astra / high.
Allocation digest 19cc8347575b0e8e482d8719957f0916bac14156f1752a717f9a110a510e897d.
Brief SHA-256 verified: de931007aac056ca861df9760bf5bb78a77bf6a71690950fd834f79316d92cf1.
Clean dispatch HEAD c82680abeb1401f4138eec67ccf5e8e20fa4d2f7; branch bootstrap/acct-inspect-20260915; worktree /Volumes/CorbanuDrive/Corbanu/worktrees/acct-inspect-20260915.
Bounded correction under active portfolio-agent-cost-accounting / PF-60-S03 (in_progress). Product heading **Measurement targets**: “The following metrics must be instrumented, with targets set through the decision rights defined above.”

## Disclosed limit: whole-store denominator
The scan budget still depends on all attempts plus compact-day rows, including unrelated historical owners. The first affected workflow is the supervisor/subagent hierarchy this inspector exists to explain. This is honest refusal with the wrong denominator; it is not corrected by this allocation.
The new public-API test accounting_inspect_whole_store_denominator_thresholds pins adjacent Ready/TooLarge counts for a supervisor with seven children (one request each), and for an empty current day. It populates real accounting tables, clones a genuine maintenance-produced compact payload on unrelated old-day owners, verifies exact total row counts, and calls AccountingStore::inspect_day.
Measured thresholds (test passed in 3.037s): seven-child hierarchy Ready at 95,236 rows, TooLarge at 95,237; empty current day Ready at 571,428 rows, TooLarge at 571,429. Counts include eight raw attempts plus unrelated compact rows. The hierarchy consumes 42 scan-row multiples (initial count + eight five-scan owner validations + candidate discovery), plus visits. Even the empty day requires seven multiples plus one visit. The brief's approximately 100,000 and 666,000 figures therefore understate how early refusal begins, especially the second threshold.
Named fix: an index on owner and dispatch-day, with the associated schema/migration and indexed inspection-query changes. That is a product decision outside this allocation; no index, schema or denominator change was attempted.

## Changes
Dropped the candidate query's LIMIT 20001. Candidate traversal still charges the shared visit budget before any continue; SQL truncation can no longer silently omit descendants if that budget changes. Candidate discovery still materializes its result vector; this is not a streaming or transient-allocation bound.
Graph bytes now count only retained map keys and parent strings, after unrelated-root rollback. Parsed-and-dropped source text is not charged. The provisional 10,000-node check remains inside insertion, and the 4 MiB selected packet/graph check remains before retention.
Corrected accounting_inspect_provisional_unrelated_chain_is_bounded: before, its oversized whitespace-padded CLI source asserted TooLarge; after, it requires Ready with one selected attempt and zero unknown-parent attempts. The >10,000-hop refusal remains unchanged. Every existing test remains.
Extended accounting_inspect_work_budget_spans_range_buckets with a positive three-day Day-grouped range over the 12,001-row / 600-root fixture: three complete buckets, each containing a Ready value equal to the corresponding standalone day. Its existing 72-hour TooLarge assertion remains.
Lineage ownership still uses untyped string equality on the tree surface, unchanged here. The attempts surface instead rejects noncanonical stored ownership spellings and checks typed ThreadId identity. Harmonizing those surfaces is out of scope.

## Verification
Read docs/development/test-isolation.md before tests; used guarded just test with disposable profiles and native-keyring denial.
State gate: just test -p codex-state accounting, exit 0; 166 passed, 0 failed, 187 skipped, 32.973s test execution (6m 11s compilation). Exact failure names: none. No retries, flaky or LEAK markers. New threshold test 3.037s; corrected provisional-chain test 6.674s; positive/negative range test 7.443s. Nextest run 1bc7399d-b674-41ea-aef0-aac726be9c4b. Log acct-bounds-24-state.log.
TUI gate: just test -p codex-tui usage, exit 0; 91 passed, 0 failed, 4,071 skipped, 0.432s test execution (18m 09s compilation). Exact failure names: none. No retries, flaky or LEAK markers. Nextest run c881c45a-45f0-42b4-81a0-b4cd5928f8ab. Log acct-bounds-24-tui.log.
Sprint checker: passed, 115 current / 127 archived. Scoped rustfmt on the three changed Rust files only; unstable imports_granularity warnings disclosed. Final whitespace/scope check: passed; only the three allocated Rust files and this allocated receipt changed.
Changed lines against dispatch HEAD (additions + deletions): 118 inside tests; 19 production Rust + 29 receipt = 48 outside tests; 166 total. Below the 120 target / 200 hard outside-test budget. Generated ignored logs excluded from authored counts and retained separately; final log line counts: 415 state + 691 TUI = 1,106.
Brief corrections: measured boundaries are 95,237 and 571,429 rows, as detailed above. The existing oversized-source fixture is valid JSON (whitespace-padded "cli"), not a corrupt source. Source text is held transiently during parsing, but not retained in the ancestry map.

Worker return to Fable only. Receiving, independent review, isolated code-blind execution/evidence review, true-TUI/live-repository qualification and human acceptance remain unclaimed sprint gates. Plan/sprint ledgers are outside this worker's writable scope; this receipt supplies manager bookkeeping. No credentials read, live profile, push or release.
