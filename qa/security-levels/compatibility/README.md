# Expanded Permissive compatibility control

Contract version 2 preserves the immutable PF-21-S01 oracle and adds a second,
independently constructed control. `upstream-control-v2.json` derives its test
expectations only from the last qualified pre-baseline upstream-aligned Corbanu
release, `b0dc0624326c706fec5329fd48ed44f243937469` (0.1.35). It descends from
convergence commit `45a60f03d2f6c041d284b41cc3f33c416d9eeed1`, whose upstream Codex parent is
`413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`.

The harness builds the pre-feature baseline, upstream-aligned control, and final
candidate in separate source and target directories. It runs all nine expanded
cases against all three builds, then runs the original five immutable probes on
the candidate. Source expectations are recomputed from the upstream commit;
candidate output never creates or refreshes a golden value. The candidate Rust
tree must be clean, every expanded filter must name its exact source function,
and every expanded command must execute exactly one test.

`drift-ledger-v2.json` is intentionally empty: the reviewed source blocks for
all nine cases are byte-identical across upstream, baseline, and the candidate
at allocation. A future difference must identify the affected control identity,
case, both exact source hashes, disposition, and rationale. Unknown entries,
unobserved entries, rejected entries, or a ledger older than 30 days fail.

The report records only allowlisted platform/tool facts and whether three
behavior-affecting ambient variables were present. It never records environment
values, user configuration, credentials, or credential paths.
