# PF-41-S03 durable security-event foundation evidence

## Candidate and scope

- Dispatch base: `9d08b15fa94676c1383ee1605b77e7cc7218dcc4`.
- Allocation commit: `e0c23fe95165636d621dae8c16a5366c4f7250ac`.
- Final implementation candidate: `3f8cef302caaf9658a84b7c488c08cead50a6402`.
- Contract versions: security-audit event schema v1, integrity checkpoint v1,
  journal record v1 and consumer fixture v1.
- Activation posture: fixture-only and fail closed. No producer, consumer,
  runtime route or protected profile is registered by this lane.

All repository changes are inside the literal PF-41-S03 write scope:

- `codex-rs/security-audit/`
- `qa/security-levels/audit-foundation/`
- `qa/security-levels/sprints/PF-41-S03/`
- `docs/sprints/current/p0-security-levels/pf-41-s03-durable-security-event-foundation.md`

## Contract result

The implementation provides versioned, canonical SHA-256 identities for
security events, decisions, actions and reservations. It reuses PF-16–20 actor,
request, grant, mandate, receipt and revocation types while retaining only a
secret-free request identity. Serialized grant and mandate IDs are correlation
values, never reusable authority. The non-serializable, non-cloneable dispatch
permit is issued only after a durable intent checkpoint and remains subject to
live PF-16–20 revalidation by each real consumer.

The reference journal has explicit bounds and segmented append-only records.
Its commit order is record write and sync, atomic no-clobber publish, segment
directory sync, external protected-root compare-and-store, then acknowledgment.
The `IntegrityRootStore` contract requires PF-20 controller ownership, exact
CAS and durable completion. `Ok(None)` is restricted to an authenticated first
install; missing/deleted roots or keys fail closed. A local hash chain alone is
explicitly not a host-compromise boundary.

Recovery validates storage names, bounds, event identities, sequence, causal
links, live policy and run generations, generation monotonicity, hash chain,
owner/key generation and the
protected high-water mark. Records ahead of the protected root are ambiguous
commits and never replay automatically. An operator can accept exactly one
published record only by matching its returned event ID and live
policy/revocation state through `reconcile_ambiguous_commit`; the journal stays
blocked until a new full recovery, and a dispatch intent remains pending for an
explicit unknown receipt. Unknown outcomes are terminal.
Pending reports expose the durable intent timestamp, and unknown reconciliation
clamps a backwards wall clock to that value so an operator cannot be stranded
by a clock step-back.
Temporary cleanup recognizes only exact journal temporary names and cannot
delete unrelated files. Emergency restriction applies the PF-19 fence before
the audit write; a failed write exposes an audit gap and PF-20/reconstructed
state disagreement blocks restart.

Successful recovery seeds a validated process-local chain/tail cache. Each
append compares the PF-20 checkpoint with the cached checkpoint and validates
only the candidate record. A root change or failure invalidates the cache and
restart/recovery rebuilds it, removing the prior O(n) scan from every dispatch
without treating the local tail as the protected authority. Dispatch identity
deduplication is indexed by action plus hashed deduplication key and therefore
survives clock, session, task, policy/run generation and fresh grant/mandate
authority changes. The action digest contains only stable effect semantics
required for replay fencing; live authority correlation is recorded but kept
outside that identity. A live unresolved intent blocks a different dispatch,
while an exact retry preserves the original event, action, reservation and
sequence identity.

`reconcile_ambiguous_commit` accepts only one local record beyond an existing
protected checkpoint, and validates the protected prefix hash, policy/run
generation, producer, owner generation and integrity-key identity before its
CAS. Missing roots, changed owners and anchor mismatches fail closed. A real
`IntegrityRootError::Timeout` after local publication maps to `CommitUnknown`.
The normal write path returns distinct unavailable, concurrent-change and
invalid-root errors for definite CAS failures, and append-time chain rejection
returns a structured invariant reason.
During operator reconciliation, definite conflict/invalid errors map to a
mismatch and missing/unavailable roots remain unavailable; only a timeout is
ambiguous. All blocking write-path errors invalidate the validated tail cache.
Transport acknowledgement loss is outside the in-process journal contract;
callers retry the same stable semantics and deduplication key and receive the
original detailed duplicate outcome.

