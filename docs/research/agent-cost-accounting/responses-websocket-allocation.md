# PF-60-S02 next executable unit: Responses WS sampling and HTTP fallback — allocation

Manager-accepted same-S02 allocation, September 14, 2026 (Fable manager).
Acceptance source: the frozen `acct-ws-freeze-01` dispatch explicitly instructs
freezing the accepted next-unit design. This records that allocation, not an
implementation, review, build lease, functional acceptance or receiving result.
Authority: the existing PF-60 product initiative; Travis's September 14 accounting
resumption is recorded in the [received HTTP allocation](responses-dispatch-allocation.md).
Security remains with its existing owner.

## Manager acceptance and bounds

- Design source: Astra High action `acct-next-design-01`, inspected base
  `1fdb3fc1d85fdfaa041577146da0f0de0c1c3704`; captured terminal transcript
  `/private/tmp/fmgr.Q1SIYZ/acct-next-proposal.md`, SHA-256
  `7afe4607e45f4e8e4d0419811ec87baabe25e24df261fc76f5924aad64d81a4c`.
- Freeze action: `acct-ws-freeze-01`; allocation digest
  `4d4dd6a4375ce997705696c0b5f49b3f45ae9ec2ffc57e823973c28dfea5f247`;
  claim `265a378f-49b7-4e80-ad5b-c411c5ecbbc9`; brief SHA-256
  `fedd543e8357b40dcb54b432df788e69a408d1e6f6f7d31af2a58a333d351871`.
  Both input hashes verified before this documentation freeze.
- Documentation worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-acct-ws-freeze-20260914`;
  branch `bootstrap/acct-ws-freeze-20260914`; clean starting HEAD
  `e3bd579bf4e0c7c58ad863af2a9c6098e2297f98`.
- Implementation worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-acct-ws-20260914`;
  branch `bootstrap/acct-ws-20260914`; allocated base
  `e3bd579bf4e0c7c58ad863af2a9c6098e2297f98`.
  Manager verifies the actual clean implementation checkout and current reservations
  before source dispatch; these coordinates do not assert an agent is running.
- Predecessor: Responses HTTP `e47e41870` received at `d81bad635`;
  historical 634/634 combined and 63/63 Core accounting results remain in the
  [HTTP receipt](../../../qa/portfolio/agent-cost-accounting/pf-60-s02/responses-dispatch-increment.md).
  Original-contract goldens `f4507cb50` accepted at `855ab3382` are consumed history.
- Worker owns ONLY the 18 literal files and purposes in the frozen ownership table.
  Manager retains allocation/plan/sprint documents, manifests, locks and BUILD
  files. Any additional path requires exact reallocation before editing.
- Accepted target **3000 total / 1050 non-test**; **STOP before exceeding 3300
  total / 1200 non-test**, or touching any additional path. Count additions plus
  deletions, receipt text and mixed glue conservatively. This unit's accepted
  exception covers the coherent send-boundary/fallback/recovery work below;
  prior size exceptions are not inherited and tests must not be cut to fit.
- Accepted coverage is **Responses WS sampling plus HTTP fallback**. Preserve
  the separate HTTP-only mode. Startup `generate=false` prewarm remains
  unqualified and excluded from accounting; preserve its connection reuse and
  never attribute its evidence to the next turn or call it free. Chat/Corbanu,
  auxiliary routes and complete application coverage remain subsequent work.
- **OFF invariant:** ordinary configuration stays Disabled; no public
  TOML/CLI/env/Feature/TUI activation. OFF creates no sampling context, new auth
  resolution, accounting installation/database requirement or usage observation.
  Preserve ordinary WS behavior and installed-but-OFF deletion cleanup.
- Receiving gate: one fresh independent Fable material review plus necessary
  scoped correction, preserving prior allowance usage and failures; all 36
  named future cases and vectors, retained regressions, shared state/TaskNode
  and combined API/proxy receiving gates below. S02 stays open; S03 stays dependent.
- Manager must compare current allocations and actual diffs and assign an exclusive
  build-target lease before execution. This freeze does not claim those checks,
  future tests, internal-only N/A acceptance or independent review have occurred.
  Read [test isolation](../../development/test-isolation.md) before any tests;
  Rust execution uses this checkout's guarded `just test` only. No live profile
  or credential-store access; a native prompt invalidates the run and stops
  successor/retry dispatch.

