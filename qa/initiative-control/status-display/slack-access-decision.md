# PF-80-S01 — allow private-channel alerts and replies

## Summary

Live Slack qualification needs approval to expand the existing Corbanu Decision
Alerts app from webhook-only permission to posting and reading private-channel
replies. This question was asked directly in the manager task after actual
browser inspection around23:00UTC, September12. No answer recorded yet.

Affected sprint: [PF-80-S01](../../../docs/sprints/current/initiative-delivery-control/pf-80-s01-delivery-control.md).
Owner: Travis Good for access approval; integration manager for configuration
and execution. Only the live Slack path is held; offline engineering/review,
accounting and PF13 continue. Slack cannot deliver its own permission question.

## What was actually observed

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

The previous status that only testing remained was incomplete: this is an
observed app-configuration prerequisite. Notification status: asked in the
manager conversation; dashboard projection pending normal synchronization;
Slack not delivered. Existing resolved PF27 approval remains resolved.
