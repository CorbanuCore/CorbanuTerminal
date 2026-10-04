# RETURN — acct-inspect-fix-04

Status: scoped P2 correction verified for receiving review. All 34 original named tests pass, plus both new regressions. No functional or human-test qualification claimed. The final action RETURN supplies the commit SHA.

## Provenance and authority

- Worker: gpt-6-astra, high; action acct-inspect-fix-04.
- Allocation digest: 28be5ee8a93f7e756ed7027c9120e99f849909f9d9be43b282b85d5097e4c577.
- Claim: a50186dc-3af1-4258-bf6b-d0abb9c8b1e6.
- Frozen brief: /private/tmp/fmgr.Q1SIYZ/briefs/acct-inspect-fix-04.json.
- Verified SHA-256: 761648d2bd0049bb56fabf84ccfad7b30f8e9640658ca0ffc747dd340bfa518c.
- Clean launch HEAD: ddee507164ae2d0ee395a596e1b926eaaef1e0ea.
- Branch: bootstrap/acct-inspect-20260915.
- Worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/acct-inspect-20260915.
- Read the independent review at /private/tmp/fmgr.Q1SIYZ/acct03-review-body.txt; only its single P2 is addressed.
- Change class: bounded fix within active plan docs/plans/active/portfolio-agent-cost-accounting.md and in-progress sprint PF-60-S03.
- Exact product-spec heading: **Measurement targets**, docs/corbanu-product-spec.md. Requirement excerpt: “The following metrics must be instrumented, with targets set through the decision rights defined above.”

## Correction

The inspector now returns CheckpointLag for a requested day beyond the stored checkpoint, an unmaintained installation, or a retained reader that requires activation/maintenance. A stale contribution or estimate still returns NeedsRefresh. The read transaction, collection path, schema, migrations, validation, and reconciliation are unchanged.

Exact rendered cases: installed/current empty day → “No recorded attempts in this day; collection coverage unknown.”; checkpoint lag → “Snapshot is not current; newer activity is unverified”; genuinely stale contribution/estimate → “Recorded totals unavailable — stored contributions need refresh. Retry rereads only; no repair performed.”

The empty-day text above is the unchanged existing wording for the frozen contract's empty-range case. A beyond-checkpoint empty day uses the separate lag wording, rather than claiming current coverage.

## Regression and checks

- New regression: accounting_inspect_empty_day_after_checkpoint_renders_lag. It opens a synthetic ledger at UTC day 0, verifies the empty current-day view and checkpoint, then reads empty UTC day 1 twice through the actual store and renderer. It requires lag wording, rejects stale-contribution wording, and rejects any displayed dollar amount.
- Before correction: exit 100, 0/1 passed; both nextest attempts failed with the stale-contribution warning. [Raw failure](fix-04-before.log).
- Additional new state case: accounting_inspect_never_maintained_is_checkpoint_lag. A synthetic staging checkpoint returns lag without changing any retained/native table.
- Existing accounting_inspect_checkpoint_and_stale_estimate now also checks beyond-checkpoint lag and unchanged tables; its two genuine stale-evidence assertions remain.
- Existing accounting_inspect_availability_state_snapshots preserves the refresh-warning snapshot and adds the distinct lag copy.
- All Rust checks use this checkout's guarded just test after reading docs/development/test-isolation.md. Only changed Rust files were formatted using rustfmt --edition 2024 --config skip_children=true. No workspace formatter, raw cargo test/nextest, live profile, credential reads, subagents, or push.
- After correction: the new renderer regression passed in the 19/19 focused TUI run (exit 0). The new staging case and both genuine stale-evidence assertions passed in the 407/407 state/TaskNode run (exit 0). Neither final run reported retries, failed cases, or leaky markers.
- Matched every name from impl-03-case-map.json to a passing final log entry: 34/34 original cases passed, none missing.
- Source delta: 87 additions / 5 deletions (92 changed lines). accounting_lifecycle.rs +6/-5; accounting_store.rs +2/-0; accounting_store_tests.rs +25/-0; tokens.rs +1/-0; tokens_tests.rs +53/-0. tokens.rs is 798 physical lines.
- No native credential prompt or live-profile access was observed. The wrapper reported its disposable profile and native-keyring denial.
- Full TUI suite was not rerun, as directed. Prior full-suite failures and the manager's attribution remain preserved in earlier evidence; this correction does not claim to resolve them.

All test commands below ran from codex-rs after scoped formatting; the before run formatted only the new regression and preceded all production changes.

| Exact command | Exit | Result / raw log |
| --- | ---: | --- |
| `just test -p codex-tui --lib accounting_inspect_empty_day_after_checkpoint_renders_lag --locked --offline` | 100 | 0/1 passed; expected before-fix failure, including nextest retry; [before log](fix-04-before.log) |
| `just test -p codex-tui --lib accounting_inspect_ --locked --offline` | 0 | 19/19 passed, 4019 skipped; [final TUI log](fix-04-tui-final.log) |
| `just test -p codex-state -p codex-tasknode-session --locked --offline` | 0 | 407/407 passed, 0 skipped; [final state/TaskNode log](fix-04-state-tasknode-final.log) |

Final TUI run: a509d1a2-8011-469d-9a61-0c7ccd02c54f. Final state/TaskNode run: d7245a4f-a650-46ab-8c26-bed7fdbd9691. Logs preserve warnings and the original failed attempts verbatim.

Governance: python3 docs/plans/check.py and python3 docs/sprints/check.py passed at launch and after the sprint ledger update (each exit 0; active plans 3/3, current sprints 115, archived 127). git diff --check passed (exit 0). Final status inspection confirms only the allocated Rust files, sprint record, and QA evidence changed.

## Remaining qualification

This is a correction return to Fable for receiving/integration, not an unqualified human-test handoff. The sprint stays in_progress. Its existing true-TUI, isolated independent code-blind design/execution/evidence review, live-repository evidence and human acceptance obligations remain open. No release, packaged-candidate, benchmark, or functional acceptance claim is made.
