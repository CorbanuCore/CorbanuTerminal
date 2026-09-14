# PF-60 workstream record — agent cost accounting

## Purpose and evidence vocabulary

**Planned — outcome:** Make the cost of one run and its descendants explainable across providers, including after restart. Missing usage or prices must remain visible as unknowns. This record consolidates the [active PF-60 plan][plan] and its evidence for review; the plan and sprints retain execution authority.

**Verified — document scope:** Routine research/status documentation under [repository policy][policy], prepared for action `tn-record-accounting-01`, Task Node task `task_b3e8506327fb906173fd68b2f642221b`. Source snapshot: `1fdb3fc1d85fdfaa041577146da0f0de0c1c3704`, branch `bootstrap/tasknode-record-acct-20260914`. Only this document is a deliverable. This records bounded historical results, not completion of PF-60 implementation.

**Verified — label convention:** In this record, **verified** means the cited repository source, commit or receipt supports the statement at that snapshot; historical test results are attributed to their executor/manager, not rerun by this author. **Gated** identifies an unmet prerequisite and its owner. **Planned** identifies intended behavior or future verification, not executed evidence. Table labels apply to every statement in their row; quotation attribution applies to the entire quoted requirement.

**Verified — product linkage:** The plan cites the exact heading **Product measurement**, excerpt “No commercial performance numbers have been supplied.” The current [product specification][spec] places that sentence under **Measurement targets**, within **Product measurement**. The [Responses allocation][responses-allocation] also records this heading distinction. This is context for the existing PF-60 initiative, not additional authorization.

## Task Node requirement and provenance

**Verified — frozen dispatch receipt:** Allocation digest `e567b0703720e99d03aed332239fed9ac73682f7f4ccbd3d1a6b98cf25b3b877`; claim `58fb24b3-5aef-4d60-8d4f-72075b51b1cb`; assigned runtime `gpt-6-astra`, effort `high`. The supplied brief was read first and verified using `shasum -a 256`: `d79390a2cf5e5685952f266c2832aa841f4a4e23058d7ef4e99e183fe56a0dc8`, matching the frozen assignment. Its private location is deliberately omitted.

**Verified — requirement attribution:** The following is the CLI-read task requirement supplied by the manager in that digest-verified brief, copied verbatim for task `task_b3e8506327fb906173fd68b2f642221b`. This worker did not independently query the live Task Node CLI or establish a later task state.

> Submit the complete PF-60 workstream record document containing the accounting contract with the five cost categories distinguished (measured tokens, estimated cost, billed cost, subscription allowance, unknowns), the four sequential sprints S01-S04 with scope and exit criteria, the synthetic fixture verification plan, and the gating/delivery contract. The status packet must accurately reflect reviewed state and state that human approval is pending; no evidence of implementation completion may be claimed.

## Status packet at the source snapshot

