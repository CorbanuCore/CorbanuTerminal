# PF-27-S04 round-five integration stage

This is an unfinished product initiative, not protected-mode availability.
Product citation: **Non-negotiable controls** — “Permit agents to reference
credentials only by label; resolve them solely inside the trusted execution
boundary.” The current sprint and active plan govern the allocation.

## Provenance and frozen scope

- Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/security-round5-broker`.
- Branch: `feat/security-round5-broker`; source base `07791288b`.
- Allocation: `4f263ca73`; recovered leaf checkpoint: `90ae3a0cf`.
- Recovery source: `recovery/p0-security-isolated-broker-2026-09-02`,
  `cdb821289`. Only assigned code/evidence leaves were recovered; allocation,
  shared registrations and other sprint records were not overwritten.
- New stage: `103d5a106`, `9b90047ef`, `6981d8367`.
- Intended behavior: digest-bound durable dispatch audit and bounded native
  Linux socket/session primitives. No configuration, activation, UI, generic
  secret resolver, platform installer, or shared security-event schema change.

## Implemented stage

- A PF-41 `ReferenceJournal` adapter checks the exact controller/session/task/
  run/generation/reference/operation before reserving a durable dispatch permit.
  It binds the complete original authorization and exact operation/path into a
  digest. Shared events retain PF-41's minimized schema; raw path, destination
  and purpose are not copied into that journal. This supersedes the historical
  handoff's suggestion to persist a raw path in the shared event schema.
- Recovery, protected root and live authorization are external service
  obligations; the adapter cannot create authority or silently recover an
  ambiguous dispatch. A failed/ambiguous commit cannot produce a permit.
- Linux socket peers come from actual `SO_PEERCRED` observations, not frame
  fields. Frames/receipts have allocation bounds and I/O deadlines. EOF,
  malformed input, denial and timeout close the registered generation.
- Client close interrupts in-flight reads/writes without waiting for their
  mutex, and subsequent calls cannot replay on the closed connection.
- A registered runtime connection delegates every frame through the existing
  generation and revocation checks. A fresh same-run generation cannot grant
  its credentials to an old retained connection.

## Executed development checks

Execution was remote-only on the authorized RTX Linux host, under
`/home/travis/worktrees/security-round5-broker`. Shared target/cache builds were
serialized with `/home/travis/security-round5/locks/build.lock` and eight Cargo
jobs. No local compilation occurred.

| Check | Observed result |
| --- | --- |
| `just fix -p codex-secret-broker` | Passed |
| `just fmt` | Initial attempt stopped because `uv` was unavailable; final run pending |
| `just test -p codex-secret-broker` | Development run: 42 passed, 0 skipped; final post-format run pending |
| Native child-service death | Actual fixture subprocess killed after connection; client returned unknown and refused replay |
| Native replacement/revoke | Existing runtime over real Unix sockets: old connection denied after generation replacement, new connection works, revoke denies next dispatch |
| PF-41 persistence | Durable-before-permit, minimized record, wrong binding, ambiguous root, pending restart and terminal outcomes passed |

Development nextest run: `85e821df-db1d-4a75-9dc5-40f12a845c36`.
Raw log: `/home/travis/security-round5/evidence/broker/development-tests.log`.
All credentials in these fixtures are synthetic. Unit fixture authorization
reports are explicitly synthetic and cannot count as platform qualification.

## Pending gates

- Final format/tests and serialized Core/Vault/network-proxy registrations.
- Numbered Astra High autoreview and Fable 5.1 High external TMUX review;
  zero review invocations so far, maximum five per lane.
- Supporting candidate TMUX workflow and combined-tree affected suites.
- Real dedicated-UID Linux service provisioning, macOS authenticated XPC/helper
  isolation, Windows service SID/AppContainer/named-pipe token isolation.
- Real broker-side provider request/response streaming data plane and native
  cached TLS-handler/upload/concurrent-revoke proof against the actual service.
- All-OS qualification, both applicable live repositories, PF-26 final
  candidate evidence, human acceptance and due benchmark evidence.

The service binary still refuses unqualified launch. This stage does not
enable Moderate/Aggressive, finish secretless agent launches, or complete PF-27.
