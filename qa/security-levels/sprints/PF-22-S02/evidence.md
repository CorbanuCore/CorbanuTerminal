# PF-22-S02 evidence

PF-22-S02 remains `in_progress`. The implementation, request-binding fix, and
the remediation of the first Claude Opus 5 Max review are committed. A fresh
Opus rereview of the remediated candidate and the integration-owner
combined-tree/archive work remain open.

## Candidate identity and contracts

- Allocation commit: `7fca549f731d95e7c8a63a93cd2aae6daa6fb6b3`.
- Recorded dispatch base: `43d2d86488d5c1b2eb5cbc401ee8371dbdb76bf4`.
- Initial implementation: `85837f64b5a833910864eb962ce1853d9c9321db`.
- P1 request-binding remediation: `f21025579bd802091c47cfc76ecb521f68a186bb`
  (`BoundedGrant` and mandate
  authority are now bound to the exact durable request/approved preview;
  cross-request substitution fails closed).
- Pending-review evidence checkpoint: `a59add9b1105020ddae464aa9996be23cbfcfbd8`.
- Opus P1/P2 remediation: `c0c00d443df7ba167c164a60685235d80cd6875b`.
- Protected runtime contract: `PROTECTED_RUNTIME_CONTRACT_VERSION = 2`.
- Upstream seam register contract: `PF-22-S02-v2`; pinned inherited upstream
  revision: `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`.
- Pre-Opus-remediation compatibility candidate: `corbanu 0.1.35`, SHA-256
  `4f945cf64ab9d05a9a66035951807300828ea85806e92721528040dd45b52f97`.

The implementation is deliberately cohesive: 558 production lines define one
fail-closed state machine and 723 focused test lines provide its mechanical
proof. The checker is a separate 391-line governance boundary with 108 lines
of focused tests. Splitting the state transition/fence/journal composition
across modules would weaken reviewability without reducing the security
contract. The greater-than-800-line lane is therefore accepted as one
security contract plus its mechanical proof, not unrelated feature work.

## Literal changed paths

- `codex-rs/Cargo.lock`
- `codex-rs/core/Cargo.toml`
- `codex-rs/core/src/security/effective_policy.rs`
- `codex-rs/core/src/security/mod.rs`
- `codex-rs/core/src/security/protected_runtime.rs`
- `codex-rs/core/src/security/protected_runtime_tests.rs`
- `qa/security-levels/upstream-seams.json`
- `scripts/security-upstream-seams-check`
- `scripts/security_upstream_seams_check.py`
- `scripts/tests/test_security_upstream_seams.py`
- `qa/security-levels/sprints/PF-22-S02/evidence.md`
- `docs/sprints/current/p0-security-levels/pf-22-s02-protected-runtime-and-upstream-seams.md`

No agent-control hook was changed. An opportunistic `just fix` edit to
`core/src/session/output_text_stream.rs` was restored byte-for-byte before the
candidate commits.

## Review remediation and disposition

The first substantive Opus review reported no P0, two P1 and six P2 findings.
Commit `c0c00d443df7ba167c164a60685235d80cd6875b` applies the confirmed,
in-scope findings:

- A trusted live dispatch time now drives grant/mandate validity, revocation
  fences and journal timestamps without changing the cryptographically frozen
  request/preview binding. A regression uses distinct preview, approval and
  dispatch times, and a second case proves a grant that expires before live
  dispatch fails closed.
- Mandates are one-shot in the durable journal: the internal deduplication key
  is derived from `mandate_id`, so a caller cannot replay one mandate by
  changing its request deduplication key.
- Readiness is bounded to a five-minute measurement window and the runtime
  independently re-derives `effective == max(requested, creator_required)`.
- The revocation read guard is explicitly dropped before journal I/O. It is
  intentionally retained through the single bounded effect: PF-19 defines the
  held guard as the effect's revocation linearization point. Dropping it before
  the effect and checking again afterward cannot undo a side effect that raced
  a revocation, so the reviewer's suggested change would weaken that consumed
  contract.
- `ProtectedDispatch` is `must_use`; the adapter consumes it through one
  terminal `resolve` operation, which updates both fence and durable journal.
  Event context is exposed to the in-crate adapter. A `Drop` implementation
  deliberately does not fabricate a terminal result: an unresolved durable
  intent is the correct crash/unknown state and is already surfaced by
  recovery.
- The seam checker now ignores comments and string literals, requires an exact
  top-level type or exact-impl method definition, confines all paths to the
  repository, checks Markdown evidence anchors at the current and pinned
  revisions, requires one exact tested revision, and rejects a source that has
  drifted since that revision. Nine negative/positive checker cases cover the
  strengthened boundary. The broad dead-code allowance is narrowed so tests
  compile the module without hiding unused API.

One finding is a recorded consumer-integration limit, not safely implementable
from PF-22's consumed interfaces: `CurrentProtectedRuntime.run_generation` and
the readiness generation must be supplied by the PF-23/PF-24 adapter from the
same authenticated live-session source used as the journal recovery anchor.
`RecoveryReport` does not expose an independently authenticated expected-run
value, so PF-22 can cross-check its two inputs but cannot manufacture that
root of trust. Adapter wiring and a combined-tree negative test remain an
explicit integration gate below.

