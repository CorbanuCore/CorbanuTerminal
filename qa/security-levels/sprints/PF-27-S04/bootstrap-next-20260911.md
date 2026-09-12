# PF27 next stage: trusted-child admission and lifetime

Planning checkpoint, September 11, 2026 (Phoenix). No new runtime implementation,
build, independent review or privileged setup is claimed by this document.

## Scope and authority

Existing product initiative, active `p0-security-levels`, sole security sprint
`PF-27-S04`, owner `/root`. Product heading **Non-negotiable controls**:
“Permit agents to reference credentials only by label; resolve them solely
inside the trusted execution boundary.” The exact sprint allocation remains
`feat/security-broker-resume-20260911` in
`/Volumes/CorbanuDrive/Corbanu/worktrees/security-broker-resume-20260911`, allocation
base `d870c92dab2bf3fbb602dc3b8447fe9f3534aecb`. Current source/main checkpoint is
`c065b8b03f54e856f34e1a0b4315a89d9857aef2`.

This narrows the next implementation proposal; it does not approve the entire
[historical launcher proposal](../../planning/parallel-handoffs-2026-09-04-round-5/linux-synthetic-root-launcher-proposal-v2.md).
The completed [PF20 consumer contract](../PF-20-S03/pf27-consumer-handoff.md)
is authoritative for the native seam. In particular, parent-created socketpairs
are not post-exec child authentication, and same-user tests are not containment.

## Smallest useful source step

Implement a Linux-only service-owned child-admission/lifetime component, not an
installer or a working production root launcher. Keep the normal service's exit
78 and protected-mode eligibility unchanged.

1. The trusted launch path retains actual `std::process::Child` handles and
   immutable role/namespace/generation metadata. No worker-supplied PID, path,
   role, UID or namespace may enroll a child or select authority.
2. Admit an accepted post-exec connection using kernel peer identity against
   that retained child; reject unknown peers, consumed slots and stale/dead
   generations before forwarding a stream. Separate journal and policy roles.
3. Preserve ownership of the selected Child through the eventual
   `ControllerRoot::serve_child(stream, &mut child)` call. PF20 performs its own
   live-child/pidfd check; do not replace it with registry trust alone. Design
   concurrency so a blocked namespace channel cannot block unrelated admission
   or shutdown indefinitely.
4. Make failure/termination an explicit terminal generation state. Timeout,
   EOF, handler failure or either authorized child's death fences the synthetic
   run. Explicit replacement requires fresh children and metadata; no automatic
   reconnect, re-enrollment, CAS retry or transfer of an old capability.
5. Bound registry slots, pending admissions and shutdown waits. Reap or stop
   only owned test children; no broad process-name cleanup.

Literal proposed source scope: `codex-rs/secret-broker-service/` only, plus its
existing PF27 QA records. Shared Cargo/Bazel/lock updates, if a dependency is
necessary, remain serialized by root with the manager. Prefer existing native
peer helpers; do not add another unsafe credential parser casually.
No PF20 API widening, Core policy factory, Vault hookup, listener installation,
identity dropping or provider data-plane work belongs to this stage. If the
small component cannot be useful without those surfaces, report that design
constraint and amend the stage before expanding implementation.

## Qualification matrix to freeze before coding

| Case | Required observable result | Evidence class |
| --- | --- | --- |
| Spawned post-exec child connects | Its retained handle selects only its assigned role/generation | Actual subprocess + Unix socket |
| Unknown child / caller-provided identity | No admission or dispatch to the protected-root handler | Actual peer plus negative API tests |
| Parent-created socketpair | Rejected as child-originated channel | Actual kernel peer test |
| Dead child, reused entry, second connection | No reuse of consumed/dead authority | Actual child lifecycle |
| Journal versus policy routing | No namespace selection by wire input or connection order | Deterministic routing + subprocess tests |
| Full registry / stalled connection | Bounded rejection; other shutdown remains possible | Resource/deadline tests |
| Channel failure / child death | Entire synthetic generation fenced; old handle remains invalid | Supervision tests |
| Explicit new generation | Only fresh children may be admitted; stale peers still denied | Restart tests |
| Normal executable | Still exit 78, no listener or filesystem setup | Exact built binary through private TMUX |

Unit/fake-handler tests prove routing and lifetime only. Root-owned peer
authentication, actual namespace CAS, fixed-system listener and separate-UID
denial are explicitly **unverified** until the later native qualification gate.
Do not expose an alternate production PF20 constructor to make unprivileged
tests pass. A test failure due solely to lack of root is not a native pass.

RTX builds use the existing shared build lock, eight jobs and short on-disk
TMPDIR. Run fix/format first, strict Clippy in both service configurations, all
new tests and affected service/broker integration suites, then private-TMUX
subprocess lifecycle checks with command text and Enter sent separately. Record
actual executed counts, source and binary digests. Do not rerun the already
completed service-stage review or unrelated PF13 qualification.

No user-facing UI changes: fresh blind UX design and live application repository
workflows are not applicable to this internal component. These subprocess/TMUX
checks do not replace PF26's later final-candidate TUI and live-repository gates.
No human-ready package, benchmark improvement or all-platform result is claimed.

## Review and deployment gates

The six historical broker reviews remain spent. Travis granted **five additional
slots, replenishing to five every six hours**; see [the budget ledger](review-budget.md)
for the exact anchor and accounting rule. The pending two-pass request is resolved.
Plan review 7 (Astra High code review) and review 8 (Fable 5.1 High final external
evidence/code review through Corbanu/TMUX); do not consume the other slots without
need. No new reviewer has started. Preserve lifetime review history at resets.

No privileged installation is requested now. A later native stage must present
an exact manifest with frozen source/binary hashes, executable argv/FD schema,
validated account/UID/group names, exact directory/socket modes and ownership,
complete unit and capabilities, separate one-time synthetic enrollment, bounded
fault probes, stop procedure and preservation/cleanup disposition. The v2
proposal still has unresolved ExecStart and read-only `/run` details and is not
an executable manifest. Never fill those gaps by deploying today's inert binary.

Until separately approved: no accounts, services, fixed root listener,
ownership/ACL changes, TPM changes, real Vault migration or protected activation.
PF27 remains in progress, PF35 external/draft and PF30 frozen/draft. Coordinate
shared registrations and any main landing with the accounting/Task Node manager.
