# Direct OpenAI Chat dispatch increment — stopped preflight

Date: 2026-09-15. Action: `acct-chat-impl-01`.
Worker: gpt-6-astra / high. Receiving owner: Fable manager.
Status: **STOPPED BEFORE IMPLEMENTATION**. No acceptance or running build lease claimed.

## Input identity

- Allocation digest: `ebf257ff8e5bb23b824ca37b7161f3de4d1d3765274a35a9571d49d56cb0a1f2`.
- Claim: `fda65791-bd9a-4f02-9503-006e9d348107`.
- Brief: `/private/tmp/fmgr.Q1SIYZ/briefs/acct-chat-impl-01.json`.
- Verified brief SHA-256: `56288b462ae3dc17e92b447715608cd874e5b2f92026a2f1b251a7adb014e323`.
- Frozen design: `/private/tmp/fmgr.Q1SIYZ/acct-chat-proposal.md`.
- Verified design SHA-256: `0736d39224191dab79f1215b286d3939f4109cde55c0fc4c00a93679f1f5268b`.
- Clean launch/base: `70b7cd33f9a17c1cbb219f4d0d7b00d4a6cf5acf`.
- Branch: `bootstrap/acct-chat-20260915`.
- Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/acct-chat-20260915`.

The proposed implementation is a PF-60 product initiative. Exact product-spec
heading: **Measurement targets**. Requirement excerpt: “No commercial performance
numbers have been supplied. The following metrics must be instrumented, with
targets set through the decision rights defined above.”
This receipt and ledger annotation are routine process evidence only.

## Blocking prerequisite and owner

The frozen design section 1 requires the manager, before implementation, to
reconcile the plan/sprint to the actual checkout, replace consumed mandates,
record the size exception and exclusive target lease, check reservations/diffs,
and run both governance checkers.

The actual S02 front matter still assigns the consumed Responses WebSocket unit:
`bootstrap-acct-ws-20260914`, branch `bootstrap/acct-ws-20260914`, base
`73fc51b1a73ec6304fb51603b6fb46ee9f61b56b`, and its old 18-file scope.
Its execution mandate explicitly defers Chat. Its precondition boxes for manager
checkout/reservation/build-target verification and governance reconciliation are
unchecked. The active plan's implementation_worktrees does not contain the
assigned Chat checkout. No Chat allocation file or Chat target lease was located
in the checked repository allocation/receipt records or supplied brief/design.

START authorizes the frozen assignment; it does not assert that these required
records and exclusive lease exist. The active plan is outside the worker's
writable scope. The design also assigns allocation/plan/lease work to the manager.
Do not invent manager approval or repair manager-owned scope by implementation.

Owner/action: Fable manager must reconcile the exact Chat allocation and matching
plan/sprint coordinates, record reservation/diff audit and an exclusive build
target, then redispatch against the resulting frozen base. No new product
decision, boundary renegotiation or human permission is requested.

## Changed-file scope and accounting

Only this receipt and one unchecked S02 Remaining-ledger entry are changed.
The explicit brief instruction to update Remaining authorizes that process edit;
no additional runtime path is touched. No Rust code or test function was added.
All changed lines, including this receipt and the sprint ledger, count as non-test.
Target remains 3000 total / 1150 non-test; STOP remains 3300 / 1300.
The final RETURN reports exact additions+deletions and file lengths from Git.
This is a stopped-attempt evidence commit, not an implementation candidate.

## Commands and outcomes

Commands ran from the assigned checkout. Test-isolation guidance was read before
any validation. Both digest commands exited 0 and matched their frozen values.

| Command | Exit | Actual result |
| --- | --- | --- |
| `shasum -a 256 /private/tmp/fmgr.Q1SIYZ/briefs/acct-chat-impl-01.json` | 0 | Matching brief digest |
| `shasum -a 256 /private/tmp/fmgr.Q1SIYZ/acct-chat-proposal.md` | 0 | Matching design digest |
| `git status --short` at launch | 0 | Empty; clean checkout |
| `git branch --show-current` | 0 | Assigned branch |
| `git rev-parse HEAD` at launch | 0 | Assigned base |
| `python3 docs/plans/check.py` | 0 | active 3/3; available slots 0 |
| `python3 docs/sprints/check.py` | 0 | current 116; archived 126 |

The checkers validate the records' internal consistency, not their agreement
with this new dispatch. Their success does not cure the observed stale allocation.
Rust test commands: **none**. Pass 0, fail 0, runner skips 0; all 38 required cases
are unimplemented/unexecuted, not runner skips. No build, fix, format, mutation,
provider call, native credential access, TUI or live-repository run occurred.
Initial receipt generation stopped before writing because its name parser
omitted the digit-containing 401 case; corrected parser verifies exactly 38 names.

## Case-to-evidence map

All names below come from frozen section 6. “Blocked preflight” means there is
no implementation assertion, execution log, POST count, database readback, or
passing result. All associated table vectors remain unresolved; none is waived.

| Frozen test name | Evidence / disposition |
| --- | --- |
| `chat_accounting_presence_matrix` | NOT RUN — blocked preflight |
| `chat_accounting_top_level_containers_only` | NOT RUN — blocked preflight |
| `chat_accounting_raw_before_lossy_chunk_conversion` | NOT RUN — blocked preflight |
| `chat_accounting_error_envelope_usage_precedes_error` | NOT RUN — blocked preflight |
| `chat_accounting_done_and_finish_reason_are_not_usage` | NOT RUN — blocked preflight |
| `chat_accounting_positions_and_cumulative_patches` | NOT RUN — blocked preflight |
| `chat_accounting_observation_barrier_and_rejection` | NOT RUN — blocked preflight |
| `chat_accounting_invalid_evidence_stops_before_done` | NOT RUN — blocked preflight |
| `chat_accounting_consumer_cancel_and_interruption` | NOT RUN — blocked preflight |
| `chat_accounting_none_preserves_legacy` | NOT RUN — blocked preflight |
| `accounting_chat_bootstrap_is_lazy_once_and_mode_local` | NOT RUN — blocked preflight |
| `accounting_chat_direct_auth_and_gateway_eligibility` | NOT RUN — blocked preflight |
| `accounting_chat_exact_final_endpoint_binding` | NOT RUN — blocked preflight |
| `accounting_chat_auth_and_guard_before_admission` | NOT RUN — blocked preflight |
| `accounting_chat_bootstrap_cancel_scope_and_latch` | NOT RUN — blocked preflight |
| `accounting_chat_response_local_attempt_identity` | NOT RUN — blocked preflight |
| `accounting_chat_role_inheritance_reserved_id_parity` | NOT RUN — blocked preflight |
| `accounting_chat_prices_exact_source_and_unknown` | NOT RUN — blocked preflight |
| `accounting_chat_native_off_and_mode_isolation` | NOT RUN — blocked preflight |
| `accounting_chat_native_literal_partial_and_zero_goldens` | NOT RUN — blocked preflight |
| `accounting_chat_native_cumulative_and_separate_usage` | NOT RUN — blocked preflight |
| `accounting_chat_native_top_level_error_retains_usage` | NOT RUN — blocked preflight |
| `accounting_chat_native_http_retry_policy` | NOT RUN — blocked preflight |
| `accounting_chat_native_outer_retry_prefix_and_ids` | NOT RUN — blocked preflight |
| `accounting_chat_native_api_key_401_no_invented_refresh` | NOT RUN — blocked preflight |
| `accounting_chat_native_redirects_no_follow_or_repair` | NOT RUN — blocked preflight |
| `accounting_chat_native_mismatched_endpoint_never_sends` | NOT RUN — blocked preflight |
| `accounting_chat_native_admission_barrier_and_failure` | NOT RUN — blocked preflight |
| `accounting_chat_native_observation_failure_no_repair` | NOT RUN — blocked preflight |
| `accounting_chat_native_invalid_evidence_no_repair` | NOT RUN — blocked preflight |
| `accounting_chat_native_cancellation_and_two_reopens` | NOT RUN — blocked preflight |
| `accounting_chat_native_spawned_role_children_and_fork` | NOT RUN — blocked preflight |
| `accounting_chat_native_delete_rejects_late_usage` | NOT RUN — blocked preflight |
| `accounting_chat_native_immutable_prices_and_unpriced_rows` | NOT RUN — blocked preflight |
| `accounting_chat_native_sampling_auxiliary_scope` | NOT RUN — blocked preflight |
| `accounting_chat_native_gateway_exclusion_and_header_parity` | NOT RUN — blocked preflight |
| `accounting_chat_native_missing_usage_and_correlation` | NOT RUN — blocked preflight |
| `accounting_chat_native_finish_reason_and_tool_parity` | NOT RUN — blocked preflight |

C1-C4, all seven aggregate populations, all retry/redirect/ownership/cancellation
vectors and all three mutation demonstrations are likewise unexecuted.
No historical test count is attributed to this worker.

## Remaining limitations

Direct Chat collection remains unimplemented by this action; collection stays OFF.
The two disclosed WS P3s, prewarm and auxiliary collection, gateway economics,
historical/full provenance, settlement, retention limits and complete application
coverage retain their prior unresolved status. S02 stays open; S03 stays dependent.
No independent review, internal-only integrator acceptance, functional handoff,
human acceptance, benchmark or release readiness is claimed.

## Second attempt — acct-chat-impl-02, 2026-09-15

Status: **STOPPED BEFORE IMPLEMENTATION** under the brief's explicit prerequisite
contradiction rule. The first attempt above remains historical evidence.

- Worker: gpt-6-astra / high; receiving owner: Fable manager.
- Allocation digest: `b37998ec13a52f0826742841199b6435a5a3bfca049b069f466f0c339ac9b726`.
- Claim: `d36b38d1-c6eb-4b4d-9718-3d0a7070655a`.
- Brief: `/private/tmp/fmgr.Q1SIYZ/briefs/acct-chat-impl-02.json`.
- Verified brief SHA-256: `ef969c8c56555d9ec31dfbd5fb04273a36a571ed8b41bbfa2958176c81200597`.
- Design SHA-256 verified again: `0736d39224191dab79f1215b286d3939f4109cde55c0fc4c00a93679f1f5268b`.
- Actual clean launch/base: `74a663e6e029a0d9b7846277605a7d8f0d6028a5`.
- Branch/worktree: `bootstrap/acct-chat-20260915` at `/Volumes/CorbanuDrive/Corbanu/worktrees/acct-chat-20260915`.
- Read the granted exclusive lease at `/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/acct-chat-20260915/LEASE.json`; its action, branch, worktree and base match this dispatch. No build used the lease or any target.

### Exact surviving contradiction

At the assigned base, S02's file
`docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s02-idempotent-usage-persistence-and-replay.md`
still has the WS owner/lane/scope/gate on lines 8–11 and WS worktree/branch/base on
lines 12–14. Its execution mandate on lines 26–27 requires the 18-file WS unit
and defers Chat. Its code boundaries on lines 38–40 require WS's 36 tests and
exclude price work. The new Chat introduction on line 22 and checked preconditions
on lines 46–47 coexist with those consumed executable instructions; they did not
replace them. The active plan's Chat row at
`docs/plans/active/portfolio-agent-cost-accounting.md:201` records base
`70b7cd33f9a17c1cbb219f4d0d7b00d4a6cf5acf`, unlike the actual/assigned/leased base
`74a663e6e029a0d9b7846277605a7d8f0d6028a5`.

The brief says to name the exact contradicting file/line and stop if a prerequisite
is still missing. Fable manager owns reconciling the executable S02 fields/mandate
and the plan's base before redispatch. The lease and published Chat allocation
now exist; their absence is no longer a blocker. No scope renegotiation or human
permission is requested. No manager approval or fix is inferred from commit prose.

### Second-attempt commands, manifest and case disposition

Both `shasum -a 256` commands (this attempt's brief and the same design) exited 0
with the digests above. Launch `git status --short` was empty; branch and HEAD
matched the dispatch. `python3 docs/plans/check.py` exited 0 (active 3/3), and
`python3 docs/sprints/check.py` exited 0 (current 116, archived 126). These validate
declared records, not agreement with this dispatch. Final checks are in RETURN.

Only this receipt and the S02 Remaining ledger change; all additions and deletions
count as non-test against 3000/1150 target and 3300/1300 STOP. Exact manifest,
lengths, SHA-256s, additions/deletions and commit are returned from the final tree.
No runtime path is changed. The explicit brief authorizes the Remaining update.

Rust test commands: **none**. Pass 0, fail 0, runner skips 0; all 38 names in the
case-to-evidence map above have the same **NOT RUN — blocked preflight** disposition
for this second attempt. Every associated vector, C1–C4 and all three mutations
remain unimplemented/unexecuted. No run UUID, JUnit, POST/frame or SQL evidence
exists for this action. No Rust build/fix/format, provider call, credential access,
TUI, live-repository run or independent review occurred. The preserved limitations
above apply unchanged; S02 remains open and S03 dependent.

## Third attempt — acct-chat-impl-03 (2026-09-15): STOPPED on native contract

Action `acct-chat-impl-03`; allocation digest `969742e4ec785623f7078d612106a866d2061ec53f53987732696ec10c0b2352`;
claim `160a39fa-5403-4ec4-8beb-b536fbee43da`; worker `gpt-6-astra / high`.
The brief SHA-256 is `eb0a92c6920a707c163bb4aa75f02fbc6db20be0ee4d5381967e5a278cdbab57`;
the design SHA-256 is `0736d39224191dab79f1215b286d3939f4109cde55c0fc4c00a93679f1f5268b`.
Both were read first and verified with `shasum -a 256`, exit 0.
Worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/acct-chat-20260915`, branch
`bootstrap/acct-chat-20260915`, clean dispatch base `57cbefabb27f13fc4611e81e1b136b65a0831eed`;
lease `acct-chat-20260915` matched. The branch-point discrepancy was reconciled as instructed.
This is the existing PF-60 product initiative, S02 in_progress. Product heading **Measurement targets**:
“No commercial performance numbers have been supplied. The following metrics must be instrumented,
with targets set through the decision rights defined above.” No new activation or product authority.