The accepted proposal follows, preserving its contracts, file purposes, test
cases, counts and goldens. Its historical inspected checkout, proposed wording
and requests to accept/freeze/replace governance are provenance; the acceptance
and allocated coordinates above supersede those preparation requests.
Remaining dispatch and evidence gates still apply. The captured return is marked
`• RETURN`, rather than the brief's `# RETURN`; terminal indentation, table
borders and wrapped table identifiers are normalized to Markdown below.
This documentation-only freeze is routine preparation for PF-60, with no
interactive behavior change; TUI, live-repository and benchmark execution are
not applicable to this edit. Later implementation qualification is not waived.

---

# PF-60-S02 next executable unit: Responses WS sampling and HTTP fallback

Action: acct-next-design-01
Allocation digest: b14556a2d7d103c3b5225c61cad8cc0c1ed5f544e0fccfa7f5656c419b90783a
Claim: 08d36d76-95e9-4de5-a713-b70f51492d07
Designer: gpt-6-astra, effort high
Inspected base: 1fdb3fc1d85fdfaa041577146da0f0de0c1c3704
Worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911

Initial and final HEAD matched the frozen base; working tree remained clean. No repository edits, builds, tests, provider calls, credential reads, pushes or subagents were performed.

## Authority and receiving prerequisites

This is design preparation for the existing PF-60 product initiative.

Exact product heading: Measurement targets. Requirement excerpt: “No commercial performance numbers have been supplied. The following metrics must be instrumented, with targets set through the
decision rights defined above.”

Plan: docs/plans/active/portfolio-agent-cost-accounting.md.
Sprint: docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s02-idempotent-usage-persistence-and-replay.md, currently in_progress.

The sprint records Responses HTTP candidate e47e41870 received at d81bad635, with 634/634 combined tests and 63/63 Core accounting tests. These are historical receiving results, not verification
performed by this designer.

Before implementation, Fable must:

1. Replace the consumed HTTP allocation with this exact same-S02 unit.
2. Record the actual clean implementation worktree, branch and full base commit consistently in plan and sprint.
3. Reconcile stale pause, two-path and consumed-golden instructions without erasing their history.
4. Accept the proposed file/size bounds and sampling-only coverage boundary below.
5. Compare current reservations and actual diffs, assign the build lease, and run governance checkers.

S02 remains open; S03 is not activated by this increment.

## Recommendation and inspected alternatives

Implement WS sampling first.

The HTTP unit already provides the same Responses numeric decoder, Inclusive reducer, native ownership, immutable price binding and failure latch needed for WS. The important missing behavior is one
logical request spanning WS dispatches, reconnects and HTTP fallback.

The WS work is nevertheless substantial: the real send occurs in WsStream’s pump, cached connections retain their original handshake route, and startup prewarm precedes sampling ownership.

Chat is a subsequent allocation because its actual path introduces distinct questions:

- ModelClientSession::stream_chat_completions_api generates x-pfterminal-request-id outside its auth-recovery loop, but outer sampling retries invoke the method again. That header cannot replace
  local durable request/attempt identity.

- ChatCompletionsClient::stream_request uses existing HTTP retry machinery and can reuse the accepted accounting transport.
- process_chat_sse handles error envelopes before typed chunks and completes on [DONE]. Numeric evidence must be extracted before these terminal paths and before ChatUsage conversion.
- ChatUsage defaults missing prompt/completion/total/cache/reasoning fields to zero; conversion hardcodes cache-write zero. None establishes measured zero.
- create_pfterminal_plan_provider uses WireApi::Chat, a configurable gateway endpoint and its own credential route. Direct-provider catalog prices cannot automatically price that route.
- CorbanuApiPricing.version supplies neither historical effective intervals nor request settlement attribution. Balance, reserved and available funds remain separate account snapshots.

This unit makes no Chat, Corbanu gateway, balance, settlement or compatible-provider claim.

## Frozen outcome and coverage boundary

Add an internal in-memory mode, provisionally:

AccountingMode::DirectOpenAiResponses { scope, approved_endpoint }

It covers eligible sampling requests over WS and their native HTTP fallback. Preserve DirectOpenAiResponsesHttp with its existing HTTP-only behavior and tests; do not silently broaden its meaning.

Eligibility remains exact configured provider openai, Responses wire, approved direct endpoint and positively identified API-key route. No public configuration, feature flag, CLI, environment or TUI
activation.

