# RETURN — acct-inspect-impl-02 (stopped, incomplete)

Action: acct-inspect-impl-02
Allocation digest: 2054adf644713c38f3a6c1fd11e6933b9744578eb9061f487734d4cec97b6c02
Claim: 66257291-bd92-47f7-85d2-499e4af56fa0
Worker: gpt-6-astra / high
Worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/acct-inspect-20260915
Branch: bootstrap/acct-inspect-20260915
Verified launch HEAD: 41dd8c5ae84855f75434b5cbb22834c6413f26a4
Verified brief: /private/tmp/fmgr.Q1SIYZ/briefs/acct-inspect-impl-02.json
SHA-256: bfe98aca0cff675d271b99507eaded975151fe951915ee21e9be36a6584d16de

The brief bytes matched. Its internal allocation property still says
acct-inspect-impl-01; this return uses the explicit dispatch ID impl-02.
The launch tree was clean. Plan/sprint coordinates, in_progress status,
archived S02 dependency and PF-83 overlap release were verified.
This was implementation under the existing PF-60 product initiative, not
authority for a new collection flow. Product heading: **Measurement targets**;
excerpt: “No commercial performance numbers have been supplied. The following
metrics must be instrumented, with targets set through the decision rights
defined above.” Plan: docs/plans/active/portfolio-agent-cost-accounting.md;
sprint: PF-60-S03, still in_progress and incomplete.

## STOP-FMT and disposition

The dispatch says STOP “on any additional path.” The required `just fmt`
ran the repository-wide formatter and changed 74 unallocated paths in addition
to the draft source edits. The observed high-water diff was 85 source/process
paths and 24,159 additions + deletions. The full observed numstat/path list is
[format-scope-stop](acct-inspect-impl-02-format-scope-stop.txt).
These transient changes exceeded scope and total size; they were not approved
or hidden as an allowed formatting exception.

Implementation stopped on detecting that output. The 74 formatter-only files
were restored to the clean launch versions. The still-running scoped fix
command was interrupted via its command session (not by PID); it exited 130.
No Rust tests had been dispatched. No test binary, live profile, credential
read, native credential prompt, provider call or push occurred.

Worker process error: broad formatting was launched while `just fix` was still
compiling. The required sequential fix → format → final tests order was not
completed. This return does not present either tool as final-tree qualification.

All 11 draft source files were then restored to launch HEAD. The exact draft is
preserved only as [unvalidated.patch](acct-inspect-impl-02-unvalidated.patch).
It is **not compiled, tested, reviewed, accepted, enabled or integrated**.
No state-only substitute is delivered. The final source tree has no runtime
behavior changes; the commit contains only this return, the draft artifact,
scope evidence and an honest sprint ledger update.

Fable owns redispatch of the complete frozen vertical unit with a scope-safe
fix/format procedure. Recommend formatting only the allocated files under an
explicit bounded procedure, then running the full required guarded test commands.
Do not apply the patch as a completed feature.

## Draft manifest and conservative size

The discarded draft changed 11 of the 21 allocated source/test paths:
725 additions + 3 deletions = **728 total / 728 non-test** lines.
No test or snapshot file was edited. Target was 2,500 total / 1,000 non-test;
STOP was 2,800 / 1,150 or any additional source path.
The draft alone was below both numeric ceilings, but the actual broad formatter
diff crossed the path and total ceilings. Restoring it does not erase that event.

| Draft path | Added | Deleted | Changed |
| --- | ---: | ---: | ---: |
| `codex-rs/state/src/runtime/accounting_estimates.rs` | 11 | 0 | 11 |
| `codex-rs/state/src/runtime/accounting_lifecycle.rs` | 127 | 0 | 127 |
| `codex-rs/state/src/runtime/accounting_pricing.rs` | 1 | 1 | 2 |
| `codex-rs/state/src/runtime/accounting_store.rs` | 59 | 0 | 59 |
| `codex-rs/tui/src/app/event_dispatch.rs` | 32 | 0 | 32 |
| `codex-rs/tui/src/app_event.rs` | 25 | 0 | 25 |
| `codex-rs/tui/src/chatwidget.rs` | 1 | 0 | 1 |
| `codex-rs/tui/src/chatwidget/constructor.rs` | 1 | 0 | 1 |
| `codex-rs/tui/src/chatwidget/slash_dispatch.rs` | 4 | 2 | 6 |
| `codex-rs/tui/src/chatwidget/tokens.rs` | 451 | 0 | 451 |
| `codex-rs/tui/src/chatwidget/usage.rs` | 13 | 0 | 13 |