| State | Record and evidence | Meaning and limits |
| --- | --- | --- |
| **Verified** | [S01 archived completed][s01]; candidate `1c5978690` received at `0415a00dc3`; manager recorded 48 passing synthetic tests and approved defaults in [Accounting decision][defaults]. | Contract/fixture handoff accepted. No accounting runtime, live billing or release acceptance follows. |
| **Verified** | [September 14 04:45 UTC coordinator update][handoffs]: `accounting-product-resumption`, feed revision 33, coordinator revision 388. | Travis reopened accounting work. The September 13 pause and September 14 01:40 pause observation are historical for accounting. This is separate from collection enablement. |
| **Verified** | Direct Anthropic candidate `bc9b8b0d8` received at `e9d1345ee`; [manager receiving receipt][anthropic-receiving] records independent review03 with no findings and exact candidate hashes. | Accepted bounded direct-Anthropic dispatch increment. Role-reload and redirect findings/corrections and failed full-suite results remain preserved in the [worker receipt][anthropic-receipt]. |
| **Verified** | Original-evidence compact import `281ee4ec1` received at `81d0f90e7`; [receiving receipt][compact-receiving] records clean corrective review and 595 shared / 100 selected Core / 20 focused / 6 external passes. | Scoped native import/replay/retention proof, with overlapping selectors. No legacy-source acquisition or whole-S02 acceptance. |
| **Verified** | Original-contract native goldens `f4507cb50`, combined proof `855ab3382`; [checkpoint record][pause] and [golden receipt][goldens]. | Three native golden tests, 598 shared and 100 selected Core tests recorded. The S02 unchecked item requesting these goldens is stale; it does not undo the receiving record. Descriptive invoice/allowance/balance fields are outside the native DTO proof. |
| **Verified** | Responses HTTP `e47e41870` received at `d81bad635100498e5e8769a616533f0172b4946d`; [S02 Done ledger][s02] and [September 14 09:45 UTC update][handoffs]. | Independent review recorded zero findings; 385/385 worker focused, 634/634 combined shared and 63/63 receiving Core accounting tests. These are scoped results, not a full-Core/workspace pass. |
| **Verified** | [Responses receipt][responses-receipt], “Second manager disposition: corrected role premise.” | Manager corrected two frozen role cases to preserve the reserved built-in `openai` provider rule. Revised cases passed; earlier failures, tooling STOP and erroneous JUnit copies remain historical evidence. |
| **Verified** | [S02 front matter and Remaining][s02], [S03][s03], [S04][s04]. | S02 is `in_progress`; S03 and S04 are `draft`; collection **OFF**. Direct Anthropic and direct API-key Responses HTTP are received coverage increments; WS, Chat/Corbanu and auxiliary routes remain open. |
| **Gated** | Whole-S02 acceptance: remaining route/legacy acquisition coverage, unresolved vector limitations in [Responses receipt][responses-receipt], final evidence reconciliation and accepted exit handoff. Owner: accounting integrator. | Passing named tests does not prove every frozen vector. No S02 completion, S03 activation or unqualified functional handoff. |
| **Gated** | Human approval of this workstream record and runtime candidate: Travis has not filled the approval section below; [plan Human acceptance][plan] leaves runtime testing pending. | Historical S01 defaults approval stands. It does not approve this packet, runtime qualification or live collection. |

**Verified — source reconciliation:** The [plan][plan] and [S02][s02] retain older “next store,” two-path goldens and suspended-assignment prose. The dated receiving records above and current S02 front matter identify received increments; none authorizes a new source allocation. The [Responses receipt][responses-receipt] retains partial-vector caveats even after the role correction. This packet preserves those caveats and leaves their disposition with the integrator; no shared ledger was edited.

## Accounting contract

### Five distinct categories

**Verified — approved design contract:** The following distinctions come from the [contract][contract] and Travis's recorded [Accounting decision][defaults]. “Verified” here establishes the approved meaning, not implementation across every provider.

| State | Category | Meaning, provenance and separation |
| --- | --- | --- |
| **Verified** | **Measured tokens** | Provider-reported numeric fields with wire dialect, field presence, source reference and observation time. Preserve input, cache read, cache write, output, reasoning and total independently. A provider count is not a billing certificate. Preflight token estimates carry their own method/version and never replace measured fields. |
| **Verified** | **Estimated cost** | Exact token arithmetic against an identified immutable, prospectively approved price snapshot. Retain priced components, unknown components, currency, rates/unit and original price identity. Measured tokens can still yield only estimated money. Never overwrite historical estimates with today's catalog or add billed amounts to estimates. |
| **Verified** | **Billed cost** | Requires authoritative settlement/invoice line identity, currency, issuer-precision amount and explicit request allocation. Funding/top-ups are not request spend. No invoice ingestion or native bill-to-response join is established by the contract or native goldens; unavailable billed totals remain unknown. |
| **Verified** | **Subscription allowance** | The contract calls this **provider allowance**: a provider capacity/credit/window snapshot with source, unit and freshness. It is not USD cost, an invoice or Corbanu API money; do not infer cash value or per-task consumption from a percentage/reset window. Legacy Plan entitlements are excluded. |
| **Verified** | **Unknowns** | Explicit absence/ambiguity, not zero: `not_reported`, `interrupted`, `legacy_presence_lost`, `unattributed` or `conflict`. Preserve known subtotals and unknown-attempt counts. A full total is unavailable if a participating required component is unknown; missing request/lineage coverage remains separately visible. Explicit measured zero is valid. |

**Verified — separate account snapshot:** [Contract vocabulary][contract] keeps Corbanu API **balance**, **reserved** and **available** separate, preserving exact micro-USD and display strings with account/as-of provenance. These snapshots do not form a sixth spend estimate and do not convert allowance into cash. Balance deltas cannot allocate request spend; inconsistent returned representations are retained and flagged, not repaired locally.

