# PF20S03 scoped review contract

Review the frozen branch diff from immutable base
`601602fa7e53fcb5b41753a0b3607addd45d4415` to the exact HEAD supplied by the
coordinator. Do not edit files or launch nested reviewers. Report concrete,
actionable defects with exact source locations; distinguish adjacent deployment
work from defects in the claimed dependency.

The user explicitly accepted protection against restored old Corbanu data under
trusted OS/admin/controller storage assumptions. Whole-machine/controller-store
rollback, TPM and off-host anchors are outside this sprint. Do not infer that
creating a local file alone establishes protected ownership.

The allocated deliverable is the new Linux protected-state leaf, typed policy
adapter to Core's existing private anchor, existing PF41 trait implementation,
bounded authenticated native transport, and synthetic/native subprocess proofs.
Shared Cargo/Core registrations and exact Cargo lock edge are coordinator-owned
and included. No existing runtime flow activates this provider; it is an explicit
dependency for the separately owned PF27 trusted service composition. Product
profiles must remain visibly unverified and preserve Permissive behavior.

Key invariants: one-time independently durable enrollment, no normal-start reset;
authenticated Genesis and complete owner/key/namespace/install binding; exact
full-value CAS and checked successors; lifetime exclusive controller lock;
bounded no-follow private descriptor-relative IO; file sync, atomic publication,
directory sync before success; corrupt/torn/missing authority and uncertain
publication fail closed without retry or falling back. PF41 is record-first/root-
last; PF20 policy is anchor-first. Neither consumer ordering may be inverted.

Public production factory is actual-root-only/fixed-path; the private in-crate
fixture constructors are not exported. Native transport requires actual Child,
pidfd and post-exec peer matching, fresh channel key, sequence/direction binding,
bounded absolute I/O deadlines and no transparent reconnect. A narrow root/client
is NOT ProtectedModeAuthorization. PF27 owns listener routing, trusted executable
selection, worker credential/ptrace/IPC/filesystem isolation and independent
platform measurements. No privileged installation is authorized or tested here.
Read `pf27-consumer-handoff.md` for exact current seams and limitations.

Same-user child/lock tests and injected ENOSPC/short-write/fsync/receipt failures
prove algorithm/protocol behavior only. Core's platform fixture is explicitly
synthetic. Do not treat it as separate-principal qualification. No physical power
cut, real Vault migration, protected activation or non-Linux adapter is claimed.
Owner/key rotation is rejected pending a separately authorized migration contract.

Diff baseline is 16 Rust/build files, 2,107 added lines including 707 test lines,
plus allocation/design/QA documentation; cohesive production modules are each
below 500 lines. The coordinator approved this staged leaf total exceeding the
usual 800-line guidance. Preserve exact owner boundaries: no broker or Core
client/session/memory changes, no policy protocol redesign or new deployment
permission inferred from review findings.

Review budget for this genuinely new PF20 track is at most five invocations,
normally Astra High then Fable5.1 High through Corbanu/private TMUX. Frozen PF30S01
remains exhausted at five; this review does not re-review that lane.
