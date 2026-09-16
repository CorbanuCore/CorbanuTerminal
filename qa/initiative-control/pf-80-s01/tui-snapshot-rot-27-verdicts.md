# Snapshot verdicts — tui-snapshot-rot-27

All eleven are stale goldens. Each content edit below was reviewed individually;
no insta acceptance command or update-all setting was used. Assertion-line
metadata is left alone. Final fresh-target execution is recorded in the return.

Provider evidence: commit c19c328da1affd918df1fbcaa60e1545ae58db43 deliberately
introduced model_with_provider_display_name and uses it for both model and
model-with-reasoning status fields (chatwidget/status_surfaces.rs:908–943).
The ambient fixture therefore must display "Ambient GLM 5.2 via ambient standard".
The same commit's custom_model_routes_are_distinguishable_before_selection test
establishes route identity as intentional. Footer rendering uses
truncate_line_with_ellipsis_if_overflow (bottom_pane/footer.rs:1346,1395).
These cases change only the passive footer, not working-state or review rows.

Project evidence: commit 9a2b34213b5530ab3c616400c1e4d355aa78156d explicitly
specifies stable placeholders for missing preview values and title-specific
project fallback. status_surfaces.rs:490–521 resolves a repository/config-layer
root, otherwise the title uses the cwd basename. status_surface_preview.rs:47
specifies "my-project". Existing test
missing_project_root_uses_different_status_and_title_preview_sources asserts
exactly "my-project" / "project". The two snapshot fixtures now explicitly cache
None, preventing host /tmp layout from deciding their root.

All names below have prefix
codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__
and suffix .snap.

| Snapshot | Verdict and individual acceptance reason |
| --- | --- |
| chatwidget_tall | Accept: intentional "via ambient" route label; footer now truncates at "TPS:…". Tall transcript/working layout is unchanged. |
| guardian_goal_continuation_drops_stale_reviews | Accept: intentional route label; 70-column fixture truncates at "Corbanu Termina…". Stale-review cleanup and composer content are unchanged. |
| guardian_parallel_reviews_render_aggregate_status | Accept: same intentional route identity at the 70-column width; aggregate reviewer row remains unchanged. |
| image_generation_begin_restores_working_status | Accept: route label uses footer width, truncating at "TPS:…"; the image-generation working row remains unchanged. |
| preamble_keeps_working_status | Accept: route label and corresponding footer ellipsis; preamble working-state row is unchanged. |
| reasoning_delta_restores_recreated_status_indicator | Accept: route label and footer ellipsis; recreated status indicator remains unchanged. |
| status_surface_previews_hardcoded_only | Accept: no project root means the documented preview placeholder "my-project", not host-derived "tmp"; title fields remain unchanged. Explicit None cache fixes the fixture precondition. |
| status_surface_previews_mixed | Accept: status uses "my-project" placeholder and title uses cwd basename "project"; live branch and thread remain unchanged. Explicit None cache fixes the fixture precondition. |
| status_widget_active | Accept: intentional route label and footer ellipsis; active status widget is unchanged. |
| unified_exec_begin_restores_working_status | Accept: intentional route label and footer ellipsis; restored working row is unchanged. |
| unified_exec_wait_status_renders_command_in_single_details_row | Accept: intentional route label; narrow footer now ends "/tmp/p…" instead of the old "Pos…" fragment. Command remains in one details row. |

No one of these eleven remains unaccepted; no missing product decision is needed
for their described content. This is test maintenance, not a new product change
or fresh acceptance of the implementation's user workflows.