### Stop evidence and manager disposition needed

The frozen design section 6 says native `finish_reason_and_tool_parity` must perform
“length continuation then stop two distinct sampling requests”; section 6 also says a
fixture retry/count mismatch is a counterexample requiring manager disposition.
The actual native source `codex-rs/core/src/session/turn.rs`, at this attempt's lines 3640–3646,
matches `CompletionFinishReason::Length` and immediately returns `CodexErr::InvalidRequest`:
“was stopped without executing further model work”. The inherited block is unchanged.
The native parity test sent a content-bearing length response, observed one POST,
then timed out waiting for the required second POST. The auxiliary-scope test independently
hit the same length prerequisite before reaching compaction. Neither is a pass.
Changing that terminal branch would change an additional native behavior beyond this allocation.
STOP was taken on this concrete counterexample; the frozen expected count was not weakened.

Fable receiving manager owns disposition: reconcile the length case with existing native behavior
and select an already-supported two-sampling trigger for auxiliary scope, or separately authorize
a changed continuation contract. Recommendation: preserve the existing terminal behavior.
The spawned-role fixture also failed to find a Chat spawn tool; the outgoing converter drops
`ToolSpec::Namespace` in `client.rs::tool_spec_to_chat_tool`. Fixture/configuration investigation
remains; no namespace/tool conversion repair or invented spawn evidence was added.
OFF/wire-isolation's extra cross-wire vector expected one POST but observed zero; unresolved.
No new test dispatch or runtime correction followed the confirmed contract STOP.

