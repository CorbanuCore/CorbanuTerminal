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

## Fourth attempt — acct-chat-impl-04 (2026-09-15)

Worker `gpt-6-astra / high`; receiving owner Fable manager. Clean launch/commit
`7b486e2d37bdf73c2b6dfc3674ff49bc9fa536c4`, same branch/worktree above.
Allocation `2a22884dfee9de20161082fd8f6fa951eb7e5ce7602d7b2de3ec51bf9346e27c`;
claim `4dace49d-0c55-44f9-9e01-3ba73b823960`. Brief SHA-256
`28cb691da77941421b4c2fad45f53e79adab3467cac2a2165f48c381ecf4508f`;
design SHA-256 `0736d39224191dab79f1215b286d3939f4109cde55c0fc4c00a93679f1f5268b`.
Both verified before work with `shasum -a 256`, exit 0. Frozen continuation explicitly
reconciles prior dispatch prerequisites and authorizes this target; stale lease text is historical.
Existing PF-60 initiative / **Measurement targets** citation above still governs S02.
Collection remains OFF; no activation, gateway economics, prewarm/auxiliary collection or P3 repair.

### Corrected contracts and prior failures

- Length: frozen expectation “length continuation then stop two distinct sampling requests”.
  Actual `session/turn.rs`: `Some(codex_api::CompletionFinishReason::Length) =>` returns
  `CodexErr::InvalidRequest`, “was stopped without executing further model work”.
  Manager explicitly dispositioned this in favour of production. The corrected same named
  case asserts that error, exactly one attempt and its retained observation; no successor invented.
- Auxiliary: replace length with a real safe shell call plus stop; both sampling UUIDs differ
  within one turn. Assert successful `ExecCommandEnd` exit 0 in both tool cases.
  Frozen “Native manual/local compaction uses its separate session” meets another production
  counterexample: `supports_remote_compaction()` is `self.is_openai() || is_azure_responses_provider(...)`.
  `tasks/compact.rs` selects remote even for direct Chat. Following the manager's production-wins
  disposition, the fixture accepts one unary `/v1/responses/compact` POST; it adds no accounting
  rows. Do not label that a local Chat compaction or a collected/free auxiliary operation.
- Spawn: the first fixture searched only for `spawn_agent`, which was namespaced and omitted
  by the existing Chat converter. Use the already-exposed `spawn_agent_plaintext` alias.
  Five physical sampling POSTs, three native owners, two spawn edges, the role-only retry
  and fork/no-copy assertions now execute. No namespace converter/handler change.
- OFF/cross-wire: Anthropic failed before sending because fixture model gpt-5.6-sol had no
  catalogued maximum output limit. Supply synthetic 1024 in that cross-wire fixture's local
  catalogue; both cross-wire vectors now send once with no accounting installation.
- Prior C1 compilation: explicit `map_err(anyhow::Error::msg)` was already in the launch
  candidate; final `--lib` now compiles and runs all eight new unit cases.
- A1's three cases (raw_before_lossy_chunk_conversion, positions_and_cumulative_patches,
  observation_barrier_and_rejection) failed their completion-count assertions on both initial
  and automatic retry attempts. Their fixtures omitted assistant content, which native
  `ChatStreamState::complete` requires. A2 added content before [DONE]. It was a fixture
  correction, not `--retries 0` making unchanged code pass. Final API executes all three
  without retries. No post-correction flake observed; original failed runs remain above.
- Fourth-attempt diagnostic D: 26/28 passed; only cross-wire and auxiliary fixtures still
  failed. After those corrections, M1 ran 27/28, failing only the deliberate mutation.
- N0 below failed compilation (DayTotals lacks Serialize) while adding numeric evidence
  output; no tests executed and no matching JUnit. Changed only the fixture print to Debug.

### Resource investigation — not resolved by a passing rerun

The prior two LEAK cases used SQLite pools whose error path returned before awaited close,
and the held-socket fixture spawned detached connection tasks. Close read pools on both query
outcomes, explicitly close both cancellation-fixture runtimes and deletion runtimes, drop the
held deletion sender, and own socket children in a JoinSet. These are concrete cleanup changes,
not proof that those resources caused nextest's historical output-handle markers.
Final L has no LEAK; final N has a new LEAK on cumulative_and_separate_usage. L/N process
sampling (40ms nominal interval; command names/ancestry, no credentials) ended with no sampled
descendants. Dedicated serial P re-executes the two historical cases plus the new case:
3/3, no LEAK, no sampled children of those three named test PIDs.
Short-lived processes can evade sampling. We have not identified the historical/concurrent
output-handle owner or proved a runner false positive. **Leak attribution remains unresolved**;
N is assertion-green but not leak-clean. P does not overwrite or waive N or prior C2.
Fable owns disposition/further bounded diagnosis; no unsupported claim of an inherent runner bug.

