# RETURN — pf83-rebind-39

Allocation digest: `564f27ab37af5c398e254114482061516a9e4c5a72aa314d8a9a912c57729cba`.
Claim: `14eead9b-2ae0-4ad9-8c9e-0f8191a9546d`.
Runtime: `gpt-6-astra`, effort `high`.
Brief SHA-256 verified before other reads: `622836dcbc688c7a546f1635e343ed93574b7b0736b254784c720c0baec2e574`.
Base: `23915e5c6f06df102ee500e5b0ccc81e64fe315c`.
Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/pf83-rebind-20260916`.
No commit, push, release, install, live-profile use or human-acceptance claim.

## Authority

Product-initiative revision under active plan `docs/plans/active/p0-security-levels.md`, feature PF-83, sprint PF-83-S01 (`in_progress`).
Product heading **Permission selection confirmation — TO BUILD**: “Existing active-turn approval/sandbox snapshots and pending approvals are not retroactively changed.”
This explicit manager allocation governs the authorized Core revision. The older canonical plan/sprint coordinates and exclusions still describe the prior worktree/next-turn-only allocation; reconciliation remains manager-owned and outside this worker's writable scope. The sprint checker passed (115 current, 127 archived).
Independent functional execution/evidence review, exact-package real-key TUI proof, both live repositories, named-human acceptance and release/benchmark qualification remain with the integrator; these automated results do not close those gates.

## Repairs and decisions

- Added `CodexThread::inject_extension_if_running`, its Session entry point, and an internal `InjectionSource::{RunningTurn, Extension}` distinction. The goal extension now calls the extension entry point. Ordinary `Session::inject_if_running` and `inject_no_new_turn` retain current-turn delivery for code-mode notify, user-shell output, Guardian approval notices and saved-rule fragments. Only extension admission defers on mismatched authorization or a non-steerable task. The extra CodexThread impl is in the allocated `session/inject.rs`; no out-of-scope bridge file was edited.
- `authorization_changed_turn_receives_code_mode_notify_and_user_shell_output` exercises the real CodeModeService dispatch worker → CoreTurnHost notify → Session injection and real auxiliary user-shell execution after a permission change. It verifies both outputs enter the running turn's consumable queue, no deferred leftovers remain, and its captured authority is unchanged. The provider fixture captures the real delegate without starting a JavaScript VM; the separate code-mode lane exercises the existing VM integration.
- Interrupted deferred input stays in a visible session FIFO, signals a Notify wake and activity notification, and is selected by the submission loop as ordinary `Op::UserInput`. It passes through the same user-input handler, steerability and authorization checks used by natural-completion follow-ups. It is never spliced into `get_pending_input_for_task_start`. Response items use extension admission before idle recording; retained communication uses its native operation.
- `has_pending_input` and `subscribe_activity` see interrupted storage; `TurnInputQueue::has_user_input` sees turn-local deferred user input. Active-turn deferred work deliberately remains excluded from the model loop's same-turn continuation predicate so that the old turn can finish.
- Existing interrupt-survival coverage now asserts visibility and absence from task-start input. Added `authorization_changed_interrupted_input_wakes_fresh_admission` and `interrupted_input_cannot_be_spliced_into_review_task` cover wake/fresh authority and re-deferral for Review.
- The no-active-turn race re-derives the latest context through `new_turn_with_sub_id` with only the caller's original output schema as an update. It does not reapply stale permission settings; metadata, additional context and client/parent IDs retain their existing separate paths. The retained `authorization_changed_defer_race_starts_fresh_turn_without_error` now supplies an object schema and asserts exact schema equality alongside latest authority.
- Reverted unconditional `mark_mailbox_ready_for_next_turn` in `clear_pending`. `abort_does_not_advance_unrelated_queue_only_mail` proves abort does not advance unrelated queue-only mail; natural-completion behavior remains intact.
- Changed error data to echo the original typed v2 `UserInput` array, not Core's serialized input. Clients can rely on `code: "authorizationChanged"`, `inputDisposition: "returned"`, and v2 input variant/field serialization. The error envelope remains the existing ad hoc JSON object; no new generated protocol type is claimed. The native RPC test checks both text and `localImage`, distinguishing v2 camelCase variant spelling from Core's internal spelling.

## Disclosed limits and brief assessment

The requested limit remains: in `Session::on_task_finished`, the non-abort `Err(err)` arm logs “session task returned an unexpected error” and produces `(None, None, false)`. Its subsequent pending-input loop only schedules a follow-up when `completed_naturally`; otherwise it records input into history without execution. This is newly reachable by authorization-deferred input and has no dedicated regression in this revision.

The brief's three reported defects were confirmed. Its build prerequisites were incomplete for the requested code-mode integration lane: the standalone `codex-code-mode-host` binary was also required; building it resolved the retained setup failure without source changes. One precision point: `has_pending_input` is also the model's same-turn continuation predicate, so counting all turn-local deferred work there would prevent completion. Visibility is repaired through interrupted-storage checks and activity/user-input predicates while preserving that boundary. The completion path still uses its pre-existing TurnInput follow-up representation (without per-request schema/metadata); this revision fixes the specifically requested completion-race schema loss, not that separate normal-deferral limitation.

F04's function remains byte-identical to HEAD. SHA-256 of the function through the next test annotation: `0d2667d9cfecc5660c30084b34c44289a72a61044246cdfd8ba9c2ce9914e98a`. No existing test was removed, ignored or relaxed.

## Validation

All commands run from `codex-rs` with one shared dedicated `CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/pf83-rebind-31`. The prior allocation's dedicated cache is reused across every lane. All tests use the checkout's guarded `just test`, with `INSTA_UPDATE=no --locked --offline --retries 0 --test-threads 1`. Test-isolation guidance was read before execution. No raw cargo tests or workspace-wide formatting/fix commands were used. Raw logs are retained beside this receipt with prefix `pf83-rebind-39-` (ignored by git).

| Command (test commands also use the common flags above) | Outcome | Log suffix |
| --- | --- | --- |
| `cargo build --locked --offline -p codex-cli --bin codex` | exit 0; initial build and final-source rebuild retained separately | `build-cli.log`, `build-cli-final.log` |
| `cargo build --locked --offline -p codex-rmcp-client --bins` | exit 0 | `build-rmcp.log` |
| `just test -p codex-app-server settings` | 24 run / 24 passed / 0 failed; 1074 skipped; exit 0 | `settings.log` |
| `just test -p codex-app-server turn_steer` | 5 run / 5 passed / 0 failed; 1093 skipped; exit 0 | `turn-steer.log` |
| `just test -p codex-tui permission` | 91 run / 91 passed / 0 failed; 4074 skipped; exit 0 | `tui-permission.log` |
| `just test -p codex-core --lib 'session::'` | 341 run / 341 passed / 0 failed; 2141 skipped; exit 0 | `core-session.log` |
| `just test -p codex-core --test all -E 'test(suite::pending_input::)'` | 11 run / 11 passed / 0 failed; 1167 skipped; exit 0 | `pending-input.log` |
| `just test -p codex-core -E 'test(authorization_changed_turn_receives_code_mode_notify_and_user_shell_output) \| test(code_mode_notify_injects_additional_exec_tool_output_into_active_context)'` | 2 run / 1 passed / 1 failed; 3662 skipped; exit 100 | `injections.log` |

Only failure: `suite::code_mode::code_mode_notify_injects_additional_exec_tool_output_into_active_context`, `core/tests/suite/code_mode.rs:2932`, expected notify marker absent. Captured output was `unsupported custom tool call: exec`. The dedicated target lacked `debug/codex-code-mode-host`; the unavailable provider falls back to direct tools before notify can execute. The new actual-dispatcher/user-shell permission-change regression passed in both the session suite and injection lane. No failure is attributed to baseline without a baseline run. The subsequent `cargo build --locked --offline -p codex-code-mode-host --bin codex-code-mode-host` passed (exit 0; `build-code-mode-host.log`). Repeating the identical injection command once with the host present passed **2 run / 2 passed / 0 failed, 3662 skipped, exit 0** (`injections-replay.log`, run `2636fa9e-2b79-4e21-becb-e16ddfb0645f`). No source changed between the failed attempt and this replay. Every required lane has a passing final-source result; the initial failed attempt remains evidence.

The only slow classification was `suite::v2::client_metadata::turn_steer_updates_client_metadata_on_follow_up_responses_request_v2` (45.907 seconds), passed. No automatic retry, timeout or leaky-pass classification occurred. No native credential prompt was observed.

Only edited Rust files were formatted with `rustfmt --edition 2024 --config skip_children=true`; stable-toolchain imports-granularity warnings only. `git diff --check` and the sprint checker pass. Scope inspection shows only seven allocated Rust files and this receipt. Rust changes: **148 outside tests** (+128/−20), **348 inside tests** (+335/−13), **496 total**. All pre-existing tests retained. This receipt adds **56 non-test documentation lines**, for **204 total changed lines outside tests** including documentation (below the hard 260-line ceiling), and **552 total changed tracked/intended lines**. Raw build/test logs are ignored artifacts.
