# PF27 executable probe — post-exec preparation

Product initiative PF-27-S04; same `/root` allocation, branch and worktree.
Baseline main `1b7e9f3df948b355baf60ba9f713a4ab820a8300` includes the completed
root adapter. Product heading **Non-negotiable controls**: “Permit agents to
reference credentials only by label; resolve them solely inside the trusted execution boundary.”

## Bounded source allocation

Add opt-in synthetic binary `codex-protected-root-probe` with source confined to
`codex-rs/secret-broker-service/src/probe/`, separate sibling test modules and
`tests/probe_lifecycle.rs`. Allocate that crate's Cargo binary registration and
the existing Linux rustix dependency's `thread` feature, plus lock parity only.
No normal main, PF20, Core, Vault or unrelated dependency source edits.
Keep production modules below500 lines and total source/test change below800;
target below500 for this first executable stage. No second new feature/sprint.

First implement **post-exec hardening and a bounded inspection command**, not
the entire native launcher. This source stage must not open the fixed root
socket, read enrollment/root files, change UID/GID/groups, install anything,
perform CAS or start a service. It changes only the dedicated probe process's
own irreversible hardening state. Native launch modes remain unavailable.
This is not user-facing Corbanu TUI behavior; no blind UX design is applicable.

## Verified implementation inputs

Read directly from the RTX pinned toolchain/package source on September12:

- Rust1.95 `library/std/src/os/unix/process.rs`, `CommandExt::uid`: documents
  clearing supplementary groups when no group list is supplied. Its actual
  `sys/process/unix/unix.rs:306–325` ignores **EPERM** from that `setgroups` call.
  Therefore spawn success cannot prove groups were removed; inspect them after
  exec. Explicit `CommandExt::groups` remains unstable in this pinned toolchain.
- nix0.30.1 `src/sys/prctl.rs` supplies safe setters/getters for dumpable,
  keepcaps and no-new-privileges. `src/unistd.rs` supplies safe getresuid/getresgid
  and getgroups. Do not substitute real/effective-only checks for saved IDs.
- rustix1.1.4 `src/thread/libcap.rs` supplies `capabilities` and
  `set_capabilities`; `src/thread/prctl.rs` supplies
  `clear_ambient_capability_set`. Enabling its existing `thread` feature is
  sufficient; no new capability package or unsafe local wrapper is needed.
- Closing arbitrary raw FDs would risk Rust ownership invariants. Instead the
  future single-threaded launcher starts with a verified closed inheritance
  allowlist and creates only close-on-exec descriptors. Post-exec inspection
  rejects unexpected inherited FDs; it does not adopt/close them unsafely.

These references establish available APIs, not native containment measurements.
Read the precise structs/implementations again before using them. Do not modify
the main agent, SSH shell, TMUX server or parent application's hardening state.

