# Standalone bounded fix — scope and acknowledgement gate

Recorded 2026-09-14 UTC before any source/test edit. Outcome: **source implementation stopped at the existing acknowledgement seam**, as explicitly instructed by the parent. This is not a runtime-fixed or human-ready candidate.

## Authority and coordinates

- Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/live-permission-transition-20260913`.
- Branch: `fix/live-permission-transition-20260913`.
- Verified HEAD/base: `005cc644f59b1e762e5497b329e106c67925d4ed`.
- Verified ancestor of HEAD: `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`; canonical provenance is retained in the prior manager checkpoint.
- Parent accepts a standalone **bounded fix** for requested/confirmed UI state and honest next-turn scope using existing applied acknowledgements. No authorization semantics, protocol shape, persisted config contract, or Core lifecycle changes are authorized.
- Fresh Fable decision receipt `/private/tmp/perm-manager.SZvxXN/f-rxuuo_ha/receipt.json`: `status=completed`, `shutdown.clean=true`, `forced=false`, `session_gone=true`, `errors=[]`. Its suggestions are advisory and subordinate to the parent's current instructions.
- Updated security-owner disposition supplied by parent: **APPROVE standalone bounded fix only**; no children or active broker collision. PF27 remains paused/reserved. This is coordination, not code review, acceptance, or global clearance for others. No further owner wakeup or reservation transfer is needed for this bounded task.
- The parent's nonblocking next-turn versus mid-turn preference question does not revoke this bounded authority.

## Supersession and product citation

`allocation-proposal.md` is explicitly superseded as **historical, rejected broad allocation evidence**. Its global-quiescence design and PF22S03 allocation are not executable instructions. The prior README, source findings, verification and manager checkpoint remain preserved originals; this additive record governs the present worker. No PF27, plan, or sprint reservation file is edited.

Product specification: `docs/corbanu-product-spec.md`, exact heading **Live MVP versus the P0 security controls**: “The shipping MVP already has a wallet, vault, scoped signing, approvals, and general workspace sandboxing. These are live product capabilities.” Also **Shipping MVP — LIVE**, Workspaces: “approvals, existing general sandboxing”. The repair would restore truthful presentation of those existing capabilities. It does not implement the separate `/security` immediate-transition promise or change Permissive.

## Exact edit scope

The maximum allowed implementation scope is six literal source/test paths and approximately 200 non-test changed lines. Inspection found the explicit stop condition before a feasible source edit set could be authorized. Consequently the final executable scope is:

- Source/test paths to edit: **none (0 of 6)**.
- Estimated and actual non-test source LOC: **0 of 200**.
- Actual test LOC: **0**.

This is an insufficiency return, not a conditional six-file architecture proposal. No unlisted source work is implied. A later parent decision must establish a feasible exact source/test set before implementation resumes.

Documentation/evidence is separate and consists only of these new files:

1. `qa/reliability/live-permission-transition-20260913/bounded-fix-scope.md`
2. `qa/reliability/live-permission-transition-20260913/bounded-fix-checkpoint.md`

## Smallest unresolved seam

The TUI API has a session-state notification, but no delivered per-selection applied result. A queued RPC response cannot establish application. Thread identity and matching permission values cannot establish which selection completed; even matching the whole snapshot cannot distinguish a delayed earlier identical state. The failure path also lacks a mapping back to the TUI selection. See the checkpoint for exact source evidence.

The required transition is a selected request becoming confirmed, failed, or superseded based on its own applied outcome. That cannot currently be proven using the delivered API without crossing the allowed TUI boundary. Do not invent correlation from timing, notification counts, matching values, a timeout, an idle flag, or a later model-setting update. No Core/protocol/app-server mutation is authorized here, and no new architecture is proposed.

Current-turn approval/sandbox snapshots must remain distinct from selected session settings. Shared MCP/network refresh means even a blanket claim that *all* permission effects wait for the next turn would be too broad. Pending approvals, ongoing commands, and process lifetime remain untouched. No automatic approval, interruption, replay, current-instance change, or global-default change is permitted.

## Evidence and later gates

Existing tests may be inspected or run selectively with Rust 1.95.0, `just test`, locked/offline dependencies, and the private target in this worktree. Any such result is implementer supporting evidence only. There is no candidate for independent Fable code review or enforced code-blind execution yet. Original F01–F11 remain frozen and unexecuted; no missing or out-of-scope case is waived by this worker.
