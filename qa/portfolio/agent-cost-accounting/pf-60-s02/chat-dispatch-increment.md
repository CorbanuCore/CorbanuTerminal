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
