# Next bounded PF27 increment: asynchronous single-child owner

Proposal for receiving integration owner; not yet allocated or implemented.
The separate adapter source30b471a47/Rust2d270c5c has real non-root OS proof;
Astra23 and Fable24 both completed clean with no findings. Approval
of the isolated adapter is recorded; this record does not reopen that decision.

Keep the original PF-27-S04 owner/worktree/branch/base. Product citation:
**Non-negotiable controls** — “Permit agents to reference credentials only by
label; resolve them solely inside the trusted execution boundary.”

## Smallest next ownership boundary

Add a synthetic-only, one-at-a-time launch owner under service
`src/launch/manifest/{spawn.rs,spawn_tests.rs}`, narrow private conversion of
the already-profile-inspected image in `manifest/{sealed.rs,mod.rs}`, and a
service synthetic-feature dependency on codex-linux-pidfd-spawn. Workspace
Cargo/lock/Bazel edges and PF27 evidence/ledgers remain serialized. New private
QA lifecycle fixture may exercise only this non-root synthetic boundary.
Do not change Core/Vault/PF20/children.rs or native service installation.

Before claiming actual code scope, inspect the existing recipe/image types and
freeze exact API declarations; public PID/FD/Command export is not acceptable.
Existing `TrustedChildRun` requires actual std::process::Child and callback
`&mut Child`; the new pidfd token cannot be fabricated into that type.
Its two-child admission/PF20 root integration therefore remains a subsequent
distinct interface stage, not a hidden extension of this one-child owner.

## Required contract and tests

- Acquire a process-local permit before consuming an image or creating a worker.
  A second launch cannot start while a prior spawn/cleanup remains unresolved.
- Establish the worker and control state before invoking the blocking primitive.
  Worker creation failure returns the still-owned image without child creation.
  No new thread or process may be allocated from caller Drop.
- Worker exclusively owns pending image, eventual pidfd and cleanup. Caller
  deadline/cancel/drop sets sticky cancellation and returns CleanupPending while
  preserving ownership and the permit; no successful-stop inference from timeout.
- A late returned child is terminated/reaped before permit release. Early exit
  is not readiness. Release only after positive observed cleanup, not channel
  disconnect, worker panic, numeric PID lookup or an elapsed grace period.
- Expose bounded polling status, not direct spawn cancellation or a hard cleanup
  deadline. Persistent wait/signal failure retains the owner/quarantine; define
  recovery behavior rather than freeing a permit to make tests green.
- Deterministic private fault seams gate before/after spawn publication, inject
  worker creation and signal/wait failures, and prove cancellation, early Drop,
  worker panic/error, no relaunch, and observed-reap ordering. Pair them with real
  static hold/early-exit pidfd lifecycle tests in the TMUX harness on RTX.

This stage still does not establish process-crash durable supervisor recovery,
cross-host positive qualification, privileged isolation, native admission,
real credential substitution or protected activation. It must retain those
unfinished conditions rather than claim full PF27.

Target ~200–300 runtime plus300–400 tests; split further if actual integration
needs a public protocol change or exceeds the bounded size. Two necessary
new-source closeouts (Astra High/Fable5.1High through Corbanu/TMUX) come from the
remaining scheduled review budget, reserved only after manager allocation and
actual dispatch. No review or implementation is started by this proposal.
