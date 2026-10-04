# PF-60-S02 next executable unit: accounted OpenAI Responses HTTP sampling — allocation

Manager-accepted same-S02 allocation, September 14 2026 07:05 UTC (Fable takeover manager).
Authority: Travis opened the accounting product-resumption gate on September 14
(decision `accounting-product-resumption`, feed revision 33); security remains
paused until the isolated executor is qualified. Predecessor: original-contract
goldens `f4507cb50`, combined verification `855ab3382` accepted; collection OFF.

## Manager acceptance and bounds

- Design source: Astra High action `responses-dispatch-design-01` (session
  `01a09e32-6151-7ef2-b80a-908ea8822f5d`), read-only against goldens tip
  `08eaae600dbbd468e4143f64a1ee2c28baf2acf4`; proposal SHA-256
  `47e842bdd5c33fcdce6d653fba8dda294376f4d882761fbeb415d5a648887c0c`, frozen
  verbatim below.
- Worker owns ONLY the 20 literal files in the ownership table. Manager retains
  `core/Cargo.toml`, `codex-rs/Cargo.lock`, `MODULE.bazel.lock`, BUILD files,
  plan/sprint documents and this allocation. Any other path returns for exact
  reallocation first.
- Accepted target **2700 total / 1050 non-test**; **STOP at 3000 total / 1150
  non-test** or any added path. Report actual counts including deletions.
- Implementation base: a fresh worktree branched from integration tip
  `cd78726cffecb4899d12055dfdd113dc2fb10141` (patch-equivalent to the accounting
  source tip and containing all received increments), so receiving is a clean
  merge. Exclusive Mac build target for this worker; no concurrent Core builds.
- Preserve every accepted Anthropic assertion, receipt and failed-run record.
  Default OFF; no TOML/CLI/env/Feature/TUI activation; no dependency, protocol,
  public config or lock change.
- Receiving gate: one independent fresh Fable review of the exact candidate;
  focused selectors with nonzero counts; combined `just test -p codex-core -p
  codex-api -p codex-state` on the receiving tree; `git diff --check`;
  plan/sprint checkers. Full-Core baseline comparison retained as in the
  Anthropic unit. No live/whole-sprint acceptance from this increment.
- Follow-ups listed in the proposal (WS, Chat, auxiliary routes, ChatGPT
  economics, S03/S04) stay unqualified and are not implied by this unit.

---

# PF-60-S02 next executable unit: accounted OpenAI Responses HTTP sampling

Design return for Fable manager. Action `responses-dispatch-design-01`; allocation digest `fe6bbc729834ffe68d58788fe4d14d8e29026d6f8c150f721e5d535bb72480dd`; claim `022575b0-4fde-4e17-88d5-a13e0fe5f77f`. Author runtime: gpt-6-astra / high. September 13, 2026.

## Status, authority and allocation prerequisites

This is a read-only source-design proposal, not source authorization or executed verification. Inspected worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-contract-goldens-20260913`, branch `workstream/accounting-contract-goldens-20260913`, HEAD `08eaae600dbbd468e4143f64a1ee2c28baf2acf4`; initial git status was clean. No repository writes, build, tests, credential files, provider calls or lock changes were performed.

The design task is routine preparation for the existing **product initiative**, PF-60. Exact product heading: **Measurement targets**, excerpt: “No commercial performance numbers have been supplied. The following metrics must be instrumented, with targets set through the decision rights defined above.” The active plan also retains its older Product measurement heading reference; use the actual heading above as the durable current citation. Plan: `docs/plans/active/portfolio-agent-cost-accounting.md`. Sprint: `docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s02-idempotent-usage-persistence-and-replay.md`, in_progress, feature PF-60, dependency S01 accepted/archived.

Read first: `docs/research/agent-cost-accounting/anthropic-dispatch-allocation.md`, `qa/portfolio/agent-cost-accounting/pf-60-s02/anthropic-dispatch-increment.md`, and `docs/research/agent-cost-accounting/contract.md`. The latter's acceptance update supersedes its historical pending wording. Preserve the Anthropic redirect and role-reload corrections, failed attempts, reviews and receiving limitations; none is a new pass for Responses.

**Before source dispatch, the manager must replace the current two-path original-contract-golden allocation with an exact same-S02 allocation.** The current plan/sprint still record base `81d0f90e77c1e9217a16e70fef1019ff9aa13753` and forbid runtime/Core writes for that old unit. Record acceptance of its predecessor, the new actual clean launch commit, matching plan/sprint worktree coordinates, this literal ownership list, size allowance and receiving gate. Run plan/sprint checkers at that time. This design does not update their shared records or assume the old allocation permits these edits.

## Proposed outcome and exclusions

Extend the existing normal-library, explicitly enabled collector to direct OpenAI API-key **Responses HTTP streaming** sampling, including an HTTP send reached by native WebSocket fallback. Durably admit each actual HTTP attempt with its original price binding; observe numeric wire presence before display conversion; retain owned evidence through retries, cancellation and reopen. Use the accepted state facade, arithmetic, schema and retention unchanged.

Eligible identity is configured provider ID `openai`, `WireApi::Responses`, an explicitly approved direct API endpoint, and a positively identified API-key auth route. A provider display name or an Authorization header is insufficient. The standard approved endpoint is `https://api.openai.com/v1`; an embedding can explicitly bind its isolated loopback endpoint for synthetic proof. Do not copy trust from a user-edited base URL.

