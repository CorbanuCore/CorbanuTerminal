# Bounded repair worker checkpoint

2026-09-14 UTC. Governing record: [bounded-fix-scope.md](bounded-fix-scope.md). **No source patch or candidate.** Parent's explicit existing-ACK insufficiency stop condition applies. Owner coordination approves standalone bounded work only and is not review/acceptance; PF27 remains paused/reserved, with no reported active broker/child collision.

## Source proof of the unresolved seam

All references below are at verified base `005cc644f59b1e762e5497b329e106c67925d4ed` in the dedicated worktree. These are source observations, not runtime reproduction claims.

| Source | Observed behavior and implication |
| --- | --- |
| `codex-rs/tui/src/chatwidget/permission_popups.rs:241` | `approval_preset_actions` queues an override, local policy/profile/reviewer mutations and `Permissions updated` history without applied confirmation. |
| `codex-rs/tui/src/app/config_persistence.rs:173` | Named-profile selection validates a candidate locally, mutates app/widget/runtime overrides and cached session, then queues the override and success history. Local validation is not backend application. |
| `codex-rs/tui/src/app/thread_settings.rs:175` | `send_thread_settings_update` only reports failure of the RPC call. It has no selection-specific applied completion. |
| `codex-rs/tui/src/app_server_session.rs:1175` | `thread_settings_update` returns `Result<()>`; an unsupported method also returns `Ok(())` and downgrades capability. Even RPC success cannot uniformly mean submission, much less application. |
| `codex-rs/app-server/src/request_processors/turn_processor.rs:489` and `:890` | `submit_core_op` returns the Core operation ID. `thread_settings_update_inner` discards that result after submission and returns an empty response. It does not wait for settings application. |
| `codex-rs/core/src/session/handlers.rs:133` and `:211` | Core applies settings, then separately reads the current session snapshot and emits `ThreadSettingsApplied` with `Event.id = sub_id`. A failed apply emits `Error(BadRequest)` with the same Core operation ID. The Core event has an ID, but it is not delivered as a permission-selection acknowledgement to TUI. |
| `codex-rs/app-server/src/bespoke_event_handling.rs:1186` | Settings handling converts the snapshot and emits `ThreadSettingsUpdated` only when `note_thread_settings` reports a change. It omits the Core event ID. |
| `codex-rs/app-server/src/thread_state.rs:181` and `:233` | Equality deduplication suppresses unchanged snapshots. Existing test `note_thread_settings_reports_only_effective_changes` expects `[true, false, true, false]` for repeated initial/updated states. Notification counting cannot track operation completion. |
| `codex-rs/app-server-protocol/src/protocol/v2/thread.rs:379` and `:406` | Response is `{}`. Notification has only `thread_id` and `thread_settings`, with no operation/request ID or revision. |
| `codex-rs/app-server/src/bespoke_event_handling.rs:916` and `:1610` | Asynchronous BadRequest is sent through generic `ErrorNotification`, using the Core event ID as `turn_id`. It is not returned as the already-completed settings RPC's error. The TUI does not know that Core operation ID. |
| `codex-rs/tui/src/app/thread_routing.rs:1166` and `codex-rs/tui/src/chatwidget/settings.rs:401` | Notifications update the cached session and widget by thread identity. Neither handler establishes selection identity. |

Concrete ambiguity: a delayed settings event for a previous Full Access state can arrive while a new Full Access selection is pending after intervening changes. Its thread and values match the new request, but it does not acknowledge that request. A model/reasoning update can also emit the same permission values. Serializing local picker clicks alone cannot disambiguate already queued events or other settings producers. A valid same-state application can emit no new notification, and a failed application cannot be reliably mapped for rollback. No timeout or idle observation resolves these facts.

Checking a later session snapshot could justify describing that observed session state; it would not prove this selection was applied or establish the running turn's authority. A wording-only partial patch would leave the requested correlated pending/success/failure behavior unresolved. The parent explicitly instructed this worker to return that finding rather than invent correlation or widen the implementation boundary.

The smallest unresolved requirement is **an applied outcome tied to the submitted selection, including failure and unchanged-state completion, available to the TUI**. Its existing transport delivery is insufficient. No wire/Core/app-server fix is proposed or attempted here. Parent review/classification is required for any work outside the supplied TUI-only authority.

## Current-turn and next-turn findings

