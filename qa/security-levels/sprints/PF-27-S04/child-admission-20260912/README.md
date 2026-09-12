# PF27 child admission — construction evidence

Product initiative PF-27-S04, existing exact allocation; product heading
**Non-negotiable controls**: “Permit agents to reference credentials only by
label; resolve them solely inside the trusted execution boundary.”

## Frozen review scope

Baseline `f2e6d77e4` contains the separately supplied PF35 documentation update;
it is not part of this runtime review. New production code is the 325-line
`secret-broker-service/src/children.rs` after repairs, plus explicit exports and one Linux
rustix dependency/lock edge. The fixture child and five lifecycle tests exercise
real post-exec Unix sockets. No Core, PF20, Vault or broker implementation changes.
No default behavior change: the normal binary still exits 78.

This stage is the service-owned admission/lifetime seam, not a complete launcher.
Expected UID/role and actual Child objects come only from the trusted caller.
The caller must drive `poll_health` in its supervising loop. PF20's future
handler still performs its own independent peer/pidfd check and enforces its
ten-second absolute deadlines. Test callbacks are not installed root controllers.
Neither this library nor the fixtures grant protected-mode eligibility.

Review actual identity/ownership and lifetime behavior, wrong-peer/duplicate
rejection, terminal fencing, handler concurrency, cleanup, failure paths and
test adequacy. Do not redesign the full native service, widen PF20 APIs or add
privileged installation. Distinguish an actual flaw in this seam from the
explicitly unimplemented root/containment/provider wiring. No nested reviewers.

## Execution

Fresh RTX build worktree:
`/home/travis/worktrees/security-broker-child-admission-20260912`, staged from
`6db51b4f5` plus these source edits. Previous RTX resume checkout and evidence
remain unchanged. The later PF35 docs do not change the Rust tree.

[qualify-rtx.sh](qualify-rtx.sh) completed fix/format, strict default and fixture
Clippy, Cargo/Bazel parity, default 1/1, synthetic suite 11/11 and full affected
broker/Vault/proxy suite successfully. Counts/logs and TMUX evidence will be
copied into this record before closeout. Initial development Clippy found
redundant method-call closures; `just fix` repaired them before final tests.
Initial nine-test development run passed before the last two cases were added.

New cases cover role routing independent of arrival, stalled-handler shutdown,
duplicate/parent-socketpair/wrong-UID denial, child death, handler failure,
unknown child, old-generation denial and propagation of a handler deadline.
Fixed two-child capacity prevents dynamic worker registration. No namespace is
selected from channel bytes. Generation numbers are trusted bookkeeping, not
an authority token.

## Open gates

Final Fable review 9 completed cleanly; see closeout below. Native gates remain open.
No human-facing feature was added; blind UX design and live application-repo
workflows are N/A for this source seam, not waived for later PF26 qualification.
No Mac/Windows isolation, native root listener, separate-UID containment,
real namespace CAS, Vault migration, stream/upload cancellation or protected
activation is proven. No benchmark or human-test-ready package is claimed.

## Astra review 7 and repair scope

Executed the global autoreview helper with `--engine codex --codex-bin
/opt/homebrew/bin/codex --model gpt-6-astra --thinking high --mode local`, this
README as the scope packet, and explicit text/JSON output. Thread
`01a09425-0612-7483-a78d-597d55ddfb1b`. The helper exited 1; preserve
[both original findings](astra-seven.json), not a fabricated clean result.

Both are verified in-scope blockers, first repair cycle:

- P1: live captured pidfd plus numeric SO_PEERCRED alone did not identify a
  retained socket across PID reuse. Admission now obtains `SO_PEERPIDFD` using
  nix's existing safe API and rejects a dead socket process before PID/UID
  matching. It still owns the unreaped target Child before slot consumption.
  Unsupported socket-pidfd kernels deny; no numeric-only fallback. The stale
  connection test now asserts failure at that liveness gate, not merely an
  unknown numeric PID. This is a real dead-peer regression; it does not claim
  that the test forced kernel PID wrap/reuse.
- P2: early-drop thread creation could panic under resource exhaustion. Reaper
  capacity is now reserved during capture; failure returns both original Child
  objects. Drop never spawns a thread. New tests inject reaper creation failure
  and prove early drop reaps both children using the reserved reaper. Cleanup
  remains nonblocking on the supervisor; incomplete reaping is not success.

No owner boundary or runtime contract was expanded by these repairs. The
production binary remains unavailable. Original 11-test proof is retained in
`rtx/`; post-repair proof is kept separately rather than overwriting it.

The first post-repair wider run passed 335/338, with three unchanged
network-proxy tests failing at live `api.github.com` DNS/allowlist evaluation
after about five seconds. No proxy source was edited. A selected serial rerun
passed 3/3 in 0.054 seconds; `getent` resolved a public GitHub address.
The source uses `lookup_host` plus a timeout before the asserted MITM paths;
transient DNS failure is the likely explanation, not proven by direct resolver
tracing. Preserve the failed `rtx-final/affected.log` and selected rerun, then
require a fresh complete affected run before closeout. Do not hide these failures
or weaken allowlist checks to make them pass.