WebSocket requests/prewarm themselves, Chat Completions, Corbanu/gateway/custom-provider collection, subscription/ChatGPT backend routes, and auxiliary requests are follow-ups. An HTTP fallback is recorded only for its HTTP segment; its preceding WebSocket work remains unmeasured. Never describe that logical request, thread or application as fully covered.

No public activation, TUI, TOML/env/CLI/Feature/schema switch, plan-price conversion, invoice ingestion, historical import, new registry, model-catalog update, migration, protocol event change or state engine rewrite.

## Inspected seams and implementation decisions

| Source / symbol at the frozen HEAD | Design consequence |
| --- | --- |
| `core/src/config/mod.rs::AccountingMode` | Currently Disabled or DirectAnthropic. Add one typed in-memory DirectOpenAiResponsesHttp variant with scope UUID and approved endpoint; ordinary loader stays Disabled. |
| `core/src/session/turn.rs::run_sampling_request` | Existing Anthropic context is created outside outer stream retries and checked before retry/success. Create one deferred Responses context at this same logical boundary; clear it with RAII on return/cancel. |
| `core/src/client.rs::ModelClientSession`, `stream_with_same_turn_attempt`, `stream_responses_api` | Responses may choose WS before HTTP. Attach a separate optional deferred Responses handle to this turn-scoped client, initialized to None in its constructor. Only stream_responses_api resolves it; never install state for a WS-only turn. |
| `model-provider/src/provider.rs::auth/api_provider`, `model-provider/src/auth.rs::resolve_provider_auth` | Existing auth and endpoint resolution are distinct. Inspect only resolved auth kind/config shape to identify the supported API-key branch; do not extract tokens or change provider auth code. |
| `core/src/agent/role.rs::reload::build_next_config` | The accepted runtime-provider overlay and wire validation are hardcoded to Anthropic. Extend mode-specific provider/wire matching narrowly; preserve explicit role fields, including retries, timeout, endpoint and WS capability. |
| `codex-api/src/endpoint/responses.rs::stream_request/stream_encoded` | Exact request model is available before EncodedJsonBody encoding. Add an optional typed observer to ResponsesClient and forward it only to HTTP SSE construction. |
| `codex-api/src/endpoint/session.rs::stream_encoded_json_with` | Prepared request is cloned per retry, auth applies inside that retry, then HttpTransport::stream executes. Reuse the accounting transport at this boundary; do not edit generic retry/session logic. |
| `core/src/memory_stage_one.rs::StageOneGuardedTransport::map_inner` | Already supports wrapping its inner transport. Keep auth -> existing stage-one guard -> accounting admission -> Reqwest send. No guard edits required. |
| `core/src/accounting_transport.rs::ResponseEvidence/AccountingTransport` | Existing response-local OnceLock binds the successful attempt below HTTP retries. Reuse it, add the Responses observer implementation, preserve failed intents and immutable old-stream attribution. |
| `codex-api/src/sse/responses.rs::process_sse_with_treatment` | Raw SSE is presently deserialized before response.usage handling; typed fields can lose absence or cause an event to be skipped. Observe raw numeric evidence first and await storage before downstream handling. |
| `ResponseCompletedUsage::into<TokenUsage>`, `ResponsesStreamEvent::token_usage` | Missing cache/write/reasoning details become zeros; response.usage can supply fallback completion usage. The collector must never reconstruct presence from these converted values. |
| `login/src/auth/default_client.rs::create_client_for_route_without_redirects` and `http-client/src/client_builder.rs` | Accepted no-redirect construction already preserves headers, UA, cookies, configured proxy/sandbox policy and fallible CA fallback. Reuse it without changes. |
| `state/src/runtime/accounting_types.rs::Dialect/Patch/replay` | Inclusive dialect already validates cache/output subsets and total consistency. Missing cache-write leaves noncached input unknown. No new dialect or schema is needed. |

