# PF-60-S02 next executable unit: direct OpenAI Chat Completions sampling

Private design proposal for the Fable receiving manager, September 15, 2026.
Action: acct-chat-design-01
Allocation digest: db5437543f7eb22e39aef666704d5e4bf9f856a62741bf075129ece189cd2b8c
Claim: 654f6d77-677c-476d-9473-b1d4cf8ba72f
Designer: gpt-6-astra / high
Inspected base: 70b7cd33f9a17c1cbb219f4d0d7b00d4a6cf5acf
Inspected worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911
Inspected branch: integrate/management-workstreams-20260911

This is source-informed design, not implementation, independent code-blind test design, manager acceptance or a build lease. Repository access was read-only. No builds, tests, provider calls, credential reads, subagents, commits or pushes were performed. The only output is this private file. Read-only discovery included a missing illustrative startup-prewarm path; the actual core/src/session_startup_prewarm.rs was subsequently located and inspected.

## 1. Decision and authority

Choose **(a): direct OpenAI API-key Chat wire sampling accounting**, excluding all Corbanu plan gateway collection and economics. Keep the two disclosed P3s and prewarm/auxiliary collection as explicit subsequent work.

This is routine design preparation for the existing PF-60 **product initiative**. Exact product-spec heading: **Measurement targets**. Requirement excerpt: “No commercial performance numbers have been supplied. The following metrics must be instrumented, with targets set through the decision rights defined above.”

Plan: docs/plans/active/portfolio-agent-cost-accounting.md, active.
Sprint: docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s02-idempotent-usage-persistence-and-replay.md, in_progress, feature PF-60.
The contract's September 11 acceptance update supersedes its historical pending-policy prose. No new product authorization, live activation or billing authority is inferred.

Received predecessors, verified by local commit metadata and the current sprint:
- WS sampling: c86634e211633e68b84ea15b2f6faf15ea376868, receiving 50382ef13.
- Two native vectors and corrections: 16c43b5feab145604b855071142a8025c2d0bcbb, receiving 99b6e0d2c.
- Existing HTTP, original-contract, store, retention and native-owner receipts remain dependencies. Their counts, failures, corrections and limitations are historical evidence, not tests run here.

The current sprint front matter still describes the consumed 18-file WS allocation and old implementation coordinates. Before implementation the manager must freeze this successor, update the plan/sprint consistently to an actual clean worktree/branch/full base SHA, reconcile consumed mandates, record the size exception and exclusive target lease, check actual reservations/diffs, and run plan/sprint checkers. This private proposal assigns no invented implementation checkout. Keep S02 open and S03 dependent.

### Why this unit, rather than option (b)

1. Chat uses the same EndpointSession HTTP retry seam already covered by AccountingTransport. It needs no WS pump, connection provenance, fallback state machine or schema change. The new uncertainty is lossless Chat usage capture and explicit direct-route price provenance; both have narrow seams.
2. The accepted original-contract native golden already maps Chat's five numeric fields to Inclusive with write Missing. Therefore the next adapter can exercise actual dispatch against an already accepted numeric contract without inventing cache-write evidence.
3. The provider-publication P3 concerns WS Admission::check -> StageOneMemoryBinding::check_stream, which compares the published ModelClient while update_settings has already changed session_configuration but has not yet replaced that client. Chat's retained pre-send StageOneGuardedTransport uses the full asynchronous binding.check path. This proposal neither imports the per-frame WS check into Chat nor claims to close its publication window.
4. The consuming with_stage_one_memory_binding builder is now a shared OnceLock setter returning a typed error. Its naming/signature is misleading after the clone fix, but it is not needed to capture Chat usage. Renaming it and updating memory/fixture callers is a separate narrow cleanup.
5. Startup warmup is a real generate=false response.create before run_sampling_request ownership, and memory/compaction use separate client sessions. Merely classifying these operations would not collect them. A prewarm collector needs an explicit startup owner/turn lifecycle and price/usage attribution decision; a general auxiliary collector needs per-operation boundaries. Attaching them opportunistically to the next sampling request is wrong.

The two P3s remain non-blocking as recorded, not dismissed or declared fixed. If later independent review finds a material dependency on them, stop and return a concrete dependency to the manager. Do not expand this unit into a policy-publication repair. Complete application accounting remains unqualified.

## 2. Code findings and resolved predecessor concerns

Paths in this table are relative to codex-rs at the inspected base.

