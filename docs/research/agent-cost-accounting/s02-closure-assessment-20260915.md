# PF-60-S02 disposition: close the bounded sampling milestone, carry coverage into S03

Action: acct-next-design-02  
Allocation digest: 42f0479372d13709cfb40f72158866823a22725bdfdc8dd584bc24989b2c548d  
Claim: 319c2a35-8016-4554-b709-867d950f40e3  
Designer: gpt-6-astra / high  
Date: September 15, 2026  
Inspected HEAD: 73f262d8b2990ecd560e9466a6e0e96448d708f2  
Worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911  
Brief: /private/tmp/fmgr.Q1SIYZ/briefs/acct-next-design-02.json  
Verified brief SHA-256: a2d95fae62ccde632253d553ba44f34ae68200206665bb6e41044afe40ea7361

## Recommendation

**Choose bounded S02 closure and successor planning, not another S02 collector.** The persistence/replay substrate and the four received direct sampling paths are a coherent completed engineering milestone. The remaining candidates require new operation ownership, gateway evidence authority, historical acquisition contracts, or application-wide coverage semantics. They are not cheap residual adapter wiring.

This is a recommendation to Fable, **not a declaration that S02 is already completed or that S03 may start now**. S02 is still in_progress; S03 is draft and dependent. Before archival, Fable must reconcile the stale ledgers, attach actual final receiving evidence, and explicitly record the bounded contract and transfer of outstanding obligations. Any unmet evidence requirement remains a blocker. No requirement is checked merely because this proposal recommends moving it.

Move the remaining concerns into S03's coverage/inspection contract: truthful presentation of recorded evidence, unrecorded operations, unknown economics, legacy availability and separate balances. Keep the actual missing collectors/acquisition/economic contracts as named, gated PF-60 follow-ups in the plan and S03 handoff; do not silently add them to S03's current one-file UI mandate. S03 must either display their absence honestly or have a separately frozen prerequisite implementation unit. Complete application accounting and billed gateway cost stay unqualified until their prerequisites are met.

**New executable implementation unit: none proposed. Size target and STOP ceiling: not applicable.** This proposal grants zero repository write paths, zero test executions, zero build leases, and no new review spend. A future S03 code allocation still needs its own exact files, budgets, cases and receiving gate.

## Authority and inspection limits

This is routine read-only design preparation within the existing PF-60 product initiative. Applied the corbanu-terminal-development skill. Canonical product heading in docs/corbanu-product-spec.md is **Measurement targets**; excerpt: “No commercial performance numbers have been supplied. The following metrics must be instrumented, with targets set through the decision rights defined above.” Its Corbanu Plan row includes purchases, renewal, usage, revenue and model-level unit economics. The older Product measurement wording in the plan/contract is historical and should be reconciled without changing the approved outcome.

Read the S02 Done/Remaining ledgers, accepted contract, Chat/Responses HTTP/WS allocations, active plan, S03 draft and plan/sprint processes. Inspected the receipt collection, including original native goldens, compact-import receiving, direct adapters and cleanup04. Prior test results below are attributed to those records, not new executions. Read actual source for every candidate discussed below. No repository edits, builds, tests, provider calls, credential-store/profile reads, subagents, commits or pushes were performed. Reading Rust type definitions and route configuration code did not execute their credential-resolution paths. Initial repository status was clean and HEAD matched the assigned base.

This is source-informed design, not the independent code-blind functional design required for a user-facing handoff. No approval, release readiness, live-provider verification or all-platform result is asserted.

## 1. What the current receipts establish, and what needs reconciliation

The current S02 record reports received Responses HTTP at d81bad635, WebSocket sampling at c86634e21, additional native WS vectors at 16c43b5fe, and direct Chat at 82f9dde0e. It also preserves accepted native store, exact pricing/bindings, retention/deletion, native ownership, compact import and original-contract goldens. The historical counts establish their recorded candidates only:

