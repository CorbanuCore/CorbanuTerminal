# PF-27-S03 Opus review response

Independent review ran read-only through TMUX, Corbanu Terminal, and Claude Opus
5.0 Max. The first review transcript is stored outside the repository at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-foundation-platform/tmux-artifacts/pf27-opus-max-review-1/pane.txt`
with SHA-256
`90f1b82fbadf717e37c11087610a287ccb3f9ff458fe0ab801e76f5f997c63c0`.

## Accepted findings and repairs

1. **P1 — Windows process probe could terminate its controller.** The worker now
   uses a query-only `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` handle on
   Windows and retains signal-zero only on POSIX.
2. **P1 — Linux process-memory probe could classify an unmapped-address `EIO` as
   denial.** The controller now supplies a known-mapped synthetic canary address;
   only access-denial/process-gone errors count as denial, while other errors are
   untested probe failures.
3. **P2 — Rust eligibility helper accepted an incomplete capability slice.** It
   is replaced by a full activation-envelope validator that checks versions,
   probe and target identity, freshness/lifetime, duplicates/missing results,
   every capability status, and the eligibility claim. Seven standalone Rust
   regressions pin the fail-closed behavior.
4. **P2 — CLI validation did not bind a report to the intended target.** Strict
   `--validate` now binds both current machine/boot identity and the exact probe
   SHA. Explicit `--validate-evidence` validates archived evidence without
   granting current-target authority or requiring archival freshness.

After each repair round, the exact SHA-bound probe was rerun on macOS and Linux,
validated strictly on its own target, and correctly remained ineligible.

The second independent review transcript is stored at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-foundation-platform/tmux-artifacts/pf27-opus-max-review-2/pane.txt`
with SHA-256
`1a1fcf754787133deb5fc15bf7ecfe6efbca94818c07a2de3082f9e3a049d531`.
It verified the original four repairs and found seven additional edge cases.
All were accepted and repaired: missing `sudo` is untested, status/observation
pairs are constrained in schema/Python/Rust, `ESRCH` is a probe error, Rust
rejects malformed identities, Windows uses the OS-reported boot timestamp,
`codesign` timeout is bounded, inherited descriptors must match the canary, and
archival validation waives only staleness while retaining the real-clock future
check. The exact repaired probe was rerun on both executed platforms afterward.

The third independent review transcript is stored at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-foundation-platform/tmux-artifacts/pf27-opus-max-review-3/pane.txt`
with SHA-256
`3abe0a644df9cb576035367f7c1caf871818be58f585c5bd698586f29b84e0b3`.
Its four implementation findings were repaired by making Windows elevation
explicitly untested without a real noninteractive attempt, classifying only
specific Mach protection failures as denial, binding network acceptance to a
random worker canary while making true denial reachable, and separating IPC
worker timeout/error from peer mismatch. Its eligibility-design concern was
resolved without weakening the required signing/entitlement gate: an explicit
all-supported eligible regression now exercises the positive path, while the
generic fixture remains intentionally non-qualifying until a real enforced
runtime predicate exists. Both executed targets then passed 8/8 contract
regressions and emitted/validated 10/10 final capability records.

The fourth independent review transcript is stored at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-foundation-platform/tmux-artifacts/pf27-opus-max-review-4/pane.txt`
with SHA-256
`0c3f5d35b6687c13d84957e57d4a89f0074daed808cedb6dddb0581de0a57d33`.
All six findings were accepted: IPC now binds the OS peer to the actual spawned
PID, UID, and random controller canary; Rust directly tests well-formed identity
mismatch and the missing-capability branch; malformed peer input becomes a
stable untested result; handle closure requires a successful inheritance
control; macOS system helpers are absolute; and the JSON Schema requires every
distinct capability. Exact-SHA macOS/Linux evidence was regenerated afterward.

The fifth independent review transcript is stored at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-foundation-platform/tmux-artifacts/pf27-opus-max-review-5/pane.txt`
with SHA-256
`d13cd2a2d807be5d58a416d33f57c6725722219ca0063731bda0ec95c3e1c338`.
All three findings were accepted: malformed JSON value types now use the stable
fail-closed error path without a traceback; unknown operating systems cannot
produce a reboot-stable target identity; and the extensionless command is now a
small stable shim over a conventionally named implementation with discovered
regression tests. Standard Ruff discovery covers both new Python files, and the
exact repaired implementation was rerun on macOS and Linux afterward.

The sixth independent review transcript is stored at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-foundation-platform/tmux-artifacts/pf27-opus-max-review-6/pane.txt`
with SHA-256
`5a2f94e51aac9a20081b45cd21de0a05d2c9d495312aa78b9fc8476475146f90`.
Its three findings were accepted: `--require-eligible` now returns the documented denial
status for both strict and archival validation of ineligible reports;
`struct.error` is contained by the IPC stable-error path; and Python target
metadata validation now matches the JSON Schema type and length constraints.
Five discovered Python tests pin these repairs, and exact-SHA macOS/Linux
evidence was regenerated again.

## Final review

