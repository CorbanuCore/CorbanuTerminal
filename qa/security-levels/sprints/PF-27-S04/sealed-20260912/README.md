# PF27 sealed image — construction qualified

Product initiative PF-27-S04, same sole owner/allocation. Accepted scope and
frozen cases: [sealed image contract](../sealed-image-next-20260912.md).
Product heading **Non-negotiable controls**: “Permit agents to reference
credentials only by label; resolve them solely inside the trusted execution boundary.”

Baseline `e66438e33`; formatted/staged Rust tree
`0aa65bd04f5e30f3a21e1309aac7aa6b83336859`. Five source/test files,390 additions:
132 runtime/wiring and258 tests. New production module125 lines. No dependency
or lock changes, no Core/PF20/Vault/normal startup changes. Keep source and QA
commits separate. Source commit `ee1ac023c`; reviews examined the staged source
before that commit, not an empty post-commit diff.

Inspection now privately retains the checked digest. Consuming `.seal()` checks
its held source against the saved metadata, creates a private memfd with
CLOEXEC|ALLOW_SEALING|EXEC, rewinds and copies with a512MiB bound, checks source
metadata again, adds and verifies WRITE|SHRINK|GROW|SEAL, and hashes the sealed
destination using the same ELF identity/size checks. No source-path reopen.
Failure drops both source and destination handles through ownership. The new
non-Clone result exposes no file/path/descriptor getter or command/spawn method.
Only production KernelOps is reachable from the public method; private injected
ops are test-only construction seams, not alternate authority constructors.

This freezes main-image bytes only. It does not establish PT_INTERP/dynamic
dependency trust, descriptor-bound exec, launch/storage-stall deadlines, aggregate
supervisor memory budget, root-positive behavior, installation or native
eligibility. No vm.memfd_noexec/sysctl/permission changes or fallback flags.
No new executable invocation is added; existing native modes still deny78.

## Proof and review status

RTX `/home/travis/worktrees/security-broker-sealed-20260912`; evidence
`/home/travis/security-round5/evidence/pf27-sealed-20260912/verified`.
Fix/format and scoped strict Clippy default/synthetic, Cargo/Bazel parity and
default3, synthetic39, affected356 (2 existing PF20 helper skips) and build have
passed. Actual-key TMUX39 and denial/inspection checks also passed before Fable
dispatch. No compiler or test failure occurred in this stage. Logs are committed
under rtx/, with trailing whitespace/blank EOF lines normalized only; raw output
remains on RTX. Candidate probe SHA256:
`836d74a13c910359ab71abe90523b682211445d355e5c4851164423910d7d555`.
All candidate hashes are in rtx/binaries.sha256. Local and RTX Rust trees match.

New tests: actual non-root kernel seal/CLOEXEC/write/truncate/grow/add-seal denial
with writable pre-seal control; exact readable image bytes; inode-bound owned-FD
closure on success and each of eight failed operation boundaries; incomplete
seals and corrupted destination rejection; stale source, wrong digest,
empty/short/invalid ELF; bounded copy and sanitized partial read/write failure.
No huge repeated allocations, mmap, ptrace, credentials or unrelated processes.
Existing manifest and probe cases remain unchanged. These are Linux construction
tests, not root-installed or all-platform containment evidence.

Actual-key TMUX exercised normal deny78, existing probe inspection/native
deny/preparation nonroot deny, and full synthetic tests. No new product UX is
introduced, so a code-blind UX design and live-repository benchmark do not apply
to this uncalled internal stage. Later product integration still requires them.
Inherited full transitive codex-api telemetry lint remains manager-owned;
scoped strict checks are not its resolution.

Both manager-authorized passes finished with exit0 and no findings:
[Astra High18](astra-eighteen.json) and [Fable5.1High19](fable-nineteen.json)
via Corbanu/private TMUX. Original text, JSON and exit artifacts are retained.
Fable notes kernel support is required for MFD_EXEC; unsupported kernels or
restrictive memfd policy fail closed. No fallback or system-policy change is
authorized. Both reservations are charged; old contingency and all1–17 history
remain intact. No fresh main window is granted.