- Original native goldens: three external cases transcribe the original fixture identities, exact partial sums, native family edges, two reopens and compact parity. They explicitly do not store every descriptive S01 field or establish real gateway prices/bills.
- Compact import receiving: 20 focused and six external cases, combined 595 and Core 100; original race-test failure and corrected deterministic ordering retained. Accepted original-evidence import is expressly not legacy-source acquisition.
- WS receiving in the sprint: all 36 frozen cases and later two native vectors, with admission/fallback identity, no-redirect and unknown semantics. Safety/verification/moderation and late binding tests required actual discriminating mutations; receipt history must remain intact.
- Chat receiving in the sprint: all 38 frozen cases; manager disposition preserves production's terminal length behavior. Recorded receiving counts include API 234, Core accounting 44, native/regression 97, shared state/TaskNode 390 and proxy 28. These overlap and must not be summed into a unique-test total.
- Cleanup04 worker receipt: pre-fix failure on both provider publication paths, final focused 5/5 and uninstrumented broader 158/158 with zero LEAK. The preceding diagnostic 158-case run had one LEAK, and historical output-handle ownership remains unresolved. No blanket leak or full-Core waiver follows.

The brief says cleanup is received. Git history at the inspected base contains the provider fix 8c683a5d9 merged through 533a16077, and the earlier cleanup bcca36636. Source confirms both real publication owners replace the ModelClient under the state lock before publishing changed session configuration. Nevertheless, the checked-in worker receipt still says manager receiving remains outstanding, and S02 retains unchecked cleanup04 receiving plus superseded impl-01/02/04 and cleanup03 entries. The integration history proves source inclusion, not every receiving test or independent acceptance.

Fable should add the authoritative receiving disposition and reference the actual final-tree artifacts, resolve each superseded entry explicitly, and preserve old attempts in historical receipt sections. Do not redo accepted adapters because an old stop remains unchecked; do not convert historical failures into passes. The plan's sprint map also still calls the normal-library store “next” despite the later completed work. Its current coordinates and S02's consumed Chat mandate need a final receiving handoff, not another guessed worker path.

## 2. Gateway economics: what is and is not possible today

### Observed code

| Verified existing file / symbol | Finding and consequence |
| --- | --- |
| codex-rs/model-provider-info/src/lib.rs :: create_pfterminal_plan_provider | Chat wire, configurable CORBANU_API_BASE_URL with legacy fallback, dedicated gateway env-key route, requires_openai_auth=false. Model spelling and a direct-looking URL do not make this a direct OpenAI-priced request. |
| Same file :: create_pfterminal_plan_anthropic_provider | A gateway Anthropic-wire sibling shares gateway/key configuration. Even adding gateway Chat would not complete Corbanu gateway coverage. |
| codex-rs/core/src/accounting_chat.rs :: eligible | Requires the direct OpenAI identity and typed API-key path and expressly excludes plan/routing/auth overrides. Simply widening this predicate would attach the wrong economics. |
| codex-rs/core/src/client.rs :: stream_chat_completions_api | Generates a gateway request header outside its auth recovery loop; separately attaches a Chat observer only through the supported sampling slot. The gateway currently has no accounting collection under this adapter. |
| codex-rs/codex-api/src/telemetry.rs :: run_with_request_telemetry | Rotates a released-reservation request identity only on qualifying evidence; uncertain outcomes keep server duplicate protection. That header is not a local physical-attempt key or settlement receipt. |
| codex-rs/wallet/src/corbanu_api.rs :: CorbanuApiPricing | Four exact decimal price strings plus version. No effective interval, dispatch price guarantee, invoice line, settlement amount, request allocation or billing revision. |
| Same file :: CorbanuApiBalance / CorbanuApiAccount | Separate balance/reserved/available strings in micro-USD and USD; account has keys/models. Neither DTO associates a movement with an inference attempt. |
| codex-rs/tui/src/chatwidget/wallet_api.rs :: load_read_only_account and model display | Models and account are fetched through separate read paths; display treats prices as per-million. Reading this implementation supplies no actual price observation or new authority to call it from accounting. |
| codex-rs/core/src/accounting_prices.rs :: chat_original / openai_rows | Projects eligible bundled direct OpenAI rows using prospective local acceptance times. It is not a gateway quotation source. |

### Direct answer

