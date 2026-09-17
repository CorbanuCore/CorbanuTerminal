# Snapshot verdicts — tui-snapshot-rot-32

Base: dc7f8da90d2b9ff34d8c43cd12afb15239619d7c. The eleven individual
acceptances and the fixes to normalization were already committed in this base
by the interrupted prior worker. This continuation independently checked their
diffs, current rendering paths and originating changes; it did not re-accept
them in bulk or overwrite the old attempts.

Class: routine test maintenance. Product context: **Product principles**,
“Distinguish Corbanu-controlled inference from third-party inference at selection
and use” and “Maintain continuous Codex parity without removing Corbanu-specific
behavior.” No new product contract or production behavior is implemented.
Plan/sprint, live-repository workflows, benchmarks, named-human acceptance and
independent functional handoff are N/A for this test-only change. This record
does not qualify a product candidate or waive those gates for later user-facing
work; the manager/integrator retains acceptance of this N/A assessment.

## Evidence for the eleven decisions

Provider identity was intentionally added by
c19c328da1affd918df1fbcaa60e1545ae58db43 in
`chatwidget/status_surfaces.rs`: both model status forms use
`model_with_provider_display_name`, which appends `via {provider}` for
non-OpenAI providers. The same commit adds
`custom_model_routes_are_distinguishable_before_selection`, explicitly
requiring distinguishable route labels. `bottom_pane/footer.rs` truncates
overflow with an ellipsis. None of the nine footer-only changes alters the
tested working-state/review/command rows.

Project preview behavior originates in
9a2b34213b5530ab3c616400c1e4d355aa78156d, whose description explicitly requires
stable placeholders when live values are missing and title-specific project
fallback. Current `bottom_pane/status_surface_preview.rs` selects
`my-project`; `chatwidget/status_surfaces.rs` falls back to the current
directory basename for titles. Existing
`missing_project_root_uses_different_status_and_title_preview_sources`
asserts the exact pair `my-project` / `project`.
The two snapshot tests now explicitly cache a missing project root, matching
their intended fixture instead of depending on the host's /tmp layout.

Each name below is a separate file under
`codex-rs/tui/src/chatwidget/snapshots/`, prefixed
`codex_tui__chatwidget__tests__` and suffixed `.snap`.

| Snapshot | Verdict | Reason |
| --- | --- | --- |
| chatwidget_tall | Retain acceptance | Adds intentional `via ambient`; longer footer ends `TPS:…`. Tall transcript and working rows are unchanged. |
| guardian_goal_continuation_drops_stale_reviews | Retain acceptance | Adds route identity; 70-column footer ends `Corbanu Termina…`. Stale-review cleanup content is unchanged. |
| guardian_parallel_reviews_render_aggregate_status | Retain acceptance | Same deliberate route label and 70-column clipping; aggregate reviewer row is unchanged. |
| image_generation_begin_restores_working_status | Retain acceptance | Adds route label and corresponding `TPS:…` clipping; image-generation working row is unchanged. |
| preamble_keeps_working_status | Retain acceptance | Adds route label and corresponding footer clipping; preamble working row is unchanged. |
| reasoning_delta_restores_recreated_status_indicator | Retain acceptance | Adds route label and footer clipping; recreated working indicator remains unchanged. |
| status_surface_previews_hardcoded_only | Retain acceptance | With an explicit missing root, status preview correctly uses `my-project` instead of host-derived `tmp`; title remains unchanged. |
| status_surface_previews_mixed | Retain acceptance | Missing root gives status placeholder `my-project` and title cwd basename `project`; live branch and thread stay intact. |
| status_widget_active | Retain acceptance | Adds intentional route identity and footer clipping; active status widget remains unchanged. |
| unified_exec_begin_restores_working_status | Retain acceptance | Adds intentional route identity and footer clipping; restored working row remains unchanged. |
| unified_exec_wait_status_renders_command_in_single_details_row | Retain acceptance | Route identity consumes the narrow footer, ending at `/tmp/p…`; command is still rendered in one details row. |

All eleven have a reasoned acceptance. None awaits missing product information.
Fresh execution results are recorded separately in the return receipt.

## Normalizer audit

The inherited helper matches only the compiled version in session headers or
`Update available! {version} -> `, replacing it with `<V>`. It retains the
target version, arrow, unrelated versions and malformed/wrong current versions.
The three update-history goldens are the pnpm, standalone_unix and
standalone_windows files; all three now use `<V>`. Only the two standalone
files have current test functions. The pnpm golden is orphaned: there is no
pnpm snapshot assertion, and `UpdateAction` now has only StandaloneUnix and
StandaloneWindows variants. The claim that all three still break on every
bump is therefore inaccurate for the current source. No pnpm execution pass
is claimed.

The Unix/macOS early return in `normalized_backend_snapshot` already calls
`normalize_snapshot_paths`. Its regression test covers both header and update
notice shapes.

The marker is three ASCII columns, shorter than every valid Cargo version
(minimum `0.0.0`, five columns). Reclaimed columns are added as padding, so
zero existing padding cannot cause line growth. Existing tests cover 0–3
trailing spaces, four version lengths, both shapes and both frame positions.
This continuation adds a fixed-width update-notice test across those versions,
complementing the fixed-width header test. Flush output is handled; it is not
assumed impossible.

The two app snapshots containing `<VERSION>` use explicitly injected fixture
versions via `clear_ui_header_lines_with_version`; they are intentionally
stable and are not missing normalizer conversions.

## Additional unresolved version-bearing golden

`codex-rs/tui/src/snapshots/codex_tui__update_prompt__tests__update_prompt_modal.snap`
still contains `0.1.1`. This is not fixture-injected: `update_prompt.rs:115`
initializes the current version from `CARGO_PKG_VERSION`, and its snapshot
assertion at line 271 bypasses normalization. Thus the brief's three notices
are not the complete set of release-sensitive goldens.

Fixing that assertion requires write scope for
`codex-rs/tui/src/update_prompt.rs`, which this allocation expressly excludes.
The golden also contains old release URLs and an npm action while its fixture
uses StandaloneUnix; those differences need individual source-backed review
before acceptance. The modal golden is left unchanged. Its module is also gated by
`#![cfg(not(debug_assertions))]`, so the guarded debug lane excludes this test.
The focused filter requested it but selected only the two standalone history
tests. No modal failure or pass is claimed from execution; this finding is
source-based. Validation needs the separately isolated release/native lane,
which the ordinary test wrapper deliberately rejects. The broader “no golden
encodes a release-changing value” objective remains incomplete for this
out-of-scope, release-only assertion.