### Identity, replay, aggregation and pricing

| State | Approved contract and constraints | Source |
| --- | --- | --- |
| **Verified** | One logical sampling `request_id` owns a retry chain; each actual send has its own durable `attempt_id` before dispatch and optional `retry_of`. Ownership uses native thread/turn identities. Optional provider IDs are scoped correlations, not deduplication authority. Preflight denial is a non-dispatch fact; interrupted or failed sends can consume usage. | [Contract request identity][contract] |
| **Verified** | Observations key by `(attempt_id, revision)` with original durable sequence/provenance. Identical duplicates are no-ops; conflicting payload/immutable identity fails visibly. Apply cumulative patches by replacement, not addition; omitted fields remain preserved. A crash after dispatch without a receipt stays unknown. | [Contract replay][contract] |
| **Verified** | Use native state and hierarchy; commit identity/observation/checkpoint atomically at the applicable boundary. Notification replay, inherited/forked history and context rollback create no new charge. Sum each owned attempt once across unique validated descendants, never parent inclusive subtotal plus child subtotals. Reject cycles/conflicting parents; unknown lineage cannot imply complete coverage. | [Contract][contract], [S02 handoff][s02-allocation] |
| **Verified** | Token counts are exact nonnegative integers; prices/money use exact decimal/rational arithmetic. Initial estimates are USD-only; no implicit FX. Sum before half-even rounding to six USD decimals and mark rounded/sub-micro amounts. Native balance precision and issuer billed precision remain separate. | [Approved defaults][defaults] |
| **Verified** | For supported inclusive-input dialects, disjoint noncached input = input − cache read − cache write; reasoning is already within output. Token-only USD estimate = `(noncached × input rate + read × read rate + write × write rate + output × output rate) / 1,000,000`. Missing split/rate leaves affected cost unknown; a zero bucket needs no rate. Native Anthropic's separately reported noncached input remains independently priceable. | [Contract][contract], [corrected S02 handoff][s02-allocation] |
| **Verified** | Price identity includes scope, exact model, currency, rates/unit, provenance and capture/effective times. Select using original dispatch time and half-open effective bounds. Approved provider-published/native catalog snapshots apply prospectively from observation/approval; no backdating, live price fetching or implied invoice ingestion. Unsupported fees/economics are not silently zero. | [Contract][contract], [approved defaults][defaults] |

### Retention and deletion

**Verified — approved policy:** [Defaults][defaults] retain numeric detail for 90 days and aggregates for 365 days, with minimal opaque replay tombstones for the 365-day replay horizon; older imports are rejected. User deletion removes attributable detail and aggregates, retaining only minimal replay records. The design requires deletion explanation and export opportunity; this is not evidence that the UI is implemented.

**Verified — conservative daily expiry:** Travis approved expiry at UTC-day-start + 365 days in the [retention handoff][retention]. A daily bucket can lose a newer contribution up to 24 hours early; it cannot retain one past 365 days. The rolling replay horizon and 90-day detail boundary are unchanged. Compact transfer, admission-floor advancement, read coverage and deletion must remain atomic/replay-safe; a later catalog must not reprice compacted history.

**Gated — retention coverage limits:** [Golden receipt][goldens] and [S02 Remaining][s02] do not establish legacy original-evidence acquisition, permanent deleted-owner revocation, physical database/WAL erasure or cross-database erasure. Anonymous fencing covers the approved horizon, not permanent identity revocation. Sub-day historical display awaits S03's exact query contract; old UTC-day aggregates cannot reconstruct arbitrary partial-day values.

## Four sequential sprints

**Verified — sequencing contract:** The [plan sprint map][plan] contains exactly one PF-60 sequence, S01 → S02 → S03 → S04, with parallel sprint limit 1. Under the [sprint process][sprint-process], predecessor acceptance/archive and a ready/in-progress, exactly allocated sprint precede implementation. This record grants no successor activation.

