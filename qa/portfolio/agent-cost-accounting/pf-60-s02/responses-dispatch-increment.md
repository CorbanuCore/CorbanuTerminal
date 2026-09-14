# Responses dispatch increment — partial implementation STOP

Action: responses-dispatch-impl-02
Allocation digest: 4ccf4088510fd6a6ae0d02c730bbbbee4b260881c9e256f96e2bba6f049ca3ca
Claim: 96c6f8c9-d632-4176-881c-aa5ce13f9c9d
Worker runtime: gpt-6-astra / high
Base: fda20ff3a34d8fbae3fde25a1b196b0c8809826e
Branch: workstream/accounting-responses-dispatch-20260914
Worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/accounting-responses-dispatch-20260914
Allocation: docs/research/agent-cost-accounting/responses-dispatch-allocation.md
Date: September 14, 2026

## Result and STOP boundary

Partial implementation committed for manager recovery, NOT receiving acceptance.
The required scoped `just fix -p codex-core -p codex-api --locked` automatically
removed `pub use responses::spawn_response_stream;` from
`codex-rs/codex-api/src/sse/mod.rs`, an unallocated twenty-first source path.
The worker detected this in status/diff, stopped implementation, waited for the
command to finish, and restored exactly that one-line automatic edit.
The restored file has zero diff against the base. No extra candidate path remains.
No other unallocated file changed. This tooling incident is retained, not hidden
as compliance. The user explicitly required STOP on an added path.

The fix command exited 101: after removing the re-export in its library pass,
the API test pass reported E0425 at the new legacy-wrapper comparison.
A possible bounded continuation is to retain use of the existing wrapper in
the allocated endpoint's None-observer branch. That correction was not applied
after STOP. Manager disposition is needed before resuming this stopped action;
the incident does not establish that a twenty-first source path is necessary.

No new test executed. Source contains all 34 frozen test function names, but
runnable-name discovery failed during compilation, so these are declarations,
not 34 proven runnable or passing cases. No zero-match pass is claimed.

## Authority and initial state

This is the existing PF-60 product initiative. Exact product heading:
**Measurement targets**: “No commercial performance numbers have been supplied.
The following metrics must be instrumented, with targets set through the decision
rights defined above.” Active plan: portfolio-agent-cost-accounting.md.
Sprint: PF-60-S02, in_progress, with the authorized 20-file front-matter scope.

Initial branch/HEAD matched the dispatch exactly and git status was clean.
The plan and sprint still record base 25920ec8d17f3f8eb17cdb89f90892ca5848ae22,
and sprint body retains stale pause/two-path wording. The explicit user dispatch
at fda20ff3 was treated as current authority; no manager-owned record was edited.
Preflight and final plan/sprint checkers both passed: active 3/3, current 115,
archived 126. These counts do not validate the stale prose or qualify this code.
Prior receipt 6db0edc7 is historical; this file replaces it as explicitly directed.

## Implemented but not finally verified

- Internal DirectOpenAiResponsesHttp mode; ordinary config remains Disabled.
- Sampling-local UUID and deferred OnceCell bootstrap, scope cleanup and failure
  latch; only HTTP resolves it, leaving WS-only routing/prewarm unchanged.
- Typed API-key/config eligibility and approved endpoint comparison; native
  no-redirect constructor; admission remains inside the existing stage-one guard.
- Provider/dialect/request reuse of the existing ledger; per-response immutable
  attempt/source evidence and awaited numeric observation.
- Missing/null/nonnegative-i64 decoding before lossy Responses conversion,
  completed/failed/incomplete/separate usage handling and conflict rejection.
- Exact prospective bundled OpenAI rate projection with a separate source tuple,
  default-tier assumption and permanent null prices for unsupported economics.
  Existing Anthropic projection/source identity and accepted evidence retained.
- Mode-specific role provider overlay; existing provider fields are preserved.
- API/Core/native test drafts covering all frozen names, including real synthetic
  HTTP retry/redirect/barrier/cancel/reopen/child/delete/WS fixtures.

Remaining: execute and reconcile every frozen case and vector, complete
formatting and Clippy, fix fixture/runtime failures within manager disposition,
run all affected and compatibility selectors, then independent Fable review and
receiving gates. Several test drafts cover only part of their named case (for
example guard denial at constructor rather than a live guarded transport, role
OFF/full-auth vectors, unrelated-owner deletion and complete price replay).
Naming a test does not close those requirements. No out-of-scope disposition
or exception to the frozen acceptance model is inferred.

