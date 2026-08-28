---
work_package_id: "RW-TMUX-02"
title: "Tmux failure artifacts, slash dispatch, and Ubuntu smoke lane"
change_class: routine
status: completed
owner: "Terminal engineering"
source_plan: "research/tmux-testing/tmuxPlan.html"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-tmux-harness"
branch: "codex/tmux-artifacts-slash-smoke"
base_commit: "b7aff3e3bfbeebe7897f8ccb10c569ff18f9eff6"
depends_on: "RW-TMUX-01 (completed)"
stage_1_branch: "codex/tmux-failure-artifacts"
stage_1_commit: "510beb079"
stage_2_commit: "303ebfe10"
reproduction_fix_commit: "1193aded2"
process_cleanup_commit: "2683005ab"
ci_trigger_fix_commit: "d4f1343dc"
created: 2026-08-24
updated: 2026-08-24
---

# RW-TMUX-02 - Tmux failure artifacts, slash dispatch, and Ubuntu smoke lane

## Execution mandate

- Deliver lazy failure bundles, one real-TUI single-Enter slash regression, and one focused Ubuntu tmux smoke job.
- Exclude product changes, control mode, remaining resize migrations, release-matrix adoption, live providers, Windows tmux, and success artifacts.
- Keep fewer than 500 non-mechanical and 800 total changed lines; split before exceeding the repository limit.

## Authority and classification

- **Routine** test infrastructure; stop and reclassify if product or security behavior must change.
- Source: [Corbanu Terminal Testing Harness Plan](../../research/tmux-testing/tmuxPlan.html).

## Functional requirements

1. A timeout, tmux command failure, or panic emits one scenario diagnostic directory automatically.
2. Bundles include a manifest, reason, viewport, scrollback, pane metadata, commands, inputs, dimensions, reproduction command, and registered fixtures/logs.
3. Artifact creation is lazy: a successful scenario leaves no bundle.
4. Commands redact secret-bearing assignments; authentication files are never registered.
5. A real Corbanu process receives literal `/model` plus one separate Enter and displays `Select Model and Effort`.
6. The scenario exits through `/exit`; server/session cleanup remains private and panic-safe.
7. Ubuntu installs and requires tmux, runs the focused set, and uploads bundles only on failure.

## Non-functional requirements

1. Artifact names are stable for CI while scenario directories remain parallel-safe.
2. Artifact writing is best-effort and must not replace the original failure.
3. Captures and polling remain bounded; reusable code adds no fixed sleeps.
4. No live credentials, external model, or provider network is required.
5. Existing macOS behavior and non-tmux Windows test coverage remain unchanged.
6. The focused smoke runtime target is p95 below 30 seconds after compilation.

## Code boundaries

- Harness: `codex-rs/tui/tests/support/{tmux,tmux_artifacts,tmux_command,tmux_process,tmux_tests}.rs`.
- Scenarios: `codex-rs/tui/tests/suite/{slash_dispatch,resize_reflow}.rs` and suite index.
- CI/evidence: `.github/workflows/tmux-smoke.yml`; `qa/work-packages/evidence/RW-TMUX-02/RESULTS.md`.

Review staging: failure-artifact infrastructure lands first on `codex/tmux-failure-artifacts`; the slash scenario and CI lane form the smaller stacked final stage on this branch.

## Done

- [x] RW-TMUX-01 dependency completed and pushed.
- [x] Worktree, branch, and base commit allocated.
- [x] Source plan, harness, TUI test policy, and CI conventions inspected.
- [x] Scope classified Routine with product behavior explicitly excluded.

## Remaining

- [x] Add lazy, redacted artifact recording and automatic failure emission.
- [x] Cover timeout, command-failure, panic, redaction, and no-success-artifact contracts.
- [x] Add the real-TUI `/model` single-Enter regression and clean `/exit` proof.
- [x] Name the resize and slash scenarios as the focused `tmux_smoke` set.
- [x] Add Ubuntu hard tmux availability and failure-only artifact upload.
- [x] Update this ledger and final evidence from the exact final tree.

## Required tests

- [x] `cd codex-rs && just fix -p codex-tui`.
- [x] `cd codex-rs && just fmt`.
- [x] Artifact contract tests pass and inspect complete bundle contents.
- [x] `cd codex-rs && just test -p codex-tui --test all tmux --retries 0` passes 9/9.
- [x] Focused `tmux_smoke` set completes 20 local runs with no failure/leak and p95 7 seconds.
- [x] `cargo insta pending-snapshots --manifest-path tui/Cargo.toml` reports none.
- [x] GitHub Actions YAML parses and references valid actions and exact filters.
- [x] Final local autoreview reports no actionable issue.
- [x] `git diff --check` passes and the final tree is clean after commit.

## Exit evidence

- [x] Remaining/tests are complete or scoped out with a recorded reason.
- [x] Evidence records commit, commands, counts, durations, manifest, and cleanup.
- [x] No product source/docs, snapshots, live credentials, or release claims changed.
- [x] Status changes to `completed` before push.

## Follow-on queue

- `RW-TMUX-03`: bounded control-mode parsing, one multi-pane lifecycle proof,
  and migration of the ignored width-resize/restore scenario.
- `RW-TMUX-04`: migrate the repeated-resize scenario and adopt the proven typed
  support incrementally in the release matrix.
