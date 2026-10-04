# PF-80-S01 native contract and decisions

Status: offline preparation implemented; in progress / awaiting manager review.
Product citation: **Internal delivery control — TO BUILD**, “Use sequential
sprints per initiative”. Plan: `docs/plans/active/initiative-delivery-control.md`.
Sprint: `docs/sprints/current/initiative-delivery-control/pf-80-s01-delivery-control.md`.

## Inspected source, not live account evidence

Receiving native baseline: `f17e5a54daba11c6da2554f14b62f11ace752142`.
First-party TaskNode: `40d2df72710a644f33a2b30831061e8265716db0`, read-only at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/tasknode-beta.Rf2AJJ/tasknode-source`.
No auth store, private account, SSH secret, wallet or real API was accessed.
The source manifest records exact file hashes separately from commit pins.

| Existing source / symbol | Verified source behavior | Integration consequence |
| --- | --- | --- |
| `codex-rs/cli/src/tasknode_cmd.rs`: `TaskNodeCommand`, `resolve_tasknode_profile`, `tasknode_session_scope` | Native CLI supports linking, status and task workflows; inherited profile conflicts or absent agent scope fail closed. No arbitrary Campaign Tracker event command exists. | Do not invent a CLI verb or use a task mutation as progress transport. |
| `codex-rs/tasknode-session/src/client.rs`: `Client::for_session`, `identity`, `request_blocking` | Shared bearer transport binds saved and requested origins, refuses redirects, and fences responses by credential identity. | Reuse this client with the selected native profile for any later sender. |
| `codex-rs/tasknode-session/src/tracker.rs`: `TrackerStore::new`, `capture`, `sync` | Encrypted queue partitioned by profile/account/origin; capture generates a new UUID/time/sequence; sync processes up to 20 pending events. | Neither capture nor bulk sync preserves an externally approved one-event selection. Do not inject the historical queue into native capture. |
| `codex-rs/tui/src/chatwidget/campaign_tracker.rs`: `tracker_session`, `tracker_api_key`, `campaign_tracker_sync` | Native profile session and Corbanu API key feed existing tracker requests; pending events can sync while recording is paused. | Recording pause is not the internal adapter's posting-OFF switch. Do not open/sync the native live queue during qualification. |
| `server/campaign-tracker-routes.js` | Enrollment enablement and events both check entitlement; events pass `validateEvent` then `trackerIngest`. | Goal-only still requires entitlement. |
| `server/campaign-tracker-contract.js`: `validateEvent` | Goal content at most 2048 UTF-8 bytes; bounded identifiers, safe integer sequence, UTC timestamp, allowed fields. | Preserve exact metadata/event identity, reject unknown or unsafe payloads. |
| `server/repositories/campaign-tracker.js`: `trackerIngest` | Identity scoped by account/event ID; changed payload yields 409. New events require enabled workspace and owned task projections. Goals get `summaryState: not_applicable`; output summaries are separate. | No task acceptance, reward or completion is implied. A deleted/tombstoned response is not delivery success. |
| `server/campaign-tracker-model.js`: `trackerEntitlement` | `/v1/account` must show an active period (including legacy period) or positive `corbanuApi.balanceMicrousd`. | Preserve main's funded API-balance semantics; this inspection does not show this account qualifies. |

Ownership lookup checks task projection existence under the authenticated
account, without a lifecycle-status predicate in this ingest path. That suggests
an owned Proposed target may be technically linkable, but does not establish a
supported operator workflow or authorize acceptance. Explicit board/history
grants are separate from task ownership and workspace enrollment. A local
workspace-only receipt does not prove account identity after credential rotation.

## Transport decision for this slice

Reuse the existing product-owned Python projection, validation and outbox.
Retain its inherited HTTP adapter for audited compatibility and negative
fixtures; add no auth provisioning or transport system. The new preparation
path has no transport argument and cannot call native or Python delivery.
The inherited bulk flush remains inaccessible through an exact-ID CLI selector.
There is no live single-event sender in this commit.

Smallest later implementation allocation: expose one native Campaign Tracker
goal-event send using the existing profile-bound `Client`, explicit origin and
account checks, exact expected event ID plus full payload digest, current
enrollment/entitlement/ownership verification, and one-attempt semantics. It
must not call `TrackerStore::capture` or `sync`, iterate an outbox, accept tasks,
or reset IDs on timeout. Manager must allocate the CLI/session paths and native
tests separately. Keep this worker's Rust boundary read-only.

## Questions requiring operator / product decisions

1. Credential scope: which exact native profile, TaskNode account ID and origin
   own this private delivery-control workspace? Who is allowed to operate the
   sender, and on which machine? Can the existing native session remain local
   rather than exporting broad authority to the server? No progress-only scope
   is established by this inspection; do not distribute the human session.
2. Credential lifecycle: what supported login, expiry/revocation and rotation
   workflow is approved? How will enrollment evidence bind account, origin,
   workspace and receipt time so a swapped account cannot reuse an old receipt?
   A workspace-only JSON flag is insufficient for later live approval.
3. Target lifecycle: are the existing Proposed personal targets supported as
   goal-progress destinations while remaining Proposed? Confirm the exact owned
   PF-80 destination ID and its current status. What read-only receipt proves
   ownership? Do not accept/create/submit a task to make the test work.
4. Entitlement: does the intended account currently satisfy the supported
   active-period or funded-balance rule, and which credential is approved for
   that check? Source capability does not establish live entitlement, pricing,
   billing permission, or remote credential provisioning authority.
5. Enrollment: is the exact workspace already enrolled on the intended account?
   Who may authorize enrollment if absent? First-event approval must explicitly
   cover that separate mutation if required; this worker has not performed it.
6. Destination visibility: who can read the event? Are private account history,
   collaborator relationships and any board grants sufficient and approved?
   No public board grant or beta entitlement follows from personal task IDs.
7. First payload: which **one newly observed PF-80 event**, exact `cc-` ID, full
   payload hash, source commit/time and destination list may be sent once? Has
   the human reviewed content, branch metadata, privacy, freshness and the
   explicit no-acceptance disclaimer? Synthetic fixtures cannot be that event.
8. Recovery: after timeout, who checks the same account/event ID and approves a
   same-payload retry? A 401/403/402/409, wrong response ID or tombstone must hold
   the record; do not remap, generate a replacement ID or enable bulk retry.

All answers remain pending. Approval must name a concrete payload and scope;
the preparation JSON always has `send_authorized: false`. No delivery or human
acceptance claim follows from these tests.
