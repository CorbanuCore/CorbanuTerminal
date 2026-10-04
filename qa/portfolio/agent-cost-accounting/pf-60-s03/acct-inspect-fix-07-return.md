# RETURN — acct-inspect-fix-07

- Allocation: `d589d6b411a59f7e2d3f900a62f74aabf388e6b0bf69d238aaa70a81aaf7eac0`; claim `86d8ac9a-a2a2-4266-8750-6340b52770b4`; worker gpt-6-astra, high.
- Brief SHA-256 verified before other reads: `c2bd33ad18fda68d5924e279d138dbb5511f62dd2309b7dded3abff534d9aa59`. Independent review body read.
- Launch HEAD `9f9ab0d82467454a48bf00bdd84d0cb259480388`, branch `bootstrap/acct-inspect-20260915`, worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/acct-inspect-20260915`; initially clean. Output SHA is the commit containing this receipt, returned separately.
- Class: bounded correction to active accounting plan / PF-60-S03 (`in_progress`). Exact product heading **Product measurement**, excerpt “No commercial performance numbers have been supplied.” PF-60 requires explaining run/descendant totals while distinguishing measured usage, estimates and billed amounts. Frozen launch base supersedes the older plan/sprint allocation base for this dispatch.

An unknown-ancestry candidate's non-Ready state now increments an unavailable-thread count and displays an unknown-population note; it cannot replace the root verdict or supply the root's retention coverage. Confirmed descendants retain their existing non-Ready refusal. No amount is invented for unavailable threads.

Only the candidate-count bound was added: more than 512 selected-day owners refuses before per-candidate inspection. Hoisting validation would also require refactoring the retained-day reader/retention planner, outside this correction's small scope and literal write allocation. Repeated whole-store validation below the bound remains; the bound conservatively includes unrelated owners. Existing attempt, byte, observation and ancestry caps remain unchanged.

Production inspection adds no SQL, connection, lock, transaction, schema or index; it uses the existing snapshot. Custom ranges and grouping intervals remain unimplemented. The compact regression uses synthetic compact evidence to separate attribution loss from the global age cutoff, then checks both unknown and confirmed-descendant handling.

## Regression evidence

| Test (all prefixed `accounting_inspect_`) | Before | Final |
| --- | --- | --- |
| `unknown_unavailable_preserves_root` | FAIL: NeedsRefresh replaced Ready | PASS |
| `unknown_compact_preserves_root_coverage` | FAIL: another owner's DetailUnavailable replaced Ready | PASS |
| `candidate_cap_precedes_unavailable_candidates` | FAIL: NeedsRefresh instead of TooLarge | PASS |
| `unknown_unavailable_note` | FAIL: missing note | PASS |
| `group_partial_price_stays_unknown` | PASS on correct renderer | PASS |

Pricing mutant: in the group-page renderer, replace `estimate(t.known_usd, t.unknown_estimates, t.attempts)` with `estimate(t.known_usd, 0, t.attempts)`. Result: FAIL as required (exit 100, both attempts failed on the descendant group's missing “+ unknown costs” assertion); the mutant was restored before final formatting/testing. The new fixture has an incomplete descendant/provider quote and asserts both “+ unknown costs” and “Full recorded estimate: unavailable”.

## Commands and results

Rust commands run from `codex-rs`, after reading `docs/development/test-isolation.md`, using the guarded checkout wrapper. Logs below are losslessly compressed; automatic failed-test retries remain preserved.

| Exact command | Exit / result | Log |
| --- | --- | --- |
| `shasum -a 256 /private/tmp/fmgr.Q1SIYZ/briefs/acct-inspect-fix-07.json` | 0; matches frozen hash | hash above |
| `just test -p codex-state -p codex-tui -E 'test(accounting_inspect_unknown_unavailable) \| test(accounting_inspect_candidate_cap_precedes) \| test(accounting_inspect_group_partial_price)' --locked --offline` | 100; 1 passed, 3 expected failures | [before](acct-inspect-fix-07-before.log.gz) |
| `just test -p codex-state accounting_inspect_unknown_compact_preserves_root_coverage --locked --offline` | 100; expected failure | [compact before](acct-inspect-fix-07-compact-before.log.gz) |
| `just test -p codex-tui accounting_inspect_group_partial_price_stays_unknown --locked --offline` | 100; expected mutant failure | [mutant](acct-inspect-fix-07-mutant.log.gz) |
| `rustfmt --edition 2024 --config skip_children=true state/src/runtime/accounting_store.rs state/src/runtime/accounting_lifecycle.rs state/src/runtime/accounting_store_tests.rs tui/src/chatwidget/tokens.rs tui/src/chatwidget/tokens_tests.rs` | 0 after restoration; stable-toolchain imports_granularity warning only | scoped files only |
| `just test -p codex-state -p codex-tasknode-session -p codex-tui -E 'package(codex-state) \| package(codex-tasknode-session) \| test(accounting_inspect_)' --locked --offline` | 0; 442/442 passed, 4110 filtered skips, 29.710s execution | [final](acct-inspect-fix-07-final-tests.log.gz) |
| `python3 docs/plans/check.py`; `python3 docs/sprints/check.py`; `git diff --check` (repository root) | 0 each | — |

Final tests: **335 state + 80 TaskNode-session + 27 focused TUI = 442 passed**. All **52 accounting_inspect_ tests** passed: the prior 47 (including all nine breakdown tests) plus these five. No test failure, timeout or leaky-process marker appeared. The existing descendant stale/compact and single-snapshot cases also passed.

Changed lines against the frozen base, additions/deletions:

| File | Added | Deleted |
| --- | ---: | ---: |
| `codex-rs/state/src/runtime/accounting_store.rs` | 11 | 0 |
| `codex-rs/state/src/runtime/accounting_lifecycle.rs` | 1 | 0 |
| `codex-rs/state/src/runtime/accounting_store_tests.rs` | 92 | 0 |
| `codex-rs/tui/src/chatwidget/tokens.rs` | 9 | 0 |
| `codex-rs/tui/src/chatwidget/tokens_tests.rs` | 45 | 0 |
| `docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md` | 2 | 0 |
| This receipt | 57 | 0 |
| Four `acct-inspect-fix-07-*.log.gz` evidence files | binary | binary |

Rust: **158 total / 21 non-test lines**. Including sprint/receipt text: **217 total / 80 non-test lines**, within the 250/100 target and 450/200 stop limits. `git status --short` and the final staged path audit confirm only allocated paths changed. No workspace formatter/fixer was run.

This is a correction return to Fable, not a qualified functional/human-test handoff. True-TUI keys, independent isolated code-blind execution/evidence review, live-repository qualification and named-human acceptance remain open in PF-60-S03; no gate is marked passed here. No native credential prompt was observed. No live profile, credential read, full TUI suite, workspace formatter, push or release was used.
