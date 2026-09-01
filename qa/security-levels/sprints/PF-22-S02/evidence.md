# PF-22-S02 evidence

PF-22-S02 remains `in_progress`. The implementation, request-binding fix, and
five Claude Opus 5 Max remediation cycles are committed. A final clean Opus
rereview of the fifth remediated candidate and the integration-owner
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
- Opus rereview P1/P2 remediation:
  `545456f07c48157c4a0d91fc6a981ab8e0561636`.
- Opus final-pass P2 remediation:
  `24a51cddb53f60a5ce6dad80c3806d96f2946c2f`.
- Opus final-pass retry remediation:
  `d658c1d22ce69c26c63fe06d69797163d7bdfd3b`.
- Opus ship-pass lifecycle remediation:
  `15ed4d95a55d4c815c043b40a1809e56aab6ad7f`.
- Protected runtime contract: `PROTECTED_RUNTIME_CONTRACT_VERSION = 6`.
- Upstream seam register contract: `PF-22-S02-v6`; pinned inherited upstream
  revision: `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`.
- Pre-Opus-remediation compatibility candidate: `corbanu 0.1.35`, SHA-256
  `4f945cf64ab9d05a9a66035951807300828ea85806e92721528040dd45b52f97`.

The implementation is deliberately cohesive: 717 production lines define one
fail-closed state machine and 1,218 focused test lines provide its mechanical
proof. The checker is a separate 400-line governance boundary with 126 lines
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
- `codex-rs/security-audit/src/journal.rs`
- `codex-rs/security-audit/src/journal_types.rs`
- `codex-rs/security-audit/src/journal_tests.rs`
- `codex-rs/security-audit/tests/consumer_contract.rs`
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
- The first remediation made mandates one-shot by `mandate_id`; the second
  review correctly showed that reapproval changes that ID. The final key is
  derived from the effect-stable `preview_digest`, so neither a caller-selected
  key nor a second approval over the same preview escapes the durable fence.
- Readiness is bounded to a five-minute measurement window and the runtime
  independently re-derives `effective == max(requested, creator_required)`.
- The revocation read guard is explicitly dropped before journal I/O. It is
  intentionally retained through the single bounded effect: PF-19 defines the
  held guard as the effect's revocation linearization point. Dropping it before
  the effect and checking again afterward cannot undo a side effect that raced
  a revocation, so the reviewer's suggested change would weaken that consumed
  contract.
- `ProtectedDispatch` is `must_use`; its one exact terminal resolution updates
  both fence and durable journal, retaining the permit only when a journal
  attempt is known non-ambiguous. Event context is exposed to the in-crate
  adapter. A `Drop` implementation
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

The first rereview of `7fca549f7..114c2726a` then reported no P0, two P1 and
five P2 findings. Commit `545456f07c48157c4a0d91fc6a981ab8e0561636`
remediates every confirmed item without changing the accepted PF-19 guard or
inventing an automatic `Drop` outcome:

- Mandate deduplication now uses the stable preview digest; two approvals at
  different times over one preview hit the same terminal reservation.
- A grant fenced before first admission can be durably closed as Unknown,
  Denied or Cancelled. A mandate without durable admission proof closes only
  as Unknown; completed outcomes still require the non-forgeable receipt
  contract, so the recovery fix cannot fabricate a mandate result.
- `resolve` no longer accepts caller-supplied event generations; it derives
  the exact context from the bound runtime.
- The exact readiness window is immutable for the runtime, and an atomic
  accepted-time high-water mark rejects clock regression. An invalid
  out-of-window future observation does not poison the accepted clock.
- Actor chain, session and task are bound into the runtime snapshot and checked
  against both every current policy snapshot and every authorization request.
- Empty/ready recovery report shape and checkpoint sequence, producer, owner
  generation, non-future policy generation and non-future run generation are
  independently checked against values available to PF-22 today. The
  authenticated source of the expected run generation remains the honest
  PF-23/PF-24 integration gate described above.
- The seam lexer parses complete plain, escaped, Unicode and byte-character
  literals before recognizing lifetimes, preserving brace scope; its tenth
  regression case proves following top-level and nested definitions remain
  distinguishable.

