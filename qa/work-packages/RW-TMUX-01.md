---
work_package_id: "RW-TMUX-01"
title: "Typed tmux harness foundation"
change_class: routine
status: completed
owner: "Terminal engineering"
source_plan: "research/tmux-testing/tmuxPlan.html"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-tmux-harness"
branch: "codex/tmux-harness-foundation"
base_commit: "2e7bdcb8e4ec7070aaa383b00ac5c6f106c3b9c7"
depends_on: "none"
created: 2026-08-24
updated: 2026-08-24
---

# RW-TMUX-01 - Typed tmux harness foundation

## Execution mandate

- Deliver: isolated, typed tmux test support for the `codex-tui` integration
  test binary and migrate one existing resize/reflow test onto it.
- Excludes: product behavior changes, slash-command coverage, CI workflow
  changes, full artifact bundles, control-mode parsing, release-matrix
  migration, Windows terminal support, and migration of the other two ignored
  resize tests.
- Change-size ceiling: fewer than 500 non-mechanical changed lines where
  practical and fewer than 800 total changed lines. Stop and split the package
  before exceeding the repository review limit.

## Authority and classification

- Change class: **Routine** test infrastructure.
- Product authority: not applicable; this record cannot authorize product
  behavior, security policy, persistent state, external contracts, or release
  claims.
- Reclassification trigger: stop if implementation requires a user-visible TUI
  change or any product/security boundary change, then route that scope through
  the applicable active plan and product sprint.
- Source: [Corbanu Terminal Testing Harness Plan](../../research/tmux-testing/tmuxPlan.html).

## Code boundaries

- Existing integration-test root: `codex-rs/tui/tests/all.rs`.
- Planned support module: `codex-rs/tui/tests/support/mod.rs`.
- Planned harness: `codex-rs/tui/tests/support/tmux.rs`.
- Planned harness tests: `codex-rs/tui/tests/support/tmux_tests.rs`.
- Migrated workflow: `codex-rs/tui/tests/suite/resize_reflow.rs::tmux_split_preserves_fresh_session_composer_row_after_resize_reflow`.
- Explicitly untouched: `codex-rs/tui/src/**`, product configuration, protocol,
  Core agent logic, snapshots, `qa/orchestrate_tui_matrix.sh`, and GitHub
  Actions.

## Harness contract for this package

1. `TmuxServer` starts and targets a unique `tmux -L` socket and kills only that
   server during cleanup.
2. `TmuxSession` creates one fixed-size session and records the immutable primary
   pane id returned by tmux.
3. `TmuxPane` targets operations by pane id, never by a mutable index or title.
4. Literal text and named keys use separate methods; tests prove that the string
   `Enter` is not interpreted as the Enter key.
5. Live viewport and bounded scrollback capture are distinct operations.
6. A stable wait requires a semantic match followed by two identical live
   viewport captures within a bounded timeout.
7. Vertical split, pane close, and viewport capture are sufficient to express
   the selected resize/reflow workflow.
8. Every failed tmux command reports the rendered command, exit status, stdout,
   and stderr. Wait failures include the last live viewport.
9. Success, error, and panic paths clean up the private server without touching
   the developer's default tmux server.

## Preconditions

- [x] Repository root, `codex-rs/AGENTS.md`, and `codex-rs/tui/AGENTS.md` read.
- [x] Work is classified Routine and introduces no product behavior.
- [x] Source research and existing resize tests inspected.
- [x] tmux 3.7c installed and available on `PATH`.
- [x] Exact worktree, branch, and base commit allocated.
- [x] Dependencies are available from existing `codex-tui` dev dependencies;
  no Cargo dependency change is planned.

## Done

- [x] Source plan converted into this bounded execution record.
- [x] Scope reduced to one coherent review unit under the repository size limit.
- [x] Dedicated implementation worktree created from the recorded base commit.
- [x] Product behavior, slash dispatch, CI, and release-matrix work excluded.

## Remaining

- [x] Add private `support` aggregation to `codex-rs/tui/tests/all.rs`.
- [x] Implement unique-socket server lifecycle and command diagnostics in
  `tests/support/tmux.rs`.
- [x] Implement typed session, pane, literal input, named key, capture, split,
  close, and stable-wait operations needed by the selected workflow.
- [x] Add integration-harness tests in the separate sibling
  `tests/support/tmux_tests.rs` module for isolation, literal-versus-key input,
  live viewport versus scrollback, stable waits, diagnostics, and cleanup.
- [x] Migrate only
  `tmux_split_preserves_fresh_session_composer_row_after_resize_reflow` from
  direct tmux commands and fixed sleeps to the support API.
- [x] Remove that test's `#[ignore]`; skip clearly when tmux is unavailable on a
  non-gating developer host. Leave the other two ignored tests unchanged.
- [x] Keep mock-provider setup and assertions in the workflow test; do not add a
  live provider or credential dependency.
- [x] Inspect the final diff for scope, platform guards, leaked processes, and
  the 800-line ceiling before verification.

## Verification

- [x] Prerequisite: `tmux -V` reports the tested version.
- [x] Fix: `cd codex-rs && just fix -p codex-tui`.
- [x] Format after fixes: `cd codex-rs && just fmt`.
- [x] Focused TMUX tests: `cd codex-rs && just test -p codex-tui --test all
  tmux` passes 7/7.
- [x] Final owning integration binary: `cd codex-rs && just test -p codex-tui
  --test all` passes 17/17 with two intentional skips.
- [x] Crate-wide command attempted twice: `cd codex-rs && just test -p
  codex-tui` is blocked before execution by the host linker (`ld: B/BL out of
  range`) while linking the oversized library test binary. This is unrelated
  to the changed integration-test code and is recorded in the evidence.
- [x] Snapshot audit: `cd codex-rs && cargo insta pending-snapshots
  --manifest-path tui/Cargo.toml` reports no unintended pending snapshots.
- [x] Stability: the migrated focused test completes 20 consecutive runs with
  zero failures and no private tmux server left after each run.
- [x] Harness stability: all six harness contract tests complete 20 consecutive
  runs with zero failures and no private socket root left after any run.
- [x] Default-server isolation: a sentinel session on the developer's default
  tmux server survives the complete focused test sequence unchanged.
- [x] Final tree: `git diff --check` passes after every code-changing tool.

## Evidence contract

Record final evidence under `qa/work-packages/evidence/RW-TMUX-01/`:

- implementation commit and changed-path summary;
- tmux version and exact final-tree test commands;
- focused and full `codex-tui` test output;
- 20-run repetition summary with durations and failure count;
- before/after default-server sentinel proof;
- post-run private-server and child-process cleanup proof; and
- final diff line count and review result.

## Exit evidence

- [x] Every Remaining item is complete or explicitly removed through a recorded
  scope decision.
- [x] Every applicable Verification item passes on the final tree; the unrelated
  host linker limitation is recorded with the exact error.
- [x] Evidence directory is complete and linked from this record:
  [RW-TMUX-01 results](evidence/RW-TMUX-01/RESULTS.md).
- [x] No user-facing behavior, product documentation, or release claim changed.
- [x] Status is changed to `completed`; final implementation commit and
  completion date are recorded here.

## Follow-on queue

- `RW-TMUX-02`: automatic artifact bundle, single-Enter slash dispatch
  regression, and a dedicated Ubuntu tmux smoke lane.
- `RW-TMUX-03`: control-mode event parsing, multi-pane lifecycle assertions, and
  incremental release-matrix adoption.
