# PF-80-S01 — native schema-to-request adapter

Manager allocation, September 11 local / September 12 UTC; same active PF-80 /
PF-80-S01. **Internal delivery control — TO BUILD**, “Use sequential sprints per
initiative”. Accepted engine 7e25982b5 is integrated at
`486d2fb9481c01f7d73a6f8d9992c6e1b96359dd`, with all55 Task Node tests passing
within combined269 (two process-leak warnings), plus129 Python/governance tests.
No live authority or production caller is allocated.

## Exact increment and ownership

One Astra High worker, existing Task Node checkout/branch in active plan,
clean-fast-forwarded to reviewed allocation commit; parent records launch HEAD.
Literal repository-relative writes:

- `codex-rs/tasknode-session/src/delivery_adapter.rs`
- `codex-rs/tasknode-session/src/delivery_adapter_tests.rs`
- `codex-rs/tasknode-session/src/delivery_send.rs`: only child registration
  `#[path = "delivery_adapter.rs"] mod delivery_adapter;`.
- `codex-rs/tasknode-session/src/client.rs`: extract private shared blocking HTTP
  factory/request-builder helpers; existing request_blocking uses them with
  unchanged send/read/limit/decode. Add thin cfg(test) crate-visible build-only/
  decode-only accessors delegating to the same helpers and decode_response.
- `codex-rs/tasknode-session/src/recovery.rs`: only thin cfg(test) crate-visible
  accessor delegating to resolve_from_store with injected store/request closure.
- `qa/initiative-control/pf-80-s01/native-adapter/`

Parent serializes these shared seams. Adapter is a private child of delivery_send
and wholly test-only. No resolver decisions, auth/storage behavior, public API,
Client URL/redirect/timeout/response-limit changes. No delivery_goal, lib.rs,
old tests/evidence, CLI/TUI, dependencies, BUILD, lockfiles or shared-plan writes.
Manager receipt-stage reallocation, September 12: permit <=435 test lines within
the unchanged hard <=500 non-test / <=800 total additions plus deletions.
Returned scope is260 adapter +67 Client +13 resolver/registration +433 tests
+26 evidence =799,366 non-test. Smaller implementation funds the additional
required test matrices; no path, behavior, authority or hard-budget expansion.
The original300-test sub-budget was exceeded and the worker correctly stopped;
this explicit manager reallocation resolves that allocation-only hold, not review
or acceptance. Re-slice if hard limits are exceeded; never omit/compress checks.

## Concrete result and source contracts

Consume pinned raw fixture responses via actual native resolver/Client codec,
derive observed identity/enrollment/ownership/entitlement facts, use the accepted
engine, and build its exact reqwest request with the production-shared builder.
Inspect method/URL/headers/body entirely in memory. Never send or execute it.
This proves construction/decoding and schema consumption, not network delivery,
live credential provenance or a permission grant. Operator lifecycle support,
one-event approval and posting intent remain separate synthetic fixture facts.

First-party local source pin: `40d2df72710a644f33a2b30831061e8265716db0`, source
checkout `/Volumes/CorbanuDrive/Corbanu/.codex-work/tasknode-beta.Rf2AJJ/tasknode-source`.
Read and hash relevant files into evidence; do not modify them or fetch live data.

- GET `/api/terminal/tasknode/status`: explicit ok:true/accountId.
- GET `/api/terminal/tasknode/campaign-tracker/status`: ok/accountId and
  enrollments with actual snake_case workspace_id/enabled fields. Require one
  exact enabled workspace; reject duplicate/ambiguous rows. This is not ownership
  or entitlement proof.
- GET `/api/terminal/tasknode/tasks/{encoded exact ID}`: terminal route uses
  getTerminalTaskProjectionDetail, filtering task_id AND subject_wallet AND
  authenticated nonempty account_id. Ownership observation comes from that
  request/account fence, not an invented owner field. Response task.fullId AND
  task.taskId must match; task.id truncates to12 and is display-only. Parse
  statusKey from pinned shared lifecycle; unknown status held. Ingest does not
  prove the operator supports Proposed lifecycle: retain separate fixture policy.
