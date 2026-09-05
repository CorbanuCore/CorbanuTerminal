# PF-20-S03 — local controller integrity-root candidate

Product initiative under **Non-negotiable controls**: “Record tamper-evident
policy decisions, tool calls, approvals, signatures, and transaction or order
IDs without secrets.” The **Data-rollback scope decision — 2026-09-04** permits
an independent local controller checkpoint to detect restored agent-accessible
Corbanu data. Whole-machine/controller-store rollback is excluded.

Allocation `989d24c3a`, immutable base `601602fa7e53fcb5b41753a0b3607addd45d4415`,
branch `feat/security-local-anchor`. This is an in-progress candidate, not a
production deployment or release claim.

## Implemented scope and stages

The new `codex-protected-state` leaf depends on existing config/audit/policy and
crypto/OS crates, never Core, Vault or broker. A narrow Core wrapper preserves
the private authoritative-state anchor's exact payload and CAS. Shared module
and dependency registration is coordinator commit `695704d8e`.

The implementation is split into cohesive modules/checkpoints rather than
expanding a central Core file: typed checkpoint/error contract; Linux descriptor
operations; enrollment/durable store; native transport; thin Core adapter;
separate failure/subprocess tests. The coordinator explicitly accepted the
necessary >800-line total including tests. Current recovery checkpoints are
`2c6e27474`, `d9f346cbf`, `740ee0d8a`, `190114002`, `36f7b96cc` and `8939d5041`; they are not individually
claimed as reviewed or final-qualified trees. Final size and exact evidence will
be recorded after formatting. Current source/build delta is 16 files and 2,022
added lines, including 670 test lines. Largest production module is 405 lines.

### Root contract

- Production factory is kernel-root-only and uses fixed `/etc/corbanu-protected-state`
  and `/var/lib/corbanu-protected-state` roots, with `journal` and `policy`
  subdirectories. It validates root-owned non-writable ancestors, no-follow
  descriptor-relative access, owner/private mode, regular files/single links,
  bounded records and selected local filesystem semantics. No arbitrary
  file/path/UID capability constructor is exported.
- `Enrollment` is data chosen by an administrator, not permission. Explicit
  root-only enrollment creates an exclusive persistent fence, new random key
  and installation binding, authenticated Genesis head, then the complete
  registry. Existing/partial enrollment cannot reset or overwrite itself.
  Normal startup opens existing only; missing/corrupt registry/head/key denies.
- HMAC-SHA256 authenticates exact checkpoint bytes, installation, owner/key and
  namespace. Full-value CAS validates successor sequence and nondecreasing
  policy/run generations, holds a lifetime lock inode plus process mutex, syncs
  a temporary file before atomic head replacement and directory sync. No success
  is returned before durable completion. Policy owner/key rotation is not
  implemented; it must receive a separate explicit migration contract.
- Failed reads/writes latch the controller instance unavailable. Published-but-
  uncertain writes and lost receipts are ambiguous, mapped to PF-41's existing
  `IntegrityRootError::Timeout`; definite conflict/invalid errors remain distinct.
  Pending/torn files never fall back to an older authority. Human recovery is
  not replaced by blind retries or automatic reenrollment.

### Native dependency, not a protected launch

`ControllerRoot::serve_child` receives the trusted launcher's actual Child
handle, opens a pidfd while checking the child remains live, and verifies the
kernel-observed connected PID. A parent-created socketpair does not pass for the
post-exec child. A fresh channel key binds monotone request/reply sequences and
directions; error/EOF/death consumes the channel. Frames are bounded before
allocation and use one total I/O deadline, not a resetting per-byte timeout.

The production client connects only to the fixed root-owned system socket and
verifies the kernel root peer; construction is not an arbitrary UID assertion.
The current service API assumes PF-27's trusted launcher selected the actual
executable/worker boundaries. It does not inspect or authorize model-selected
executables, install a listener, or grant arbitrary worker access to the root.
PF-27 must enforce its actual launch registry, process-debug/handle/network
isolation and full platform report. A typed root/client value is narrower than
`ProtectedModeAuthorization` and does not enable protection by itself.

Idle/incomplete frames expire after the bounded native deadline. Clients do not
automatically reconnect or retry a possibly committed operation. PF-27 owns
explicit new-generation bootstrap after closure; no stale key is reusable.

### Consumer semantics retained

PF-41 remains record-first/root-last, with its existing recovery and no-replay
reconciliation. PF-20 policy remains anchor-first/state-files-afterward, with
exact anchored-pending recovery. The Core wrapper copies the complete schema,
revision, owner and both digests; it never creates transition authorization.
The first-record ambiguous PF-41 case remains a visible recovery limitation of
the existing consumer contract, not a hidden new reset path.

## Evidence and explicit limits

Initial RTX proof: 11/11 leaf tests passed, including real post-exec IPC,
cross-process lock contention, and real PF-41 rejection of missing/restored
data against a retained root. A later consolidated leaf run passed 14/14 after
replay/deadline/definite-rejection tests. Two ignored helper tests are invoked by
their actual subprocess parent tests, not omitted coverage.

Fault injection models no-space, partial write, file-sync, post-rename directory-
sync and post-durable receipt loss. These are not disk-filling or physical
power-cut experiments. Same-user native fixtures prove protocol/process behavior,
not genuine worker denial of controller files/memory. Core's ordering fixture
uses an explicitly synthetic platform report, never production readiness.

Consolidated RTX `final-proof.log` passed leaf15, Core17, audit46 and config229;
the final overflow test brings `tmux-proof.log` leaf proof to16/16. Full formatter
and scoped leaf/Core fixes passed. Existing unrelated Core dead-code warnings
remain. The first new Core fixture failed its private-directory precondition,
was corrected to explicit0700, and then passed; this was not a product bypass.
Cargo/Bazel update/check passed with no MODULE delta; Bazel was shut down under
the shared build lock. Initial missing Bazel PATH was corrected to the existing
installation, without installing new tooling.

Remote evidence root: `/home/travis/security-round5/evidence/anchor/`.
Immutable CLI `candidate-8939/codex` SHA256:
`42960f8888ac28fc45bcee805e1be701b0334239dacbc713474821318c547e1f`.
This represents `8939d5041` plus synchronized formatter-only changes.
Actual-key TMUX, integration seam agreement, and Astra/Fable reviews remain
pending. Review budget: new PF20 track0/5; frozen PF30S01 remains5/5 and is not
reviewed again. See `pf27-consumer-handoff.md`; proposed root-anchor composition
must be resolved against the historical nonlogin-controller deployment design
before final freeze. No installation permission is inferred.

No elevated setup, principal/ACL/service modification, production Vault transfer
or system factory invocation has been performed. Selected Linux filesystem
support is ext-family/XFS; unknown, network, tmpfs/overlay and missing native APIs
deny. No macOS/Windows native implementation is claimed. Root's default service
factory requires administrative deployment, so ordinary TUI remains visibly
unverified and Permissive behavior unchanged.

No repository-specific TensorCash/Isometric behavior exists in this leaf.
Their final composed/release qualification and named-human acceptance remain
separate gates, not inferred from supporting `/security`/`/status` TMUX smoke.
