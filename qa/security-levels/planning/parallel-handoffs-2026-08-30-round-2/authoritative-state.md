# Authoritative-state lane handoff

Sprint: PF-20-S02, currently `ready`.

Owner: Codex authoritative-state lane. Integration owner: Codex
ingress/classifier lane.

Work only in `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-authoritative-state`
on `feat/p0-security-authoritative-state`, allocated from
`5521b681fff0ecb50b17c10bc1dd1356cbecc1b6`. Put all build and temporary output
under `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-authoritative-state/`.

## Authority and scope

This is a live allocation: set PF-20-S02 to `in_progress`, run both governance
checkers, then implement. No additional worktree, branch, owner, or G0 decision
is missing.

Literal scope:

```text
codex-rs/config/src/security_state.rs
codex-rs/config/src/lib.rs
codex-rs/core/src/security/authoritative_state.rs
codex-rs/core/src/security/authoritative_state_tests.rs
codex-rs/core/src/security/mod.rs
qa/security-levels/sprints/PF-20-S02/
docs/sprints/current/p0-security-levels/pf-20-s02-protected-authoritative-state.md
```

Do not touch Cargo/Bazel manifests or locks, generated config schemas, ordinary
config persistence, PF-27 probe sources, plan/index/MkDocs, or another sprint.
Hand any genuinely required schema/manifest/shared-file edit to the integration
owner with a focused patch proposal.

## Deliverable

Implement controller-owned authoritative level, grant, revocation/kill
generation, and recovery state distinct from agent-editable preferences. Bind
mutations to the completed PF-27-S03 controller identity contract; never trust a
model-supplied role. Reject untrusted overwrite, delete, rename, symlink,
replacement, permission weakening, stale snapshot, and rollback across restart.

Distinguish a genuine legacy first install from missing state after protected
activation. Implement compare-and-activate revisions, ownership-scoped recovery,
crash-boundary tests, and fail-closed unsupported-platform outcomes. Preserve
PF-20-S01 Permissive/config behavior byte-for-byte where not explicitly extended.

## Verification and review

Run `just fix -p codex-config`, `just fix -p codex-core`, `just fmt`, the full
`codex-config` suite, focused Core config/security tests, PF-27-S03 tamper probes
where applicable, both governance checkers, and `git diff --check`. Use synthetic
state only; never use real credentials.

Use TMUX + Corbanu Terminal + Claude Opus 5.0 Max for independent review. Ask
about first-install ambiguity, symlink/rename/rollback, TOCTOU and crash windows,
identity forgery, stale revisions, ownership confusion, recovery ordering,
unsupported OS fallback, and Permissive drift. Repeat until clean.

Hand back the candidate/base, scope audit, state/revision contract, exact tests
and platform outcomes, limitations, and transcript hash. Integration follows
PF-19-S02; the integration owner serializes schema/shared edits and archives only
after combined-tree probes pass.

