# Auxiliary WebSocket timeout attribution

Verdict: **load-sensitive timeout**, a timing-based attribution rather than a
reproduced correctness defect. The exact test is
`suite::accounting_responses_ws_recovery::accounting_responses_ws_native_auxiliary_scope_and_event_parity`.

| Execution | Target test result | Target duration |
| --- | --- | ---: |
| Round 66 default accounting lane | PASS | 58.790 s |
| Round 66 feature lane, attempt 1 | TIMEOUT | 60.013 s |
| Round 66 feature lane, built-in retry | TIMEOUT | 60.012 s |
| Round 69 alone, developer-accounting enabled | PASS, no retry | 31.743 s |
| Round 69 full feature lane, four threads | PASS, no retry | 31.335 s |
| Round 69 full default lane, four threads | PASS, no retry | 31.650 s |

The alone replay and full feature lane used the same dedicated target,
`NEXTEST_TEST_THREADS=4`, unchanged 60-second timeout and normal retry policy,
after CLI, rmcp-client and code-mode-host prerequisites were built. The full
feature lane passed 127/127 in 45.654 seconds. This was an actual in-lane replay,
not a second isolated invocation mislabeled as a lane.

The prior default run was already within 1.210 seconds of the same timeout.
Other tests were broadly slower in that prior feature campaign:
`accounting_responses_ws_native_two_reopens_and_original_prices` took
11.471 seconds versus 3.271 now, and
`accounting_anthropic_native_spawned_children_role_reload_and_fork_own_only_new_sends`
took 1.837 seconds versus 0.689 now. Those comparisons support campaign-level
load/setup sensitivity. The Rust tree is identical between round 66's
`270a6644e01d780ef93f8715dd0051481643dc7d` and this allocation's base
`a06048a8a4cd1422c9ef80dcef5503ee297d8d4c`: both `codex-rs` Git tree IDs are
`d1f029a6967f2212f85e98052a5567bcabe9e35c`. No Rust code was changed in this round.

Limits: these timings do not identify a particular contended resource or
exclude an intermittent defect. No OS scheduler trace was captured in the old
run. The old timeout logs end after the compact-phase fixture count and before
the first metadata-matrix success marker; therefore it would be unsupported to
claim the old attempts completed all 72 matrix cases. The new successful test
does execute all 8 × 3 × 3 cases and their existing assertions.
No assertion failure was reproduced and no timeout increase or test exemption
was used. The historical failed gate remains failed.

Evidence: original `../acct-receipt-66/gates-01/core-default.log.gz` and
`core-feature.log.gz`; current `gates-02/alone-feature.log.gz`,
`core-feature.log.gz`, lane JSON and `test-results.json`.
The original alone-feature JSON omitted derived counts because the parser
expected plural “tests”; `summarize.py` accepts singular/plural and derives its
1/1 result directly from the hash-checked log without overwriting that record.