### Candidate state and limits

The 18 allocated Rust paths contain a partial implementation and 38 named cases:
internal Chat mode/role inheritance; deferred per-sampling UUID scope; exact endpoint/no-redirect
transport admission; response-local numeric observer before terminal parsing; five-field presence;
cache-write always Missing; Chat-specific prospective bundled-price provenance.
Collection remains OFF by default, with no public configuration/feature/CLI/environment/TUI activation.
Native gateway collection and pricing remain excluded; its header-rotation fixture passed.
The native admission is a durable dispatch intent, not proof that bytes reached the server.
Failed sends remain unknown. Price binding is prospective local acceptance, not billed settlement.

This candidate is **unfinished and not ready for receiving or human testing**. Source changed while
the Core build was in progress: added predicate vectors, post-auth URL mutation, stronger identity
readback, early cancellation and unrelated-owner/OFF deletion assertions are not final-tree qualified.
Formatting was previewed read-only with rustfmt stdout; no final formatting, Clippy/fix or final-tree
test campaign was performed after STOP. Raw source is retained, not assertion-compressed to meet a gate.
The final manifest and conservative formatted-size projection are in RETURN; receipt and sprint
bookkeeping count as non-test. The explicitly requested S02 Remaining update is the sole bookkeeping
path outside the design's 19-file manifest. No other code path or shared manifest/schema was edited.

