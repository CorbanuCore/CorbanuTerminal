# Unaccepted snapshot findings — tui-snapshot-rot-21

These 11 generated `.snap.new` files are evidence, not accepted goldens. The nine
footer diffs add the existing provider suffix and clip the longer line to the
fixture width. The two project-preview diffs use the current preview fallback
(`my-project`) and current-directory basename (`project`) instead of `tmp`.
No baseline attribution or approval of these expectations is claimed. No file
listed below was accepted or used to change its corresponding golden.

## codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__chatwidget_tall.snap

```diff
--- codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__chatwidget_tall.snap
+++ codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__chatwidget_tall.snap.new
@@ -21,4 +21,4 @@
 
 › Ask Corbanu Terminal to do anything
 
-  Ambient GLM 5.2 standard · /tmp/project · Corbanu Terminal · TPS: -- tok/s
+  Ambient GLM 5.2 via ambient standard · /tmp/project · Corbanu Terminal · TPS:…
```

## codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__guardian_goal_continuation_drops_stale_reviews.snap

```diff
--- codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__guardian_goal_continuation_drops_stale_reviews.snap
+++ codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__guardian_goal_continuation_drops_stale_reviews.snap.new
@@ -4,4 +4,4 @@
 
 › Ask Corbanu Terminal to do anything
 
-  Ambient GLM 5.2 standard · /tmp/project · Corbanu Terminal · TPS: -…
+  Ambient GLM 5.2 via ambient standard · /tmp/project · Corbanu Termina…
```

## codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__guardian_parallel_reviews_render_aggregate_status.snap

```diff
--- codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__guardian_parallel_reviews_render_aggregate_status.snap
+++ codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__guardian_parallel_reviews_render_aggregate_status.snap.new
@@ -5,4 +5,4 @@
 
 › Ask Corbanu Terminal to do anything
 
-  Ambient GLM 5.2 standard · /tmp/project · Corbanu Terminal · TPS: -…
+  Ambient GLM 5.2 via ambient standard · /tmp/project · Corbanu Termina…
```

## codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__image_generation_begin_restores_working_status.snap

```diff
--- codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__image_generation_begin_restores_working_status.snap
+++ codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__image_generation_begin_restores_working_status.snap.new
@@ -4,4 +4,4 @@
 "                                                                                "
 "› Ask Corbanu Terminal to do anything                                           "
 "                                                                                "
-"  Ambient GLM 5.2 standard · /tmp/project · Corbanu Terminal · TPS: -- tok/s  "
+"  Ambient GLM 5.2 via ambient standard · /tmp/project · Corbanu Terminal · TPS:…"
```

## codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__preamble_keeps_working_status.snap

```diff
--- codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__preamble_keeps_working_status.snap
+++ codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__preamble_keeps_working_status.snap.new
@@ -4,4 +4,4 @@
 "                                                                                "
 "› Ask Corbanu Terminal to do anything                                           "
 "                                                                                "
-"  Ambient GLM 5.2 standard · /tmp/project · Corbanu Terminal · TPS: -- tok/s  "
+"  Ambient GLM 5.2 via ambient standard · /tmp/project · Corbanu Terminal · TPS:…"
```

## codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__reasoning_delta_restores_recreated_status_indicator.snap

```diff
--- codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__reasoning_delta_restores_recreated_status_indicator.snap
+++ codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__reasoning_delta_restores_recreated_status_indicator.snap.new
@@ -4,4 +4,4 @@
 "                                                                                "
 "› Ask Corbanu Terminal to do anything                                           "
 "                                                                                "
-"  Ambient GLM 5.2 standard · /tmp/project · Corbanu Terminal · TPS: -- tok/s    "
+"  Ambient GLM 5.2 via ambient standard · /tmp/project · Corbanu Terminal · TPS:…"
```

## codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_surface_previews_hardcoded_only.snap

```diff
--- codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_surface_previews_hardcoded_only.snap
+++ codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_surface_previews_hardcoded_only.snap.new
@@ -1,2 +1,2 @@
-status line: tmp · feat/awesome-feature · thread title · Read Only · Ask for approval
+status line: my-project · feat/awesome-feature · thread title · Read Only · Ask for approval
 terminal title: thread title | feat/awesome-feature | Tasks 0/0
```

## codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_surface_previews_mixed.snap

```diff
--- codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_surface_previews_mixed.snap
+++ codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_surface_previews_mixed.snap.new
@@ -1,2 +1,2 @@
-status line: tmp · feature/mixed-preview · Mixed preview thread
-terminal title: tmp | Mixed preview thread | Tasks 0/0
+status line: my-project · feature/mixed-preview · Mixed preview thread
+terminal title: project | Mixed preview thread | Tasks 0/0
```

## codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_widget_active.snap

```diff
--- codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_widget_active.snap
+++ codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__status_widget_active.snap.new
@@ -4,4 +4,4 @@
 "                                                                                "
 "› Ask Corbanu Terminal to do anything                                           "
 "                                                                                "
-"  Ambient GLM 5.2 standard · /tmp/project · Corbanu Terminal · TPS: -- tok/s    "
+"  Ambient GLM 5.2 via ambient standard · /tmp/project · Corbanu Terminal · TPS:…"
```

## codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__unified_exec_begin_restores_working_status.snap

```diff
--- codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__unified_exec_begin_restores_working_status.snap
+++ codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__unified_exec_begin_restores_working_status.snap.new
@@ -4,4 +4,4 @@
 "                                                                                "
 "› Ask Corbanu Terminal to do anything                                           "
 "                                                                                "
-"  Ambient GLM 5.2 standard · /tmp/project · Corbanu Terminal · TPS: -- tok/s  "
+"  Ambient GLM 5.2 via ambient standard · /tmp/project · Corbanu Terminal · TPS:…"
```

## codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__unified_exec_wait_status_renders_command_in_single_details_row.snap

```diff
--- codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__unified_exec_wait_status_renders_command_in_single_details_row.snap
+++ codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__unified_exec_wait_status_renders_command_in_single_details_row.snap.new
@@ -4,4 +4,4 @@
 
 › Ask Corbanu Terminal to do anything
 
-  Ambient GLM 5.2 standard · /tmp/project · Pos…
+  Ambient GLM 5.2 via ambient standard · /tmp/p…
```
