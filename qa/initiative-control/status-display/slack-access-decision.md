# PF-80-S01 — allow private-channel alerts and replies

## Summary

Travis explicitly approved the requested Corbanu Decision Alerts access in the
manager task on September12: "Yes, you may grant Corbanu decision alerts, etc.
Please proceed." The original access question is resolved, not still waiting.
The manager immediately saved message.groups and configured chat:write plus
groups:history, preserving incoming-webhook and no user-token scopes.
Reinstallation is complete: Travis separately answered "Yes, accept and finish
installation", the manager clicked Allow, and Slack returned to the installed
OAuth page without the pending-scope-change warning.

Affected sprint: [PF-80-S01](../../../docs/sprints/current/initiative-delivery-control/pf-80-s01-delivery-control.md).
Owner: integration manager for configuration and test execution. The completed
reinstall selected AmbientCrypto and private the-corbanu-project only. The app
is now listed in that channel's Agents & apps panel (bot user U0C1EBT4UJV).
That U-ID is not the transport's still-unverified bot B-ID. Socket Mode remains
ON and its settings now show Event Subscriptions enabled. Bot scopes are exactly
chat:write, groups:history and incoming-webhook; user-token scopes remain empty.
Offline engineering, accounting and PF13 can continue.

## Historical preflight, before the approved installation

The signed-in AmbientCrypto session can open the private the-corbanu-project
channel. Corbanu Decision Alerts exists in that workspace. Socket Mode is ON,
but its settings show event subscriptions OFF. Bot OAuth scopes list only
incoming-webhook, not chat:write or groups:history. The channel has the original
integration-add notice and no automated alert history; no bot is listed in its
Agents & apps panel. Token presence alone does not establish the needed scopes.
No credentials, unrelated conversations or private browser output are included
in this record. No messages, scope changes, invitations or credential copies
were made during the preflight.

## Question

The following original question was approved. Preserve it as decision history.

May the manager grant Corbanu Decision Alerts permission to post messages and
read messages in private channels it joins, enable private-channel reply events,
reinstall the existing app and add it only to the-corbanu-project in AmbientCrypto?

Recommendation: approve chat:write, groups:history and message.groups for this
purpose; retain Socket Mode, with only the existing approved channel invitation.
The read scope applies to private channels the bot joins, not an OAuth guarantee
of single-channel access. Do not invite it elsewhere. No DM, public-history,
file, admin or arbitrary command permission is requested.

Alternative: keep outbound-webhook-only access; automatic reply receipt and
agent routing cannot be qualified or enabled. Test-only browser messages would
not establish the bot's sender/listener path and are not a substitute.

## What approval would and would not do

It unlocks the actual permission/install/channel-membership setup. The manager
then checks supported credential injection, actual connection and a clearly
labeled no-op alert/reply/record/agent-acknowledgment roundtrip. No real project
decision, financial operation or Task Node action is approved by a test reply.
New credential entry, if required, must follow the supported user handoff; no
browser/session or Keychain extraction. Remote authenticated dashboard/phone
and independently isolated functional acceptance remain separate gates.

## Actual remaining setup and proactive testing

The supported control.py decision-slack launcher and SDK runtime exist. A
read-only check found CORBANU_SLACK_BOT_TOKEN and CORBANU_SLACK_APP_TOKEN absent
from this runner's environment; no values, browser credentials or Keychain were
read. That does not prove the user lacks credentials elsewhere. The existing
interface accepts environment injection; no inspected automatic loader exists.
Channel membership is now verified. App-level connections:write token metadata
could not be verified: both the Socket Mode App Level Token link and Basic
Information navigation lead to the legacy page without an App-Level Tokens
section. No credential was revealed, copied, created or regenerated. The user
must supply the existing bot token and app-level token through private credential
provisioning, never chat or committed files. Their actual API identity and
connection, including bot B-ID, still require verification. No live alert/reply
roundtrip has run.

The final Allow agreement was separately approved and installation verified;
do not resurrect either already-approved question. Next is private credential
provisioning into CORBANU_SLACK_BOT_TOKEN and CORBANU_SLACK_APP_TOKEN, then
the bounded no-op alert/reply/native-ACK test. The manager must initiate available
tests or assign eligible execution, as now recorded in the sprint process and
saved manager automation. Independent isolated execution, phone and overnight
acceptance remain distinct from operator smoke testing. PF27 stays resolved.