The preserved patch itself has 866 artifact lines, including diff context;
the scope evidence has 172 lines. These are separately disclosed evidence
deltas, not production source and not a means to hide the 728-line draft.
Final committed path counts are supplied in the action's final return.

Draft patch SHA-256:
`41ac1ecc86c3744b8d8dd9c8b570ce945ece6a61d6b4b573adeb9822d930d01e`.
Empty context-line prefixes were stripped for artifact whitespace hygiene;
`git apply --check` exited 0 without applying it. The first staged whitespace
check exited 2 for Markdown hard-break spaces and patch context spaces; those
were removed. No code content changed during that normalization.
The return commit identifies the exact evidence tree; no candidate binary exists.

## Requested rendering explanation — source intent only, not observed behavior

The restored runtime has no new `/usage requests` command. In the retained draft:

- Partial money text is `Known estimated token cost: $0.001210 + unknown costs`,
  immediately followed in the text sequence by
  `Full recorded estimate: unavailable (1 of 2 attempts incomplete)`.
  If the known subtotal is zero, the leading line is
  `Estimated token cost: unknown`; it does not headline zero as the full estimate.
- A missing measured component reads
  `Cache write: unknown — no retained numeric evidence`.
  Aggregate metric copy is `Input: 100 known + unknown in 1 attempts`
  (grammatical singular handling remains unfinished).
- Missing price reads
  `Price: unavailable — no dispatch-time price snapshot`.
  Missing bucket rate reads `unknown — rate unavailable`.
  Existing exact decimal serialization and half-even display are reused;
  rounded/sub-micro markers are retained.
- Compacted participation yields no total or request list. Draft copy begins
  `Request explanation unavailable — compacted history lost request/provider attribution`
  and says `No partial-day total shown`, with checkpoint/retention metadata.
  No attribution is invented from surviving snapshot IDs.

These are intended strings in an unvalidated artifact, not a UI demonstration.
No screenshot, narrow-screen key run, native-store query or numeric golden
establishes their correctness.

## Known unfinished draft issues for receiving triage

- All 34 frozen tests and four existing snapshot updates are missing.
- The whole-store preflight uses a conservative 4 MiB stored-input ceiling
  and applies the observation bound to all attempts. That is stricter than the
  frozen selected-result limits; exact accepted/rejected boundaries need work.
  Tombstone/reference materialization and projected-result overhead also need
  review. Do not claim the query allocation is satisfied.
- Snapshot navigation does not preserve the previously selected list row.
  Cancellation, queued navigation, account/profile transitions, thread clearing
  and retained-view disposal require the specified lifecycle tests and fixes.
- Detail zero copy still conflates reported and derived zero; report each metric
  precisely. Header dates are epoch milliseconds rather than the intended
  human-readable UTC boundaries. Retention metadata for unavailable states needs
  completeness checking. These are unfinished implementation details.
- Collection mode, migrations, dependencies, manifests and credential code
  were never edited. This is source inspection, not test proof of OFF behavior.

## Commands, exit codes and counts

Read docs/development/test-isolation.md before any test. No raw cargo test or
nextest command was run. No Rust test functions were executed; passes/failures/
skips/leaky counts are **not measured**, not “0 passed” success.

| Command | Exit | Observed result |
| --- | --- | --- |
| `shasum -a 256 /private/tmp/fmgr.Q1SIYZ/briefs/acct-inspect-impl-02.json` | 0 | Exact expected digest |
| `git rev-parse HEAD`; `git branch --show-current`; `git status --short` | 0 | Exact base/branch; clean launch |
| `python3 docs/sprints/check.py` (launch) | 0 | current 115; archived 127 |
| `python3 docs/plans/check.py` (launch) | 0 | active 3/3; available 0 |
| `just fix -p codex-state -p codex-tui --locked --offline` from codex-rs | 130 | Interrupted compilation after scope STOP; no final result |
| `just fmt` from codex-rs | 0 | Out-of-scope formatting incident; not an accepted pass |
| `git diff --check` after restoring unrelated formatter edits | 0 | Whitespace only; no functional claim |