### Commands, run IDs and counts

All commands start with
`CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/acct-chat-20260915`
and run this checkout's guarded `just test`. Every run used `--locked --offline --retries 0`.
No live profile/native credential prompt or raw test runner was used.
Rust toolchain 1.95.0-aarch64-apple-darwin; rustfmt 1.9.0-stable (59807616e1).
Manual scoped fixes and `rustfmt --edition 2024 --config skip_children=true <18 allocated Rust files>`
precede final affected tests; the last fixture-print correction was formatted before N.
Stable rustfmt warns imports_granularity is nightly-only. No broad formatter or unguarded
`just fix`/Clippy build was run under the frozen guarded-test-only build restriction.
Final API/library source bytes were unchanged by subsequent native-fixture-only corrections.

Artifact root is the exact target above. Each stem below has raw `.log` and matching `.xml`
copied before the next run; N0 has only a log. L/N/P also have `-processes.json`.
Suffix commands below include their package/selector; append the common flags above.
Counts are passed/failed/filtered; JUnit execution skips are 0 for every executed run.

| ID / artifact stem (prefix impl04-) | Command after just test | UUID | Exit; counts |
| --- | --- | --- | --- |
| D / core-diagnostic | -p codex-core -E 'test(accounting_chat_)' | ce42f488-5152-41fe-94cf-4d5f2bdfa711 | 100; 26/2/3622 |
| M1- / m1-broken | -p codex-core -E 'test(accounting_chat_)' | c9d7b5a4-fa11-46ac-9bb4-1ff4b2b3f4e0 | 100; 27/1/3622 |
| M1+ / m1-restored | -p codex-core -E 'test(accounting_chat_native_top_level_error_retains_usage)' | 9e20856d-d8d7-47f3-87f6-9b69f547805f | 0; 1/0/3649 |
| M2- / m2-broken | -p codex-core -E 'test(accounting_chat_native_literal_partial_and_zero_goldens)' | 5cc66b2c-c07d-4d45-b97c-33563202638d | 100; 0/1/3649 |
| M2-C4- / m2-c4-broken | same M2 command; C4 temporarily first | 93c055dc-aa3c-4f45-ae84-a84d854813ae | 100; 0/1/3649 |
| M2+ / m2-restored | same M2 command; original vector order restored | 81e6ce93-0652-482c-b4f1-9a92bded7ac2 | 0; 1/0/3649 |
| M3-old / m3-original-broken | -p codex-core -E 'test(accounting_chat_native_invalid_evidence_no_repair)' | 7dfdce81-1c26-4c2e-9e8a-20631f55cf9d | 0; 1/0/3649; nondiscriminating |
| M3- / m3-strengthened-broken | same M3 command; strengthened assertion | 8fa0609a-d48c-48e1-a9b3-2b30fd710887 | 100; 0/1/3649 |
| A / final-api-transport | -p codex-api -p codex-login -p codex-http-client | 53117b3e-7350-4ae8-a96f-00c25f6c1841 | 0; 499/0/0 |
| L / final-core-lib | -p codex-core --lib -E 'test(accounting) \| test(session_startup_prewarm) \| test(incremental) \| test(agent::role::) \| test(stage_one) \| test(chat_completions)' | 56d02d3c-a861-4b08-b1f3-d96f033e0231 | 0; 86/0/2382 |
| N0 / final-core-native | same N command | no run UUID; compilation failed | 101; 0/0/unknown |
| N / final-core-native-2 | -p codex-core --test all -E 'test(accounting_) \| test(prewarm) \| test(incremental) \| test(chat_completions)' --success-output final | 47a8f9a2-2209-43f3-ae46-dd9b6d42d795 | 0; 97/0/1081; 1 LEAK, 1 slow |
| P / leak-probe | -p codex-core -E 'test(accounting_chat_bootstrap_cancel_scope_and_latch) \| test(accounting_chat_native_delete_rejects_late_usage) \| test(accounting_chat_native_cumulative_and_separate_usage)' --test-threads 1 | d65acf99-c88f-44f2-ba58-134720226ff5 | 0; 3/0/3647 |

M1 bypassed the pre-error observer: persisted observation count failed 0 versus 1, then restored passed.
M2 mapped write Missing to Number(0): C1 incorrectly became 0.00161/full-known and C4 lost
unknown populations; both failed, then restored C1/C2/C4 passed.
M3 removed only the outer Chat check. Original case still passed because resolve's independent
latch prevented a second POST. Preserve that counterexample: the frozen “no-repair POST assertion”
alone cannot distinguish this layer. Added no-StreamError/reconnect assertion; mutation then failed.
Restored M3 passes in N, preserving the no-second-POST assertions and all five bad-evidence vectors.
No mutation remains in the candidate; no extra test function or production retry behavior was added.

