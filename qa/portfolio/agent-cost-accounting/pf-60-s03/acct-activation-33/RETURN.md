# RETURN — acct-activation-33

- Runtime: gpt-6-astra, high. Allocation digest: `3c4c28ce19cb90674fc142ad899c5bc90433659f4c719aa61d622cc0b15ecd1d`; claim: `601ab86c-3a99-41cb-965f-941497df1d20`.
- Frozen brief: `/private/tmp/fmgr.Q1SIYZ/briefs/acct-activation-29.json`; verified SHA-256 `bf126eb81a081c4719814121b7abf305d0b8d4c19fc1ece6f0d4f04335eaa434` before implementation. Nested allocation labels retained as supplied.
- Base/HEAD: `bbd4c5eeac38b7e7e9b06e39482669c68e6228dd`; branch `bootstrap/acct-activation-20260916`; worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/acct-activation-20260916`. Changes remain uncommitted; no push.
- Classification: internal corrective increment of the active accounting product initiative, PF-60-S03 (`in_progress`), plan `docs/plans/active/portfolio-agent-cost-accounting.md`. Product citation: **Product measurement**, **Measurement targets** — “The following metrics must be instrumented, with targets set through the decision rights defined above.” The current user allocation explicitly selects this activation worktree; canonical plan/sprint coordinates still name the earlier inspector worktree and need manager reconciliation. Both governance checkers pass, but do not validate this actual-coordinate discrepancy.

## Corrections

`ConfigOverrides.accounting: Option<AccountingMode>` exists only with `developer-accounting`. `None` selects the developer route; `Some(Disabled)` opts out; an explicit enabled mode retains its endpoint and scope. Core `test_config()` uses the opt-out. `rebuild_preserving_session_layers` passes the existing mode into the loader so integration fixtures' explicit modes survive refresh. The default production initialization remains `accounting: AccountingMode::Disabled`; new production override logic compiles out without the feature. No configuration-file, CLI, environment, persistence or default-feature surface was added.

The activation test constructs a typed Anthropic config with an empty `ConfigLayerStack`, temporary cwd and temporary home. It does not traverse actual cwd, project, user, system or managed layers. It remains in both lanes to verify the default-OFF branch too. Runtime: **0.061s default**, **0.046s feature**. The new `accounting_developer_loader_override_survives_refresh` test passes (**0.062s**) for both OFF and an explicit enabled endpoint/scope. The former manual OFF assignment in the role test is now an assertion that the harness starts OFF. Every existing test is retained.

Removed `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_shows_auto_review_permissions.snap.new`. The corresponding tracked `.snap` is untouched.

## Test runs

All Rust tests used this checkout's guarded `just test` after reading `docs/development/test-isolation.md`. No live profile or native credential prompt was used/observed. Formatting was limited to the three changed Rust files using `rustfmt --edition 2024 --config skip_children=true`; no workspace formatter/fix was run. The final import-only correction was confined to core's test module and did not change the TUI or release-guard production inputs.

Exact target-directory base: `/Volumes/CorbanuDrive/Corbanu/worktrees/acct-activation-20260916/qa/portfolio/agent-cost-accounting/pf-60-s03/acct-activation-33`.

| Lane (commands run from `codex-rs`) | CARGO_TARGET_DIR under the exact base above | Final result | Log |
| --- | --- | --- | --- |
| `just test -p codex-core accounting` | `default/target` | 124 run, 124 passed, 0 failed; 1 slow; 3529 filter-excluded; exit 0; 67.659s | Summary inlined below |
| `just test -p codex-core accounting --features codex-core/developer-accounting` | `feature/target` | 127 run, 127 passed, 0 failed; 1 slow, 2 leaky; 3529 filter-excluded; exit 0; 53.299s | Summary inlined below |
| `just test -p codex-tui usage` | `tui/target` | 91 run, 91 passed, 0 failed; 4071 filter-excluded; exit 0; 0.453s | Summary inlined below |

Each directory was created fresh for this action and never shared between lanes/features. The two core directories were reused only for their own corrected same-feature build after the initial test-code compilation failure: wrong namespaces for `LOCAL_FS` and `ConfigToml`, exit 101, zero tests executed. Raw attempts remain local and gitignored; both failed with ``error[E0425]: cannot find value `LOCAL_FS` in crate `codex_file_system` `` and ``error[E0603]: struct import `ConfigToml` is private``. No test retries or timeouts occurred in the final runs.

The feature lane's leaky tests are `accounting::chat::tests::accounting_chat_frame_guard_denies_provider_change_before_client_publication` and `accounting::responses::tests::accounting_responses_auth_route_eligibility`. Nextest counted them passed; this receipt does not diagnose or waive their leaks. `accounting_responses_ws_native_auxiliary_scope_and_event_parity` was slow but passed in both core lanes (48.950s default, 34.223s feature).

Historical nextest output, transcribed from the local logs by acct-activation-37 (raw logs are not committed):

```text
Summary [  67.659s] 124 tests run: 124 passed (1 slow), 3529 skipped
Summary [  53.299s] 127 tests run: 127 passed (1 slow, 2 leaky), 3529 skipped
Summary [   0.453s] 91 tests run: 91 passed, 4071 skipped
```

## Individually verified feature-enabled cases

Every case below is present as PASS in `feature.log`:

| Test | Seconds |
| --- | ---: |
| `accounting_anthropic_default_off_does_not_install_and_installed_off_deletion_cleans` | 2.978 |
| `accounting_chat_native_off_and_mode_isolation` | 13.927 |
| `accounting_responses_native_off_and_unsupported` | 8.348 |
| `accounting_responses_ws_native_off_and_http_only_compatibility` | 4.008 |
| `accounting_admission_failure_and_route_mismatch_are_sticky_without_send` | 4.187 |
| `accounting_anthropic_role_endpoint_mismatch_has_no_unapproved_send` | 3.321 |
| `accounting_chat_native_mismatched_endpoint_never_sends` | 10.721 |
| `accounting_responses_native_endpoint_mismatch` | 8.429 |
| `accounting_responses_ws_native_endpoint_and_cached_auth_mismatch` | 4.923 |
| `accounting_role_reload_preserves_internal_binding_without_changing_off_route_behavior` | 0.105 |
| `accounting_developer_activation_samples_http_and_installs_inspectable_day` | 1.619 |

## Distribution guard

`#[cfg(all(feature = "developer-accounting", not(debug_assertions)))] compile_error!(...)` rejects developer collection when debug assertions are disabled. The repository's release workflows invoke `cargo build --release`, and its release profile does not enable debug assertions. This covers both explicit feature selection and `--all-features` in that distribution configuration.