Source SHA256 values observed directly on RTX (paths relative to the named
toolchain's `library/std/` or registry package, not repository source copies):

| Source | SHA256 |
| --- | --- |
| Rust1.95 `src/os/unix/process.rs` | `bcabc07a96413b1ec16b44f740b30db2f9e323c06bbe8c1c414ad170651a96ba` |
| Rust1.95 `src/sys/process/unix/unix.rs` | `cabba94153bcdae7674f5743886d518db848710aee0ee20abfecf8bb159d3c4b` |
| nix0.30.1 `src/sys/prctl.rs` | `58162ca18f11269bcda2bd24c5c479bf3b21c28c9b7497785ca4abbb2e0f9bdc` |
| nix0.30.1 `src/unistd.rs` | `26b45c0e0861ca82a9300eb952bc81bf626bbde6d3606fd780ad27fc86360ba7` |
| rustix1.1.4 `src/thread/libcap.rs` | `73f233865e06180aedea39082a5cf36541147d9faad37b7efc3707bf9de67461` |
| rustix1.1.4 `src/thread/prctl.rs` | `ebc63d13874b10d93a1c55868e965638192a1184f0d894502ffad2f1623ca549` |

## Exact initial CLI and result contract

- `--inspect-post-exec`: the only runnable mode in this stage. Require a real
  non-root process with equal real/effective/saved UID and equal GID triplet.
  Apply no-new-privileges, disable keepcaps, clear ambient and effective/permitted/
  inheritable capability sets, then set nondumpable after exec. Re-read all
  affected properties and fail on any unsupported/failed/mismatched result.
- Emit one bounded machine-readable record of booleans/counts only: hardening
  checks, supplementary-group count, descriptor allowlist result, and
  `native_eligible: false`. No environment values, descriptor targets/contents,
  arbitrary command output, credentials, process-memory contents or socket keys.
- Successful inspection exit0 means the named self-hardening checks were
  measured, **not** that groups were changed, inherited descriptors removed,
  the process was launched under an approved identity, or native admission is
  eligible. Groups/FD mismatch must be explicit in the record. Hardening failure
  denies with a bounded generic diagnostic, no partial success record.
- `--journal-child`, `--policy-child`, `--open-existing`, no args, extra args,
  unknown flags and combined modes all return exit78 without native activity.
  Their future behavior is the prior launch contract, not a promise of this build.
- Inspection is synthetic-only and does not call `NativeAnchorClient`. Its stdout
  pipe is an intentional diagnostic exception to the future native children's
  `/dev/null` stdio contract; do not treat that pipe as an authorized root channel.

Descriptor enumeration must bound work, exclude its own temporary enumeration
handle correctly, and avoid treating a disappeared handle as proof that a
different live handle is safe. Do not start background threads before inspection.
No FD contents or target paths may enter the report. Nonzero supplementary groups
are expected for an ordinary unprivileged test runner and remain an explicit
native-readiness failure, never a test excuse to weaken the future allowlist.

## Frozen functional/construction cases

1. Dedicated actual post-exec process reports verified no-new-privileges,
   empty effective/permitted/inheritable/ambient sets and nondumpable state;
   parent process remains unchanged. These are kernel observations, not mocks.
2. Report always says native eligibility false; no command connects to a root
   socket, changes filesystem ownership or changes identities/groups.
3. Deliberately inherited harmless synthetic FD is reported as outside the
   allowlist; its bytes/path never appear. Use a subprocess-owned fixture handle,
   not real credentials or unrelated parent descriptors.
4. Supplementary groups are observed accurately and never silently cleared or
   relabeled as empty. Saved-ID and root rejection have private unit cases plus
   actual non-root execution; do not claim root-run denial was measured without
   running it, and do not run a privileged test for this stage.
5. Native/unknown/combined/no-arg modes remain exit78 and produce no inspection
   success. Default service binary remains unchanged and unavailable.
6. Unsupported syscall/property mismatch and malformed/bounded observation data
   fail closed in private helper tests, with no persistent or parent effects.

Fix/format before new final RTX tests, strict changed-crate Clippy, Cargo/Bazel
parity, build and actual-key private TMUX success/denial checks. Preserve inherited
full transitive lint failure separately rather than modifying shared codex-api.
Record the new exact executable/tree/hash; do not reuse adapter proof as executable
proof. Add tests to the existing stage suites without relabeling native gates.

## Integration, review and remaining native boundary

Manager confirmed no conflicting registration changes and preserved root's
serialized build ownership; this was coordination, not an independent review.
Only scoped branch checkpoints may be pushed without a fresh
coordinated main window. Current review window is spent until11:25:45Z; prepare
source/tests without claiming independent review or landing it beforehand.
Next review number12; follow the existing fixed-window ledger, no early reset.

After this bounded stage: actual pinned launcher, distinct approved UID/GID/group
contract, root-owned manifest, fixed listener and channel bootstrap need their
own exact source amendment. The historical supplementary anchor-group proposal
must be reconciled with stable Rust launch mechanics; do not silently replace
it with world-connectable sockets or shared principals. Final installation needs
explicit approval of real frozen paths/digests/argv/FDs/unit/IDs/enrollment/stop
actions. No privilege, account, ACL, service, real Vault or protected activation
approval is inferred from this document or successful inspection.
