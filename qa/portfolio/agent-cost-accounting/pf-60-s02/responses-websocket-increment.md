# PF-60-S02 Responses WebSocket increment — blocked dispatch receipt

## Result and provenance

No implementation candidate. Source files remain unchanged. The worker stopped
before source edits, build, Rust test execution or fixture dispatch because the
frozen allocation requires a manager-assigned exclusive build-target lease before
execution, and no target assignment was supplied or found in the inspected records.
This is a worker report of missing evidence, not a product-authority disposition.

- Action: `acct-ws-impl-01`; worker: `gpt-6-astra`, effort `high`.
- Allocation digest: `7456562d282e275f6d6d67a71921abeed07f3319ba747ccce87de00ef2fe030c`.
- Claim: `4d16bd34-aec3-4a89-89be-cc8f8da6cd2b`.
- Brief: `/private/tmp/fmgr.Q1SIYZ/briefs/acct-ws-impl-01.json`.
- Verified with `shasum -a 256`: `f15fc799344a9f7b26e4118a599e7c5bb9231b8cc685723e5125af26cf0ca8c8`.
- Actual clean source base and dispatched base: `556da8be7555a13b3000718da10c767645f4aa52`.
- Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-acct-ws-20260914`.
- Branch: `bootstrap/acct-ws-20260914`.
- Plan and sprint record the parent receiving base `73fc51b1a73ec6304fb51603b6fb46ee9f61b56b`;
  the dispatched HEAD is the manager's subsequent coordinate-update commit.
  This ancestry is disclosed, not treated as a new product decision or an independent blocker.
- Change class of assigned work: product initiative, active PF-60, sprint PF-60-S02
  `in_progress`. This receipt alone is routine evidence text.
- Product heading: **Measurement targets**. Requirement excerpt:
  “No commercial performance numbers have been supplied. The following metrics must be
  instrumented, with targets set through the decision rights defined above.”

## Missing manager prerequisite

The [frozen allocation](../../../../docs/research/agent-cost-accounting/responses-websocket-allocation.md)
states: “Manager must compare current allocations and actual diffs and assign an exclusive
build-target lease before execution.” Its preserved prerequisite list likewise puts build-lease
assignment before implementation. The current sprint's corresponding precondition is unchecked.

The START message authorizes this assignment but does not identify a build target or its exclusive
lease. The brief and `/private/tmp/fmgr.Q1SIYZ/integration-receipts/receive-acct-ws-freeze-01.json`
contain no such assignment. A filename search for lease/dispatch/allocation/accounting-WS records
in that manager directory found no lease record. This does not prove no lease exists elsewhere.
The worker did not claim an unused target, inspect credentials, or invent manager verification.

Owner: Fable receiving manager. Needed continuation: supply the exclusive build-target path
and lease evidence, with the allocation's reservation/diff check recorded, then redispatch.
Plan/sprint/allocation edits remain outside this worker's writable scope. No scope or size
reallocation is requested; no size STOP was reached.

## Attempts and checks

Read `docs/development/test-isolation.md` before any tests. No Rust tests, builds,
provider calls, native credential access, native prompts, subagents or pushes occurred.

| Command / inspection | Actual result |
| --- | --- |
| `shasum -a 256 /private/tmp/fmgr.Q1SIYZ/briefs/acct-ws-impl-01.json` | Exit 0; matched frozen hash above |
| `git status --short`, `git rev-parse HEAD`, `git branch --show-current` | Exit 0; clean dispatched checkout and matching branch/base |
| `python3 docs/plans/check.py` before receipt | Exit 0; active 3/3, available slots 0 |
| `python3 docs/sprints/check.py` before receipt | Exit 0; current 116, archived 126 |
| `git diff --check` before receipt | Exit 0 |
| `rustc --version` | Exit 0; rustc 1.95.0 (59807616e 2026-04-14) |
| `just --version` | Exit 0; just 1.58.0 |
| Read-only `rg` over build configuration and brief | Exit 2: absent root `.cargo/config.toml` and `codex-rs/justfile`; existing root `justfile` was inspected. No test was attempted |

The required focused guarded commands remain **not run**:

- `just test -p codex-api -E 'test(responses_websocket_accounting)' --locked --offline`
- `just test -p codex-core --lib -E 'test(accounting_responses_ws)' --locked --offline`
- `just test -p codex-core --test all -E 'test(accounting_responses_ws_native)' --locked --offline`

Rust runs: 0. Run IDs and JUnit/raw test artifacts: none. Executed tests: 0;
passing/failing/skipped/flaky/leaky execution counts are not available because no run exists.
All focused, affected and retained regressions remain unexecuted. Shared state/TaskNode and
combined API/proxy receiving gates remain manager-owned and unexecuted for this assignment.
No historical result is presented as a new pass. No fix/format command was needed for Rust:
there are no Rust edits.

## All 36 cases: unresolved, not implemented or executed

Each row and every vector defined under that case in the frozen allocation retains the same
disposition: pending manager dispatch prerequisite and subsequent implementation/execution.
No case or vector has been deleted, accepted as out of scope, or counted as passing.

| Case | Disposition |
| --- | --- |
| `responses_websocket_accounting_none_preserves_legacy` | Unimplemented; unexecuted |
| `responses_websocket_accounting_pump_admission_barrier` | Unimplemented; unexecuted |
| `responses_websocket_accounting_queued_cancel_and_send_failure` | Unimplemented; unexecuted |
| `responses_websocket_accounting_positions_and_replacement` | Unimplemented; unexecuted |
| `responses_websocket_accounting_usage_and_terminal_containers` | Unimplemented; unexecuted |
| `responses_websocket_accounting_invalid_evidence_stops` | Unimplemented; unexecuted |
| `responses_websocket_accounting_observation_barrier` | Unimplemented; unexecuted |
| `responses_websocket_accounting_response_local_binding` | Unimplemented; unexecuted |
| `accounting_responses_ws_mode_scope_and_lazy_bootstrap` | Unimplemented; unexecuted |
| `accounting_responses_ws_auth_route_eligibility` | Unimplemented; unexecuted |
| `accounting_responses_ws_exact_endpoint_binding` | Unimplemented; unexecuted |
| `accounting_responses_ws_cached_connection_provenance` | Unimplemented; unexecuted |
| `accounting_responses_ws_fallback_keeps_request_and_predecessor` | Unimplemented; unexecuted |
| `accounting_responses_ws_original_price_binding` | Unimplemented; unexecuted |
| `accounting_responses_ws_failure_and_cancellation_latch` | Unimplemented; unexecuted |
| `accounting_responses_ws_role_inheritance_preserves_reserved_provider_rule` | Unimplemented; unexecuted |
| `accounting_responses_ws_native_off_and_http_only_compatibility` | Unimplemented; unexecuted |
| `accounting_responses_ws_native_complete_and_partial_goldens` | Unimplemented; unexecuted |
| `accounting_responses_ws_native_prewarm_preconnect_and_cached_reuse` | Unimplemented; unexecuted |
| `accounting_responses_ws_native_incremental_sampling_and_turn_identity` | Unimplemented; unexecuted |
| `accounting_responses_ws_native_upgrade_required_http_fallback` | Unimplemented; unexecuted |
| `accounting_responses_ws_native_ws_prefix_then_http_fallback` | Unimplemented; unexecuted |
| `accounting_responses_ws_native_connection_limit_reconnect` | Unimplemented; unexecuted |
| `accounting_responses_ws_native_previous_response_missing_full_retry` | Unimplemented; unexecuted |
| `accounting_responses_ws_native_fallback_http_transport_retry` | Unimplemented; unexecuted |
| `accounting_responses_ws_native_handshake_and_postdispatch_errors` | Unimplemented; unexecuted |
| `accounting_responses_ws_native_redirects_never_escape_binding` | Unimplemented; unexecuted |
| `accounting_responses_ws_native_endpoint_and_cached_auth_mismatch` | Unimplemented; unexecuted |
| `accounting_responses_ws_native_admission_and_guard_barriers` | Unimplemented; unexecuted |
| `accounting_responses_ws_native_observation_failure_no_repair` | Unimplemented; unexecuted |
| `accounting_responses_ws_native_cancel_before_and_after_dispatch` | Unimplemented; unexecuted |
| `accounting_responses_ws_native_two_reopens_and_original_prices` | Unimplemented; unexecuted |
| `accounting_responses_ws_native_spawned_role_children_and_fork` | Unimplemented; unexecuted |
| `accounting_responses_ws_native_delete_rejects_late_usage` | Unimplemented; unexecuted |
| `accounting_responses_ws_native_unknown_prices_and_no_usage` | Unimplemented; unexecuted |
| `accounting_responses_ws_native_auxiliary_scope_and_event_parity` | Unimplemented; unexecuted |

## Numeric and transport evidence limitations

Actual handshake/frame/POST counts: unavailable; no fixture ran. There is no independent SQLite
readback, admission/observation barrier proof, cancellation-window evidence, owner graph,
price snapshot, source-position evidence, or two-reopen result from this assignment.

Required literal goldens remain frozen expectations, with no new execution evidence:

- Input 100, read 20, write 0, output 40, reasoning 10, total 140; rates 5/30/0.5 USD per million:
  noncached 80, estimate 0.00161 USD.
- Omitted write: known read/output subtotal 0.00121 USD, full estimate null.
- Two distinct complete attempts with those counts: 0.00322 USD.
- Repeated cumulative evidence within one attempt: 0.00161 USD.

These are prospective token-estimate expectations under the accepted default-tier assumption,
not billed amounts. The required source remains `openai-responses-api-key-bundled-v1`.

## Scope and remaining contract

Only this receipt is changed; all 17 allocated Rust paths remain unchanged.
No full implementation diff, independent material review, or receiving result exists.
The receipt commit and final per-file hash/count are supplied in the worker return.

All runtime contracts remain unimplemented: combined mode; one sampling identity across WS,
reconnect and HTTP fallback; real pump admission and cancellation; immutable cached connection
provenance and eligibility checks; redirect failure latch and guard ordering; reuse of the
accepted decoder before terminal processing; and immutable original price snapshots.

Startup prewarm accounting, Chat/Corbanu, auxiliary collection, unsupported economics and
complete application coverage retain the allocation's deferred status. Existing HTTP-only mode
and its tests are unchanged. Default collection remains OFF. No S02 completion or S03 activation
is claimed. Internal-only functional N/A still needs named-integrator acceptance; later S03/S04
isolated code-blind execution, independent evidence review, true-TUI and applicable live-repository
qualification remain required. No human-test, benchmark, live-provider or release readiness.

## Final-tree checks

Final receipt-tree governance and whitespace results, commit, file hash and conservative size
counts are returned with this receipt. They cannot qualify the absent implementation.

## acct-ws-impl-02 — verified lease, divergent dispatched base

This continuation preserves the entire acct-ws-impl-01 receipt above. Its missing-lease
blocker is resolved; implementation remains unstarted for a different, observed checkout
mismatch. This is a routine evidence-only return, not an implementation candidate.

- Action: `acct-ws-impl-02`; model/effort: `gpt-6-astra` / `high`.
- Allocation digest: `77864ff8dca2eb259e5af3fe40328e56c0d2e894a42ed993017a278973c458e2`.
- Claim: `53946144-79c3-42e1-857e-f32d5ec95482`.
- Brief: `/private/tmp/fmgr.Q1SIYZ/briefs/acct-ws-impl-02.json`.
- Brief SHA-256 verified: `1a09926d65ce7e1a34cc9daf884938d75c1ef7d6497b6198992ca5e49505bfbf`.
- Lease: `/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/acct-ws-20260915/LEASE.json`.
- Lease SHA-256 verified: `b537e887800e87bec2ae4156afb1e834978a16176cd1e7a4a3e8f6a1afa94e2d`.
- Lease target: `/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/acct-ws-20260915`;
  no build used it in this action. The lease records reservation/diff comparison and exclusivity.
- Frozen dispatch and lease base: `7e6728f1ddfeedd5e0443851547018457aba5a62`.
- Actual clean starting HEAD: `4364dd53ef77a23f690901fcdc0690f6439961d7`.
- Branch/worktree match the dispatch: `bootstrap/acct-ws-20260914` at
  `/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-acct-ws-20260914`.
- Merge base of actual HEAD and frozen base: `556da8be7555a13b3000718da10c767645f4aa52`.
  `git log --oneline 7e6728f1ddfeedd5e0443851547018457aba5a62..HEAD` returns the
  predecessor receipt commit `4364dd53e`; actual HEAD is not descended from the frozen base.
- `git diff --stat HEAD 7e6728f1ddfeedd5e0443851547018457aba5a62` reports nine paths,
  236 insertions and 167 deletions, including deletion of this 158-line predecessor receipt
  and eight unallocated Task Node/control paths. These are existing branch differences,
  not edits by this worker. The brief's assertion that the receipt exists on its base
  is contradicted by that diff.

**STOP / owner:** Fable receiving manager must reconcile the implementation checkout and
frozen base while preserving predecessor history, then issue consistent dispatch/lease
coordinates. Recommended action: prepare the branch with the intended receiving base and
this receipt history, and redispatch from that exact commit. Alternatively explicitly
allocate the actual branch tip and corresponding lease. The worker has no allocation to
merge the eight unrelated paths or edit manager-owned plan/sprint/lease records. No new
product decision or size exception is requested; no size STOP was reached.

Read test-isolation guidance before checks. Both hash commands and all Git inspections
exited 0. `python3 docs/plans/check.py` exited 0 (active 3/3, slots 0), and
`python3 docs/sprints/check.py` exited 0 (current 116, archived 126). These structural
checks do not prove actual checkout/base agreement. Final receipt-tree repetitions and
`git diff --check` are reported in RETURN. No Rust fix/format is needed without Rust edits.

No guarded Rust test command was run: the three exact focused commands listed above and
all affected/retained/receiving suites remain unexecuted. Builds and Rust runs: 0; run IDs,
JUnit artifacts and executed test counts: none. Pass/fail/skip/flaky/leaky counts are
unavailable, not passing zero-test results. No credential prompt, live-profile access,
provider call, subagent, push or transport fixture dispatch occurred.

All 36 named cases and every vector in the preserved table remain unimplemented and
unexecuted, now pending checkout reconciliation. All runtime contract items, all four
literal goldens, socket counts, independent SQLite readback, barriers, ownership,
cancellation and two-reopen proof remain without new evidence for the same reason.
Only this receipt changed; the 17 allocated Rust paths remain unchanged. Independent
review and receiving gates are absent. Deferred coverage and later functional gates
remain exactly as disclosed above; no implementation or readiness claim is made.

## acct-ws-impl-03 — implementation and worker evidence

This continuation preserves both blocked dispatches above as history. This attempt
implements the allocated runtime and 36 named test functions. It does not complete
S02, activate S03, supply independent review, or qualify a human-test/release handoff.

- Action: `acct-ws-impl-03`; worker: `gpt-6-astra`, effort `high`.
- Allocation digest: `59e0c45038e7efd589e9dd3ac90a3d9957ce02b52eb8c3ed9b01b117cc597e65`.
- Claim: `e6576b49-2b96-4787-b31e-db459eb7e161`.
- Brief: `/private/tmp/fmgr.Q1SIYZ/briefs/acct-ws-impl-03.json`;
  SHA-256 `a2098ced27d8e36fa81ca8121f0036c8129047975a6453f53f6cc0aaf571b4b5`.
- Assigned and actual clean base: `59260f56e0f201fdadef9f73f25ab68a791041e5`.
  Worktree and branch remain the literal allocated coordinates above.
- Lease file and verified SHA-256 remain
  `b537e887800e87bec2ae4156afb1e834978a16176cd1e7a4a3e8f6a1afa94e2d`.
  The current brief explicitly grants its reuse for this corrected dispatch;
  predecessor action/base fields in the lease are historical, not silently edited.
- Every build used
  `CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/acct-ws-20260915`.
  Nextest's configured report store is separately `codex-rs/target/nextest/local/`;
  that report location is not a Rust build target.
- Classification: existing PF-60 product initiative, PF-60-S02 `in_progress`.
  Product heading **Measurement targets**; excerpt: “No commercial performance
  numbers have been supplied. The following metrics must be instrumented, with
  targets set through the decision rights defined above.”
- Toolchain: rustc 1.95.0 (59807616e 2026-04-14), rustfmt 1.9.0-stable,
  just 1.58.0; macOS aarch64. Test isolation was read before execution.
  Every Rust test used the checkout's guarded `just test`; no live profile,
  native credential prompt, provider inference, push or review subagent occurred.

### Implemented contract

The separate internal combined mode shares one deferred request identity with HTTP
fallback. The HTTP-only mode, its existing tests and default Disabled configuration
remain intact. WS frames receive durable intent and original-price binding in the
socket pump immediately before send, after serialization and the existing guard.
Queue/consumer cancellation closes the result channel; the pump checks it before
admission and send. A committed admission with an uncertain or cancelled send
remains unknown: this is the dispatch crash window, not proof of free inference.

Established connections carry nonsecret endpoint/API-key provenance captured from
existing resolved setup, including preconnect/prewarm. Current and cached routes
must agree before accounted sampling; the pump retains the immutable binding.
Handshake redirects latch accounting failure and cannot become repair HTTP sends.
HTTP fallback retains the existing no-redirect client and final-URL admission.

WS text uses the existing Responses decoder before wrapped errors and typed event
conversion. Each response retains its own observer/source and checked text position;
writes are awaited before completion. Missing/null fields do not manufacture zero.
Failure/cancellation latches and existing connection invalidation remain effective.
Original pricing reuses `responses_original` unchanged, including its exact
`openai-responses-api-key-bundled-v1` source tuple and null unsupported economics.

### Commands and attempt ledger

All commands below have the exact CARGO_TARGET_DIR prefix above, run from this
checkout, and end in `--locked --offline`. Shell log capture uses `set -o pipefail`
and `2>&1 | tee /private/tmp/acct-ws-impl-03-evidence/<log>`.
The suffixes and selectors below are literal, not suggested future commands.

| Key | Guarded command after the prefix |
| --- | --- |
| A | `just test -p codex-api -E 'test(responses_websocket_accounting)' --locked --offline` |
| C | `just test -p codex-core --lib -E 'test(accounting_responses_ws)' --locked --offline` |
| N | `just test -p codex-core --test all -E 'test(accounting_responses_ws_native)' --locked --offline` |
| R | `just test -p codex-core -E 'test(accounting) \| test(websocket) \| test(prewarm) \| test(incremental) \| test(stage_one) \| test(agent::role)' --locked --offline` |
| T | `just test -p codex-api -p codex-websocket-client -p codex-http-client -p codex-login -E 'package(codex-api) \| package(codex-websocket-client) \| (package(codex-http-client) & (test(redirect) \| test(custom_ca))) \| (package(codex-login) & (test(redirect) \| test(custom_ca)))' --locked --offline` |

Markdown escapes in the table display literal shell `|` operators inside the
single-quoted selector. They are not backslashes to pass to the shell.

| Attempt | Run ID | Exit; result; filtered out |
| --- | --- | --- |
| A1 | `3032d42c-c16f-4149-b481-28ff6cfefc8c` | 100; 2 pass, 6 fail (each tried twice); 216 filtered |
| A2 | `bb864a39-356d-493e-b8ef-37b68766107b` | 100; 6 pass, 2 fail (each tried twice); 216 filtered |
| C1 | No run ID: compilation failed | 101; zero tests executed |
| C2, `core-unit-02.log` | `ca4d9672-52bc-4a71-b640-66e963187c1e` | 0; 8 pass; 2451 filtered |
| N1, `native-01.log` | `39ca17cb-a8e3-42a0-a126-1292c8b18ebf` | 0; 20 pass, 1 leaky; 1137 filtered |
| A3, `api-03.log` | `d34c5c65-ec23-4a5c-a8bb-4ede48675bf4` | 0; 8 pass; 216 filtered |
| N2, `native-02.log` | `5d28c06e-0f65-4e2e-9153-b0fd62b45e8f` | 0; 20 pass; 1137 filtered |
| R1, `core-regression-01.log` | `da31f485-5669-4ae6-81b3-c3a755f44fe9` | 0; 200 pass, 2 leaky; 3420 filtered |
| T1, `api-transport-regression-01.log` | `d5bb49a9-41e0-4890-9fc4-f78e5c4c8e9d` | 0; 262 pass; 239 filtered |

A1's six socket cases failed before dispatch because the new fixture did not
negotiate the compression extension used by the retained production connector.
A2 corrected handshake configuration; its terminal-container and observation-barrier
fixtures still omitted typed display-usage fields. Complete synthetic usage fixed
those fixtures without changing the decoder or production terminal semantics.
C1 rejected dynamic SQL under SQLx's audited-string API and a String-to-anyhow
conversion; literal fixture queries and explicit error conversion fixed both.

N1's leaky case was `fallback_http_transport_retry`. R1's leaky cases were
`accounting_price_projection_retains_absent_read_and_write_and_exact_milli` and
`accounting_responses_ws_cached_connection_provenance`. Later clean passes do not
erase these attempts. No flaky pass was reported in the completed runs above;
execution skips were zero, distinct from the reported selector exclusions.
Existing dead-code warnings in state/protected-state/Core support remain disclosed.

Artifacts live at `/private/tmp/acct-ws-impl-03-evidence/`. Preserved JUnit files
are named by their actual XML `uuid`, matching the run IDs above. A1's JUnit was
overwritten before preservation; A1/A2/C1 raw output remains in the dispatch tool
transcript, not a fabricated standalone log. A2's matching JUnit is preserved.
Final run results and artifact hashes are appended below.

### Case map and vectors

Prefixes: API rows use `responses_websocket_accounting_`; Core rows use
`accounting_responses_ws_`; native rows use `accounting_responses_ws_native_`.
Each suffix is the exact corresponding name in the preserved 36-case table.
Passing names alone are not an independent review of assertion completeness.

| Suite / suffix | Assertions and evidence |
| --- | --- |
| API / none_preserves_legacy | Ordinary frame/model, legacy invalid numeric tolerance and normal completion |
| API / pump_admission_barrier | Real peer receives no frame while admission held; release sends once; denial sends none |
| API / queued_cancel_and_send_failure | Consumer cancellation while waiting for stream ownership/admission; peer failure after admission retains no invented usage |
| API / positions_and_replacement | Intervening text advances positions; repeated values, missing and null remain distinct |
| API / usage_and_terminal_containers | Completed/failed/incomplete plus both usage locations; conflicting dual containers reject |
| API / invalid_evidence_stops | All six fields: negative, fraction, string, boolean, overflow; malformed JSON/container/detail; no completion |
| API / observation_barrier | Held numeric persistence blocks completion; rejection terminates |
| API / response_local_binding | Separate observers, positions and counts through independent real connections |
| Core / mode_scope_and_lazy_bootstrap | Disabled, HTTP-only and combined distinction; no early table installation; one resolved Sampling |
| Core / auth_route_eligibility | API key positive; command/bearer/header/env-header/Chat/subscription/agent identity/missing-auth negatives; initialized exclusion latches |
| Core / exact_endpoint_binding | HTTP/HTTPS scheme mapping; userinfo/query/fragment/scheme rejection; host/path/port mismatch |
| Core / cached_connection_provenance | Matching reuse; stale endpoint, unknown provenance and unsupported original route reject |
| Core / fallback_keeps_request_and_predecessor | Shared WS/HTTP request UUID, distinct linked attempts, late response-local observation |
| Core / original_price_binding | Literal source UUID tuple and 5/30/0.5 rates; Astra/alias/remote/tier nulls; previous snapshot retained |
| Core / failure_and_cancellation_latch | Pre-permit cancellation; bootstrap cancellation/missing DB; observation rejection and scope cleanup |
| Core / role_inheritance_preserves_reserved_provider_rule | Instruction-only role inherits mode; reserved openai provider override rejects |
| Native / off_and_http_only_compatibility | One handshake, one warmup and one sampling frame; no accounting tables |
| Native / complete_and_partial_goldens | One admitted sampling frame per write-present/absent vector; separate typed SQLite reads and exact totals |
| Native / prewarm_preconnect_and_cached_reuse | One connection; warmup reports 999/999 and remains unobserved; incremental sampling retains only its own counts |
| Native / incremental_sampling_and_turn_identity | Real shell tool continuation; incremental input/output; two distinct request IDs in one turn |
| Native / upgrade_required_http_fallback | One 426 handshake, zero WS frames, one HTTP POST and admitted attempt |
| Native / ws_prefix_then_http_fallback | One WS prefix plus one HTTP POST; linked attempts/source IDs, exact 0.00322 total |
| Native / connection_limit_reconnect | Two handshakes, one warmup, two linked sampling attempts; first unknown |
| Native / previous_response_missing_full_retry | Native incremental rejection then full request without previous_response_id; two linked attempts |
| Native / fallback_http_transport_retry | One WS frame then HTTP 503/success; three linked attempts, two unknown estimates |
| Native / handshake_and_postdispatch_errors | Handshake 401 creates no intent; actual post-frame 400 leaves one unknown intent; no repair POST |
| Native / redirects_never_escape_binding | Five WS redirects and five 426-to-HTTP redirect vectors; no target request; no repair |
| Native / endpoint_and_cached_auth_mismatch | New endpoint mismatch; held subscription-auth warmup followed by current API key fails without sampling frame/table |
| Native / admission_and_guard_barriers | SQLite BEGIN IMMEDIATE blocks frames; trigger denial admits none; actual native wrong-owner stage-one factory denial |
| Native / observation_failure_no_repair | One committed prefix, failed next write, no extra frame/POST and unchanged committed observations |
| Native / cancel_before_and_after_dispatch | Held database before dispatch and real dispatched socket cancellation; no late frame; unknown intent retained |
| Native / two_reopens_and_original_prices | Unknown/priced-prefix/null-price vectors; two native resumes preserve IDs, positions, snapshots and totals; fresh request differs |
| Native / spawned_role_children_and_fork | Real root plus two children, role reload, held concurrent frames, child retry ownership/edges, uncharged native fork |
| Native / delete_rejects_late_usage | Held owner deletion fences late usage; unrelated owner survives; installed-but-OFF resume deletion cleans its rows |
| Native / unknown_prices_and_no_usage | Astra nonzero usage has null price; successful absent usage remains unknown |
| Native / auxiliary_scope_and_event_parity | Native compaction POST leaves sampling attempts/observations unchanged; normal completion preserved |

The native support reuses existing readback helpers without editing them. It logs
synthetic attempt/observation rows from separate SQLite connections; all admissions
come from actual native UserInput dispatch, never seeded accounting attempts.
The four literal numerical expectations are now executable assertions: full
0.00161 with noncached 80, omitted-write subtotal 0.00121/full unknown, two attempts
0.00322, and repeated cumulative evidence within one attempt 0.00161.

### Qualification limits and receiving work

Sampling only: startup generate=false, handshake-only preconnect, Chat/Corbanu,
auxiliary collection and complete application/economic coverage remain deferred.
Prewarm token evidence is deliberately excluded, not attributed later or called free.

The new native guard test proves a real owner-factory denial, not an injected
stage-one denial at an already-running accounted WS frame. Existing guard regression
tests supply supporting evidence; a stronger combined-path assertion remains an
explicit receiving-review question. Safety/model-verification/moderation event
parity is covered by retained API tests and unchanged downstream processing;
the new native auxiliary case asserts compaction/completion, not every metadata
variant. Those two frozen native vectors remain only partially proven: the current
native harness does not inject a stage-one binding into UserInput sampling, and the
new auxiliary test does not emit the full metadata matrix. Manager disposition or
further scoped fixture work is required before claiming every vector complete.
These narrower assertions are not approved as equivalent to every frozen vector.

The manager still owns independent Fable material review, review allowance,
receiving-tree shared state/TaskNode and combined API/proxy gates, and plan/sprint
bookkeeping. None is asserted performed by this worker. Internal-only functional
N/A requires named-integrator acceptance; later S03/S04 isolated code-blind execution,
independent evidence review, true-TUI and applicable live-repository qualification
remain required. No TensorCash/Isometric, live-provider, benchmark, cross-platform
or release qualification is claimed.

Only the 18 allocated repository paths are changed. Excluded support, decoder,
price/catalog, auth/transport-policy, manifests, locks and BUILD files are untouched.
Manual corrections and file-scoped rustfmt precede final Rust tests. Whole-crate
automatic fix/global format was not used because its writes could exceed this
literal allocation; the lease's guarded-test-only execution restriction is retained.
Stable rustfmt reports that imports_granularity is nightly-only; formatting itself
exits 0. Final checks, exact diff counts, hashes and candidate commit are returned.

### Final formatted-tree verification

| Command key / log | Run ID | Exit; result; filtered |
| --- | --- | --- |
| A / `api-final.log` | `28acbd12-4329-440c-81e2-f205aaad23e6` | 0; 8 passed; 216 filtered |
| C / `core-unit-final.log` | `b48bb343-ceec-41cd-8f48-ba7767e8e8eb` | 0; 8 passed; 2451 filtered |
| N plus `--success-output immediate` / `native-final.log` | `64dd17f5-8042-40f7-85bf-4c1814befcb3` | 0; 20 passed; 1137 filtered |
| R / `core-regression-final.log` | `64afca8e-d250-4e09-b1e7-ef14b3fe2d0f` | 0; 200 passed; 3420 filtered |
| T / `api-transport-regression-final.log` | `c494fd6a-d666-44cf-a045-facf86ea75a3` | 0; 262 passed; 239 filtered |

The final native command inserts `--success-output immediate` immediately before
`--locked --offline`; it preserves successful raw fixture and SQLite output.
All five final runs report no failed, execution-skipped, flaky or leaky tests.
Explicit path attributes were then added to the two native suite registrations;
the subsequent final Core regression run verifies those final registrations.

Raw native counts (handshakes, warmups, sampling frames, HTTP POSTs):
successful/partial/OFF/HTTP-only sampling `(1,1,1,0)`; incremental tool continuation
`(1,1,2,0)`; connection-limit and previous-response recovery `(2,1,2,0)`;
prefix fallback `(1,1,1,1)`; HTTP retry `(1,1,1,2)`; cached subscription mismatch
`(1,1,0,0)`. Compaction adds one auxiliary POST without an accounting attempt.
For each of 301/302/303/307/308, direct WS rejection produced two handshakes
(prewarm and sampling), zero frames/POSTs/target requests. Each 426-to-HTTP
redirect vector produced one handshake, zero frames, one POST/attempt and zero
target requests. This is observed output, not a substituted retry policy.

The final native log contains separate-connection attempt and observation JSON
for all native scenarios, including linked request/attempt IDs, owner IDs, source
positions and absence-preserving patches. Numerical assertions and the two-reopen
case verify the four literal goldens and unchanged original/null snapshots.

Structural checks before this receipt's final bookkeeping: plan checker exit 0,
active 3/3; sprint checker exit 0, current 116/archived 126; whitespace exit 0.
Final repetitions are included in RETURN. No size STOP, scope reallocation,
new product decision, independent material review or receiving acceptance occurred.

Final log SHA-256 values (under `/private/tmp/acct-ws-impl-03-evidence/`):

| Log | SHA-256 |
| --- | --- |
| `api-final.log` | `f5d2c2fd96dcb1e63fbf87d4100a4144211fe852fc1b49bc0e392441b7dc0bc1` |
| `core-unit-final.log` | `2aba7fbea0663e1c1f8fcc9f9fcab1d200ca7543e6da120fb44445de10ac5d10` |
| `native-final.log` | `0d9986268d21137ab72df75b19743c60c1f36f06651be85a080d3f4da79bfb80` |
| `core-regression-final.log` | `b5f6a46efce392ce347689d2f18bb89f68698789fb0e52b5c53d9be6e897b368` |
| `api-transport-regression-final.log` | `71ee92c3d624f172292d79286fb8570e4db1c0934b1fc96cb1b1052c8b4283d0` |

Saved JUnit files use their actual run UUID as basename. The final Core and transport
JUnit hashes are `809dec1878b570351230361c23995ef319e5e13dc055b6234825004e77577c0d`
and `57e64f780e8256009225177dcb647ff4295226de276a3eedf6f13519dd62d2df`.
Generated `candidate.patch` and `SHA256SUMS` in that evidence directory accompany
RETURN; the manifest covers allocated files, saved XML, logs and the full patch.
Final file-scoped rustfmt check exits 0; no `.snap.new` exists in Core/API.
Final conservative size: 2707 changed lines, 735 non-test; all 18 allocated paths.
Targets 3000/1050 and STOP 3300/1200 are respected. Candidate commit is supplied
in RETURN, avoiding a self-referential commit hash in this committed receipt.

## acct-ws-impl-04 — discriminating redirect-latch regression

Routine test/evidence revision of PF-60-S02: no retained production behavior change.
Product heading **Measurement targets**; excerpt: “No commercial performance
numbers have been supplied. The following metrics must be instrumented, with
targets set through the decision rights defined above.”

- Worker: `gpt-6-astra` / `high`; action `acct-ws-impl-04`.
- Allocation digest: `8966074f735c6c7ccc8cfcbfb7966466a924a00fc0831792197c6cea32423f0d`.
- Claim: `50bfb2bd-324b-436f-a196-c6f5780ca498`.
- Brief: `/private/tmp/fmgr.Q1SIYZ/briefs/acct-ws-impl-04.json`; verified by
  `shasum -a 256`: `8b496f26b1cb1399d60c3cbede04604c814acc405b16c1ab5b49dc8c174d0ac8`.
- Actual clean dispatch base: `cb2b0a9110d3ae7d305a8220d0ab4a0a32cec64b`.
  Branch/worktree remain the allocated coordinates above. The current brief
  explicitly reuses the exclusive target `acct-ws-20260915`.
- Independent review input: `/private/tmp/fmgr.Q1SIYZ/acctws3-review.json`,
  overall verdict “patch is correct”; single P3: the native redirect case
  passed identically with the redirect-latch match arm deleted.

The existing `accounting_responses_ws_native_redirects_never_escape_binding`
now requires the exact fatal accounting error for each direct WS redirect.
A plain unexpected-status transport error cannot satisfy that assertion. All
original statuses (301/302/303/307/308), fallback variants, retry settings, error
presence, zero target requests and POST/attempt count assertions are retained.
This observes the accounting-specific failure at the sampling retry boundary;
it does not infer latching merely from the absence of network sends.

### Mutation experiment

Before writing the assertion, removed exactly this eight-line match arm from
`codex-rs/core/src/client.rs` using an exact-text edit:

```rust
                Err(ApiError::Transport(TransportError::Http { status, .. }))
                    if status.is_redirection() && sampling.is_some() =>
                {
                    if let Some(deferred) = &deferred {
                        deferred.reject();
                    }
                    return Err(CodexErr::Fatal(crate::accounting::FAILURE.into()));
                }