Paths in this table are relative to codex-rs.

### Deferred activation without changing WS routing

Keep the accepted eager Anthropic lifecycle unchanged. A small private `DeferredResponsesSampling` in the new Core adapter owns one preallocated logical request UUID, the native session/turn reference, cloned mode and a once-initialized Sampling handle. It also owns a fail-visible latch for bootstrap errors. It is a sampling-local object, not another ledger or provider registry.

Construct/attach that object only for the matching mode/provider/wire in run_sampling_request. Creating it performs no database operation, rollout materialization, auth resolution or network I/O. On the first eligible HTTP call, after normal client setup identifies the supported auth route, it strictly materializes the native owner, requires StateDbHandle and activates the accepted store, then creates the existing Sampling with the preallocated UUID, provider openai, exact approved /responses URL and Dialect::Inclusive. Later HTTP retries reuse that same Sampling. Use existing tokio synchronization and dependencies; no new crate.

A WS-only execution never resolves it. If native WS fallback selects HTTP during that same sampling call, initialization occurs there before the HTTP send. Do not disable WS, mutate provider capability, prewarm, fallback budget or incremental WS state to simplify accounting. Test fixtures may explicitly configure HTTP as existing supported provider configuration.

Reuse the Sampling operation permit and explicit fresh-time policy. Once-initialization cancellation cannot manufacture successful activation, an attempt or a predecessor. Any bootstrap/admission/observation failure is latched and checked at the outer loop's existing failure boundary; no repair send or swallowed error. The RAII scope removes the deferred handle even when initialization or sampling is cancelled. Evidence already attached to a physical stream keeps only the accepted Sampling handle and may finish its in-progress atomic write under existing cancellation semantics.

## OFF invariant, auth and route binding

Disabled remains the derived/default variant and is explicitly initialized by the ordinary config loader. OFF creates no deferred context, resolves no new auth, installs/opens no accounting schema, parses no new usage and adds no accounting database requirement, including when optional state is absent. Existing installed-store deletion cleanup remains active while OFF.

DirectAnthropic opt-in does not enable OpenAI, and the new mode does not enable Anthropic or another configured provider. Unsupported wire/provider/auth paths retain existing behavior and acquire no measured-zero rows. A positive API-key route whose actual URL differs from its approved binding fails before admission/send; never silently fall back to unaccounted sending for this binding mismatch.

Use resolved `CodexAuth::ApiKey` / its existing AuthMode plus provider configuration checks, not header inspection or account labels. The ordinary provider-key path converts stored/env API keys to that existing type. Account only the branch whose effective auth is demonstrably that API key: custom command auth, experimental bearer override, header auth, agent identity and subscription backend are outside the new qualification. If an override could replace resolved API-key auth, exclude it rather than assuming its economics. Synthetic native tests inject the existing typed synthetic API-key auth through test builders; do not imitate API-key proof with an arbitrary bearer string. No new API in login/model-provider and no new credential access is needed.

Re-evaluate supported auth/routing on each new endpoint-client invocation. If an already initialized accounted logical request changes to an unsupported auth route, fail it visibly instead of continuing a partly tracked retry chain. A wholly unsupported request has no collector. A real 401 keeps native recovery behavior; API-key fixtures must prove the actual terminal/retry policy rather than assume a subscription refresh.

ON HTTP uses `create_client_for_route_without_redirects` inside the existing stage-one wrapper, with Class::Api and the actual resolved /responses URL. That constructor must remain the one used after fallback and retries. Keep the accepted 3xx TransportError::Build/fatal latch: 301/302/303/307/308 never cause a second endpoint send. OFF retains the ordinary client constructor and redirect behavior. Do not substitute bare reqwest clients, copy login routing logic into Core, loosen proxy/CA fallback, or alter safety/moderation processing.

Validate approved URL syntax as the Anthropic unit does: no userinfo, query or fragment; http/https only. Compare the full actual final request URL after auth against the bound path, including port/path/query. Do not authorize suffix matches, arbitrary hosts or redirects. Compare exact outbound request.model, captured after request construction/preparation, not ModelInfo display names or response-model hints. Current standard auth changes headers, not the body; no gateway/signing/custom-auth coverage is claimed.

## Raw usage contract

Add an optional numeric-only `ResponsesUsageObserver`, `ResponsesUsagePatch`, `ResponsesTokenPresence` and bounded invalid-evidence error in a separate API module. Missing, Null and exact nonnegative i64 Number remain distinct. Absent/null usage containers yield no numeric patch. Absent/null detail objects produce Missing/Null for their relevant fields; neither creates zero. Reject malformed containers and negative, fractional, string, boolean or overflow numeric fields while ON.

