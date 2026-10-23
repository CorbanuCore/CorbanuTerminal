# RETURN — acct-inspect-ranges-08

Allocation digest: `ab7027da661bd2b7345f6d417c870ff0b458ef71b204b14a397b5cda8e88be10`; claim `74017647-3185-4b79-ac93-4887cdcd1f2e`. Brief SHA-256 verified: `73baf9e9309134ec0c351882d6515b37a4ffc298c697aec41a57694379101a5e`.
Base: `0814fea63dac68e9fe393e6fc388aced3bf2619c`; branch `bootstrap/acct-inspect-20260915`; worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/acct-inspect-20260915`. Candidate is the commit containing this receipt; worker return supplies its SHA.
Product initiative, active PF-60 / in-progress PF-60-S03. Product heading **Product measurement**, excerpt “No commercial performance numbers have been supplied.” Existing plan and manager-frozen UTC alignment govern this increment. Large existing modules were extended only because the allocation freezes their literal paths.

Control: `/usage requests START END hour|day|week|month`. Dates mean UTC midnight; timestamps require Z and millisecond-or-coarser precision. Bounds are half-open. Invalid, reversed and zero-width ranges produce distinct refusal reasons. Existing single-day controls remain available.
- Hour: aligned UTC hours with retained raw detail, including a raw suffix on a day with compacted earlier requests; refuses expired precision or partial-hour totals.
- Day: UTC midnight to midnight; refuses partial-day or unavailable-attribution totals.
- Week: ISO Monday to Monday, including month/year crossings; refuses partial or unavailable constituent-day totals.
- Month: UTC calendar month, including leap February and December/January; refuses partial or unavailable constituent-day totals.
Every range page names the requested interval, timezone and grouping; bucket pages name full boundaries and effective retained time coverage. The overview enumerates each bucket's effective coverage. Coverage intersects requested bounds, aggregate retention floor and completed checkpoint (inclusive millisecond represented as an exclusive end); it is not a collection-completeness or attribution claim. Missing detail is separately explained. Loading/errors disclose pending/unavailable coverage.
Oldest compact aggregate day is explicitly ledger-wide and separate from the wall-clock 90-day cutoff. Unsupported older precision offers concrete whole-UTC-day bounds without rounding the request or restoring attribution.
All range and ancestry reads share the existing deferred snapshot; no added connection, transaction, lock, schema, index, collection call, write, repair or repricing. Exact arithmetic, unknown estimates, missing-rate versus unpriced, provider/model and root/descendant breakdowns remain inherited. Unavailable unknown-parent counts over multi-day buckets are labelled thread-day entries.

## Verification
- Initial `just test -p codex-state -p codex-tui -E 'test(accounting_inspect_)' --locked --offline`: exit 0; all 52 existing cases passed.
- Final state `just test -p codex-state -p codex-tasknode-session --locked --offline`: exit 0; 421/421 passed, zero skips.
- Final focused TUI `just test -p codex-tui -E 'test(accounting_inspect_)' --locked --offline`: exit 0; 33/33 passed (4,110 intentionally filtered). Both focused runs passed. Combined final package/focused result: 454/454, including all 52 original inspector cases and 12 new range cases.
- From `codex-rs`: `rustfmt --edition 2024 --config skip_children=true state/src/runtime/accounting_store.rs state/src/runtime/accounting_store_tests.rs state/src/runtime/accounting_lifecycle.rs tui/src/chatwidget/tokens.rs tui/src/chatwidget/tokens_tests.rs tui/src/chatwidget/tests/usage.rs tui/src/app_event.rs tui/src/app/event_dispatch.rs tui/src/app/tests.rs`: exit 0, followed by scoped status inspection; no workspace formatter.
- `python3 docs/plans/check.py`, `python3 docs/sprints/check.py`, `git diff --check`: exit 0 after sprint ledger updates; final commit check repeated.
Raw local logs: `ranges-08-initial-tests.log`, `ranges-08-state-tasknode.log`, `ranges-08-focused-tui.log`, `ranges-08-final-tui.log` beside this receipt. They are ignored build artifacts, retained locally rather than inflated into the source diff. Final TUI log SHA-256: `e074e3b480cc55eecf48484ea67d12be90105d37bcd90703580b187ffefa11b0`; state/TaskNode log SHA-256: `c8d4edffba99a19476484bd7de7e4bf22a873a3b9f33f9dfb217d39745f5f1c2`.

New cases (all prefixed `accounting_inspect_range_`):
- `invalid_empty_reversed_and_unavailable`: passed.
- `iso_week_calendar_month_boundaries`: passed.
- `hours_half_open_and_cutoff_suffix`: passed.
- `partial_edges_and_checkpoint_coverage`: passed.
- `mixed_compaction_exact_no_double_count`: passed.
- `tree_unknown_and_single_snapshot`: passed.
- `partial_no_amount_and_explicit_coverage`, `breakdowns_and_navigation_reconcile`, `unavailable_precision_and_freshness`, `command_refresh_and_refusal`, `week_and_month_merge_exact_attempts`, `app_dispatch_preserves_query`: all passed.

Changed source lines (added/deleted): `state/src/runtime/accounting_store.rs` 231/3; `accounting_lifecycle.rs` 20/3; `accounting_store_tests.rs` 298/0; `tui/src/app/event_dispatch.rs` 18/8; `tui/src/app/tests.rs` 33/0; `tui/src/app_event.rs` 1/0; `tui/src/chatwidget/tokens.rs` 282/4; `tokens_tests.rs` 253/0; `tests/usage.rs` 1/0. Source total 1,155 changed lines, 570 non-test; below STOP 1,400/650, above target 900/400. Sprint record 5/3; this new receipt is the only other changed path.

## Limits and reconciliation
No numerical mismatch observed. A mixed compact/raw range deliberately has no full range total: compacted days lost attribution. Complete raw buckets keep exact independent sums; a week/month containing unavailable or partial days has no bucket subtotal. Partial requested boundary buckets and checkpoint/retention-clipped buckets are excluded, never prorated.
The inherited store-wide 10,000-row / 4 MiB / 512-attempt / 4,096-observation checks and per-day 512-candidate bound are unchanged. Unrelated history can still refuse a small request. Range materialization additionally accounts against the same 512-attempt and 4 MiB ceilings across its disjoint slices. Each visited day slice reserves 8,192 bytes, so even an empty hourly request beyond 512 hours (21 days 8 hours) is refused; quote bytes reduce that bound. Day/week/month visits beyond 512 day slices likewise refuse. These are conservative refusals, never truncation.
Ranges repeat the inherited whole-store/ancestry validation for each bucket/day intersection (24 times per day for hour grouping). The existing 15-second app timeout can therefore refuse a range sooner. No cap, retention or candidate-bound rework was attempted.
Independent review, isolated code-blind design/execution/evidence review, true-TUI keys, live-repository qualification and named-human acceptance remain OPEN under S03. This is an implementation return, not human-test or release readiness. No push or live profile use.
