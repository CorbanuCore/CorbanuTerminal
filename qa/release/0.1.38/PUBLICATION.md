# Corbanu Terminal 0.1.38 — authorized publication

## Authority and scope

Date: 2026-09-05 UTC.
Release owner: the requesting repository operator, who explicitly instructed
“k plz run a release” in this session. Publication is authorized under the root
AGENTS.md release rule; this does not imply separate named-human QA acceptance.
Change class: release, packaging the reconciled accepted work and bounded fixes.

Product specification: **Shipping MVP — LIVE**, “Linux/macOS/Windows, the
`corbanu` command,” “Multi-provider inference,” and “model-aware delegation,
durable mailboxes, supervision, resume, and recovery.” Included initiative and
sprint evidence remains linked from [the candidate record](RELEASE-CANDIDATE.md).
No new product initiative, plan slot, or sprint is introduced by packaging.

Source branch: `integration/reconcile-release-0.1.38`.
Worktree: `/home/pfrpc/repos/worktrees/corbanu-release-0.1.38-reconcile`.
Last runtime/evidence commit: `753070610fdc548bbbca770887233c3cdf463783`.
This publication preparation changes documentation only. Cargo already declares
0.1.38. At authorization, the latest published version is 0.1.37 and no 0.1.38
tag or release exists. No published version or asset is being replaced.

## Release execution

Use `.github/workflows/corbanu-terminal-release.yml` from the source branch,
with `release_version=0.1.38`, `publish_release=true`, `make_latest=true` and no
artifact reuse. The run's captured head SHA is the immutable build source.
The workflow creates `rust-v0.1.38` and publishes only after its required jobs
succeed. Expected assets: five platform packages, two macOS DMGs, their checksum
manifests, and Unix/PowerShell installer scripts.

Status: authorized; dispatch and its exact run identity will be recorded below.

## Evidence and disclosed gaps

- Latest focused qualification: 183 Core tests and 22 harness tests pass;
  explicit and inherited Astra children pass actual TUI tool work,
  cancellation/recovery and cold resume in both TensorCash and Isometric Game.
  Four children produced 20 real provider responses and 12 paired tool calls.
  [Exact evidence](astra-subagents.md) includes the intermittent existing resume
  fixture timeout on an earlier run, without concealing or relabeling it.
- Prior Luna/Kimi, native Astra, provider/auth and reconciliation evidence is
  preserved in the candidate record; those results apply to their recorded
  source trees and are not represented as new full-suite coverage.
- Local Unix installer release-contract check: 4/4 pass before dispatch.
- Linux/macOS/Windows release builds and package smoke tests are pending this
  workflow; no new cross-platform pass is claimed before completion.
- Full workspace tests and separate named-human acceptance are not complete.
- Benchmark bootstrap remains incomplete: no qualifying competitive
  Corbanu/Hermes/Kilo cycle or complete coding task/model matrix, frozen limits,
  runtime/spend evidence, or qualifying baseline. No counter reset or zero-spend
  assumption. See [the cadence ledger](../../../benchmarks/README.md).
- The previously disclosed debug/trace keyboard credential-logging concern
  remains unresolved. Real-credential local QA used `RUST_LOG=warn`; avoid
  debug/trace logging while entering credentials. This release does not claim
  to repair that separately scoped protected-data issue.
- Existing documentation-link warnings and dependency alerts are not claimed
  fixed by this release. No new source, credential, auth, wallet or logging
  behavior is changed as part of publication preparation.

These gaps are disclosed, not passed. The explicit human release instruction
authorizes publication without delaying for missing benchmark, acceptance or
other qualification evidence. Operational build/publish failures will still be
reported and addressed in scope.