Observe these source locations before ResponsesStreamEvent deserialization and before any native conversion/error/completion:

- `response.completed.response.usage`.
- `response.failed.response.usage` and `response.incomplete.response.usage`, when supplied: consumed usage must survive terminal error handling.
- `response.usage.usage`, or its existing supported `response.usage.response.usage` form when the top-level container is absent/null. If both non-null containers occur, accept identical numeric evidence once; conflicting evidence fails visibly instead of selecting whichever parser happens to win.

Other lifecycle/content events do not become usage observations. Preserve existing OFF parser behavior, shared process_responses_event behavior, safety events, telemetry and public ResponseEvent/TokenUsage shapes. Keep the existing four-argument spawn_response_stream as the None-observer wrapper; add an internal observer-capable HTTP entry point. WS users of shared event processing must not acquire the observer.

| Raw numeric field | State patch | Semantics |
| --- | --- | --- |
| input_tokens | input | Inclusive input, not Anthropic noncached input. |
| input_tokens_details.cached_tokens | read | Cache-read subset of input. |
| input_tokens_details.cache_write_tokens | write | Preserve if explicitly supplied; absence is unknown. |
| output_tokens | output | Includes reasoning subset. |
| output_tokens_details.reasoning_tokens | reasoning | Never add again to output or price separately. |
| total_tokens | total | Preserve provider field presence; reject inconsistent known input+output via accepted reducer. |

Advance a checked monotonically increasing source position for each physical SSE data event, starting at one; recognized evidence retains that position as revision and sequence. Each successful physical stream has its own immutable source UUID from ResponseEvidence. Do not use provider IDs, arrival time, response sequence hints or callback completion order as durable identity. Identical replay of an observation uses the same source/position; cumulative later patches replace fields, not add totals. Null/omitted fields do not retract earlier known values.

Await observation persistence before forwarding corresponding usage-derived completion or terminal handling. Decode, validation or persistence failure calls the observer/latch, sends a bounded error and terminates the parser without a successful completion. No detached accounting queue. Cancellation/EOF/timeout retains the committed prefix or unknown admitted intent; no fabricated final counts.

The unchanged Inclusive reducer requires input/read/write to derive noncached input. Therefore ordinary Responses payloads without cache_write_tokens can have known input/read/output/reasoning but unknown noncached input and partial cost. This is intentional fidelity to the accepted contract. Do not insert a measured zero or alter state arithmetic to obtain a complete-looking estimate. If the product later accepts a provider-guaranteed structural zero, that needs a separate presence/provenance decision.

## Original price snapshot binding

Extend the existing bundled projection narrowly by provider and verified billing route. Preserve the Anthropic v1 source tuple/rates exactly. OpenAI must select one exact unique bundled slug with Eligible orchestration metadata and provider_id openai. Accept its Metered rates, or only the API-key side of AuthDependent after the route checks above. Local, Plan, PlanSchedule, disabled rows, duplicate rows, fuzzy aliases, remote-only models and unknown auth bind no price.

Observed repository examples: gpt-5.6-sol has AuthDependent API-key milli-USD rates 5000 input / 30000 output / 500 cache-read per million tokens. Project exact USD strings 5 / 30 / 0.5 without floats. gpt-6-astra is Disabled with manual-selection/unknown-economics metadata and has no eligible price; usage must still be collected, with unknown cost. These are frozen catalog facts, not verified live prices or model availability.

Use SourceKind::NativeCatalog and a new versioned UUIDv5 source tuple, e.g. openai-responses-http-apikey-bundled-v1, provider, exact model, units, billing branch, supported tier contract and raw rates. Bind the opaque endpoint/account scope UUID; persist no URL, key, account ID or fingerprint. Cache-write rate stays None. No reasoning surcharge. Zero known buckets need no rate under the existing engine.

Capture the final request.service_tier along with model for pricing eligibility. The first projection covers explicitly approved ordinary/default token estimates only: None/default request semantics may use the prospectively approved bundled default estimate; priority/flex/auto/unknown selections bind no snapshot and still record tokens. Do not infer a multiplier or change the request tier. These are estimates under a recorded default-tier assumption, never evidence of the provider's selected billed tier. Any later requirement to ingest server-selected tiers needs its own contract.

Create and persist the immutable snapshot and binding atomically at admission before the send. Capture/approval/effective-from are local prospective acceptance at that attempt, never backdated provider effectiveness. Every observation and two native reopens must retain exact original rates/source/snapshot identity. A later model-catalog override, different auth/config or newer price cannot reprice an admitted attempt, including an attempt originally bound to no price.

