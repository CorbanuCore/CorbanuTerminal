# PF-60-S02 first native-state increment — frozen worker receipt

Product initiative PF-60; current sprint PF-60-S02 remains in_progress. Product citation: **Product measurement** — “No commercial performance numbers have been supplied.” The corbanu-terminal-development skill routed this implementation through the approved plan, sprint and reviewed allocation; shared ledgers remain parent-owned. Approved v1 defaults and accepted/archived S01 were not reopened.

Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-pf60-s01-20260911`; historical branch: `workstream/accounting-pf60-s01-20260911`. Clean launch and unchanged HEAD: `51cc326b6c84fb252eeb775ac730b2a2b58e5110`; recorded allocation ancestor: `265bf0c3e164e497d172f1a8e5a56bf4cd54ed46` (ancestry check passed). Source authority: reviewed `docs/research/agent-cost-accounting/s02-allocation.md`, approved S01 contract and literal synthetic Rust fixtures; no Python oracle imports.

Exactly five changed files: the four Rust paths below and this receipt. Rust change: 335 non-test lines (134 journal + 199 types + 2 registration), 428 test lines, 763 total. Implementation remains below 500 non-test lines; the larger test share exercises actual SQLite transactions and reopening. No staging, commit, integration, additional agents/reviewers/model calls, network, credential/private-data reads, production collection or billing.

| Frozen Rust file (repository relative) | SHA-256 |
| --- | --- |
| `codex-rs/state/src/runtime.rs` | `93c2dd7aaa0ee3840b290ecd0c57641c98097a0762e35b3367b3d92d79a5e81f` |
| `codex-rs/state/src/runtime/accounting.rs` | `3ec5116d8bde5c3202e13f24599ae754b2fa2051f4558e9ec103fbc0b48c578c` |
| `codex-rs/state/src/runtime/accounting_types.rs` | `6ada01cd46f9e28deed5d5c986ff66bf8216f113f6bccf22634d2d8140909eed` |
| `codex-rs/state/src/runtime/accounting_tests.rs` | `39cac743587ba298558007140be892eac6a9a84e8d881be4fe32d78369dc5e70` |

Rust patch SHA-256: `555fab5b86aa840bb6adf00b9d1ba87c809bee2809c19cbc953a8600147f6726`. Reproduce from repository root by concatenating `git diff --no-ext-diff --binary --full-index HEAD -- codex-rs/state/src/runtime.rs`, then `git diff --no-ext-diff --no-index --binary --full-index -- /dev/null PATH` for accounting.rs, accounting_types.rs, accounting_tests.rs in that order; pipe bytes to `shasum -a 256`. New-file diff exit 1 means differences. Delivery supplies the five-file patch hash including this receipt, avoiding a self-referential digest.

Validation captured 2026-09-12 UTC (September 11 Phoenix); focused result observed by 00:15:21Z, governance/hash capture 00:16:16Z. Toolchain: preinstalled rustc 1.95.0 (59807616e), aarch64-apple-darwin, cargo-nextest 0.9.143. Rust commands used `RUSTUP_TOOLCHAIN=1.95.0 RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true`; formatter also used `UV_OFFLINE=true`. No installs/fetches or manifest/toolchain repairs. A macOS sandbox denied network and worktree writes except the four Rust files during formatting, or codex-rs/target during build/tests; no shared processes or locks were killed.

| Command (codex-rs unless stated) | Actual result |
| --- | --- |
| `just fmt` | Exit 1: Rust, Just and Bazel groups succeeded; Python SDK/scripts groups failed. uv warned about existing `exclude-newer = "7 days"` settings; creating their .venv directories was denied by the scope guard. No out-of-scope changes. Formatting ran before all final Rust checks. |
| `CARGO_NET_OFFLINE=true just test -p codex-state runtime::accounting::tests` | Exit 0; 12 passed, 0 failed, 190 filtered; build 51.27s, tests 0.179s; run `5fcf7855-194f-4c3d-9518-0248bed322d2`. |
| `CARGO_NET_OFFLINE=true just test -p codex-state` | Exit 0; 202 passed, 0 failed, 0 skipped; 6.345s; run `9d211fe3-f5af-4b56-9b47-d12238a7b9fe`. One existing case marked LEAK: `extract::tests::turn_context_sets_permission_profile_metadata`; no accounting case marked leaky. |
| `cargo check --offline --locked -p codex-state --lib` | Exit 0; normal library compiled in 26.59s. No compile blocker and no direct cargo test invocation. |
| Root: `python3 docs/plans/check.py`; `python3 docs/sprints/check.py`; `git diff --check` | Exit 0 each; 3/3 active, 115 current / 122 archived; no whitespace errors. |

OFF proof: runtime.rs adds only `#[cfg(test)] mod accounting;`. Draft DDL runs solely through explicit test setup using the existing StateRuntime pool. The normal-init regression found no draft tables; normal-library compilation excludes the entire private journal. No lib.rs, migration, manifest, lock, BUILD, config, API/Core, throttle, TaskNode or shared registration beyond that declaration changed.

Behavior: caller UUID request/attempt/scope identities and native ThreadId; bounded turn/provider/model metadata; immutable request ownership and retry predecessor validation. BEGIN IMMEDIATE serializes writers; identity, revisions and exact source positions commit together. Typed canonical duplicates are no-ops; payload/presence/position conflicts and invalid ordered prefixes roll back all writes. Counts are exact nonnegative i64, revisions positive, batches capped at 256. Unknown fields, fractional/bool/negative/overflow counts, impossible inclusive cache/reasoning subsets and conflicting totals are rejected.

Evidence includes immutable-field conflicts, source-position collisions, concurrent same-key writers, rollback of new and existing attempts, omitted/null/zero round trips, and revision gaps 1/3/7 with positions 10/30/70. Actual closed SQLite pools reopen from disposable on-disk homes, then replay twice without duplicate rows; an intent with no observation stays unknown. Native Anthropic input 50/read 10/write absent preserves noncached 50 and unknown inclusive input; explicit write 0/output 5 produces inclusive 60/total 65. Unknown compatible input never inherits native noncached semantics. No costs are computed.

Parent owns independent review, literal five-file/hash verification, combined-tree checks and local integration. Human review prerequisites: this pinned uncommitted candidate, the registered commands, preinstalled toolchain/cache and disposable synthetic homes. There is no user accounting control or human product-test readiness. Close/reopen is not a process-kill test; crash interruption between transactions, production migrations/dispatch/presence wiring, prices, lineage, retention, tombstones and full S02 golden totals need later exact allocations. The selected future per-thread/day aggregate approach remains unchanged; no aggregate/pricing/retention engine was added. Full S02/S03, actual-key TUI, live-repository qualification, named runtime acceptance, benchmarks and release gates remain open.
