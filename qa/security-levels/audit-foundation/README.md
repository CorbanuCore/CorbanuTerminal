# Durable security-event foundation v1

PF-41-S03 defines a fixture-only consumer contract. It does not register an
event producer, expose a runtime route, enable a protected profile, or qualify a
broker, quarantine, financial, or Sweep adapter.

## Ownership and durability

- `codex-security-audit` owns versioned secret-free event, decision, action and
  reservation identities plus the reference append/recovery protocol.
- PF-16–20 continue to own actors, requests, grants, mandates, receipts,
  revocations, policy generations and controller-authoritative state.
- A producer owns exactly one journal ownership generation. Events from a
  different producer, an old owner generation, or a missing integrity key fail
  closed.
- The local record hash chain detects corruption but is not a host-compromise
  boundary. `IntegrityRootStore` must be implemented by the PF-20
  controller-owned protected store and must compare-and-store the exact prior
  checkpoint durably.
- The cross-store commit order is record write, record sync, atomic no-clobber
  publish, directory sync, protected-root compare-and-store, then
  acknowledgment. A record ahead of the protected root is an ambiguous commit
  and is never replayed automatically. An operator may advance the protected
  root over exactly one published record only through
  `reconcile_ambiguous_commit`, after matching the returned event ID and live
  policy/revocation state. The journal stays blocked until another full
  recovery; a reconciled dispatch intent is then exposed as pending and must be
  resolved unknown rather than replayed.

## Dispatch contract

Consumers call `reserve_dispatch` before external side effects. Only a returned,
non-serializable `DispatchPermit` proves that intent reached the protected root.
Disk full, deadline expiry, saturation, missing keys, writer conflict and failed
persistence return no permit. After dispatch, consumers append either a
completed result (with the existing PF-18 mandate receipt where applicable) or
an explicit unknown result. Unknown is terminal and requires human or
adapter-specific reconciliation; it is not a replay instruction.

Every journal starts blocked and requires successful recovery against the live
PF-20 policy generation and revocation state before its first append. Retrying
the same action and deduplication key never returns another permit, even when
the retry timestamp or policy/run generation changes: unresolved duplicates
return `AlreadyReserved` and resolved duplicates return `AlreadyResolved`.
Recovery exposes all durable intents without a terminal receipt as
`pending_dispatches`, reports `ReconciliationRequired`, and blocks new dispatch
until each is appended as explicitly unknown through
`reconcile_dispatch_as_unknown`. Normal resolution accepts the current live
event context so a policy/run generation advance cannot strand an older intent.

Full recovery validates every local record and seeds a process-local validated
chain/tail cache. Normal appends compare the protected checkpoint with that
cached checkpoint and validate only the new record, avoiding an O(n) rescan on
the dispatch path. A protected-root change, restart, failed/ambiguous append or
explicit recovery invalidates or rebuilds the cache. The external protected
checkpoint remains authoritative.

Emergency restriction uses PF-19 state first and attempts the audit append
second. Failed audit persistence cannot delay or undo the fence. Recovery
compares the reconstructed restriction ledger with PF-20 state, exposes any gap,
and blocks protected dispatch until reconciled.

On Windows, where Rust cannot fsync a directory handle, the local directory
sync step is a documented no-op matching the existing PF-20 store. The external
protected root remains authoritative: loss of a local entry is detected as
truncation and blocks recovery rather than becoming silent success.

[`consumer-contract-v1.json`](consumer-contract-v1.json) is a machine-readable
handoff fixture. Its `runtime_activation` value is permanently `false`; real
consumer adapters and final PF-41/PF-26 qualification supply their own evidence.
