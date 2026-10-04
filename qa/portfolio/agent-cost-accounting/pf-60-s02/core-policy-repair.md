# PF-60-S02 Core policy correction — worker receipt

Bounded corrective engineering within active PF-60, not full S02 completion.
Product citation: Product measurement, “No commercial performance numbers have
been supplied.” Allocation: docs/research/agent-cost-accounting/core-policy-repair-allocation.md.
Corbanu development and remote-tests skills governed scope and native fixture use.

## Inputs and ownership

- Worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/accounting-core-policy-20260912.
- Branch: workstream/accounting-core-policy-20260912.
- Base: 37f23b991d6d79ba782aadc50dc5652674c91d20.
- Verified clean launch/unchanged HEAD: 93dc55215c34529c6f78e50aa877a9be0bfbddc1.
- Launch Rust tree: 99265431b6d8f5c0e5742234927cb9bc9443c188.
- Exact ten allocated paths only; no commit/review/children/live actions by worker.
- Target950 total/300 non-test; STOP1100/375, deletions and this receipt included.
- Exclusive target: /Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911/codex-rs/target.
- Process inventory before build found no competing cargo/rustc/clippy/nextest.
- Old workers and canonical source untouched. No lock/dependency/cache/env repair.
- Raw evidence: /Volumes/CorbanuDrive/Corbanu/.codex-work/accounting-core-policy.KP0JC0.

## Implementation and proof

One process-wide semaphore permit precedes operation clocks and durable store
work. Short std locks only copy/publish the retry pointer, after committed
admission. No lock spans HTTP and no Tokio/OwnedMutexGuard workaround is used.
Failure is rechecked after acquiring capacity. An incomplete operation guard
rejects its Sampling; queued cancellation does not reject or write.
Slot read/attach return the existing fatal error on poison, take/reject stale
evidence and reject incoming attachment, leaving poison intact. Drop is
nonpanicking; healthy scope cleanup still preserves healthy response evidence.
The existing turn Result boundary propagates attachment failure. Profile JSON
uses concrete String/Null conversion, not serialization fallback or new routing.

Seven controlled policy tests cover queued admit/observe/start cancellation,
in-operation admit/observe/start cancellation, permit release, failure recheck,
linear retry IDs and after-gate time, previous poison before and after admission,
and poisoned Slot read/attach/drop. Two disk reopens retain durable attempt rows.
The uncertain-completion guard test deliberately retains an already committed
row; it is not a process-kill claim or proof arbitrary SQL cancellation rolls back.
The writer-barrier cancellation case blocks the first SQL write specifically.
One new actual-native local HTTP case holds one owner's response while another
owner admits and records usage, interrupts the first owner, completes a fresh
turn, then resumes twice with a fresh send and no speculative retry predecessor.
All existing accounting/role/stage-one expectations remain; profile whole maps
cover null, empty, quote, backslash, control and Unicode with exact roundtrips.
Two old test connection helpers use the public writable pool, owned detached
connection and closed pool, explicitly retaining foreign keys and SQL faults.
Native payload reads explicitly close their connection before reopen checks.
The old unit attempts reader uses the public read-only pool so a read snapshot
does not initialize writable pragmas under its separate intentional writer lock.

## Commands and outcomes

All Rust commands run from this worker's codex-rs with prefix:
```sh
env RUSTUP_TOOLCHAIN=1.95.0 RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true UV_OFFLINE=true CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911/codex-rs/target
```

Exact command/timing/exit records and output SHA-256s are in the evidence manifest.
Initial governance passed3/115/126. Scoped fix used the recorded sandbox denying
network and worker writes except nine allocated Rust files: just fix -p codex-core
--locked exited101, failed to bind TCP listener to manage locking, OS error1.
No policy relaxation attempted. Scoped rustfmt --edition 2024 --config
skip_children=true over nine literal files exited0; nine stable-toolchain
imports_granularity=Item warnings remain. No broad formatter or source fix.

First focused attempt core-focused-01.log exited101 before test execution:
three new fixture pool calls passed PathBuf instead of &Path (one support source
compiled twice), and Count lacked PartialOrd for a new timestamp assertion.
Unused Connection warnings accompanied that compile failure. Corrected by
borrowing paths, comparing public i64 conversions and explicit connection close.
No assertion removed. One apply_patch context mismatch made no partial change
to that attempted combined patch; reapplied against literal formatted context.
Second focused run exited100:99/100 passed, one existing writer-barrier case
failed both automatic attempts with SQLite code5/database locked. Its read
helper's new writable-pool initialization collided with the held writer; fixed
only the read helper as above, preserving the writable fault path and assertions.
Run7c20eb87-15b9-4410-9172-f4df10f46094,29.200s; raw log and JUnit retained.
Formatting was rerun before the final100 tests. No subsequent Rust edits.

| Final gate | Actual result |
| --- | --- |
| Core accounting/role/stage-one/exec-env/shell selectors | exit0;100/100 passed,0 execution skips,3466 filtered;19.155s;runb82e51a6-4445-44e7-8316-b2f0dd5f63bf |
| Full state/API/TaskNode, serial | exit0;575/575 passed,0 skipped;30.957s;run28309b41-1a31-4152-b7b4-af9898626a04 |
| Six-crate existing-policy Clippy --no-deps | exit101; original eight Core diagnostics absent; newly exposed native helper unwrap_used failures below |
| Normal six-library check | exit0;49.80s |