## Final proof frozen for review 8

The fresh complete run passed: strict default/fixture Clippy, Cargo/Bazel parity,
default service 3/3 (including two cleanup unit tests), synthetic suite 13/13,
affected broker/Vault/proxy 338/338, build and actual-key TMUX 13/13 plus default
exit78. See `rtx-verified/`; the earlier failed wider run remains in `rtx-final/`.
No source modification was needed for the transient proxy-test failures.
The local and RTX **Rust tree both equal**
`dcdba3e5a52c24994e15993b02af5cead0ff3831`.

Candidate directory:
`/home/travis/security-round5/evidence/pf27-child-admission-20260912/verified/candidate`.
Service SHA256 `86ebabe916e8ae71be5d1e7c8fb08510ee69936523bc79445240d8affdb8b776`;
fixture SHA256 `d4dc850a7e36966dd024e46e1febedfaac177f48c454ddf682a3a4ac1a01aaa7`.
The new library seam is exercised by Rust test binaries, not wired into the
default service or fixture main. Those two executable digests therefore stayed
unchanged across the admission-library repairs; this is not deployment proof.

## Fable review 8 and narrow test repair

Fable 5.1 High ran through the global helper, `--engine codex` with the existing
Corbanu Claude-plan wrapper, `--model claude-fable-5-1-plan --thinking high
--mode local`, in private TMUX `fable-eight`. Thread
`01a0942e-ee96-72a1-a046-8dd50a06a357`. Helper exit 1, overall patch correct,
no remaining runtime finding; [original result](fable-eight.json) retained.

Accepted P2: the dead-peer test assumed socket-pidfd creation always succeeds
and yields a readable dead handle. Verified Linux v6.15
[SO_PEERPIDFD handling](https://github.com/torvalds/linux/blob/v6.15/net/core/sock.c#L1777)
propagates `pidfd_prepare` errors;
[pidfd_prepare](https://github.com/torvalds/linux/blob/v6.15/kernel/fork.c#L2110)
returns EINVAL when that process has been reaped. Adjusted only the test to
accept ConnectionAborted or the specific ENODATA/EINVAL socket-identity refusal,
still before generic PID matching. Fresh-child admission must still succeed.
ENOPROTOOPT is not treated as a passing positive workflow: hosts lacking
SO_PEERPIDFD remain an unmet qualification prerequisite. RTX kernel is
7.0.0-31-generic; no run on 6.5–6.15 is claimed.

Second repair-cycle scope audit: only this test assertion remains actionable,
same owner/contract, no production source change or architecture expansion.
All prior runtime findings are resolved. Rerun affected proof, then one Fable
review of this repair/evidence; do not add panels or another plan/design pass.

Final post-portability-fix proof is in `rtx-portable/`: strict default/fixture
Clippy, parity, default 3/3, synthetic 13/13, affected 338/338, build, TMUX13/13
and exit78 all pass. Local/RTX Rust tree is now
`a940e60257f23bf7ecffddec234afa135522ada6`; production/fixture executable hashes
are unchanged. Review 9 is the requested brief Fable follow-up on this test-only
repair and evidence, not a new architecture/design panel. Review 8's substantive
runtime assessment remains preserved; no further production source changed.

## Qualified construction closeout

Source commit **`bd70f0e6b561c5ade88b3c6f8a87ebd7b937e5f3`** contains the exact
tested Rust tree `a940e60257f23bf7ecffddec234afa135522ada6`.
No production or test code changed after final formatting, tests or review 9.

Review 9: same Corbanu/TMUX Fable 5.1 High helper invocation as review 8, with
a focused follow-up prompt about the test-only repair and evidence. TMUX session
`fable-nine`, thread `01a09436-777b-7170-981e-b348c2bab214`. Helper **exit 0**,
[no findings](fable-nine.json), patch correct. The reviewer independently checked
file blobs but could not recompute a tree through its read-only environment;
root separately computed matching local/RTX Git Rust-tree objects. No extra
review is needed. Three additional reviews used; two remain in this window.

Final checks: default 3, synthetic 13, affected 338, strict Clippy in both
configurations, Cargo/Bazel lock parity, exact source/binary build and supporting
TMUX13 + default exit78 all passed. The all-platform/native/real-data/full-PF27
gates above remain open. The installed Mac application is unchanged. No new
human acceptance or benchmark result is claimed for this internal source seam.

Evidence publication: the repository globally ignores `.log`; these scoped
synthetic qualification logs are explicitly tracked. Published TMUX text strips
only trailing blank screen rows for whitespace checks; commands, output and
assertions are unchanged. Raw captures remain on RTX. No credential files or
unrelated runtime logs are included.