| Source / symbol | Observed behavior | Frozen decision |
| --- | --- | --- |
| core/src/client.rs::ModelClientSession::stream_chat_completions_api, approximately 2989-3130 | Generates the plan request header once before its auth-recovery loop; outer sampling retry calls the method again. | Never use x-pfterminal-request-id, x-client-request-id, response IDs, call-sequence counters or attempt-index telemetry as a durable accounting key. Allocate request UUID outside outer retries; admit each physical POST separately. Leave gateway headers untouched. |
| codex-api/src/telemetry.rs::run_with_request_telemetry | Independently rotates Corbanu headers only after a matching released-reservation response; uncertain outcomes retain server idempotency identity. | Preserve this behavior and its existing regression. It is further evidence that a transport header is not the local logical/physical accounting identity. No telemetry edit is allocated. |
| codex-api/src/endpoint/chat_completions.rs::ChatCompletionsClient::stream_request | Encodes the final request, then uses EndpointSession::stream_encoded_json_with; starts an SSE task after HTTP headers. | Add a None-default numeric observer, pass it to the parser, and reuse the existing response-local transport binding below HTTP retries. |
| codex-api/src/endpoint/session.rs::stream_encoded_json_with | Clones a prepared request per retry, applies auth, then calls transport.stream. | Auth -> existing stage-one guard -> durable accounting admission -> original transport send. No generic retry/session edits. |
| codex-api/src/endpoint/chat_completions.rs::process_chat_sse, approximately 1118-1266 | [DONE] completes; error envelopes terminate before typed chunks; malformed typed chunks otherwise skip. | Advance source position on each delivered data event; treat [DONE] as a nonnumeric sentinel. For all other events decode/await top-level usage BEFORE error-envelope handling and typed deserialization. A top-level usage object beside error is retained before the error. |
| ChatUsage / From<ChatUsage> for TokenUsage, approximately 266-307; ChatStreamState::process_chunk | Missing numbers become zero; conversion fixes write to zero; later usage replaces native display usage. | A separate five-field presence DTO reads wire evidence. Collector never reads converted TokenUsage. Cache-write is always Missing in this supported dialect, including when all five reported values are explicit zero. |
| core/src/client.rs::build_chat_completions_request; codex-api/src/common.rs::ChatCompletionsRequest | Sends stream_options.include_usage=true; exact outgoing model can differ from a display slug. DTO has no service_tier. provider/plugins/providerOptions are possible routing extensions. | Preserve request generation. Use final serialized model and an explicit no-tier-on-wire/default estimate assumption. Exclude routing extensions. Do not price a UI-selected priority tier that Chat did not serialize. |
| model-provider-info/src/lib.rs::create_pfterminal_plan_provider, approximately 1271-1301 | Chat wire, gateway-specific env-key route, configurable CORBANU_API_BASE_URL / legacy fallback endpoint, requires_openai_auth=false. | Exclude the entire gateway route even when model spelling or API-key auth matches OpenAI. A gateway pointed at a direct-looking endpoint still does not inherit direct economics. No credential/env lookup is added. |
| wallet/src/corbanu_api.rs::CorbanuApiPricing / CorbanuApiBalance | Pricing has four decimal strings and version; balance/reserved/available have separate exact strings. No effective interval or request settlement attribution. | Read neither DTO at collection time. Never bind direct bundled prices to the gateway, backdate pricing.version, derive task spend from balance deltas, or label token estimates billed. |
| core/src/accounting.rs / accounting_transport.rs | Native owner, atomic admitted attempt and original snapshot, serialized writes, failure/cancellation latch, successful-response OnceLock. | Reuse; add only a typed Chat pricing/wire selection and Chat observer adaptation. Failed HTTP attempts stay unknown and are not collapsed. |
| state/src/runtime/accounting_types.rs::Dialect / Attempt / replay | Inclusive already validates known subsets/totals. Attempt stores opaque scope and arithmetic dialect, not a literal wire or URL. | Reuse unchanged. Do not claim this adapter repairs every descriptive S01 provenance field. Freeze the Chat route/scope association in receiving evidence; full persisted wire/endpoint provenance remains a disclosed later coverage issue. |
| core/src/agent/role.rs::reload::build_next_config | Runtime accounting mode survives role reload; built-in openai TOML override is rejected before the overlay. | Add Chat to internal inheritance/wire matching only. Instruction-only role is positive; reserved-provider override remains negative. |

The design inspected the S02 receipt set, including state/price/retention/native/import, policy-repair, HTTP, WS and original-golden evidence. Relevant retained lessons: proof names do not establish every vector; do not require both SQLite race winners by chance; preserve raw failed runs and mismatched JUnit-copy history; share immutable original/null price authority; no physical erasure or permanent 365-day identity-fence claim.

## 3. Frozen outcome, OFF invariant and eligibility

Add a separate internal in-memory mode:

    AccountingMode::DirectOpenAiChat { scope, approved_endpoint }

Existing Disabled, DirectAnthropic, DirectOpenAiResponsesHttp and DirectOpenAiResponses retain their meanings. The normal loader remains Disabled. No ConfigToml/schema, Feature, CLI, environment, TUI or app-server activation.

OFF:
- Creates no Chat sampling context, accounting installation/open, numeric decoder invocation or observation.
- Requires no accounting database, including ordinary profiles without optional state.
- Resolves no extra auth and makes no additional request.
- Keeps legacy parsing, errors, metadata, request shape, timeout and redirect behavior.
- Preserves already-installed accounting cleanup on public deletion while collection is OFF.

At run_sampling_request, attach the new deferred context only when the internal mode matches, configured model_provider_id is exactly openai, and the turn provider wire is exactly Chat. A deferred context creates a UUID only, not state or credentials. Initialize its native owner/store only after normal client setup qualifies the actual route.