## Automated evidence

All target, Cargo home, temporary, log and review data is under
`/Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/`. Because shared
workspace registration is integration-owner-only, the lane tests the exact
crate source through the isolated manifest at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/harness/Cargo.toml`.

| Check | Result |
| --- | --- |
| `rustfmt +nightly-2025-09-18 --edition 2024 codex-rs/security-audit/src/*.rs codex-rs/security-audit/tests/*.rs` | PASS |
| `cargo +1.95.0 test --manifest-path /Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/harness/Cargo.toml` with CorbanuDrive target/temp | PASS; 40 unit/fault + 1 public integration = 41/41 |
| `cargo +1.95.0 clippy --manifest-path /Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/harness/Cargo.toml --all-targets -- -D warnings` | PASS; isolated workspace mirrors every root workspace Clippy deny |
| `python3 -m unittest discover -s qa/security-levels/audit-foundation -p 'test_*.py' -v` | PASS; 3/3 |
| `python3 -m unittest docs.plans.tests.test_check docs.sprints.tests.test_check` | PASS; 23/23 |
| `python3 docs/plans/check.py && python3 docs/sprints/check.py` | PASS; active 1/2, current 61, archived 94 |
| `git diff --check` | PASS |

Exact-commit final artifacts for `3f8cef302caaf9658a84b7c488c08cead50a6402`:

- Rust test log: `test-rust-final-4.log`, SHA-256
  `c067b2ba4769044f7f4fccbad7861a67b8e347091558f7c73ce00113c1c40ffb`.
- full-workspace-lint Clippy log: `clippy-final-4.log`, SHA-256
  `8c233a54790c4800d3dd585436af679aa838da259bfdb129412e2d51cdb75cdc`.
- fixture log: `test-fixture-final-4.log`, SHA-256
  `33060fd73862c5e1af97335504d5b45211fce6d229657f556d8deceeadc52ce4`.
- governance test log: `test-governance-final-4.log`, SHA-256
  `6088abc864f7d56c5fe6b87bf1e29d29b5316cd06831624ef6f758ff86d337`.
- isolated manifest with the exact root Clippy deny set: `harness/Cargo.toml`,
  SHA-256
  `b1de95677cf4f772334b66a7be5d07b0de7146643034ef31f901bccc941066e9`.

All paths above are beneath
`/Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/`. Earlier failing
attempts were overwritten by these exact-candidate logs; PASS claims refer only
to the hashes above.

The fault suite covers disk full before write; crash after record sync; crash
after no-clobber publish; protected-root unavailability and a real
protected-root timeout; retry/deduplication; truncation,
mutation, rotation, saturation, missing key, owner rotation and concurrent
writer recovery. It also verifies immediate emergency fencing when the audit
write fails and restart blocking while the restriction ledger has a gap.

The post-review regressions additionally cover duplicate-reservation permit
rejection before and after terminal resolution; mandatory first recovery;
nonzero first-install and forward policy generations with rollback rejection;
post-publish directory-sync ambiguity; visible recovered pending intents;
explicit unknown reconciliation; new-dispatch blocking during reconciliation;
terminal receipt recording after a live generation advance; clock-, session-,
task-, generation- and reissued-authority-independent retry classification with
original identity reporting; live dropped-permit fencing; exact one-record
operator reconciliation with protected-prefix, owner and precise CAS-error
validation; run-generation rollback at recovery and ambiguous reconciliation;
observable/clamped pending
timestamps; structured chain failures; precise write-path CAS errors;
validated-tail cache reuse; and cache invalidation on protected-root,
writer-lock and persistence failures.

The foundation was landed in reviewable commits: initial event/storage/journal
contracts and fixture, identity redaction, recovery/cache hardening, and focused
review remediations. Journal lifecycle/append (`journal.rs`, 373 lines),
filesystem publication (`journal_io.rs`, 131 lines), recovery/reconciliation
(`journal_recovery.rs`, 372 lines), event contracts (`event.rs`, 461 lines),
and canonical event identities/errors (`event_identity.rs`, 118 lines) are
cohesive production boundaries below the 500-line target. The former debug-only
micro-module was folded into the core type. Record-first/root-last ordering
stays directly visible in the append and recovery state machines rather than
being split at individual transitions.

## TMUX smoke and independent review

The final implementation candidate `3f8cef302caaf9658a84b7c488c08cead50a6402`
ran in real TMUX session `pf41-durable-smoke-final-5` from the candidate worktree
using Corbanu Terminal v0.1.35 with `RUST_LOG=trace`, read-only/never
permissions, exact model `claude-opus-5-plan` at `max`, and an explicit
CorbanuDrive `log_dir`. `/status` confirmed the candidate directory, connected
Claude Plan account, exact model and Corbanu version; `/quit` exited the session
cleanly. Command text and Enter were sent as separate TMUX operations.

- Status capture:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/tmux-smoke-final-5/status-pane.txt`,
  SHA-256 `8d4d62a3d24b4e0411babd7cff47375a6e1638a67d29e73cfb58ddecbf6d7388`.
- Trace log:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/tmux-smoke-final-5/logs/codex-tui.log`,
  SHA-256 `36f55656a0445489410921b57e6510585f65817d3133215143e96f37e6fbf8fc`.

The first read-only review used real TMUX session `pf27-opus5-g1-review`
with exact model `claude-opus-5-plan` at `max`. It found nine actionable
issues: duplicate-permit replay, policy-generation recovery deadlocks, Windows
directory sync, post-publish ambiguity classification, mandatory startup
recovery, invisible unresolved intents, generation-advanced resolution,
redundant full scans and an inaccurate authority-construction claim. Candidate
`7ef637790252036771742e3117d04197fa8e32d4` fixes all nine, adds the regressions
above, reduces append to one full scan plus incremental candidate validation,
and makes unvalidated correlation construction explicit. Transcript:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/review/opus-first-review.txt`,
SHA-256 `895ab3dccf56f07c7f0cd835d96b7e5c28f7714c44db8910a45d7d83141ef2af`.

A second read-only review in the same real TMUX/Corbanu harness used exact model
`claude-opus-5-plan` at `max` and reviewed evidence candidate `b8123ea8` plus
implementation `7ef63779`. It found six remaining issues: generation-dependent
deduplication, contradictory failed test artifacts, timestamp-dependent retry
classification, no ambiguous-root operator procedure, an O(n) append rescan and
an unverified crate-local Bazel target. Candidate `8fc225e3e534c596bc5b26f91d614926df2b5362`
fixes the five lane-owned runtime/evidence issues and makes the Bazel integration
gate explicit. Transcript:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/review/opus-second-review.txt`,
SHA-256 `8c6e22e6088d441b1e2a3b533d0027d8dedfebdc12b14dfc96e27c83a710fae6`.

A third fresh read-only review in the same real TMUX/Corbanu harness used exact
model `claude-opus-5-plan` at `max` and reviewed evidence candidate `09d476c3a`
plus implementation `8fc225e3e`. It found four issues: action identity still
included retry-variant clock/session/task fields; ambiguous reconciliation did
not authenticate its protected prefix and owner; two documented runtime errors
existed only as test injectors; and duplicate errors discarded the original
acknowledgement identity. Candidate `5c8564de07b182c48b31ba0477806837c87fa7c4`
fixes all four and adds focused regressions. Transcript:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/review/opus-third-review.txt`,
SHA-256 `9b28a866538e79cbe7fd0d042fbcf3dc46f6d3eecb62a5d7e491a4e9cc286d2b`.

A fourth fresh read-only review in the same real TMUX/Corbanu harness used exact
model `claude-opus-5-plan` at `max` and reviewed immutable evidence candidate
`c8c4de729` plus implementation `5c8564de0`. It found eight issues: authority
reissue still changed effect identity; live unresolved permits fenced only
after restart; an oversized state-machine module and missing change-size
analysis; incomplete Bazel-lock handoff; inconsistent blocked/cache bookkeeping;
imprecise ambiguous-reconciliation CAS errors; and no final evidence sealing
yet. Candidate `1c190e4fee649167ecc241e8113722d174b74a4c` fixes the seven
implementation/handoff items and this evidence records its exact logs and smoke.
Transcript:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/review/opus-fourth-review.txt`,
SHA-256 `46e15a585b4018c45e15a2aa7c4ffd90defe115f969ed47fc97a5936f3c4c56b`.

A fifth fresh read-only review in the same real TMUX/Corbanu harness used exact
model `claude-opus-5-plan` at `max` and reviewed immutable evidence candidate
`8b96f054de57d703608e3df885d2461f79c2b8c6`. It found six remaining issues: a
run-generation rollback could report recovery ready; a pending intent's
timestamp was not observable for reconciliation after a backwards clock step;
append collapsed chain invariant failures; the normal write path classified
definite integrity-root errors as ambiguous; the journal split did not meet the
500-line target and retained a debug-only micro-module; and duplicate
acknowledgement fields did not explain an original sequence paired with the
current anchoring checkpoint. Implementation candidate
`4a2f1deb7e54410a08277c98fe4cec01e3c16cf5` fixes all six and adds focused
regressions. Transcript:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/review/opus-fifth-review.txt`,
SHA-256 `f45170f3737f2121a4def69c8aadb6012e198ae97bc0a8d068b1bca5a85b047e`.

A sixth fresh read-only review in the same real TMUX/Corbanu harness used exact
model `claude-opus-5-plan` at `max` and reviewed immutable evidence candidate
`200b1a0f934ffda91a27f0f4aa417a24b9b3af8f`. It reconfirmed every earlier
runtime remediation and found three remaining issues: the evidence referenced
an obsolete candidate smoke, `event.rs` exceeded the 500-line production-module
target, and ambiguous reconciliation did not compare the live run generation.
Implementation candidate `3f8cef302caaf9658a84b7c488c08cead50a6402`
fixes all three, adds a run-generation regression, splits canonical identity and
error contracts into the cohesive `event_identity.rs` boundary, and is the
exact candidate exercised by the smoke and logs above. Transcript:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/review/opus-sixth-review.txt`,
SHA-256 `1cc6962a61ef8423a65f198420677a1af7824ed4915ea4516124adefa5ddd288`.

The final post-remediation read-only Claude Opus 5 Plan Max review is recorded
here after it completes. Prompt and Enter are sent as separate TMUX operations;
the raw transcript remains outside Git and only its SHA-256 digest enters this
evidence.

## Consumer and integration handoff

The crate-local `codex-rs/security-audit/BUILD.bazel` already exists, but this
lane cannot truthfully validate it until the shared dependency universe exists.
The integration owner must add `security-audit` to the root Rust workspace, add
the `codex-security-audit` path dependency and `fslock` workspace dependency,
update `Cargo.lock`, register the crate and dependencies in root Bazel/crate
universe surfaces, run `just bazel-lock-update`, include the resulting
`MODULE.bazel.lock` update, and run the specific security-audit Bazel target plus
the required repository Bazel validation. No Bazel PASS is claimed here. Those
shared surfaces are intentionally unchanged. Integration must then rerun this
full suite plus consumed PF-19/PF-20 tests and governance before archive.

PF-20 must supply the authenticated protected-root adapter. Later broker,
quarantine, financial and Sweep consumers must revalidate live authorization,
revocation and dispatch-fence state immediately before each effect, implement
adapter-specific reconciliation for unknown outcomes, expose an operator-owned
procedure that matches `CommitUnknown.event_id` before calling
`reconcile_ambiguous_commit`, and provide real adapter evidence. PF-41-S02 and
PF-26 retain joined inspection/export and final
end-to-end chain qualification. This fixture cannot activate any consumer.
