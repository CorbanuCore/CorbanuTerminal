# PF27 post-exec probe — tested, review pending

Product initiative PF-27-S04, existing `/root` allocation.
Product heading **Non-negotiable controls**: “Permit agents to reference
credentials only by label; resolve them solely inside the trusted execution boundary.”
Frozen scope/cases: [probe allocation](../probe-next-20260912.md).
Review baseline **`a666c6cd9cd0dc0821d49cafc4ed8fd63d1c0603`**, not an empty
local diff after committing this checkpoint. No review has been dispatched for
this stage; prior stage reviews do not qualify it. No main landing yet.

## Scope for review

Optional synthetic executable only: service Cargo registration, existing rustix
`thread` feature, `src/probe/{main,hardening,hardening_tests}.rs` and
`tests/probe_lifecycle.rs`. Runtime source198 lines, tests149, manifest7 changed
lines:354 total. Default service, Core, PF20, Vault and provider code unchanged.
No new dependency packages; regenerated Cargo/Bazel locks remain unchanged.

Only `--inspect-post-exec` operates, on its own non-root single-threaded process.
It verifies UID/GID triplets, applies/rechecks no-new-privileges, empty capability
sets, disabled keepcaps and nondumpable state. It reports only flags/group count,
descriptor allowlist and constant `native_eligible:false`. Unknown, combined,
missing and native mode arguments return78 without native activity. Group and
FD observations do not imply those properties were repaired or isolated. No
listener, root-store read, enrollment, identity/group change or native CAS occurs.

Review the actual syscall/error boundary, saved-ID checks, single-threaded
assumption, descriptor enumeration lifetime/bounds, no sensitive output, denied
native modes and test adequacy. Do not broaden this into the actual launcher or
weaken `forbid(unsafe_code)`. Private unit tests do not harden their own test
runner; kernel changes occur only in separately executed probe processes.
No nested review or new design panel. TUI/blind UX design is N/A for this
uncalled synthetic helper, not waived for later product integration.

## First-run finding and repair

Initial service fixture run:22/23, clean-launch case failed because inspection
reported `descriptor_allowlist:false`. The qualification shell opens build-lock
FD9 without close-on-exec; direct reproduction under that same `exec9`/`flock9`
setup verified `os.get_inheritable(9)` is True. The probe was correctly observing
an inherited descriptor, not failing to harden its process.

Corrected the clean-start fixture to use Python's explicit `close_fds=True`
before executing the probe. The separate case deliberately passes a temporary
synthetic FD with `pass_fds` and requires allowlist false without bytes/path
disclosure. Neither production FD logic nor its criteria were weakened.
Added oversized input to the pure ambient-state parser's existing malformed
observation cases and enforced that parser's bound directly as well as at read.
Preserve [the initial failed run](initial/fixture.log); final proof is separate.

## Final-source test evidence

Local/RTX Rust tree **`0934e62ae687f680746476c9374d35092a68ab83`**.
Source checkpoint **`deb7ed06e`**, tested but independent review pending.
Fresh remote worktree `/home/travis/worktrees/security-broker-probe-20260912`
based on `a666c6cd9`; previous stage worktrees untouched. Commands in
`qualify-rtx.sh` use the shared build lock, eight jobs and on-disk TMPDIR.

- Fix/format passed before final runs.
- Strict changed-crate Clippy default/fixture `--no-deps` passed.
  Inherited full transitive telemetry lint failure remains open in the
  [adapter evidence](../root-composition-20260912/initial/clippy-default.log);
  no full transitive lint pass is claimed or repeated unnecessarily here.
- Cargo/Bazel parity passed; both lock files unchanged after regeneration.
- Default service3/3; synthetic service/probe23/23; affected356 passed with
  two pre-existing PF20 subprocess-entrypoint skips, none newly disabled.
- Build and actual-key private TMUX inspection/denial plus23/23 passed.
  TMUX captures join wrapped lines for JSON validation. Inspection reports
  native eligibility false; native mode and default service return78.

Raw output remains on RTX under
`/home/travis/security-round5/evidence/pf27-probe-20260912/verified-rerun`.
Published text strips trailing spaces/blank rows only; `.log` evidence is explicitly
tracked before review so it is included in the branch bundle. No real credentials,
environment values or descriptor targets are in these synthetic artifacts.

Candidate `codex-protected-root-probe` SHA256
**`5be9dcc53582a6f6c528f898361b51f646809223a1a9bcd6be14f7aa78aee832`**,
under that RTX evidence directory's `candidate/`. Other binary hashes are in
`rtx/binaries.sha256`. These are test binaries, not installed application updates.

## Review / native gates

September12 08:43Z amendment: manager relayed Travis's integrator discretion and
granted +2 scoped reviews (Astra12, required Fable13) now, without waiting for
the scheduled reset. See ../review-budget.md; historical usage is preserved.
This supersedes only the review wait in the original checkpoint below.

Five slots were already spent in the window ending2026-09-12T11:25:45Z.
Do not dispatch another review early or repeat finished reviews6–11. At the
actual reset, compute the current allowance, reserve review12, then use Astra
High against this committed branch diff. Use required Fable5.1High via
Corbanu/privateTMUX for the external closeout within the same allowance.
Inspect actual findings; preserve failures and any scope/repair dispositions.

This checkpoint is **not independently reviewed or qualified for main yet**.
No human-test-ready build, benchmark or native containment claim. Actual pinned
launcher, group/FD contract, root manifest/listener, separate-principal probes,
enrollment, positive root CAS, real Vault migration and protected activation
remain open and require their own bounded implementation/approval as recorded.