All must be true to collect:
1. Above internal mode/provider-ID/wire predicate.
2. Resolved existing auth is positively typed CodexAuth::ApiKey; no AgentIdentity telemetry/route.
3. No provider command auth, experimental bearer override, custom API-key header, authorization/api-key HTTP-header or env-header override. Check header NAMES without reading configured values.
4. No Corbanu plan identity, compatible/gateway provider identity, ChatGPT/subscription, Bedrock, header-auth or anonymous route. Require the normal direct OpenAI provider identity in addition to configured ID; a label alone is insufficient.
5. No chat_completions_provider routing override; outgoing provider, provider_options and plugins are absent. Do not let an unusual request body silently route through a gateway while retaining direct prices.
6. An embedding explicitly supplies the approved direct endpoint and opaque account/endpoint scope. Normal expected base is https://api.openai.com/v1. The local test embedding may bind its exact loopback fixture. User-edited base URLs do not self-authorize a scope.

Re-evaluate qualification after normal setup for each endpoint-client invocation. A wholly unsupported operation proceeds with native behavior and no collector; its coverage is unknown, not zero. Once a logical request has been initialized for accounting, changing to an unsupported route or violating its binding is fatal: never continue its retries unaccounted. A positively eligible route with wrong endpoint is rejected before initialization/admission/send. Distinguish unsupported from binding failure.

Use a separate opaque scope for direct Chat versus gateway/other endpoint/account associations in the native fixtures. Do not derive a scope from tokens, fingerprints or wallet addresses. The existing facade validates scope identity, not human authority to approve a route. No new scope registry is proposed.

## 4. Implementation sequence and mechanical contract

### A. Small private deferred adapter

New core/src/accounting_chat.rs owns DeferredChatSampling, its private slot/RAII scope, route predicate and Chat-to-state patch mapping. It mirrors the accepted deferred lifecycle without renaming or generalizing the WS/Responses adapter.

Allocate one logical request UUID before the outer retry loop. Resolve once using existing native rollout materialization and StateDbHandle. Own the same Sampling throughout HTTP transport retry, terminal stream retry and any actual supported auth recovery. Distinct sampling calls in one turn get distinct request UUIDs. Clear the slot on success, failure and cancellation; an auxiliary call on a reused client cannot inherit it.

The deferred bootstrap guard and Sampling::Completion must latch failures/cancellation even if SQL may have committed. Outer run_sampling_request checks the latch before success/retry handling. Do not infer a predecessor after an uncertain admission or spawn a detached write task.

### B. Bound HTTP transport and identities

Create ResponseEvidence for each new Chat endpoint-client invocation inside the auth-recovery loop. Reuse AccountingTransport inside StageOneGuardedTransport::map_inner. EndpointSession retains its normal prepared-request/auth/retry behavior.

Immediately before each physical transport.stream, validate the final post-auth URL, durably admit a fresh attempt UUID and its immutable original/null price binding, then call the original transport. Attempts share logical request, owner and chained retry_of. A 503/429/401 or uncertain send may have consumed resources; its admitted intent stays unknown. An HTTP transport failure leaves the successful-response OnceLock empty so an allowed retry can admit a fresh attempt. The response that succeeds binds exactly its own attempt/source; later retries cannot overwrite an old observer.

Do not alter headers to implement identity, and do not deduplicate by repeated/missing provider IDs. Admission establishes a durable dispatch intent, not proof that a network packet reached the server. Preserve this crash/cancellation window in evidence.

ON uses create_client_for_route_without_redirects with ClientRouteClass::Api and the existing HTTP client factory. Retain proxy, sandbox route, default headers/UA/cookies and fallible custom-CA behavior. No bare reqwest client, new dialer, redirect policy copy or guard change.

Reject userinfo, query, fragment and non-http(s) approved bases. Compare the complete final URL against the exact approved /chat/completions path, including scheme, host, port and path. No suffix matching, query allowance, /responses alias or redirect repair. Existing AccountingTransport's redirection rejection must latch before outer retry can send again. Prove 301/302/303/307/308 with two actual local endpoints and retries enabled. OFF retains its original redirect semantics.

### C. Numeric observer and parser

New API module chat_accounting.rs defines documented ChatUsageObserver, ChatUsagePatch, ChatTokenPresence and bounded InvalidChatUsage. Keep it independent of Core/state. Export only those through codex-api/src/lib.rs.

Whitelist exactly:
- usage.prompt_tokens -> input
- usage.prompt_tokens_details.cached_tokens -> read
- usage.completion_tokens -> output
- usage.completion_tokens_details.reasoning_tokens -> reasoning
- usage.total_tokens -> total
- state write -> Missing, always

Capture only top-level usage in an SSE JSON object, including a usage-only chunk, normal chunk or top-level error envelope with sibling usage. Do not decode nested error.usage, a Responses-style response.usage, OpenRouter billing/cost fields, gateway settlement fields or vendor cache-write extensions as this dialect's evidence. Unknown extra keys do not become evidence.