- Entitlement consumer in server/campaign-tracker-model.js uses gateway
  `/v1/account`: current period or fallback legacy period, startsAt <= now < endsAt,
  OR positive exact balanceMicrousd. Parse bound injected synthetic gateway JSON
  only. No gateway client, invented route, guessed producer schema or claim that
  Task Node and gateway account IDs match. Document supported date/integer wire
  forms and conservative held forms; no floating-point funding inference.
- POST events shape `{event,apiKey}`; approved digest remains over `{event}`.
  Reuse accepted strict goal receipt rules, not raw server error messages.

The existing resolve_scoped touches Vault/filesystem/link endpoints: DO NOT CALL.
Use actual inner resolve_from_store through the tiny test accessor with MemoryStore
and injected responses. Preserve its existing time behavior; fixture dates must
be explicit and valid for the actual resolver clock, without environment mutation.
Client::identity fences account/origin/token, not profile/provenance. Retain separate
profile binding and native origin normalization; no anonymous/default fallback.

## Driver and proof

Bind each raw observation to invocation, profile, account, normalized origin,
Client identity, source method/path, selected event/digest/workspace/tasks.
Gateway observation also binds the synthetic API-key handle; never substitute
terminal bearer or unrelated gateway-key observation for entitlement evidence.
Derive four observed facts from these schemas; only three operator facts remain
synthetic. Resolve named/default profiles through actual MemoryStore logic,
including pending exchange/status validation, mismatch and preserved failed state.
No mock resolved=true bypass. Decode injected bytes with the real Client decoder.

Build-only FixtureExchange invokes the shared builder, captures actual reqwest
Request, and decodes a raw fixture receipt through Client. Assert fixed HTTPS
endpoint/POST/bearer/Content-Type and exact serialized payload; strip synthetic
apiKey and prove original approved payload/digest. Never call send/execute/stream.
No new socket, server subprocess, credential read, live endpoint or enrollment.
Existing unchanged full-crate loopback regression is allowed, with no new live
claim. Redirect/timeout network behavior is not proved by build-only tests.

Tests: complete schema-driven success; account/workspace/snake_case/duplicates;
full-ID collisions beyond12/missing IDs/wrong endpoint/lifecycle; current/legacy
period boundaries and positive/zero/negative/malformed funding; unrelated key;
missing ok/malformed bytes/type; profile/client changes before/after/cancel/second
use; strict 401/402/403/409/429/5xx/redirect/deleted/pending results; credential
canaries absent from exported diagnostics. Do not relax engine's unknown-expiry hold.

Guard just fmt to allocated Rust files, disclose failures; cached1.95.0 offline
just test -p codex-tasknode-session with delivery_adapter, delivery_send,
delivery_goal selectors then full crate; normal library cargo check --offline
--locked -p codex-tasknode-session --lib. Both governance and whitespace checks.
Return uncommitted hashes, nonzero counts, line budget, failures and boundaries;
parent owns independent Astra High review/corrections and combined-tree testing.
No additional reviewers, main pushes, releases or exhausted-budget reset.

Upstream `https://github.com/openai/codex.git`, common ancestor
413492cd6c3a4d4f8dff6f406247ccda5a9d88aa, cached upstream
1d74c3ba1ee98be2025ab066dcc3fd654fe8a3b6. Product-owned crate, tiny unchanged-
behavior Client refactor/test accessor only; preserve auth/history/permissions,
recovery, response caps and cancellation semantics. No upgrade qualification.
Existing Bazel source glob handles sibling tests; use literals, no external includes.

## Human/live handoff, not an implementation blocker

Native terminal issuance can have null expiry; engine currently holds it. Do not
invent expiry or relax validity. Before live activation assemble named operator,
exact candidate/controls/profile/account/origin/targets/visibility/API-key source,
enrollment, explicit one-event ID/digest and recovery packet. Reconcile existing
answers, then ask only unresolved choices, including validity checks for null
expiry and supported Proposed workflow. No secrets requested for this offline
increment. Human TUI/live-repo, benchmarks, whole-sprint/PF79/PF81 and release
acceptance remain open; source fixtures are not live operator acceptance.