The unit covers:

- Each sampling response.create dispatch.
- Incremental requests and full-request retries.
- Connection-limit and missing-previous-response recovery.
- WS-to-HTTP fallback and subsequent HTTP transport retries.
- Native ownership, cancellation, deletion and two reopens.

Startup prewarm remains explicitly unqualified accounting work. session_startup_prewarm.rs invokes generate=false before run_sampling_request establishes the accounting owner. Handshake-only
preconnect and probe also do not represent sampling inference attempts.

Preserve those workflows and their connection reuse. Do not assign prewarm usage to the next user turn, manufacture zero usage, or call prewarm free. Even if a warmup reports tokens, it remains
outside this sampling allocation. Record this limitation and require manager acceptance of the bounded scope before dispatch. Complete application coverage remains open.

## Implementation contract

### One sampling identity across transports

Extend the accepted deferred Responses context to support both modes. Allocate its logical request UUID outside outer stream retries, as today.

The new mode resolves the same Sampling for WS dispatch and HTTP fallback. Fallback must not allocate a new logical request or reset retry_of. Each admitted physical request gets a fresh attempt
UUID; every response gets immutable attempt/source binding.

The HTTP-only mode must still leave WS-only executions without installed accounting.

Retain the accepted store, schema, arithmetic, retention, native owner materialization and write serialization. No new ledger or transport registry.

### Admission at the actual WS send boundary

Inspected flow:

stream_request → spawned response task → stream mutex → run_websocket_response_stream → send_websocket_request → command queue → pump’s inner.send.

Admission at stream_request alone is too early: a queued or cancelled request may never reach the pump.

Add an optional accounted-send path that:

1. Serializes the final request before admission.
2. Carries only exact model, requested tier, validated route binding and the accounting callback.
3. Reaches the pump under existing exclusive response-stream ownership.
4. Checks cancellation, awaits durable admission and immutable original-price binding immediately before inner.send.
5. Returns the bound numeric observer to the response parser.

Keep the ordinary WsCommand::Send path intact. A distinct accounted command/result can avoid forcing accounting errors into WsError.

Handshake, DNS, TLS, ping/pong and reconnect establishment do not create inference attempts. A failed or uncertain frame send after admission retains an unknown attempt. A cancelled admission may
already have committed: preserve the accepted fail-closed completion guard and never guess its predecessor.

Cancellation while queued must prevent a later unsolicited send. Monitor consumer closure while waiting for the stream mutex and send result; the pump must check whether its result receiver has
closed before admission/send. Cancellation after a committed admission can leave an unknown intent without a confirmed frame, which must be disclosed as the dispatch crash window.

On accounted stream failure, invalidate the failed connection through the existing error cleanup so later requests cannot consume its leftover frames. Do not detach accounting writes.

### Binding and cached-connection parity

The approved endpoint remains an HTTP(S) base, normally https://api.openai.com/v1. Derive one exact WS endpoint by changing only https→wss or http→ws on the validated /responses URL.

Reject userinfo, query, fragment, unsupported scheme, wrong host, path or port. Check the actual WS URL before opening an accounted sampling connection and again against the immutable established-
connection binding before admitting a frame.

Do not validate a cached connection using only newly resolved configuration. Preserve nonsecret handshake provenance with that connection:

- Actual endpoint.
- Whether its resolved route satisfied the API-key eligibility predicate.
- Any narrowly necessary route identity required to reject incompatible reuse.

Populate this from existing client setup on normal connect and preconnect/prewarm paths. It must require no additional auth resolution, credential access, fingerprint or accounting database. Keep it
coupled to connection replacement/reset.

Recheck current eligibility as well. A cached connection with incompatible or unknown provenance must fail before an accounted frame; do not “repair” this by resetting healthy connections or
disabling incremental reuse. Positive prewarm reuse must work when both original and current bindings qualify.

Use the accepted exclusions for command auth, experimental bearer overrides, auth headers, subscription/ChatGPT and agent-identity routes. A header’s presence is not proof of API-key economics. An
already-accounted logical request cannot continue unaccounted after an incompatible route change.

### No redirects, guard ordering and fallback

Retain WebSocketConnector, its configured proxy resolution, TLS/custom-CA behavior and dialer. There is no reason to introduce a bare connector or edit protected transport policy.