## Final-tree local verification

All build, cache, temporary, log, compatibility, and TMUX artifacts were kept
under `/Volumes/CorbanuDrive/Corbanu/.codex-work/pf22-protected-runtime/`.

- `cd codex-rs && just fix -p codex-core && just fmt`: completed; the unrelated
  inherited formatting suggestion described above was restored, and final
  `just fmt` is clean.
- `cd codex-rs && just test -p codex-core protected_runtime`: 6 passed, 3,458
  skipped. This includes distinct preview/approval/dispatch times, mandate
  one-shot enforcement, readiness TTL/effective-level negatives, positive
  grant/mandate flows and negative cross-request substitution cases.
- `cd codex-rs && just test -p codex-core effective_policy`: 7 passed, 3,457
  skipped.
- `cd codex-rs && just test -p codex-core security_inheritance`: 3 passed,
  3,461 skipped.
- `cd codex-rs && just test -p codex-core authoritative_state`: 15 passed,
  3,449 skipped.
- `cd codex-rs && just test -p codex-security-policy revocation`: 10 passed, 37
  skipped.
- `cd codex-rs && just test -p codex-security-audit`: 44 passed, 0 skipped; one
  test was reported leaky by nextest, not failed.
- `python3 scripts/security-upstream-seams-check --manifest
  qa/security-levels/upstream-seams.json`: passed.
- `python3 -m unittest discover -s scripts/tests -p
  'test_security_upstream_seams.py'`: 9 passed.
- `python3 docs/sprints/check.py`: passed, 59 current and 96 archived.
- `python3 docs/plans/check.py`: passed, one active of two allowed and
  one available slot at the recorded checkpoint.
- `cd codex-rs && BAZELISK_HOME=... TEST_TMPDIR=... just bazel-lock-update`:
  completed with drive-local paths and no `MODULE.bazel.lock` change.
- `git diff --check`: passed.

`cargo clippy -p codex-core --tests --no-deps` was also attempted. PF-22's
initial `too_many_arguments` warning was remediated with a narrow, documented
allow on the composition boundary. The command remains non-green only because
of the inherited, out-of-scope `uninlined_format_args` finding in
`core/src/session/output_text_stream.rs`; PF-22 did not alter that file.

Post-Opus-remediation logs are under the drive-local `logs/` directory with
these SHA-256 values:

- `protected-runtime-opus-remediation.txt`:
  `fa257355f1322ef0d348fe50475285d8b3094264b1ae9fd8bc1b768ea03c199f`
- `effective-policy-opus-remediation.txt`:
  `8ea6f12f3e954b788f56c1261a8bf8314fdc4383e6b624ccef39e515edaeb02c`
- `security-inheritance-opus-remediation.txt`:
  `a81482dcd94ec6561eb983da91641aa919cab5217046e3eb3f5839803b2976a1`
- `authoritative-state-opus-remediation.txt`:
  `4df73d875c64a1e3f89b4e9694449a9f4afcb93f450f92a3c0fd48aa6a8d5c01`
- `revocation-opus-remediation.txt`:
  `a622a6180860930661fbb2ced7ecf04197bb86928fefd9389748f6d11aece421`
- `security-audit-opus-remediation.txt`:
  `cfe6d8b6ee514790e4a3cb8aa96a841963c08ec33da416ded238d96b71bba366`

Final governance log SHA-256 values:

- `seam-check-opus-remediation.txt`:
  `ad96a6c7d905ea21201b901f2f82ecf57117629ce349442422c88baec872a49b`
- `seam-unit-opus-remediation.txt`:
  `ad189066cd91f4f8ad8452f22394b7aead5252d58c1a3811f19d64b1a67f335a`
- `sprint-check-opus-remediation.txt`:
  `6b3b45c8eeb1144c261838f505ab6cbcf0f7e78800b5c3bad3077afe794e3fa3`
- `plan-check-opus-remediation.txt`:
  `9386e473c028f912d1685f25a88db8d21e57b8a9ad0929b07fcca3262ecbc8fb`

## PF-21 compatibility comparison

