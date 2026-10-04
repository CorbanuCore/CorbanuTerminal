# pf83-rebind-31 implementation receipt

Allocation digest: `a3a7eb5959914e37cc000d64c61188eb98121bb8c1ad5809c6b0b77b6bef4940`.
Claim: `6c833c9f-d19a-4e53-8d02-61d490322eb7`.
Brief SHA-256 verified: `2321e38f57860b0c0126497f10141a426b217607da6a8929e7c00fa9714310d6`.
Base: `a2fbf7db67c7a31cef442ba1b413fa7cd3d5766b`.
Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/pf83-rebind-20260916`.
Branch: `bootstrap/pf83-rebind-20260916`. No commit or push by this worker.

## Authority and scope

Product initiative under active `docs/plans/active/p0-security-levels.md`,
PF-83, sprint PF-83-S01 (`in_progress`). Product heading:
**Permission selection confirmation — TO BUILD**; excerpt:
"Existing active-turn approval/sandbox snapshots and pending approvals are not retroactively changed."
The September 16 decision in the sprint's Remaining ledger and the frozen manager
brief authorize the new admission boundary and Core paths. The older product/plan
prose and sprint exclusions still describe next-turn-only authority and no Core
writes; their coordinates name the prior worktree. This worker follows the
explicit new allocation; canonical record reconciliation remains manager-owned
and outside its writable scope. Sprint checker passed: 115 current, 127 archived.

## Chosen repair and concrete rebinding obstacle

Refuse steering when current effective authorization differs from the active
turn's captured authorization. User message on both Core and app-server paths:
"Permissions changed since this turn started. Wait for it to finish or stop it, then submit your message again."
No automatic interruption, replay, or grant; a fresh submission after the turn
finishes uses the existing fresh-turn path.

A context swap alone cannot implement safe rebinding: `run_turn` mixes pending
inputs into the same model history, and its next tools can reuse an existing exec
process. `tools/handlers/unified_exec/write_stdin.rs::handle_call` passes process
ID and bytes to the shared manager without a new approval/sandbox decision.
`unified_exec/process_manager.rs::write_stdin` looks up the existing process and
calls `process.write(...)`. A full-access interactive shell started earlier can
therefore receive a newly steered write even if the next step has a restrictive
context. Changing that process's authority or terminating it violates this
assignment's running-work constraint. Adding per-process admission checks or
work-origin tracking requires paths outside this allocation. Also, a single
follow-up model response can continue old work and respond to new input; blindly
swapping its context could loosen the old work. The refusal fallback avoids both
problems, at the disclosed cost of requiring user resubmission.

## Admission path

`Session::steer_input` checks current effective session configuration against
`active_task.turn_context`: approval policy, effective permission profile,
approval reviewer and Windows sandbox level. The existing active-turn lock and
session-state lock cover comparison through enqueue, serializing with
`update_settings`. Mismatch returns `SteerInputError::AuthorizationChanged`
before additional-context merge, metadata mutation or input admission.
App-server `TurnRequestProcessor::turn_steer` maps it to an invalid-request error
with the actionable message, without misclassifying it as a different error.
Equal effective authorization (including no-op updates) still admits input.

## Final-tree validation

Dedicated target for every lane:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/pf83-rebind-31`.
Commands below ran from `codex-rs` with that `CARGO_TARGET_DIR`.
Every test command also used `--locked --offline`.
Logs use the `pf83-rebind-31-` prefix beside this receipt; `.log` files
are locally retained and ignored by git. All tests used the checkout's guarded
`just test`; test-isolation guidance was read before dispatch. No live profile
used or native credential prompt observed. Only changed Rust files formatted,
with `rustfmt --edition 2024 --config skip_children=true`.

| Command | Result | Log suffix |
| --- | --- | --- |
| `cargo build --locked --offline -p codex-cli --bin codex` | exit 0 | `build-cli.log` |
| `cargo build --locked --offline -p codex-rmcp-client --bins` | exit 0 | `build-rmcp.log` |
| `just test -p codex-app-server -E 'test(thread_settings_confirmation_f04_restricts_work_steered_after_applied)' --run-ignored only` | 1 passed, 1 flaky; 1096 skipped; exit 0 | `f04-ignored.log` |
| `just test -p codex-app-server settings` | 22 passed (4 slow, 1 leaky), 1 failed; 1074 skipped; exit 100 | `settings.log` |
| `just test -p codex-tui permission` | 91 passed (3 leaky); 4074 skipped; exit 0 | `tui.log` |
| `just test -p codex-core --lib 'session::'` | 331 passed (1 flaky); 2141 skipped; exit 0 | `core-session.log` |
| `just test -p codex-app-server -E 'test(thread_settings_confirmation_leaves_mcp_status_responses_unaffected)' -j 1 --retries 0` | 1 passed; 1096 skipped; exit 0 | `mcp-replay.log` |
| `just test -p codex-app-server settings -j 1 --retries 0` | 23 passed; 1074 skipped; exit 0 | `settings-serial.log` |