Missing versus Null versus explicit nonnegative i64 Number remain distinct. Absent/null usage means no observation; an empty usage object may preserve an all-Missing observation, never six zeros. Missing/null detail object propagates its appropriate absence to the supported child. Reject malformed containers, negative, fractional, string, boolean and overflowing supported values. An invalid JSON data event while ON is a bounded accounting failure; OFF retains its original skip behavior. Never include response bodies in accounting errors.

Increment a checked response-local source position for every delivered SSE data event, starting at one, including nonusage chunks and [DONE]. Comments/byte keepalives are transport activity, not positions. [DONE] creates no usage; do not JSON-decode it. Persist each valid numeric patch before ChatErrorEnvelope handling, ChatCompletionChunk parsing or state.process_chunk. At [DONE], all prior evidence is already committed before state.complete. Finish_reason is not usage or settlement.

Implement ResponseEvidence's Chat observer using the existing observe_patch. Every response has one immutable source UUID; revision/sequence use the original data-event position. Repeated cumulative chunks create later revisions but replace fields, never add snapshots as new spend. Null/omitted patches retain prior numeric evidence. Impossible known cache/input, reasoning/output or total sums fail through the unchanged Inclusive reducer.

Await persistence before forwarding the corresponding event. On invalid evidence or store rejection, latch and terminate with no Completed or repair POST. An accounted parser must stop when its consumer closes, including while awaiting its next data event/observation; cancellation can leave a committed prefix or unknown intent. Reuse the existing completion guards; no new detached collector. Keep the ordinary None-observer entry/wrapper so existing inline parser tests need no wholesale rewrite.

Preserve the native ChatUsage conversion and public TokenUsage/ResponseEvent shapes. Accounting can retain valid raw partial fields even when the legacy typed chunk cannot deserialize them. Do not substitute its partial patch into UI totals in this S02 unit.

### D. Immutable price snapshots

Add a narrowly typed internal wire/pricing discriminator to Sampling. openai alone must no longer select responses_original indiscriminately. Anthropic and Responses retain their existing projection/source bytes. Chat selects chat_original, not a gateway catalog.

Reuse exact bundled-row selection and integer milli-USD conversion. Accept only one exact outgoing model slug, Eligible orchestration, provider_id openai, Metered or the API-key branch of AuthDependent. Disabled/Astra unknown economics, duplicates, aliases, remote-only models, Plan/PlanSchedule/Local and missing rates remain unpriced/partial. Do not mutate the catalog or query models/account endpoints.

New source tuple, canonical JSON serialized then UUIDv5 as existing code does:

    ("openai-chat-api-key-bundled-v1", "openai", exact_model,
     "api_key", "default", "USD/million", input_milli, output_milli, cached_input_milli)

The Chat DTO has no service_tier. This tuple explicitly labels a prospective default-tier TOKEN ESTIMATE under the existing bundled-price authority, not evidence of actual service-tier settlement. Do not add a service-tier field to the request or read Config.service_tier as though it were sent. If a later change allows a tier/routing extension on wire, qualify it separately; do not accidentally price it through this branch.

Capture/admit atomically at dispatch: fresh snapshot ID; opaque scope; exact model; NativeCatalog; exact decimal rates; observed_at=approved_at=effective_from equal admission time; effective_end None. These are local prospective acceptance times, not a provider's historical effective dates. Bind a snapshot or explicit null once per attempt. Later catalog/config changes and two native reopens cannot fill null or rewrite old rates. No replay-time pricing, historical rebilling, settlement or balance adjustment.

Cache-write count and rate stay unknown. Inclusive noncached input therefore stays unknown for this Chat dialect. Read/output known subtotals can be estimated; a full request estimate remains null. Do not borrow the WS complete 0.00161 golden by fabricating write=0.

Frozen literal goldens, using bundled fixture rates input=5/output=30/read=0.5 USD per million (not a live-price claim):
- C1: prompt=100, cached=20, completion=40, reasoning=10, total=140. Known read USD=0.00001, output USD=0.0012, subtotal=0.00121; write/noncached/full estimate unknown.
- C2: same, cached omitted. Known subtotal=0.0012; read/write/noncached/full estimate unknown.
- C3: two distinct C1 attempts: known subtotal=0.00242 and two unknown full estimates. Repeated identical cumulative C1 within ONE attempt stays 0.00121 with one unknown full estimate.
- C4: all FIVE supported numbers explicitly 0: those fields are measured zero, write and noncached unknown, known subtotal 0, full estimate null. A missing usage response has no observations and six unknown raw metrics; it is not C4.
All seven aggregate metric populations must be asserted, not only the money strings.

## 5. Exact proposed worker ownership and size

Exactly 19 repository-relative files. Budget estimates count additions plus deletions. T/N = total/non-test; receipts and all mixed registration/glue count as non-test. New runtime modules should stay below 500 lines.