Actual negative build: `CARGO_TARGET_DIR="$PWD/../qa/portfolio/agent-cost-accounting/pf-60-s03/acct-activation-33/release-guard/target" cargo check --release -p codex-core --features codex-core/developer-accounting`, from `codex-rs`. **Exit 101**, solely at `core/src/accounting.rs:6`: `developer-accounting is debug-only and must not be enabled in distribution builds`. The diagnostic above is the relevant evidence; the full log remains local and gitignored. No release binary was executed.

Limit: there is no universal “will be distributed” Rust cfg. A manually distributed debug binary, or a release profile deliberately overridden to enable debug assertions, is not detected by this guard. The enforced property is rejection under the repository's current release configuration, not detection of every possible packaging action. Stronger arbitrary-packaging enforcement would need a mandatory distribution marker/build check, outside this source-only allocation.

## Size, final identity and limits

Production source: **21 changed lines** (17 added, 4 removed). Test source: **57 changed lines** (50 added, 7 removed), including the `#[cfg(test)]` helper in `config/mod.rs`. Deleted pending test artifact: **22 lines**. Total source/test/artifact diff: **100 lines**; tests plus artifact: **79**. QA receipt/raw logs and their local build-output ignore file are separate evidence, not production code. No dependency, manifest, state, or tracked snapshot change.

Final SHA-256:

- `codex-rs/core/src/config/mod.rs`: `897229ffded097c21a4c26b25f87b7bacb1f829e87fb34a94929aed29786764a`
- `codex-rs/core/src/accounting.rs`: `7e6088ed032c0a752f4f2fd1dc224c76777763ec4bb0b2b288511881059b6674`
- `codex-rs/core/src/accounting_tests.rs`: `602cf4f475b974fceb51dafee91e8dcb5444f18e8d24e32893e5bc3f3bfed9f3`

`git diff --check`, plan checker and sprint checker pass. All modified/untracked paths are inside the frozen writable scope. This is an internal correction/evidence return, not a functional/human-test or release approval. True-TUI, independent code-blind execution/evidence review, live-repository qualification, acceptance, documentation and benchmark gates for PF-60-S03 remain with the manager; no new completion claim is made for them.

Brief corrections: activation is in `load_config_with_layer_stack`; `load_from_base_config_with_overrides` is a test-only adapter. Integration builders already explicitly assign their requested OFF/enabled modes; refresh discarded those choices. Thus the failure mechanism was more specific than all suite fixtures starting enabled. Built-in Anthropic's provider constructor fixes its base URL, so arbitrary TOML provider overrides are not the demonstrated endpoint failure; the old test still had unnecessary ambient layer traversal. The claimed stale-target cause of the historical default failures was not independently reproduced here; fresh, separated lanes were used as instructed.
