# September 10 call follow-up — planning amendments

Status: **draft planning updates approved by the user; no implementation,
allocation, deployment, live Task Node write or financial action authorized.**
This amends the [portfolio](portfolio-2026-09-09.md) and
[scrum packet](scrum-2026-09-10.md), not the current active-plan policy.

## Source and decisions

Source `SRC-ALEX-20260910-02` is the latest private call. Alex's opening voice
was confirmed by the user; automatic transcription and short-turn attribution
still have uncertainty. Only curated product requirements are reproduced.
The recording, raw transcript and unrelated personal discussion stay private.
Vendor names, prices, determinism and regulatory claims are unverified.

| Discussion anchor | Amendment | Owning plan / sprint |
| --- | --- | --- |
| 02:09–03:20 | General-purpose index API, then optional creator claims, fee sharing and leaderboards | PF-70 S01–S05; one initiative with sequential phases |
| 03:32–06:16 | Inspect Alex's existing transcript/SEC/price caches; qualify replayable input packets and inference | PF-70 S01–S02; exchange relevant findings with PF-65, without coupling privacy and determinism |
| 00:00–02:09; 45:00–46:07 | Instrument sourcing, entity eligibility and onboarding diligence | PF-68-S01; no account/entity creation |
| 06:16–06:40 | Maintain Corbanu API docs as models and capabilities change | PF-64-S02; API delivery also owns its own verified docs |
| 65:23–67:26 | Separately owned SSD, materials and Apple/hedge research | Research ticket register below; no trading or publishing |
| User clarification; earlier source 93:55–94:45 and 106:46–108:45 | Reuse native Task Node and Campaign Tracker; inspect the described long-running operator before proposing another manager | PF-75 S01–S04 and PF-72 S01–S03 |

The explicit Campaign Tracker/operator passage located so far is in
`SRC-ALEX-20260909-01`, the earlier call. The user's subsequent clarification
identifies `/tasknode`; code inspection confirms it independently.

## Native Task Node boundary

Source baseline inspected: `6ca801add7eeda9feb9723e7f4935fbf6b29a73b`.
This is source evidence, not current installed-build or account acceptance.

| Existing boundary | Reuse / remaining work |
| --- | --- |
| `codex-rs/tui/src/chatwidget/tasknode_menu.rs` and `slash_dispatch.rs` | Human `/tasknode` workflows; retain reachable linking/recovery controls |
| `codex-rs/cli/src/tasknode_cmd.rs` | JSON task/request/context/evidence/verification operations; no generic progress or tracker subcommand currently exposed |
| `codex-rs/tasknode-session/src/{lib,recovery,client,commands}.rs` | Reuse profile/account/origin-bound session resolution and existing request recovery; do not invent another login store |
| `codex-rs/tasknode-session/src/tracker.rs` | Existing encrypted, bounded event outbox; audit retry identity and ownership before sharing responsibilities |
| `codex-rs/tui/src/chatwidget/campaign_tracker.rs` | TUI capture/replay; events initially have `taskIds: []` and `coverage: observed_tui`; explicit task mapping and other-harness coverage remain work |
| `qa/campaign-tracker/2026-09-07-verglas/README.md` | Prior live pilot evidence with stated limits, not qualification of every worker machine |

PF-75 owns operational mapping and acceptance rehearsal. PF-72 owns outcome/cost
attribution, not another tracker. Basic mapping/capture/progress does not depend
on PF-60; spend-derived ROI in PF-72-S02 does require PF-60-S01.

The separate private delivery-control adapter already uses the Campaign Tracker
endpoint for mapped `kind: goal` events. It is not a general evidence client.
Reconcile its credential and outbox ownership with the native stack; keep only
the thin capability gap, rather than blindly replacing it with a nonexistent
CLI command. Its source/deployment is outside this main-line planning packet.

Contract requirements for PF-75:

- One manager-owned mapping binds initiative, sprint, Task Node task, native
  run, machine, repository and candidate commit; reject unmapped/wrong-account work.
- Keep execution, activity/progress, submitted evidence, human acceptance and
  server verification/reward distinct. A summary or evidence receipt is not
  completion; the manager cannot accept its own work.
- Workers report separately; one publisher renders the private HTML view,
  sprint Markdown and human plans about every half hour. Show source time,
  publication time, stale/blocked delivery, machine and coverage, not invented progress.
- Reuse the intended native profile; Safari login alone is not native auth.
  Never give worker agents broad publisher credentials or silently borrow another account.
- Qualify TUI, standalone CLI, Desktop and remote-worker capture separately.
  Missing capture is visible, not zero effort. Enrollment is not a sharing grant.
- Persist report identity; reconcile uncertain writes before retry. A new CLI
  invocation may create a new idempotency key. Test offline/restart/duplicate,
  expired/pending-link/cancel, wrong-profile and revoked-permission cases.
- Rehearsals are dry-run by default. Live posting needs an explicit destination,
  permitted action/payload, account, consent, budget and named-human acceptance gate.

## Research ticket register

These are recorded call assignments, not agent dispatches, new product
initiatives, orders or publication instructions. All remain **not started**;
timing, source access, review owner and budget need confirmation.

| Ticket | Call-assigned owner | Deliverable and acceptance |
| --- | --- | --- |
| R-20260910-SSD | Travis | SSD exposure research memo: candidate universe, dated primary evidence, valuation/supply-demand assumptions, counterthesis and rejection criteria; no preferred purchase presumed |
| R-20260910-MATERIALS | Alex | Materials/robotics exposure memo: value-chain map, bottleneck evidence, substitutes, cyclicality and counterthesis |
| R-20260910-HEDGE | Alex | Apple/hedge memo: quantify the local-AI-beneficiary counterargument, compare the hedge hypothesis with alternatives, and allow a no-trade conclusion |

Use PF-74's source/claim/independent-review rubric. Its existing S02 remains
**one** bounded exemplar packet, selected from this register; it does not
silently authorize all three or force private research through public distribution.
Any later agent assistance needs its own bounded assignment.

Parking lot, unassigned: high-purity silicon supply, satellite/mobile spectrum
and emerging memory hardware claims. No extra initiative is created per idea.

## Sequential scope and gates

PF-70 expands to five draft sprints: inspect prototype/API contract → replay and
paper evidence → product decision → flag-gated API candidate → optional creator
economics/ownership/leaderboard decision. Creator implementation and payouts
require a later approved amendment; they are not hidden inside the API sprint.

There remain 16 portfolio proposals, now 52 draft sprints. This does not create
a fourth operating lane. The desired ceiling is three independent initiatives,
one sequential sprint each; existing main policy/allocations still require the
separate reconciled transition in the scrum packet.
