# PF27 next allocation — inspect the root manifest and executable

**Accepted for bounded implementation September12 by integration owner task
`01a08522-76a1-7ad1-afe6-ad690d55c0d7`. No privileged execution authorized.**
Preparation follows accepted main `cc931adf6333e8348e03ee5fc801c77548aec138`.
Product initiative PF-27-S04, sole owner `/root`, worktree
`/Volumes/CorbanuDrive/Corbanu/worktrees/security-broker-resume-20260911`, branch
`feat/security-broker-resume-20260911`, original allocation base unchanged:
`d870c92dab2bf3fbb602dc3b8447fe9f3534aecb`.
Product heading **Non-negotiable controls**: “Permit agents to reference
credentials only by label; resolve them solely inside the trusted execution boundary.”

## One bounded deliverable

Inspect the fixed root-owned synthetic launch manifest and the executable it
identifies, through checked open handles. Produce an internal inspection result,
not a spawn operation or protected-mode authorization. Retain default service
exit78 and native_eligible:false. Do not revisit the accepted launcher recipe.

Proposed literal source scope within the existing sprint:

- `codex-rs/secret-broker-service/src/launch/manifest/` for schema, opened-file
  verification and separate tests;
- `src/launch/mod.rs` and `src/lib.rs` in that crate for minimal gated wiring;
- `tests/manifest_lifecycle.rs` for the non-root factory-denial control;
- that crate's Cargo/Bazel files, plus serialized existing workspace dependency
  feature/lock parity; no Core, PF20, Vault or normal-startup changes;
- this QA packet, active security plan, PF27 sprint and review ledger.

Target under500 changed source/test lines; hard800, modules below500. If the
coherent implementation exceeds that boundary, split it before implementation.
Keep source and mechanical qualification evidence in separate commits.

## Frozen candidate contract

1. Read only `/etc/corbanu-protected-test/launch.json`. Public factory first
   requires actual root real/effective/saved UID and GID, before filesystem I/O.
   No public alternate root, expected-owner override or arbitrary manifest path.
2. Strict JSON schema1, at most8192 bytes: required `schema_version` integer1,
   `purpose` equal to `synthetic-identity-preparation`, `source_commit` exactly
   40 lowercase hexadecimal characters, `probe_sha256` exactly64 lowercase hex,
   `principals` containing exactly `journal`, `policy`, `worker` objects with
   numeric `uid` and `gid`, and numeric `anchor_gid`. Reject unknown, duplicate,
   missing, trailing, wrong-type and out-of-range input at every object level.
3. Reuse the complete role-table validation from `SyntheticLaunchRecipe`:
   distinct nonzero/non-sentinel UIDs and primary GIDs, separate anchor GID,
   journal/policy receive only that supplementary group, worker none. Never
   deserialize arbitrary argv, environment, executable name, socket or namespace.
4. Derive the sole image path:
   `/opt/corbanu-protected-test/<source_commit>/codex-protected-root-probe`.
   Keep host-local checked path types; no worker path or URI as authority.
   The root manifest declares commit-to-hash association; this is not independent
   provenance verification that a binary was built from that commit.
5. Open `/` once, check it, then walk each fixed component relative to retained
   directory handles using safe rustix openat2, BENEATH/NO_SYMLINKS/NO_MAGICLINKS,
   CLOEXEC and appropriate directory/read-only/nonblocking flags. Check every
   opened ancestor: directory, root UID/GID, no group/other write or special
   mode bits. Exact managed modes: manifest parent0700, deployment parent and
   commit directory0755. Do not normalize permissions or create missing paths.
   Unsupported syscall/flags deny, with no weaker path-open fallback.
6. Manifest must be a regular single-link root:root0600 file. Image must be a
   regular single-link root:root0755 file, no set-ID bits, nonempty and no larger
   than512MiB. Nonblocking opens must not hang on a substituted FIFO. Reject
   symlinks, devices, sockets, directories, hardlinks and ownership/mode drift.
7. Stream SHA-256 over the same held image handle, bounded by512MiB plus a
   one-byte overflow probe; do not allocate the whole image. Check ELF magic,
   ELF64 little-endian x86-64 identity for this RTX-only stage. Reject scripts,
   wrong architecture and digest mismatch. Compare handle metadata before and
   after reads (identity, mode, owner, links, size, mtime and ctime), including
   manifest reads. Changed input denies. Errors expose categories, not content.
