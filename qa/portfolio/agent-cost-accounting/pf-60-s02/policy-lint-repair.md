# PF-60-S02 policy-lint repair — frozen worker return

## Mandate and candidate

Bounded behavior-preserving correction within active PF-60/S02 (in_progress), not a new product boundary. Product specification heading **Product measurement**: “No commercial performance numbers have been supplied.” Read the full policy-lint-repair allocation, current plan/sprint, root1.7/Rust policies and isolated-execution contract. The corbanu-terminal-development skill governed scope, format-before-test order and evidence separation; manager owns shared ledgers and independent review.

Worker W = /Volumes/CorbanuDrive/Corbanu/worktrees/accounting-policy-repair-20260912; branch workstream/accounting-policy-repair-20260912. Clean launch HEAD179172541dda91e4f9e185269a0fad95f3866bab resolves the supplied short ref. Allocation basea89a48548f644a64cbb8cd9da090b5b75578c922 is an ancestor; base-to-launch Rust diff empty. Launch Rust tree205e89cd6a2c4739f76950da15367f30e636d26d. Old worker/canonical source untouched.

Private evidence R = /Volumes/CorbanuDrive/Corbanu/.codex-work/accounting-policy-repair.pDfwmV. Only the allocated six Rust files plus this receipt changed. No source edits after formatting; candidate uncommitted. No Cargo/lock/dependency edits, installation, cache repair, child agents, reviews, live calls or credentials. Ordinary test subprocesses remain part of the existing process-interruption proof.

## Literal repair and preserved proof

- Embedded accounting CREATE-name extraction returns an anyhow contextual error instead of panicking. Ledger/schema validation, SQL and rejection rules otherwise unchanged.
- SQLite test shim accepts the original pool/connect options verbatim, under cfg(test), crate-only visibility. Production shim and public API unchanged; no new allow/expect attributes. Existing single connections, zero busy timeout, foreign keys, original paths and before_acquire channel/Notify barriers retained. Separate memory/goals fixture options unchanged.
- SQLx error-code closures become method references; remove one needless borrowed Path. No assertion, case, error marker, timeout, synchronization, rollback/reopen or process proof removed.
- Telemetry poisoned-lock recovery becomes the identical method reference. Released-state and exact sent-ID correlation conditions unchanged; successful rotation still produces UUIDv4 and only replaces already-present Corbanu header names. Invalid replacement-header construction returns non-retryable TransportError::Build through existing request telemetry, never empty ID/success or reuse of a released identity. codex-client's existing RetryOn does not retry Build; no retry policy changes.
- No added tests of static embedded strings or guaranteed UUID formatting. Existing telemetry matrix exercises both header names, five statuses, released/settled/reserved/absent state and matching/mismatched correlation. Existing schema rejection, full DB rollback, native deletion and real contention tests are the changed-tree regression evidence. Impossible malformed embedded/UUID inputs were not newly fault-injected.

## Commands and actual results

All Rust commands run from W/codex-rs with exact prefix:
`env RUSTUP_TOOLCHAIN=1.95.0 RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true UV_OFFLINE=true CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911/codex-rs/target`.
Local process inventory showed no competing cargo/rustc/clippy/nextest build before starting; observed remote security build used a different remote target. This exclusive target lease was used serially, never the old accounting target.

| Command after prefix | Session / exit / actual result |
| --- | --- |
| sandbox-exec -p PROFILE just fix -p codex-state -p codex-api --locked | exit101: TCP lock-listener bind denied, os error1; no successful fix claim. |
| rustfmt --edition 2024 --config skip_children=true SIX_PATHS | exit0 before final tests; six stable-toolchain imports_granularity=Item warnings. |
| just clippy -p codex-core -p codex-api -p codex-login -p codex-http-client -p codex-state -p codex-tasknode-session --locked --no-deps | session14935 exit101: eight newly exposed errors in unallocated state/tests/accounting_store.rs. No extra -D warnings. |
| just test -p codex-state -p codex-api -p codex-tasknode-session --test-threads 1 --locked | session15863 exit0;575/575 passed,0 skipped,30.044s; build48.63s; nextest dffe6d20-4286-4ba6-a676-9454b52305d6. |
| just test -p codex-state -p codex-api -E 'test(accounting) \| test(telemetry::tests::corbanu_retry)' --test-threads 1 --locked | session67442 exit0;105/105 passed,390 filtered/skipped,14.184s; build38.02s; nextest7511835b-027c-474a-b014-bd92fa0021c8. |
| cargo check --offline --locked -p codex-core -p codex-api -p codex-login -p codex-http-client -p codex-state -p codex-tasknode-session --lib | session59514 exit0;47.23s. |
| python3 docs/plans/check.py; python3 docs/sprints/check.py (separate root invocations) | each exit0:3 active,115 current,126 archived. |
| git diff --check | exit0. |