Original failed attempts remain evidence:
- F04: first explicit ignored run timed out at 60 seconds; automatic retry passed in 0.536 seconds. After removing only the ignore annotation, it passed without retries in both complete settings runs.
- `suite::v2::thread_settings_update::thread_settings_confirmation_leaves_mcp_status_responses_unaffected`: both initial attempts failed with `deadline has elapsed` at MCP startup before steering. Binary existed; no inherited MCP initialization barrier. Isolated replay passed in 22.473 seconds and complete serial settings replay passed. The cause was not established; this is not a claimed baseline attribution.
- `session::tests::shell_tool_cancellation_waits_for_runtime_cleanup`: first attempt failed with `No such file or directory (os error 2)`; automatic retry passed.

Leaky passes, not hidden failures:
- App-server initial run: `thread_state::settings_confirmation_tests::settings_confirmation_disconnect_only_clears_its_connection`.
- TUI: `app::tests::inactive_thread_permissions_approval_preserves_file_system_permissions`, `app::tests::thread_read_session_state_does_not_reuse_primary_permission_profile`, and `app::thread_session_state::tests::permission_settings_sync_updates_active_snapshot_without_rewriting_side_thread`.
- The final serial settings run had no leak, slow, flaky or failure classifications.

## Acceptance and negative controls

F04 is un-ignored and passing. Its entire function body, including assertion text,
is byte-identical to the base: SHA-256
`c6b986f43d3e5d155e870a9aa1102f209c63559009bf9fc9af59b9ec7e9cc2ff`.
Refusal prevents the new input reaching the old turn's model and prevents the
outside-workspace write; the fixture's independent fresh turn still requires approval.

| Control | Passing test and witness |
| --- | --- |
| (a) Loosening preserves running work's authority | `thread_settings_confirmation_loosening_preserves_running_work_authority`: approved process crosses a filesystem barrier after Applied; its original-work follow-up still requests approval and writes nothing after decline. |
| (b) Granted approval survives later changes | `thread_settings_confirmation_tightening_preserves_granted_approval`: an accepted command reaches a barrier, permissions loosen then tighten, and that same process completes its approved filesystem write after release. |
| (c) Unchanged authorization preserves steering | `steer_input_returns_active_turn_id` and `steer_input_accepts_unchanged_authorization_after_settings_update`: original input admission retained; no-op authorization plus unrelated reasoning update still admits. |
| (d) Both next-turn directions preserved | `thread_settings_confirmation_next_turn_tightening_and_loosening`: observed approval/write tuples remain (0,true), (1,false), (0,true). |

Additional coverage: `steer_input_rejects_changed_authorization_without_mutating_active_work`
checks approval-only, sandbox-only and reviewer-only changes, exact error text,
empty pending input and preserved task/context identity. Existing
`thread_settings_confirmation_preserves_pending_approval` also passes.

## Scope, size and limitations

Production Rust: 26 additions + 4 deletions = **30 changed lines**.
Tests: 191 additions + 1 deletion = **192 changed lines**. Every existing test retained.
QA receipt: 128 added lines; **158 changed lines outside tests** (30 Rust + 128 QA), within the 180 target and 300 hard limit. Raw logs are not source changes.
Final scope audit found exactly five non-ignored changed/untracked files, all allowed.
`git diff --check`, plan checker (3 active/3 slots) and sprint checker (115 current,
127 archived) passed. No workspace-wide formatting, commit or push.

Display half untouched: no settings-confirmation implementation, thread-state,
or TUI display edits. The brief's sandbox description is conceptual: the actual
turn field is an effective `PermissionProfile`, with separate Windows sandbox
level, rather than a directly stored `sandbox_policy`. The acceptance test's
old pending-decision prose remains unchanged as instructed. No other brief
correction was needed beyond the canonical-document drift described above.

This is an implementation return, not independent review, true-TUI acceptance,
code-blind functional qualification, live-repository proof, or release approval.
Those manager-owned gates remain required before an unqualified human handoff.