The seventh independent review inspected the complete final candidate through
TMUX + Corbanu Terminal + Claude Opus 5.0 Max and reported **NO FINDINGS**. Its
read-only transcript is stored at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-foundation-platform/tmux-artifacts/pf27-opus-max-review-7/pane.txt`
with SHA-256
`e4ed8ba00cec04510f021b227e8398536ebf68cfe947b31876df1bee91bdd5ae`.
It recomputed all ten then-published hashes, independently checked both result files
against every schema constraint, verified all review-six repairs and earlier
fail-closed classes, and confirmed the exact declared scope had no runtime
consumer before the serialized G1 registration.

## Windows completion qualification

After the authorized Windows endpoint returned to the connected Tailscale
tailnet, the exact reviewed implementation and shim ran directly on Windows 11
with Python 3.13.15. The target passed 8/8 contract regressions, generated and
strictly validated 10/10 capability records, remained correctly ineligible, and
returned exit 2 under `--require-eligible`. The retrieved result then passed
local archival validation and was byte-compared with the compact repository
artifact. The remote synthetic fixture directory was removed. A final
three-platform review is required because this evidence postdates review seven.

The eighth independent review transcript is stored at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-foundation-platform/tmux-artifacts/pf27-opus-max-review-8/pane.txt`
with SHA-256
`ab3845a65da3094d98a62111d81e023a1ae2ebff8c9975f3e9d1f2744e67e376`.
All four findings were accepted. The frozen mechanism document now records the
measured Windows blockers; the probe detects an elevated Windows token and
classifies it as unsupported; every result is reproduced with a documented
`jq -c` compaction step and exact `cmp`; and the matrix contains only metadata
bound into each result artifact. The source repair changed the probe identity,
so macOS, Linux, and Windows evidence was regenerated and revalidated from the
same implementation. A final post-repair review remains pending.

The ninth independent review transcript is stored at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-foundation-platform/tmux-artifacts/pf27-opus-max-review-9/pane.txt`
with SHA-256
`65b8bc77f1db521045bad4b75d640cd7f2b5563e541c921716910182bff0fbbb`.
Both low findings were accepted. The mechanism record now names the measured
`PROCESS_VM_READ` bypass without conflating it with the query-only identity
probe, and every expected ctypes failure class maps to the stable
`windows_token_probe_error` code with regression coverage. The probe identity
changed, so exact compact macOS/Linux/Windows evidence was regenerated again.
A clean follow-up review remains pending.

The tenth independent review inspected the complete regenerated final tree and
reported **NO FINDINGS**. Its read-only transcript is stored at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-foundation-platform/tmux-artifacts/pf27-opus-max-review-10/pane.txt`
with SHA-256
`8ef76718e93b3b62003eb1285abd1990c56c24160de00035a073a2956844ab60`.
It verified all eleven pre-G1 hashes, exact `jq -c` reproduction for all three results,
six cross-target denials, every envelope/count/eligibility claim, both
review-nine repairs, stable Windows token semantics, and the absence of a
runtime consumer before the serialized G1 registration.

## G1 integration review

The eleventh independent review inspected the merged and registered G1 tree
through TMUX + Corbanu Terminal + Claude Opus 5.0 Max. Its read-only transcript
is stored outside the repository at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-foundation-platform/tmux-artifacts/pf27-opus-max-g1-review/pane.txt`
with SHA-256
`4be9db8c33e5e563ee38e7e734623b926cefd513cf64ff5271fa16e90f0ab20c`.
It accepted the registration parity, all published fixture hashes, platform
counts, absence of a runtime consumer, and merge integrity, then reported ten
integration findings. All ten were accepted and repaired:

1. PF-27-S03 state and the 58-current/18-archive ledgers are consistent.
2. The mainline Codex ingress/classifier integration owner is preserved and the
   post-PF-34 transfer is recorded explicitly.
3. The foundation/platform lane and literal G1 scope are present in the owner
   table and plan narrative.
4. Rust tests consume the exact frozen schema and all three result fixtures
   through Cargo and Bazel compile-data declarations.
5. Every remaining Rust rejection variant and malformed digest class has direct
   regression coverage.
6. A three-operating-system CI workflow runs Python contract tests, archival
   fixture validation, and the registered Rust suite.
7. Successful activation returns an opaque, unforgeable authorization witness
   rather than trusting a self-asserted boolean.
8. The pre-existing sprint-count narrative is reconciled.
9. Release-evidence text distinguishes the registered broker/ingress crates from
   the still-planned retriever/search-extension crates.
10. Linux IPC uses a short abstract socket address, with a regression proving a
    caller's long temporary path cannot degrade the result to a generic error.

Because the last repair changed the probe identity, the exact implementation
again passed 7/7 discovered tests and 8/8 self-tests on all three targets,
generated and strictly validated 10/10 capability records on each target, and
remained correctly ineligible. The compact results pass local archival
validation and are bound into the 16/16 Rust gate suite.

The twelfth independent follow-up reviewed exact commit `9f12e5f7e` through the
same TMUX + Corbanu Terminal + Claude Opus 5.0 Max session. Its transcript is
stored outside the repository at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-foundation-platform/tmux-artifacts/pf27-opus-max-g1-review-12/pane.txt`
with SHA-256
`d172187468c26789d046b2e1d3a90d617c6fcaf09fac4decb0192f188834bb96`.
It verified the ten prior repairs and all nineteen published hashes, then found
four follow-up defects. All four were accepted: CI now uses the actual sprint
result paths, the final 59-record reference is 58, the cross-platform IPC test
asserts exact OS-specific outcomes, and Linux uses an atomic mode-0700 directory
under a fixed short prefix rather than an enumerable abstract socket. The source
identity changed, so macOS, Linux, and Windows evidence was regenerated and
strictly revalidated once more.
