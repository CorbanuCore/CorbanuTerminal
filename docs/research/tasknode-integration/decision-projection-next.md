# PF-80-S01 — contextual decision projection preparation

Status: **prepared, not executable or dispatched**. September 12, 2026.
Product: **Internal delivery control — TO BUILD**, “Show blockers, rendered
sprints, human test plans, machines, run logs and freshness.” The
[active plan](../../plans/active/initiative-delivery-control.md) and
[decision amendment](../../plans/decision-escalation.md) own approved behavior.

Native validity `bda5b35b4` is independently reviewed and tested in staging.
Latest accepted main is reconciled at `64cc30cf19a25f74ae07125eece91f6f766e8ebf`;
322 combined native tests and 83 dashboard tests pass. This is not publication
or canonical receiving acceptance. The older source command remains pending
approval and new Facilities edits require ownership reconciliation.

## Proposed first slice and launch gate

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

Before UX dispatch, freeze an independent intent-only design with fresh context
and preserved original cases. Reconcile existing review usage and reserve design,
code review and evidence-check capacity without resetting any exhausted track.
The code-informed preparation proposal is not that independent test design.
No new review invocation or allowance is recorded by this preparation document.

## Contract to finalize before allocation

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
