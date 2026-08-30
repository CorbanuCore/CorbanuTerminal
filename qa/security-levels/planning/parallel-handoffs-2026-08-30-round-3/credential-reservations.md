# PF-13-S06 credential-reservations handoff

Owner: Pauli. Branch/worktree/base and literal scope are authoritative in the
sprint front matter. Shared exports, manifests, locks and navigation belong to
the integration owner.

Implement only the PF-13-S06 remaining ledger. Extend the accepted opaque
credential capability and `BoundedGrant`; do not add a parallel authorization
type, resolve a vault value, activate a transport or create a raw-secret return
path. Reserve worst-case bounded request/token/byte/spend use before dispatch
and settle only authenticated trusted metering. Cancellation, partial results,
retries, duplicates, concurrency, expiry, revocation and unknown outcomes must
never replenish or double-spend authority. Preserve the private unguessable
token and digest-only public identity.

Test every named exhaustion, forgery, changed-binding and settlement case in
the sprint, record the later broker/transport consumer handoff, and keep
Permissive/runtime behavior unchanged. Perform the round-three common TMUX and
Opus review protocol before committing the handback.
