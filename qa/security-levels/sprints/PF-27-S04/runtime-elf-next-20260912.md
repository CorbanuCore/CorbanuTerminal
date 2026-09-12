# PF27 proposal — inspect the sealed image's static ELF profile

**Proposed, not allocated.** Receiving owner requested this proposal after
accepting main checkpoint `97d4cb6e8bcaa00a7d1b4ea237cc823df6ffdbaa`.
The main window is released. No implementation, review, image execution or
installation is authorized by this document.

## Coordinates and purpose

- Plan: `docs/plans/active/p0-security-levels.md`, feature PF-27.
- Sprint: PF-27-S04, `in_progress`; sole security owner `/root`.
- Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/security-broker-resume-20260911`.
- Branch: `feat/security-broker-resume-20260911`.
- Original allocation base: `d870c92dab2bf3fbb602dc3b8447fe9f3534aecb` (unchanged).
- Proposed incremental source baseline: `97d4cb6e8bcaa00a7d1b4ea237cc823df6ffdbaa`,
  Rust subtree `0aa65bd04f5e30f3a21e1309aac7aa6b83336859`.
- Receiving task: `01a08522-76a1-7ad1-afe6-ad690d55c0d7`; its reported combined
  checkpoint `c5c75ef7444d7c6233f992054e04d856af2febca` remains manager-owned.

Product heading **Non-negotiable controls**: “Permit agents to reference
credentials only by label; resolve them solely inside the trusted execution
boundary.” This is a proposed construction increment inside the existing product
initiative, not a new feature or release. It makes the sealed-byte format check
part of the internal preparation API instead of relying on a build-time readelf
receipt. Acceptance must not imply permission to run those bytes.

## Smallest source allocation requested

Only `codex-rs/secret-broker-service/src/launch/manifest/`:
new `elf.rs` and `elf_tests.rs`; narrow wiring in `mod.rs` and `sealed.rs`, and
colocated sealed-image tests where necessary. Add QA under
`qa/security-levels/sprints/PF-27-S04/runtime-elf-20260912/` and update the existing
plan/sprint/evidence ledgers after allocation. No Cargo/Bazel/dependency/lock,
probe implementation, Core, Vault, PF20, service configuration or dashboard edits.

Recommendation: a consuming method on `SyntheticSealedImage` performs bounded
read-only parsing of its private sealed descriptor and returns a distinct opaque,
non-Clone inspection result retaining that same image and recipe. No public
constructor, descriptor/path getter, command builder or spawn method. Keep
`seal()`'s existing immutable-byte contract; do not relabel all sealed files as
ELF-approved. Do not reopen a pathname or invoke readelf from the runtime API.

## Proposed deliberately narrow contract

1. Read from the already sealed, digest-bound image; verify required seals and
   retain the existing 512 MiB bound. Use fixed-size reads and checked arithmetic,
   not a second image-sized allocation or unchecked on-disk counts.
2. Accept only ELF64 little-endian, current ELF version, x86-64, ET_DYN static
   PIE. Reject other profiles rather than introducing a generic ELF loader.
3. Validate ELF/program-header sizes, count and complete in-file ranges. Proposed
   limits: at most 128 program headers and 4096 dynamic entries; reject extended
   numbering and overflow/truncation. Validate load-segment file/memory bounds,
   alignment, and entry-point membership in an executable load segment.
4. Reject PT_INTERP. Require one bounded PT_DYNAMIC with complete entries and
   DT_NULL termination, backed by a valid file-backed load range. Require
   DF_1_PIE. PT_DYNAMIC itself is allowed for static-PIE self-relocation.
5. Reject DT_NEEDED and external-loader directives, including DT_FILTER,
   DT_AUXILIARY, DT_AUDIT, DT_DEPAUDIT, DT_RPATH and DT_RUNPATH. Enumerate supported
   dynamic tags for the observed profile; reject unsupported/ambiguous structures
   rather than claiming a general dependency-closure validator.
6. Parsing or I/O failure yields a sanitized denial and no approved result.
   Ownership/drop semantics close the consumed image on failure. No mutation of
   the sealed bytes, identity, environment, host settings or manifest format.

Exact accepted tags/structural rules must be recorded with implementation tests;
an incompatible observed artifact returns for a bounded contract decision rather
than silently weakening the rules. These are engineering recommendations for
agreement, not new product authority or a production artifact-strategy decision.

## Proposed proof matrix

| Case | Required observation |
| --- | --- |
| Existing static candidate `f6ca8e3368dc…` | Accepted by parsing only; hash/size match the full static-stage receipt |
| GNU control `836d74a13c91…` | Denied for interpreter/dependency profile without execution |
| Rejected musl candidate `37fb0d7c5fd7…` | Denied despite no DT_NEEDED because PT_INTERP remains |
| Minimal valid synthetic static PIE | Accepted; PT_DYNAMIC alone is not rejected |
| Wrong magic/class/endian/machine/type/version | Each denied independently |
| Truncated/overflowing/oversized tables or ranges | Denied without panic or unbounded allocation |
| Dynamic termination/count/backing violations | Denied; bytes after a valid terminator are not mistaken for loader entries |
| Each external-loader tag, missing PIE, unsupported profile | Denied independently; retain mutation fixtures |
| Read failure, missing seals, dropped/rejected result | No approval and descriptor cleanup verified |
| Source-path replacement after sealing | Parsing remains bound to the held immutable bytes |

After allocation, build/tests on RTX under the existing serialized build lock;
run format before final focused/default/synthetic and affected-suite evidence.
Use actual-key private TMUX to run the parser tests against the three frozen
artifacts as data. Test-process execution is distinct from invoking the probe:
no probe execution, ldd, privileged command or system installation is proposed.
Preserve original failures and candidate-bound receipts. No reviews are dispatched
by this proposal; agree their allocation before dispatch and preserve the ledger.

## What this cannot prove

This uncalled internal API has no user-facing UX; code-blind UX design and
live-repository benchmarks are not applicable to this increment. No human-test or
release readiness claim. Static ELF inspection cannot establish runtime absence
of dlopen/file access, libc/OpenSSL vulnerability suitability, syscall isolation,
relocation correctness or successful execution. Descriptor-bound exec, retained
Child/pidfd supervision, identities/groups/FD mapping, deadlines/resource budgets,
native/all-OS qualification and exact privileged installation authority remain
separate gates. Do not change their status when this validator passes.
