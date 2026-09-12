# Decision visibility and alerts — PF-80 planning amendment

Status September12: canonical decision feed and offline Slack recovery are
reviewed/integrated;169 receiving Python tests pass. Real Slack SDK transport,
native-agent bridge and manager registration are now allocated in the
[real transport mandate](../research/tasknode-integration/slack-live-allocation.md).
Connection/HTTPS/phone reply/restart and actual agent ACK remain unqualified;
no live alert or reply delivery is claimed. Product: **Internal delivery control
— TO BUILD**, “Show blockers, rendered sprints, human test plans, machines, run
logs and freshness.” Travis requested contextual dashboard decisions and Slack
alerts, selecting AmbientCrypto / private “The Corbanu Project” in this task.
This belongs to PF-80-S01, not a fourth initiative or separate scheduler.

## Shared registration candidate — not deployed or live-qualified

The manager staging checkout implements explicit `control.py decision-slack`
delegation. Ordinary publish/serve never starts a listener or sends a message.
`decision-slack status --store PATH` is OFF unless explicitly opted in and needs
neither an existing transport store, the Slack SDK, credentials nor network.

Before a normal source export, the manager can explicitly produce a local cache:
`decision-slack --publish-state STATE project-status --store STORE [--live]`.
Here `--live` only reads the existing private journals; it does not connect.
Omitting it records OFF without reading STORE. The strict owner-only cache binds
the exact decision-feed digest and each question's ID/revision/context digest.
It contains redacted delivery phases and current reply-state counts, not raw
answers, external message IDs, credential data or a work authorization.

Publication captures cache and feed together in a pinned version2 envelope;
absence retains version1 compatibility. Malformed or stale-context cache means
unknown Slack status without discarding a valid decision feed. Resolved cards
show the original answered question revision, not unrelated historical totals.
Saved assessment and last-verification timestamps are explicit; after15minutes
the backend treats the observation as stale. A saved status is not a continuously
verified connection, and no canonical decision is revised by cache publication.

Eighteen feed tests pass on this uncommitted staging tree, including actual local
SDK ingress, two-question separation, answered-revision mapping, OFF silence,
strict malformed-input handling, transfer pins, rollback and freshness. Synthetic
publisher stdout is not deployment. Combined transport review, source integration,
normal publication and all actual Slack/phone/agent-ACK cases remain mandatory.

## Single manager-owned record

One stable decision ID and revision owns: initiative/sprint, owner, raised time,
question, plain-language background, evidence links, options and recommendation,
tradeoffs, which work actually stopped, work that can continue, resolution and
answer provenance. Track open/acknowledged/resolved/superseded separately from
delivery pending/sent/failed/uncertain. Acknowledgement is not approval; a send
receipt is not proof of reading. Agent reports may propose blockers but cannot
change human decisions or auto-accept their own output.

## Dashboard contract

Put “Needs your decision” above the workstream map, with a count and oldest age.
Cards show the full context above, affected sprint/test-plan links and alert
state. Separate waiting on Travis, manager preparation, infrastructure failure
and running implementation; show actual worker count, not scheduler activity.
Resolved decisions stay inspectable but leave the open queue. Missing/stale
decision input means unknown/stale, never “nothing needs you.” Escape all text,
allow only approved safe links, and exclude credentials and raw private logs.

Manager decision freshness/source revision is separate from publication time and
code-source commit. The current recovery-source dashboard cannot imply it includes
the receiving branch's work. Add an explicitly allowlisted manager-decision
projection with its own provenance; qualify export, ingest, render and health
together before any source/publication change. Preserve the last good snapshot
and visibly flag stale input on failure. This document is not that implementation.

## Historical outbound-only setup

Use one channel-scoped incoming webhook/app connection for outbound alerts only.
Verify the workspace, exact private channel ID, audience and credentials through
supported setup; do not scrape desktop tokens or copy broad human sessions.
Keep the webhook secret out of git, dashboard exports and logs. Installation
scope is limited to the selected channel; no workspace-history permission is
needed for v1. Setup verified in the signed-in UI: private channel
`C0C0X2ELFKR` in workspace `T074X5KNENT`, app `A0C1CC2P4SE` named Corbanu
Decision Alerts, incoming-webhook only, bound to that channel. No other people
were invited. Webhook issuance was verified with the secret redacted; it is not
stored in this repository or wired into the sender. No test message was sent.

Send an initial alert for a new blocking decision and a material revision or
resolution. Persist a delivery ledger keyed by decision ID/revision/event and
serialize send ownership so ten-minute refreshes/restarts do not repeat normal
successes. Ambiguous timeouts become delivery-uncertain, not silently successful
or automatically retried as though unsent; exact-once delivery is not promised.
Known rejection/network failure needs bounded recovery and visible failure state.
Reminders/digests are a later preference, not ten-minute notification spam.

