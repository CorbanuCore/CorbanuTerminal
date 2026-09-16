# RETURN — acct-inspect-breakdown-06

- Allocation digest: `921c5284f691aa5a8126f6bff21a0b3a885887028a17dabb75750c3c29fc2fde`.
- Claim: `d20a573b-8890-4926-8eb1-da0f52335462`; worker: gpt-6-astra, high.
- Frozen brief SHA-256 verified: `e4122e0f338c73d5d7a3d556dda84c737c54043543b5659a0243b6f3d0a6db99`.
- Launch base: `eebfa11c495a6e12d5d2f969f13b5cc0ddac5ad9`; branch `bootstrap/acct-inspect-20260915`; worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/acct-inspect-20260915`. Plan/sprint front matter retains the earlier allocation base; this user-dispatched action explicitly supplied the rebased launch coordinates. No out-of-scope plan edit was made.
- Class: product-initiative increment, existing active accounting plan, PF-60-S03 in_progress. Product heading **Product measurement**, excerpt “No commercial performance numbers have been supplied.” PF-60 outcome: inspect a run and descendants while distinguishing measured usage, estimates and billed amounts.
- Output commit: the commit containing this receipt (full SHA returned to Fable).

## Rendered views

| View | Shows | Refuses to imply |
| --- | --- | --- |
| Recorded requests — root and descendants | One UTC day's resolved root total, measured known/unknown counts, exact known USD, coverage/checkpoint, request links, unknown-parent count | Complete collection, lifetime totals, billed cost, inclusion of unresolved ancestry |
| Root's own attempts | Root-owned attempt count and exact/partial estimate, constituent request links | Inclusion of descendants or unknown parents |
| Descendant attempts | Unique attempts in resolved spawned descendants, including closed edges and multiple generations, exact/partial estimate and request links | Inclusive-parent subtotal addition or unresolved membership |
| Provider/model groups | Disjoint provider/model pairs over the same resolved root population, exact/partial estimates and request links; absent attribution uses an explicit unknown label | Dropping unattributed attempts or summing independently rounded labels |
| Unknown provider/model attribution, empty | No recorded attempts in that bucket, coverage unknown | A zero-dollar estimate for an empty population |
| Unknown parent population | Separately counted and estimated same-day attempts with orphan, cyclic, conflicting or unavailable ancestry; attempt drill-down | Membership in this root; these may belong to unrelated runs and are excluded from its total |
| Logical request / attempt | Existing IDs, token evidence, original price/components and retry provenance; attempt estimates and billing unavailability | Cross-day request completeness, replacement pricing or authoritative billing |
| Unavailable states | Existing checkpoint-lag, stale-contribution, compact/detail-expired, missing-store/thread, corruption and cap diagnostics | Any partial amount or navigation into an unavailable total |

All ready pages carry the checkpoint-lag warning when applicable. Group pages reuse the packet's read time and half-open UTC bounds. Estimate and billed labels remain separate; the difference explicitly remains unknown without settlement evidence.

## Limits and unfinished requirements

The installed ledger has **no billed amount, settlement identity or request-to-invoice join**. No positive billed-amount/difference path is implemented or tested; adding pretend settlement data would misrepresent retained evidence and changing its acquisition/schema is outside this allocation. Estimates are never added to billed amounts. The manager must allocate an authoritative billed-evidence seam before positive billing comparison can be completed.

The ledger currently rejects empty provider/model identity strings. Native inspection preserves that rejection. The defensive unknown-attribution renderer is tested with a typed boundary packet, not claimed as proof that missing-identity rows are admissible through native persistence. An empty unknown bucket remains visible for ordinary validated packets.

The original 10,000 retained-row, 4 MiB accounting-input, 512-attempt and 4,096-observation checks remain unchanged. The combined tree plus unknown-parent packet now shares the 512-attempt and 4 MiB packet budget; visited ancestry is additionally bounded to 10,000 entries and counts toward packet bytes. These checks can refuse a root day sooner than single-owner inspection. Unknown-parent history may also make the whole inspection unavailable. Reusing the whole-store validator per participating run can cost more time; no cap or timeout was raised.

Reconciliation: resolved root = own + descendants, and provider/model pairs partition that same population exactly before display rounding. Unknown-parent amounts deliberately do **not** reconcile into the root, because membership is unknown. Known subtotals do **not** reconcile to a full cost when rates/usage are missing; the full estimate remains unavailable. There is no billed total to reconcile. Rounded display fragments need not sum; exact USD remains available.

This is an implementation return to Fable, **not** a human-test-ready or accepted candidate. True-TUI keys, isolated independent code-blind design/execution/evidence review, live-repository qualification and named-human acceptance remain open in S03. No release, push, collection activation, native credential access or live profile use was performed.

## Validation

- Before tests, read `docs/development/test-isolation.md`; all Rust runs use checkout `just test`.
- Scoped formatting only: from `codex-rs`, `rustfmt --edition 2024 --config skip_children=true state/src/runtime/accounting_store.rs state/src/runtime/accounting_lifecycle.rs state/src/runtime/accounting_store_tests.rs tui/src/chatwidget/tokens.rs tui/src/chatwidget/tokens_tests.rs`: exit 0. Stable rustfmt warned that repository `imports_granularity` needs nightly; no workspace formatter/fixer was run. Status checks found only allocated edits.
- Initial `just test -p codex-state -p codex-tui accounting_inspect_ --locked --offline`: exit 100; 46 tests, 44 passed (one leaky marker), one failed and one timed out, 4420 filtered skips. The stale fixture violated a foreign key before deletion-order correction; the cap fixture timed out twice while seeding via repeated maintenance writes, replaced by a single fixture transaction. Raw attempts are preserved. This build ran while fixtures were being finalized and is not final-tree evidence.
- Final command from `codex-rs`: `just test -p codex-state -p codex-tasknode-session -p codex-tui -E 'package(codex-state) | package(codex-tasknode-session) | test(accounting_inspect_)' --locked --offline`: exit 0; **437/437 passed**, 4110 filtered skips, test execution 28.902s. Full state 332, full TaskNode-session 80, focused TUI 25. All **47 accounting_inspect_** tests passed, including the original 21 TUI cases and all nine new cases. Two existing tests received leaky-process markers: `extract::tests::turn_context_does_not_override_session_cwd` and `model::thread_metadata::tests::thread_row_preserves_model_defined_reasoning_effort_values`. No full TUI suite was run.
- Raw logs: [initial failed attempts](acct-inspect-breakdown-06-focused-initial.log.gz) and [final passing run](acct-inspect-breakdown-06-final-tests.log.gz). The initial leaky marker was `accounting_inspect_public_reopens_twice`; it passed without that marker in the final run. No native credential prompt was observed.
- `python3 docs/plans/check.py`: exit 0 (3/3 active); `python3 docs/sprints/check.py`: exit 0 (115 current, 127 archived); `git diff --check`: exit 0.

New tests — all PASS on the final tree:

- `accounting_inspect_tree_reconciles_and_quarantines_unknown_parents`
- `accounting_inspect_conflicting_and_missing_edges_are_unknown`
- `accounting_inspect_descendant_stale_and_compact_refuse_tree_total`
- `accounting_inspect_tree_lineage_and_cost_share_one_snapshot`
- `accounting_inspect_tree_combined_attempt_cap_is_not_per_run`
- `accounting_inspect_rendered_tree_and_provider_model_reconcile`
- `accounting_inspect_estimate_only_never_invents_billed_or_difference`
- `accounting_inspect_breakdown_freshness_and_empty_attribution`
- `accounting_inspect_breakdown_navigation_reuses_packet`

Changed lines against the frozen launch base (additions/deletions):

| File | Added | Deleted |
| --- | ---: | ---: |
| `codex-rs/state/src/runtime/accounting_lifecycle.rs` | 15 | 0 |
| `codex-rs/state/src/runtime/accounting_store.rs` | 150 | 1 |
| `codex-rs/state/src/runtime/accounting_store_tests.rs` | 206 | 0 |
| `codex-rs/tui/src/chatwidget/tokens.rs` | 157 | 3 |
| `codex-rs/tui/src/chatwidget/tokens_tests.rs` | 202 | 3 |
| `docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md` | 3 | 1 |
| This receipt | 72 | 0 |
| `acct-inspect-breakdown-06-focused-initial.log.gz` | binary | binary |
| `acct-inspect-breakdown-06-final-tests.log.gz` | binary | binary |

Rust change: **737 total lines / 326 non-test lines**; including sprint/receipt text: **813 / 402**, below both stop ceilings (non-test target exceeded by two documentation lines). All changed files are allocated. No collection-path changes, schema changes, custom ranges, grouping intervals, workspace-wide formatting or pushes.
