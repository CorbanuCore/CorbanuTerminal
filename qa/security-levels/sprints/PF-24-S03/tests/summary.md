# PF-24-S03 test results

Commands from `codex-rs` after `just fmt` and `just fix -p codex-tui -p codex-features`, candidate `72735863b6`.

| Run | Result |
| --- | --- |
| `just test -p codex-features` | Summary [   0.038s] 33 tests run: 33 passed, 0 skipped |
| `just test -p codex-tui` (full) | Summary [1550.674s] 4228 tests run: 4207 passed (1 slow, 1 flaky, 1 leaky), 21 failed, 8 skipped |
| Rerun of the 21 failures' groups with `TMPDIR=/tmp/pf24t` on the merged tree `477ea8becc` (`ide_context::ipc default_daemon_auto_connect legacy_daemon_upgrade command_popup security`) | Summary [  13.180s] 45 tests run: 44 passed, 1 failed, 4191 skipped |

The full-run failures are host-only: the socket groups fail with `path must be shorter than SUN_LEN` under the long default `TMPDIR` and pass with a short one; the `multi_provider_onboarding` tmux cases log the same `SUN_LEN` error (not rerun). `default_command_popup_items_snapshot` fails locally on unchanged product strings (`/gpu`, `/security`, Codex vs Corbanu Terminal naming), untouched by this change. All 31 `security` tests pass.

Failed in the full run:

- codex-tui bottom_pane::command_popup::tests::default_command_popup_items_snapshot
- codex-tui chatwidget::wallet_menu::tests::legacy_daemon_upgrade_guidance_is_visible_in_wallet_surface
- codex-tui ide_context::ipc::tests::fetch_ide_context_does_not_fall_back_after_primary_protocol_error
- codex-tui ide_context::ipc::tests::fetch_ide_context_does_not_fall_back_after_primary_timeout
- codex-tui ide_context::ipc::tests::fetch_ide_context_falls_back_to_legacy_socket
- codex-tui ide_context::ipc::tests::fetch_ide_context_falls_back_to_pre_migration_uid_zero_legacy_socket
- codex-tui ide_context::ipc::tests::fetch_ide_context_falls_back_to_uid_zero_legacy_socket
- codex-tui ide_context::ipc::tests::fetch_ide_context_prefers_primary_socket
- codex-tui ide_context::ipc::tests::fetch_ide_context_uses_unregistered_request_route
- codex-tui ide_context::ipc::tests::validate_unix_socket_path_rejects_unsafe_parent_directory
- codex-tui tests::default_daemon_auto_connect_probes_socket_only
- codex-tui::all suite::multi_provider_onboarding::canary_tree_scan_skips_unix_sockets_and_scans_regular_files
- codex-tui::all suite::multi_provider_onboarding::tmux_corbanu_env_aliases_restart_account_read_and_legacy_daemon_recovery
- codex-tui::all suite::multi_provider_onboarding::tmux_corbanu_env_legacy_alias_restart_and_account_read
- codex-tui::all suite::multi_provider_onboarding::tmux_corbanu_env_plan_alias_restart_and_account_read
- codex-tui::all suite::multi_provider_onboarding::tmux_deferred_corbanu_api_cancel_with_fallback_continues_to_chat
- codex-tui::all suite::multi_provider_onboarding::tmux_fresh_wallet_api_cancel_without_fallback_returns_to_setup
- codex-tui::all suite::multi_provider_onboarding::tmux_fresh_wallet_api_handoff_preserves_existing_current_provider
- codex-tui::all suite::multi_provider_onboarding::tmux_fresh_wallet_api_key_success_activates_without_restart
- codex-tui::all suite::multi_provider_onboarding::tmux_locked_wallet_api_unlocks_and_cancels
- codex-tui::all suite::multi_provider_onboarding::tmux_only_corbanu_api_cancel_returns_to_shared_provider_list