The next Opus pass reported no P0/P1 and three P2 findings. Commit
`24a51cddb53f60a5ce6dad80c3806d96f2946c2f` applies the two safe in-scope
fixes and narrows the third claim:

- Caller-controlled grant deduplication keys are hashed into a
  `grant-effect:` domain. They can no longer collide with the
  `mandate-preview:` namespace or pre-burn a mandate's stable reservation.
- Every `ProtectedRuntime` now has an opaque random instance identity copied
  into its dispatches. Both authorization and resolution reject a dispatch
  presented to a different runtime, even when caller-visible generations and
  revocation state happen to match.
- A mandate that expires or is revoked before first admission is regression
  tested to close conservatively as Unknown. PF-22 deliberately does not
  relax the generic audit contract to accept caller-asserted receiptless
  Denied/Cancelled outcomes: the current event shape has no authenticated
  admission fact with which to distinguish that case. A durable admission
  marker or non-forgeable `DispatchFence`-to-audit proof is a separately
  scoped future contract dependency, not a completed PF-22 capability.

The next final pass verified all three fixes above and reported no P0/P1 plus
one P2 retry-safety finding. The integration owner explicitly expanded scope
to the smallest security-audit API/test surface, and commit
`d658c1d22ce69c26c63fe06d69797163d7bdfd3b` remediates it:

- `ReferenceJournal::resolve_dispatch` borrows the deliberately non-clone
  `DispatchPermit`. The journal remains exactly-once authority: the identical
  durable resolution is acknowledged idempotently and a conflicting terminal
  resolution is rejected.
- `ProtectedDispatch` retains that permit and the exact terminal outcome across
  validation or pre-commit errors. A retry cannot change Completed to Unknown;
  the permit is removed only after an acknowledgement or immediately when the
  journal reports `CommitUnknown`.
- A pre-write disk-full fault proves the same permit can recover and persist
  Completed; a timestamp-regression case proves an already Executed effect can
  correct its event time without losing the exact outcome. An after-rename
  `CommitUnknown` fault proves direct reuse stays blocked and recovery reports
  records ahead of the integrity root.

The ship pass verified the retry contract and reported no P0/P1 plus one P2
lifecycle finding: borrowing `ProtectedDispatch` for retry left a
never-admitted fence Queued after successful resolution, so the surviving
handle could authorize a post-terminal effect. Commit
`15ed4d95a55d4c815c043b40a1809e56aab6ad7f` closes that path before any
runtime/fence/effect work: authorization requires a live permit and no pending
resolution. Focused negatives prove effects cannot run while resolution is
pending or after ordinary Completed/Unknown, mandate Completed, grant
Cancelled, never-admitted grant Cancelled, or never-admitted mandate Unknown.

## Final-tree local verification

All build, cache, temporary, log, compatibility, and TMUX artifacts were kept
under `/Volumes/CorbanuDrive/Corbanu/.codex-work/pf22-protected-runtime/`.

- `cd codex-rs && just fix -p codex-core && just fmt`: completed; the unrelated
  inherited formatting suggestion described above was restored, and final
  `just fmt` is clean.
- `cd codex-rs && just test -p codex-core protected_runtime`: 6 passed, 3,458
  skipped. This includes distinct preview/approval/dispatch times, mandate
  reapproval replay, never-admitted cancellation, immutable/monotonic
  readiness, actor/request and recovery-checkpoint bindings, readiness
  TTL/effective-level negatives, positive grant/mandate flows, negative
  cross-request substitution cases, exact resolution retry, and effect denial
  during retry-pending and after every exercised terminal path.
- `cd codex-rs && just test -p codex-core effective_policy`: 7 passed, 3,457
  skipped.
- `cd codex-rs && just test -p codex-core security_inheritance`: 3 passed,
  3,461 skipped.
- `cd codex-rs && just test -p codex-core authoritative_state`: 15 passed,
  3,449 skipped.
- `cd codex-rs && just test -p codex-security-policy revocation`: 10 passed, 37
  skipped.
