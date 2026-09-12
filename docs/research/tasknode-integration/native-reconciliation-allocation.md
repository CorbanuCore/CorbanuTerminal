# PF-80-S01 — uncertain goal observation reconciliation

Manager allocation, September 12 UTC. Same active PF-80 / PF-80-S01 product
initiative; **Internal delivery control — TO BUILD**, “Use sequential sprints per
initiative”. Native adapter d7bf73a52 and accounting storage d898fbac0 are reviewed
and integrated at `56295f66825e87dc924705d86fcbcebf258ba7f9`;283native/129Python-
governance tests and both normal-library checks pass. No production/live claim.

## Exact scope and OFF boundary

Reuse Task Node checkout/branch in active plan; clean-fast-forward to reviewed
allocation commit. Parent records launch HEAD and one Astra High worker identity.
Five literal paths:

- `codex-rs/tasknode-session/src/delivery_reconcile.rs`
- `codex-rs/tasknode-session/src/delivery_reconcile_tests.rs`
- `codex-rs/tasknode-session/src/delivery_send.rs`: only private child registration
  `#[path = "delivery_reconcile.rs"] mod delivery_reconcile;`.
- `qa/initiative-control/pf-80-s01/native-reconciliation/receipt.md`
- `qa/initiative-control/pf-80-s01/native-reconciliation/SHA256SUMS`

Private child of existing test-only engine. No production constructor/caller,
Client/recovery/lib.rs/CLI/TUI/tracker/goal edits, auth behavior, dependencies,
BUILD, old tests/evidence or shared plans. Reuse actual Client build/decode-only
accessors; never send/execute/stream, touch Vault or read an outbox. Parent
serializes child registration. Target240implementation+360tests+3registration+
45receipt+10manifest; hard500non-test/800total additions AND deletions. Preserve
required cases; re-slice instead of compression or dropping validation.

## Grounded contract

After an uncertain POST, preserve original event bytes/ID/digest and invocation/
profile/account/origin/client binding. This helper consumes that immutable
selection plus synthetic current session and injected activity response bytes.
It constructs and inspects a GET entirely in memory. It never sends, retries,
resets consumed Attempt or grants OneEventAuthorization/Posting. Positive result
means an exact fixture observation, not a production delivery or task completion.

