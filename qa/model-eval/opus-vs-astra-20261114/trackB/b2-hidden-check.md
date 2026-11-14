# Hidden check — task B2 (#2102)

Reference tests = the merged fix's test file and its snapshot, added unchanged as an extra test module (the engineers never saw them). It contains 2 tests: restored_openai_appears_in_already_open_model_picker (snapshot; the picker was opened while only Claude Plan was available, then OpenAI is restored; expected: the open picker refreshes but stays on the Claude Plan tab the person was looking at) and provider_manager_late_refresh_updates_picker_without_reopening_dismissed_menu (already existed on the base). Own tests = each solution's regression tests at its final commit (expected pass) and at its test-only commit (expected fail).

Clippy re-run by us with the pinned toolchain 1.99.1: `cargo clippy -p codex-tui --all-targets -- -D warnings` passes (exit 0) on both solutions.

## [opus]
```
## hidden: merged #2103 provider_picker_refresh.rs + snapshot added unchanged as module provider_picker_refresh_ref, on [opus] HEAD 8e1cb654c6
test app::tests::provider_picker_refresh_ref::provider_manager_late_refresh_updates_picker_without_reopening_dismissed_menu ... ok
test app::tests::provider_picker_refresh_ref::restored_openai_appears_in_already_open_model_picker ... FAILED
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ Snapshot Summary ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Snapshot file: tui/src/app/tests/snapshots/codex_tui__app__tests__provider_picker_refresh_ref__model_picker_openai_restored_while_open.snap
Snapshot: model_picker_openai_restored_while_open
-old snapshot
+new results
    2       │-  Claude Code plan
          2 │+  OpenAI Codex plan
    4       │-  OpenAI  [Claude Plan]
          4 │+  [OpenAI]  Claude Plan
    6       │-› 1. Claude Opus 5.5 Plan   Model: claude-opus-5-5-plan. Claude Opus 5.5 through Claude Code subscription auth in
    7       │-                            Corbanu Terminal.
    8       │-  2. Claude Opus 5 Plan     Model: claude-opus-5-plan. Claude Opus 5 through Claude Code subscription auth in Corbanu
    9       │-                            Terminal.
   10       │-  3. Claude Fable 5.1 Plan  Model: claude-fable-5-1-plan. Claude Fable 5.1 through Claude Code subscription auth in
   11       │-                            Corbanu Terminal.
   12       │-  4. Claude Fable 5 Plan    Model: claude-fable-5-plan. Claude Fable 5 through Claude Code subscription auth in
   13       │-                            Corbanu Terminal.
          6 │+› 1. GPT-6 Astra    Model: gpt-6-astra. Advanced reasoning and coding. Requires eligible OpenAI account access.
          7 │+  2. GPT-6.1 Sol    Model: gpt-6.1-sol. Newer GPT-6 Sol: near-Astra reasoning and coding at Sol pricing. Requires
          8 │+                    eligible OpenAI account access; ChatGPT sign-in may reject it.
          9 │+  3. GPT-6 Sol      Model: gpt-6-sol. Cost-efficient high-end GPT-6 reasoning and coding, below Astra and above Luna.
         10 │+                    Requires eligible OpenAI account access; ChatGPT sign-in may reject it.
         11 │+  4. GPT-5.6-Sol    Model: gpt-5.6-sol. Latest frontier agentic coding model.
         12 │+  5. GPT-6 Luna     Model: gpt-6-luna. Efficient GPT-6 model for focused coding and high-volume tasks.
         13 │+  6. GPT-5.6-Terra  Model: gpt-5.6-terra. Balanced agentic coding model for everyday work.
         14 │+  7. GPT-5.6-Luna   Model: gpt-5.6-luna. Fast and affordable agentic coding model.
thread 'app::tests::provider_picker_refresh_ref::restored_openai_appears_in_already_open_model_picker' (66249249) panicked at /Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/insta-1.48.0/src/runtime.rs:719:13:
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 4780 filtered out; finished in 0.40s
error: test failed, to rerun pass `-p codex-tui --lib`

## own tests (provider_picker_refresh) at HEAD 8e1cb654c6 [opus]
test app::tests::provider_picker_refresh::provider_manager_late_refresh_updates_picker_without_reopening_dismissed_menu ... ok
test app::tests::provider_picker_refresh::late_provider_recovery_refreshes_open_model_picker_and_keeps_tab ... ok
test app::tests::provider_picker_refresh::late_provider_recovery_leaves_dismissed_picker_and_reasoning_dialog ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 4777 filtered out; finished in 1.80s
## own tests at test-only commit 300de03efe (expect FAIL)
test app::tests::provider_picker_refresh::provider_manager_late_refresh_updates_picker_without_reopening_dismissed_menu ... ok
test app::tests::provider_picker_refresh::late_provider_recovery_leaves_dismissed_picker_and_reasoning_dialog ... ok
test app::tests::provider_picker_refresh::late_provider_recovery_refreshes_open_model_picker_and_keeps_tab ... FAILED
thread 'app::tests::provider_picker_refresh::late_provider_recovery_refreshes_open_model_picker_and_keeps_tab' (66260444) panicked at tui/src/app/tests/provider_picker_refresh.rs:159:5:
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 4777 filtered out; finished in 1.86s
error: test failed, to rerun pass `-p codex-tui --lib`
```