PROFILE is (version 1)(allow default)(deny network*)(deny file-write* (subpath "W")) followed by allow file-write* with exactly six literal W/source paths from the hash table below; no unallocated source/lock writes. SIX_PATHS are those same six paths relative to W/codex-rs. R/command-results.json preserves fully expanded literal commands. Broad just fmt would format unrelated files; used the recorded literal-path rustfmt equivalent. No direct cargo test or environment repair.

Full suite counts: API208/state287/TaskNode80. Both test runs have no LEAK/FLAKY marker or final-summary leak; this does not erase historical leaks. State build reports10 lib warnings and21 lib-test warnings (9duplicates), all retained. Scoped lint reports state10 lib/33 lib-test warnings(9duplicates), TaskNode lib-test2, external state test1; traversal stopped on errors, not six-crate/workspace lint-clean. Normal check reports state10, TaskNode17, protected-state4 warnings.

Focused executed proof includes corbanu_retry_rotates_only_confirmed_released_attempts; public_deletion_samples_time_after_cleanup_and_writer_acquisition; absent_store_delete_locks_before_inspection_and_ordinary_writer; separate_store_failures_preserve_retry_graph_without_global_rollback_claim; fourteen_reachable_sql_faults_rollback_reopen_retry_both_routes. Full names and results remain in logs/JUnit. Earlier A/B/C1/C2/native/store/caller receipts and tests remain intact.

## Newly exposed scope blocker — no expansion performed

Existing-policy lint now rejects state/tests/accounting_store.rs at line37 (direct SqliteConnection::connect_with) and lines50,57,62,74,76,78,89 (unwrap_used). All eight are outside this seven-path allocation. No remaining error is reported in the edited paths, but early traversal termination prevents claiming all six crates passed. Exact diagnostic text: R/policy-clippy.log lines176–266. Manager must classify an explicit additional external-test path/normal-library shim approach before editing; cfg(test) shim is deliberately unavailable to that external crate. Do not weaken its normal-library proof or authorize a production API by implication. No additional repair or blind lint rerun performed.

## Preserved history and applicability

Prior accepted receiving71+840 tests/normal check are parent-attributed dependencies, not rerun or relabeled here. Original strict Clippy101 protocol large_enum_variant and policy Clippy101 (session55970) remain failures. Original policy log /Volumes/CorbanuDrive/Corbanu/.codex-work/accounting-policy-lint.K8OrUC/policy-clippy.log SHA256828fe3ffa3c6cc17f2c93920fe9632dec3197da77b1f08cc792b4da87dc846a9 stays untouched. Manager explicitly allocated the three preexisting telemetry findings attributed401ddcc206/c6b48e6b36; no claim Anthropic introduced them.

Anthropic01–03, original compile/fix/cache failures, full-Core132/133/147 failed runs and their baseline comparisons,133 baseline-common plus14 serial-passing additional failures, failed parallel shared gates and historical LEAK provenance remain in the accepted anthropic-dispatch-increment.md and private receiving/baseline receipts. Neither passing serial tests nor this lint correction waives those failures. No full-Core/workspace repeat, Linux/RTX inference or new review consumed; parent owns one new Astra code review plus necessary correction and receiving integration.

Integrator's allocation records increment-specific isolated-functional N/A: internal behavior-preserving/default-OFF correction with no user-facing handoff. Independent fresh designer/executor and separate evidence review with actual schema2 isolation/probe/execution receipts remain required before affected collection/replay user handoff and S03 totals/range/interval acceptance. Manager provisions the harness; no duplicate implementation/human blocker. S02 is not complete, S03 remains draft,90..365 compact-only import proposal retained but not dispatched. No TUI/live-repository/human/benchmark/release readiness claim.

## Hashes and size

Source additions/deletions count124 total. Conservative non-test32 includes all nine cfg(test) shim lines; remaining92 test changes. This69-line receipt is counted wholly non-test: final193total/101non-test across seven paths, within target300/150 and STOP450/225. Counts and receipt digest are in R/frozen-candidate-manifest.json; no inherited size/review exceptions.

| Source path | Added/deleted | SHA256 |
| --- | --- | --- |
| codex-rs/state/src/runtime/accounting_store.rs | 2/1 | e61863e550dc20e67884abf0afd4d39513b9b9db410a056b55d7088a83706214 |
| codex-rs/state/src/runtime/accounting_store_tests.rs | 33/27 | ccfa243df65edc9cd970a6a63cbebe0cf4e17d0d8b1a251a08d1b397e47cfc7a |
| codex-rs/state/src/runtime/accounting_retention_atomic_test_support.rs | 5/4 | a39c294c168d40ccec120fb5e075174a0ae292c5d86d5a0a775e9efdb82601b7 |
| codex-rs/state/src/runtime/accounting_native_tests.rs | 10/13 | 8b7ad01ee1f957682fe6075231f05a63f78d44f1951daa6faf11c8638ce7b60f |
| codex-rs/state/src/sqlite.rs | 9/0 | 7b55e84012c399127b0805d2b9f4765091985c72c3d0b784aec276048b8b8249 |
| codex-rs/codex-api/src/telemetry.rs | 14/6 | 01f6528922cbcc05901e042ee7edb0047c944531d70f6ebe36223b7687e3052c |

