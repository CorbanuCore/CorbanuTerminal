# RETURN — acct-activation-46

Runtime gpt-6-astra/high; allocation digest `4b01dabebc8997b4c417039f1e88cf44f41cb0763a30576b15ea68093d0d5e2d`; claim `1fc06505-a846-4053-8bfc-8300a4d12b77`.
Verified brief SHA-256 `7b765960919f8ff8d03c526005b4f1dc1398026786968efb304b4f950885d336`; base `08aad7877a41f9aa6823911f70cb22a667603631`; worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/acct-activation-20260916`.
Classification: bounded fix restoring CI coverage for the existing never-distribute guard, PF-60-S03 (`in_progress`), active plan `docs/plans/active/portfolio-agent-cost-accounting.md`. Product heading **Measurement targets**: “The following metrics must be instrumented, with targets set through the decision rights defined above.” Current assignment supplies the worktree; plan/sprint coordinate reconciliation remains with the manager.

CI now builds `corbanu` with `--profile dev-small --features codex-core/developer-accounting` and invokes `python3 -B scripts/test_build_codex_package.py --feature-artifact codex-rs/target/dev-small/corbanu`. The step **Verify developer accounting cannot be packaged** runs in `corbanu-terminal-ci.yml`'s `deploy-smoke` job after the default package smoke test and before artifact upload.
The four-test discriminator requires a real feature artifact, independently asserts its marker exists, and requires rejection before creation of the explicit package directory/archive. It also tests every source-output field, clean inputs and missing-marker failure. Removing the emitter fails the real-artifact assertion; breaking the scan fails rejection assertions. No feature artifact is executed.

Local feature build: `cargo build --profile dev-small --bin corbanu --features codex-core/developer-accounting` from `codex-rs`, inherited shared target, exit 0, 3.92s. Built artifact SHA-256: `81dfe1709e7796b4991b7bfdc868bbb2df8fd988397e9d464787d683068a26ba`.
Both initial and final guard runs passed 4/4; both initial and final missing-marker controls failed 1/4 (exit 1). Local CI-equivalent test used `--feature-artifact /Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/target/dev-small/corbanu`: 4/4 passed, exit 0. Actual rejection (printed once for each package mode):
```text
Refusing distribution package: developer-accounting enabled in /Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/target/dev-small/corbanu
```
Missing-marker control used the unmarked `scripts/codex_package/__init__.py` as the required artifact: exit 1; 4 run, 3 passed, 1 failed, exact case `PackageGuardTests.test_feature_artifact_rejected_before_directory_or_archive`; diagnostic `AssertionError: Feature artifact is missing the required marker`. This intentional negative control is not reported as a passing test invocation.
Disabled-scan control replaced the wrapper guard with the underlying unguarded builder in memory, without source edits: exit 1; 1 test run, 2 failed subtests (directory and archive) in `PackageGuardTests.test_feature_artifact_rejected_before_directory_or_archive`, diagnostic `AssertionError: Guard failed before resource resolution`. Network resource resolution is blocked in this test even when the guard regresses.
Existing packaging suite: `python3 -B -m unittest discover -s scripts/codex_package -p 'test_*.py'`, exit 0, 28/28 passed.

**Unresolved scope conflict:** the guard still lives in `scripts/build_codex_package.py::distribution_source_binaries`, installed by `main` monkeypatching `cli.build_source_binaries`. Direct library callers can still bypass it. There is no technical reason it cannot move: move the byte scan into `scripts/codex_package/cargo.py::build_source_binaries` immediately after `validate_source_outputs(outputs)`, then remove the wrapper patch and update the new tests to use that library entry point. But `scripts/codex_package/cargo.py` is absent from this allocation's exclusive writable scope. No out-of-scope edit or alternative monkeypatch was made. Finding 2 remains open; expand the allocation by that exact file to finish it.
Known limits requested by the brief: (1) an artifact whose marker was stripped after the build is not detected; (2) a binary packaged by hand or by a future script that never calls `build_source_binaries` is outside this mechanism. These are disclosed, not fixed.

Evidence repair: all 14 original acct-activation-37 `*.txt` logs have adjacent lossless `*.txt.gz` copies; every decompressed copy was compared with its original. The ignore rule explicitly permits compressed copies, and the old receipt links its original final Core/TUI/rejection logs. Earlier failing runs remain preserved with their original attribution.

Rust gates use this checkout's guarded `just test` after reading `docs/development/test-isolation.md`. Default Core and TUI reuse `/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/target`; feature Core reuses the existing, separate `qa/portfolio/agent-cost-accounting/pf-60-s03/acct-activation-33/feature/target`. No fresh per-run target was created; the feature cache is not the shared receiving cache, disclosed as a deviation from a strict reading of the gate.
Initial default Core command `just test -p codex-core accounting` remained in compilation and was interrupted through its exec session: exit 130, zero tests executed, no failing test names. The compiler sample completed (exit 0) and is preserved; no cause attribution is claimed.
Final commands all run from `codex-rs` with `CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 NEXTEST_TEST_THREADS=1`:
| Command | Result |
| --- | --- |
| `just test -p codex-core accounting` | Exit 100; 124 run, 103 passed (7 flaky), 21 failed, 3529 skipped, 1562.301s; 49 failed attempts across 28 cases |
| `just test -p codex-core accounting --features codex-core/developer-accounting` | Exit 0; 127/127 passed, 0 failed, 1 slow, 3529 skipped, 97.745s |
| `just test -p codex-tui usage` | Exit 0; 91/91 passed, 0 failed, 4071 skipped, 4.372s |
Default failures all report `Error: deadline has elapsed`. Exact case names below: F = final failure (21); R = first attempt failed, retry passed (7). No attribution or waiver is claimed.
```text
F suite::accounting_anthropic::accounting_anthropic_role_disables_actual_child_request_retry
F suite::accounting_anthropic::accounting_anthropic_role_endpoint_mismatch_has_no_unapproved_send
F suite::accounting_anthropic_recovery::accounting_anthropic_observation_failure_stops_real_sampling_without_repair_send
F suite::accounting_anthropic_recovery::accounting_anthropic_stream_retry_retains_start_usage_without_merging_reused_provider_id
F suite::accounting_chat::accounting_chat_native_finish_reason_and_tool_parity
F suite::accounting_chat::accounting_chat_native_sampling_auxiliary_scope
F suite::accounting_chat_recovery::accounting_chat_native_admission_barrier_and_failure
F suite::accounting_chat_recovery::accounting_chat_native_cancellation_and_two_reopens
F suite::accounting_chat_recovery::accounting_chat_native_delete_rejects_late_usage
F suite::accounting_chat_recovery::accounting_chat_native_invalid_evidence_no_repair
F suite::accounting_chat_recovery::accounting_chat_native_observation_failure_no_repair
F suite::accounting_chat_recovery::accounting_chat_native_outer_retry_prefix_and_ids
F suite::accounting_chat_recovery::accounting_chat_native_spawned_role_children_and_fork
F suite::accounting_responses_recovery::accounting_responses_native_admission_barrier
F suite::accounting_responses_recovery::accounting_responses_native_cancel_two_reopens
F suite::accounting_responses_recovery::accounting_responses_native_delete_rejects_late_usage
F suite::accounting_responses_recovery::accounting_responses_native_observation_failure
F suite::accounting_responses_recovery::accounting_responses_native_outer_retry_prefix
F suite::accounting_responses_recovery::accounting_responses_native_spawned_role_children
F suite::accounting_responses_ws::accounting_responses_ws_native_upgrade_required_http_fallback
F suite::accounting_responses_ws_recovery::accounting_responses_ws_native_auxiliary_scope_and_event_parity
R suite::accounting_anthropic::accounting_anthropic_cancel_held_http_other_owner_fresh_turn_and_two_resumes
R suite::accounting_anthropic::accounting_anthropic_native_spawned_children_role_reload_and_fork_own_only_new_sends
R suite::accounting_anthropic::accounting_anthropic_role_extends_actual_child_idle_timeout
R suite::accounting_anthropic_recovery::accounting_anthropic_cancel_keeps_intent_or_observed_start_through_reopen
R suite::accounting_responses_ws::accounting_responses_ws_native_off_and_http_only_compatibility
R suite::accounting_responses_ws::accounting_responses_ws_native_ws_prefix_then_http_fallback
R suite::accounting_responses_ws_recovery::accounting_responses_ws_native_two_reopens_and_original_prices
```
Evidence: [default Core final](core-default-replay.log.gz), [guard rejection](guard-test-final.log.gz), [missing-marker control](absent-marker-final.log.gz), [disabled-scan control](disabled-scan.log.gz), [packaging suite](packaging-tests.log.gz), [feature build](feature-build.log.gz), [feature Core](core-feature.log.gz), [TUI](tui-default.log.gz), [interrupted default build](core-default.log.gz), [compiler sample](default-compiler-sample.log.gz). Decompress with `gzip -dc`; all original attempts remain local and each completed log has a lossless compressed copy.
Changed-line accounting against the frozen base: **102 inside tests, 78 outside tests, 180 textual lines total** (additions plus deletions); within the 90-line non-test target and 150 hard limit. Separately, 27 binary gzip archives total 108,191 bytes and preserve 10,356 decompressed log lines; these are disclosed evidence artifacts, not counted as authored source lines. The final scope audit and `git diff --check` pass. Changes remain uncommitted; no push.
No Rust formatter was needed because no Rust source changed. No raw Cargo tests, live-profile use, credential reads, push or release. This is an internal source/evidence return; independent functional/TUI/live-repository/human-acceptance gates remain with PF-60-S03 and no human-test readiness is claimed. Code-blind functional design is proposed N/A for this internal packaging-test correction, subject to integrator acceptance.
Brief corrections: requested library relocation is incompatible with the supplied writable scope. Source inspection also shows that CLI omission of `--package-dir` creates an empty temporary root before source validation; the prior before-directory claim is therefore too broad literally. The tests prove rejection before assembly and before creation of explicit outputs. CI wiring is locally checked; no remote CI run is claimed.
