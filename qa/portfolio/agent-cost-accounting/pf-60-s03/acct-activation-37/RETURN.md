# RETURN — acct-activation-37

Runtime: gpt-6-astra/high; allocation digest `6b798c29997c281e2172bc3042f11a0d79eed7f8e6d71b139a1a987e0b534973`; claim `b56f20e4-205e-42f0-bc59-6931290dcf58`. Brief SHA-256 verified: `38c49bf7dd4daee781eda78442322ca330b2b783c45ab0efcc8b7408bddaeb07`.
Base: `d05bf9e715819fe8150b6d6b7daac07a305143d5`; assigned worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/acct-activation-20260916`. Internal corrective increment of active PF-60-S03, plan `portfolio-agent-cost-accounting.md`; product heading **Product measurement / Measurement targets**: “The following metrics must be instrumented, with targets set through the decision rights defined above.” Canonical plan/sprint coordinates still need manager reconciliation with this explicit allocation.
The additional guard keys on invoking the canonical package builder with executable bytes containing `CORBANU_DEVELOPER_ACCOUNTING_NOT_FOR_DISTRIBUTION`. The developer selector retains those bytes through `black_box`; symbol stripping does not remove them. The builder checks every source/prebuilt Rust executable before assembly, including extras, and refuses both archives and directory-only handoffs. Thus packaging intent, rather than debug assertions, defines this boundary; the existing release compile error remains. Manual copying/alternative archivers are outside this entrypoint.
The real feature build exposed an omitted prerequisite: `error[E0063]: missing field accounting in initializer of ConfigOverrides` at `exec/src/lib.rs:434` (exit 101). Removed the feature-only override field within Core; embeddings set the existing `Config.accounting`, and refresh copies it after rebuilding. The test helper explicitly opts out. No exec file, dependency, user configuration or default activation changed.
Reproduction below uses the shared target `/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/target` and the action evidence directory as `E`; commands run at repository root unless the subshell changes directory.
```sh
export CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/target
E=qa/portfolio/agent-cost-accounting/pf-60-s03/acct-activation-37
(cd codex-rs && cargo build --profile dev-small --features codex-core/developer-accounting --bin corbanu --bin corbanu-acp --bin corbanu-walletd --bin codex-code-mode-host)
python3 -B scripts/build_codex_package.py --target aarch64-apple-darwin --variant corbanu --entrypoint-bin "$CARGO_TARGET_DIR/dev-small/corbanu" --extra-bin "corbanu-debug=$CARGO_TARGET_DIR/dev-small/corbanu" --extra-bin "corbanu-acp=$CARGO_TARGET_DIR/dev-small/corbanu-acp" --extra-bin "corbanu-walletd=$CARGO_TARGET_DIR/dev-small/corbanu-walletd" --code-mode-host-bin "$CARGO_TARGET_DIR/dev-small/codex-code-mode-host" --cargo-profile dev-small --package-dir "$E/rejected-package" --archive-output "$E/corbanu-terminal-package-aarch64-apple-darwin.tar.gz"
```
Final-tree build: exit 0, `Finished dev-small profile [unoptimized] target(s) in 52.33s`. Package invocation: **exit 1**, actual output:
```text
Refusing distribution package: developer-accounting enabled in /Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/target/dev-small/corbanu
```
After command completion, neither the package directory nor archive existed. Corbanu SHA-256: `81dfe1709e7796b4991b7bfdc868bbb2df8fd988397e9d464787d683068a26ba`. This is the native macOS equivalent of CI's prebuilt dev-small packaging lane; Linux itself was not executed. No packaged binary was executed.

Final Rust gates use separate sequential commands from `codex-rs`, all with `CARGO_INCREMENTAL=0 NEXTEST_TEST_THREADS=1` and the same shared target above (feature/TUI also use `CARGO_BUILD_JOBS=1`), through guarded `just test` after reading the isolation policy. No native credential prompt or live-profile access was observed. The Core failures below block an unqualified handoff:
| Command | Run / passed / failed | Failure names |
| --- | --- | --- |
| `just test -p codex-core accounting` | 124 / 109 / 15; exit 100; 1199.846s; 1 slow; 3529 skipped | D names below |
| `just test -p codex-core accounting --features codex-core/developer-accounting` | 127 / 105 / 22; exit 100; 1398.605s; 1 slow, 2 flaky; 3529 skipped | D+F names below |
| `just test -p codex-tui usage` | 91 / 91 / 0; exit 0; 4.363s; 4071 skipped | None |
Python: all 3 new packaging-boundary tests pass (including every source output and both archive/directory CLI rejection); all 28 existing packaging tests pass. Existing Rust tests retained; the refresh test now also switches provider and asserts the mode stays pinned.
Exact failure names: D = failed in both Core lanes (15); F = additional feature-lane failures (7). All report `Error: deadline has elapsed`; no attribution or waiver.

```text
D suite::accounting_chat::accounting_chat_native_finish_reason_and_tool_parity
D suite::accounting_chat::accounting_chat_native_sampling_auxiliary_scope
D suite::accounting_chat_recovery::accounting_chat_native_admission_barrier_and_failure
D suite::accounting_chat_recovery::accounting_chat_native_cancellation_and_two_reopens
D suite::accounting_chat_recovery::accounting_chat_native_delete_rejects_late_usage
D suite::accounting_chat_recovery::accounting_chat_native_invalid_evidence_no_repair
D suite::accounting_chat_recovery::accounting_chat_native_observation_failure_no_repair
D suite::accounting_chat_recovery::accounting_chat_native_outer_retry_prefix_and_ids
D suite::accounting_chat_recovery::accounting_chat_native_spawned_role_children_and_fork
D suite::accounting_responses_recovery::accounting_responses_native_admission_barrier
D suite::accounting_responses_recovery::accounting_responses_native_cancel_two_reopens
D suite::accounting_responses_recovery::accounting_responses_native_delete_rejects_late_usage
D suite::accounting_responses_recovery::accounting_responses_native_observation_failure
D suite::accounting_responses_recovery::accounting_responses_native_outer_retry_prefix
D suite::accounting_responses_recovery::accounting_responses_native_spawned_role_children
F suite::accounting_anthropic::accounting_anthropic_cancel_held_http_other_owner_fresh_turn_and_two_resumes
F suite::accounting_anthropic::accounting_anthropic_native_spawned_children_role_reload_and_fork_own_only_new_sends
F suite::accounting_anthropic::accounting_anthropic_role_disables_actual_child_request_retry
F suite::accounting_anthropic::accounting_anthropic_role_disables_actual_child_stream_retry
F suite::accounting_anthropic::accounting_anthropic_role_endpoint_mismatch_has_no_unapproved_send
F suite::accounting_anthropic::accounting_anthropic_role_extends_actual_child_idle_timeout
F suite::accounting_anthropic_recovery::accounting_anthropic_cancel_keeps_intent_or_observed_start_through_reopen
```
Feature flaky passes (failed first attempt, passed retry): `suite::accounting_anthropic_recovery::accounting_anthropic_deleted_native_owner_cannot_be_resurrected_by_delayed_usage`, `suite::accounting_anthropic_recovery::accounting_anthropic_observation_failure_stops_real_sampling_without_repair_send`.

Pin decision: retain it for qualification. Provider/wire changes are unrecorded on the new route; the inspector retains historical rows under their original attribution, without a new route-change banner. A changed endpoint on the bound route fails before send with the existing accounting error. Start a new session to collect a newly selected route; explicit OFF stays OFF. This does not redirect inference to the old endpoint or misattribute a new-route request.
Receipt repair (acct-activation-46): all fourteen original `*.txt` attempts are now preserved byte-for-byte as adjacent `*.txt.gz` files, including [default final](core-default-serial.txt.gz), [feature final](core-feature-replay.txt.gz), [TUI final](tui-default-final.txt.gz) and [package rejection](package-rejection-final.txt.gz). Decompress with `gzip -dc`; the original attempts and historical results above are unchanged.
Preserved attempts: initial dev-small build failed at the exhaustive initializer; initial default compile was interrupted after the prerequisite source fix (exit 130, zero tests). Before restricting refresh preservation to the developer feature, interrupted default attempts reported `74/124 run: 49 passed (1 slow, 2 leaky), 2 failed, 23 timed out` and `56/124 run: 51 passed, 5 failed` (exit 130 each). These are incomplete runs, not passes. The first feature compile was also interrupted (exit 130, zero tests) after sleeping about ten minutes with under three CPU seconds; retry uses `CARGO_BUILD_JOBS=1`. A diagnostic sample was stopped during symbol processing (exit 143). No root cause or baseline attribution is asserted.
Changed lines: 64 non-test source; 86 test lines (24 Rust, including the config test helper, plus 62 Python); 78 receipt/ignore lines; **142 outside tests, 228 total**, below the hard 150 non-test limit. Plan/sprint checkers and `git diff --check` pass. No changes outside the assigned paths; no push or release. This is an internal source/evidence return, not functional or human-test acceptance; PF-60-S03's TUI, independent execution/review, live-repository and acceptance gates remain with the manager.
Brief corrections: the dev-small archive hole was real; prior feature-enabled Core tests did not establish that the terminal itself built. Its ConfigOverrides compile failure was an additional blocker, corrected within scope. The prior default Core pass was not reproduced in the mandated shared target: this final run failed 15 cases. No baseline attribution or waiver is claimed.