**No billed cost can truthfully be attributed to a gateway request using the existing DTOs and collector. Nor can the existing implementation produce a supported full gateway request estimate.** Balance deltas are confounded by funding, concurrent requests, reservations and release. A pricing version is an identifier, not a historical effective interval. A request header is correlation/idempotency metadata, not proof of settled money. Do not infer zero, free prewarm, upstream pass-through prices, or full input cost from the native Chat cache-write zero.

There is an important distinction: it would be possible in a future authorized unit to record gateway dispatches and reported numeric usage with **cost unknown**, without first obtaining settlement. Likewise, an explicitly approved, gateway-specific prospective quote could support a labeled partial token estimate. Neither requires an invoice merely to admit usage; neither is implemented or authorized by widening the direct adapter today. Lack of historical dates does not prohibit a new locally accepted prospective estimate. It prohibits backdating a current catalog and claiming historical/billed accuracy.

### What must exist first

For an honest prospective estimate:

1. An approved gateway/opaque account/endpoint scope, exact model and wire/usage semantics; a distinct gateway mode and eligibility predicate. Preserve existing credential resolution, guards and transport policy. Do not access keys, wallet signatures or account endpoints merely to identify scope.
2. A gateway price authority accepted for this purpose: exact currency, per-token unit, rate buckets, version/content identity, capture time and scope. Either authoritative effective intervals or an explicit local prospective-only validity policy with freshness/expiry and version-change handling. Admission must freeze the original snapshot or explicit null atomically; later retrieval/replay cannot fill old nulls or change rates.
3. Lossless gateway usage extraction and billable bucket semantics. With current five-field Chat semantics, cache-write remains missing, so noncached input and the full estimate remain unknown; known read/output subtotals could be priced only after a valid quote exists. Structural zero or a new gateway cache field needs source authority and a frozen contract.
4. Gateway-specific retry/idempotency treatment, no-redirect binding and cancellation/failure proof. Separate local physical attempts from server charge identities; do not count a repeated settled server identity twice or suppress distinct measured attempts without reconciliation evidence.

For billed cost additionally: authoritative settlement/invoice evidence with issuer/account scope, request/charge identity, currency and exact amount; a stable mapping to local requests/attempts including idempotent retries; final/pending/reversed status, revisions/refunds and rules for non-token fees. A billing record may directly establish an amount without reconstructing rates, but must establish attribution. Balance snapshots remain separate funding/reservation facts with as-of provenance.

Existing CorbanuApiPricing and CorbanuApiBalance do not supply these contracts. No gateway server implementation, live behavior or promised endpoint was verified in this assignment. Required evidence must come from the gateway owner; this proposal invents no server API.

## 3. Startup prewarm and auxiliary collection: keep excluded

**Neither is now a cheap qualification extension.** The WS/Chat work reduces parser and transport effort, but does not settle operation ownership or make absent evidence complete.

### Startup

codex-rs/core/src/session_startup_prewarm.rs :: schedule_startup_prewarm_inner constructs a startup TurnContext using INITIAL_SUBMIT_ID, captures tools, creates a new client session and calls prewarm_websocket before run_turn. The existing startup context is a useful seam; it is not the durable sampling identity/owner created by run_sampling_request.

codex-rs/core/src/client.rs :: new_session initializes all accounting slots empty. stream_responses_websocket explicitly selects no deferred collector when warmup=true, sets generate=false, and prewarm_websocket drains until completion. By contrast, codex-rs/core/src/session/turn.rs :: run_sampling_request attaches the sampled-operation context outside retries and scopes its removal. Accepted cached-route provenance supports safe subsequent sampling reuse; it does not account for warmup.

A future startup unit must define a durable startup-operation identity attached to the native thread without borrowing the next user turn, ownership when startup never reaches a turn, cancellation/timeout and connection-discard treatment, resume/restart deduplication, and optional-store materialization effects. It must admit only actual response.create intents and preserve reported fields without interpreting generate=false as a billing exemption. Handshake-only preconnect must remain distinct. It must prove warmup and subsequent sampling never double count or share the wrong price/attempt. This requires contract and lifecycle work beyond toggling the current warmup exclusion.

### Auxiliary operations