## Commands, toolchain and failed attempts

All Rust build/list/fix commands ran from W/codex-rs with this exact prefix:

```text
env RUSTUP_TOOLCHAIN=1.95.0 RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-pf60-s01-20260911/codex-rs/target
```

Toolchain: pinned 1.95.0, aarch64-apple-darwin; assigned exclusive Mac target.
The first build compiled dependencies. No clean/cache invalidation/install,
manifest/lock/BUILD repair, real provider call or credential inspection occurred.
Nextest itself invokes Cargo test --no-run; the worker did not run cargo test.

| Command after prefix | Actual result |
| --- | --- |
| cargo check --offline --locked -p codex-core -p codex-api --lib | Session 98337, exit 101: private Session import E0603 and unhandled PlanSchedule E0004. Both corrected within scope. |
| Same normal-library check | Session 98659, exit 0, 37.25s. Earlier tree only; one new unused admit warning plus existing dependency warnings. Not final-tree proof. |
| cargo nextest list -p codex-core -p codex-api --lib -E 'test(accounting_responses) \| test(responses_accounting)' --locked | Session 81516, exit 101: SQL query lifetime E0521 and missing FutureExt E0599. Both corrected within scope. Zero tests executed. |
| cargo nextest list -p codex-core -p codex-api -E 'test(accounting_responses) \| test(responses_accounting)' --locked | Session 79499, exit 101: two expected-count u128/i64 E0308 mismatches. Corrected within scope. Zero tests executed. |
| just fix -p codex-core -p codex-api --locked | Session 29084, exit 101; automatic unallocated re-export removal and subsequent E0425 described above. No Clippy pass. |
| Scoped rustfmt --edition 2024 --config skip_children=true | Two successful passes, 15 then 3 literal allocated Rust files; stable imports_granularity warnings. Later edits followed, so no final formatting pass. |
| git diff --check after rollback | Exit 0. Restored sse/mod.rs exact-base comparison exit 0. |
| python3 docs/plans/check.py; python3 docs/sprints/check.py | Both exit 0 before source and after STOP, counts above. |

Required just-test commands for codex-api, Core selectors, codex-state and
receiving combined suites were not reached. Pass/fail/skip execution counts are
0/0/0; compiler failures above are separate failed build attempts.
Cargo fmt/just fmt, final Clippy, full Core baseline comparison, retained
login/http-client selectors and receiving gates remain open.

Raw complete logs are local /tmp artifacts; retain them before temporary cleanup:

| Artifact basename under /tmp | SHA-256 |
| --- | --- |
| responses-dispatch-check-01.log | 429d8d781495aef90606c054afb20711f74a5c4322aab1da111a4d74795bf7ce |
| responses-dispatch-check-02.log | 11f47c90973cd0ab3a40200518fafb97d575b318086bad0693b495d7d1ac2e4b |
| responses-dispatch-list-01.log | 7206c4200e5a7e409642b9bb457cf56be2c54f975e082cf43e4ce34fc6efb51e |
| responses-dispatch-list-02.log | a0551947f246b6c34be556ac87544d343806861a61e8d9ceb20feb53d4ab4f27 |
| responses-dispatch-fix-01.log | 81cc7c8433ceb6a4a9de0c787ad15bd9a90aeb2b63e2723fe0607d9fa5c11c89 |
| responses-dispatch-fmt-01.log | 0b1640c2204b6c3704c7c20ce2cb1366421ef44f91e2390349767534013d1a41 |
| responses-dispatch-fmt-02.log | 40fb37a0cb48659f9e05e0228f8de81e088ad05b38d0d3ef39f45fdc488dddd6 |

## Counts and limitations

Source subtotal: 19 files, 2385 added+removed lines, 665 non-test, 1720 test.
Final 20-file totals: +2484/-88 = 2572 total / 852 non-test / 1720 test.
Dedicated test/support bodies count as test; all mixed glue/registration and
receipt additions AND deletions count as non-test. Bounds remain 3000/1150.

No public activation, protocol/config-schema/dependency change, push, child agent,
independent review or external communication. Anthropic assertions/receipt and
historical failed runs are unchanged. Native current operation is Op::UserInput;
the allocation's Op::UserTurn wording is stale and no protocol change was made.

This internal stage has no claimed human-test readiness, live-provider,
TensorCash/Isometric, TUI, whole-S02 or release qualification. Code-blind N/A
requires integrator acceptance; later S03/S04 functional gates remain mandatory.
No missing or failed evidence has been represented as passed.
