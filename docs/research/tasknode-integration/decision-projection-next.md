# PF-80-S01 — contextual decision projection preparation

Status: **allocated for same-sprint execution; launch ID/HEAD in manager receipt**. September 12, 2026.
Product: **Internal delivery control — TO BUILD**, “Show blockers, rendered
sprints, human test plans, machines, run logs and freshness.” The
[active plan](../../plans/active/initiative-delivery-control.md) and
[decision amendment](../../plans/decision-escalation.md) own approved behavior.

Native validity `bda5b35b4` and accepted main are now in canonical receiving at
`87e31f521672e627e6230d48fc16a4cfaa7ff44c`. The old publication task was declined;
verify its termination before any new sync, but do not freeze local work.
Independent Volta design DEC-001..026 is frozen in the linked sprint's QA record.

## Allocated first slice and launch gate

One native Astra High implementation worker, sequentially reusing
`/Volumes/CorbanuDrive/Corbanu/worktrees/tasknode-pf80-s01-20260911`, branch
`workstream/tasknode-pf80-s01-20260911`. The manager delegates only these paths:

- New `scripts/initiative_control/decisions.py`: strict offline records,
  caller-injected-time validation, atomic fixture persistence and pure projection.
- `scripts/initiative_control/attention.py`: `render_decisions` and only necessary
  shared safe-link extraction; preserve existing manager-notice behavior.
- New `scripts/initiative_control/test_decisions.py`: record/persistence tests.
- `scripts/initiative_control/test_attention.py`: rendering regressions.
- New `qa/initiative-control/pf-80-s01/decision-projection/receipt.md`: evidence.

First integrate the accepted staging candidate into canonical receiving, safely
fast-forward the clean idle worker, record exact base/launch HEAD in active plan
and sprint, replace the frozen validity allocation and run governance/overlap
checks. No new implementation may rely on this draft as its active allocation.
The manager owns shared state/export/source configuration and all later wiring.

Volta01a094c9-1364-7783-9493-9900d25f6fae used fresh context, no tools, code or
results; all DEC-001..026 cases and questions are preserved unchanged. The
integrator authorizes +3 passes for design (used), candidate code and final
independent evidence, retaining earlier usage under root AGENTS.md. Do not
repeat clean reviews. Full UI/feed cases remain required at later wiring.

## Contract and fixture interpretation

Fixture decisions use explicit manager assessment time; more than20minutes
without a newer assessment is stale, matching source freshness. A render does
not refresh it. Invalid/conflicting input rejects with unknown/unavailable state,
never an empty-queue claim. Unknown optional context is explicitly labeled;
required IDs and revision provenance cannot be invented. Retain complete
append-only question/resolution history without pruning in this slice. Links
are limited to approved sanitized published documents, never raw private logs.
Later browser proof targets desktop1440x900 and phone390x844 plus keyboard;
no browser/phone/live-delivery pass is claimed by this offline increment.

One stable manager decision ID with append-only revisions binds exact initiative,
sprint references, owner, raised/updated times, summary, background, impact,
stopped/continuing work, options/recommendation, precise question, context/test
links, status and scoped resolution provenance. Open, acknowledged, resolved and
superseded are separate from delivery status. Acknowledgement never resolves a
question or changes operational authority. Retain the exact question revision
answered and previous records; never silently rewrite an applied decision.

Use strict bounded JSON, canonical digests and explicit source feed ID/revision/
assessment time. Reject duplicate/unknown fields, invalid revisions/time, unsafe
paths, controls and synthetic secret canaries with content-free diagnostics.
Restrict links to the approved published corpus. Exact current and historical
sprint identities must not collide; unresolved links explain missing context.

Persist only publication-safe manager material in owner-only fixture state,
with locked compare-and-swap plus atomic replacement. Exact retries retain the
same identity and assessment time. No last-writer-wins or silent history deletion.
Do not create or migrate the real decision file in this slice. Existing atomic
file helpers require inspection; do not assume they prove power-loss durability.

Pure projection distinguishes missing input (unknown), assessed empty input,
fresh open questions and stale last-known questions. Re-rendering never refreshes
assessment time. Show separate alert status **Slack not connected**. A record or
message receipt is not proof of notification, reading, resolution or agent work.

The unconnected renderer provides a linked one-line summary, expandable details,
owner/options/recommendation and an answerable question. Stable anchors survive
resolution in inspectable history. Escape every field. Keep manager bookkeeping
question-free. Response direction is the manager task until Slack is qualified;
there is no dashboard approval form or executable reply text.

No production caller in this slice: do not modify collect/export/activate/serve,
jobs, services, CLI, native validity, posting flags, credentials or live Slack.
No network or real state mutation, worker commits/pushes/reviews/extra agents.

## Budget and proof

Provisional worker target: 270 record code + 100 renderer + 160 record tests +
110 rendering tests + 35 receipt = 675 changed lines. Hard 800 total/500 non-test,
including new files, additions and deletions; manager allocation edits must be
included in integration accounting. Return actual size for re-slicing if required
validation/history/recovery tests do not fit; no compressed or omitted proof.

Prove new/revised/acknowledged/resolved/superseded records, stable IDs, exact retry,
concurrent CAS conflict, history rewrite/rollback rejection, reload and failure
before replacement preserving last-good state. Cover invalid schemas, timestamps,
references, byte limits, synthetic secrets and safe errors. Compare complete
objects; never derive expected values using the implementation under test.
Render full context and exact links; preserve resolved anchors, stale/unknown
states, historical ID explanations, escaping and question-free operational notices.

Use the existing pinned dashboard venv and focused/full unittest discovery.
The manager audits scope and final hashes, reviews the new candidate within the
preserved allowance, and runs combined dashboard/governance checks. Native tests
are not new-feature evidence for this Python-only slice. No user-interface,
phone, Slack, TUI or human acceptance is claimed from synthetic rendering.

## Sequential follow-ups and human proof

Next qualify the complete sanitized feed export/import/render/health chain with
independent decision provenance; retain Facilities and last-good output. Only
then consider real state/publication wiring. Next come pure Slack parent/thread
payloads, durable alert attempts, verified reply intake and manager handoff routing,
each separately allocated with coherent recovery and tests. Incoming-webhook
setup alone cannot receive replies. Use the already selected private channel;
no new scopes or credentials are authorized by this document.

The eventual human journey is: away from Desktop, see a contextual alert, open
the exact private sprint/decision link, answer its precise question, and see the
verified answer retained and routed to the current agent. Test stale questions,
ambiguous answers, unrelated actors, duplicates, uncertain delivery, restart and
stopped owners. Freeze the independent cases before result disclosure; preserve
all limitations. PF79/PF81 remain dependent drafts and Task Node posting stays OFF.