R/frozen-candidate.diff includes literal tracked diff plus this untracked receipt, not a committed/staged substitute. Full artifact hashes/commands are in R/command-results.json and final manifest. Manifests/locks remained byte-identical: Cargo.toml cd6e30f5bb8d63b2d05ae4a7410f0da886d0d38a4f149d5a384b3d7e4350e5fe; Cargo.lock03bd0f12d8633f3134511a6abaf2bdd7918389887aa393fc119fbfcab473b0a8; MODULE.bazel.lock c8d7e3f8c8bec8f8e71cc3d1d39fcb952eec0f07a41cdac48401a6f64a60d979.

Nextest writes JUnit under W/codex-rs/target despite the explicit shared build target. Initially copied canonical's stale840 JUnit, detected UUID/count mismatch immediately and retained it as canonical-stale-840.junit.xml (not current evidence). Preserved actual575 JUnit before focused overwrite; focused105 JUnit separately retained. Raw current logs are authoritative with matching UUIDs. Read-only exploration also encountered a nonexistent http-client/src/retry.rs and absent root .config; located actual codex-client/src/retry.rs and codex-rs/.config without repair.

## Eight-path continuation — current frozen return

Manager amendment17aaf9b947871f59c96120efb765e2a425a27355 was received docs-only at current HEAD8cccd477a0a4913bbf5b58d3f95d5e86ec2594db; allocation base remains a89a48548f644a64cbb8cd9da090b5b75578c922. Read full amended allocation/plan/sprint and root/Rust policies. The complete first69-line receipt above and R's frozen193/101 candidate/raw101/575/105 artifacts remain historical, unchanged; this section supersedes their then-current scope/blocker.

Added only codex-rs/state/tests/accounting_store.rs: existing PUBLIC open_read_write_pool, acquired owned connection detached before explicit pool.close(), existing conn.close() before every reopen. Assert live PRAGMA foreign_keys=1. Normal-library linkage/write-capable schema faults and all original IDs/times/amounts/assertions remain; fixture helpers now propagate Result/context/?. Production API/options and prior six source hashes unchanged. No allowances, cfg(test) imports, assertion removal or new test substitution.

New evidence R2 = /Volumes/CorbanuDrive/Corbanu/.codex-work/accounting-policy-external.qOmp6p. Same pinned/offline/autoinstallOFF prefix and exclusive canonical build target as above, no competing local build found; no cache/lock/source repair. Exact expanded commands, logs and matching per-run JUnit hashes are in R2/command-results.json.

| Gate, after the same prefix | Actual result |
| --- | --- |
| Scoped sandbox just fix -p codex-state -p codex-api --locked, then literal seven-source rustfmt | fix101, same lock-listener denial; fmt0, seven stable imports_granularity warnings; before both final tests. |
| just test -p codex-state --test accounting_store --test-threads 1 --locked | session26273 exit0;4/4 passed,0 skipped,0.893s; run07b9ed55-2a38-49c3-a974-026d1774c6b0; build27.88s. |
| just test -p codex-state -p codex-api -p codex-tasknode-session --test-threads 1 --locked | session14321 exit0;575/575 passed,0 skipped,31.090s; run02a92418-fa0e-424e-84fd-8301ddffb7f8; build3.65s. |
| Same six-crate just clippy --locked --no-deps | session32035 exit101; eight newly exposed Core errors, none in the eight-path allocation. Not six-crate/workspace lint-clean. |
| Same six-crate cargo check --offline --locked --lib | session59574 exit0,1.15s. |
| Both governance checkers and diff checks | exit0;3 active/115 current/126 archived; tracked and new receipt whitespace clean. |

New scope blocker: core/src/client.rs:2507, core/src/accounting.rs:39,46 and core/src/exec_env.rs:32 use expect; core/src/accounting.rs:85,116,117,162 hold disallowed tokio MutexGuard across await. Exact diagnostics R2/policy-clippy.log:378–453; no edits or automatic scope expansion. Manager classification required for those three paths and synchronization semantics before repair; no inference that a test pass waives them. First independent code review remains unspent/parent-owned.

Both new test runs have no LEAK/FLAKY marker. Full test warnings: state10 lib/21 lib-test(9duplicates); normal check state10/TaskNode17/protected-state4. Clippy retains state10 lib/33 lib-test(9duplicates), external test1, TaskNode lib-test2 and protected-state4 warnings. Original warnings/failures/history and increment-specific isolated-functional N/A/later collection-replay/S03 gate above remain intact.

Final eight-path diff and per-file hashes/counts are R2/frozen-candidate.diff and R2/frozen-candidate-manifest.json. Source206total/32conservative non-test; external test45added/37deleted,390lines,SHA256b75e29186a944dd3796fbd889a913d3d76d309e8c78e081f737830e740cd292b. Receipt92lines gives298total/124non-test, within300/150 target and450/225STOP. Frozen uncommitted; no source edits after fmt, no commits/reviews/live/shared-ledger changes.
