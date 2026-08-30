# PF-41-S03 durable-events handoff

Owner: Huygens. Branch/worktree/base and literal scope are authoritative in the
sprint front matter. Root workspace Cargo/Bazel/lock/navigation registration is
integration-owner-only.

Implement only the PF-41-S03 durable secret-free event and recovery foundation.
Reuse PF-16–20 identities and generations. Define bounded append/acknowledgment,
integrity checkpoint and recovery interfaces plus a reference durable store.
Intent/reservation precedes dispatch; completion or explicit unknown follows.
Disk full, timeout, failed acknowledgment, ambiguous commit, rollback,
truncation, rotation, saturation, missing keys and concurrent writers must not
become silent success or automatic replay. Emergency restriction still fences
new dispatch immediately when audit persistence fails, while the durable gap
remains visible and fail-closed on restart.

Fixtures may demonstrate the contract but cannot activate producers, protected
profiles, quarantine, brokers, finance or Sweep. Record exact durability and
ownership guarantees for later consumers. Perform the round-three common TMUX
and Opus review protocol before committing the handback.