Literal golden: input 100, cached 20, cache-write explicit 0, output 40, reasoning 10, total 140, Sol default rates => noncached 80 and estimated token cost USD 0.00161. Omit cache-write instead => noncached unknown, read USD 0.00001 and output USD 0.0012 known, subtotal USD 0.00121, fully priced total null. These intentionally invented counts test frozen catalog arithmetic, not a bill. Nonzero cache-write without a rate also leaves the full estimate unknown.

## Exact proposed source/evidence ownership and size

Manager must reserve these **20 literal files** before implementation. Existing high-touch files receive only glue at the named seams; new runtime logic and new tests go in separate modules. Counts below are estimated added+removed lines, including both sides of replacement. All mixed existing glue and registrations are conservatively non-test; dedicated test/support bodies are test.

| # | Repository-relative path | Existing/new | Total / non-test estimate | Justification |
| --- | --- | --- | ---: | --- |
| 1 | codex-rs/core/src/config/mod.rs | edit | 10 / 10 | Add the internal OFF-preserving Responses mode. |
| 2 | codex-rs/core/src/agent/role.rs | edit | 24 / 24 | Mode-specific accepted runtime-provider overlay and wire checks. |
| 3 | codex-rs/core/src/session/turn.rs | edit | 30 / 30 | Sampling-local deferred handle, RAII cleanup and fatal-latch check. |
| 4 | codex-rs/core/src/client.rs | edit | 65 / 65 | Optional handle initialization and actual HTTP observer/transport wiring. |
| 5 | codex-rs/core/src/accounting.rs | edit | 75 / 75 | Register adapter; narrowly reuse admission/observation with provider/dialect/request identity; retain Anthropic wrappers. |
| 6 | codex-rs/core/src/accounting_transport.rs | edit | 30 / 30 | Responses observer on existing immutable response evidence; carry supported price eligibility per physical request. |
| 7 | codex-rs/core/src/accounting_prices.rs | edit | 75 / 75 | Exact OpenAI API-key projection and supported tier gate, preserve Anthropic tuple. |
| 8 | codex-rs/core/src/accounting_prices_tests.rs | edit | 70 / 0 | Literal OpenAI price, unknown row/tier and Anthropic provenance regressions. |
| 9 | codex-rs/core/tests/suite/mod.rs | edit | 2 / 2 | Register two native Responses test modules. |
| 10 | codex-rs/codex-api/src/lib.rs | edit | 5 / 5 | Export only the small numeric observer surface needed by Core. |
| 11 | codex-rs/codex-api/src/endpoint/responses.rs | edit | 25 / 25 | Optional observer builder, default None, HTTP-only forwarding and module registration. |
| 12 | codex-rs/codex-api/src/sse/responses.rs | edit | 45 / 45 | Observer-capable spawn wrapper and awaited pre-conversion hook/source positions. |
| 13 | codex-rs/core/src/accounting_responses.rs | new | 175 / 175 | Deferred native bootstrap, route eligibility and numeric-to-state adapter, separate from engine. |
| 14 | codex-rs/core/src/accounting_responses_tests.rs | new | 220 / 0 | Bootstrap/failure/ownership/auth/scope unit proof. |
| 15 | codex-rs/codex-api/src/endpoint/responses_accounting.rs | new | 140 / 140 | Whitelisted presence decoder and observer DTO/trait; no Core/state dependency. |
| 16 | codex-rs/codex-api/src/endpoint/responses_accounting_tests.rs | new | 300 / 0 | Raw SSE presence, terminal evidence, ordering and callback barrier tests. |
| 17 | codex-rs/core/tests/suite/accounting_responses.rs | new | 360 / 0 | Actual native Responses sampling/price/child/exclusion goldens. |
| 18 | codex-rs/core/tests/suite/accounting_responses_recovery.rs | new | 380 / 0 | HTTP/stream retries, redirects, cancel/reopen/delete and fallback. |
| 19 | codex-rs/core/tests/suite/accounting_responses_support.rs | new | 200 / 0 | Bounded synthetic endpoint, barrier/readback helpers, reuse existing common helpers. |
| 20 | qa/portfolio/agent-cost-accounting/pf-60-s02/responses-dispatch-increment.md | new | 110 / 110 | Exact implementation/receiving evidence and explicit limitations. |