### Exact execution history

Every Rust test used the checkout's guarded `just test` after reading test-isolation.md.
Every command below was prefixed with
`CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/acct-chat-20260915`.
Disposable profiles/native-keyring denial were reported by the wrapper; no live-profile access,
native credential prompt, credential read, external provider request or raw cargo test was used.
The wrapper internally invokes Cargo; its rendered build command is not a separately dispatched command.

| Run | Exact guarded command after target prefix | Exit | Passed / failed / selected / filtered skips |
| --- | --- | ---: | --- |
| A1 | `just test -p codex-api -E 'test(chat_accounting_)' --locked --offline` | 100 | 7 / 3 / 10 / 224; default retry repeated the three failures |
| A2 | `just test -p codex-api -E 'test(chat_accounting_)' --locked --offline --retries 0` | 0 | 10 / 0 / 10 / 224 |
| C1 | `just test -p codex-core --lib -E 'test(accounting_chat_)' --locked --offline --retries 0` | 101 | 0 / 0 / 0 / unknown; compile E0277: role fixture's String error required explicit conversion |
| C2 | `just test -p codex-core -E 'test(accounting_chat_)' --locked --offline --retries 0` | 100 | 24 / 4 / 28 / 3622; 2 of the passing cases marked LEAK |

A1's three completion failures lacked assistant content in the fixture. The fixture was corrected,
then A2 passed without changing its completion assertions. C1's fixture conversion was corrected.
C2's four failures and two LEAK markers remain unresolved. A LEAK marker is not a credential prompt.
Matching JUnit copies are retained under the leased target (nextest itself writes its configured
`codex-rs/target/nextest/local/junit.xml`; Rust compilation used the exclusive target):
A1 `api-attempt-1.xml`, UUID `477a712e-28ea-4f87-84bc-fdd0cbc636f6`;
A2 `api-attempt-2.xml`, UUID `ab61521d-8faf-4c47-a8a0-3057b2993f8f`;
C2 `core-attempt-2.xml`, UUID `edacc8c8-74e1-4d62-94da-ca2c4e21ae2b`.
C1 never reached nextest, so no matching JUnit exists. Build/error output remains in this action's
tool transcript. JUnit's execution skips are zero; the table separately records filtered skips.

