---
work_package_id: "RW-TMUX-03"
title: "Tmux control mode and bounded multi-pane lifecycle"
change_class: routine
status: completed
owner: "Terminal engineering"
source_plan: "research/tmux-testing/tmuxPlan.html"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-tmux-harness"
branch: "codex/tmux-control-mode"
base_commit: "20ce0cae2a7efb54afb2d52b82719e064bdd5b9f"
depends_on: "RW-TMUX-01 and RW-TMUX-02 (completed)"
created: 2026-08-24
updated: 2026-08-24
---

# RW-TMUX-03 - Tmux control mode and bounded multi-pane lifecycle

## Execution mandate

- Add a bounded tmux control-mode reader and parser to the existing private test
  support; do not create a new workspace crate.
- Prove the parser with one product-neutral multi-pane lifecycle scenario.
- Migrate only
  `tmux_width_resize_restore_keeps_visible_content_anchored` from direct tmux
  commands and fixed sleeps to the typed harness, then remove its `#[ignore]`.
- Keep the change under 500 non-mechanical and 800 total changed lines. Split the
  parser, transport, fixtures, and tests into private sibling modules as needed.

## Authority and classification

- **Routine** internal test infrastructure. Stop and reclassify if implementation
  changes TUI behavior, authorization, persistent state, or an external contract.
- Source: [Corbanu Terminal Testing Harness Plan](../../research/tmux-testing/tmuxPlan.html).

## Scope decisions

1. Use tmux `-C` control mode against the harness's existing private server.
2. Treat control mode as an event source for command boundaries, pane output,
   layout changes, and pane removal. Viewport capture remains the rendering oracle.
3. Preserve the existing shell-command path for simple tests; migration is
   justified only when event ordering or multi-pane identity matters.
4. Defer the second ignored repeated-resize scenario and all
   `qa/orchestrate_tui_matrix.sh` adoption to `RW-TMUX-04`.

## Functional requirements

1. Start and stop one control-mode client without contacting the developer's
   default tmux server or changing the existing server/session ownership model.
2. Parse `%begin`, `%end`, and `%error` command blocks and correlate each block
   with its tmux command sequence identifier.
3. Parse `%output`, `%extended-output`, `%layout-change`,
   `%window-pane-changed`, `%pane-mode-changed`, `%pause`, `%continue`, and
   `%exit` as typed events while preserving unknown notifications as bounded raw
   events for forward compatibility.
4. Decode tmux control-mode escaped pane output without lossy UTF-8 assumptions;
   malformed framing or escapes return structured errors with the source line.
5. Bound every input line to 256 KiB and the queued event backlog to 1,024
   events or 4 MiB, whichever comes first. Crossing any limit fails the scenario
   and emits the failure bundle.
6. Provide deadline-based waits for a typed event predicate and for a command
   result. No reusable path may use a fixed sleep.
7. Add one product-neutral lifecycle test that creates two panes, sends a unique
   marker to each immutable pane ID, observes isolated output and a layout
   change, closes the secondary pane, observes command completion and the
   resulting layout change, then proves `list-panes` no longer returns its ID.
8. Add the control transcript and parser error, when present, to TMUX-02's lazy
   failure bundle without emitting success artifacts.
9. Migrate the ignored width-resize/restore test to the typed harness and stable
   semantic waits; retain its existing composer and history anchoring assertions.

## Non-functional requirements

1. Parsing is deterministic and separately testable from process transport.
2. Reads, waits, buffers, event queues, and artifact files are explicitly bounded.
3. Reader threads or tasks are joined on success, error, timeout, and panic; no
   control client, pane, tmux server, or child process may outlive its test case.
4. Existing macOS and Ubuntu tmux behavior remains supported. Windows continues
   to use PTY/ConPTY coverage and compiles without requiring tmux.
5. No live credentials, provider network, retries, default tmux socket, or
   developer-global configuration is used.
6. The two TMUX-03 scenarios have a measured local p95 runtime below 15 seconds
   after compilation and keep the complete CI tmux lane below 30 seconds.

