# PF27 proposal — one owned child launched from the inspected sealed image

**Proposal only; implementation and invocation not yet allocated.** Parser
checkpoint `406aa3c5f58af0f3422558545dc821a64f777e6b` is verified on main;
its window is released. Manager accepted Fable22's corrected evidence gap,
preserving the original exit1/P3. Runtime tree after test repair:
`2edd1107d9590195956a30cf2b6a84c6cfb7c0f0`.

PF-27-S04 remains `in_progress`, sole owner `/root`, worktree
`/Volumes/CorbanuDrive/Corbanu/worktrees/security-broker-resume-20260911`, branch
`feat/security-broker-resume-20260911`, original base
`d870c92dab2bf3fbb602dc3b8447fe9f3534aecb`; incremental baseline406aa3c5f.
Product heading **Non-negotiable controls**: “Permit agents to reference
credentials only by label; resolve them solely inside the trusted execution
boundary.” Existing PF27 initiative, no new product authority or release claim.

## Next useful construction increment

Connect one profile-inspected image to an owned child-launch result, without
exposing arbitrary commands or activating a service. The previous increments
inspect and hold bytes but never bind an actual process to them. This stage
must prove that binding and child ownership together, not merely add a public
Command factory. Keep the full broker pair, admission/listener wiring and
privileged service installation out of this allocation.

Proposed literal source boundary, all beneath
`codex-rs/secret-broker-service/src/launch/`:
`manifest/sealed.rs`, `manifest/mod.rs`, new `manifest/spawn.rs`, new
`manifest/spawn_tests.rs`, and `mod.rs` for a private recipe construction seam.
QA under `qa/security-levels/sprints/PF-27-S04/descriptor-launch-20260912/`,
this proposal and existing plan/sprint/review ledgers. No lib export, probe-mode,
Core/Vault/PF20/children.rs, graph/lock/default or dashboard change without an
exact new seam decision. Target under500 runtime and800 total nonmechanical.

## Engineering decision to agree before coding

Preserve `forbid(unsafe_code)` and an actual retained `std::process::Child`.
Recommended Linux-specific route to investigate within this allocation:
`Command` uses the held memfd through `/proc/self/fd/<owned-fd>`, keeping the
descriptor alive through spawn and CLOEXEC at successful exec. This is a
procfs-assisted descriptor binding, **not** an execveat syscall claim. Do not
reopen the original deployment path, clear CLOEXEC, use a shell, or expose that
path/descriptor/Command to callers. Verify the pinned Rust spawn behavior and
trusted procfs assumptions before implementation; ambiguity returns to the
manager, not a fallback to the original path or unsafe pre_exec callback.

An alternative is a separately reviewed descriptor-exec adapter with a different
process-ownership abstraction. That changes the source/API boundary and is not
silently included. The manager should accept or amend the recommended route.

## Proposed ownership and proof contract

- Consume the opaque profile-inspected image; use only its validated role recipe,
  fixed non-secret argv, empty environment, `/` cwd and null stdio. No caller
  supplied executable, environment, arbitrary FD map or argument vector.
- Validate prerequisites before spawning. Preserve actual-root checks for the
  future privileged path. A private test seam supplies test-owned images only;
  it is not a public alternate authority constructor.
- On success retain the actual Child and stable pidfd; do not publish a PID as
  ownership. Define bounded polling/stop and cleanup ownership before returning
  an object. Any failure after spawn must retain/reap the child, not detach it.
  Reuse the existing cleanup principles; no new thread allocation from Drop.
- Agree launch deadline handling, reaper startup failure and ownership transfer
  explicitly; a timed-out caller is not proof the child never launched. If this
  cannot fit the proposed coherent boundary, report measured scope before coding.
- Freeze tests for successful binding, source-path replacement, sanitized spawn
  failure, missing procfs, invalid/lost descriptor, early child exit, pidfd failure,
  timeout/late spawn completion, cancellation/drop, and exact argv/env/stdio.
  Prove cleanup without PID reuse or secret-bearing output. Do not drop cases
  merely to fit the line target.

## Explicit invocation decision requested

Unlike the completed parser, meaningful proof now requires executing a test
artifact. Request **non-root, isolated synthetic invocation only**, using the
recorded static probe and existing modes after inspecting their exact behavior;
record full hash, argv, profile, environment and expected denial/observation
before running. No new probe mode, root-positive run or installed executable.
If long-lived positive cleanup requires another fixture, return its exact
source/command proposal first. Current permission remains no invocation.

After acceptance: RTX serialized fix/format, focused/default/synthetic/affected
checks, actual-key TMUX evidence, and separately allocated necessary reviews.
No reviews are started or reserved by this proposal. This internal stage has no
product UX or human-readiness claim. Native/all-OS containment, runtime dependency
or CVE suitability, aggregate supervisor budgets, full identity/channel mapping
and privileged installation remain open even if a non-root launch works.
