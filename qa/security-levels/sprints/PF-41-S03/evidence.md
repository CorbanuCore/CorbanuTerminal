# PF-41-S03 durable security-event foundation evidence

## Candidate and scope

- Dispatch base: `9d08b15fa94676c1383ee1605b77e7cc7218dcc4`.
- Allocation commit: `e0c23fe95165636d621dae8c16a5366c4f7250ac`.
- Final implementation candidate: `8fc225e3e534c596bc5b26f91d614926df2b5362`.
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
links, generation monotonicity, hash chain, owner/key generation and the
protected high-water mark. Records ahead of the protected root are ambiguous
commits and never replay automatically. An operator can accept exactly one
published record only by matching its returned event ID and live
policy/revocation state through `reconcile_ambiguous_commit`; the journal stays
blocked until a new full recovery, and a dispatch intent remains pending for an
explicit unknown receipt. Unknown outcomes are terminal.
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
survives timestamp and policy/run generation changes.

## Automated evidence

All target, Cargo home, temporary, log and review data is under
`/Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/`. Because shared
workspace registration is integration-owner-only, the lane tests the exact
crate source through the isolated manifest at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/harness/Cargo.toml`.

| Check | Result |
| --- | --- |
| `rustfmt +nightly-2025-09-18 --edition 2024 codex-rs/security-audit/src/*.rs codex-rs/security-audit/tests/*.rs` | PASS |
| `cargo +1.95.0 test --manifest-path /Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/harness/Cargo.toml` with CorbanuDrive target/temp | PASS; 34 unit/fault + 1 public integration = 35/35 |
| `cargo +1.95.0 clippy --manifest-path /Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/harness/Cargo.toml --all-targets -- -D warnings` | PASS; isolated workspace mirrors every root workspace Clippy deny |
| `python3 -m unittest discover -s qa/security-levels/audit-foundation -p 'test_*.py' -v` | PASS; 3/3 |
| `python3 -m unittest docs.plans.tests.test_check docs.sprints.tests.test_check` | PASS; 23/23 |
| `python3 docs/plans/check.py && python3 docs/sprints/check.py` | PASS; active 1/2, current 61, archived 94 |
| `git diff --check` | PASS |

Exact-commit final artifacts for `8fc225e3e534c596bc5b26f91d614926df2b5362`:

- Rust test log: `test-rust-remediation.log`, SHA-256
  `aa0692c55b6e1cc42592565e6fe247ab1e18318d3445eda52d8b750b56daefd0`.
- full-workspace-lint Clippy log: `clippy-remediation.log`, SHA-256
  `4073809757782ba97b75c54de485c378016c5759b132d3aae576d6f3c8c1d920`.
- fixture log: `test-fixture-remediation.log`, SHA-256
  `c80a081792c1328f46d1f385824d459d636bcb7a3b834976c060563bf2cef3fb`.
- governance test log: `test-governance-remediation.log`, SHA-256
  `0b95fd98e4e2151675c48f59f686dd31c8a977408f4ba112449907ab76a9b1f4`.
- isolated manifest with the exact root Clippy deny set: `harness/Cargo.toml`,
  SHA-256
  `b1de95677cf4f772334b66a7be5d07b0de7146643034ef31f901bccc941066e9`.

All paths above are beneath
`/Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/`. Earlier failing
attempts were overwritten by these exact-candidate logs; PASS claims refer only
to the hashes above.

The fault suite covers disk full before write; timeout before write; crash after
record sync; crash after no-clobber publish; protected-root unavailability;
lost acknowledgment after root commit; retry/deduplication; truncation,
mutation, rotation, saturation, missing key, owner rotation and concurrent
writer recovery. It also verifies immediate emergency fencing when the audit
write fails and restart blocking while the restriction ledger has a gap.

The post-review regressions additionally cover duplicate-reservation permit
rejection before and after terminal resolution; mandatory first recovery;
nonzero first-install and forward policy generations with rollback rejection;
post-publish directory-sync ambiguity; visible recovered pending intents;
explicit unknown reconciliation; new-dispatch blocking during reconciliation;
terminal receipt recording after a live generation advance; timestamp- and
generation-independent retry classification; exact one-record operator
reconciliation; validated-tail cache reuse; and cache invalidation when the
protected root changes.

## TMUX smoke and independent review

The final implementation candidate `8fc225e3e534c596bc5b26f91d614926df2b5362`
ran in real TMUX session `pf41-durable-smoke-final` from the candidate worktree
using Corbanu Terminal v0.1.35 with `RUST_LOG=trace`, read-only/never
permissions, exact model `claude-opus-5-plan` at `max`, and an explicit
CorbanuDrive `log_dir`. `/status` confirmed the candidate directory, connected
Claude Plan account, exact model and Corbanu version; `/quit` exited the session
cleanly. Command text and Enter were sent as separate TMUX operations.

- Status capture:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/tmux-smoke-final/status-pane.txt`,
  SHA-256 `ee6206e6562cb90ca19a2e91bd87ef3cf468ea5fd1744f67e3222da5c60a4d4a`.
- Trace log:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/tmux-smoke-final/logs/codex-tui.log`,
  SHA-256 `1c8cab1957b808595498edfa25c9feafe683f5b30d07ffdeff01cc4c4c68dbb5`.

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
universe surfaces, and run the specific security-audit Bazel target plus the
required repository Bazel validation. No Bazel PASS is claimed here. Those
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