## [astra]
```
## hidden: merged #2103 provider_picker_refresh.rs + snapshot added unchanged as module provider_picker_refresh_ref, on [astra] HEAD a0a8daca79
test app::tests::provider_picker_refresh_ref::provider_manager_late_refresh_updates_picker_without_reopening_dismissed_menu ... ok
test app::tests::provider_picker_refresh_ref::restored_openai_appears_in_already_open_model_picker ... FAILED
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ Snapshot Summary ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Snapshot file: tui/src/app/tests/snapshots/codex_tui__app__tests__provider_picker_refresh_ref__model_picker_openai_restored_while_open.snap
Snapshot: model_picker_openai_restored_while_open
-old snapshot
+new results
    2       │-  Claude Code plan
          2 │+  OpenAI Codex plan
    4       │-  OpenAI  [Claude Plan]
          4 │+  [OpenAI]  Claude Plan
    6       │-› 1. Claude Opus 5.5 Plan   Model: claude-opus-5-5-plan. Claude Opus 5.5 through Claude Code subscription auth in
    7       │-                            Corbanu Terminal.
    8       │-  2. Claude Opus 5 Plan     Model: claude-opus-5-plan. Claude Opus 5 through Claude Code subscription auth in Corbanu
    9       │-                            Terminal.
   10       │-  3. Claude Fable 5.1 Plan  Model: claude-fable-5-1-plan. Claude Fable 5.1 through Claude Code subscription auth in
   11       │-                            Corbanu Terminal.
   12       │-  4. Claude Fable 5 Plan    Model: claude-fable-5-plan. Claude Fable 5 through Claude Code subscription auth in
   13       │-                            Corbanu Terminal.
          6 │+› 1. GPT-6 Astra    Model: gpt-6-astra. Advanced reasoning and coding. Requires eligible OpenAI account access.
          7 │+  2. GPT-6.1 Sol    Model: gpt-6.1-sol. Newer GPT-6 Sol: near-Astra reasoning and coding at Sol pricing. Requires
          8 │+                    eligible OpenAI account access; ChatGPT sign-in may reject it.
          9 │+  3. GPT-6 Sol      Model: gpt-6-sol. Cost-efficient high-end GPT-6 reasoning and coding, below Astra and above Luna.
         10 │+                    Requires eligible OpenAI account access; ChatGPT sign-in may reject it.
         11 │+  4. GPT-5.6-Sol    Model: gpt-5.6-sol. Latest frontier agentic coding model.
         12 │+  5. GPT-6 Luna     Model: gpt-6-luna. Efficient GPT-6 model for focused coding and high-volume tasks.
         13 │+  6. GPT-5.6-Terra  Model: gpt-5.6-terra. Balanced agentic coding model for everyday work.
         14 │+  7. GPT-5.6-Luna   Model: gpt-5.6-luna. Fast and affordable agentic coding model.
thread 'app::tests::provider_picker_refresh_ref::restored_openai_appears_in_already_open_model_picker' (66265026) panicked at /Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/insta-1.48.0/src/runtime.rs:719:13:
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 4779 filtered out; finished in 0.41s
error: test failed, to rerun pass `-p codex-tui --lib`

## own tests (provider_picker_refresh) at HEAD a0a8daca79 [astra]
test app::tests::provider_picker_refresh::provider_manager_late_refresh_updates_picker_without_reopening_dismissed_menu ... ok
test app::tests::provider_picker_refresh::provider_manager_recovery_refreshes_open_model_picker_and_preserves_tab ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 4777 filtered out; finished in 0.30s
## own tests at test-only commit 40efd3a488 (expect FAIL)
test app::tests::provider_picker_refresh::provider_manager_late_refresh_updates_picker_without_reopening_dismissed_menu ... ok
test app::tests::provider_picker_refresh::provider_manager_recovery_refreshes_open_model_picker_and_preserves_tab ... FAILED
thread 'app::tests::provider_picker_refresh::provider_manager_recovery_refreshes_open_model_picker_and_preserves_tab' (66272950) panicked at tui/src/app/tests/provider_picker_refresh.rs:84:5:
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 4777 filtered out; finished in 0.37s
error: test failed, to rerun pass `-p codex-tui --lib`
```