Estimate **2441 total / 911 non-test / 1530 test**. Proposed target **2700 total / 1050 non-test**; **STOP before exceeding 3000 total or 1150 non-test**, or adding any path. These are requested manager bounds, not inherited permission. Count all actual new-file lines and deletions, receipts and compatibility glue. Report a concrete scope/size problem before crossing; do not compress tests, remove assertions or conceal mixed-file code as test.

This exceeds the normal 800-line guidance because the smallest useful native increment crosses lossless SSE capture, per-HTTP admission, lazy fallback/ownership, exact price binding and real retry/reopen evidence. A parser-only stage would not provide accounted native requests. The existing ledger/transport/builders avoid repeating the prior 3900-line Anthropic/store project. Manager should accept this bounded coherent exception explicitly or return a smaller contract before implementation.

No edits to login/http-client, EndpointSession, generic retry, model-provider/auth, memory_stage_one, protocol, state, accepted Anthropic source/tests/receipts beyond the enumerated shared adapter files, catalog, manifests, Cargo.lock, MODULE.bazel.lock or BUILD files. Existing dependencies include SQLx dev proof and tokio/UUID/state/API/models-manager. New modules use explicit path registrations and in-code synthetic data; existing Bazel Rust macro covers sibling sources, so no new compile_data is proposed. Unexpected dependency/BUILD need is a STOP and reallocation, not permission to modify locks.

Manager-owned preparation, outside the worker list: update the existing active plan and current sprint and create `docs/research/agent-cost-accounting/responses-dispatch-allocation.md` to freeze this mandate. Record those governance deltas separately and in the combined review ledger; do not treat them as worker writes or hide them from integration review.

## Focused test plan: named cases and counts

All cases below are future tests. **No test has been run for this proposal.** Names form the required selector manifest. Expected minimum new runnable functions: **8 API + 8 Core unit + 18 native integration = 34**, all selected and executed; loop vectors are not extra tests. One price function in the eight unit cases lives in the existing prices test file; seven in the new adapter test file. Extra discovered requirements return to the manager if they exceed bounds.

### API: eight functions

1. `responses_accounting_presence_matrix`: all six fields Missing/Null/0/positive/i64MAX; malformed primitive/range vectors fail; no leakage of provider body in the bounded error.
2. `responses_accounting_completed_before_lossy_conversion`: raw partial input/details survive even when native completed parsing cannot represent that partial shape; persist the patch, retain native error behavior.
3. `responses_accounting_separate_usage_containers`: both supported response.usage locations, identical dual evidence once, conflict rejected, absent/null containers no invented counts.
4. `responses_accounting_terminal_usage_retained`: failed/incomplete numeric usage commits before terminal handling, including immediate fatal error paths.
5. `responses_accounting_positions_and_replacement`: source positions include intervening non-usage events; known->null/omitted preserves known; later cumulative count replaces; identical repeated snapshot never doubles totals.
6. `responses_accounting_awaits_observation_before_completion`: held observer prevents Completed; release permits exactly one; rejection permits none.
7. `responses_accounting_invalid_usage_latches_and_stops`: invalid evidence reaches observer error path and terminates; no later successful completion.
8. `responses_accounting_none_preserves_legacy_stream`: same payload sequence with no observer retains native display/error/events, including safety metadata; no new callback/parsing branch.

### Core: eight functions

1. `accounting_responses_bootstrap_is_lazy_and_once`: pending context has one UUID, no installed accounting before HTTP; one successful bootstrap reused across invocations.
2. `accounting_responses_auth_route_eligibility`: API-key positive, unsupported command/bearer/header/ChatGPT/agent-identity/other-provider negatives; changing an initialized route fails.
3. `accounting_responses_auth_and_guard_precede_admission`: real asynchronous synthetic AuthProvider denial and existing stage-one denial => zero transport calls and zero intents.
4. `accounting_responses_bootstrap_failure_and_cancellation`: missing owner/state, failed activation, waiting cancellation and once-init error preserve fatal/unknown semantics without sends.
5. `accounting_responses_scope_cleanup_and_failure_latch`: outer checks see API observer failures; cancellation/drop clears scope; subsequent auxiliary call has no observer.
6. `accounting_responses_response_local_identity`: old delayed response writes to old attempt after a newer retry; distinct source/attempt IDs and stable logical UUID.
7. `accounting_responses_role_overlay_preserves_binding`: full-provider equality preserves explicit retries/timeouts/endpoint/WS fields; incompatible wire rejected; inherited mode unchanged; OFF behavior preserved.
8. `accounting_responses_prices_exact_and_unknown`: Sol literal rates/prospective tuple, Astra absent economics, duplicate/remote/alias/Plan/Local/unknown auth/tier rejected, zero-vs-absent rate rules; Anthropic source identity remains unchanged.

