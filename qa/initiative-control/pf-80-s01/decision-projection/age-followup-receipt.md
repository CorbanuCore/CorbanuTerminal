# PF-80-S01 F01 age follow-up — uncommitted worker receipt

Bounded fix within the active PF-80 plan, sprint PF-80-S01 `in_progress`.
Product: **Internal delivery control — TO BUILD**, “Show blockers, rendered
sprints, human test plans, machines, run logs and freshness.”
Authority: `docs/research/tasknode-integration/decision-age-followup.md` and the
user's five-file mandate. Corbanu skill routed policy/spec/sprint checks; the
explicit allocation leaves plan ledgers, review, browser proof and publication
with Codex management. No delegation, model review, commit or push performed.

Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/tasknode-pf80-s01-20260911`.
Branch: `workstream/tasknode-pf80-s01-20260911`.
Clean launch/unchanged HEAD: `3ba7b1211248032fef770567bf86fe2602a44162`.
Allocation base: `adb36c528153247e252cb90bb23050298f8ebbe0`, verified ancestor.

## Change and evidence boundary

`attention.py` emits one escaped `data-raised-at` span for the oldest currently
open/acknowledged decision's immutable first raised timestamp. Resolved and
superseded history cannot determine the age. Empty/unknown input has no target.
`status.js` updates only that generated span on initial load and every existing
30-second health tick, before fetching health. It floors actual elapsed browser
minutes; invalid/future browser timestamps show age unknown. Timers may be
throttled by browsers; this is no promise of background wall-clock scheduling.
The stale count changes its prefix text node without removing the age span.
Raised/context/assessment dates, user text, links and stale qualifications survive.

Added deterministic Python state/oldest-selection cases and an embedded Node
test of the actual fixture export/activate/render/publish fragment and published
JavaScript. It covers initial old snapshot/reload, 59,999/60,000/60,001 ms,
subsequent minutes, freshness threshold, stale-on-load and transition, immutable
dates, user text/link labels, invalid/future age, healthy/failed/offline/HTTP-error
and indefinitely pending health. Existing unknown/empty/history cases remain.
The DOM harness is supporting evidence, not an actual browser execution.

Final verification: 122 Python tests passed in 25.710s, including both embedded
Node checks; separate Facilities Node regression passed (one file-level test).
Plans passed (3/3 active); sprints passed (115 current/126 archived); diff check passed.
Pinned interpreter: `/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/bin/python`.
Commands: `PYTHONDONTWRITEBYTECODE=1 <python> -B -m unittest discover -s scripts/initiative_control -p 'test_*.py'`;
`node --test scripts/initiative_control/test_facilities_js.cjs`;
`<python> -B docs/plans/check.py`; `<python> -B docs/sprints/check.py`; `git diff --check`.
All data mutations in tests use disposable synthetic state; activation's service
calls are mocked despite its printed startup message. Full discovery includes
the existing temporary loopback HTTP fixture; no actual dashboard, remote
network, Slack, credentials or services were accessed or changed.

## Preserved independent finding and parent handoff

Original `72854ec77` browser packet remains the original candidate only:
24 frozen cases supported, DEC021 partial redaction/history evidence,
DEC025 advisory scanning gap, all 714 evidence hashes checked by the parent.
F01 independently confirmed frozen oldest-raised minutes across time/reload;
existing stale transitions worked. No old reviews or original browser cases repeated or
reclassified here. DEC001..026 remain unchanged in `design-proposal.md`, SHA-256
`c4f95f0123b1a28603a3ec67214d029e597a20fb1d5d46853a77c23ecad89c6d`.
Follow-up mandate SHA-256: `dce3da3749c5587190f4f45a01806570900174b1df3c3c29e18deb17f7212b42`.
Private original locations remain `.codex-work/volta-dec.NOKrd0` and
`.codex-work/decision-evidence.qVSn2p/verdict.md`; not reread as a new review.
Facilities phone-table observation remains separate; no Facilities edits.

Parent owns one new F01 code review and one targeted independent evidence
recheck under the recorded extension, preserving all previous review spending.
Execute the exact final package on desktop/phone/keyboard: initial stale reload,
cross a minute with failed/disconnected health, retain last-known count and dates,
resolve the oldest question, inspect retained history and targeted DEC021 fields.
DEC025 stays advisory. Human acceptance is absent; no human-ready claim.
Native/TUI and TensorCash/Isometric proof are not applicable to this static
projection increment under the plan. No release/benchmark qualification claimed.

## Exact next allocation proposal — offline Slack sender/reply contract

Manager allocates after accepting F01, in the same PF-80-S01 reservation and
worktree, with a newly recorded clean launch/base. This is preparation only.
Proposed literal writes, all new files (not created by this worker):

- `scripts/initiative_control/decision_alerts.py`
- `scripts/initiative_control/decision_replies.py`
- `scripts/initiative_control/test_decision_alerts.py`
- `scripts/initiative_control/test_decision_replies.py`
- `qa/initiative-control/pf-80-s01/decision-projection/slack-contract-receipt.md`

Build one offline injected-transport sender and verified-event reply processor;
no production entry, new scheduler, auth store, export/schema change or live call.
Read/reuse `decisions.py` validation/canonical digest/append-only revisions and
`save_fixture` locked CAS for manager-approved resolutions. Reuse inspected
`control.py` file helper conventions, adding owner-only/no-symlink validation and
directory fsync locally where needed; `atomic_json` alone lacks directory fsync.
`tasknode.py` provides stable enqueue/index/lock patterns, but its batch `flush`
and automatic transient retry are unsuitable for uncertain Slack delivery.
Keep a separate Slack ledger; never migrate/replay the Task Node/PF-76 outbox.

Read-only native references: `codex-rs/tasknode-session/src/lib.rs` SessionScope,
`client.rs` identity fence, `delivery_send.rs` consumed attempt/binding and
`delivery_reconcile.rs` exact observation. These are Task Node contracts, not
Slack credentials or a Slack-capable client. Preserve explicit default versus
named native profile for eventual manager routing; do not call native resolvers.

Bind each immutable alert to feed ID, decision ID, presented question revision,
canonical payload digest, event kind, team/channel/app/bot identity and credential
generation supplied by the future supported connection. Persist parent message
timestamp before its details/question thread send; each phase has its own stable
dedup identity. Lock send ownership and durably record attempt before exchange.
Track pending/sending/sent/failed/uncertain separately from decision status.
Crash-after-attempt or timeout remains uncertain across restart; do not resend
or infer success from a missing lookup. Bound explicit recovery to the same
identity/payload and exact receipt evidence; no exactly-once transport promise.

Intake accepts injected already-verified envelopes only from the pinned team,
channel, thread and immutable human-owner ID. Durably dedup event ID plus message
identity before processing acknowledgment. Retain revision binding, actor/time,
answer and edit/delete audit events. Old/conflicting/ambiguous/resolved questions
need clarification; never execute reply text. Manager validates scope, then CAS
records resolution and a durable handoff to the current owner/allocation.
Track received/needs-clarification/recorded/queued/delivered/agent-acknowledged;
stopped owners remain pending-manager-dispatch. Use a retained reply ID to resume
across the resolution/handoff crash window without duplicate logical dispatch.

Recorded setup only, from `docs/plans/decision-escalation.md`: AmbientCrypto
team `T074X5KNENT`, private “The Corbanu Project” channel `C0C0X2ELFKR`,
Corbanu Decision Alerts app `A0C1CC2P4SE`, incoming-webhook only, unconnected.
Two-way setup still needs verified consent/reinstall for bot `chat:write` and
`groups:history`, `message.groups` subscription, Socket Mode app token with
`connections:write`, bot membership/identity and Travis's immutable Slack user ID.
Bot/app credentials and credential generation are not supplied or verified;
no token was read and no connection readiness is invented. Keep the bot in only
that private channel and enforce team/channel allowlists regardless of scopes.
The requested scopes are the repository's proposed contract, not a live audit.

Synthetic-first tests: new/revised/resolved parent/thread payloads, safe remote
decision links and redaction/mention escaping; duplicate/concurrent sends;
partial-thread failures, timeout and crash/restart; identity rotation/cancellation;
wrong actor/team/channel/thread; old/out-of-order/duplicate/edit/delete events;
ambiguous answers, CAS conflicts, stopped/stale owner and handoff restart dedup.
Use fake transport/credentials/envelopes and private temporary fixtures only.
Suggested bounded target: 400 total/250 non-test; report measured expansion to
integrator before widening. Shared registration/production wiring remains parent work.

First-live gate, later and parent-owned: reviewed exact candidate, supported
connection/scopes/identity verification, owner-only credentials, accessible
authenticated remote decision URL, and one harmless pinned question/payload.
Observe one parent/details delivery, Travis's real thread reply, receiver restart,
dashboard resolution and logical routing to the current agent with acknowledgment.
Retain every delivery/reply receipt and uncertainty; no bulk flush or Task Node
acceptance/reward/financial action. Device notification/read state is not inferred.
UI evidence gaps do not prevent this independent offline successor allocation;
PF-79/PF-81 remain dependent drafts and live posting remains OFF.

## Candidate identity and measured scope

Only the four allocated implementation/test files above plus this receipt differ.
Implementation/test patch SHA-256 (`git diff --binary --` those four files):
`4d7e976cba5820bf91940d9a27e53e4826c4f28406d2da9c393a0695672ae36e`.

| File under `scripts/initiative_control/` | SHA-256 |
| --- | --- |
| attention.py | `6737bda4678443905cb7d65aea4573a75454c37a50c81da203d56ff0b0069073` |
| status.js | `88f9fcfbd8908c334ef120c811811dfba67ffd3b217c6957b6b9eb0f8a323f71` |
| test_attention.py | `a42021b051c31d49bf9b82b3f6def1fe0d59449d3c108eec7a0cf87cec19de52` |
| test_decision_feed.py | `8b2728522cbb947c835ab0736dfe9f7ab26d7964043829e1ff96730f174e746e` |

Final receipt hash and complete five-file diff hash are reported after this file
is finalized, avoiding a self-referential digest. Measured expansion: 315 total
changed lines, 179 non-test including this 165-line receipt (additions + deletions).
Target 400/250; overage zero. Return uncommitted for parent review and integration.

## Integrator correction — September 12, 12:28 UTC

The preceding candidate hashes/results remain original evidence. First scoped
Astra High review exited1 with P2: age ID `decision-oldest-age` collided with
legal record ID `oldest-age`, redirecting its permalink or erasing closed history.
Parent verified the renderer and JS lookup, classified this as an in-scope
introduced blocker and changed only the allocated five-file scope. The generated
age ID is now `oldest-open-decision-age`, outside the permanent `decision-*`
namespace. Inspected other new IDs in this diff; no other new collision target.
Extended the actual-rendered-fragment JS regression with that legal record ID
in open/resolved/superseded states: unique detail ID, actual permalink opening,
retained children/history and independently advancing age are checked.

Integrator grants one additional corrective code review, retaining the original
F01 code pass and all earlier design/code/evidence usage; no budget reset. Browser
executor paused and acknowledged no fixture/package/browser capture yet. It will
use corrected frozen source and retain the original cases plus this regression.
Original private review: `.codex-work/manager-continuation.9Id1V1/f01-age-review.json`.
Corrected four-file patch SHA-256:
`3b037ddf46d0aaacdc82ae97d63470cae841fac54812650f4711b535dffe68ed`.
Final tests/review/browser disposition are recorded by the integrator separately;
this amendment does not assert a new pass or human-test readiness.
