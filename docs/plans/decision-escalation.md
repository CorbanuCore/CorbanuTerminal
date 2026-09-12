# Decision visibility and alerts — PF-80 planning amendment

Status: implementation planned; Slack channel/app created, sender not wired or
delivery-tested. Product: **Internal delivery control
— TO BUILD**, “Show blockers, rendered sprints, human test plans, machines, run
logs and freshness.” Travis requested contextual dashboard decisions and Slack
alerts, selecting AmbientCrypto / private “The Corbanu Project” in this task.
This belongs to PF-80-S01, not a fourth initiative or separate scheduler.

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

## Slack v1

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
agents or grant permissions; v1 answers are recorded by the manager from this
authorized task. Two-way Slack approvals need a separately reviewed identity,
revision and authorization contract. Test a real alert with Travis before calling
the notification path operational; device push/DND settings also affect receipt.

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