### Complete final 38-case map

Prefixes unchanged: A = chat_accounting_; U = accounting_chat_; N = accounting_chat_native_.
Run IDs refer to the table. Each row is an executed named function, not an added table-vector count.
N's raw success output preserves CHAT_POSTS/CHAT_HELD_POST, independent CHAT_ATTEMPTS /
CHAT_OBSERVATIONS, literal CHAT_PRICES and complete CHAT_TOTALS. Raw native telemetry is synthetic.
All 38 named assertions pass; cumulative's concurrent LEAK remains explicitly unqualified.

| Prefix / suffix | Run and asserted evidence |
| --- | --- |
| A presence_matrix | A; five fields null/zero/positive/max and malformed scalar/detail vectors |
| A top_level_containers_only | A; absent/null/empty, sibling error, ignored nested/vendor evidence |
| A raw_before_lossy_chunk_conversion | A; raw partial/null observation before typed conversion |
| A error_envelope_usage_precedes_error | A; held write blocks error; sibling/prior usage retained |
| A done_and_finish_reason_are_not_usage | A; stop/length/tool/content_filter/error/unknown parity |
| A positions_and_cumulative_patches | A; positions 1/3/5, repeated/null/missing patches, comments |
| A observation_barrier_and_rejection | A; held completion, release and rejection |
| A invalid_evidence_stops_before_done | A; malformed JSON/count, bounded error, no completion |
| A consumer_cancel_and_interruption | A; closed consumer/held observer, EOF/idle/activity timeout |
| A none_preserves_legacy | A; legacy corpus and ordinary wrapper event equality |
| U bootstrap_is_lazy_once_and_mode_local | L; deferred UUID, no installation before resolve, once |
| U direct_auth_and_gateway_eligibility | L; typed direct positive, override/gateway/routing negatives |
| U exact_final_endpoint_binding | L; complete URL vectors and actual post-auth mutation denial |
| U auth_and_guard_before_admission | L; auth rejection and live stage-one guard, zero sends/rows |
| U bootstrap_cancel_scope_and_latch | L/P; missing state/cancel/write rejection/RAII; old LEAK retained |
| U response_local_attempt_identity | L; reversed observer order, immutable attempt/source association |
| U role_inheritance_reserved_id_parity | L; instruction reload/complete mode; reserved override rejected |
| U prices_exact_source_and_unknown | L; literal tuple/time/rates, unsupported rows, retained sources |
| N off_and_mode_isolation | N; five Chat modes/state vectors plus Responses/Anthropic cross-wire |
| N literal_partial_and_zero_goldens | N/M2+; C1/C2/C4, seven populations, unknown write/full price |
| N cumulative_and_separate_usage | N assertions PASS + LEAK; P PASS; one subtotal, positions 1/3/5 |
| N top_level_error_retains_usage | N/M1+; one persisted prefix with/without sibling usage |
| N http_retry_policy | N; 503 two linked attempts; direct 429 one unknown intent |
| N outer_retry_prefix_and_ids | N; prefix/EOF then success, two sources, C3 0.00242/null |
| N api_key_401_no_invented_refresh | N; one unknown attempt, no manufactured refresh |
| N redirects_no_follow_or_repair | N; all five ON 3xx target-zero; OFF 307/308 follow controls |
| N mismatched_endpoint_never_sends | N; wrong approved endpoint variants, zero installation/POST |
| N admission_barrier_and_failure | N; real writer lock stops POST; trigger prevents second admission |
| N observation_failure_no_repair | N; trigger preserves prior rows, one attempt/no repair |
| N invalid_evidence_no_repair | N/M3 restored; five invalid vectors, prefix, no reconnect/repair |
| N cancellation_and_two_reopens | N; pre-admission/unknown/prefix cancel, two reopens, fresh request |
| N spawned_role_children_and_fork | N; 5 POSTs/3 owners/2 edges, role retry only, fork adds no rows |
| N delete_rejects_late_usage | N/P; public delete/late denial, other owner, OFF deletion; old LEAK retained |
| N immutable_prices_and_unpriced_rows | N; eligible/Astra/remote, priority absent on wire, two reopens |
| N sampling_auxiliary_scope | N; two same-turn sampling IDs via successful tool; remote compact excluded |
| N gateway_exclusion_and_header_parity | N; gateway success/released retry, header rotation/no rows |
| N missing_usage_and_correlation | N; absent/null/EOF, independent requests with reused provider ID |
| N finish_reason_and_tool_parity | N; one stop/length/error attempt; successful tool then second sampling |

### Qualification and remaining handoff

