---
title: "Persistent linked sessions"
status: active
change_class: product-initiative
priority: P0
owner: "Jim Ricketts"
activation_authority: "Final product authority through direct user instruction"
activation_basis: "The user explicitly required all authenticated linked sessions to remain valid without time-based expiration."
target_release: "Next Corbanu candidate"
deadline: 2026-08-29
created: 2026-08-29
updated: 2026-08-29
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "Shipping MVP — LIVE"
  requirement_excerpt: "Task Node and identity: Tasks, evidence, verification, rewards, balances, chat, context, linked identity, and live Task Node-linked Nostr identity."
implementation_worktrees:
  - path: "/home/pfrpc/repos/CorbanuTerminal-session-persistence"
    branch: "codex/session-persistence"
    base_commit: "1bdc515bff48a4d9048dae7d06c6214e884265bc"
---

# Persistent linked sessions

Policy: repository-root `AGENTS.md`

Plan lifecycle: `docs/plans/index.md`

## Activation record

| Field | Value |
| --- | --- |
| Status | **Active** |
| Active-plan slot | **2 of 2** |
| Product authority | Final product authority through direct user instruction |
| Authoritative decision | Authenticated linked sessions do not expire with time |
| Target release | Next Corbanu candidate |
| Deadline | **2026-08-29** |

## User pain

A stored, previously proven GitHub-linked credential can become unusable solely
because an old timestamp passed. The terminal then blocks status, tasks,
rewards, chat, and context and demands a browser relink even though the user
never logged out and the server-side credential was never revoked.

## Product intent and ideal flow

Once linking succeeds, the encrypted local credential remains usable until the
user explicitly unlinks it or the server revokes it. Legacy credentials with
old expiry metadata resume automatically. A pending one-time browser link can
still expire safely without disturbing an active credential.

## Product linkage

| Field | Value |
| --- | --- |
| Exact product-spec heading | **Shipping MVP — LIVE** |
| Requirement excerpt | “Task Node and identity: Tasks, evidence, verification, rewards, balances, chat, context, linked identity, and live Task Node-linked Nostr identity.” |
| Product outcome advanced | Linked identity behaves as a durable native terminal capability |
| North-star criterion advanced | Persistent identity remains available for the full trader workflow |

## Scope

### In

- Treat a proven active terminal credential as valid until server revocation.
- Ignore historical `expiresAt` metadata when resolving local active state.
- Keep explicit unlink and server-side revocation effective.
- Keep one-time browser-link attempts short-lived and separate from active state.
- Add focused storage, CLI-resolution, and TUI regression coverage.

### Out

- Changing OAuth state, polling-request, wallet-signature, or approval challenge lifetimes.
- Weakening explicit logout, unlink, token revocation, or vault protections.
- Changing unrelated provider-account authentication.

## Invariants

- Local wall-clock time never invalidates an authenticated linked credential.
- A server rejection remains authoritative and never silently grants access.
- A pending or failed replacement link never overwrites the active credential.
- Tokens remain encrypted at rest and absent from diagnostics.

## Ownership and implementation worktrees

| Owner | Worktree | Branch | Base commit | Scope |
| --- | --- | --- | --- | --- |
| Jim Ricketts | `/home/pfrpc/repos/CorbanuTerminal-session-persistence` | `codex/session-persistence` | `1bdc515bff48a4d9048dae7d06c6214e884265bc` | Client compatibility, tests, TUI proof |

## Useful code references

| Path or symbol | Why it matters |
| --- | --- |
| `codex-rs/tasknode-session/src/lib.rs::ActiveSession` | Owns encrypted local session compatibility |
| `codex-rs/cli/src/tasknode_cmd.rs::require_active_session` | Gates CLI access before a server request |
| `codex-rs/tui/src/chatwidget/tasknode_menu.rs::ensure_tasknode_session` | Gates interactive access and renders auth state |
| `server/db/migrations/124_terminal_sessions_persistent_until_revoked.sql` | Existing server contract for new credentials |

## Sprint execution map

| Feature ID | Current sprint records | Completion evidence |
| --- | --- | --- |
| PF-27 | [PF-27-S01](../../sprints/current/persistent-linked-sessions/pf-27-s01-ignore-legacy-session-expiry.md) | pending |

## Acceptance flows

| Flow | Starting state | User action | Expected visible result | Pass criterion |
| --- | --- | --- | --- | --- |
| Primary success | Active encrypted credential with past legacy expiry | Open linked-service menu and run Status | Linked identity and current status load | No relink prompt; authenticated request succeeds |
| Failure/cancel | Token explicitly revoked or user unlinks | Run Status | Clear link-required state | No protected data is returned |
| Recovery/resume | Active credential plus abandoned pending link | Restart and run Status | Active credential wins; pending attempt is optional | Existing authority is not overwritten |

## Implementation sequence

1. Restore every unrevoked historical server credential to the until-revoked contract.
2. Remove local time-based rejection of active credentials.
3. Run focused Rust, server, and true-TUI qualification.

## Automated evidence

| Check | Final-tree command | Result | Artifact |
| --- | --- | --- | --- |
| Session crate | `cd codex-rs && just test -p codex-tasknode-session` | pending | pending |
| CLI/TUI affected tests | focused `just test` commands | pending | pending |
| Server repository | `npm run terminal-auth-repository-smoke` | pending | pending |
| Plan and sprint records | `python3 docs/plans/check.py && python3 docs/sprints/check.py` | pending | pending |

## True-TUI evidence

| Flow | Candidate binary | Test repo/worktree | Keys/actions | Visible checkpoints | Result | Artifact |
| --- | --- | --- | --- | --- | --- | --- |
| Legacy expired credential | pending | exact worktree above | Open menu, run Status | Linked status without expiry warning | pending | pending |
| Explicitly revoked credential | pending | exact worktree above | Open menu, run Status | Link-required recovery | pending | pending |

## Live-repository applicability

| Repository | Applicable to this initiative? | Resolved checkout/test worktree | Base commit | Reason or result |
| --- | --- | --- | --- | --- |
| TensorCash | no | N/A | N/A | Identity-session resolution is repository-independent |
| Isometric Game | no | N/A | N/A | No visual or workspace-specific behavior changes |

## Human acceptance

| Tester | Date | Candidate version/commit | Flow | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| Named human tester required | pending | pending | Linked status and explicit unlink | pending | pending |

## Documentation

| Finished-feature doc | Product-spec citation present | Verified candidate |
| --- | --- | --- |
| `docs/authentication.md` | pending | pending |

## Dependencies, decisions, and blockers

| Item | Type | Owner | Needed by | State / decision |
| --- | --- | --- | --- | --- |
| Server historical-session restoration | integration dependency | Lead developer | TUI qualification | in progress |
| Human acceptance | hard release gate | Named human tester | release | pending |

## Release linkage

- Release record: `qa/release/<version>/`
- Benchmark tracker row: not due for this focused identity repair
- Remaining blocker: true-TUI proof and named human acceptance

## Completion

- [x] Product linkage, scope, invariants, and worktrees are current.
- [x] Every implementation unit is represented by a valid single-feature sprint.
- [ ] Required final-tree automated evidence passes.
- [ ] Required true-TUI and live-repository evidence passes.
- [ ] Human acceptance passes.
- [ ] Finished documentation matches the candidate.
- [ ] Release and benchmark records are linked.
- [ ] No hard release gate remains pending.