| # | Literal path | Purpose | Estimated T/N |
| --- | --- | --- | --- |
| 1 | codex-rs/core/src/config/mod.rs | Internal DirectOpenAiChat mode only; ordinary loader stays Disabled. | 20/20 |
| 2 | codex-rs/core/src/agent/role.rs | Chat mode inheritance/runtime wire match; keep reserved-provider rejection. | 25/25 |
| 3 | codex-rs/core/src/session/turn.rs | Attach deferred Chat context outside outer retries and check its latch. | 35/35 |
| 4 | codex-rs/core/src/client.rs | Private slot initialization and narrow Chat auth/transport/observer wiring. No WS/guard refactor. | 100/100 |
| 5 | codex-rs/core/src/accounting.rs | Register Chat adapter; mode->endpoint/dialect and typed original-price selection. | 70/70 |
| 6 | codex-rs/core/src/accounting_transport.rs | Implement Chat observer on existing immutable evidence; preserve admitted-send contract. | 45/45 |
| 7 | codex-rs/core/src/accounting_prices.rs | Chat prospective projection/source tuple; share arithmetic without changing existing tuple bytes. | 75/75 |
| 8 | codex-rs/core/src/accounting_prices_tests.rs | One composite Chat price/source/unknown regression, existing projections retained. | 100/0 |
| 9 | codex-rs/core/src/accounting_chat.rs | New bounded deferred lifecycle, eligibility and five-field state adapter. | 190/190 |
| 10 | codex-rs/core/src/accounting_chat_tests.rs | Seven Core unit cases and bounded synthetic setup. | 260/0 |
| 11 | codex-rs/codex-api/src/lib.rs | Export the small numeric Chat observer surface. | 8/8 |
| 12 | codex-rs/codex-api/src/endpoint/chat_completions.rs | None-default observer builder and pre-terminal parser hook/cancellation. Preserve legacy wrapper and conversion. | 100/100 |
| 13 | codex-rs/codex-api/src/endpoint/chat_accounting.rs | New five-field presence decoder/trait/DTOs, explicitly no cache-write source. | 150/150 |
| 14 | codex-rs/codex-api/src/endpoint/chat_accounting_tests.rs | Ten API cases, including actual process_chat_sse ordering/barriers. | 320/0 |
| 15 | codex-rs/core/tests/suite/mod.rs | Register two native suites only. | 4/4 |
| 16 | codex-rs/core/tests/suite/accounting_chat.rs | Native success/unknown/ownership/scope/gateway-exclusion cases. | 450/0 |
| 17 | codex-rs/core/tests/suite/accounting_chat_recovery.rs | Native retries, barriers, redirects, cancel/delete/two-reopen cases. | 500/0 |
| 18 | codex-rs/core/tests/suite/accounting_chat_support.rs | Bounded Chat fixtures, held sockets and separate-connection readback using existing helpers. | 250/0 |
| 19 | qa/portfolio/agent-cost-accounting/pf-60-s02/chat-dispatch-increment.md | Implementation/receiving receipt, full case map, attempts and limitations. | 160/160 |

Estimated **2862 total / 982 non-test**. Proposed target **3000 total / 1150 non-test**. **STOP BEFORE exceeding 3300 total / 1300 non-test, or editing any extra path.** This is a new requested manager exception; no WS extension transfers. Do not compress assertions or omit failed-run evidence to fit.

A parser-only stage would not establish native request accounting. This coherent stage joins presence capture, exact admission, immutable price binding and real retry/reopen proof; it reuses the accepted state and HTTP machinery. If the actual diff cannot meet the ceiling, freeze and ask the integrator for a narrower staged allocation or explicit extension with the exact diff. No self-widening.

API module registration can use a path-qualified child in chat_completions.rs and re-export through lib.rs; do not add endpoint/mod.rs casually. Keep parser tests in a sibling file registered from the parent that can access private parser helpers. New inline fixture data avoids BUILD compile_data changes.

Excluded: state/schema/migrations, existing accounting tests/receipts except the enumerated prices test file, Responses decoder/adapters/WS modules, startup-prewarm/compaction/memory policy files, codex_thread fixture seams, model-provider/login/auth/wallet, telemetry/generic retry, HTTP/websocket client policy, protocol/common request DTO, TUI/app-server, manifests/locks/BUILD files and unrelated worktrees.

The manager separately owns a future docs/research/agent-cost-accounting/chat-dispatch-allocation.md plus active plan/current sprint and review/build ledgers. Those are not worker paths.

## 6. Frozen future tests: 38 runnable functions

Expected new functions: **10 API + 8 Core unit (7 adapter + 1 price) + 20 native = 38**. None executed by this design worker. Table vectors do not add to function counts. Freeze exact native fixture retry settings/counts before implementation execution; mismatch is a counterexample requiring manager disposition, not permission to change retry behavior.

### API: prefix chat_accounting_, ten