Prove with real two-endpoint fixtures that WS handshake 301/302/303/307/308 produces no target request. In the accounted sampling path, latch a redirect failure so generic retry/fallback cannot turn
a binding rejection into a repair send.

HTTP fallback must continue using create_client_for_route_without_redirects, including HTTP retries. Preserve accepted HTTP comparison against the final request URL after auth.

Preserve normal WS handshake authentication and the existing stage-one check before dispatch. Admission must follow applicable guards. Do not change auth, stage-one, fallback budgets,
previous_response_id, safety buffering, model verification or moderation semantics.

### Numeric extraction and observation ordering

Reuse endpoint/responses_accounting.rs::decode; do not create a second Responses decoder.

For accounted WS text messages, decode and persist numeric evidence before wrapped-error handling, ResponsesStreamEvent deserialization or process_responses_event.

Supported locations remain:

- response.completed.response.usage
- response.failed.response.usage
- response.incomplete.response.usage
- Both accepted response.usage container forms

A generic wrapped error is not a newly authorized usage dialect. Preserve previously committed usage when it terminates the stream.

Increment a checked source position for every delivered text message, including intervening nonusage messages. Ping/pong and transport frames are not usage positions. Each dispatched response has
its own source UUID; revisions never use provider IDs, timestamps or callback completion order.

Reuse Missing/Null/nonnegative-i64 semantics for all six fields. Reject malformed, fractional, negative and overflowing evidence while ON. Missing cache-write remains unknown. Cumulative updates
replace fields; omitted/null values do not retract known counts.

Await persistence before forwarding the corresponding completion or terminal handling. Decode/storage failure latches a bounded accounting error and prevents completion, retry and HTTP repair. EOF,
timeout and cancellation preserve the committed prefix or unknown intent.

The OFF and HTTP-only WS paths retain legacy parsing and event behavior.

### Original price snapshots

Reuse accounting_prices.rs::responses_original unchanged.

Its accepted source tuple is:

openai-responses-api-key-bundled-v1

It already describes API-key Responses economics without an HTTP-specific identity. Creating a different price source merely because transport changes would be misleading.

Capture exact final request model and requested tier. Only eligible exact bundled OpenAI rows and accepted API-key billing branches receive prospective snapshots. Preserve null bindings for
unsupported tiers, Astra/disabled economics, aliases, remote-only models and missing prices.

Snapshot admission is atomic with each attempt. Prior attempt prices remain immutable through reconnect, fallback, catalog/config changes and two reopens; a later attempt may receive its own newly
captured snapshot. No backdating or repricing.

Required literal goldens:

- Input 100, read 20, write 0, output 40, reasoning 10, total 140; rates 5/30/0.5 USD per million → noncached 80, estimate 0.00161 USD.
- Omitted write → known read/output subtotal 0.00121 USD, full estimate null.
- Two distinct complete attempts with those counts → 0.00322 USD.
- Repeated cumulative evidence within one attempt remains 0.00161 USD.

These are token estimates under the accepted default-tier assumption, not billed amounts.

## Exact proposed worker ownership

Reserve exactly these 18 repository-relative files. New runtime behavior belongs in the dedicated modules; existing large files receive narrow glue.

| # | Path | Purpose |
| --- | --- | --- |
| 1 | codex-rs/core/src/config/mod.rs | Add internal combined Responses mode; preserve defaults and HTTP-only mode. |
| 2 | codex-rs/core/src/agent/role.rs | Include new mode in existing inheritance/wire checks. |
| 3 | codex-rs/core/src/session/turn.rs | Attach combined-mode deferred sampling outside retries. |
| 4 | codex-rs/core/src/client.rs | WS dispatch wiring, connection provenance, eligibility and fallback reuse. |
| 5 | codex-rs/core/src/accounting.rs | Register adapter and narrowly reuse mode/admission machinery. |
| 6 | codex-rs/core/src/accounting_responses.rs | Combined-mode resolution and HTTP/WS endpoint binding. |
| 7 | codex-rs/core/src/accounting_transport.rs | Construct immutable response evidence for an already-admitted WS attempt. |
| 8 | codex-rs/core/src/accounting_websocket.rs | New Core bridge for pump admission and connection binding. |
| 9 | codex-rs/core/src/accounting_websocket_tests.rs | New eight-case Core contract suite. |
| 10 | codex-rs/codex-api/src/lib.rs | Minimal accounting callback export needed by Core. |
| 11 | codex-rs/codex-api/src/endpoint/responses_websocket.rs | Optional accounted stream/pump path and pre-conversion observation. |
| 12 | codex-rs/codex-api/src/endpoint/responses_websocket_accounting.rs | New callback/dispatch DTOs and response-local observation helper. |
| 13 | codex-rs/codex-api/src/endpoint/responses_websocket_accounting_tests.rs | New eight-case API suite. |
| 14 | codex-rs/core/tests/suite/mod.rs | Register two native suites. |
| 15 | codex-rs/core/tests/suite/accounting_responses_ws.rs | New native success, routing and retry cases. |
| 16 | codex-rs/core/tests/suite/accounting_responses_ws_recovery.rs | New cancellation, persistence, deletion and ownership cases. |
| 17 | codex-rs/core/tests/suite/accounting_responses_ws_support.rs | Bounded synthetic WS/HTTP fixtures and separate-connection readback. |
| 18 | qa/portfolio/agent-cost-accounting/pf-60-s02/responses-websocket-increment.md | New receipt preserving attempts, counts, hashes and limitations. |

