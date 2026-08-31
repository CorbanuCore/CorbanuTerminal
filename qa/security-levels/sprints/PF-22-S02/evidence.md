# PF-22-S02 evidence

PF-22-S02 remains `in_progress`. The implementation and the request-binding
remediation are committed, while the mandatory Claude Opus 5 Max review and
integration-owner combined-tree/archive work remain open.

## Candidate identity and contracts

- Allocation commit: `7fca549f731d95e7c8a63a93cd2aae6daa6fb6b3`.
- Recorded dispatch base: `43d2d86488d5c1b2eb5cbc401ee8371dbdb76bf4`.
- Initial implementation: `85837f64b5a833910864eb962ce1853d9c9321db`.
- P1 request-binding remediation: `f21025579bd802091c47cfc76ecb521f68a186bb`
  (`BoundedGrant` and mandate
  authority are now bound to the exact durable request/approved preview;
  cross-request substitution fails closed).
- Protected runtime contract: `PROTECTED_RUNTIME_CONTRACT_VERSION = 1`.
- Upstream seam register contract: `PF-22-S02-v1`; pinned inherited upstream
  revision: `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`.
- Post-remediation candidate: `corbanu 0.1.35`, SHA-256
  `4f945cf64ab9d05a9a66035951807300828ea85806e92721528040dd45b52f97`.

The implementation is deliberately cohesive: 518 production lines define one
fail-closed state machine and 623 test lines provide its mechanical proof. The
remaining change is manifest registration, the exact seam checker, and this
evidence. Splitting the state transition/fence/journal composition across
modules would weaken reviewability without reducing the security contract.

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

## Final-tree local verification

All build, cache, temporary, log, compatibility, and TMUX artifacts were kept
under `/Volumes/CorbanuDrive/Corbanu/.codex-work/pf22-protected-runtime/`.

- `cd codex-rs && just fix -p codex-core && just fmt`: completed; the unrelated
  inherited formatting suggestion described above was restored, and final
  `just fmt` is clean.
- `cd codex-rs && just test -p codex-core protected_runtime`: 6 passed, 3,458
  skipped. This includes positive grant/mandate flows and negative grant and
  mandate cross-request substitution cases.
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
  'test_security_upstream_seams.py'`: 6 passed.
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

Final test-log SHA-256 values:

- `protected-runtime-final.txt`: `cc3cc44e607d88eea7f1fd008586a57f9adc5817b3ca95a0d403d535f683bf09`
- `effective-policy-final.txt`: `d3d706975c5210c95e26e0ec7f1697a271a6a59d300f9c456ecc72ad8e1dce1a`
- `security-inheritance-final.txt`: `af16775d311b2d41c51ea41abfee34f40b75cb1a538e1026d6d32d98dd3b08a7`
- `authoritative-state-final.txt`: `f9dc4ee9a35d462a2d9846cc100f1a5517031b7a3d2e561bf868be4b5741f381`
- `revocation-final.txt`: `a1d3a58f07acf36a619e0c19d04c0b5a326fb0414383e961368b1ebbecc023eb`
- `security-audit-final.txt`: `c107ef0e72c6c56b077dd58637595dcec58f03cf9f9c6ed40fe4f8b092e73900`

## PF-21 compatibility comparison

The pre-remediation implementation commit `85837f64b` passed all 36 cases, but
that result is superseded as candidate closure by the P1 binding fix. Its
report is retained at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/pf22-protected-runtime/compat-run/compatibility-report.json`
(SHA-256 `520fae0d70fd9f2ef22f4ebbd775f168763b996e3f6a153254939f3a5c61c569`).

The required post-remediation comparison used:

```text
python3 scripts/security-level-compat --baseline 3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb --upstream af5a4e39b590e7517120fd935ccfac8cbf7cf131 --candidate /Volumes/CorbanuDrive/Corbanu/.codex-work/pf22-protected-runtime/candidate-target/debug/corbanu --cache-root /Volumes/CorbanuDrive/Corbanu/.codex-work/pf22-protected-runtime/compat-cache --temp-root /Volumes/CorbanuDrive/Corbanu/.codex-work/pf22-protected-runtime/compat-tmp --output /Volumes/CorbanuDrive/Corbanu/.codex-work/pf22-protected-runtime/compat-run-final
```

Post-remediation result: **passed 36/36** (nine baseline, nine upstream and nine
candidate expanded cases; four candidate protected cases; five immutable
probes). No case or probe failed, the candidate runtime tree was clean, control
cleanup had no warning, and the temporary control run root was removed. The
report source is remediation commit `f21025579bd802091c47cfc76ecb521f68a186bb`.
Its `source_dirty_paths` records only the two closeout Markdown files being
drafted concurrently with the isolated control replay; no Rust/runtime path was
dirty. Final report:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/pf22-protected-runtime/compat-run-final/compatibility-report.json`,
SHA-256 `b5609927b183fe22c046ac714946f1bfefd7dba6d26c8d6847534ff18031e673`.

## TMUX evidence

The exact compatibility-built binary was launched in a real 200x60 TMUX
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

The mandatory read-only review was attempted in a real TMUX/Corbanu Terminal
session configured for Claude Opus 5, effort `max`, read-only sandbox and
approval policy `never`. It could not start because `claude auth status`
reported `Not logged in`; this is an external authentication gate, not a clean
review verdict.

- Preserved auth-blocked transcript:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/pf22-protected-runtime/tmux-review/auth-blocked-transcript.txt`,
  SHA-256 `a8760102dcffef194e6888ceea4ee4768f8b555f314fd36a6aaf77ea18aaaeb1`.
- Findings status: no Opus findings were produced because provider
  authentication failed. The lane must be reviewed after `/login`; any valid
  P0/P1/P2 finding must be remediated and rereviewed before integration.
- The continuation prompt must review the post-remediation candidate and call
  out both the exact authority-to-request binding and the >800-line cohesion
  rationale above.

## Upstream seam register

The v1 register pins exact paths, symbols, revisions, owners, semantic
contracts, regression commands and evidence. Child inheritance is verified.
Ingress, egress, and the trusted human persistence adapter remain explicitly
pending for their hook-owning PF-23/PF-24 sprints; PF-22 does not claim those
adapters.

## Remaining gates and consumer handoff

- Complete the Claude Opus 5 Max read-only review and remediation loop.
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