- codex-rs/core/src/compact.rs :: run_compact_task_inner_impl creates a fresh model client session and its loop streams separately from run_sampling_request.
- codex-rs/core/src/compact_remote_request.rs :: run_remote_compact_attempt calls compact_conversation_history through ModelClient; this is a separate endpoint/request path.
- codex-rs/core/src/compact_remote_v2_attempt.rs can use a supplied client session or create its own. It returns optional converted TokenUsage and retains a TODO about raw usage being emitted only after output validation. This is an additional partial/failed-output evidence seam, not proof that sampled Responses already covers remote compaction.
- codex-rs/core/src/memory_stage_one.rs :: StageOneMemoryClient::extract makes a new session, enforces its own completion/termination checks and consumes converted completion usage. Its auxiliary operation does not inherit a sampling observer.

Attaching a long-lived observer to every ModelClientSession would change intentional scope cleanup, conflate sampled and auxiliary retries, and risk carrying the wrong owner/guard/price. “Auxiliary” is not one wire dialect or one mechanical unit. Defer it; when scheduled, choose exactly one operation and freeze attribution, raw evidence ordering, guards, no-redirect parity, price authority and real cancellation/recovery cases. Preserve existing exclusions as **not collected / coverage incomplete**, never zero.

## 4. Legacy acquisition and complete application coverage

### The import API is not an acquisition pipeline

codex-rs/state/src/runtime/accounting_store.rs defines RetainedImport as a complete immutable original-evidence bundle: Attempt, observations and OriginalPriceEvidence::Bound or Unpriced. Its import_retained accepts bounded complete attempts aged 90..365 days. codex-rs/state/src/runtime/accounting_late_import.rs validates native owner, original identity/time, revision conflicts, replay fences, exact prices and transactional compact reduction. A repository search for import_retained calls found no non-test acquisition caller beyond the facade definition.

Old rollout snapshots cannot satisfy this interface by inference. codex-rs/rollout/src/policy.rs persists TokenCount and ModelResponseCompleted but marks RawResponseCompleted transient. codex-rs/app-server/src/request_processors/token_usage_replay.rs restores a connection-scoped latest snapshot, with fallback turn-position attribution; it does not reconstruct attempt identity or numeric presence. Subtracting snapshots or pairing adjacent model/usage events would invent retry attribution and potentially charge copied history.

Keep legacy acquisition deferred. S03 can expose legacy session/snapshot availability separately with legacy_presence_lost / unattributed / unavailable coverage as appropriate, but must not synthesize RetainedImport attempts. A later importer needs a concrete authorized source carrying stable original ownership, identities, revisions, dispatch times and original price/null evidence, plus bounded traversal, consent/disclosure rules and conflict/restart tests. If a source never retained those fields, request-level reconstruction is impossible; accept a labeled legacy snapshot or unavailable detail instead of promising recovery. Do not reuse today's configuration or price to repair history.

### Complete coverage is a separate assertion

codex-rs/state/src/runtime/accounting_types.rs :: Attempt stores logical/physical identities, native owner/turn, provider/model, opaque scope, arithmetic dialect and dispatch time. It does not persist literal wire/endpoint provenance, operation kind or a record of every excluded dispatch. The compact values preserve seven measured known/unknown populations, known USD, unknown estimates and attempt count. These describe **recorded attempts**. They cannot count requests never observed or prove all native children/providers/operations were collected.

S03 needs two independent presentation dimensions:

- **Value completeness:** known token/money subtotals and unknown populations among recorded attempts; immutable estimate provenance, billed unavailable where absent.
- **Collection coverage:** whether the requested threads/time/providers/operations were actually collected. Disabled collection, unsupported gateway/auth, startup/auxiliary work, legacy absence, retention cutoff and missing lineage must not disappear because the recorded subtotal happens to be exact.

For a run with one accounted direct request plus uncollected compaction or gateway work, show the recorded subtotal with incomplete/unknown run coverage. Do not show a full run total. No rows under ordinary Disabled means “accounting not collected/unavailable,” not $0. In absence of a trustworthy persisted coverage boundary, S03 must conservatively disclose unknown coverage; a mode value observed today cannot certify a past interval.