| Sprint / current state | Scope | Exit criteria and outstanding gate |
| --- | --- | --- |
| **Verified — PF-60-S01 completed/archived** | Accounting contract, native source mapping, raw synthetic inputs, literal expected totals/provenance and reviewed S02 handoff. | **Verified:** [S01][s01] records `1c5978690` / `0415a00dc3`, 48 combined fixture tests, reviewed corrections/handoff and Travis's v1 defaults approval. Its bounded contract/fixture exit is met; runtime is outside that completion. |
| **Verified — PF-60-S02 in_progress** | Idempotent native usage persistence/replay, attempt ownership, presence, immutable original prices, retention/import/deletion and allocated dispatch adapters. | **Planned:** Replay every fixture twice and after restart; persisted/reconstructed values exactly match approved expectations. Record failure/cancel/recovery, compatibility, final-tree checks and applicable functional evidence; receiving owner accepts a concrete handoff and archives S02. **Gated:** WS/Chat/auxiliary routes, legacy acquisition, remaining vectors and final exit evidence are open in [S02][s02]. |
| **Verified — PF-60-S03 draft** | **Planned:** Inspectable root/descendant and campaign totals, provider/model breakdown, estimate/billed distinctions, freshness, missing-price/backend states, custom ranges/grouping intervals. Explicitly show timezone, half-open bounds, coverage, oldest aggregate day and 90-day drill-down cutoff; never prorate compact days or fabricate hourly detail. | **Planned:** A user can explain each displayed total through constituent requests without reading storage; narrow-screen/mixed-provider/range/DST/deletion/reopen tests and actual-key evidence pass. **Gated:** S02 acceptance/archive, exact files/worktree/base/owner and frozen timezone/week/month/query semantics are unallocated in [S03][s03]. |
| **Verified — PF-60-S04 draft** | **Planned:** Final candidate qualification and handoff: three-provider parent/child synthetic journeys, one separately authorized live-provider check, cancellation, unknown price, retry duplication, restart and historical inspection. | **Planned:** All cost flows agree with goldens on one recorded binary; unknown and estimated values remain distinct. Complete independent functional evidence, applicable live-repository proof, named-human acceptance and finished documentation. **Gated:** Accepted/archived S03, exact binary/allocation, live-check authority and qualification artifacts are absent from [S04][s04]. |

## Synthetic fixture verification plan

**Verified — immutable fixture input:** [S01 fixtures][fixtures] SHA-256 is `96ba9416a8d0e69436fdd07c4ca6ddbb20cc30ece757b983eb1de2d708ca95dd`, rechecked for this record and matching the [golden receipt][goldens]. Counts, identities, invoice/account figures and prices are invented. Provider labels identify dialects, not actual usage, authenticated accounts or real quotes. The [hand calculations][fixture-guide] and literal expectations are distinct from the reference reducer.

| State | Fixture / action / observable expectation | Existing evidence and required limit |
| --- | --- | --- |
| **Verified** | Root plus two children: three logical requests, four dispatched attempts including retry; independent historical thread excluded. Known estimated USD `0.001084`, two unknown estimates, full estimate null. Separate billed known USD `0.000300`, full billed total null. | [S01 arithmetic][fixture-guide]; native family arithmetic in [goldens][goldens]. Native DTO tests do not prove billed/allowance/account ingestion. |
| **Verified** | Family tokens: input 420; cache read 90; cache write known 60 with one unknown; output known 60 with one unknown; reasoning known 8 with two unknowns; total known 400 with one unknown. Historical priced attempt remains USD `0.000110`; historical missing split retains known output USD `0.000040`, full cost unknown. | [S01 arithmetic][fixture-guide] and [native golden transcription][goldens]; do not add cache/reasoning subsets twice or substitute new prices. |
| **Planned** | Reverify duplicate/reordered observations, all retry-revision permutations, conflicting revisions, missing/reused provider IDs, partial Anthropic patches, explicit zero vs missing/null and failed/cancelled attempts. Expected: no double charge; conflicts rejected; consumed partial usage survives; unknowns remain explicit. | Preserve [S01 cases][fixture-guide] and [native golden proof][goldens]; new adapters need their own actual dispatch/presence vectors. |
| **Planned** | Run native partial checkpoint → close/reopen → replay twice → two complete-state reopens; compare identities, source positions, bindings, snapshots, owner/day values and complete table state. Add applicable actual process interruption around admission/receipt/commit boundaries. | [Goldens][goldens] already records on-disk reopen proof; it is not universal process-kill or power-loss proof. [Compact receiving][compact-receiving] supports only its bounded process/failure cases. |
| **Planned** | Exercise exact 90/365-day boundaries, compact late imports, repeated sweeps, stale prices, two owners sharing snapshots, deletion while OFF, rollback at each stage and genuine writer contention with both deterministic orders. Expected: exact values, no orphan copies or replay resurrection within the horizon, unrelated owner preserved. | Reuse accepted [retention contract][retention] and [compact evidence][compact-receiving]; preserve failed race-diversity and metadata attempts. Do not infer legacy acquisition from synthetic bundles. |
| **Planned** | Invalid counts/subsets/totals, price gaps/overlaps/model/currency/unit mismatch, lineage/retry cycles and malformed money must reject or expose the contract's unknown state. Change allowance/account snapshots without changing requests: request totals stay identical. | [S01 negative cases][fixture-guide]; changing future prices never changes a prior estimate. |
| **Gated** | Complete route matrix and user display/recovery execution. | WS-only/prewarm work, Chat/Corbanu, auxiliary routes and unsupported subscription economics lack full S02 evidence. [Responses allocation][responses-allocation] says HTTP fallback accounts only its HTTP segment; preceding WS usage is not fully covered. S03/S04 execution must expose that coverage gap. |