`codex-rs/core/src/session/turn_context.rs:824` clones the latest session configuration under the state lock, applies turn overrides, and carries the resulting configuration into the new turn. Existing `core/tests/suite/prompt_caching.rs::overrides_turn_context_but_keeps_cached_prefix_and_key_constant` submits a settings change before a second turn and checks changed permission context. This is a fixture route for future focused regression work, not evidence that both permission directions, command admission, or continuation have passed here.

`core/src/session/tests.rs::refreshed_mcp_binding_captures_current_approval_authority` explicitly preserves the old turn's approval policy while checking refreshed MCP authority. `mcp_elicitation_reviewer_uses_latest_runtime_authority` exercises reviewer refresh during active work. Both remain unchanged. A future UI repair must distinguish selected/session settings from the current turn's approval/sandbox snapshot and avoid claiming all services defer until next turn. No global execution-quiescence claim is made.

## Verification ledger

- Coordinates: assigned branch and HEAD match. `git merge-base --is-ancestor 413492cd6c3a4d4f8dff6f406247ccda5a9d88aa HEAD` succeeded.
- Frozen case SHA256 rechecked: `c97bf701cd7aff2e26f08884f35dbc9a4fc33b1eef14eee6328d2ea1f1411726`; original unchanged.
- Pinned toolchain: `rustc 1.95.0 (59807616e 2026-04-14)`; `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`.
- Governance checks passed: plans `active 3/3; available slots 0`; sprints `current 115; archived 126`.
- Existing governance unit tests passed: plans 5 tests; sprints 22 tests. These do not validate runtime permissions.
- Focused existing deduplication unit test **passed** using the exact command below: 1 passed, 260 skipped; build 2m 04s, test 0.024s, nextest summary 0.028s. Exit 0. Nextest run `146f6464-7bad-4546-a8a8-e5e3ae6096ed`; worker command session `55099` completed normally. This confirms the existing deduplication behavior, not end-to-end permission correctness.

```sh
RUSTUP_TOOLCHAIN=1.95.0 CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/worktrees/live-permission-transition-20260913/.codex-work/live-permission-transition-target just test -p codex-app-server --lib --locked --offline -E 'test(note_thread_settings_reports_only_effective_changes)'
```

This selects one pure existing unit test, not a full crate suite or whole workspace. Its cold compilation uses only the assigned private target. No installs, dependency changes, real credentials, inference, live-app interaction, package replacement, commits, pushes, or merges are part of this run. Source is unchanged, so no formatting/fix pass is warranted.

Final change accounting: **2 new QA Markdown files, 115 added lines, 0 removed lines; 0 source/test files and 0 source/test LOC changed**. `git diff --numstat` is empty because all product tracked files remain unchanged and the QA files are untracked. The private build cache also appears as untracked `.codex-work/` and is retained solely as build output; it is not a source change or distributable candidate. Prior QA files were not edited. `git diff --check` passed for tracked files; both new records were read back before handoff. No owned test process remains running.

Exploratory searches included nonexistent `app-server-client/src/transport.rs` and `core/src/codex.rs`; these were source-location misses, not test results. Actual relevant client and event paths were read directly.

## Frozen-case coverage and outstanding gates

| Cases | Current disposition |
| --- | --- |
| F01/F02 idle changes, both directions | Unexecuted; selection acknowledgement blocks this candidate. Both-direction command behavior remains unproven. |
| F03/F04 active changes, both directions | Unexecuted. Next-turn scope was authorized as interim work, but original admission/safety expectations remain preserved. No unilateral waiver. |
| F05 pending approvals | Unexecuted. No approval handling was changed; that is not a behavioral pass. |
| F06 in-flight operations | Unexecuted. No quiescence, cancellation, process, or revocation implementation is authorized. Original case remains open. |
| F07 rapid/conflicting selections | Unexecuted; concrete source-level acknowledgement ambiguity above directly affects this requirement. |
| F08 failure/cancellation | Unexecuted; asynchronous failure cannot be tied to this selection through the existing TUI API. |
| F09 continuation/new turn | Unexecuted. Existing fixtures inspected; no two-direction runtime result or Core behavior change. |
| F10 restart/recovery | Unexecuted; no persistence change or coverage waiver. |
| F11 route neutrality | Unexecuted; unsupported settings-RPC fallback is an additional compatibility concern. |

Independent Fable code review, enforced code-blind execution, independent evidence review, exact-package true-TUI proof, applicable live-repository qualification, human acceptance, and release/benchmark assessment have not occurred for a candidate. Coordination is not any of these gates. Prior QA originals are retained. Parent receives a concrete insufficiency finding; no human-ready or runtime-fixed claim is made.