Required Rust commands **not run** (exit N/A; executed count 0):
```sh
just test -p codex-state --lib accounting_inspect_ --locked --offline
just test -p codex-state --test accounting_store accounting_inspect_ --locked --offline
just test -p codex-tui --lib accounting_inspect_ --locked --offline
just test -p codex-state -p codex-tasknode-session --locked --offline
just test -p codex-tui --locked --offline
```

Final evidence-tree governance check: `python3 docs/plans/check.py && python3 docs/sprints/check.py && git diff --check` exited 0 (active 3/3; current 115, archived 127). This checks records, not the discarded feature.

## All 34 frozen cases — explicit no-evidence map

STOP-FMT is the scope incident above. Every case remains an implementation and
execution obligation; none is waived, covered by another count or passed.
Expected future new-function counts remain 10 private state + 6 external state
+ 12 widget + 6 app. Raw test outputs/JUnit/binary digests do not exist for this run.

| # | Frozen test | Evidence/disposition |
| ---: | --- | --- |
| 1 | `accounting_inspect_absent_schema_is_read_only` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 2 | `accounting_inspect_schema_rejection_matrix` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 3 | `accounting_inspect_raw_day_reconciles_contributions` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 4 | `accounting_inspect_original_null_and_price_are_immutable` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 5 | `accounting_inspect_missing_binding_is_not_unpriced` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 6 | `accounting_inspect_checkpoint_and_stale_estimate` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 7 | `accounting_inspect_retention_and_compact_matrix` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 8 | `accounting_inspect_half_open_date_bounds` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 9 | `accounting_inspect_corruption_and_unknowns` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 10 | `accounting_inspect_single_snapshot_concurrent_writer` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 11 | `accounting_inspect_public_reopens_twice` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 12 | `accounting_inspect_public_reads_do_not_write` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 13 | `accounting_inspect_public_identity_retry_scope` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 14 | `accounting_inspect_public_literal_partial_goldens` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 15 | `accounting_inspect_public_delete_and_empty` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 16 | `accounting_inspect_public_limits_no_truncation` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 17 | `accounting_inspect_partial_and_unknown_copy` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 18 | `accounting_inspect_unpriced_zero_and_missing_rate` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 19 | `accounting_inspect_exact_rounding_reconciliation` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 20 | `accounting_inspect_request_attempt_price_detail` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 21 | `accounting_inspect_narrow_and_long_fields` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 22 | `accounting_inspect_availability_state_snapshots` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 23 | `accounting_inspect_coverage_never_claims_run_complete` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 24 | `accounting_inspect_snapshot_navigation_roundtrip` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 25 | `accounting_inspect_menu_and_reset_regression` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 26 | `accounting_inspect_cancel_refresh_generation` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 27 | `accounting_inspect_command_without_account_auth` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 28 | `accounting_inspect_command_date_validation` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 29 | `accounting_inspect_app_real_store_to_view` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 30 | `accounting_inspect_app_remote_does_not_read_local` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 31 | `accounting_inspect_app_thread_switch_stale_reply` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 32 | `accounting_inspect_app_error_retry_and_timeout` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 33 | `accounting_inspect_app_restart_and_profile_isolation` | Not authored or executed; STOP-FMT, no evidence/pass. |
| 34 | `accounting_inspect_app_off_and_old_usage_routes` | Not authored or executed; STOP-FMT, no evidence/pass. |

## Deferred gates and successor

Complete S03-A before receiving review or human-test handoff. Then Fable owns
one independent material code review, combined state/TaskNode and TUI checks,
a fresh-context code-blind design, a separate enforced-isolation code-blind
executor using the exact read-only package, and independent evidence review.
Prepare synthetic installed, absent, unknown, partial, compact and corrupt state
through an isolated setup operator. Keep ordinary profiles/collection OFF.
The executor must use real PTY keys and preserve negative access probes, cancel,
retry, refresh, thread switch, restart and 40/80-column evidence.
Resolve TensorCash and Isometric Game paths/bases and disposable worktrees.
No native, live-repository, platform, human, benchmark or release acceptance is
claimed here. Existing broader S03/S02 inherited obligations remain unchecked.