The current S03 draft already requests root/descendant breakdown, historical availability, freshness, custom half-open ranges, selectable grouping and raw/compact retention distinctions. Those are the right place to expose these constraints. Its one-file planned output is insufficient as an executable allocation; manager must separately freeze exact query/UI/API boundaries. Campaign membership and arbitrary partial-day queries are not granted by native spawn edges or daily totals. No duplicate ledger or hidden repair is recommended.

## 5. Exact bounded S02 claims at closure

S02 may claim, once Fable records final acceptance:

1. An optional versioned native-state accounting substrate, normal collection Disabled, with atomic durable intent and original-price/null admission, numeric observations and replay.
2. Direct Anthropic sampling and direct OpenAI API-key Responses HTTP, combined Responses WS/HTTP fallback, and Chat sampling under their existing explicit internal predicates and approved scopes. Each mode keeps its own qualification boundary; these are not a new universal provider switch.
3. Logical sampling identities outside retries, distinct admitted physical attempts, response-local observation identity and native ownership; recorded retries/prefixes are retained through cancellation and reopen without snapshot double addition.
4. Presence-aware raw numeric evidence for the supported dialects before lossy/terminal handling; unknown is not measured zero. Chat cache-write/full cost remain unknown; missing Responses write similarly limits estimates.
5. Exact prospective token estimates from the accepted direct bundled authority, immutable source/rate/null bindings, partial subtotals and unknown populations. These are estimates, not invoices.
6. Accepted native retention/deletion and compact original-evidence import within the 90-day detail / 365-day replay and conservative UTC-day expiry contract, with the exact bounded recovery/process proof in its receipts.
7. The received scope cleanup, shared-binding API and provider-publication corrections, with their actual regression/mutation evidence and preserved historical failures.

S02 must not claim:

- Public activation, an ordinary-user enabled collector, a usable S03 inspection flow or complete mixed-provider/application/campaign accounting.
- Gateway collection/economics, billed request settlement, balance-derived spend, subscription/Plan-allowance conversion, arbitrary compatible-provider economics, non-token fees or historical repricing.
- Startup/auxiliary/compaction/memory/realtime/model-discovery collection or zero spend for those operations.
- Legacy-source scanning, lossless recovery of absent historical fields, all descriptive S01 provenance fields, or a verified billing join from provider IDs.
- Permanent deleted-ID revocation beyond the accepted horizon, physical/WAL/cross-database erasure, power-loss guarantees beyond executed proofs, or unlimited drill-down.
- Globally leak-clean/full-Core/all-platform results, blanket waiver of prior failed runs, native human acceptance, isolated code-blind functional execution, true-TUI/live-repository/benchmark/release qualification.

## 6. Closure and S03 receiving gate owned by Fable

These are manager reconciliation requirements, not instructions executed by this design worker:

1. Record acceptance of the bounded S02 milestone and map every Remaining entry: superseded stop, received correction, unmet receiving evidence, or explicit successor obligation. Keep the original history. Add cleanup's actual receiving/review attribution rather than treating a merge commit or 158-case worker pass as sufficient.
2. Link all applicable final-tree manifests, source/candidate/receiving identities, actual commands/run UUIDs and corresponding raw/JUnit evidence. Include the established shared state/TaskNode-session and combined API/proxy gates, retained transport/guard regressions and correction proof. Reuse valid accepted evidence when unchanged; run genuinely missing required receiving checks. Do not copy old counts or demand redundant clean-code opinions. All future tests require reading docs/development/test-isolation.md and guarded just test; no live profiles or native credential prompts.
3. Carry unresolved historical LEAK/output-handle uncertainty and full-Core failures with explicit limits. A clean rerun is not cause attribution. Do not introduce a speculative production lifetime change or infer a waiver. Any substantive unresolved finding or unmet required final-tree test still blocks closure.
4. Reconcile the active plan's outdated scope/table/coordinates and S02's consumed allocation. Preserve S01's approved vocabulary, exact arithmetic, unknown policy and retention. Moving implementation sequencing is manager work; dropping an approved product outcome or changing its contract needs product authority. This proposal recommends no such outcome deletion. If the proposed bounded closure changes a previously mandatory S02 acceptance criterion, obtain the explicit applicable disposition before archival rather than marking it passed.
5. Accept a reasoned increment-specific internal-stage functional N/A, where applicable, naming S03/S04's later gate. Archive S02 only once its revised, authorized acceptance and every required evidence item are met. Then allocate one dependency-complete S03 successor with actual matching worktree/branch/base and literal files. Run the plan/sprint checkers at that handoff; this design ran neither.
6. S03 owns honest inspection of present/absent coverage. Gateway, prewarm, individual auxiliary operations, acquisition and fuller provenance stay individually identified in its handoff and plan, with prerequisites above. No blanket “move to S03” code authority. Preserve no public collection activation absent its own frozen scope and relevant product decision.
7. Before any applicable user-facing handoff, obtain frozen independent code-blind design, separately confined exact-package execution with real keys and negative controls, independent evidence review, applicable TensorCash/Isometric proof and human/release evidence. This proposal does not satisfy those gates. No repeated approval request for already-authorized routine bookkeeping is needed.

