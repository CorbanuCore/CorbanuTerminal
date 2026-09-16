# RETURN — acct-inspect-fix-05

P3 fixed and verified for Fable's receiving review. Final commit SHA is supplied in the action RETURN; Fable will move the sprint's disclosed finding from Remaining. No functional/human-test readiness claimed.
Class: bounded fix. Product specification heading **Measurement targets**: “The following metrics must be instrumented, with targets set through the decision rights defined above.” Active plan: docs/plans/active/portfolio-agent-cost-accounting.md; sprint PF-60-S03 remains in_progress.
Provenance: gpt-6-astra/high; allocation digest 2b3feaa44ae972a19a44ea8de2fbaac29db9ef05c98b7e5b68987ea5d0d35482; claim 974819f5-7223-46a3-96ce-21abb00685c6. Brief /private/tmp/fmgr.Q1SIYZ/briefs/acct-inspect-fix-05.json verified SHA-256 ad9ea8afb96db329981d1a9d697cc141a6d86a63b99f94a088978741606c62fb.
Clean launch: 5ee737b874752d59df158833f935f3cdda31e30e; branch bootstrap/acct-inspect-20260915; worktree /Volumes/CorbanuDrive/Corbanu/worktrees/acct-inspect-20260915.
Only the inspector's NeedsMaintenance branch gains a read-only freshness check for selected-day raw contributions still inside retention. No collection, schema, migration, public API or persistent-storage change.

Current empty day → “No recorded attempts in this day; collection coverage unknown.”
Maintenance with healthy raw totals → “Snapshot is not current; newer activity is unverified”.
Stale raw totals, including simultaneous maintenance → “Recorded totals unavailable — stored contributions need refresh. Retry rereads only; no repair performed.”

| New test | Valid before-fix run | Final run |
| --- | --- | --- |
| accounting_inspect_maintenance_with_stale_raw_renders_refresh | FAIL: actual lag wording, expected refresh wording; both nextest attempts | PASS: missing contribution and missing estimate cases |
| accounting_inspect_maintenance_with_healthy_raw_renders_lag | PASS | PASS |

Rust commands ran from codex-rs through the guarded wrapper after reading docs/development/test-isolation.md. Table commands show the executed test/check invocation; test stdout/stderr were redirected with `> ../qa/portfolio/agent-cost-accounting/pf-60-s03/<linked-log-name> 2>&1`.
| Exact invocation | Exit | Evidence/result |
| --- | ---: | --- |
| `INSTA_UPDATE=no just test -p codex-tui --lib accounting_inspect_maintenance_with_ --locked --offline` | 100 each, three runs | [Initial fixture failure](fix-05-before.log), [second fixture failure](fix-05-before-corrected-fixture.log), [valid before-fix reproduction](fix-05-before-valid-fixture.log): healthy PASS, stale FAIL |
| `INSTA_UPDATE=no just test -p codex-tui --lib accounting_inspect_ --locked --offline` | 100, then 0 | [First post-fix run](fix-05-tui-final.log): 20/21, repeated-inline-snapshot restriction; [final replay](fix-05-tui-final-replay.log): 21/21, all 19 existing cases included |
| `just test -p codex-state -p codex-tasknode-session --locked --offline` | 0 | [Final state/TaskNode](fix-05-state-tasknode-final.log): 407/407 |
| `rustfmt --edition 2024 --config skip_children=true state/src/runtime/accounting_lifecycle.rs tui/src/chatwidget/tokens_tests.rs` | 0 | Final scoped formatting before final tests; stable-toolchain imports_granularity warnings only. Earlier tests-only formatting used the same command with only tui/src/chatwidget/tokens_tests.rs. |
| `python3 docs/plans/check.py` and `python3 docs/sprints/check.py` (repository root) | 0 each | Active plans 3/3; current sprints 115, archived 127 |
| `git diff --check` and `git status --short` (repository root) | 0 each | No whitespace defects or changes outside assigned scope |
| `shasum -a 256 /private/tmp/fmgr.Q1SIYZ/briefs/acct-inspect-fix-05.json` | 0 | Exact expected digest matched before any other file read |

Original attempts are preserved: the first two runs failed because the public writer rejects over-age raw imports. The corrected disposable fixture models legacy raw evidence with Python's standard sqlite3 module, validates NeedsMaintenance through the actual retained reader, and rereads the real inspector before passing its result to the actual renderer. Snapshot context lines were corrected before the valid baseline. The first post-fix run reached the second stale case and exposed Insta's duplicate-inline restriction; allow_duplicates changes no expected text.
Final TUI run ID fb251ecd-752b-4746-8977-af653af6f307; state/TaskNode b15e32e3-d102-4ef2-a31b-57d1193ac84b. Neither final run reported retries, failures or leaky markers. Disposable profiles/native-keyring denial reported; no native credential prompt or live-profile access observed.
Source/test delta: accounting_lifecycle.rs +19/-1 (20); tokens_tests.rs +96/-0 (96): 116 changed lines, 20 production lines. This receipt adds 34 lines. Generated raw logs separately: before +630; before-corrected-fixture +630; before-valid-fixture +636; tui-final +660; tui-final-replay +583; state-tasknode-final +655 (all deletions 0).
No workspace formatter, full TUI suite, raw cargo test/nextest, credential reads, subagents or push. Existing full-suite attribution is unchanged.
Remaining: manager receiving/integration and the sprint's true-TUI, isolated independent code-blind execution/evidence review, live-repository and human-acceptance gates. This correction does not discharge them or claim release/benchmark qualification.
Changed files are only the two allocated Rust files and this QA directory; sprint ledger intentionally left to Fable per frozen brief.
