# PF-22-S02 protected-runtime handoff

Owner, branch, worktree, base and literal scope are authoritative in the sprint
front matter. This is a product-initiative implementation under the active P0
security plan.

## Deliverable

Compose the completed PF-19 revocation fence, PF-20 authoritative state,
PF-21 compatibility oracle and PF-41 durable event/recovery contracts into one
fail-closed protected-runtime seam. Preserve PF-22-S01 explicit-child and
auxiliary/Guardian inheritance. Do not implement an ingress adapter, credential
broker, TUI or protected activation.

Prefer a cohesive new `protected_runtime` module over growing the existing
large authoritative-state module. The runtime view must bind configured,
creator-required and effective containment; owner/policy/run/revocation/kill
generations; measured backend/identity readiness; expiry; audit recovery; and
unknown ingress/egress route handling. A stale or unsupported prerequisite
blocks the affected operation and cannot silently downgrade.

## Scope discipline

PF-22 owns `core/src/security/mod.rs`, Core manifest/lock edits and the exact
effective-policy/agent-hook files listed in its sprint. PF-27 may not edit those
surfaces. Add agent hook changes only when the implementation proves they are
required; avoid unrelated Core cleanup.

The seam manifest records exact upstream symbol/revision, Corbanu owner,
semantic contract, regression command and last tested revision. Unverified
entries remain pending rather than becoming filename-only claims.

## Proof

- `cd codex-rs && just fix -p codex-core && just fmt`
- focused `codex-core` effective-policy, inheritance, protected-runtime and
  authoritative-state tests
- `just test -p codex-security-policy revocation`
- `just test -p codex-security-audit`
- seam checker CLI and Python unit tests
- plan/sprint checkers and final PF-21 compatibility comparison
- supporting TMUX smoke and TMUX/Corbanu/Claude Opus 5 Max review

Record exact test counts and evidence under
`qa/security-levels/sprints/PF-22-S02/`. Do not run remote Windows/Linux final
checks until instructed. Hand back one committed candidate; do not merge or
push `main`.

