# PF-80-S01 — real Slack transport and manager handoff

Manager allocation, September 12. Product initiative under active
`initiative-delivery-control`, PF-80-S01 in progress. Product heading:
**Internal delivery control — TO BUILD**, “Show blockers, rendered sprints,
human test plans, machines, run logs and freshness.” No fourth initiative.

## Accepted predecessor and coordinates

Offline recovery730577f1d is integrated at9877c058d794db7cd2d611be7970e8c75db1422e.
Corrective review04 is clean; receiving169 Python tests, Facilities and governance
pass. Original1151/1274/1412/1506 sizes and failed reviews remain in the
[receipt](../../../qa/initiative-control/pf-80-s01/decision-projection/slack-contract-receipt.md).
Live delivery, credentials, phone access and actual agent ACK are unqualified.

Worker: James, `/Volumes/CorbanuDrive/Corbanu/worktrees/tasknode-pf80-s01-20260911`,
branch `workstream/tasknode-pf80-s01-20260911`; manager receiving is
`/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911`, branch
`integrate/management-workstreams-20260911`. Both retain recorded plan base
0368ed402a9dac4276b427313b9d7335670ee91c. Manager records actual clean launch
after accepting this allocation; one worker, no overlap with accounting/security.

## Exact ownership and size

Paths are repository-relative to the owner's checkout. No other paths allocated.

| Owner | Path | Estimated changed lines |
| --- | --- | ---: |
| Worker | scripts/initiative_control/slack_transport.py | 300 |
| Worker | scripts/initiative_control/decision_manager.py | 240 |
| Worker | scripts/initiative_control/decision_alerts.py | 35 |
| Worker | scripts/initiative_control/test_slack_transport.py | 330 |
| Worker | scripts/initiative_control/test_decision_manager.py | 300 |
| Worker | scripts/initiative_control/test_decision_alerts.py | 35 |
| Worker | qa/initiative-control/pf-80-s01/decision-projection/slack-live-receipt.md | 80 |
| Manager | scripts/initiative_control/control.py | 25 |
| Manager | scripts/initiative_control/decision_feed.py | 45 |
| Manager | scripts/initiative_control/attention.py | 15 |
| Manager | scripts/initiative_control/test_decision_feed.py | 60 |
| Manager | scripts/initiative_control/requirements.txt | 1 |
| Manager | docs/research/tasknode-integration/slack-live-allocation.md | 130 |
| Manager | docs/plans/active/initiative-delivery-control.md | 12 |
| Manager | docs/sprints/current/initiative-delivery-control/pf-80-s01-delivery-control.md | 20 |
| Manager | docs/plans/decision-escalation.md | 15 |
| Manager | docs/plans/workstream-manager-handoff-2026-09-11.md | 25 |

Estimate1668total/943non-test; manager grants coherent-unit target1800total/1000
non-test, STOP for reallocation before exceeding1900/1050. This explicit above800
allowance covers real I/O plus integrated recovery, not inherited F01 permission.
Preserve readable code, tests and failed evidence; no compression to meet counts.
One new independent code pass plus scoped correction; prior review usage retained.
Code-blind design/evidence remain required before human-ready handoff, not waived.

## Implementation contract

September12 15:45 correction allocation: first frozen seven-path candidate was
1381total/688non-test with198 passing Python tests, not live qualification.
Independent review46183 found four in-scope blockers: lost ingress hold under
lock contention, outbound bot echoes stopping reception, expired qualification
preventing retained-event drain, and threaded-parent receipt recovery. Manager
read/classified these and dispatched one coherent first correction to James.
Also restore the agreed OAuth scope boundary and separate first60s proof from
ongoing manager-owned listening with bounded reconnect; production time must
advance. No new Slack permissions or live actions are authorized by this repair.
Integrator grants worker1900total/900non-test, parent600/450 reserve, combined
17-path ceiling2500/1350 for this correction plus shared registration. Preserve
original1800/1000 target1900/1050 stop and all earlier counts/reviews. One scoped
corrective review follows this changed candidate; no unchanged repeat review.
Shared registration remains manager-owned and must receive independent combined
review before acceptance; the integrator grants that one additional material-diff
pass, without resetting earlier offline/code-blind/F01 review usage.