Final affected suites: 682 passing assertions (499 A + 86 L + 97 N), no failed tests or runner
execution skips; N has one unresolved LEAK. P is additional diagnostic evidence, not replacement.
The exact new manifest is 10 API + 8 Core unit + 20 native = 38, reconciled against JUnit.
Governance checkers pass (plans 3/3, current sprints 116/archive126); final whitespace check passes.
Per-file manifest, SHA-256s and conservative size accounting are in the worker RETURN.
Independent Fable review, receiving state/TaskNode-session and combined API/proxy gates remain
manager-owned and unexecuted here. No Clippy pass, leak-clean qualification or acceptance claimed.
Internal-only TUI/code-blind N/A remains proposed for inaccessible activation/OFF; integrator
acceptance and later S03/S04 confined execution/independent review/live repositories remain required.
No human-test readiness, human sign-off, benchmark, release or push. S02 open; S03 dependent.

Final size against implementation start 57cbefabb: 3232 additions+deletions / 917 non-test;
conservative per-file max with the unformatted launch candidate: 3237 / 922. Target overrun
237 total; STOP margin 63 total / 378 non-test. No size-ceiling extension is inferred.

## Fifth attempt — acct-chat-impl-05 (2026-09-15)

Routine fixture/evidence correction dispatched by Fable; worker gpt-6-astra/high, base
`1585a89c25a43d26672eaa9a88fc75ab20193bba`, same branch/worktree and PF-60/S02 linkage above.
Allocation `e4e43bea9a7e3176fcca6d290cf58b00121ccd32ae72bf666bab6d0f708b7231`;
claim `76cacab2-d973-47dc-a86e-4bd2263873de`. Brief hash verified with `shasum -a 256`, exit 0:
`2965c02743fad0f69e809446c161fb95ef267dcefdb2affc2f2536298eed6b93`.
The supplied Opus review accepted substance (confidence 0.72) with these two P3 findings:

- **LEAK cleanup:** cumulative_and_separate_usage now stops/drops the fixture and awaits
  `db.close()`, matching sibling cleanup. The original concurrent selector now passes cleanly.
  This closes the omitted teardown; no claim that historical output-handle ownership was proven.
- **Literal wire evidence:** `posts()` prints captured request count, method, `request.url.path()`,
  `body["model"]` and `body["stream_options"]["include_usage"]` for each observed request.
  Gate previously admitted/answered compaction in every fixture. Explicit `GateRoutes::ChatOnly`
  now rejects that route; only sampling_auxiliary_scope selects `ChatAndCompact`, retaining its
  existing exact compact-path assertion and no-added-accounting checks. No new cases or production edits.

Final command from this checkout (default concurrent local profile; no serial override):
`CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/acct-chat-20260915 just test -p codex-core --test all -E 'test(accounting_) | test(prewarm) | test(incremental) | test(chat_completions)' --success-output final --locked --offline --retries 0`
Run `25cff5a7-fe7e-453a-b841-ad8543d7d33e`: exit 0; **97 passed, 0 failed, 0 LEAK**, 1 slow,
1081 filtered skips, 0 JUnit execution skips, 46.585s. All 20 Chat cases executed.
Raw `.log` and matching `.xml`: target above, stem `impl05-final-core-native`; JUnit UUID verified.
Observed 17 CHAT_POSTS lines (one per request, count is the captured batch size), including:
`CHAT_POSTS 1 POST /v1/chat/completions model=gpt-5.6-sol include_usage=true`
Exactly one `CHAT_HELD_POST compact=true`. The unchanged API/library selectors retain impl04 evidence.

Before tests, from `codex-rs`: `rustfmt --edition 2024 --config skip_children=true core/tests/suite/accounting_chat.rs core/tests/suite/accounting_chat_recovery.rs core/tests/suite/accounting_chat_support.rs`, exit 0.
Same command with `--check`, exit 0 (3 files); stable-toolchain imports_granularity warning persists.
Scoped formatting preserves the writable boundary; no broad formatter or unguarded build.
Final `git diff --check`, `python3 docs/plans/check.py`, `python3 docs/sprints/check.py`: exit 0 each;
plans 3/3 active, sprints 116 current / 126 archived. Guarded disposable profiles only; no native prompt.
Earlier LEAKs, failures, length disposition and nondiscriminating M3 history above remain unchanged.
This internal test-only correction adds no user workflow; TUI/code-blind/live-repo qualification is
N/A for this round. Existing S03/S04 functional gates and manager receiving remain outstanding;
S02 remains in_progress. No human acceptance, benchmark, release, public activation or push claimed.
Size against 57cbefabb: 3286 total / 956 non-test; prior conservative formatting allowance yields
3291 / 961 (STOP margin 9 / 339). No size extension or scope expansion claimed.