Pinned first-party source40d2df72710a644f33a2b30831061e8265716db0 at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/tasknode-beta.Rf2AJJ/tasknode-source`:
read/hash server/campaign-tracker-routes.js, repositories/campaign-tracker.js
trackerRead/trackerIngest and campaign-tracker-contract.js. Do not fetch or edit.
Manager inspected actual route allowlist, annotated output and404 semantics.

1. Build only GET /api/terminal/tasknode/campaign-tracker/activity with exactly
   accountId and id query parameters, safely encoded from frozen selection.
   Bind synthetic raw result to exact method/path/invocation/profile/account/
   normalized HTTPS origin/client/event/digest. Reject unrelated responses.
2. Use actual native Client identity/codec and existing engine fences where
   applicable. Preserve cancellation, known-expiry checks and ExpiryUnknown;
   no new auth state machine or fabricated TTL. Identity change remains held,
   not rebinding old evidence. Do not call native resolve_scoped or live status.
3. Require successful status, ok:true, exactly one items record and explicit
   null nextCursor. Match record accountId plus every field of approved goal
   event: exact id/kind/content/workspace/full task IDs/metadata/timestamp/etc.
   Compare original typed event projection, not a hash of the server-annotated
   whole record. Original approved digest remains over Python-canonical {event}.
4. Pinned server adds accountId,revision,summaryState,receivedAt,capabilities,
   handleAtExecution,summary. Validate relevant annotation types and require
   summaryState:not_applicable for goal; hold redacted/incomplete/malformed/
   conflicting/duplicate/paginated or non-goal responses. Never manufacture
   missing original fields or infer equality from truncated IDs.
5. Exact-ID404 is UNCERTAIN, not proof no write occurred: source filters expired,
   inaccessible and other non-readable rows; tombstones are not ordinary items.
   Hold401/402/403/409/429/5xx/redirect/malformed responses with typed diagnostics.
   No server error text, credentials or activity content in exported diagnostics.
6. Keep original selection/outcome immutable and prove no retry or sender call.
   A successful observation is not authorization for a subsequent write. Missing
   observation never creates a new event or erases uncertainty.

## Required proof

Literal source-shaped exact goal response and exact built GET/headers/query;
all original event and binding mutations; content redaction, missing/duplicate
items, absent/non-null cursor, malformed annotations and wrong summary state;
strict HTTP/decoder errors and404ambiguity; cancel/expiry/profile/account/token
changes; original bytes/ID/digest unchanged and synthetic credential canaries
absent from typed diagnostics. Inspect whole request/response objects where useful.
No sockets/new processes/live account read. Existing unchanged crate loopback
regressions may run, with no claim they qualify this helper's network behavior.

Guard just fmt to allocated Rust paths before final tests; cached1.95.0,
RUSTUP_AUTO_INSTALL=0,CARGO_NET_OFFLINE=true, own target. No toolchain/dependency
repair, denied-surface bypass or lock/process cleanup. Use just test -p
codex-tasknode-session delivery_reconcile, delivery_adapter, delivery_send,
delivery_goal then full crate; normal cargo check --offline --locked -p
codex-tasknode-session --lib. Python54 suite with existing pinned venv, both
governance checkers and whitespace. Record nonzero counts and formatter failures.
Return uncommitted exact paths/hashes/line counts/failures/OFF proof. Parent owns
one independent Astra High review plus scoped corrections and combined tests;
no other agents/model calls/reviews or budget resets.

## Human packet and actual remaining decisions

Historical account/task creation approval stands: IridiumMaster/@iridiumeagle,
PF-80 target task_789a0f3bd75b41d1eca20cae698f04cf, historically Proposed.
Never recreate or accept it to make tests pass. Handles are not today's opaque
account/profile proof; no live validation is supplied by source fixtures.
Parent delivered two grouped setup-direction/nullable-expiry design questions
in the manager task on September12. Answers must be recorded when received;
neither question grants credential reads, enrollment or first-post authority.
No answer is needed for this existing-fence offline increment.

Keep credentials on the human operator's machine unless separately authorized;
no broad session export to Linux/agents. Before later live qualification, pin
binary/source/hash, named operator, exact default/named native profile, normalized
origin/account, workspace, target ownership/status, intended readers, approved
API-key source (never value), entitlement/enrollment and ONE new PF-80 event
with exact reviewed ID/digest/content/time/source. Reconcile historical PF-76
without rewriting/replaying it. No beta/public tasks/rewards or bulk flush.

Human-style sequence must use actual controls: CLI tasknode --profile PROFILE
--origin ORIGIN link status inspects local state; status may resolve/promote a
pending link and request server identity. link start and link cancel are existing
CLI controls; cancel preserves active session. TUI /tasknode link starts a link:
there is no /tasknode link cancel verb. Exact target task show is read-only after
native resolution. No native exact-event send/get/retry command currently exists.
Do not substitute task-request retry, Sync now or a new ID as recovery.

Important prerequisite verified in existing TUI campaign_tracker.rs:
30-second ticks can drain pending events even while recording is paused. Do not
launch the linked production TUI as a harmless read-only test. A later TUI run
needs an independently qualified isolated scope with no inherited queued/enrolled
tracker state, or stays blocked. No hidden file repair or credential manipulation.
Use fixtures now for failed replacement, account mismatch, expiry/cancel/rotation;
later approved real-key tests send text and Enter separately and verify supported
cancel/relink/restart. Existing temporary-link expiry proof is not token validity.
Server-issued token expiry may be null; relink may repeat that, so never promise
relink resolves ExpiryUnknown. Do not alter validity before a separate reviewed
contract. Even GET activity has server audit/maintenance effects; live access
must be explicitly scoped, not called here. Missing exact-ID row stays uncertain.

S01 remains in_progress. Operator validity/Proposed support/visibility/entitlement/
enrollment/exact-payload acceptance, supported user-facing recovery and default-OFF
runtime promotion are later allocations. PF79/PF81 dependencies, TUI/live-repository,
human/benchmark/release gates remain. Upstream https://github.com/openai/codex.git,
common ancestor413492cd6c3a4d4f8dff6f406247ccda5a9d88aa,
cached1d74c3ba1ee98be2025ab066dcc3fd654fe8a3b6; no upgrade qualification.