1. Real synchronous `slack_sdk.WebClient.chat_postMessage` and built-in
   `slack_sdk.socket_mode.SocketModeClient`; no webhook fallback or new scheduler.
   Pin `slack-sdk==3.44.1`: [official package](https://pypi.org/project/slack-sdk/3.44.1/),
   verified September12, non-yanked wheel SHA256
   d6f20a0fbe3fecf9cac955c99d686301b48a645b7045472c3a0cdd186d7c42b2.
   No optional dependencies needed for built-in sync transport; verify in tests.
2. Preserve offline APIs, stable alert/resolution slots, store-to-feed locking,
   intent before I/O/CAS, uncertain hold and exact recovery. `decisions.py` and
   `decision_replies.py` are read-only. Add explicit transport extension init,
   bounded1MiB journal/100 pending events and independent transport lock using
   existing durable Store primitives. Missing/corrupt existing state never resets.
3. POST timeout5s, `retry_handlers=[]`, exact authenticated response association
   persisted before callback return. Known nonacceptance only is rejection;
   timeout/malformed/5xx/internal/fatal/unknown errors stay uncertain. Respect
   rate-limit metadata without auto-retry/reset. Retained receipts reconcile;
   ambiguous history/missing messages never prove nonsend. Bounded lookup3x15.
4. Socket callbacks bind authenticated session/generation/team/app/channel and
   immutable human ID; no JSON verified flag, browser-token scraping or replay
   authority. Persist normalized scoped event before ACK, target under3s; ingress
   cannot wait on dispatch/feed locks. Drain copies then releases ingress lock
   before intake, and marks afterward. Restart dedups actual event ID, not envelope.
   Bound16KiB input, preserve edit/delete audit; nested edit actor mismatch holds,
   deletion can recover original authorship only, never invent deleting actor.
   Unknown original, outage gap, stale generation and storage failure visibly hold.
5. SDK reconnect/shutdown must not resend POSTs. No raw frames, WSS URLs, tokens,
   unrelated messages or tool logs retained/exported. Credentials supplied via
   owner-managed injection, never argv, prompt, JSON config or repository secrets.
6. Manager-owned `control.py decision-slack` delegates to `decision_manager.main`:
   status/init-transport/qualify/send/listen/drain/interpret/resume/dispatch/
   reconcile/project-status. OFF status requires no credentials, SDK or network.
   Explicit live flag plus current qualified binding gates operations; explicit
   qualification session allows only identity/connectivity checks, not sending.
   Bounded drain10events/30s, first listener60s, bridge20s. No extra timer/service.
7. Optional strict redacted Slack status extends existing decision snapshot
   envelope, backward-reading v1; pin status and feed together through capture,
   export/ingest/render/health. Do not change canonical decision schema/revision
   merely to update transport status. Last verified is not permanently connected.
   Display received/clarification/recorded/queued/delivered/actual-ACK separately.
   Same-thread clarification/receipt posts have stable durable identity and
   uncertain recovery; no ten-minute notification spam or raw answer export.

## Native-agent manager bridge — authoritative correction to private proposal

Python has no Codex tool SDK. A bounded real bidirectional stdio session emits
one reserved request; the manager calls supported `multi_agent_v1.send_input`
for its exact existing native subagent, then returns actual tool acceptance.
It observes `wait_agent` final assistant output for the exact ACK tuple. Do not
use user-task tools or create a task to reach James/Mendel. Record real submission
ID as provenance; a derived receipt ID is correlation, not a server receipt.
Manager explicitly revalidates exact agent/allocation and existing owner.running
eligibility contract; completed turn is not a cancelled allocation, and a merely
running unrelated task is not eligible. No reinterpretation of stale owner flags.

Preserve handoff/agent/allocation/payload_digest/receipt_id exactly. Request only
acknowledgment first, not execution. An outbound example, generic done or another
agent's text is not ACK. Drain ingress before dispatch and after ACK; recheck
watermark, feed, audit, identity, cancellation and allocation before separately
unlocking authorized work. No shell/model execution of Slack text. Late acceptance
or crash is uncertain: reconcile exact retained tool/assistant evidence, never
blindly resend. Missing history holds. Native tools have no assumed exactly-once
remote guarantee; actual receiver repeat-ID behavior must be qualified.

## Tests and actual enablement gates

Use actual SDK HTTP/callback seams with controlled endpoints and real private
Store files/processes: OFF silence; POST request counts/no retries; all uncertain
boundaries; wrong identities; edit/delete/duplicates; ingress-before-ACK crash,
fsync failure/full/corrupt state; independent-lock contention; restart drain;
real stdio EOF/timeout/late acceptance; CAS/context races; fake/wrong/stale ACK;
idempotent exact ACK; feed compatibility/redaction and last-good publication.
Run all Python/Facilities/governance and final structured review on combined tree.
Native TUI/TensorCash/Isometric are not applicable to this Python operator slice;
actual Slack, phone/dashboard and native-agent handoff proof is applicable.

Manager qualifies existing AmbientCryptoT074X5KNENT/privateC0C0X2ELFKR/appA0C1CC2P4SE,
bot chat:write/groups:history, message.groups, app connections:write, actual bot
and immutable Travis identity/membership plus credential generation. Recipient
already approved; no credential absence established. Manager must qualify an
existing approved authenticated HTTPS gateway/route and phone login return,
all linked sprint/docs/status/assets access, unauthenticated denial and no public
cache/bypass. Local8769 is not a remote URL. New exposure/purchase needs authority.

After code/identity/HTTPS qualification, create one fresh test-only decision by
existing CAS: ask Travis to reply **ACK TEST ONLY** in its Slack thread. Record
actual parent/details/event IDs; restart receiver before interpretation, prove
one reply/resolution, publish through normal sync and inspect phone history.
Use existing responsible native agent for acknowledgment-only dispatch; retain
actual tool submission and assistant ACK, restart/reconcile without duplicate
assignment, then verify truthful Slack/dashboard confirmation. No task posting,
spending, code changes or deployment authorized by this test. Only completed real
gates permit ongoing Slack enablement. DEC021 partial/DEC025 advisory remain;
no broad human readiness, PF-80 completion, PF-79 activation or release inferred.

Primary contracts: [Web client](https://docs.slack.dev/tools/python-slack-sdk/web/),
[Socket Mode](https://docs.slack.dev/tools/python-slack-sdk/socket-mode/),
[postMessage](https://docs.slack.dev/reference/methods/chat.postMessage/).