**Planned — safe execution and receipts:** Before any test campaign, follow [safe automated tests][test-isolation]. Use only the checkout's guarded `just test` for ordinary Rust tests, synthetic isolated profiles and nonzero discovered selectors. Stop successors/retries on a native credential prompt or live-profile read and retain the contaminated attempt. Future receipts must identify candidate/input digests, command, UTC times, exit/counts, expected/actual results, raw artifact location and limitations. Preserve failed attempts and separate corrected replays; overlapping selectors are not unique-test totals.

## Gating and delivery contract

| State | Gate / owner | Required delivery or exact blocker |
| --- | --- | --- |
| **Verified** | Approved policy — Travis | S01 vocabulary/defaults and conservative UTC-day expiry have recorded approval in [defaults][defaults] and [retention][retention]. Those decisions do not need repeating. |
| **Gated** | Next S02 allocation — Fable/accounting integrator | Reconcile consumed status text and unresolved frozen-vector dispositions; select one bounded remaining route, matching plan/sprint coordinates, exact writable scope, budget and receiving checks. This design worker has no implementation mandate. |
| **Gated** | Functional handoff — integrator, independent designer/executor/reviewer | Follow [code-blind functional policy][functional]: frozen intent-only design, separate code-blind executor, enforced filesystem/tool/process-IPC/network separation, actual negative probes and positive package/PTY controls, isolated state/mediated inference, case-to-evidence mapping and independent evidence check. Missing enforcement or unresolved mandatory cases blocks applicable handoff. The present document is internal status research; no changed interactive path is being qualified. |
| **Gated** | Final runtime proof — S03/S04 owners | Exact packaged candidate, actual PTY keys (prompt and Enter separately), success/cancel-failure/recovery/resume, representative profiles and visible unknown/coverage behavior remain pending in [plan][plan]. Increment-specific internal-only N/A, such as [goldens][goldens], cannot waive later functional gates. |
| **Gated** | Security qualification — existing security owner | [Coordinator handoffs][handoffs] retain the separate security resumption/isolated-executor condition; [checkpoint][pause] leaves native/all-platform and independent acceptance open. Accounting receipts are not completion of PF-27, PF-35 or broader security work. No duplicate security assignment or acceptance is created here. |
| **Gated** | Live collection / financial and external actions — separately authorized owners | Collection remains **OFF**. This packet grants no live provider spending, invoice ingestion, Task Node posting, task acceptance, reward action, Slack message, push or release. S04's live check requires its own scoped authority and safe environment. |
| **Gated** | Release readiness — release owner and Travis | Target release/date and release record remain unresolved in [plan][plan]; final applicable tests, actual TUI flows, live-repository evidence, named-human acceptance and due benchmarks are pending. Apply [root release policy][policy], including its explicit human release-authority rule; this task contains no release instruction. |
| **Planned** | Live-repository and documentation delivery — S04/release owners | Resolve exact TensorCash and Isometric Game inputs/base commits before disposable-worktree qualification, document feature applicability, and satisfy both repositories for a release suite under [policy][policy]. Publish finished user documentation only for verified candidate behavior; keep this unfinished workstream record in research. |
| **Planned** | Local record delivery — this worker, then Fable | Commit only this document; return commit hash, document path and a one-paragraph summary. Fable reviews/reconciles the packet and obtains the pending human decision. A local commit/RETURN is not Task Node submission, receiving approval or implementation completion. |

