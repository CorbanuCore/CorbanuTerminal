# PF-83-S01 implementation checkpoint

Implementation frozen for independent review, not functional acceptance. Coordinates: worktree
`/Volumes/CorbanuDrive/Corbanu/worktrees/live-permission-transition-20260913`,
branch `fix/live-permission-transition-20260913`, base
`005cc644f59b1e762e5497b329e106c67925d4ed`. Product initiative;
**Permission selection confirmation — TO BUILD**, “A submitted selection is not
a confirmed change.” Expanded scope and size refinement are recorded in the
parent-owned allocation. No Core product source edits, commits, installs or live
profile/application changes.

## Current implementation

- Opt-in `confirm: true` settings request; legacy requests still serialize `{}`.
  `outcome: applied` reports this Core operation's success, including empty and
  unchanged operations. `outcome: uncertain` covers lost completion/10-second
  deadline. Validation and correlated Core errors remain RPC errors.
- Pending state is registered before submission and stores connection request
  identity and listener generation under the thread's state. Core event IDs are
  matched before notification deduplication. One deferred owner replies; listener
  teardown/replacement, disconnect and shutdown drop pending senders.
  The deadline owner starts before bounded submission, which stays in the native
  per-thread request sequence to retain enqueue order. The end-to-end server
  deadline is ten seconds; the TUI transport wait is fifteen seconds.
- TUI sends confirmation asynchronously and serializes picker requests, labels
  requested/applied-for-next-turn/failed/unsupported/uncertain states, and ignores
  obsolete completion IDs. A reply never copies an old snapshot or requested
  values over newer observed state. Failed/uncertain requests are not cached as
  future-turn overrides. Later turns preserve Core's session permission settings.
- New-turn submission while pending restores the saved draft via a narrow helper
  in allocated `chatwidget/settings.rs`. Stop/approval paths remain available; no automatic stop, approval or
  request replay. Built-in reviewer persistence is deferred until confirmed
  success. Current-turn authority and live MCP refresh remain Core-owned.

## Actual attempts retained

- Initial `just fmt` completed; it reformatted unrelated baseline Rust/Python
  files. Those formatting-only diffs were reversed, with no prior user edits in
  those files. Repeat formatting follows the same bounded diff audit.
  Exact restored paths: `codex-rs/core/tests/suite/accounting_anthropic_recovery.rs`,
  `codex-rs/login/src/auth/default_client_tests.rs`, changed files under
  `codex-rs/responses-api-proxy/` and `scripts/initiative_control/`. Root `just fmt`
  invokes both Rust and Python formatters. Each restoration reversed only that
  verified worker-introduced diff; no user/planning or allocated WIP was reverted.
  One wrong-directory reverse command produced "No valid patches" and made no
  changes; rerunning from the worktree root restored the intended files.
- Existing dedup test passed: nextest `8ce13edb-9bee-4272-9247-9f51e35aface`,
  one passed/260 skipped. This early result is not final-tree qualification.
- Stable schema generation passed through `just test` ignored fixture writer,
  nextest `b9273462-c59e-4102-9b35-78a9e3ef226e`, one passed/287 skipped.
  Experimental generation passed: `schema-experimental.log`, nextest
  `f50cd47d-dca7-4f61-8cd7-90cec465a2cc`. No direct `cargo test` invocation;
  nextest's own compile command appears in its normal diagnostics.
- `tui-attempt-1.log`: compile failed on the separate App test initializer missing
  the new field. Parent allocated its one-line correction; fixed.
- `server-attempt-1.log`: 8/11 passed, 3 failed during `initialize` timeout before
  settings submission (including two unchanged tests); nextest's automatic retry
  attempts are retained. Serial replay `server-attempt-2.log`: all 11 passed,
  nextest `5b089831-8a18-4ac6-adc4-a6edb3cc8328`; not final-tree qualification.