Each alert contains a redacted short question, recommendation, stopped scope,
owner and stable authenticated dashboard decision link. Never send localhost
links to remote readers. Acknowledgements or arbitrary Slack replies do not run
agents or grant permissions. Travis now requests logged replies routed to the
responsible agent; the two-way contract below supersedes that outbound-only
target, not its current unconnected implementation state. Test a real alert with Travis before calling
the notification path operational; device push/DND settings also affect receipt.

## Requested summary, details and reply loop — September 12

The dashboard uses one short linked summary, expandable context, then a specific
question only when a human decision is needed. Every sprint reference must link
to its exact published context. Historical ID collisions link to the historical
explanation, not the unrelated modern sprint. Unknown references open an honest
unavailable-context explanation. Manager cleanup explicitly says no human
decision is needed; do not manufacture questions just to fill a card.

Use the same decision ID/revision, linked sprint, summary, owner, background,
impact, recommendation/options and question in Slack. Proposed Slack presentation:
summary in the parent message; details and question in its thread, with a direct
decision link. This preserves the information hierarchy without claiming HTML
accordion support in Slack. Questions must be answerable directly in that thread.
An outbound-only webhook cannot receive those replies.

Proposed connection: existing app in AmbientCrypto, bot membership restricted to
the private Corbanu channel, `chat:write`, `groups:history`, subscription to
`message.groups`, and Socket Mode with an app token limited to `connections:write`.
Socket Mode avoids opening an inbound public endpoint on the dashboard server.
Slack scopes can expose other private channels the bot joins: the channel/team
allowlist is enforced in code as well, and this bot must not join other channels.
Verify reinstall/consent, bot identity, exact team/channel and Travis's immutable
Slack user ID in the supported UI; keep bot/app tokens in owner-only service
credentials, never source exports, logs, task prompts or browser session copies.

Reply processing contract, still awaiting allocated implementation/review:

1. Accept only a verified Slack event from the configured team/channel, a mapped
   decision thread and an allowlisted human decision owner; ignore bots, unrelated
   messages and forwarded text. A display name or quoted approval is not identity.
2. Durably log the event ID, message/thread timestamp, actor ID, decision ID and
   revision, answer and receipt time before acknowledging processing. Deduplicate
   retries and recover pending work on restart. Edited/deleted answers create new
   audit events; they do not silently rewrite an already-applied decision.
3. Associate each message with the exact presented decision revision. If the
   question changed, the decision resolved, the answer conflicts or meaning is
   ambiguous, retain it and ask for clarification; do not choose an interpretation
   that widens authority. Minimize stored content to decision-thread answers.
4. Manager validates the answer's scope, records the resolution and queues a
   handoff addressed to the current workstream owner/allocation, not a stale
   agent session. A stopped agent becomes pending-manager-dispatch, not delivered.
5. Track received, needs-clarification, recorded, queued, delivered and
   agent-acknowledged separately. Show the same resolution/provenance on dashboard
   and Slack; acknowledge to Travis what was recorded and where it was routed.
   An agent acknowledgement is not implementation completion.
6. Slack text is decision data, never executable commands. Ordinary product
   answers unlock only existing authorized work. Releases, spending, signing,
   secrets, live posting and changed access remain behind their explicit gates.

Before enabling: test wrong actor/channel/thread, duplicate/out-of-order events,
old revisions, edits/deletes, unknown send outcome, crash/restart, stopped owner,
ambiguous answers and redaction. Then a real harmless test question must receive
Travis's reply, survive receiver restart, appear on the dashboard and reach the
intended agent exactly once logically (transport retries remain possible).
No receiver, answer logging or automatic routing is operational at this update.

References: [private-channel events](https://docs.slack.dev/reference/events/message.groups/),
[Socket Mode](https://docs.slack.dev/apis/events-api/using-socket-mode/).

## Implementation and human proof

After the current native validity increment, manager allocates exact existing
`scripts/initiative_control/` seams for record validation, renderer, export and
durable alert delivery, with tests and bounded increments. No agent writes the
live state or sends alerts before that allocation/connection is qualified.
Test new/changed/resolved decisions, reboot deduplication, timeout uncertainty,
failed/stale publication, wrong/missing source, link access, redaction/XSS and
concurrent send attempts. Use a phone-sized dashboard and a real Slack test with
Travis away from Desktop; prove open decision visibility, return/answer flow and
automatic next manager action. Unit tests alone do not qualify that human flow.

Source: [Slack incoming webhooks](https://docs.slack.dev/messaging/sending-messages-using-incoming-webhooks/).
Webhooks bind to a channel and contain a secret; successful send is only delivery
evidence. No Slack message or live dashboard deployment was performed here.