Use explicit path registration for new modules. Reuse existing test support without editing it; any indispensable support-file edit requires exact reallocation.

Excluded: state/schema/migrations, price/catalog files, accepted HTTP tests/receipt, SSE decoder, generic retry, startup-prewarm owner, login/auth, model-provider, websocket-client, HTTP-client,
memory guard, protocol, public config schemas, manifests, locks and BUILD files.

Manager separately owns the allocation document and plan/sprint bookkeeping.

Proposed target: 3000 total changed lines / 1050 non-test. STOP before exceeding 3300 / 1200, or touching any additional path.

Count additions plus deletions, receipt text and mixed registration/glue conservatively. This requested exception exceeds normal 800-line guidance because physical send admission, immutable stream
evidence, fallback continuity and real recovery proof form one coherent native unit. It is not inherited permission. Manager must accept it before implementation; do not compress or drop tests to
fit.

## Frozen future tests

Expected minimum: 8 API + 8 Core unit + 20 native = 36 new runnable functions. None has been executed. Table vectors are not additional tests.

### API: eight

| Name | Required observable result |
| --- | --- |
| responses_websocket_accounting_none_preserves_legacy | No callbacks; unchanged event, error, safety and transport behavior. |
| responses_websocket_accounting_pump_admission_barrier | Held admission prevents actual frame send; release permits one; rejection permits none. |
| responses_websocket_accounting_queued_cancel_and_send_failure | Cancelled queued work never sends later; post-admission send uncertainty remains unknown. |
| responses_websocket_accounting_positions_and_replacement | Intervening text advances position; cumulative/repeated/null patches do not double count. |
| responses_websocket_accounting_usage_and_terminal_containers | All accepted containers persist before completed/failed/incomplete handling; conflicting dual containers fail. |
| responses_websocket_accounting_invalid_evidence_stops | Presence/range/malformed vectors reach bounded failure and prevent later completion. |
| responses_websocket_accounting_observation_barrier | Held persistence blocks corresponding completion; rejection terminates. |
| responses_websocket_accounting_response_local_binding | Old/new requests keep distinct immutable observers and positions, including reconnect. |

### Core unit: eight

All names begin accounting_responses_ws_.

| Suffix | Required result |
| --- | --- |
| mode_scope_and_lazy_bootstrap | OFF, HTTP-only and combined modes remain distinct; one bootstrap per logical request. |
| auth_route_eligibility | Typed API-key positive and every accepted override/subscription/provider negative; initialized-route change fails. |
| exact_endpoint_binding | Exact HTTP↔WS scheme mapping, port/path checks and forbidden URL components. |
| cached_connection_provenance | Matching prewarm connection qualifies; stale endpoint or unsupported original auth cannot borrow current eligibility. |
| fallback_keeps_request_and_predecessor | One request UUID; WS and HTTP attempt chain remains connected. |
| original_price_binding | Exact source tuple/rates, null economics/tier cases and immutable prior bindings. |
| failure_and_cancellation_latch | Admission/observation/bootstrap failure reaches outer stop; no repair dispatch or stale scope. |
| role_inheritance_preserves_reserved_provider_rule | New mode inherited; instruction-only role works; reserved openai override remains rejected. |

