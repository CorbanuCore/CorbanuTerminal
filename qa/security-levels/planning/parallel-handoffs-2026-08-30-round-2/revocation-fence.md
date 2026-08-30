# Revocation/fence lane handoff

Sprint: PF-19-S02, currently `ready`.

Owner: Codex revocation/fence lane. Integration owner: Codex
ingress/classifier lane.

Work only in `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-revocation-fence`
on `feat/p0-security-revocation-fence`, allocated from
`5521b681fff0ecb50b17c10bc1dd1356cbecc1b6`. Put all build and temporary output
under `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-revocation-fence/`.

## Authority and scope

This is a live allocation: set PF-19-S02 to `in_progress`, run both governance
checkers, then implement. No additional worktree, branch, owner, or G0 decision
is missing.

Literal scope:

```text
codex-rs/security-policy/src/revocation.rs
codex-rs/security-policy/src/security_policy_tests.rs
qa/security-levels/sprints/PF-19-S02/
docs/sprints/current/p0-security-levels/pf-19-s02-dispatch-revocation-fence.md
```

Do not touch Cargo/Bazel manifests or locks, `security-policy/src/lib.rs`, Core,
transport adapters, plan/index/MkDocs, or other sprint records. If a public
export is genuinely required, stop and hand the exact proposed shared edit to
the integration owner.

## Deliverable

Extend the accepted PF-19-S01 generation/revocation contract with explicit
queued, admitted, uploading, established-channel, and completed/unknown states.
Define the linearization point after which a kill/restriction permits no new
protected dispatch or write. Require the current generation at every dispatch;
fence stale queued work, open streams, and uploads without revoking unaffected
siblings. Audit unavailability cannot delay emergency restriction, while an
already submitted or unknown financial effect must remain honestly unknown.

Tests must cover deterministic interleavings, repeated kill, stale generations,
open channels, in-flight upload, audit-unavailable operation, sibling isolation,
and unknown financial outcomes. Preserve S01 semantics and evidence; do not
claim transport or restart qualification from a pure contract test.

## Verification and review

Run `just fix -p codex-security-policy`, `just fmt`, focused revocation tests,
the complete `codex-security-policy` suite, both governance checkers, and
`git diff --check`. Inspect the final diff after formatting.

Use TMUX + Corbanu Terminal + Claude Opus 5.0 Max for the independent review.
Ask specifically about linearization, stale-generation races, admitted versus
completed effects, stream/upload fencing, sibling over-revocation, audit failure,
idempotency, and fail-open behavior. Repeat until clean.

Hand back the candidate/base, exact diff and test counts, contract transitions,
scope audit, limitations, and transcript hash. The integration owner merges this
candidate first and archives PF-19-S02 only after combined-tree checks.

