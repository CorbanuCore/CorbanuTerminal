# Clean-host verification on the RTX workstation — 21 September

Every mass failure this lane has been fighting for two days was the Mac, not the
code. Running the identical lanes on a clean Linux host settles it.

## Host

`ambient@100.99.88.49`, 32 cores, 89 GB RAM, Linux 7.0.0, Rust 1.95.0 pinned,
cargo-nextest 0.9.145, Python 3.14.4. Source tree copied from the branch at
`041b29bb9`; tests run through the same `scripts/isolated_rust_tests.py` wrapper
the `just test` recipe uses, so profile isolation is identical.

## Raw artifacts

The five lane logs and the run summary are committed beside this record in
`rtx-20260921/`: `head-core.txt`, `head-core-feature.txt`, `head-state.txt`,
`head-tui-usage.txt`, `head-tui-tokens.txt` and `lanes.summary`. They are the
nextest output copied from `~/corbanu-acct/` on the host, unedited, renamed from
`.log` because the repository ignores that suffix - the first attempt at this
commit cited five files that git had silently dropped, and the record asserted
they were present when only the summary was. An earlier version stated the
conclusion without citing artifacts at all, which is not good enough for evidence
that overturns a previous attribution.

## Result at the branch head

| lane | result |
| --- | --- |
| `codex-core` accounting | **131 run, 131 passed** |
| `codex-core` accounting, `developer-accounting` | **135 run, 135 passed** |
| `codex-state` accounting | **166 run, 166 passed** |
| `codex-tui` usage | **92 run, 92 passed** |
| `codex-tui` tokens | 66 run, 65 passed, 1 failed |

The single tui failure is
`accounting_inspect_maintenance_with_healthy_raw_renders_lag`, the pre-existing
inline-snapshot drift that also fails at base `6322a6e7c`. Its diff still shows
the `Snapshot is not current; newer activity is unverified` line. It is not this
change and is left for its own commit.

## What this overturns

On the Mac the same lanes reported 46 and 54 failures. On this host: zero. The
three feature-only failures that blocked receipt -
`accounting_anthropic_401_and_429_are_terminal_and_preflight_schema_fault_has_no_send`,
`accounting_anthropic_redirects_never_send_or_attribute_to_unapproved_endpoint`
and `accounting_responses_ws_native_handshake_and_postdispatch_errors` - all pass
here, and the whole three-test run took 0.459 seconds against 20-35 seconds per
test on the Mac before timing out. They were wiremock and deadline casualties of
a host still re-indexing after a crash, exactly as the earlier attribution
argued, and not defects in the coverage change.

Two of those three also failed at the worker's own commit `2e6d47e17` on the Mac,
which had made them look like defects inherited from the original round. That
reading was wrong; the host was the common cause.

## Why this matters beyond this change

The Mac's accounting lanes are not trustworthy while Spotlight is indexing, and
the internal deadlines in these suites are short enough that a busy host
manufactures failures faster than a reviewer can attribute them. This lane should
verify on the RTX host by default: the full accounting surface runs in about 60
seconds there against roughly 45 minutes of wall clock here, and it does not
require arguing about which failures are real.

## Re-run after the exclusion fix and the added assertions

| lane | result |
| --- | --- |
| `codex-core` accounting | **132 run, 132 passed** |
| `codex-core` accounting, feature | **136 run, 136 passed** |
| `codex-state` accounting | **166 run, 166 passed** |
| `codex-tui` usage | **92 run, 92 passed** |
| `codex-tui` tokens | 66 run, 65 passed, 1 pre-existing failure |

The counts rose by one because
`accounting_unattributable_request_is_served_without_evidence` now exists: it
drives the real transport with a gateway-pinned body and asserts the request is
sent, the turn's sampling still passes `check()`, no attempt row is written, and
a usage event on the excluded request records nothing instead of failing the
stream. That is the coverage whose absence let an earlier version of the
pass-through - which rejected the sampling and sent anyway - go green on every
lane.