- `tui-attempt-2.log`: 3/6 passed, new unaccepted snapshot and two obsolete picker
  optimistic-history assertions failed. Parent allocated regression migration;
  the new outcome snapshot was inspected/accepted, obsolete assertions migrated.
- `focused-final-1.log`: compile failed on a migrated by-value picker assertion's
  extra dereference. Corrected; no tests executed in that attempt.
- `focused-final-2.log`: 345 executed, 344 passed, one repeated legacy TUI settings
  notification timeout. New native/RPC/picker/protocol and unchanged Core tests
  passed. This failure was investigated, not called a flake or waived.
- `tui-settings-diagnostic.log` and `tui-settings-diagnostic-2.log`: two diagnostic
  runs, each failing on both nextest attempts. The second exposes an explicit
  legacy RPC rejection before Core submission: `gpt-5.4` + `ambient` is an
  incompatible model/provider pair. The default fixture uses Ambient; the test
  ignored error history and awaited a notification that could not occur.
  Temporary diagnostics were removed. Only this test's initial fixture changed
  to OpenAI/GPT; every notification assertion and the two-second timeout remain.
  See `legacy-timeout-diagnosis.md` for base-source parity evidence and limits.
- `focused-final-3.log`: compile failed on a private helper re-export in the
  corrected legacy fixture; changed to its existing accessible `helpers` path.
  No tests executed. No helper visibility or source scope expansion.
- `focused-final-4.log`: 349 executed, 348 passed. Corrected legacy fixture,
  direct deferred-reply lifecycle and existing approval/interrupt tests passed.
  New pending-draft assertion failed on both attempts: the existing generic
  rejection helper does not restore the submitted prompt and can auto-submit
  queued input. Fixed the pending-only guard through a narrow helper in the
  already-allocated `chatwidget/settings.rs`: clear pending-start, restore the
  existing saved UserMessage, update the running indicator, report deferral.
  It does not invoke terminal-error/queue replay logic or mutate Core authority.
- `focused-final-5.log`: all 349 passed with the draft fix (49.035 seconds).
  Not the final strengthened-test tree: the parent then requested explicit saved
  attachments/mentions, queue, indicator and pending-control assertions.
- `focused-final-6.log`: 350 executed, 349 passed. Strengthened regression
  preserves local/remote images, mention bindings/text elements, queued input and
  both running/non-running states; existing approval and interrupt tests now run
  with pending confirmation. Redundant stale-result setup was compacted without
  dropping outcomes. The new saved-image fixture failed because its local image
  used index 1 despite an existing remote image occupying that index. Corrected
  fixture to the existing remote-first index 2 convention; all exact assertions
  retained, no runtime image handling changes.
- `focused-final-7.log`: all 350 passed (46.918 seconds), including full saved
  payload/queue/state checks and pending approval/interrupt tests. Added the local
  image's normal placeholder text element to the fixture as a final test-only
  strengthening; no runtime changes. Exact-tree replay is `focused-final-8.log`.
- `fmt-final-12.log`: latest formatting completed and unrelated churn restored.
- `focused-final-8.log`: **350/350 passed**, 8695 skipped, 46.091 seconds;
  nextest `778753ff-3655-43dc-a1e3-11f626d68064`, command exit 0. Includes latest
  server enqueue/cleanup and saved-message placeholder/attachment/mention tests.
  No source changes after this run. Exact command is `verification-command.md`.

## Frozen handoff

- `implementation.patch` SHA256:
  `e36396dcd62ad8daf59832302ea69f10e24770ddab3d44ce1b99122ed69cf96b`.
- `candidate-manifest.md` SHA256:
  `e292e7c7f62706ec88394fd3e3cb5daae0d06bd797240c7d979ca7db0ae8fca8`.
- `focused-final-8.log` SHA256:
  `b48311610a6e0243d37a7c0b9b9b62737917a786824b1277c6a5340a7e9178d0`.
