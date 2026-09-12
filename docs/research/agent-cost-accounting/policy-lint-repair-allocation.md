# PF-60-S02 — receiving policy-lint repair

Manager allocation, September12. Bounded corrective unit within active PF-60,
specification **Product measurement**, “No commercial performance numbers have
been supplied.” Restore existing lint policy without changing accounting,
retention, request identity or database fault semantics; collection remains OFF.

## Baseline and ownership

Reviewed Anthropic24-file candidatebc9b8b0d8 is integrated. Combined receiving
71focused and840shared tests pass, normal six-library check passes. Full-Core
historical failures remain unwaived. Existing-policy scoped Clippy exited101:
accounting expect/redundant closures/direct SQLite pools/needless borrow and API
telemetry two redundant closures/UUID-header expect. The telemetry lines are
preexisting401ddcc206/c6b48e6b36, not caused by Anthropic, but are now explicitly
allocated for small behavior-preserving repair. Strict -D warnings protocol
large_enum_variant remains separate unchanged history, not silently suppressed.

Mendel, existing native Astra High worker, moves sequentially to
`/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-policy-repair-20260912`, branch
`workstream/accounting-policy-repair-20260912`, base
`a89a48548f644a64cbb8cd9da090b5b75578c922`. Original accounting checkout is frozen
history. Manager grants exclusive edits to these seven paths; no other active
worker owns state/sqlite or API telemetry. Cargo/lock and security are read-only.

- codex-rs/state/src/runtime/accounting_store.rs
- codex-rs/state/src/runtime/accounting_store_tests.rs
- codex-rs/state/src/runtime/accounting_retention_atomic_test_support.rs
- codex-rs/state/src/runtime/accounting_native_tests.rs
- codex-rs/state/src/sqlite.rs
- codex-rs/codex-api/src/telemetry.rs
- qa/portfolio/agent-cost-accounting/pf-60-s02/policy-lint-repair.md

Target300total/150non-test, STOP450/225 including receipt and deletions. One new
independent Astra High code review plus one necessary correction authorized;
prior Anthropic01–03 and all failures remain. Manager owns review dispatch and
receiving integration; worker returns a frozen uncommitted candidate first.

## Exact repair and proof

Replace embedded CREATE-name panic with a typed/contextual error, preserving
schema rejection. Use direct method references and remove the needless borrow.
Route fixture pools through the state SQLite shim. A narrow cfg(test) helper
inside sqlite.rs may accept exact SqlitePoolOptions/SqliteConnectOptions to
preserve before_acquire barriers, one connection, zero busy timeout, paths and
existing separate-store fault behavior. Do not replace these with generic
five-connection/default-timeout pools, remove concurrency assertions, add public
production API or blanket lint allowances. Ordinary production shim unchanged.

Telemetry must retain exact released-request correlation and request-ID rotation,
including poisoned-lock handling. Eliminate the expect without substituting an
empty ID, swallowing a failure as success, or changing retries. Existing tests
remain intact; extend same-file tests where needed. No pricing/catalog/provider,
public deletion/schema/migration, late-import, live collection or new feature.

Use pinned1.95/offline, approved build recipes, exclusive target and no cache
cleanup/lock changes. Run fmt before final tests; existing-policy scoped Clippy
for six affected crates with --locked --no-deps, no extra global -D warnings or
lint suppression. Retain original failing log. Newly exposed findings outside
seven paths require manager classification, not broad repair or a false pass.
Run complete state/API/TaskNode tests and affected telemetry/retention/native
cases, normal-library check and both governance checkers. Manager reruns affected
combined gates; 71+840 accepted evidence is not a replacement for changed tests.

Increment-specific independent functional execution N/A: this is an internal
behavior-preserving lint correction, with no user-facing handoff. Later PF60
interactive/caller acceptance remains subject to policy1.7; historical tests
are not upgraded. S02 remains in_progress; S03 stays draft. The next actual
feature unit is bounded90..365-day compact-only import, already proposed privately,
not a reason to withhold these concrete fixes or ask Travis another question.
