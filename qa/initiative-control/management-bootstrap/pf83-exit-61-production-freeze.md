# PF-83-S01 production surface freeze — pf83-exit-61

Frozen working-tree bytes at `8ad587541e45e519dc0b5181ac5be36ecaf90b13`, branch `bootstrap/pf83-rebind-20260916`, worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/pf83-rebind-20260916`. Initial Git status was clean. This allocation changes evidence only.

“Final for this sprint” means these are the production bytes proposed for the next independent review and packaged qualification; no further production edit is currently planned. It is not review approval, sprint completion, a promise that tests cannot find a defect, or a claim that other tracks cannot change shared files. Any subsequent production correction or integration change invalidates its digest and requires a new freeze and affected evidence. No file is currently marked expected to change. The F03 captured-running-authority comparison remains a separately authorized follow-up; F10 barrier instrumentation, if later needed in production, must reopen the freeze explicitly.

## Derivation and completeness

Read the complete non-merge source history from the sprint base `005cc644f59b1e762e5497b329e106c67925d4ed` through this HEAD, then inspected the changed-file lists for the PF-83 confirmation, unrun-command, admission/rebinding, wording and native-fixture commits below. This inventory is the union of their per-commit changed paths, not just a net diff (which could hide reverted touches), and not the older return inventories or the sprint's reservation list. For each commit use `git diff-tree --no-commit-id --name-only -r <commit>`; retain `codex-rs/` paths, exclude test/snapshot-only paths listed below, and SHA-256 the current file bytes. Tests embedded in production modules do not exclude their containing file.

The union contains **71 paths: 34 production/contract files below and 37 test-only paths below**. The 34 comprise 27 Rust production files, six generated protocol contract artifacts and one API README. Generated compressed exports are hashed as the actual binary file bytes. All 34 exist; none of the production paths is deleted. SHA-256 is of the whole current file, including integrated work from other tracks.

Source commits (full identities; test-only commits retained to make the audit explicit):

- `03503976785551a5f21df399fa71e7298b460616` — fix: confirm permission changes and preserve next-turn state
- `4fd27d0b6bb0dd2eb947ce60b50dbc2d1b21d0e5` — test(tui): cover PF-83 confirmation state through event dispatch
- `f655e60d7db92572433b5a5d4b133206d5ef5e2c` — fix(tui): preserve declined command outcomes in history
- `aff87db80be708dd20a1293c9f84da7ec7159b33` — fix(tui): preserve unknown command execution outcomes
- `cc1ad504061f3471322c09e7d4ee2a79f02fe1e9` — test(tui): refresh unconfirmed interruption snapshot
- `44729e28937c3e75a6c786e8091ed64c9f486277` — Reproduce F04 natively: work steered into a running turn after Applied runs unapproved
- `bcce23dcd51b8a740da9c46e3c3a989b9b68f0f3` — Make the F04 reproduction landable and make it claim only what it proves
- `18305db86b779f9bbebf9c75b09c3ffa467d4359` — Re-bind newly admitted work to the latest authorisation
- `23915e5c6f06df102ee500e5b0ccc81e64fe315c` — Defer the message instead of destroying it, and gate the mailbox and inject paths too
- `52af24987eb7bb27ce7a661f6c58ceb4e4536551` — Gate the extension admitter only, and stop the deferral losing schema or bypassing admission
- `a802683f4b1d2c25a5770ecbd942ba663291d3bf` — Gate the last external admitter, unbreak waiting, and stop losing the caller schema
- `44bbf0db17ac35adb00637b84a2f1300fe467eb8` — Close the mail waiter; record what the remaining two repairs need
- `d1f6589dc12e68b9c2e1ba70307ef358e7a830e5` — Gate the public route, carry the schema on natural completion, and table every admission path
- `7995a073cbfdce1a6a70759bc28d76afe47b9f49` — Say what the product now actually does with a prompt sent mid-turn
- `865a4a094d091043d0615cef1f861542b9a16f01` — Tie the permission wording to the retention behaviour it describes
- `101937a8996362cc86bc495d737d48ee1ab1aeba` — Delete the retired wording, and say what F03 actually is now
- `c124a3a1c42712ce36265910cda24cc9520f7c12` — Take F05, F06, F07 and F09 natively; say exactly what F10 and F11 still need
- `63e2fc6652a436745caaef4f6f8013de04ab12ef` — Make the F10 restricted branch refutable and delete the fixtures that could not fail
- `8ad587541e45e519dc0b5181ac5be36ecaf90b13` — Reject any escalation, not just the extremes, and name the F10(c) half for what it proves

Do not use the sprint's write_scope as a touched-file list: it misses actual later production edits to `core/src/codex_thread.rs`, `core/src/session/{handlers,inject,input_queue}.rs`, `core/src/tasks/mod.rs`, `ext/goal/src/runtime.rs`, and `tui/src/status/card.rs`; conversely, reservation alone does not prove a change. Previously released shared paths `tui/src/app/event_dispatch.rs` and `tui/src/app_event.rs` remain included because PF-83 did touch them. `core/src/session/turn.rs` changed on accounting commits, not the PF-83 commits, and is not credited to this sprint. The independent PF-80 snapshot-normalization work, test-isolation infrastructure and other accounting/resume/packaging tracks are not PF-83 production changes. The native qualification harness resides outside this repository; it is not part of this production-source inventory or a newly frozen package.

## File digests and verdicts

| Production/contract file (repository relative) | Current SHA-256 | Sprint verdict |
| --- | --- | --- |
| `codex-rs/app-server-protocol/schema/json/codex_app_server_protocol.schemas.json` | `d3da6af9a26270a43f61bff31f67c56f9d75b68bafc9740d01c667c1d8f7f647` | Final for this sprint |
| `codex-rs/app-server-protocol/schema/json/codex_app_server_protocol.v2.schemas.json` | `8d69da934937b952b2a27ceb1662edf966b56b6b303045f3c14d29f1a8c57efe` | Final for this sprint |
| `codex-rs/app-server-protocol/schema/precomputed/app-server-exports-experimental.json.zst` | `e208b950ecf381512016e4bc8fd54d8bd46e020565cd7beacd9e241d5c71ce2c` | Final for this sprint |
| `codex-rs/app-server-protocol/schema/precomputed/app-server-exports-stable.json.zst` | `b348764d7c34c6e5adc4760f073a00a99a63621dddf65206ac126b50b714a3a2` | Final for this sprint |
| `codex-rs/app-server-protocol/schema/typescript/v2/ThreadSettingsUpdateOutcome.ts` | `9b2e5a04bcacbbf7511cc2ebcf20c2e25aeae20e2a1c6a26c173aba28acb9a1e` | Final for this sprint |
| `codex-rs/app-server-protocol/schema/typescript/v2/index.ts` | `de4d07da1d7839f188617cf5e5d96f71c2b0508c7f106614a85887476f88bff6` | Final for this sprint |
| `codex-rs/app-server-protocol/src/protocol/v2/thread.rs` | `0942f01f3b5987ef7c4021d8517613be8f41367ab63135b7ccaaceb0aee9eb53` | Final for this sprint |
| `codex-rs/app-server/README.md` | `42ca64f28fe8cbac226ffce1705aa1f23774424c95c32f10c611300a8cac894f` | Final for this sprint |
| `codex-rs/app-server/src/bespoke_event_handling.rs` | `1e4cd026370957358b8a0d2fed73128a9cf6d0aa622cd31bb12d303ea9675f6b` | Final for this sprint |
| `codex-rs/app-server/src/lib.rs` | `9993e42831e728b06449f83da0d3315c3a8622de7c7d2e1dc5e9b978977d746d` | Final for this sprint |
| `codex-rs/app-server/src/request_processors/turn_processor.rs` | `28dfbd1998a3c975de2f695843725b58a8e0b73c365aef7fca88f5a03292217d` | Final for this sprint |
| `codex-rs/app-server/src/settings_confirmation.rs` | `d5e9fdec936ffc4b8f146cb441200a50c7d6907953f04518b4b8969fa16879c1` | Final for this sprint |
| `codex-rs/app-server/src/thread_state.rs` | `d2be1fea01470835a1fd69a1c88b4aacbe5a9d09cdadde20b824b70852f12d96` | Final for this sprint |
| `codex-rs/core/src/codex_thread.rs` | `5466396d3c7c83686300f3a4c8018e3a646a24666e48d030415414f79d346d3f` | Final for this sprint |
| `codex-rs/core/src/session/handlers.rs` | `83609a7c7d32ed6c1118d7552a1f104deefded6cc0e00c02ed03eaabb60f4a59` | Final for this sprint |
| `codex-rs/core/src/session/inject.rs` | `9bbcc1ffcc0a82ad01625ec4b025f046353ff6f4b6ae4b03efc589d15ba3a4d6` | Final for this sprint |
| `codex-rs/core/src/session/input_queue.rs` | `f5074c3df786f96bdf3adaa14ab4412fcb8ab5164f466647111e101c69434469` | Final for this sprint |
| `codex-rs/core/src/session/mod.rs` | `75dd04620eb4c5bf6d61f4a1992d411bb0e049d276d6d1f44daf4c18a1002192` | Final for this sprint |
| `codex-rs/core/src/tasks/mod.rs` | `29c5f4d702db6583bea8cc33bea0922e28866f5dc4c16127399ee42716497c30` | Final for this sprint |
| `codex-rs/ext/goal/src/runtime.rs` | `72c16e0a02fd20f1f32b2a43f84408dac367bae15e7d1048dddca4ccacb0114b` | Final for this sprint |
| `codex-rs/tui/src/app.rs` | `941c761e9dabfa5eeb0d55424efcb32c0bc618065d43a7ebd80273d2b4a7e8c5` | Final for this sprint |
| `codex-rs/tui/src/app/config_persistence.rs` | `27189b296bd0490005a7520945f659f3d4bd89956617687c7b6b9303e03e0dc2` | Final for this sprint |
| `codex-rs/tui/src/app/event_dispatch.rs` | `29a2eb4160416fd76c3fb32a25705ab322fa94ae7da4186c878489f554d19a77` | Final for this sprint |
| `codex-rs/tui/src/app/permission_confirmation.rs` | `e3a1573d15cd80297c25e36f9b7e8c03f1ff9c92bed26fa0cb4f5bbdcd6fc3c9` | Final for this sprint |
| `codex-rs/tui/src/app/thread_routing.rs` | `3ca8cd25a056de3ce5363c7ecc986eb2365ecc67f54af81068b7ebb24d2f1451` | Final for this sprint |
| `codex-rs/tui/src/app_event.rs` | `e02b4548da5608f48c1cf6a667c9ff13d50cf32b7c89e0f19bd3040936eec51f` | Final for this sprint |
| `codex-rs/tui/src/app_server_session.rs` | `8ecdf3c14a69b85fc9c0b2d2f98b0286568745e495c71815bebfd0d33e0ddaeb` | Final for this sprint |
| `codex-rs/tui/src/chatwidget/command_lifecycle.rs` | `c001079a4ffed65a2909a91dc0f4f02838e04347a6f84012f5327b585fdbe8f0` | Final for this sprint |
| `codex-rs/tui/src/chatwidget/input_restore.rs` | `c4665f354a1d6afb1190c1299753cae8203a2eba19cbbebb1c1ac55d31752d76` | Final for this sprint |
| `codex-rs/tui/src/chatwidget/permission_popups.rs` | `939a4f0ec21feb4af756211f38ec3264892cce4a7143fe6e79a41597bbfa8e7e` | Final for this sprint |
| `codex-rs/tui/src/chatwidget/settings.rs` | `16d2b8ec609c17cadd5ad4de2a761dac728c6d07286a11bfcd9eda36e9ac46e2` | Final for this sprint |
| `codex-rs/tui/src/exec_cell/model.rs` | `96838288303c9742cb755c9c99b3227f461392772715a513c6969bd53b30befb` | Final for this sprint |
| `codex-rs/tui/src/exec_cell/render.rs` | `37b180476494a0d9534ce004b3ef2985806d780361716ec65ecef7985a25f1a0` | Final for this sprint |
| `codex-rs/tui/src/status/card.rs` | `9814de2ad76a48c9512b04b61988771eeb8296394a4fd7dde7e6c24cc30786c4` | Final for this sprint |

## Test-only paths excluded from the production count

These are the complete remaining paths from the same union, including historical snapshot deletions. They are listed for an auditable classification; no claim that deleted snapshots still exist.

- `codex-rs/app-server/src/settings_confirmation_tests.rs`
- `codex-rs/app-server/tests/suite/v2/thread_settings_update.rs`
- `codex-rs/core/src/session/tests.rs`
- `codex-rs/tui/src/app/permission_confirmation_tests.rs`
- `codex-rs/tui/src/app/snapshots/codex_tui__app__permission_confirmation__tests__permission_confirmation_outcomes_are_explicit.snap`
- `codex-rs/tui/src/app/snapshots/codex_tui__app__permission_confirmation__tests__permission_confirmation_superseded.snap`
- `codex-rs/tui/src/app/test_support.rs`
- `codex-rs/tui/src/app/tests.rs`
- `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__interrupt_exec_marks_failed.snap`
- `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__permissions_selection_history_after_mode_switch.snap`
- `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__permissions_selection_history_full_access_to_default.snap`
- `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__permissions_selection_history_full_access_to_default@windows.snap`
- `codex-rs/tui/src/chatwidget/tests/approval_requests.rs`
- `codex-rs/tui/src/chatwidget/tests/permissions.rs`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_permissions_distinguish_next_turn_from_unreported_running_authority.snap`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_cached_limits_hide_credits_without_flag.snap`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_includes_credits_and_limits.snap`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_includes_enterprise_monthly_credit_limit.snap`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_includes_forked_from.snap`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_includes_monthly_limit.snap`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_includes_reasoning_details.snap`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_shows_active_user_defined_profile.snap`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_shows_auto_review_permissions.snap`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_shows_chatgpt_plan_without_email.snap`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_shows_missing_limits_message.snap`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_shows_refreshing_limits_notice.snap`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_shows_stale_limits_message.snap`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_shows_unavailable_limits_message.snap`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_treats_refreshing_empty_limits_as_unavailable.snap`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_truncates_halfwidth_kana_in_narrow_terminal.snap`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_truncates_in_narrow_terminal.snap`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_uses_command_backed_provider_account_identity.snap`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_uses_default_reasoning_when_config_empty.snap`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_uses_generic_limit_labels_for_unsupported_windows.snap`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_wraps_enterprise_monthly_credit_details_in_narrow_terminal.snap`
- `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__transcript_overlay_status_rate_limit_refresh.snap`
- `codex-rs/tui/src/status/tests.rs`

The four required regression lanes and the focused picker/route lane are recorded in [this allocation's return](pf83-exit-61-return.md). They do not exhaust tests for every frozen file. No fresh production review, exact-package execution, live-repository qualification, human acceptance or release is claimed. The sprint file itself is outside this worker's writable scope; the manager can link this freeze without treating its remaining review/functional gates as complete.