### Native integration: eighteen functions

Use TestCodexBuilder::build_with_auto_env, existing core_test_support::responses and actual Op::UserTurn, native spawn/resume/delete. Assert captured POST method/path/model/body and separate-connection typed attempt/observation/quote/DayTotals readback. Never seed attempts as a substitute for actual sends.

1. `accounting_responses_native_off_and_unsupported`: ordinary OFF with and without optional state; other provider/wire modes; no collector/schema/rows, expected ordinary request counts.
2. `accounting_responses_native_complete_and_partial_goldens`: actual POST fixtures for the two literal 100/20/0-or-absent/40/10/140 cases and USD 0.00161 / partial USD 0.00121-null.
3. `accounting_responses_native_separate_and_terminal_usage`: separate usage then failed/incomplete/complete variants retain only actual committed patches; actual terminal behavior remains visible.
4. `accounting_responses_native_http_retry_policy`: 503 then success => two POSTs, two chained attempts, first unknown; default OpenAI 429 terminal => one POST. Do not modify native retry_429 policy.
5. `accounting_responses_native_outer_retry_prefix`: committed usage then EOF, later successful retry => two attempts/sources in one logical request; exact prefix plus final-attempt totals, no cumulative double charge.
6. `accounting_responses_native_api_key_401`: capture actual native API-key terminal behavior and one-send unknown intent where existing policy is terminal; no invented refresh.
7. `accounting_responses_native_redirects`: two real endpoints, all five redirect codes; ON one origin POST/zero target requests/one unknown intent/zero observations/no repair; OFF follows its existing policy and installs nothing.
8. `accounting_responses_native_endpoint_mismatch`: unapproved endpoint/path/query/port under same provider rejected with zero POSTs and no admitted attempt.
9. `accounting_responses_native_admission_barrier`: separate SQLite writer lock blocks send; release permits one; injected activation/admission failure yields zero sends and no retry loop.
10. `accounting_responses_native_observation_failure`: one actual send then observation trigger failure; previous committed evidence unchanged, no Completed/no repair POST.
11. `accounting_responses_native_cancel_two_reopens`: cancel before usage and after committed usage; two actual native resumes preserve exact original IDs/prices/unknowns and generate no old sends; fresh turn gets a new request UUID.
12. `accounting_responses_native_spawned_role_children`: root plus two real children, one role reloaded, held concurrent sends and exact native ownership/spawn edges; explicit retry override honored; fork/history alone adds no charge.
13. `accounting_responses_native_delete_rejects_late_usage`: public deletion while a stream is held; late observation cannot resurrect evidence, unrelated native owner intact, installed-but-OFF deletion unchanged.
14. `accounting_responses_native_ws_fallback_http_segment`: actual existing fallback trigger using synthetic WS endpoint and captured HTTP POST; zero WS attempts in ledger, one admitted HTTP attempt, HTTP usage only and explicit incomplete whole-call coverage.
15. `accounting_responses_native_ws_only_no_install`: synthetic WS success with mode enabled leaves accounting uninstalled and no rows; preserve WS routing/prewarm behavior.
16. `accounting_responses_native_unknown_prices_and_tiers`: Astra/remote-only/priority-tier request records nonzero tokens and original null price; changing later catalog/config cannot populate old binding.
17. `accounting_responses_native_sampling_and_auxiliary_scope`: two sampling calls in one native turn get distinct request UUIDs; reused client for supported existing auxiliary fixture gets no observer or extra accounting send.
18. `accounting_responses_native_no_usage_is_unknown`: success without usage and exhausted stream failure retain admitted unknowns, never measured zeros; optional/reused provider IDs cannot collapse distinct attempts.

The native WS tests require real synthetic WS/fallback controls from existing test support, not fake AccountingStore insertion. If current support cannot trigger a bounded fallback without expanding scope, record that concrete prerequisite and return to the manager; do not mark it out of scope or pass it with a direct HTTP call.

## Receiving verification and exit evidence

The receiving owner must verify source, independent review and final combined-tree execution, not just accept a worker return:

1. Record source base, worker commit, receiving commit, branch/worktree, exact 20-file manifest, additions/deletions and conservative non-test count, full-file hashes and binary/diff provenance. Verify no extra files or lock/dependency changes. Preserve initial failed attempts and any corrective review history.
2. Independently inspect ON/OFF and auth predicates, deferred-only HTTP activation, role overlay, URL comparison after auth, no-redirect construction at every HTTP invocation, actual model/tier capture, observer placement before lossy parsing, response-local attempt binding and permanent original price/null binding. Verify no safety/auth/guard bypass.
3. Run repository fix/format procedures before final tests, constrained to authorized scope/build lease. Record exact toolchain, target, commands, exits, run IDs, runnable-name lists, passed/failed/skipped/leaky counts and raw artifacts. Use just test, not cargo test. Explicitly list tests before executing selectors so a zero-match result cannot pass.
4. Focused commands, under the receiving owner's locked/offline build setup: `just test -p codex-api responses_accounting --locked` (at least 8 new executions); `just test -p codex-core --lib accounting_responses --locked` (at least 8); `just test -p codex-core --test all accounting_responses --locked` (at least 18). Report actual counts if names/module grouping change and map every frozen case.
5. Run affected codex-api and codex-core crate suites on the final tree, plus accepted accounting/Anthropic, role/stage-one and Responses/WS compatibility selectors. Re-run retained login/http-client no-redirect/fallible-CA regression selectors (expected at least the three named existing no-redirect/fallible cases, no zero-match acceptance). Reuse their code; do not author a second policy test project.
6. On the combined receiving tree run the established shared state/TaskNode session gate, e.g. `just test -p codex-state -p codex-tasknode-session --locked`, and normal-library compilation/governance/whitespace checks. Counts are discovered on the actual receiving tree, not copied from old 473/575/595 receipts. Retain historical full-Core and shared failures; a recurrence or matched name is not a waiver or identical-cause proof. Full-workspace testing follows the repository's separate authorization rule.
7. Preserve actual local HTTP/WS request counts, typed native SQLite readback, barrier timing, old/new attempt/source identity, two-reopen snapshots and literal money/unknown assertions. No credentials, headers, prompts, tool bodies or unbounded response content in accounting artifacts; synthetic fixture payloads may be retained as fixture evidence.
8. Receive one new independent material code review plus necessary scoped corrective review under the manager's recorded allowance, preserving prior usage. Do not seek repeated opinions on unchanged clean state/builders. Scope/size or substantive recurrence returns to integrator; no worker self-review approval.
9. This source-informed proposal is **not code-blind functional design**. Proposed N/A for this internal, non-public opt-in stage requires named integrator acceptance and must name the later S03/S04 functional gate. No human-test readiness, native-product completeness, live-provider, TensorCash/Isometric or release qualification is claimed. At public activation/affected workflow handoff, require the separate frozen code-blind design, confined exact-package executor, independent evidence review and true TUI keys, with applicable live repositories.

## Reserved-lane overlap and follow-ups

PF-27's inspected September13 system-preflight allocation reserves secret-broker-service launch/manifest files and its security evidence, with no Core/Vault/lock expansion. This proposal touches none of its seven literal paths. The existing security owner retains auth/broker authority; changes to credential resolution, protected transport, sandbox policy or guard semantics are not authorized by accounting. Coordinate any discovered issue with that owner and stop the overlapping edit.

Task Node's current owned-ingress allocation reserves ten files under codex-rs/responses-api-proxy plus manager-serialized dependency/lock/build integration. Responses API proxy is a distinct crate from codex-api: do not use or edit its new synthetic transport to test this collector. Shared Mac build target/locks must be leased serially. Existing Slack supervision paths under scripts/initiative_control and its receipts are also excluded; no live Slack/Task Node messages or credential access.

Core client/config/role/turn and API SSE files are high-touch upstream seams even when current listed reservations are disjoint. Manager must compare live allocation records/diffs at dispatch and serialize any new collision, including provider-reauth-health work. This design's source inspection is not a claim that all external worktrees are now idle. No public protocol/config change or dependency refresh is required.

Follow-ups, explicitly unqualified by this unit:

- Responses WebSocket prewarm, incremental requests, reconnects and all WS attempt/token accounting; HTTP fallback preceding-WS coverage remains unknown.
- Chat Completions, including Corbanu API and compatible gateway semantics, account balances and gateway settlement distinctions.
- Auxiliary HTTP routes: compaction, memory/stage-one summaries, realtime, models discovery and other non-sampling API calls; no observer leakage into them.
- ChatGPT/subscription/agent-identity/header/command-auth economics, custom/compatible endpoints, regional/service-tier pricing and non-token/tool/media charges.
- Provider-authoritative structural cache-write zero, if desired; historical usage acquisition/import, invoice reconciliation, retrospectively consented estimates and complete application coverage.
- Public activation, S03 display, S04 independent functional/live-repository/TUI qualification and human acceptance.

The manager can turn this proposal into the next executable same-S02 allocation after the predecessor, literal reservations, bounds and receiving requirements are recorded. Until then it remains a private design artifact.