| Suffix | Required observations |
| --- | --- |
| presence_matrix | Five fields each Missing/Null/0/positive/i64MAX; missing/null/object detail forms; malformed containers and negative/fractional/string/bool/overflow rejected. Range vectors isolated so unrelated sum overflow cannot mask field validation. |
| top_level_containers_only | Normal/usage-only/error-with-sibling-usage accepted. Absent/null container produces no patch. Empty object all Missing. Nested error/response usage, billing and vendor write extensions never become supported evidence. |
| raw_before_lossy_chunk_conversion | Partial/null fields survive raw observation even when typed ChatUsage would default or skip; no accounting read from converted TokenUsage. |
| error_envelope_usage_precedes_error | Observer blocked on top-level usage beside error prevents terminal error; release commits then emits native error; prior prefix survives an error with no usage. |
| done_and_finish_reason_are_not_usage | [DONE] completes only after prior writes; finish_reason alone gives no counts; unknown/length/tool/provider-error semantics retained. No observation from the sentinel. |
| positions_and_cumulative_patches | Usage at positions 1/3/5 around nonusage data; repeated cumulative and null/omitted preserve fields without double counting; keepalive comments not positions. |
| observation_barrier_and_rejection | Held observer prevents corresponding completion; release allows one, rejection allows none and sends bounded failure. |
| invalid_evidence_stops_before_done | Malformed JSON/supported fields reach error observer and terminate before following [DONE]; no full payload leaks in accounting error. |
| consumer_cancel_and_interruption | Consumer closes before next event and during held observation; parser stops; no synthetic final patch. EOF/idle/actionable timeout preserves committed prefix and existing error behavior. |
| none_preserves_legacy | Same valid/error/malformed event corpus with observer None retains legacy event order, usage conversion, keepalive and completion behavior; zero observer calls. |

### Core unit: prefix accounting_chat_, eight

| Suffix | Required observations |
| --- | --- |
| bootstrap_is_lazy_once_and_mode_local | One preallocated request UUID; no install before resolve; one bootstrap; all previous modes keep meanings. |
| direct_auth_and_gateway_eligibility | Typed API-key/direct positive; complete negative predicate above, including gateway with same endpoint/model, explicit auth overrides and request routing extensions. Initialized route loss latches; initial unsupported never installs. |
| exact_final_endpoint_binding | Scheme/host/port/path/query/userinfo/fragment vectors; final post-auth request mismatch admits/sends zero. |
| auth_and_guard_before_admission | Awaited synthetic auth rejection and actual existing stage-one denial stop before accounting and transport; do not substitute a dead-owner constructor test for guard-dispatch proof. |
| bootstrap_cancel_scope_and_latch | Missing state, bootstrap failure, cancelled writer wait and observation failure latch; no retry; RAII clears slot even on cancellation. |
| response_local_attempt_identity | Two physical responses, later observer executes first, old writes stay with old immutable attempt/source; common request and exact predecessor; failed-send intent remains unknown. |
| role_inheritance_reserved_id_parity | Instruction-only reload preserves complete runtime provider/mode; explicit model_providers.openai TOML remains rejected unchanged; no new reserved-ID override exception. |
| prices_exact_source_and_unknown | In accounting_prices_tests.rs: literal Chat tuple/rates/prospective time; unique/duplicate/disabled/Astra/alias/remote/Plan/Local cases; default-on-wire assumption; write rate absent; exact retained Responses/Anthropic provenance unchanged. |

### Native: prefix accounting_chat_native_, twenty

Use actual Op::UserInput through native Core, bounded real loopback HTTP sockets, synthetic typed auth, public spawn/resume/delete and independent SQLite readback. Never seed attempts as a substitute for sends. Configure supports_websockets=false on the direct Chat fixture; test retained Responses/prewarm behavior separately.

Unless specified otherwise: request retries and outer retries zero, one POST per vector, one attempt, zero WS frames, no external request. Success payload includes native assistant content and a stop finish reason before [DONE]. Assert method, exact /v1/chat/completions, final model and include_usage=true.