- Patch and per-file manifest regenerated only after final8 exit 0; reverse
  application check passed without modifying the tree. 32 candidate paths,
  no out-of-scope source paths, no staged files. No owned test/build or fixture
  server processes remained at handoff (process scan excluded its own shell/rg).
- Both governance checkers pass: 3 active, 116 current, 126 archived. The three
  removed optimistic snapshots are recoverable from base and the source patch.
- No known failing focused support test remains. All historical failures remain
  in their logs, including final4 draft loss and final6 image-index fixture error.
- Limitations: no Windows execution, no actual old-server binary execution, no
  isolated baseline executable comparison, no independently enforced acceptance.
  Pending-control support covers buffered approval choice and idle native
  interrupt handling; active-command real-key interruption/continuation proof is
  still required from the independent executor. No commit/install/live changes.

All Rust commands use `RUSTUP_TOOLCHAIN=1.95.0`, locked/offline dependencies and
`CARGO_TARGET_DIR` equal to this worktree's
`.codex-work/live-permission-transition-target`. Build output is untracked and
must never be staged. Logs contain full attempts, not independent acceptance.

## Scope and compatibility receipt

Current measured changed lines: 517 non-test + 733 test = 1250 hand-authored Rust
source/test lines (additions plus deletions, including untracked leaves), under
the recorded <=650/<=1250 refinement. Snapshot changes are 27 lines, API README
two changed lines; mechanical schemas are 42 text lines plus two compressed
fixtures. Literal sprint source-path audit passed; no out-of-scope source diff.

| Client/server | Outcome and evidence |
| --- | --- |
| Legacy client / new server | Omitted `confirm` preserves literal `{}` acceptance; real RPC fixture. |
| New TUI / new server | Only correlated Core success yields explicit Applied; native and RPC both-direction/no-op tests, exact identity/lifecycle support tests. |
| New TUI / acceptance-only server | `{}` is Unsupported/unconfirmed, never success or automatic retry; wire classifier fixture, not an old binary execution. |
| New TUI / method-unsupported server | Method-not-found/gating is unconfirmed, never fallback confirmation; error classifier fixture, not an old binary execution. |
| New TUI / error or lost completion | Known rejection fails; internal/transport/deadline/lifecycle ambiguity remains uncertain, with no cached request replay. |

Three deleted optimistic-success snapshots (`permissions_selection_history_after_mode_switch`,
`permissions_selection_history_full_access_to_default`, and its `@windows` variant)
map to typed picker payload/no-mutation assertions plus App-level
`permission_confirmation_outcomes_are_explicit.snap`. Full-access consent,
disabled choices, both directions, same-state and reviewer selection remain
covered by existing migrated tests. The Windows-specific old success snapshot
was superseded, not executed on Windows; no Windows execution claim is made.

## Frozen acceptance mapping

Original F01–F11 remain byte-for-byte at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/permission-transition.eocbhB/frozen-functional-cases.md`;
SHA256 rechecked `c97bf701cd7aff2e26f08884f35dbc9a4fc33b1eef14eee6328d2ea1f1411726`.
The explicit user-approved amendment is a next-turn command permission boundary:
F01/F02 test fresh turns; F03/F04/F06 retain current-turn/in-flight authority and
follow the disclosed finish/stop-and-new-prompt path; F05 still requires explicit
pending-approval resolution; F07 allows visibly refused concurrent selections;
F08 retains cancel/failure/uncertain branches; F09 tests both-direction continuation;
F10 retains recovery without inferred completion/retry; F11 retains route neutrality.
This mapping neither rewrites nor waives any original case. All independent
execution dispositions remain open.

Parent arranges fresh Fable code review, enforced independent code-blind real-key
execution of the exact package in both disposable live repositories, and a
separate evidence review. Human acceptance, integration, benchmark/release and
installed-runtime status remain unclaimed. Sprint stays `in_progress`.
