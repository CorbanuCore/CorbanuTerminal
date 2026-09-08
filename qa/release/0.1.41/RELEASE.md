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
qualifying-cycle counter is not reset. New 0.1.41 live-repository PTY checks and
cross-platform interactive acceptance have not yet completed. Historical
sprint-checker errors remain outside this repair; no PF-42-S02 error was found.
A separate named-human acceptance record is unavailable. The explicit emergency
release authorization applies; these gaps are disclosed accurately.