| Suffix | Frozen result and count |
| --- | --- |
| off_and_mode_isolation | Disabled with optional state present/absent sends ordinary Chat, installs nothing. Anthropic/Responses modes on Chat install nothing; Chat mode on Responses/Anthropic does not attach a Chat observer. One native fixture send per applicable positive path. |
| literal_partial_and_zero_goldens | C1/C2/C4 each one POST/attempt; exact seven populations, patch presence, original snapshot and partial money. Explicit native cache-write display zero never appears as measured write. |
| cumulative_and_separate_usage | One POST, multiple top-level usage-only/cumulative patches with frozen positions; repeated C1 still 0.00121/null, never a second charge. |
| top_level_error_retains_usage | Error-with-sibling-usage then native terminal error: one POST/attempt, one committed patch, no completion; error without usage after prefix preserves it. Retries zero. |
| http_retry_policy | request_max_retries=1, stream_max_retries=0: 503 then success => two POSTs/two linked attempts, first unknown; direct OpenAI terminal 429 => one POST/unknown attempt. Preserve actual retries_transient_rate_limits behavior. |
| outer_retry_prefix_and_ids | HTTP retries zero, stream retries one: C1 prefix then EOF, next C1 success => two POSTs, one request, two sources/linked attempts, subtotal0.00242/full null. Repeat/missing provider IDs do not merge. |
| api_key_401_no_invented_refresh | Existing direct API-key 401 terminal policy: one POST, one unknown intent, no fabricated refresh request. Separately unit-prove identity continuity for any actually supported repeated setup; do not label that a native API-key refresh. |
| redirects_no_follow_or_repair | Five 3xx vectors with stream retries enabled and two real endpoints: ON one origin POST, zero target requests, one unknown attempt, zero observations, no repair. OFF positive 307/308 controls follow native behavior with no accounting installation. |
| mismatched_endpoint_never_sends | Same eligible identity with changed port/path/query/base: zero POSTs and attempts. Core post-auth URL-mutation test supplements native configured-route vectors; no real malicious auth invoked. |
| admission_barrier_and_failure | Hold actual SQLite writer lock, poll request, observe zero POST before release and one after. Trigger failure before admission commit => zero POST, no retry, full rollback. Freeze trigger identity and independent readback. |
| observation_failure_no_repair | One held POST, commit C1 then fail later observation using exact SQL trigger: prior full DB evidence unchanged, no Completed/retry, only one attempt. |
| invalid_evidence_no_repair | One POST per malformed/count/subset/total vector, no completion or second POST even with outer retries enabled; valid committed prefix preserved; write never inferred. |
| cancellation_and_two_reopens | Cancel before dispatch => zero POST; cancel after admission with no usage or committed C1 => one POST, unknown/prefix respectively. Two actual native reopens send zero old POSTs and retain exact IDs/positions/prices/nulls. New user input adds one fresh request. Disclose uncertain admission windows. |
| spawned_role_children_and_fork | Native parent calls spawn twice (one instruction-only role), then completes: four initial POSTs/attempts over three owners. Role child's first stream ends after prefix and retries once => fifth POST/attempt linked only to that child's request. Exactly two native spawn edges; parent/children distinct sampling IDs; fork/history copy adds zero attempts. |
| delete_rejects_late_usage | Public deletion while one response is held: late patch cannot recreate owner/attempt; unrelated owner's rows/prices unchanged. Repeat installed-store deletion while collection OFF preserves native counts. No permanent fence/physical-erasure claim. |
| immutable_prices_and_unpriced_rows | Eligible model and Astra/remote-only variants each one POST; nonzero usage with null price remains unknown. Two reopens/config-catalog change preserve original bytes/null binding; later new attempt may get its own snapshot, never reprice old rows. |
| sampling_auxiliary_scope | Two genuine sampling operations in one turn get separate request IDs. Native manual/local compaction uses its separate session and contributes zero accounting attempts despite its own fixture POST. Reusing a client after scope exit also has no Chat observer. Record each auxiliary POST separately, never count it as free. |
| gateway_exclusion_and_header_parity | Synthetic create_pfterminal_plan_provider route, same model and loopback shape: one success POST creates no accounting. Matching released-reservation 503 then success => two gateway POSTs and existing header rotation, zero accounting rows. Direct OpenAI request has no injected plan header. Preserve the existing telemetry rotation test. |
| missing_usage_and_correlation | Successful [DONE] without usage, usage:null, and exhausted stream failure each leave one unknown intent/no observations under zero retries. Two separate sampling calls reusing a provider ID are two attempts; no snapshot subtraction. |
| finish_reason_and_tool_parity | Stop path one sampling request; length continuation then stop two distinct sampling requests; one safe synthetic native tool call then stop two sampling requests. Provider-error finish with usage retains the one attempt's evidence while native error semantics stand. Preserve content-filter/unknown-reason parser behavior in API cases; never manufacture completion or cost. |

Supplementary mutation demonstrations within these same allocated files:
- Bypass Chat observer placement before the error-envelope arm: error-with-usage case must fail its persisted-prefix assertion.
- Map Missing write to Number(0): C1/C4 must fail their exact unknown populations/full-estimate assertions.
- Disable the Chat accounting failure check at the outer sampling boundary: a retry-enabled invalid-evidence case must fail its no-repair POST assertion.
Preserve each failed mutation run and restore source before final formatting/tests. These are proof executions, not extra runnable test names or review passes. If a mutation does not discriminate, repair the test and preserve the original counterexample; do not call it successful proof.

## 7. Reserved-lane overlap audit

At this inspected base, distinguish present ownership from stale predecessor prose:

- PF-27-S04 front matter is draft. Its September 14 note says the paused reservation was returned to PF-83, not cancelled/completed/resumed. Retain its owner and historical broad secret-broker/protected-state/network-proxy/Core broker/vault reservations. The seven-path system-preflight allocation is historical accepted work, not evidence of a running PF-27 worker. None of this proposal's 19 paths is in that literal seven-path list or the retained broker-specific Core files.
- Active PF-83-S01 reserves app-server confirmation, v2 thread protocol/schema, TUI permission/confirmation files and qa/reliability/live-permission-transition-20260913. None intersects the 19 paths. Core policy/memory code remains excluded here; this proposal does not alter permission effectiveness.
- Current PF-80-S01 reserves scripts/initiative_control/, coordinator-bootstrap research, qa/initiative-control/management-bootstrap/ and coordinatorInstructions.md; owner-daemon work is recorded under that scope. No literal intersection.
- Historical Task Node owned-ingress allocation reserves ten responses-api-proxy files (lib/synthetic/synthetic_io/synthetic_exchange/synthetic_policy/their tests/synthetic_cli/README/Cargo.toml) plus separately manager-owned root dependency/lock integration. codex-api is a different crate. This proposal touches neither those files nor shared manifests/locks and does not reuse that fixture proxy as an accounting authority.
- Core client/config/role/turn and API Chat remain high-touch seams. Current provider-recovery and any newer allocations must be compared against actual diffs by the manager immediately before source dispatch; this inspection does not establish that external worktrees are idle. Serialize target leases and shared integration. No security owner resumption or duplicate security worker is authorized.