## Code boundaries

- Primary: `codex-rs/tui/tests/support/tmux_control*.rs`.
- Narrow integration: `codex-rs/tui/tests/support/{mod,tmux,tmux_artifacts}.rs`.
- Scenario migration: `codex-rs/tui/tests/suite/resize_reflow.rs`.
- Evidence: `qa/work-packages/evidence/RW-TMUX-03/RESULTS.md`.
- Do not modify product source, `qa/orchestrate_tui_matrix.sh`, release workflows,
  or shipped documentation in this package.

## Done

- [x] RW-TMUX-01 and RW-TMUX-02 are completed and pushed.
- [x] Scope remains Routine and product behavior is excluded.
- [x] Exact ignored migration target and implementation coordinates are recorded.
- [x] Release-matrix adoption is separated into RW-TMUX-04.
- [x] Implementation started on the recorded worktree, branch, and base commit.
- [x] Bounded parser, transport, lazy artifacts, lifecycle proof, and width
  migration are implemented in private TUI test support.
- [x] Final macOS verification, 20-run reliability sampling, and live
  `just codex` qualification passed.
- [x] Source stages were pushed to `codex/tmux-control-mode`.

## Delivered

- [x] Recorded local tmux `3.7c`; the Ubuntu execution blocker and workflow
  registration follow-up are recorded in the evidence report.
- [x] Added parser fixtures for command blocks, all required notifications,
  escaped output, unknown events, malformed input, and both resource limits.
- [x] Added the private control-mode transport with bounded reads and joined cleanup.
- [x] Integrated control transcript and parser errors into lazy failure artifacts.
- [x] Added the two-pane lifecycle and event-ordering contract test.
- [x] Migrated the width-resize/restore test and removed only its `#[ignore]`.
- [x] Ran final-tree verification and wrote `evidence/RW-TMUX-03/RESULTS.md`.

## Required tests

- [x] `cd codex-rs && just fix -p codex-tui`.
- [x] `cd codex-rs && just fmt`.
- [x] Parser tests compare complete typed event values and error values.
- [x] Resource-limit tests prove deterministic failure at 256 KiB, 1,024 queued
  events, and 4 MiB without retaining input beyond the configured bounds.
- [x] The lifecycle test proves pane-specific output, layout change, pane
  removal by immutable ID, ordered command completion, and complete cleanup.
- [x] The migrated width-resize test proves baseline rows are restored and emits
  a complete bundle when an assertion is induced to fail.
- [x] `CORBANU_TMUX_REQUIRED=1 just test -p codex-tui --test all tmux --retries 0`
  passes with no `LEAK` marker or failure artifact.
- [x] Twenty consecutive runs of the two TMUX-03 scenarios pass with zero
  retries, failures, leaked processes, private sockets, or success artifacts;
  record p50 and p95 duration.
- [x] `just test -p codex-tui --test all --retries 0` passes.
- [x] `cargo insta pending-snapshots --manifest-path tui/Cargo.toml` reports none.
- [x] Manual final review and `git diff --check` pass; automated autoreview's
  model call remained healthy but returned no result after 33 minutes, as
  recorded in the evidence report.

## Exit evidence

- [x] Evidence records final commits, tmux versions, commands, counts, durations,
  parser fixtures, resource-limit results, artifact manifest, and cleanup proof.
- [x] The second ignored resize test and release matrix remain unchanged and are
  named as RW-TMUX-04 work rather than silently absorbed.
- [x] No product source, protocol, shipped documentation, credentials, persistent
  state, or release claim changed.
- [x] Status changed to `completed` after the implementation and all executable
  local gates passed; Ubuntu tmux execution remains an explicit post-registration
  CI follow-up rather than an unearned pass claim.

## Follow-on queue

- `RW-TMUX-04`: migrate the repeated-resize scenario and adopt the typed harness
  incrementally in `qa/orchestrate_tui_matrix.sh`, with matrix behavior preserved.
- Product-specific terminal scenarios remain owned by their product sprints.