8. Keep the verified image handle and metadata private in a non-Clone inspection
   result. Do not export a `Command`, raw FD, unchecked path-to-spawn API or
   authoritative “ready” boolean. Inspection alone does not bind a later exec:
   the separately allocated spawn stage must consume/revalidate this binding.
   No claim of ELF loader/shared-library closure verification in this stage;
   that and actual exec binding remain explicit privileged-launch prerequisites.

Root administrator and kernel remain trusted under the accepted data-rollback
scope. No assertion that metadata checks defeat malicious root or entire-machine
rollback. Retaining a descriptor does not by itself make file bytes immutable.
No accounts, chmod/chown, protected directories, enrollment, sockets, exec,
service units, privileged test invocation or installed application changes.

## Implementation feasibility checked without privileged operations

Pinned RTX rustix1.1.4 `src/fs/openat2.rs` exposes safe
`openat2(AsFd, path, OFlags, Mode, ResolveFlags) -> Result<OwnedFd>`.
Its `backend/linux_raw/fs/types.rs` has the named resolution flags. Add the
existing `fs` feature rather than copying PF20's private unsafe syscall wrapper;
preserve forbid(unsafe_code). Existing workspace serde/serde_json/sha2 supply
strict decoding and streaming hashes; no new package is expected.

Read-only stat on the accepted RTX probe returned309802824 bytes on September12,
so the proposed512MiB ceiling covers the actual debug artifact while bounding
I/O. It is a construction ceiling, not a release size or performance claim.
Hashing deadlines on stalled storage are not established by a byte ceiling.
Actual supported local filesystem and launch timeouts remain installation gates.

## Frozen negative cases and proof limits

- Table-driven schema: duplicates at every nesting level, omitted/unknown fields,
  unsupported version/purpose, malformed commit/hash, string/float/negative IDs,
  zero/sentinel/colliding principals, overflow and trailing data.
- Private temporary-tree seam: valid synthetic image and manifest; corruption,
  digest/length mismatch, truncated or wrong ELF identity, over-size input,
  metadata mutation and every open/stat/read failure stop the operation.
- For each managed/ancestor position: symlink, non-directory ancestor, unsafe
  mode/owner. For leaf files: multiple links, FIFO and other invalid type.
  Device/foreign-owner cases may use private injected metadata; do not create
  devices or change ownership merely to obtain a green test.
- Actual non-root public-factory invocation denies before fixed-path access;
  no normal startup or native mode becomes active. Assert no directory creation,
  no parent identity/group mutation and bounded sanitized output.
- Include positive controls so an invalid fixture is not mistaken for a denial
  success. Same-user temporary-tree tests prove construction, not root-owned
  installation/containment. Private test seams must not become public authority.

After acceptance: serialized RTX fix/format, focused and full affected suites,
scoped strict lint, Cargo/Bazel parity, exact tree/hash records and actual-key
TMUX execution of denial controls/tests. Keep inherited transitive telemetry
lint debt separate. Code-blind product UX and live-repository qualification are
not applicable to this uncalled internal Linux inspection API; later user-facing
integration still needs them. No new binary/CLI mode is required for inspection.

## Reviews and receiving decision

History1–15 remains unchanged. One unused contingency from the prior +3 extension
is preserved, not consumed by preparing this document. Required new-stage code
and final-evidence closeout would use Astra High and Fable5.1High via Corbanu/TMUX.
Request integration-owner acceptance of this contract and a scoped two-pass
allowance (or the scheduled replenishment) before those dispatches; do not
silently turn the single contingency into two passes or reset the ledger.
No automatic design panel or repeat review of accepted unchanged source.

The receiving owner accepted this exact implementation boundary and added two
necessary scoped passes, preserving the old contingency and history. No budget
wait is required. Main writes still require a fresh serialized window.
After this inspection stage: actual spawn/retained-Child cleanup, exec binding,
loader trust, fixed listener/client and synthetic native qualification remain
separate allocations. The final exact install manifest still needs explicit
approval of binaries/digests/argv/FDs/unit/UIDs/GIDs/enrollment/stop actions.