- `cd codex-rs && just test -p codex-security-audit`: 46 passed, 0 skipped; one
  test was reported leaky by nextest, not failed.
- `cd codex-rs && cargo clippy -p codex-security-audit --tests --no-deps`:
  passed without findings.
- `python3 scripts/security-upstream-seams-check --manifest
  qa/security-levels/upstream-seams.json`: passed.
- `python3 -m unittest discover -s scripts/tests -p
  'test_security_upstream_seams.py'`: 10 passed.
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

Post-rereview-remediation test-log SHA-256 values:

- `protected-runtime-rereview-remediation.txt`:
  `6a2830b45f77fa31cc7caa9a403a9369aee7a9188c9114a45ad1321a1df5f124`
- `effective-policy-rereview-remediation.txt`:
  `13a7a1eff3979c1f94dd2f01856b2fee050995a036d633a6f6a7209a0415a3b1`
- `security-inheritance-rereview-remediation.txt`:
  `7e1ed16f55ddc48e29fade20cd9f791ed38d50a5af20ac1379f0af0c493fe45a`
- `authoritative-state-rereview-remediation.txt`:
  `f71ed95807fe9ada7040a44c31edc42c3fe984222f2c29daf9193ae0beaf2cd2`
- `revocation-rereview-remediation.txt`:
  `8b4f67cbdbd937cc4de6c63819ce3769516493df399e7b29ca937beb25c5983f`
- `security-audit-rereview-remediation.txt`:
  `70f2b4480bc5266ee1a7cc95e5122b70d55d4259e65aa32e0882068deeba2563`
- `seam-check-rereview-remediation.txt`:
  `ad96a6c7d905ea21201b901f2f82ecf57117629ce349442422c88baec872a49b`
- `seam-unit-rereview-remediation.txt`:
  `cb389234a5da43b5a4bbb531d37f6764ea82dcb0df693527cc4d194e885f407e`
- `sprint-check-rereview-remediation.txt`:
  `6b3b45c8eeb1144c261838f505ab6cbcf0f7e78800b5c3bad3077afe794e3fa3`
- `plan-check-rereview-remediation.txt`:
  `9386e473c028f912d1685f25a88db8d21e57b8a9ad0929b07fcca3262ecbc8fb`

Final-pass-remediation affected-test log SHA-256 values:

- `protected-runtime-final-pass1-remediation.txt`:
  `3e1564f304b329731295253c2b304d60c605268dc80edf1bb8b5f868309a524a`
- `effective-policy-final-pass1-remediation.txt`:
  `a7aac8b32a719f434414a62cd69f6dba640f76f2f049bdec39db89335cad1c07`
- `security-inheritance-final-pass1-remediation.txt`:
  `32cc7c48e93dc651dc6b127bc276fca1dbfe9daed1a1db882031fb57b19f7e05`
- `authoritative-state-final-pass1-remediation.txt`:
  `8029fcf6c457bf60ca3385bd877d10d87fba48b6becb7e9fa11732c55ae7f7e3`
- `revocation-final-pass1-remediation.txt`:
  `e5755747ef64fa2e5d7ed1b0593c01aa4e7aa145dd77c78772d38a4863b43c87`
- `security-audit-final-pass1-remediation.txt`:
  `4bd974dbb3cf46f1c323ec5aa1d7ea51ebe647b998a5830048e72658a678e882`
- `seam-check-final-pass1-remediation.txt`:
  `ad96a6c7d905ea21201b901f2f82ecf57117629ce349442422c88baec872a49b`
- `seam-unit-final-pass1-remediation.txt`:
  `5845d1bbf79c958d55cd13e7f56deeb6f4b8a5e4f16cb8bc810851947ec5ffcc`
- `sprint-check-final-pass1-remediation.txt`:
  `6b3b45c8eeb1144c261838f505ab6cbcf0f7e78800b5c3bad3077afe794e3fa3`
- `plan-check-final-pass1-remediation.txt`:
  `9386e473c028f912d1685f25a88db8d21e57b8a9ad0929b07fcca3262ecbc8fb`