The pre-remediation implementation commit `85837f64b` passed all 36 cases, but
that result is superseded as candidate closure by the P1 binding fix. Its
report is retained at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/pf22-protected-runtime/compat-run/compatibility-report.json`
(SHA-256 `520fae0d70fd9f2ef22f4ebbd775f168763b996e3f6a153254939f3a5c61c569`).

The post-request-binding, pre-Opus-remediation comparison used:

```text
python3 scripts/security-level-compat --baseline 3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb --upstream af5a4e39b590e7517120fd935ccfac8cbf7cf131 --candidate /Volumes/CorbanuDrive/Corbanu/.codex-work/pf22-protected-runtime/candidate-target/debug/corbanu --cache-root /Volumes/CorbanuDrive/Corbanu/.codex-work/pf22-protected-runtime/compat-cache --temp-root /Volumes/CorbanuDrive/Corbanu/.codex-work/pf22-protected-runtime/compat-tmp --output /Volumes/CorbanuDrive/Corbanu/.codex-work/pf22-protected-runtime/compat-run-final
```

That result was **passed 36/36** (nine baseline, nine upstream and nine
candidate expanded cases; four candidate protected cases; five immutable
probes). No case or probe failed, the candidate runtime tree was clean, control
cleanup had no warning, and the temporary control run root was removed. The
report source is request-binding commit
`f21025579bd802091c47cfc76ecb521f68a186bb`.
Its `source_dirty_paths` records only the two closeout Markdown files being
drafted concurrently with the isolated control replay; no Rust/runtime path was
dirty. Final report:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/pf22-protected-runtime/compat-run-final/compatibility-report.json`,
SHA-256 `b5609927b183fe22c046ac714946f1bfefd7dba6d26c8d6847534ff18031e673`.

Because the Opus remediation changes protected dispatch behavior, this clean
comparison remains useful regression evidence but does not close compatibility
for `c0c00d443`; the integration owner must rerun the 36-case comparison on the
combined candidate.

## TMUX evidence

The exact pre-Opus-remediation compatibility-built binary was launched in a real 200x60 TMUX
session named `pf22-smoke-compat-candidate` with `RUST_LOG=trace`, read-only
sandbox, approval policy `never`, the allocated worktree as cwd, and
drive-local Corbanu home/log/temp paths. The literal `/status` command confirmed
`corbanu 0.1.35`, the allocated worktree, read-only/never permissions, and model
readiness. Literal `/exit` closed the session cleanly.

- Pane capture:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/pf22-protected-runtime/tmux-smoke-compat-candidate/status-pane.txt`,
  SHA-256 `cd87ae6091f15b101510bceebd2d4fbe627ecd37e3aecdbd3aa4b2746540fe73`.
- Trace log:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/pf22-protected-runtime/tmux-smoke-compat-candidate/logs/codex-tui.log`,
  SHA-256 `270c2f6af98ca6f969ca102e4396c7045da89f16d8e337534dc3c369c0731e19`.

## Review status

An initial review attempt in real TMUX/Corbanu Terminal was blocked because
`claude auth status` reported `Not logged in`. Its preserved transcript is:

- Preserved auth-blocked transcript:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/pf22-protected-runtime/tmux-review/auth-blocked-transcript.txt`,
  SHA-256 `a8760102dcffef194e6888ceea4ee4768f8b555f314fd36a6aaf77ea18aaaeb1`.

The integration owner then completed a token-authenticated, read-only review
of candidate range `7fca549f7..a59add9b1` through Corbanu Terminal using
Claude Opus 5, effort `max`:

- TMUX session `pf22-opus-long-token`, socket `pf22-long-token-review`.
- Prompt:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/claude-auth-review-runtime/pf22-review-prompt.md`,
  SHA-256 `f364fd88cbc9fc3296c42c1371b509470b85baedcd7e5f3ede6080bdaab4996e`.
- Transcript:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/claude-auth-review-runtime/pf22-opus5-max-review.txt`,
  56,393 bytes, SHA-256
  `aae8a5e3faa6916bfc47b90b8bfb551cc278fbec8793bc9fd42296a5a013ab21`.
- The model footer identified `Claude Opus 5 Plan`, `max`; an exact-pattern
  token-leak check was false.
- Verdict: no P0, two P1 and six P2. The verified dispositions and remediation
  are recorded above.

A fresh Opus 5 Max rereview of `c0c00d443` plus this evidence closeout remains
mandatory. PF-22 does not claim clean review closure until that transcript has
no actionable P0/P1/P2 findings.

## Upstream seam register

The v2 register pins repository-contained paths, exact definitions, one tested
revision, owners, semantic contracts, regression commands and verified
Markdown evidence anchors. Child inheritance is verified.
Ingress, egress, and the trusted human persistence adapter remain explicitly
pending for their hook-owning PF-23/PF-24 sprints; PF-22 does not claim those
adapters.

## Remaining gates and consumer handoff

- Complete the Claude Opus 5 Max read-only rereview of the remediated candidate
  and resolve any remaining actionable P0/P1/P2 finding.
- PF-23/PF-24 must bind the runtime and journal-recovery expected run
  generation to one authenticated live-session source and add a combined-tree
  mismatch test; PF-22 cannot prove that root from `RecoveryReport` alone.
- Integration owner reruns the affected/seam/compatibility/governance matrix on
  the combined tree and records the integration reserve actually consumed.
- PF-23 connects and verifies ingress and egress adapters; PF-24 connects and
  verifies the trusted human update/persistence adapter.
- Windows/Linux qualification was intentionally not run and remains outside
  this lane until the user changes tailnets and the integration owner directs
  it.
- Integration owner archives PF-22-S02 and updates shared plan/navigation
  ledgers only after all gates close.

Historical PF-22-S01 archive and evidence remain unchanged.
