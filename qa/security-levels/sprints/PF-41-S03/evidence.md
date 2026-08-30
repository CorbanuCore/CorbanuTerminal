# PF-41-S03 durable security-event foundation evidence

## Candidate and scope

- Dispatch base: `9d08b15fa94676c1383ee1605b77e7cc7218dcc4`.
- Allocation commit: `e0c23fe95165636d621dae8c16a5366c4f7250ac`.
- Implementation candidate: `c7072e889e539073104be6c771f704f00adbf373`.
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
commits and never replay automatically. Unknown outcomes are terminal.
Temporary cleanup recognizes only exact journal temporary names and cannot
delete unrelated files. Emergency restriction applies the PF-19 fence before
the audit write; a failed write exposes an audit gap and PF-20/reconstructed
state disagreement blocks restart.

## Automated evidence

All target, Cargo home, temporary, log and review data is under
`/Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/`. Because shared
workspace registration is integration-owner-only, the lane tests the exact
crate source through the isolated manifest at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/harness/Cargo.toml`.

| Check | Result |
| --- | --- |
| `rustfmt +nightly-2025-09-18 --edition 2024 codex-rs/security-audit/src/*.rs codex-rs/security-audit/tests/*.rs` | PASS |
| `cargo +1.95.0 test --manifest-path /Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/harness/Cargo.toml` with CorbanuDrive target/temp | PASS; 25 unit/fault + 1 public integration = 26/26 |
| `cargo +1.95.0 clippy --manifest-path /Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/harness/Cargo.toml --all-targets -- -D warnings` | PASS |
| `python3 -m unittest discover -s qa/security-levels/audit-foundation -p 'test_*.py' -v` | PASS; 3/3 |
| `python3 -m unittest docs.plans.tests.test_check docs.sprints.tests.test_check` | PASS; 23/23 |
| `python3 docs/plans/check.py && python3 docs/sprints/check.py` | PASS; active 1/2, current 61, archived 94 |
| `git diff --check` | PASS |

The fault suite covers disk full before write; timeout before write; crash after
record sync; crash after no-clobber publish; protected-root unavailability;
lost acknowledgment after root commit; retry/deduplication; truncation,
mutation, rotation, saturation, missing key, owner rotation and concurrent
writer recovery. It also verifies immediate emergency fencing when the audit
write fails and restart blocking while the restriction ledger has a gap.

## TMUX smoke and independent review

The exact implementation candidate ran in real TMUX session
`pf41-durable-smoke` from the candidate worktree using Corbanu Terminal
v0.1.35 with `RUST_LOG=trace` and an explicit CorbanuDrive `log_dir`. `/status`
confirmed the candidate directory, connected Claude Plan account and Corbanu
version; `/quit` exited the session cleanly. Command text and Enter were sent
as separate TMUX operations.

- Status capture:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/tmux-smoke/status-pane.txt`,
  SHA-256 `a89552e805022b3a57728a3c2c069f4bf076f14bd34d382582f04e15a3aa80c1`.
- Trace log:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/durable-events/tmux-smoke/logs/codex-tui.log`,
  SHA-256 `bfe4102eb8fc1deb132184f5e3a5663ca2ad7e1f6d6dca22d938b2cf70222443`.

The read-only Claude Opus 5 Plan Max review is recorded here after it
completes. Prompt and Enter are sent as separate TMUX operations; the raw
transcript remains outside Git and only its SHA-256 digest enters this evidence.

## Consumer and integration handoff

The integration owner must add `security-audit` to the root Rust workspace,
add the `codex-security-audit` path dependency and `fslock` workspace
dependency, update `Cargo.lock`, and perform required root Bazel registration.
Those shared surfaces are intentionally unchanged here. Integration must rerun
this full suite plus consumed PF-19/PF-20 tests and governance before archive.

PF-20 must supply the authenticated protected-root adapter. Later broker,
quarantine, financial and Sweep consumers must revalidate live authorization,
revocation and dispatch-fence state immediately before each effect, implement
adapter-specific reconciliation for unknown outcomes, and provide real adapter
evidence. PF-41-S02 and PF-26 retain joined inspection/export and final
end-to-end chain qualification. This fixture cannot activate any consumer.
