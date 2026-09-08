# Corbanu Terminal 0.1.41 emergency release

The requesting repository operator authorized another emergency release on
2026-09-08 after confirming that the public 0.1.40 still contains the account
recovery defect. Codex is executing that release instruction.

## Behavior and included work

Named profiles retain independently linked Task Node sessions. A completed
GitHub relink takes priority over a stale saved token; TUI and CLI status share
profile-scoped recovery and serialize one-time exchanges. Status displays the
local profile, and CLI link instructions retain the selected profile. Structured
legacy authentication errors now give current Corbanu instructions.

The Task Node server separately deploys Corbanu error/completion wording and a
GitHub account picker. Those changes are already live. An owner whose old token
was revoked still needs to complete a fresh GitHub sign-in; this repair does
not reinstate revoked sessions.

This release packages the completed product initiative
[PF-42-S02](../../../docs/sprints/archive/p0-security-levels/pf-42-s02-relink-recovery.md).
Product heading **Shipping MVP — LIVE**, rows **Task Node and identity** and
**Named profiles**: independently selectable named profiles and linked Task
Node identity. Implementation commit: `f20a2a7389e2baa5eaddcd00755ca93129591808`.
[Detailed repair evidence](../../reliability/2026-09-08-corbanu-account-recovery.md)
records the failure boundary, server deployment and installed local candidate.
Shipped guidance: [Task Node](../../../docs/features/tasknode.md).

Worktree: `/home/pfrpc/repos/worktrees/corbanu-release-0.1.39`; branch
`fix/tasknode-agent-profile-scope`; release preparation base `7a3b4b8c37`.
The directory name is historical; this candidate is 0.1.41.

## Existing implementation evidence

48 affected Rust tests passed after scoped Clippy/fix, formatting and snapshot
review. The actual terminal passed revoked-token recovery, pending and completed
GitHub linking, two-account isolation, cold restart, concurrent status and
cancellation using local authentication fixtures. The same matrix passed
against the locally installed stripped executable. No real GitHub login or
model benchmark is claimed by those fixtures.

Live server checks passed the authentication error, completion page and OAuth
account-picker redirect. A separate valid production account remained usable;
the revoked account correctly continues to require its owner's sign-in.

## Release verification and publication

All 148 workspace package versions are 0.1.41. Cargo updated the workspace
lockfile without changing any external dependency entries. Bazel lock update
passed and produced no MODULE.bazel.lock change. Five installer contract tests,
plan validation, formatting and diff checks passed. The initial Bazel attempt
correctly rejected the stale Cargo lock; it passed after the workspace lock was
updated.

The same implementation already passed the 48 affected tests and installed PTY
matrix above. A fresh 0.1.41 build/test replay is being collected while the
release workflow builds all five supported platform archives plus macOS
installers, with checksums. Exact source, workflow, artifact and installed
results will be added as they complete. Publication is not yet claimed.

Private release artifacts:
`/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-0.1.41-qa`.
Prior implementation artifacts:
`/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-multi-account-20260908`.

## Qualification disclosure

The full competitor and coding-model benchmark matrix is incomplete; the
qualifying-cycle counter is not reset. Linux 0.1.41 live-repository PTY checks passed as recorded below.
Cross-platform interactive acceptance has not completed. Historical
sprint-checker errors remain outside this repair; no PF-42-S02 error was found.
A separate named-human acceptance record is unavailable. The explicit emergency
release authorization applies; these gaps are disclosed accurately.

## Publication dispatch and fresh tests

Source `739897fd64527898fdddc2dec1ab15be287f620a` was pushed to `main`.
[Release run 34180024130](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/34180024130)
validated the release inputs and started all five platform builds with
publication and latest-release selection enabled. Publication remains pending
at this checkpoint.

The fresh versioned source passed all **48 affected Rust tests** in one run
(4,247 excluded by the explicit filter). The prior provider-path no-regex
regression, portable skill mirror check and Bazel lock drift check also passed.
No external dependency entries changed. The local 0.1.41 executable build and
new repository PTY checks are being collected.

User service `corbanu-emergency-release-0141.service` monitors the exact source
and workflow. On successful publication it verifies the tag and Linux package
checksum, applies the prepared public notes, installs the official package,
and repeats the account-recovery terminal matrix in disposable TensorCash and
Isometric Game worktrees. It preserves a newer independently installed version.
Results or failures are written to the private `publication-status.json`;
future completion is not represented as an existing pass.

## Fresh 0.1.41 interactive qualification

The Linux development executable built successfully from the immutable release
source and reports `corbanu 0.1.41`. Its SHA256 is `3af400b99acad3b5cdce8ed915e90eac06296ab5d05a95540d8990e356acaecd`.
No product source changed after the release commit.

Actual PTY keys passed the full account-recovery matrix in both disposable
repositories: revoked-token guidance, waiting for GitHub, completed relink,
separate account isolation and cold restart. Adjacent CLI checks passed
simultaneous one-time exchange, cancellation preserving the active account,
and an independently unlinked default scope.

Repository inputs:

- TensorCash: `https://github.com/agtico/tensorcash.git`, base
  `9325ed67d23355170d6ad38ad58ea776d049ae4e`, disposable worktree
  `.../corbanu-0.1.41-qa/tensorcash-canonical`.
- Isometric Game: `https://github.com/goodalexander/isometricgame.git`, base
  `59821b7a85524f186f946c4670480c7ee96483cb`, disposable worktree
  `.../corbanu-0.1.41-qa/isometricgame`.

Private evidence: `candidate-qualification.json`, per-repository
`candidate-pty-results.json` and PTY captures. These checks use local HTTP
authentication fixtures with zero model calls. They prove the interactive
account workflow, not coding benchmarks or a real GitHub browser login.
The existing local account repair remains installed while the official
cross-platform packages build. The verified user service will check the actual
official Linux package and repeat this matrix after publication.