```

Then added the assertion and formatted the test file. Both mutation and restored
runs used this identical command from the allocated worktree:

```sh
CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/acct-ws-20260915 just test -p codex-core --test all -E 'test(accounting_responses_ws_native_redirects_never_escape_binding)' --retries 0 --success-output immediate --locked --offline
```

| Tree | Run ID | Result |
| --- | --- | --- |
| Latch arm removed | `d4d6c364-bc7e-4990-8ba7-28690164132b` | Exit 100; 0 passed, 1 failed; 1156 filtered |
| Exact arm restored | `cccb60ec-c90d-492c-b4da-786cd47ea877` | Exit 0; 1 passed; 1156 filtered |

Mutation failure output (tool output chunk `e2e256`):

```text
assertion failed: `(left == right)`: WS redirect 301 must latch accounting failure
Diff < left / right > :
 [
<    "unexpected status 301 Moved Permanently: Unknown error, url: ws://127.0.0.1:54862/v1/responses",
>    "Fatal error: Native Anthropic accounting failed; request stopped without a repair send",
 ]
Summary [   0.617s] 1 test run: 0 passed, 1 failed, 1156 skipped
```

Restored output (tool output chunk `d4e926`):
`Summary [   3.137s] 1 test run: 1 passed, 1156 skipped`.
Every direct redirect printed two handshakes (prewarm and sampling), zero frames,
zero POSTs, zero attempts and zero target requests. Each 426-to-HTTP redirect
printed one handshake, zero frames, one POST/attempt and zero target requests.
`git diff --exit-code -- codex-rs/core/src/client.rs` exited 0 after restoration.

### Positive recovery controls and final checks

Unchanged native controls demonstrate that ordinary recovery remains executable:

```sh
CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/acct-ws-20260915 just test -p codex-core --test all -E 'test(accounting_responses_ws_native_connection_limit_reconnect) | test(accounting_responses_ws_native_upgrade_required_http_fallback) | test(accounting_responses_ws_native_fallback_http_transport_retry)' --retries 0 --success-output immediate --locked --offline
```

Run `168f150e-82de-40bf-855d-2f993a4920bf`: exit 0; 3 passed, 1154 filtered.
The reconnect case records two linked WS attempts; transport retry records one WS
attempt followed by two linked HTTP attempts. The 426 control succeeds over HTTP.
Final JUnit at `codex-rs/target/nextest/local/junit.xml` identifies this control run;
earlier report contents were overwritten by nextest, so the mutation/restoration
run IDs and observed output are preserved above rather than claiming saved XML.
All three runs disabled automatic retries; no flaky/leaky result was reported.

Read test-isolation guidance before testing; every Rust test used guarded
`just test` with disposable profiles and native-keyring denial. No live profile,
native credential prompt, raw cargo/nextest command, provider inference or push.
File-scoped `rustfmt --edition 2024 --config skip_children=true` and its `--check`
passed; stable rustfmt emitted the existing nightly-only imports warning.
Global format/fix was avoided to respect the literal writable manifest.
Plan/sprint checkers passed (active 3/3; current 116/archived 126); whitespace passed.

Only the test file and this receipt differ from the dispatch base; production
source is restored byte-for-byte. Internal test-only work has no changed user
workflow, so new interactive/code-blind execution is not applicable to this
revision. The later S03/S04 functional gates and all previously disclosed
qualification limits remain pending; no new acceptance or readiness is claimed.
Cumulative size from `59260f56e0f201fdadef9f73f25ab68a791041e5`: **2831 total / 841 non-test**,
including receipt and mixed glue; below targets 3000/1050 and STOP 3300/1200.

## acct-ws-vectors-01 — STOP: live stage-one injection needs additional scope

Routine inspection/evidence continuation of the received PF-60-S02 unit; no Rust
source changed. Product heading **Measurement targets**; excerpt: “No commercial
performance numbers have been supplied. The following metrics must be instrumented,
with targets set through the decision rights defined above.” S02 remains in_progress.

- Action `acct-ws-vectors-01`; worker `gpt-6-astra` / `high`.
- Allocation digest `4a580987f9e9065388b1fd56054338da96efe4622e19742f6ee745aee9766da4`;
  claim `aa63b013-4eba-42b3-a3ca-e77ecbcdac14`.
- Brief `/private/tmp/fmgr.Q1SIYZ/briefs/acct-ws-vectors-01.json`; verified first with
  `shasum -a 256`: `34dd4586723da9c252a74f5fb3e6593ccaeea3b2a6aa48a499ebf19aa40bff35`.
- Clean starting HEAD matched dispatched `a7682bc5c01e8196457932b19668fda6bd7bb7e4`;
  worktree and branch match the assignment. The recorded plan/sprint base is an
  older receiving ancestor; this dispatch explicitly extends the received unit.
- Assigned target `/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/acct-ws-20260915`
  was not used. No Rust build/test, live profile, credential read/prompt, provider
  inference, subagent or push occurred.

### Concrete boundary and smallest proposed extension

`core/src/client.rs` stores `stage_one_memory_binding` as a private optional Arc.
Its consuming `with_stage_one_memory_binding` constructor is crate-private; its
only production caller is `StageOneMemoryClient::new` in `memory_stage_one.rs`.
That module owns the binding's private fields and the opaque memory client's
private client/binding. `CodexThread::stage_one_memory_client` in `codex_thread.rs`
returns that separate memory client, not a handle to UserInput's sampling client.
Cloning/replacing a ModelClient does not update an existing session's optional
binding. The WS check at `client.rs:3546` runs before stream dispatch; the live
WS response loop has no stage-one check. Memory extraction's completion checks
belong to the separate opaque memory client and do not wrap UserInput sampling.

The required real binding cannot be constructed/extracted by the allocated native
integration fixture. A fabricated accounting error, fresh wrong-owner factory call,
client replacement or stream cancellation would not prove mid-dispatch binding.
No synthetic substitute, unsafe private-field access or new production policy
setter was introduced to claim this vector passed.

Fable manager should add exactly `codex-rs/core/src/memory_stage_one.rs` and
`codex-rs/core/src/codex_thread.rs` to a revised allocation: the former for a narrow
host-owned binding fixture seam, the latter for native-thread attachment to the
same running sampling client. Shared binding/stream checks and native cases can
remain in the already allocated client/turn/accounting/test paths. The revised
mandate must specify the attachment boundary and in-flight denial result with the
existing security owner: the frozen allocation specifies pre-dispatch guards and
zero frames on denial, whereas this brief additionally requires applying a newly
arriving binding after dispatch and preserving a committed prefix. An integration
fixture seam must not create a public production policy setter; its availability
in the integration-test library build needs explicit design in that allocation.

### Vector and experiment disposition

The brief says STOP if either vector needs an additional path. That condition was
found before edits, so neither `accounting_responses_ws_native_admission_and_guard_barriers`
nor `accounting_responses_ws_native_auxiliary_scope_and_event_parity` was extended.
Both mutation experiments: **not run**; commands/run IDs/failure output: **none**.
The metadata matrix appears reachable through the existing Gate but was not
implemented after the required STOP. Both previously disclosed gaps remain open.
No new pass, independent acceptance, human-test handoff or release readiness is claimed.

Read `docs/development/test-isolation.md` before checks. Pre-edit sprint checker
passed (current 116, archived 126). Final plan/sprint/whitespace results and exact
size are supplied in RETURN. Rust formatting is N/A because no Rust file changed.
Only this receipt is committed; manager-owned plan/sprint records remain untouched.