## Human review and approval

**Gated — human approval pending:** Reserved for Travis to fill. Neither the author nor an agent review supplies this approval. Historical S01 policy approval is recorded above; runtime and this record's approval remain separate.

**Planned — Travis's review fields (intentionally blank):**

- Reviewed document commit / candidate:
- Decision (approve / revise / hold):
- Accepted scope and explicit limitations:
- Remaining prerequisites / requested changes:
- Name:
- Date:
- Approval reference:

## Record verification

**Verified — author checks:** At source snapshot `1fdb3fc1d85fdfaa041577146da0f0de0c1c3704`, brief and fixture SHA-256 checks matched; received Anthropic `e9d1345ee`, goldens `f4507cb50` and Responses `d81bad635` are ancestors. `PYTHONDONTWRITEBYTECODE=1 python3 docs/plans/check.py` exited 0 (3 active / 3 slots); `PYTHONDONTWRITEBYTECODE=1 python3 docs/sprints/check.py` exited 0 (116 current / 126 archived). These are documentation/lifecycle checks, not runtime qualification. Historical receipt counts belong to their own trees.

**Verified — document validation:** Author read-only checks matched the quoted requirement to the frozen brief, resolved all 24 repository-relative reference links and their supplied heading anchors, confirmed the required categories/approval section and rejected private absolute paths. These checks validate the document's structure and attribution, not the truth of historical runtime results.

**Verified — evidence boundary:** This worker ran no Rust, fixture, provider, TUI, live-repository or security qualification tests. The change adds only an internal research record; code-blind functional execution is not applicable to this document-only change. Applicable product execution remains gated above, and this self-check is not an independent review or human acceptance.

[policy]: ../../../AGENTS.md
[spec]: ../../corbanu-product-spec.md#measurement-targets
[plan]: ../../plans/active/portfolio-agent-cost-accounting.md
[sprint-process]: ../../sprints/index.md
[contract]: ../agent-cost-accounting/contract.md
[defaults]: ../../plans/workstream-manager-handoff-2026-09-11.md#accounting-decision
[retention]: ../agent-cost-accounting/retention-design-handoff.md
[s02-allocation]: ../agent-cost-accounting/s02-allocation.md
[responses-allocation]: ../agent-cost-accounting/responses-dispatch-allocation.md
[s01]: ../../sprints/archive/portfolio-agent-cost-accounting/pf-60-s01-accounting-contract-and-golden-fixtures.md
[s02]: ../../sprints/current/portfolio-agent-cost-accounting/pf-60-s02-idempotent-usage-persistence-and-replay.md
[s03]: ../../sprints/current/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md
[s04]: ../../sprints/current/portfolio-agent-cost-accounting/pf-60-s04-cost-accounting-acceptance-and-handoff.md
[handoffs]: ../../../qa/initiative-control/status-display/current-handoffs.md
[pause]: ../../plans/management-pause-2026-09-13.md
[anthropic-receiving]: ../../../qa/initiative-control/status-display/combined-native-20260912-1911.md
[anthropic-receipt]: ../../../qa/portfolio/agent-cost-accounting/pf-60-s02/anthropic-dispatch-increment.md
[responses-receipt]: ../../../qa/portfolio/agent-cost-accounting/pf-60-s02/responses-dispatch-increment.md
[compact-receiving]: ../../../qa/portfolio/agent-cost-accounting/pf-60-s02/compact-late-import-receiving.md
[goldens]: ../../../qa/portfolio/agent-cost-accounting/pf-60-s02/original-contract-native-golden.md
[fixtures]: ../../../qa/portfolio/agent-cost-accounting/pf-60-s01/fixtures.json
[fixture-guide]: ../../../qa/portfolio/agent-cost-accounting/pf-60-s01/README.md
[test-isolation]: ../../development/test-isolation.md
[functional]: ../../../qa/code-blind-functional/README.md