A newly discovered collision, protected auth/transport defect or indispensable unlisted dependency is a STOP to the manager with the exact path and minimal requested change. Do not reopen human product decisions already recorded or ask for an unnecessary release permission.

## 8. Receiving evidence and gate

Worker return must include:
1. Exact clean launch/base, candidate commit/branch/worktree, all changed paths, per-file hashes, full diff and conservative additions+deletions/non-test counts against the newly allocated base. Disclose target overruns and every manager extension without resetting history.
2. chat-dispatch-increment.md with all 38 names, every table vector and C1-C4 mapped to real assertions/logs or an explicit unresolved disposition. Preserve failed builds, fixture defects, timeouts, leaks and mutation runs. A matching test name is not case coverage.
3. Raw captured POST/frame counts and synthetic request shape; independent connection readback of native ownership/edges, attempts, sources/revisions, observations, original/null snapshots and full DayTotals. Keep only synthetic payload evidence and approved numeric metadata; no real credentials/prompts/account dumps.
4. Exact pinned toolchain, exclusive target lease, commands, exits, run UUIDs, matching raw/JUnit artifacts, pass/fail/filtered/execution-skip/flaky/leaky counts and two-reopen evidence. Copy/hash JUnit before the next run and correlate its UUID.
5. Scope-constrained fix/format before final affected tests; no formatting outside worker scope. Then final whitespace and governance checks. No unguarded test runner, cache/environment repair, private competing target or live profile.
6. One fresh independent Fable material code review plus necessary scoped correction under the existing accounting review ledger. Manager records prior usage and any additional allowance under delegated discretion. No redundant review of unchanged accepted state machinery.

Future guarded selectors, from the assigned checkout/build setup:

    just test -p codex-api -E 'test(chat_accounting_)' --locked --offline
    just test -p codex-core --lib -E 'test(accounting_chat_)' --locked --offline
    just test -p codex-core --test all -E 'test(accounting_chat_native_)' --locked --offline

Expected new executions 10, 8, 20 respectively; list names using supported guarded tooling or reconcile the exact executed manifest with matching run artifacts. Zero matches are failure. No tests were run for this proposal.

After focused proof, run affected API/Core suites and retained Anthropic/Responses HTTP/WS, prewarm/incremental, role, stage-one, Chat parser/tool/retry/keepalive regressions and Corbanu released-header telemetry. Retain login/http-client no-redirect/proxy/custom-CA regression gates. Reuse tests; do not add a separate transport policy project.

Receiving manager reruns the established shared state + TaskNode-session gate and combined API + responses-api-proxy gate on the actual merged tree. Verify normal-library build, relevant lint, python3 docs/plans/check.py, python3 docs/sprints/check.py and git diff --check. Discover actual current counts; old 224/390/28 or 634/63 numbers are not new results. Preserve prior full-Core baseline failures; same-name failures alone do not establish identical cause. Complete-workspace testing retains its separate authorization rule.

Read docs/development/test-isolation.md before execution (read by this designer; no tests followed). Ordinary Rust tests use this checkout's guarded just test only. No live profile/credential store. A native credential prompt invalidates the run and stops successor/retry dispatch; never bypass a denial.

Proposed internal-only functional N/A: this increment adds no public activation or user flow, ordinary behavior remains OFF. This needs named-integrator acceptance; this source-informed design is not a code-blind functional test pass. Later S03/S04 must retain frozen independent human cases, a separately confined exact-package executor with actual negative access probes/PTY controls, independent evidence review, true-TUI keys and applicable TensorCash/Isometric evidence. No human-test, complete-S02, live-provider, benchmark, named-human acceptance or release-readiness claim.

Remaining after receiving: both disclosed WS P3s; startup generate=false and other auxiliary collection with explicit ownership/classification; gateway/compatible/subscription/agent-identity economics; historical evidence acquisition and fuller wire/endpoint provenance; non-token charges and billed settlement; permanent deletion policy beyond accepted retention; S03/S04 and complete application qualification. Do not mark these passed from direct Chat coverage.

## 9. Ten-line summary

1. Choose direct OpenAI API-key Chat sampling as the next same-S02 unit.
2. Keep Corbanu gateway collection, prices, balances and settlement excluded.
3. Allocate local request/attempt UUIDs independently of gateway/provider headers.
4. Reuse admitted HTTP sends, exact endpoint binding and no-redirect construction.
5. Persist top-level numeric usage before errors, typed chunks and [DONE] completion.
6. Measure only five supported fields; cache-write and full estimates stay unknown.
7. Bind immutable prospective Chat price snapshots from exact bundled API-key rows.
8. Preserve OFF behavior and record both WS P3s plus prewarm/auxiliary gaps.
9. Propose 19 files and 38 tests; target3000/1150, STOP3300/1300.
10. Require manager freeze/lease, independent review and actual receiving-tree evidence.