### Native integration: twenty

All names begin accounting_responses_ws_native_. Use actual native Op::UserInput, real fixture sockets and typed SQLite readback; never seed attempts instead of dispatching.

| Suffix | Required result/count |
| --- | --- |
| off_and_http_only_compatibility | Normal WS workflow; zero accounting installation/rows for OFF and accepted HTTP-only WS path. |
| complete_and_partial_goldens | One sampling frame/attempt per vector; exact 0.00161 or partial 0.00121/null result. |
| prewarm_preconnect_and_cached_reuse | Preserve handshake and generate=false; one subsequent sampling attempt; warmup evidence never attributed to it. |
| incremental_sampling_and_turn_identity | Captured incremental payloads remain native; distinct sampling calls get distinct request UUIDs despite connection reuse. |
| upgrade_required_http_fallback | One 426 handshake, zero WS frames/attempts, one HTTP POST/attempt. |
| ws_prefix_then_http_fallback | With zero short-stream retries: one WS frame with committed usage then close, one HTTP POST; two linked attempts, one request, exact aggregate. |
| connection_limit_reconnect | Retryable connection-limit response then successful WS reconnect; two sampling frames/attempts, first unknown unless usage preceded error. |
| previous_response_missing_full_retry | Incremental error causes native full resend; distinct chained attempts, no provider-ID deduplication. |
| fallback_http_transport_retry | WS prefix/failure then HTTP 503 then success; three admitted attempts with one continuous predecessor chain. |
| handshake_and_postdispatch_errors | Handshake rejection creates no WS inference attempt; post-frame terminal rejection leaves its admitted unknown attempt. Preserve configured native recovery counts. |
| redirects_never_escape_binding | All five WS handshake redirects: zero target requests/frames and no repair HTTP send; fallback HTTP redirects retain accepted no-follow behavior. |
| endpoint_and_cached_auth_mismatch | Mismatched new route or cached provenance: zero sampling frames/attempts; no silent reset or unaccounted send. |
| admission_and_guard_barriers | Real database writer barrier blocks send; storage failure and actual stage-one denial cause zero frames. |
| observation_failure_no_repair | One frame; failed usage write yields no completion, reconnect or HTTP POST; prior committed evidence unchanged. |
| cancel_before_and_after_dispatch | Queued cancellation has no later frame; post-dispatch cancellation retains unknown/prefix evidence with exact counts. |
| two_reopens_and_original_prices | Two native reopens add no old sends; IDs, source positions, original/null prices and totals match; fresh turn gets new request. |
| spawned_role_children_and_fork | Root plus two real children, instruction-only role reload and held concurrent frames; three owners, correct edges, child retry attributed correctly; fork adds no charge. |
| delete_rejects_late_usage | Delete while held; late evidence cannot resurrect owner; unrelated owner and installed-but-OFF deletion semantics preserved. |
| unknown_prices_and_no_usage | Nonzero usage with unsupported economics retains null price; successful missing-usage response remains unknown, never zero. |
| auxiliary_scope_and_event_parity | Auxiliary/compaction reuse gets no sampling observer; safety, verification, moderation and completion behavior remain intact. |

Freeze fixture retry settings and exact handshake/frame/POST expectations before execution. If observed native policy differs from a proposed expectation, preserve the counterexample and obtain
manager disposition; do not change production retry behavior to satisfy a count.

## Reserved-lane overlaps

Inspected PF-27 system-preflight allocation reserves secret-broker manifest/preflight files and its security evidence. None overlaps these 18 paths. Security remains under its existing owner; this
proposal does not resume that work.

Inspected Task Node owned-ingress allocation reserves ten responses-api-proxy paths plus manager-serialized dependency integration. That crate is distinct from codex-api; do not adopt or edit its
synthetic ingress implementation for these tests.

Slack/initiative-control scripts, visual-test infrastructure and their receipts remain excluded.

Core client/config/role/turn and API WS are high-touch seams. At dispatch, the manager must check current provider-recovery/security allocations and actual diffs, not merely these historical
reservation documents. Serialize shared build targets and all manifest/lock work. Discovery of a protected transport/auth defect returns to its owner.

## Receiving evidence and gates

The manager must receive:

1. Exact base/candidate/receiving commits, literal file manifest, hashes, full diff and conservative size accounting.
2. One fresh independent Fable material review, followed only by necessary scoped correction under the recorded review allowance. Preserve earlier accounting reviews and failures.
3. Case-to-evidence mapping for all 36 names and every listed vector; declarations or passing names do not substitute for missing assertions.
4. Raw handshake/frame/POST counts, separate SQLite readback, admission/observation barrier evidence, original/null snapshots, cancellation windows and two-reopen results.
5. Final toolchain, target lease, commands, exits, run IDs and pass/fail/skip/flaky/leaky counts. Match JUnit to the actual run ID; preserve the predecessor’s erroneous-copy history without reusing
   it.

6. Scope-constrained fix/format before final tests, followed by whitespace and governance checks.

All Rust execution uses the checkout’s guarded just test. No raw Cargo test/nextest execution, live profiles or credential-store access. Native prompts invalidate the run and stop successors/
retries.

Focused selectors:

just test -p codex-api responses_websocket_accounting --locked --offline
just test -p codex-core --lib accounting_responses_ws --locked --offline
just test -p codex-core --test all accounting_responses_ws_native --locked --offline

Expected new executions: at least 8, 8 and 20, respectively. Capture runnable names through supported guarded tooling; if that tooling cannot list names, reconcile executed names from matching run
artifacts. Never bypass isolation to obtain a list.

Then run affected Core/API suites and retained HTTP/Anthropic accounting, WS/retry/prewarm/incremental, role and stage-one regressions. Include websocket-client and retained HTTP/login no-redirect/
custom-CA compatibility tests.

On the receiving tree, run the shared state/TaskNode-session gate and the established combined API/proxy integration gate. Discover actual current counts; do not copy 634 or 63 forward as new
results. Preserve and investigate any baseline failures. Complete-workspace testing follows its separate authorization rule.

This proposal is source-informed and not code-blind functional design. Proposed internal-only N/A requires named-integrator acceptance and must name the later S03/S04 isolated functional execution,
independent evidence review and true-TUI gates. No human-test readiness, complete S02, live-provider, TensorCash/Isometric, benchmark or release qualification is claimed.

## Ten-line summary

1. Choose Responses WS sampling plus HTTP fallback before Chat/Corbanu.
2. Reuse accepted ownership, reducer, Responses decoder and API-key pricing.
3. Preserve HTTP-only mode; add a separate internal combined mode.
4. Admit attempts in the actual socket pump before sending frames.
5. Keep one logical request and predecessor chain across WS and HTTP.
6. Validate cached handshake provenance and exact endpoints without redirects.
7. Preserve numeric absence, unknown costs and immutable original snapshots.
8. Leave startup prewarm explicitly outside sampling coverage.
9. Propose 18 files, 36 tests, target 3000/1050 and STOP 3300/1200.
10. Require manager allocation, independent review and receiving-tree evidence.

## Worker return contract

Return the following to the Fable receiving manager; do not push:

1. Exact source base, candidate commit, branch/worktree, 18-file changed manifest,
   per-file hashes, full diff, additions/deletions and conservative total/non-test
   size accounting; identify every STOP or reallocation disposition.
2. Receipt path `qa/portfolio/agent-cost-accounting/pf-60-s02/responses-websocket-increment.md`
   with all 36 named cases and every vector mapped to actual evidence or explicit
   unresolved disposition; exact handshake/frame/POST counts, independent SQLite
   readback, barriers, cancellation windows, native owners, original/null prices,
   four literal goldens and two-reopen evidence.
3. Final toolchain, exclusive target lease, commands, exits, run IDs and matching
   raw/JUnit artifacts with actual pass/fail/skip/flaky/leaky counts. Preserve all
   attempts, failures and corrections; never copy historical counts as new results.
4. Focused and retained regression results after scope-constrained fix/format,
   `python3 docs/plans/check.py`, `python3 docs/sprints/check.py`, and
   `git diff --check`. Record receiving-tree results separately when the manager
   runs them; do not imply worker evidence is combined-tree evidence.
5. Independent review reference and unresolved findings when available; identify
   missing manager-owned review, dispatch or receiving evidence honestly.
6. Remaining prewarm, Chat/Corbanu, auxiliary, economic and application-coverage
   limitations; internal-only N/A remains subject to named-integrator acceptance,
   with later S03/S04 isolated code-blind execution, independent evidence review,
   true-TUI and applicable live-repository gates. No full-S02, human-test or
   release-readiness claim follows from this unit.