## 7. Reserved lanes and verified-path discipline

This proposal edits no repository path, so its overlap with all reserved implementation lanes is empty. Inspected current reservation sources:

| Existing record | Current consequence |
| --- | --- |
| docs/sprints/current/p0-security-levels/pf-27-s04-isolated-credential-broker.md | Draft after September 14 reservation return to PF-83; not cancelled/completed/resumed. Preserve broker/protected-state/vault/credential-proxy ownership and manager-owned manifests/locks. No gateway auth work is authorized here. |
| docs/sprints/current/p0-security-levels/pf-83-s01-permission-confirmation.md | In progress; app-server confirmation, v2 thread protocol/schema and TUI routing/settings/app/tests are reserved. A future S03 expansion beyond usage.rs may collide directly and must be serialized or reallocated before dispatch. |
| docs/sprints/current/initiative-delivery-control/pf-80-s01-delivery-control.md | In progress; initiative-control scripts, coordinator research/QA and coordinatorInstructions.md remain owned. No accounting design write intersects. |
| docs/research/tasknode-integration/owned-ingress-allocation-20260913.md | Historical ten-file responses-api-proxy allocation and manager-owned dependency integration; not proof of an active proxy worker. Preserve shared receiving/build coordination and do not borrow its fixtures as an accounting authority. |

The manager must compare live allocations and actual diffs again when preparing S03; these records do not prove other worktrees idle. No private target, duplicate security worker or dependency update is allocated.

Every repository file path cited in this proposal was located in this checkout and checked as an existing file. Paths in older receipts are historical provenance and are not new allocations; this proposal does not claim their private artifacts were revalidated. One initial exploratory memory-directory glob did not exist; discovery located the real memory_stage_one.rs before source inspection. No missing/guessed path appears as a proposed write target. No new repository filename or implementation checkout is invented.

## Ten-line summary

1. Recommend bounded S02 closure after Fable reconciles actual acceptance and final receiving evidence.
2. Keep S03 dependent until S02 is formally completed and archived; this proposal changes no status.
3. S02 claims native persistence/replay and four explicitly gated direct sampling paths with collection OFF.
4. Gateway DTOs cannot establish request billed cost; balances and reservation headers are not settlement.
5. A future gateway estimate needs its own scoped prospective quote and lossless usage contract.
6. Keep startup prewarm excluded until startup ownership, lifecycle and accounting semantics are frozen.
7. Keep auxiliary collection excluded and allocate individual operations rather than a global observer.
8. Existing retained import consumes original bundles; it cannot recover missing legacy evidence.
9. S03 must distinguish recorded-value completeness from collection coverage and never render absence as zero.
10. No new implementation unit, size allowance, tests, provider calls or user/release qualification is claimed.

## Final shared-checkout provenance

The checkout advanced during this read-only review from the assigned 73f262d8b2990ecd560e9466a6e0e96448d708f2 to 8f631e08cd2405cf8209fe018567fc32a8d1ade5. The intervening commit is “Record what the Slack transport re-qualification needs from the owner”: one documentation file, 57 additions; no accounting code, plan, sprint or receipt changed. The accounting findings and recommendation therefore remain grounded in the assigned base. This worker did not make that commit. Final git status was clean. A read-only path check verified all 24 literal repository file citations in this proposal; zero were missing. No tests or checkers were executed.