### Case-to-evidence map (38 names; intermediate artifacts only)

Prefixes: A = `chat_accounting_`; U = `accounting_chat_`; N = `accounting_chat_native_`.
A2 names reside in chat_accounting_tests.rs. U names reside in accounting_chat_tests.rs except
the price case in accounting_prices_tests.rs. N names reside in the two allocated native suites.
PASS below means that named intermediate executable passed, **not** that every frozen vector
or the final source was qualified. Stronger assertions written during C2 compilation need replay.

| Prefix / suffix | Evidence / disposition |
| --- | --- |
| A presence_matrix | A1/A2 PASS |
| A top_level_containers_only | A1/A2 PASS |
| A raw_before_lossy_chunk_conversion | A1 FAIL; A2 PASS after content fixture correction |
| A error_envelope_usage_precedes_error | A1/A2 PASS |
| A done_and_finish_reason_are_not_usage | A1/A2 PASS; API parity does not establish native continuation |
| A positions_and_cumulative_patches | A1 FAIL; A2 PASS after fixture correction |
| A observation_barrier_and_rejection | A1 FAIL; A2 PASS after fixture correction |
| A invalid_evidence_stops_before_done | A1/A2 PASS |
| A consumer_cancel_and_interruption | A1/A2 PASS |
| A none_preserves_legacy | A1/A2 PASS |
| U bootstrap_is_lazy_once_and_mode_local | C2 PASS |
| U direct_auth_and_gateway_eligibility | C2 PASS; added predicate vectors need final replay |
| U exact_final_endpoint_binding | C2 PASS; subsequent actual auth-mutation fixture unexecuted |
| U auth_and_guard_before_admission | C2 PASS, including live stage-one policy denial |
| U bootstrap_cancel_scope_and_latch | C2 LEAK; assertions passed, cleanup unresolved |
| U response_local_attempt_identity | C2 PASS; stronger association assertions need final replay |
| U role_inheritance_reserved_id_parity | C1 compile defect; C2 PASS |
| U prices_exact_source_and_unknown | C2 PASS |
| N off_and_mode_isolation | C2 FAIL: cross-wire vector 0 POST versus 1; unresolved |
| N literal_partial_and_zero_goldens | C2 PASS, all seven population assertions |
| N cumulative_and_separate_usage | C2 PASS |
| N top_level_error_retains_usage | C2 PASS; mutation not run |
| N http_retry_policy | C2 PASS: 503 two POSTs, terminal 429 one |
| N outer_retry_prefix_and_ids | C2 PASS: two linked attempts, C3 subtotal |
| N api_key_401_no_invented_refresh | C2 PASS: one unknown intent |
| N redirects_no_follow_or_repair | C2 PASS: five ON vectors and OFF 307/308 controls |
| N mismatched_endpoint_never_sends | C2 PASS |
| N admission_barrier_and_failure | C2 PASS |
| N observation_failure_no_repair | C2 PASS |
| N invalid_evidence_no_repair | C2 PASS; no mutation demonstration |
| N cancellation_and_two_reopens | C2 PASS; added before-dispatch variant needs final replay |
| N spawned_role_children_and_fork | C2 FAIL: no native Chat spawn tool; later vectors not reached |
| N delete_rejects_late_usage | C2 LEAK; later unrelated-owner/OFF assertions need final replay |
| N immutable_prices_and_unpriced_rows | C2 PASS |
| N sampling_auxiliary_scope | C2 FAIL: length prerequisite; compaction unexecuted |
| N gateway_exclusion_and_header_parity | C2 PASS |
| N missing_usage_and_correlation | C2 PASS |
| N finish_reason_and_tool_parity | C2 FAIL: native length terminates; tool/provider-error variants not reached |

All three required mutation demonstrations, retained regression selections, receiving API/proxy/state/
TaskNode-session gates, final formatting/fix and final-tree replay remain unexecuted after STOP.
The two WS P3s, prewarm/auxiliary collection, gateway/compatible/subscription/agent-identity economics,
historical/full endpoint provenance, non-token charges/settlement and retention-policy extensions
remain deferred exactly as frozen. S02 remains open; S03 remains dependent.
Internal-only TUI/code-blind N/A is proposed because activation is inaccessible and OFF; named-integrator
acceptance is still required. Later S03/S04 need independent confined functional execution, evidence
review, true-TUI keys and live-repository qualification. No review, human acceptance, benchmark,
live-provider, release qualification or push is claimed.

Size at stopped handoff, including this receipt and sprint: raw 2411 total / 667 non-test;
formatted preview 3041 / 751; conservative per-file max(raw, formatted) 3046 / 756.
This exceeds the 3000 total target by 46, remains below STOP 3300/1300, and is not final formatted evidence.
