# PF-80-S01 — runtime recovery decision handoff

Planning only. Product: **Internal delivery control — TO BUILD**, “Use sequential
sprints per initiative”. No new credential, read, send, enrollment or release
authority. Native reconciliation `177ec93fc` is independently reviewed and included
in combined `3898eaa658924361b94cdbfc6751f930550ddee4`; all 73 native Task Node
tests pass there. The offline preparation/engine/adapter/observation chain now
exists. No further test-only wrapper is a justified substitute for runtime design.

## Concrete gates and owners

Both directions were approved by Travis in this task on September 11 local /
September 12 UTC; they are no longer human blockers:

- Setup direction: manually operated publisher on this Mac, existing IridiumMaster /
  @iridiumeagle account, goal-only updates to the existing Proposed PF-80 task;
  credentials stay off Alex's server and worker agents. Recommend this direction.
- Native validity: design a reviewed path for revocable sessions with no expiry,
  using fresh native identity/account/profile/origin and revocation checks, without
  inventing a TTL. Recommend this design direction; current ExpiryUnknown stays held.

Do not repeat these questions. These explicit answers approve
design/setup only, not secret access, enrollment or the exact first post. Existing
account creation approval is not revoked or requested again. The known historical
target is `task_789a0f3bd75b41d1eca20cae698f04cf`, not a live ownership/status check.

Manager next action: scope a reviewed operator-accessible recovery
contract and exact implementation allocation, then prepare qualification inputs.
The manager owns record/input/retention and restart/fence design, exact executable
identity, current account/profile/origin, target lifecycle/audience/entitlement and
one immutable event packet. Unresolved manager design is not work delegated to Travis.
Request any genuinely new authority only when its exact packet is reviewable.
No worker is currently assigned native writes; the last reconciliation scope is frozen.

## Proposed next behavior — DRAFT, not executable

Prefer a narrow CLI entry for one retained uncertain goal. No command syntax is
claimed to exist. Bind original immutable event bytes/full ID/approval digest,
workspace/task/source timestamps and invocation/profile/account/origin fence.
Keep consumed attempt and original uncertainty across cancellation/restart; later
observations are separate evidence, not a reset or retry authorization.
Explicit operator action may eventually perform one exact-ID activity GET under
the accepted validity/read contract. Preserve 404 uncertainty and compare all
original fields with separately validated server annotations. Invalid/missing
records hold before native access. No ambient queue scan, new event ID, POST,
resend, automatic enrollment or task-completion claim.

Human test plan, executable only after exact candidate/scope approval: selection
and visible immutable identity; malformed input held before access; cancellation;
identity rotation; ambiguous 404; exact observation; restart retaining original
record/uncertainty. Test through the supported entry point, without hidden-state
repair. Existing fixture matrices support, but do not replace, this runtime proof.

## Actual controls and known trap

Native `tasknode ... link status` is local state; `tasknode ... status` may resolve
pending linking and query the server. CLI `link cancel` clears pending linking,
not an uncertain event. TUI `/tasknode link` starts linking; there is no TUI
`/tasknode link cancel` or native exact-event get/send/retry command today.
Neither relink nor successful status guarantees a dated expiry; server-issued
sessions normally have null expiry. Do not invent recovery instructions.

Paused tracker recording still drains pending events on the linked TUI's timer.
Do not launch the linked production TUI as a read-only probe. Later interactive
qualification requires independently qualified isolated scope without inherited
queued/enrolled tracker state; otherwise record blocked. Do not substitute Sync
now, bulk flush, task-request retry, logout or encrypted-state shell repair.
Source provenance/control details remain in the accepted native reconciliation
allocation and pinned first-party source record. No live checks were run here.

PF-80-S01 retains its reserved slot during manager-owned allocation preparation;
PF-79 beta/PF-81 remain dependent drafts. Dashboard recovery-source publication is
separate and unchanged. No main/push, source cutover, human or release acceptance.
