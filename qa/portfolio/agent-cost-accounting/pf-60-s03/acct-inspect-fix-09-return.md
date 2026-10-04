# RETURN — acct-inspect-fix-09

Allocation digest: `3690354640675e85e5a56b17ac2d5e4e395abdd95be30a3f4426289b4a6fe4a0`; claim `57fc38bd-8a36-4152-94c0-6aa5d9dc7ac3`. Model/effort: gpt-6-astra/high.
Brief SHA-256 verified: `52c215af12401192eb4b32f1d26512aee15b6cc3ba384e1a389b9c215e3a6a09`. Independent review body read as allocated.
Base: `0f170ff72c45eae99e3f6c72e147a7b94484d675`; branch `bootstrap/acct-inspect-20260915`; worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/acct-inspect-20260915`. Output is the commit containing this receipt; worker return supplies its SHA.
Classification: bounded revision within active PF-60 / in-progress PF-60-S03. Product heading **Product measurement**, excerpt “No commercial performance numbers have been supplied.” Scope is the already-authorized inspection contract and the manager's three frozen review findings.

## Result

Chose **name the line for what it is**, preserving effective coverage as aggregate retention rather than clipping it to raw-detail retention. Overview and bucket labels now explicitly say “aggregate retention coverage”; the whole-bucket sentence still states that detail availability is checked separately. An unavailable raw prefix still withholds the bucket/range total. No store behavior change remains.
Before the range total, the overview now shows the count of inspectable unknown-parent attempts excluded from that total, and, when nonzero, unavailable ancestry entries whose costs and retention coverage remain unknown. These are labelled **thread-slice entries**, not unique threads or days: an hourly range can repeat an unavailable thread across slices. Unavailable slices additionally disclose that their ancestry population is unknown; the displayed inspectable count is not represented as their population.
New ISO-week store coverage checks exactly seven day slices, exact per-day request maps and totals for attempts on two days, empty remaining days, first-day raw-detail expiry without hiding the retained second day, and aggregate expiry shrinking effective coverage and marking the bucket partial. Reads are checked against unchanged ledger rows.

## Tests and mutations

All test commands below ran from `codex-rs` through the guarded `just test`; logs are local ignored artifacts beside this receipt. No live profile or credential read, native credential prompt, full TUI suite, workspace formatter, or push was used.
- Before production edits: all **64 existing inspector tests passed**; both new UI regressions failed. The new store case was validated separately.
- `accounting_inspect_range_aggregate_coverage_does_not_claim_raw_detail`: failed before; passed after (including reviewed inline snapshot).
- `accounting_inspect_range_overview_discloses_unknown_population_before_total`: failed before; passed after (including reviewed inline snapshot).
- `accounting_inspect_range_week_day_slices_and_expired_prefix`: passing control after fixture corrections; fails both named mutants; passes in final full state package.
- Trailing-slice mutant: `day <= (upper - 1) / 86_400_000` changed to `day <= upper / 86_400_000`. Test failed with **8 slices versus 7** (exit 100).
- Unclamped-window mutant: `(lower.max(day * DAY), upper.min((day + 1) * DAY))` changed to `(lower, upper)`. Test failed because the retained second day became **DetailUnavailable instead of Ready** (exit 100).
- Both mutations restored; `git diff --exit-code -- state/src/runtime/accounting_store.rs` returned 0.
- Final state/TaskNode: **422/422 passed**, no skips, exit 0. Final focused TUI: **35/35 passed**, exit 0. Combined final result **457/457**, including all **67 inspector cases** (64 original plus 3 new).

### Exact commands and exits

Every test command redirected stdout/stderr with `> ../qa/portfolio/agent-cost-accounting/pf-60-s03/<log> 2>&1`.
| Command | Log | Exit/result |
| --- | --- | --- |
| `just test -p codex-state -p codex-tui -E 'test(accounting_inspect_)' --locked --offline` | `fix-09-baseline.log` | 100; 64 passed, 2 new UI cases failed |
| `just test -p codex-state -E 'test(accounting_inspect_range_week_day_slices_and_expired_prefix)' --locked --offline` | `fix-09-week-before.log` | 101; fixture attempted to clone a non-Clone result; changed to borrowing |
| Same command | `fix-09-week-control.log` | 100; fixture expected day 5 aggregate floor; inclusive expiry requires day 6; corrected expectation |
| `just test -p codex-state --lib -E 'test(accounting_inspect_range_week_day_slices_and_expired_prefix)' --locked --offline` | `fix-09-week-control-corrected.log` | 0; 1 passed |
| Same command, trailing-slice mutant | `fix-09-mutant-trailing-slice.log` | 100; 1 failed |
| Same command, unclamped-window mutant | `fix-09-mutant-unclamped-window.log` | 100; 1 failed |
| `just test -p codex-state -p codex-tasknode-session --locked --offline` | `fix-09-final-state-tasknode.log` | 0; 422 passed |
| `INSTA_UPDATE=no just test -p codex-tui -E 'test(accounting_inspect_)' --locked --offline` | `fix-09-final-tui.log` | 0; 35 passed |

Final scoped formatting: `rustfmt --edition 2024 --config skip_children=true state/src/runtime/accounting_store_tests.rs tui/src/chatwidget/tokens.rs tui/src/chatwidget/tokens_tests.rs`, exit 0, followed by `git status --short`, exit 0; no unallocated paths changed. Stable rustfmt warned that nightly-only imports_granularity was ignored.
From root: `python3 docs/plans/check.py`, `python3 docs/sprints/check.py`, `git diff --check`: exit 0. The first sprint update reached 101 lines and failed its 100-line cap (exit 1); removing one blank line restored compliance.

Final log SHA-256: state/TaskNode `53c4e13f1bc31400a21dba57b754a6c7a6ab2570a09f80baaeef09f03c1aaca3`; TUI `01c6cff9984bb6e019d92fc336dac48ee603e55cce586308b3dde64686d97b02`.

## Scope and handoff

Changed source lines (added/deleted): `accounting_store_tests.rs` 83/0; `tokens.rs` 28/5; `tokens_tests.rs` 71/4. Source total **191**, including **33 production**. Sprint ledger 2/1; this receipt 49/0 is the only other changed path. Entire patch **243 changed lines**, **85 outside tests** (including evidence/process text), within target 300/120.
Caps, candidate bound, collection, pricing, retention, connections, snapshots, locking, schema/indexes and range arithmetic are unchanged. No billed-versus-estimate column or behavior added.
Receiving review/integration, true-TUI keys, isolated independent code-blind design/execution/evidence review, live-repository qualification, and named-human acceptance remain open under S03. No human-test or release readiness is claimed. The sprint remains in progress; documentation here is implementation evidence only.