Final-pass2 retry-remediation log SHA-256 values:

- `protected-runtime-final-pass2-remediation.txt`:
  `cba2b2d13f73b2eb0f0f5fb687a4cb28f2c4b7ccd081068deda90e8f2ca7adce`
- `effective-policy-final-pass2-remediation.txt`:
  `5bc735fc6deccd2a4fa250157424bfde72d0b546b6f65c7a4b2bbfcccace58f5`
- `security-inheritance-final-pass2-remediation.txt`:
  `21bd5b7b5a8b82dc7b1eb8ce2387254b1af84b454955619fb2ab4ce7aedb09cf`
- `authoritative-state-final-pass2-remediation.txt`:
  `16387e4ae49d702417afc70051b042a79339212879c44628104afd9bde035251`
- `revocation-final-pass2-remediation.txt`:
  `6936b96fb0c17caea94bf5eb307752c44fc6a67fd940c016d492007e8c4912ca`
- `security-audit-final-pass2-remediation.txt`:
  `1db940c10de0f455c26345c062f3240fd9ca0400e2ca5111f829970d232c7749`
- `security-audit-clippy-final-pass2-remediation.txt`:
  `ff9ae9661b24673ef02fe519a388a9e3ea4f7e967780806c1f9bfb0c33a6623d`
- `seam-check-final-pass2-remediation.txt`:
  `ad96a6c7d905ea21201b901f2f82ecf57117629ce349442422c88baec872a49b`
- `seam-unit-final-pass2-remediation.txt`:
  `476742bad03e4ebf8c91d1954977b739c050adc4b73b40f108032adb52fc549f`
- `sprint-check-final-pass2-remediation.txt`:
  `6b3b45c8eeb1144c261838f505ab6cbcf0f7e78800b5c3bad3077afe794e3fa3`
- `plan-check-final-pass2-remediation.txt`:
  `9386e473c028f912d1685f25a88db8d21e57b8a9ad0929b07fcca3262ecbc8fb`

Ship-pass lifecycle-remediation affected-test log SHA-256 values:

- `protected-runtime-ship-pass-remediation.txt`:
  `1c0557d1bef7e71a54d53d490c164d909774675044c1b4f4192fd8d191520329`
- `effective-policy-ship-pass-remediation.txt`:
  `92538ee315313606e11423d010026c5b97c3476579ed2e080a676a1257cfac9d`
- `security-inheritance-ship-pass-remediation.txt`:
  `806f6649012ac0e44e069960fbdb59d5fdecadfb146e3336a9550dc6e3377168`
- `authoritative-state-ship-pass-remediation.txt`:
  `74078ca0018deda3d524040af8751a0383a52210f598c2d6df74193cf15399fc`
- `revocation-ship-pass-remediation.txt`:
  `f2737cf182947bf64e84f5946716136d2c055706d331d90c3e7786f3179b2b36`
- `security-audit-ship-pass-remediation.txt`:
  `51846fd1db68d9674f8a27752fb83837cc9ad17cf6c97b535be8dcdbc48f0dcc`
- `seam-check-ship-pass-remediation.txt`:
  `ad96a6c7d905ea21201b901f2f82ecf57117629ce349442422c88baec872a49b`
- `seam-unit-ship-pass-remediation.txt`:
  `8522696a746f3d2282019c90cd06c62a76f1cdc5a535f6e4233c75cd66c718bb`
- `sprint-check-ship-pass-remediation.txt`:
  `6b3b45c8eeb1144c261838f505ab6cbcf0f7e78800b5c3bad3077afe794e3fa3`
- `plan-check-ship-pass-remediation.txt`:
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

Because all five Opus remediation cycles change protected dispatch behavior,
this clean comparison remains useful regression evidence but does not close
compatibility for `15ed4d95a`; the integration owner must rerun the 36-case
comparison on the combined candidate.

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

The integration owner performed the required rereview of range
`7fca549f7..114c2726a` in the same read-only TMUX + Corbanu + Claude Opus 5 Max
configuration:

- Prompt:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/claude-auth-review-runtime/pf22-rereview-prompt.md`,
  SHA-256 `0fe357c32e9b31934d7ba1b9139be8fc521ecab5863d2bcb8891742993ecf843`.
- Transcript:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/claude-auth-review-runtime/pf22-opus5-max-rereview.txt`,
  24,635 bytes, SHA-256
  `855a73f96fadcba5715d11a6f7549174734dc3cfa3ac04a9571e611f4f327537`.
- Exact-pattern token-leak check: false.
- Verdict: no P0, two P1 and five P2. All seven were independently confirmed
  and remediated in `545456f07` as recorded above.

The integration owner then reviewed range `7fca549f7..6f8a6d5c0` in the same
read-only configuration:

- Prompt:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/claude-auth-review-runtime/pf22-final-review-prompt.md`,
  SHA-256 `15d65fba0e357f424fde8614bc7de8799174163f153a5e4cb62483dc69747d4b`.
- Transcript:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/claude-auth-review-runtime/pf22-opus5-max-final-pass1.txt`,
  16,387 bytes, SHA-256
  `b58315b2637f4b6cb8084287c873a2e96a48fc93293fc043e8a27ffe6e36204c`.
- Exact-pattern token-leak check: false.
- Verdict: no P0/P1 and three P2. The two safe in-scope findings are fixed in
  `24a51cddb`; the unsafe generic audit relaxation is rejected and replaced by
  the conservative Unknown proof and explicit future dependency above.

The integration owner then reviewed complete range
`7fca549f7..c22905769`, focusing on `24a51cddb`, in the same read-only
configuration:

- Prompt:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/claude-auth-review-runtime/pf22-final-review-pass2-prompt.md`,
  SHA-256 `3ef04ad0060555e01bda741963b39a4b15bbb57dca59c658d98bab3e2d7148b7`.
- Transcript:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/claude-auth-review-runtime/pf22-opus5-max-final-pass2.txt`,
  17,051 bytes, SHA-256
  `21a54f2bdab3319c6182df28d265459fca2a3dd01179823271c59a9ec646b995`.
- Exact-pattern token-leak check: false.
- Verdict: no P0/P1 and one P2. It verified all prior fixes and identified the
  lost-permit retry gap now fixed in `d658c1d22` under the explicitly expanded
  audit scope.

The integration owner then reviewed complete range
`7fca549f7..83f23dc06`, focusing on `d658c1d22`, in the same read-only
configuration:

- Prompt:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/claude-auth-review-runtime/pf22-ship-review-prompt.md`,
  SHA-256 `9b78b6a069d0f644d3e8758acdbd4f92284035f628ff6c4cee236bf65c24e780`.
- Transcript:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/claude-auth-review-runtime/pf22-opus5-max-ship-pass.txt`,
  14,896 bytes, SHA-256
  `7ec303b9a5775071f6946bc10e31fe304224a92be21af42ecd47a26aba8ea0b9`.
- Exact-pattern token-leak check: false.
- Verdict: no P0/P1 and one P2. It verified the retry fixes and identified the
  effect-after-terminal lifecycle gap now fixed in `15ed4d95a`.

A final focused Opus 5 Max rereview of `15ed4d95a` plus this evidence closeout
remains mandatory. PF-22 does not claim clean review closure until that
transcript has no actionable P0/P1/P2 findings.

## Upstream seam register

The v6 register pins repository-contained paths, exact definitions, one tested
revision, owners, semantic contracts, regression commands and verified
Markdown evidence anchors. Child inheritance is verified.
Ingress, egress, and the trusted human persistence adapter remain explicitly
pending for their hook-owning PF-23/PF-24 sprints; PF-22 does not claim those
adapters.

## Remaining gates and consumer handoff

- Complete the final Claude Opus 5 Max read-only rereview of the five-times-remediated candidate
  and resolve any remaining actionable P0/P1/P2 finding.
- Define in a future explicitly scoped cross-crate contract a durable admission
  marker or non-forgeable fence-to-audit proof before allowing receiptless
  mandate Denied/Cancelled closure; PF-22 only proves conservative Unknown.
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
