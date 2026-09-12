# PF27 next allocation proposal — seal the inspected executable bytes

**Accepted for bounded implementation September12 by receiving owner task
`01a08522-76a1-7ad1-afe6-ad690d55c0d7`. No executable invocation authorized.**
Follows accepted main `2c7c4e6c0212a428d332eee7fbec34a9bc1e8d39`, source
`099fa6f6dacd7783afbd98f1493b491015bc84af`, Rust tree
`5df6837f11abe2b176c18e1e69a53ef27b595194`. The main window is released.
PF-27-S04 remains the sole security sprint, owner `/root`, branch
`feat/security-broker-resume-20260911`, worktree
`/Volumes/CorbanuDrive/Corbanu/worktrees/security-broker-resume-20260911`, original
allocation base `d870c92dab2bf3fbb602dc3b8447fe9f3534aecb` unchanged.
Product heading **Non-negotiable controls**: “Permit agents to reference
credentials only by label; resolve them solely inside the trusted execution boundary.”

## Proposed decision and why this precedes spawn

Prepare an immutable, digest-checked executable image in a private Linux memfd.
Do not label the current inspection handle immutable or execute a path reopened
after checking it. Prefer this bounded byte-binding stage over combining image
mutation, dynamic loader trust, root spawn and supervision in one change.
This is an explicit proposed design choice, not a newly inferred permission.

The selected RTX probe is dynamically linked: readelf reports PT_INTERP and
NEEDED libgcc_s.so.1, libm.so.6, libc.so.6 and ld-linux-x86-64.so.2. Sealing its
main ELF image cannot establish trust in those files or the loader search rules.
Do not claim a complete dependency list from those direct NEEDED entries.
Loader/dependency resolution and actual descriptor-bound exec remain next gates.

## Exact proposed scope and contract

- Only service `src/launch/manifest/sealed.rs`, separate `sealed_tests.rs`,
  `src/launch/manifest/mod.rs` and minimal `files.rs` helper reuse; narrow export
  wiring if necessary, existing dependency feature parity and PF27 records.
- Consume the inspection result, never accept an arbitrary path or public file
  handle as equivalent authority. Preserve its expected digest and validated
  identity/recipe privately; no public alternative principal/root constructor.
- Rewind the held image, compare its current checked metadata with the recorded
  snapshot, stream at most512MiB plus one overflow byte into a fresh memfd, and
  compare source metadata again. Do not reopen the source path or load all bytes
  into a Vec. Size/error handling stays fail closed and sanitized.
- Create memfd with CLOEXEC, ALLOW_SEALING and explicit EXEC. Never change
  vm.memfd_noexec or use fallback flags to evade host policy. Unsupported or
  refused calls deny. Use a constant nonsensitive name, not a secret/path.
- Before exposing the result, add and verify WRITE, SHRINK, GROW and SEAL seals;
  then rewind and hash the sealed destination itself against the manifest digest,
  checking the same ELF identity and size. The destination, not just bytes read
  from the source, is the object whose integrity must be proven. No writable
  mappings, descriptor export, inheritable FD or post-seal copy remains.
- Successful result owns the sealed descriptor and recipe privately and is
  non-Clone, with no raw-FD/path getter or Command/spawn factory. It is an image
  preparation artifact, not authorization to launch root code. Native eligibility
  remains false; existing normal service and native modes remain exit78.
- Any creation/read/write/seek/seal/hash failure drops the partial object and
  returns no usable result. No disk deployment, system directory or persistent
  recovery state is created. On process exit the owned memfd closes normally.
- This stage adds up to512MiB of memory-backed image storage per preparation;
  avoid parallel image preparations in the future supervisor. Small fixtures
  prove the mechanism; do not allocate512MiB repeatedly in unit tests. A byte
  bound does not establish storage-stall deadlines or an aggregate supervisor
  memory budget; those remain explicit later design gates.

Pinned RTX rustix1.1.4 supplies safe memfd_create, fcntl_add_seals and
fcntl_get_seals, plus all named MemfdFlags/SealFlags under the existing fs
feature. Observed kernel7.0.0-31-generic is availability context, not proof of
successful sealing. Preserve forbid(unsafe_code); no new package expected.

## Frozen proposed construction cases

1. Non-root disposable valid ELF bytes become the exact sealed digest/length;
   kernel-reported seals contain all required bits and descriptor is CLOEXEC.
2. Actual writes, truncation, growth and seal changes fail after successful
   sealing; reads remain correct. Use positive writable pre-seal controls.
3. Source mutations, wrong digest, unsupported ELF, short/oversized data and
   injected partial read/write/seek/seal failure never yield a result. Verify
   post-copy destination hash, including a privately injected corrupted copy.
4. Descriptor ownership is released on every failed boundary and success drop;
   no object survives with partial seals. Private failure seams do not become
   public constructors or authority flags.
5. Existing manifest negative cases, non-root public-factory denial, default78
   and probe limitations remain unchanged. No root-positive run or executable
   invocation is needed to prove this new image-storage mechanism.

After receiving-owner acceptance: target under500 source/test changed lines,
hard800 and modules below500; split before exceeding. RTX serialized fix/format,
focused and affected suites, scoped strict lint/parity and actual-key TMUX
construction controls, exact tree/hash evidence and separate source/QA commits.
No new user-facing feature; no code-blind UX design or human readiness claim.

## Approval, review and remaining gates

The receiving owner accepted the exact sealed-image design and granted two
necessary new-stage Astra High/Fable5.1High Corbanu/TMUX passes; reviews1–17
stay spent, the prior contingency stays unused and the
scheduled six-hour anchor is unchanged. No review is dispatched for this proposal.

Actual exec from the sealed descriptor, dynamic loader closure, subprocess
creation/retained-Child cleanup, bounded launch deadlines, listener/native-client
wiring and all-OS qualification remain subsequent allocations. None is granted
by the proposed MFD_EXEC flag or a successful write-denial test. Final root
installation/execution still needs exact approved binaries, digests, paths,
argv/FD map, identities/groups, unit, enrollment and stop actions. No privileged
run, system ownership change, services, real Vault data or activation here.