The focused command was just test -p codex-core -E 'test(accounting) |
test(agent::role::tests) | test(memory_stage_one::tests) | test(exec_env::tests) |
test(tools::handlers::shell::tests)' --test-threads 1 --locked (literal argv in
evidence manifest). Full shared: just test -p codex-state -p codex-api
-p codex-tasknode-session --test-threads 1 --locked. No direct cargo test.
Final100/575 logs have no FAIL/RETRY/FLAKY/LEAK marker. Final Core build retains
four protected-state, ten state, one support and six Core fixture warnings;
shared state test build reports21warnings(9duplicates). Full logs are authoritative.
JUnit was copied from this worker's target/nextest/local path, verified by runUUID,
before each subsequent selector; the canonical build target's old JUnit was not used.

Clippy compiler reports24errors in test all:16 unique pre-existing source sites,
support loaded twice (duplicate_mod warning). Exact current lines:
accounting_anthropic_support.rs39,40,57,208,217,223,228,241;
accounting_anthropic.rs589,590,591,594,599,613,655,766. Full diagnostics preserved.
These are outside the allocated connection-helper/synchronization correction;
no broad helper rewrite was attempted. Manager classification is required before
expansion, not a product or human-approval question. Smallest apparent follow-up:
the same two test files plus receipt, replacing16 existing unchecked fixture
assertions with context-bearing expectations under tests/all.rs's already-existing
expect_used policy; no new lint allowance or production panic. Estimate40–65total
test/receipt lines, no runtime/API/path change. Actual implementation must retain
all fixtures/assertions and rerun formatting, affected tests and Clippy.
Warning-level findings (including state cloned_ref_to_slice_refs and an unchanged
recovery useless_conversion) are retained, not repaired or called strict-clean.

Final governance and diff check exit0 (3active/115current/126archived).
Frozen size:835total/249non-test, including142receipt lines; within950/300.
Literal candidate.diff and frozen-manifest.json in the evidence directory contain
all ten file counts/hashes, exact commands/exits/times and output/JUnit digests.

## Preserved history and limitations

Prior seven-file193/101 correction, its raw Clippy101,575/105 tests and all logs
remain unchanged; prior eight-file298/124 correction's external4/full575,
normal check and raw eight-Core-error Clippy101 are preserved. Parent policy01
review clean exit0/confidence0.96 received2939d46bf at37f23b991.
Parent receiving575/575 zero skips29.165s runab28db7b-2ab7-4f04-8543-40c72596c5ff
and six-library check0 in37.26s are parent-attributed, not rerun baseline claims.
Historical Anthropic reviews01/02 failed and03 clean remain; policy review usage
is not reset. Parent owns this allocation's new material review and correction.
Receiving71+840/normal check history stands. Both failed full-Core gates,
133 baseline-common failures plus14 separately serial-passing additional failures,
parallel fixture failures and all prior LEAK/unknown provenance are not waived.
No full-Core/workspace run was repeated solely to replace those failures.

Increment-specific internal/default-OFF isolated-functional N/A is accepted by
this allocation only. Native test fixtures use build_with_auto_env, not a Linux
executor qualification. No true-TUI/live TensorCash/Isometric/credential/provider
run, benchmark, release or human-ready claim. Manager owns binary-only isolated
execution/probes and independent evidence review before actual collection/replay
handoff and S03 UI acceptance. S02 remains in_progress; S03 draft. The90..365-day
compact-only late-import gap and approved defaults remain unchanged; it is next
bounded feature preparation after corrective receiving gates, not a new blocker.

## Authorized sixteen-assertion continuation

Manager canonical6c94ac012427b25613d9fb8352e3e650fe43c5c9 received docs-only as
52467486094726903e9e86f123ed317278f52ec3, verified actual new HEAD. Parent's
initial101-line sprint checker error and subsequent corrected3/115/126 pass
are preserved as parent-attributed history, not a worker failure or waiver.
Only the16 listed unwrap sites became descriptive expect assertions. Existing
test-binary policy already permits expect; no allowance, fallback, API, fixture
body, identity, timeout, retry, route or zero-send assertion changed.
The other seven Rust files match the first frozen manifest byte-for-byte.
First835/249 candidate,100/575 runs, Clippy101 and all preceding failures remain
in accounting-core-policy.KP0JC0; manifest SHA-256:
8ba2226f1a67c148b93fc6ed1b606c59865bd0cbc50f73b21f92262893dfc40e.
New raw evidence: /Volumes/CorbanuDrive/Corbanu/.codex-work/accounting-core-policy-continuation.4l3rME.
Scoped fix again exit101 (same denied lock listener); two-file rustfmt exit0,
two existing stable imports_granularity warnings. Both preceded final tests.
Corrected source delta adds68 changed test lines over the first candidate.
Corrected candidate:933total/279non-test,172receipt lines; exact hashes in frozen-manifest.json.

| Corrected final gate | Actual result |
| --- | --- |
| Focused100 | exit0;100passed,0execution skips,3466filtered;18.398s;runb91f3284-ff07-4f50-b640-b00fcc29329a |
| Shared575 serial | exit0;575passed,0skipped;31.031s;rune4715dce-be34-4e9f-b0b0-82edb32d9f23 |
| Six-crate existing-policy Clippy --no-deps | exit0;28.97s;existing warning-level findings retained, not workspace strict-clean |
| Normal six-library check | exit0;1.03s;warnings retained |
| Governance/diff | exit0;3active/115current/126archived;diff check clean |

First material review remains unspent and parent-owned; no worker review run.
Source frozen; no final test retries/LEAKs; earlier S02/S03/functional limits stand.
