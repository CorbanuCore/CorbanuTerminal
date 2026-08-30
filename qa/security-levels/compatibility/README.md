# Expanded Permissive compatibility control

Contract version 2 preserves the immutable PF-21-S01 oracle and adds a genuinely
different upstream source control. `upstream-control-v2.json` derives its nine
inherited-compatibility expectations from the buildable Corbanu 0.1.34 hotfix
`af5a4e39b590e7517120fd935ccfac8cbf7cf131`. It descends from convergence commit
`45a60f03d2f6c041d284b41cc3f33c416d9eeed1`, whose upstream Codex parent is
`413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`. The harness verifies the pinned
upstream and baseline `codex-rs` tree identities differ before executing either
control; the two trees differ by 30 files (+1,082/-251).

The harness builds the pre-feature baseline, upstream control, and final
candidate in separate source and target directories. It runs all nine inherited
cases against all three builds, then runs four candidate-only protected-boundary
cases and the original five immutable probes. The four protected cases directly
exercise `codex-secret-broker`, `codex-network-proxy`,
`codex-browser-isolation`, and `codex-content-security`; their source identities
are pinned to dispatch base `77f56da1ecddf6093184280b541339e1869ca7b3`, not
derived from the candidate. They prove the platform-eligibility, explicit
credential-injection opt-in, Permissive browser-inactivity, and current-verdict
screening boundaries respectively. They do not claim that unwired future
consumers have already received PF-26 end-to-end qualification.

Candidate output never creates or refreshes a golden value. The candidate
runtime inputs (`codex-rs`, root `justfile`, `scripts/just-shell.py`, and
`codex-rs/.cargo/config.toml`) must be clean; the three tracked recipe inputs
are hashed into the report. Every expanded and protected executable filter
names an exact, lexically verified Rust test and must execute exactly one test.
The five immutable S01 probes retain their accepted historical
`executed_tests > 0` contract.

`drift-ledger-v2.json` is intentionally empty: the reviewed source blocks for
all nine inherited cases are byte-identical across upstream, baseline, and the
candidate at allocation. A future difference must identify the affected control
identity, case, both exact source hashes, disposition, and rationale. Unknown,
duplicate, unobserved, rejected, or stale entries fail. The script owns and caps
the review lifetime at 30 days; the ledger cannot extend it.

Output, cache, temp, and candidate target roots inside the repository worktree
are rejected. Per-run detached control sources are removed even after failure.
At most the two currently pinned baseline/upstream target caches are retained;
stale baseline/upstream target directories are pruned on the next run. A
cleanup-only failure cannot discard otherwise complete test evidence: the
report preserves the verdict, leaked run-root path, and explicit
`cleanup_warnings` for operator remediation.

The report records only allowlisted platform/tool facts and whether three
behavior-affecting ambient variables were present. It never records environment
values, user configuration, credentials, or credential paths.
