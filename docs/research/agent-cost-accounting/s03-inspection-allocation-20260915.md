# PF-60-S03 allocation and first executable unit — proposal

Action: acct-s03-design-01  
Allocation digest: 8cce1f3297dba4f4339b5ac9b76afe7d6a9a2a93bd89caef729ba1e1d5db6d06  
Claim: 5812e4b8-3137-419f-aeee-7ffdee7f5ce1  
Designer: gpt-6-astra / high  
Date: 2026-09-15  
Brief: /private/tmp/fmgr.Q1SIYZ/briefs/acct-s03-design-01.json  
Verified brief SHA-256: d01c38daaa0f3c8780f78a166e9f48aeb09ff56688008a1095be462aa0ff4fc7  
Inspected base and HEAD: c782a340f8006938352fd2786b0ccd14a356eea0  
Inspected worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911

## Recommendation and authority

Allocate **S03-A: read-only explanation of one native thread's recorded requests for one UTC day**. Its vertical path is `/usage requests [YYYY-MM-DD]` -> non-mutating native-state query -> recorded subtotal -> logical requests -> physical attempts -> measured components and original price evidence. Default date is today in UTC. Collection stays OFF. This is a useful first increment, not completion of the run/campaign/range contract.

The present one-file S03 mandate is insufficient. A storage API change is required even for this narrow unit; no migration or persisted-data change is required for retained raw detail. Full S03 also has information-loss limits that a new display cannot repair. Do not dispatch a worker with only usage.rs authorized.

This is routine design preparation for the existing PF-60 product initiative, not implementation authority, independent code-blind design, manager approval or qualification. Product citation: **Measurement targets**, docs/corbanu-product-spec.md: “No commercial performance numbers have been supplied. The following metrics must be instrumented, with targets set through the decision rights defined above.” The active plan is docs/plans/active/portfolio-agent-cost-accounting.md. Its older “Product measurement” citation should be reconciled to the actual heading without changing approved outcomes.

Read inputs: S03 and S04 drafts; S02 Done, Remaining, Verification and closure determinations; accepted contract; September 15 closure assessment; received Chat, Responses HTTP and WS allocations. The contract's acceptance update supersedes its historical proposed-policy language. At this base S02 is still in_progress and not archived; S03 and S04 remain draft. Substantive receipt of S02 does not itself satisfy the executable-dependency rule. Fable must finish the recorded bounded closure and archive S02 before marking S03 ready. Nothing in this proposal checks those boxes.

Repository inspection was read-only. No builds, tests, checkers, provider calls, credential reads, subagents, commits or pushes occurred. The only authored file is this private proposal. Initial HEAD and the source-review checkpoint matched the assigned base; git status was clean. During final artifact hashing the shared checkout advanced to 6f3c2d7055da760d001e20e85719979f45814a9a: only docs/sprints/current/initiative-delivery-control/pf-80-s01-delivery-control.md changed (two additions/two deletions), recording unrelated qualification fixes. Its allocation coordinates/scope and all accounting source/records were unchanged. The final observed worktree remained clean; this worker made no commit. One exploratory constructor filename was missing; discovery located the actual singular constructor.rs before allocating it. Every existing source/policy path cited below was located; proposed checkout/branch names are explicitly future coordinates, not existence claims.

## 1. Actual display and storage boundaries

### Existing TUI

- codex-rs/tui/src/chatwidget/usage.rs currently builds a Usage selection menu and the earned-reset lifecycle. “Show usage” sends OpenTokenActivity. It does not query the S02 ledger or explain request costs. Its test-only Corbanu Plan item is not a production accounting entry.
- codex-rs/tui/src/chatwidget/tokens.rs manages account token activity asynchronously. codex-rs/tui/src/chatwidget/tokens/chart.rs renders an account-level last-12-months chart with daily, weekly and cumulative modes. Those buckets are not S02 request evidence.
- codex-rs/tui/src/app/background_requests.rs fetches account/usage/read; codex-rs/tui/src/app/event_dispatch.rs routes its result. codex-rs/tui/src/app.rs already holds optional StateDbHandle and AppServerTarget. The local database must never be mistaken for a remote app-server's accounting data.
- codex-rs/tui/src/chatwidget/slash_dispatch.rs gates existing `/usage` and account chart arguments through ensure_usage_command_available, which requires backend auth or linked Plan state. A direct-provider user cannot depend on that entry. Local requests need a separate argument branch before that account-only check, without invoking Plan/key lookups.
- codex-rs/tui/src/token_usage.rs contains signed native counters, lacks cache-write presence and formats a blended noncached-input-plus-output number. Neither it nor account chart totals can be relabeled as ledger totals. Existing context/account metrics retain their meanings.

### What S02 actually stores

The installed schema is codex-rs/state/accounting_migrations/0001_usage.sql, exposed through codex-rs/state/src/runtime/accounting_store.rs and the re-export in codex-rs/state/src/lib.rs. The old codex-rs/state/migrations/0041_provider_request_cache_usage.sql is throttle/preflight state, not the request ledger.

| Table | Identity and retained meaning |
| --- | --- |
| draft_accounting_attempts | attempt_id primary key, indexed request_id, typed Attempt JSON. Attempt contains logical request_id, thread_id, turn, retry_of, provider, exact model, opaque scope UUID, arithmetic Dialect and dispatched_at_ms. It is durable admission intent, not proof of a successful send or billed charge. |
| draft_accounting_observations | Primary key (attempt_id, revision), unique (source, sequence), typed presence patch. Ordered cumulative revisions replace fields; they are not additional requests. |
| draft_accounting_price_snapshots | Immutable snapshot_id and typed Snapshot JSON. |
| draft_accounting_price_bindings | One row per attempt, nullable snapshot_id. A binding row with NULL means originally unpriced. A missing binding row in an installed admitted record is not the same state. |
| draft_accounting_estimates | Primary key (attempt_id, evidence); evidence is canonical observation serialization. Multiple versions belong to one attempt and cannot all be summed. |
| draft_accounting_contributions | One current reference per attempt, thread_id, utc_day and evidence FK to an estimate. Used for current daily sums and freshness checks. |
| draft_accounting_compact_days | One (thread_id, utc_day) payload containing numeric populations and amount sums. No constituent request/attempt/turn/provider/model identities. |
| draft_accounting_compact_snapshots | Set of snapshot IDs referenced by a compact thread/day. No amount allocation to each snapshot, request or provider. |
| draft_accounting_tombstones | Anonymous attempt UUID and replay expiry only; cannot reconstruct ownership or deleted history. |
| draft_accounting_retention_checkpoint | Singleton completed_as_of_ms and admission_active; maintenance state, not a complete-collection interval. |

codex-rs/state/src/runtime/accounting_types.rs preserves Presence::Missing, Null and Number (including explicit zero). Reduced Usage has seven Option<i64> metrics: input, noncached, read, write, output, reasoning, total. NativeAnthropic and Inclusive are arithmetic dialects; Inclusive does not distinguish Responses HTTP, WS and Chat.

codex-rs/state/src/runtime/accounting_pricing.rs stores Snapshot id, provider/model/scope, USD currency, PerMillionTokens unit, four optional exact rates, source_reference UUID, source_kind, observed_at_ms, approved_at_ms and half-open effective interval. Snapshot timing describes the price source, not provider usage observation time. ObservationQuote contains Attempt, observations, reduced usage, optional original snapshot, four disjoint BucketQuotes, known_subtotal, optional all_buckets_priced and DisplayAmount. BucketQuote distinguishes Priced, MissingUsage and MissingRate. Known zero buckets need no rate. Money uses exact u128/scale arithmetic; six-place display is half-even and explicitly records rounding and nonzero sub-micro amounts.

codex-rs/state/src/runtime/accounting_lifecycle.rs stores DayTotals: seven Metric values, each with a known token SUM and an unknown ATTEMPT COUNT; known_usd, unknown_estimates and attempts. unknown_estimates counts attempts with incomplete token estimates, not missing dollar amounts. codex-rs/state/src/runtime/accounting_compact_values.rs preserves these exact populations and amounts in version-1 compact payloads. It does not preserve cost-bucket breakdown, logical-request count, price-missing count, reasons for unknowns or provider/model dimensions.

### Honest capability/gap matrix

| Required explanation | Already supported | Gap and required action |
| --- | --- | --- |
| Retained request -> attempts -> numeric evidence -> price | Raw identities, patches, immutable original price/null and validated estimate versions exist. | No normal-library inspect/list API. Add read-only facade and typed snapshot, reusing validation/reduction. Storage API change, no DDL. |
| Read without activating collection or changing state | read_day itself is non-mutating. | Obtaining AccountingStore through open installs if absent and runs maintain. Add an associated inspection entry taking existing StateRuntime, not open. A TUI read must never install, refresh estimates, prune or repair. |
| Consistent totals and drill-down | Private authority, latest-quote and contribution validation exist. | Expose one read transaction's owned rows and totals together. Never one transaction per row or separate latest-total/detail reads. |
| Wall-clock freshness | RetentionCoverage exposes checkpoint, raw cutoff, aggregate floor and oldest retained day. | read_retained_on_connection returns NeedsMaintenance whenever requested as_of exceeds checkpoint. Calling read_day(now) will normally be stale; opening to fix it mutates state. Explicitly show last maintained snapshot and wall-clock read time, suppress expired detail, and label maintenance lag. No fabricated observation timestamp. |
| Raw provider/model breakdown | Attempt retains provider/model/scope. | Query/grouping and UI changes only for retained rows. Do not use current model configuration to label history. |
| Provider/model breakdown after compaction | Daily numeric sums and a snapshot-ID set survive. | Impossible to reconstruct attribution. A future compact-format/storage change is needed if this dimension is required beyond raw retention. Existing compact days cannot be backfilled from snapshot sets. |
| Explain every historical total by requests | Raw request identities survive within detail retention. | After compaction, constituent identities are deliberately gone. No display change can restore them. Respect the accepted 90-day cutoff and explicitly distinguish aggregate-only history. Requiring request drill-down for all 365 days would require a product retention decision and storage change, not an S03 patch. |
| Root/descendants | Native thread_spawn_edges and list APIs in codex-rs/state/src/runtime/threads.rs exist; unique owner IDs can prevent double count. | Need a validated snapshot query for membership/cycles/unknown lineage. Edges are removed on deletion; there is no immutable historical accounting lineage. Full historical membership may require persisted lineage under an approved deletion contract. Never add inclusive parent subtotal to child totals. |
| Campaign membership | No campaign membership in accounting schema. | Must identify and authorize the actual membership authority and historical semantics. “All descendants” is not automatically a campaign. A new durable mapping may be needed; do not invent it in the UI. |
| Arbitrary ranges/intervals | Raw dispatch timestamps and compact UTC-day sums. | Query work for raw half-open ranges; whole-day compact composition. Hours/partial days and local-time/DST bucket edges cannot split compact UTC days. UTC hour/day, Monday-based UTC week and UTC calendar month are proposed alignment defaults for manager freeze; no silent rounding/proration. |
| Complete run cost or known collection coverage | Known and unknown populations among recorded attempts. | No persisted coverage intervals or excluded-dispatch registry. Today’s Disabled flag cannot establish history. Always label recorded-only coverage unknown. A positive completeness claim needs new collection/provenance storage and collector work. |
| Unknown reasons / terminal status / literal route | Presence, revision/source, admission time, arithmetic dialect, opaque scope. | No general stored interrupted/cancelled/success status, usage observation wall clock, literal wire/endpoint, human account name or excluded-operation provenance. Say unavailable; do not infer status or “Chat” from missing write tokens. Richer claims require persisted evidence changes. |
| Billed cost / gateway economics | Prospective token estimates only. | No settlement rows/request bill join. Billed is unavailable; balances, allowances and current prices cannot fill it. Separate authorized evidence/storage work is necessary for billed attribution. |

**Conclusion:** S03-A requires touching the state crate's reader API. The full draft cannot be delivered as a display-only change. Some missing data can be honestly marked unavailable under the existing retention/unknown contract; providing the missing dimensions affirmatively requires separately accepted storage/collection work. This proposal does not remove any draft Remaining item or move an unimplemented display contract into S04 acceptance.

## 2. User-visible truth contract

Use words as well as styling. Unknown state must survive monochrome and narrow widths.

Example header:

    Recorded requests — this thread only
    2026-09-15 00:00 UTC <= admission time < 2026-09-16 00:00 UTC
    Known estimated token cost: $0.001210 + unknown costs
    Full recorded estimate: unavailable (1 of 2 attempts incomplete)
    Collection coverage: unknown; recorded attempts only. Descendants excluded.
    Billed cost: unavailable — no settlement evidence
    Read at: ... UTC; store checkpoint: ... UTC

- A complete recorded token estimate is labeled `Estimated token cost for recorded attempts: $...`. It never becomes “run spend,” “billed,” “all work” or “current account balance.” Keep coverage-unknown visible even when every retained bucket is priced.
- Partial money always shows the known subtotal AND unknown participation beside it: `Known estimated token cost: $... + unknown costs`; full amount unavailable. Never `$...` alone or a guessed upper/lower total. The unknown attempt count does not estimate the missing amount.
- A measured component uses `Cache write: unknown — no retained numeric evidence`. Use `Input: 100 known + unknown in 1 attempt` for aggregated metrics. Explicit measured zero is `0 (reported)` where applicable. Derived values must be labeled derived; total/reasoning are not independent billable buckets.
- Unpriced attempt: `Price: unavailable — no dispatch-time price snapshot`; preserve its measured tokens. Known nonzero buckets with no rate: `Cost: unknown — rate unavailable`. Missing usage takes precedence for that bucket; the separate price badge still discloses absent snapshot. A snapshot with one absent rate says `Rate unavailable for cache read`, not “no snapshot.”
- Inclusive input with unknown cache-write makes derived noncached input unknown. Price known cache-read and output buckets if possible; do not price input minus cache-read as though cache-write were zero. Missing cache-write in Chat can remain unknown permanently. Since wire is not persisted, use the evidence-based generic reason, not a guessed provider-specific explanation.
- If no positive or complete priced amount is available, lead with `Estimated token cost: unknown`; optionally show `Known priced components: $0.000000; other components unknown`. Do not visually headline zero. An explicitly all-zero measured attempt can have zero token estimate without a price, but retains `Price snapshot: unavailable` and coverage/billed limitations.
- Use the existing half-even DisplayAmount. Append `(rounded)` whenever rounded; for a nonzero value below one micro-USD also show `nonzero, less than $0.000001` (including a rounded-up $0.000001). Detail exposes canonical exact decimal strings so users can reconcile sums before rounding. Do not add rounded per-row displays or convert through floats.
- Attempt detail shows full request and attempt UUIDs, turn, thread, provider/model, admission timestamp, retry predecessor, all seven measured/derived metrics, four disjoint priced/unknown buckets, price ID/source kind/reference, four rates/unit and effective interval. Display opaque scope as an identifier, not a credential/account name. No URLs, token fingerprints, prompt text or bodies are queried.
- Show `Completion/billing status: not recorded`. An observation prefix is evidence of numeric usage, not terminal completion. For a one-day slice, `Only attempts in this day are included; other attempts of this request may be outside the range` prevents a logical-request subtotal being mistaken for its lifetime cost.
- No schema: `Request accounting unavailable — no ledger installed. Collection remains off.` No auto-setup action. Installed but no rows: `No recorded requests in this range. Collection coverage unknown.` Neither state shows $0 spend.
- Remote target: `Request accounting unavailable for this remote connection`; do not fall back to the local profile. Missing thread/state has its own unavailable explanation. Failures expose stable bounded messages and `Retry`/`Close`; never raw SQL, serialized payloads or paths from error chains.
- Stale contribution/estimate: `Recorded totals unavailable — stored contributions need refresh`; no UI repair or guessed total. Ordinary checkpoint lag: `Snapshot is not current; newer activity is unverified`; Retry rereads only and may truthfully return the same checkpoint.
- Outside raw detail, compact-only or mixed raw/compact day: S03-A shows `Request detail unavailable for this whole UTC day` and the actual retention cutoff. It suppresses the aggregate amount in this detail-only unit rather than displaying an amount whose constituent requests it cannot show. Later aggregate UI must label aggregate-only history and preserve this limitation.

## 3. Proposed S03 allocation coordinates

These values are a proposal for manager freeze, not a created worktree, running worker or granted lease.

| Field | Proposed value |
| --- | --- |
| owner | Astra High accounting inspection worker; Fable receives/integrates, Travis remains product acceptance authority |
| parallel_lane | accounting-pf60-s03-inspection |
| worktree | /Volumes/CorbanuDrive/Corbanu/worktrees/acct-inspect-20260915 (proposed, not created) |
| branch | bootstrap/acct-inspect-20260915 (proposed, matches recent accounting convention) |
| base_commit | c782a340f8006938352fd2786b0ccd14a356eea0 is the inspected design base; replace with the exact post-closure/post-reservation receiving SHA before launch, in both plan and sprint |
| write_scope | Exactly the 21 existing files in section 5, comma-separated in front matter; no prefixes/globs |
| integration_gate | Fable receives onto integrate/management-workstreams-20260911 after S02 archival and PF-83 overlap handoff; audits literal scope/size, receives independent review and final state/TaskNode/TUI checks, then exact-package true-TUI plus independent code-blind execution/evidence. No S03 completion or human-ready claim until remaining S03 contracts and applicable gates are met. |

Manager preparation: reconcile plan's stale current-scope prose and sprint map; preserve S02 deferred coverage/history; archive S02 under its authorized bounded closure; replace S03's one-file boundary and retain its remaining root/campaign/range obligations; update both exact coordinates to an actual clean checkout; grant an exclusive build target for this lane; run docs/plans/check.py and docs/sprints/check.py. Those checks were not run here. S03 cannot honestly be ready at the design base while S02 remains in progress and PF-83 overlaps.

### Reserved-lane overlap

The proposed worker scope directly intersects PF-83-S01 at codex-rs/tui/src/app/event_dispatch.rs, codex-rs/tui/src/app_event.rs and codex-rs/tui/src/app/tests.rs. Its current reservation is recorded in docs/sprints/current/p0-security-levels/pf-83-s01-permission-confirmation.md. Different line ranges do not make those scopes disjoint. Fable and the existing security owner must receive/freeze the security candidate and record a sequential reservation handoff, or otherwise reallocate scopes under the process before source dispatch. Do not silently remove reservations or start a duplicate security worker.

PF-27-S04 is draft after returning its reservation, not resumed; its broker/protected-state/vault/manifests remain excluded. PF-80-S01 reserves coordinator scripts/research/QA, not these code files. Core, API adapters, proxy, TaskNode-session source, auth, wallet, native deletion, migrations, manifests, locks and BUILD files are outside this unit. The state/TaskNode combined receiving gate is still owed. Recheck actual live allocations/diffs at launch; these records are not evidence other worktrees are idle.

## 4. First executable unit contract — S03-A

### Ordered work

1. Add a typed read-only `inspect_day` entry through the existing accounting facade taking an existing StateRuntime, explicit native owner, UTC day and read time. Proposed symbol names are not existing API claims. It must not call AccountingStore::open, maintain, import, persist_current, refresh_contribution or any writer. No new DB initialization for missing state.
2. In one read transaction distinguish wholly absent installation from partial/unversioned/newer/checksum-mismatched schema; validate using existing rules. Absent is a typed unavailable state; corrupt or incompatible data is an error, not an empty day. Require the selected native thread to exist; never borrow another owner on failure.
3. Validate time/day and read checkpoint from that transaction. Reject backward read time. For a valid active ledger, evaluate the retained-day consistency at its recorded checkpoint, not by pretending wall clock equals it. Carry both timestamps and actual RetentionCoverage. Do not alter the old read_day semantics. A requested UTC day that touches data older than the 90-day wall-clock detail cutoff is unavailable as a whole-day detail slice, even if overdue maintenance left raw rows physically present. Never extend retention by reading stale raw data. The aggregate 365-day floor and oldest-recorded-day metadata remain separately labeled.
4. Reject compact participation in the requested day for S03-A, even if other raw attempts remain. Expose the reason without a partial-day total. Within a supported raw-only day, validate every selected attempt, source/revision identity, immutable original binding and all retained estimate versions through existing private authority/latest-quote machinery. Missing binding is corruption; NULL binding is unpriced. Match current contribution evidence to the actual observation sequence. If contributions are stale, return needs-refresh without repairing them.
5. Filter by admission timestamp using `[day_start, day_end)`, never observation arrival time. Reduce each physical attempt once from its latest cumulative evidence and original snapshot/null. Group by logical request_id only within owner and selected day; validate ownership/retry consistency, keep every retry. Derive the exact day subtotal from these rows and compare the complete DayTotals object with the validated retained-day result. No summing all estimate versions, no native TokenCount inference, no repricing.
6. Return an immutable view snapshot with typed rows, request groups, exact totals/displays, read/checkpoint times and explicit recorded-only coverage. Keep arithmetic in state. Reuse Decimal serialization for canonical exact strings; expose only the existing formatter as needed, not a float arithmetic API. No second UI money reducer.
7. Bound the initial reader: at most 10,000 retained attempt rows considered for validation, 512 selected attempts, 4,096 observations per selected attempt and 4 MiB projected result. Count/check before unbounded materialization; overflow/limit returns a typed `Range too large for this inspector` with no amount or truncated result. SQLx work runs asynchronously with a 15-second UI timeout. The initial all-row ownership validation is inherited, not a claim of indexed scalable history search. If this bound or implementation cannot be met without new schema/indexes, STOP for a separate query allocation. Later scalable range work remains on S03.
8. Route `/usage requests` and `/usage requests YYYY-MM-DD` before the account-auth guard, strictly parse a single ISO UTC date, reject invalid dates/trailing tokens without state/network access. Preserve `/usage`, daily/weekly/cumulative, reset operations and their auth predicates. Append “Recorded requests” after existing menu actions; first two positions stay stable. Account help text gains the new argument form. Direct users can invoke the argument without backend/Plan auth.
9. Fetch only for AppServerTarget::Embedded using its existing optional StateDbHandle and the currently displayed native thread. Remote, missing-state and no-thread cases are explicit unavailable states. Open a cancelable loading view. Correlate results by a fresh per-view UUID/generation plus thread and day; close, refresh, account/profile transition, thread switch or widget clear invalidate old results. A response for an older request must neither reopen the popup nor replace a newer snapshot.
10. Show summary -> logical request -> attempts -> component/price details using existing selection views and wrapped text. List scrolling must make every admitted row and every required detail field reachable at narrow sizes; do not put long essential fields solely in a clipped subtitle. Use small pages/back controls if necessary. Preserve the immutable snapshot across navigation. Refresh explicitly creates a new snapshot/generation; Esc/Back restores the prior page or closes the root. Clear retained view data on close/profile/thread invalidation; do not export this packet into model context or persisted chat history.
11. Add isolated state, native-library, UI and event-routing cases below; accept only intended inline/newly changed existing snapshots. Format/fix before final tests. Return candidate/diff hashes, literal manifest, size, case map, raw outputs, failures and limits to Fable. Manager retains the action return and records full receipt artifacts in a separately allocated evidence destination; this proposal does not guess a nonexistent S03 QA file or grant a directory-wide write.

### Scope kept OFF/excluded

All AccountingMode defaults and all collection predicates remain unchanged. Reading installed history is allowed; installing a ledger or admitting a request is not. No public activation switch, provider request, price refresh, wallet/account query, backfill/import, retention sweep, deletion repair, prompt capture, balance calculation or billing claim is added. No inferred zero for absent usage or missing collection. Remote protocol, subtree/campaign queries, arbitrary start/end controls and interval switching are later bounded S03 units, not hidden tasks in this one.

The first unit deliberately does not satisfy all S03 Remaining items. Keep them unchecked with concrete successors: (B) validated subtree/range/raw-compact query and membership contract; (C) root/descendant/provider/model/custom-range/interval UI and retention limitations; (D) any separately authorized storage dimensions needed for campaign/historical completeness. S04 remains integrated candidate acceptance, including its authorized-live-check prerequisite; it must not become a dumping ground for unfinished S03 implementation.

## 5. Exact worker file list and size

All 21 targets below exist at the inspected base. No new guessed source filename, new module registration, dependency or migration is allocated. Existing sibling test files are reused; new UI cases use inline insta snapshots so they do not create unlisted snapshot files.

| # | Existing repository-relative path | Per-file purpose | Estimated additions + deletions / non-test |
| --- | --- | --- | --- |
| 1 | codex-rs/state/src/runtime/accounting_store.rs | Non-installing public inspect entry, result types and transaction/schema states. | 150 / 150 |
| 2 | codex-rs/state/src/runtime/accounting_estimates.rs | Narrow Journal-access bridge to validated current original-bound quotes; reject missing binding. | 45 / 45 |
| 3 | codex-rs/state/src/runtime/accounting_lifecycle.rs | Owned raw-day snapshot, grouping/reconciliation, cutoff and compact refusal using existing arithmetic. | 170 / 170 |
| 4 | codex-rs/state/src/runtime/accounting_pricing.rs | Expose existing exact display formatter only as needed; no rate or arithmetic policy change. | 15 / 15 |
| 5 | codex-rs/state/src/runtime/accounting_store_tests.rs | Ten private schema/freshness/corruption/transaction regressions using existing fixtures. | 310 / 0 |
| 6 | codex-rs/state/tests/accounting_store.rs | Six normal-library disk/reopen/read-only/identity/golden/bounds tests. | 300 / 0 |
| 7 | codex-rs/tui/src/chatwidget/usage.rs | Append local inspection action and small open-entry glue; preserve reset lifecycle. | 25 / 25 |
| 8 | codex-rs/tui/src/chatwidget/tokens.rs | Local immutable inspector state, summary/detail rendering and navigation alongside existing usage lifecycle. | 300 / 300 |
| 9 | codex-rs/tui/src/chatwidget/tokens_tests.rs | Eight formatting/presentation/retention/rounding snapshots and cases. | 260 / 0 |
| 10 | codex-rs/tui/src/chatwidget.rs | One optional inspector state field and narrow type visibility only. | 8 / 8 |
| 11 | codex-rs/tui/src/chatwidget/constructor.rs | Initialize the new optional field in the actual constructor. | 2 / 2 |
| 12 | codex-rs/tui/src/chatwidget/slash_dispatch.rs | Requests/date argument before account guard; updated argument help. | 35 / 35 |
| 13 | codex-rs/tui/src/chatwidget/tests/usage.rs | Two navigation/menu tests and adjust disabled-reset navigation expectations without weakening reset assertions. | 110 / 0 |
| 14 | codex-rs/tui/src/chatwidget/tests/slash_commands.rs | Two direct-route/date cases; preserve account command regressions. | 95 / 0 |
| 15 | codex-rs/tui/src/app_event.rs | Local inspector open/load/navigation/cancel events carrying correlation and safe typed results. | 45 / 45 |
| 16 | codex-rs/tui/src/app/event_dispatch.rs | Thin embedded-only read task/result routing and current-thread guard; no new accounting engine here. | 95 / 95 |
| 17 | codex-rs/tui/src/app/tests.rs | Six actual event/native-store integration cases, including remote mismatch and stale response protection. | 300 / 0 |
| 18 | codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__usage_command_menu.snap | Existing menu snapshot gains appended action. | 12 / 0 |
| 19 | codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__usage_command_menu_without_resets.snap | Appended action while reset remains disabled. | 12 / 0 |
| 20 | codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__usage_command_menu_before_reset_refresh.snap | Existing loading/unknown reset state with appended action. | 12 / 0 |
| 21 | codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__usage_command_with_invalid_view_reports_usage.snap | Help advertises requests/date syntax. | 4 / 0 |

Estimated code/test delta: **2,305 total / 890 non-test**. Proposed target **2,500 total / 1,000 non-test**; **STOP BEFORE exceeding 2,800 total or 1,150 non-test, or adding a path**. Count additions and deletions, generated snapshots and mixed glue conservatively. Separately disclose manager governance/evidence deltas and include them in receiving review; do not hide total change size behind worker-only numbers.

This needs a new manager-accepted size exception to the normal 800-line guidance; no Chat/WS allowance transfers. It is a vertical read-only inspection slice with unknown/corruption and race evidence. A state-reader-only smaller stage is possible, but is not the selected deliverable and must not be substituted without a revised allocation. Existing large orchestration files get only narrow routing/fields. tokens.rs may exceed the soft 500-line non-test target but must stay below 800; the bounded reason is co-locating two usage lifecycles in an existing module without inventing a new source target. If safe readability requires extraction or total size exceeds the bound, STOP and ask the integrator to freeze the exact new paths/stage. Never shrink evidence or hide required fields to fit.

Manager-owned, existing governance files outside worker scope: docs/plans/active/portfolio-agent-cost-accounting.md and docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md. Keep the current sprint within the checker's 100-line limit by linking the frozen allocation/evidence; do not paste this whole design into it. Manager chooses/creates and explicitly allocates any new canonical allocation/receipt file before another writer uses it. No implementation worker may self-create it from an illustrative name.

## 6. Frozen future tests and expected counts

**34 NEW runnable functions: 10 state-private + 6 state external + 12 widget/render + 6 app.** These are design expectations, not executed results. Loop vectors are not extra tests. All new names begin `accounting_inspect_`; retained test counts are discovered at execution and never added to overlapping selector counts as unique tests.

### State private — ten, accounting_store_tests.rs

1. accounting_inspect_absent_schema_is_read_only: missing installation returns unavailable; schema/ledgers/tables unchanged, no install or maintenance.
2. accounting_inspect_schema_rejection_matrix: partial/unversioned/checksum/newer/failed ledger rejects; no repair and no empty-day laundering.
3. accounting_inspect_raw_day_reconciles_contributions: independently literal full/partial/retry goldens equal complete DayTotals and grouped attempt membership; summing multiple estimate versions would fail.
4. accounting_inspect_original_null_and_price_are_immutable: original NULL stays unpriced; changed current catalog/config cannot fill it; bound rates/source/interval remain original.
5. accounting_inspect_missing_binding_is_not_unpriced: missing binding, wrong owner or contribution reference fails explicitly; legitimate NULL succeeds as unknown.
6. accounting_inspect_checkpoint_and_stale_estimate: later read time labels checkpoint lag without writing; stale contributions/missing estimate return needs-refresh; no silent currentness or repair.
7. accounting_inspect_retention_and_compact_matrix: exact raw-cutoff boundary, compact-only, mixed compact/raw, expired aggregate and oldest-day metadata; no detail beyond current 90-day cutoff even if raw bytes remain.
8. accounting_inspect_half_open_date_bounds: exact day start included/end excluded, negative/future day/backward clock/overflow rejected; different observation arrival dates do not move attempts.
9. accounting_inspect_corruption_and_unknowns: conflicting identity/source sequence/old estimate corruption and overflow fail; Missing/Null/0 and no-observation admission remain distinguishable, no fabricated terminal reason.
10. accounting_inspect_single_snapshot_concurrent_writer: deterministic second-connection writer/read barrier proves no mix of old summary with new rows; no probabilistic race-winner expectation.

### State normal library — six, tests/accounting_store.rs

11. accounting_inspect_public_reopens_twice: ordinary installed facade setup, close/reopen twice, identical ownership/current observations/original-price packet; no replay charge.
12. accounting_inspect_public_reads_do_not_write: whole accounting/native table and migration/checkpoint comparison before/after success/absence/error/cancel; selected payload outputs only, no extra auth/network.
13. accounting_inspect_public_identity_retry_scope: two logical calls in one turn and multiple retry attempts, cross-day predecessor, same provider/model in another thread; only correct owner/day participates once.
14. accounting_inspect_public_literal_partial_goldens: explicit Inclusive 100 input/20 read/write missing/40 output yields unknown noncached/full cost and known read+output subtotal using fixed synthetic rates; full counterpart with reported write zero, all-zero-unpriced and nonzero-unpriced variants.
15. accounting_inspect_public_delete_and_empty: native deletion leaves no recoverable packet; unrelated owner intact; no-schema, absent-owner and empty day are distinct; old IDs do not resurrect.
16. accounting_inspect_public_limits_no_truncation: exact accepted/rejected row/observation/result limits; no amount for an incomplete enumeration; no silent first-page-total claim.

### Widget/render — twelve

Eight in tokens_tests.rs:
17. accounting_inspect_partial_and_unknown_copy: inline snapshots, known plus unknown and full-unavailable adjacent; all seven metrics and billed/coverage distinctions.
18. accounting_inspect_unpriced_zero_and_missing_rate: explicit zero vs unknown vs missing snapshot vs absent individual rate; no $0 headline for unknown work.
19. accounting_inspect_exact_rounding_reconciliation: canonical decimals, half-even ties, nonzero sub-micro, sum-before-rounding and subtotal detail remain auditable.
20. accounting_inspect_request_attempt_price_detail: full IDs, retry link, provider/model, scope, original price provenance and interval, absent literal wire/status disclosed.
21. accounting_inspect_narrow_and_long_fields: 40- and 80-column layouts and small viewport; unknown labels and every required detail remain reachable with ordinary keys, sanitized long provider/model strings cannot inject terminal controls.
22. accounting_inspect_availability_state_snapshots: absent/empty/missing-thread/remote/corrupt/stale/retention/too-large/loading states have distinct bounded text and recovery controls.
23. accounting_inspect_coverage_never_claims_run_complete: full numeric estimates still recorded-only; no billing, allowance, balance or collection-complete label; cross-day request caveat retained.
24. accounting_inspect_snapshot_navigation_roundtrip: group -> attempt -> component/price pages -> back retains exact snapshot and selected row; every row reachable, no hidden truncation.

Two in tests/usage.rs:
25. accounting_inspect_menu_and_reset_regression: original Show usage/reset positions and events retained, new third action works; disabled-reset keyboard navigation expectation updated explicitly.
26. accounting_inspect_cancel_refresh_generation: Escape during loading, close/reopen and refresh reject earlier results; navigation cannot reopen a cancelled view.

Two in tests/slash_commands.rs:
27. accounting_inspect_command_without_account_auth: `/usage requests` works with synthetic direct-provider/signed-out profile; no backend/Plan credential probe; existing account command auth remains intact.
28. accounting_inspect_command_date_validation: valid explicit date/default UTC day and invalid/trailing/future inputs; deterministic date injection, no environment edits or query on invalid input.

### App/event integration — six, app/tests.rs

29. accounting_inspect_app_real_store_to_view: real normal-library store populated synthetically, route through AppEvent and keys, compare visible exact component/subtotal packet with independent fixture expectations; no API usage request.
30. accounting_inspect_app_remote_does_not_read_local: local decoy ledger populated but remote target returns unsupported; no local values surfaced or remote account fallback.
31. accounting_inspect_app_thread_switch_stale_reply: held real read then switch thread/clear and release; no old data on new thread, no popup reopening; fresh read uses current owner.
32. accounting_inspect_app_error_retry_and_timeout: missing optional state, locked/failed read and controlled timeout; close works and retry issues a new correlation, no schema repair or collector activation.
33. accounting_inspect_app_restart_and_profile_isolation: isolated installed and fresh profiles; reopen preserves recorded evidence, empty profile does not inherit old view/state or install ledger; no live credential access.
34. accounting_inspect_app_off_and_old_usage_routes: new local inspector produces zero provider/account calls or admissions; ordinary account chart/reset event routes and existing collector Disabled setting remain unchanged.

Before tests read docs/development/test-isolation.md. Ordinary Rust tests use guarded `just test`, isolated synthetic profiles and denied native credential stores. Do not run raw cargo test/nextest. A native prompt or live-profile access stops successor/retry dispatch and invalidates that run. No tests ran for this design.

Proposed commands from codex-rs, after listing selected test names through the guarded harness and after scope-constrained fix/format:

- `just test -p codex-state --lib accounting_inspect_ --locked --offline` — exactly 10 new functions.
- `just test -p codex-state --test accounting_store accounting_inspect_ --locked --offline` — exactly 6 new functions.
- `just test -p codex-tui --lib accounting_inspect_ --locked --offline` — exactly 18 new functions (12 widget + 6 app).
- `just test -p codex-state -p codex-tasknode-session --locked --offline` — combined shared regression suite, discover actual count.
- `just test -p codex-tui --locked --offline` — affected crate suite, discover actual count, retain existing account/reset/slash/context/permission regressions and failures.

Do not claim offline dependency availability before checking it. Fix/format/snapshot acceptance must precede final affected runs. Record exact commands, selected names, run UUIDs, matching JUnit/raw outputs, passed/failed/skipped/leaky counts, source and binary hashes. Normal-library external tests must link production code rather than include private DDL. No full-workspace run is proposed; unchanged Core/API sampling evidence can be referenced at its actual received commits, without inventing a new all-collector qualification.

## 7. Receiving evidence and user-facing gates

Fable owns receiving. Before integration: literal file/diff/size audit, immutable source/base hashes, manager-approved overlap handoff, one independent material code review plus required substantive correction under the existing preserved review ledger. Review focuses on no mutation/activation from reads, missing-vs-zero, original price binding, cutoff enforcement, snapshot consistency, remote/local separation, stale-response leakage and request-level reconciliation. Record scoped allowance extensions through the integrator; do not reset spent reviews or seek repeated opinions on unchanged clean code.

Receive onto the recorded integration branch; run final combined state/TaskNode and TUI gates, governance and git diff --check. Preserve all old failures, wrong assumptions and new attempts. Functional receipts distinguish worker vs receiver vs independent executor; a merge, screenshot or green unit selector is not acceptance. A meaningful fixture must detect a deliberate test-only omission of unknown participation or corruption of summary/detail reconciliation, with failed/restored run evidence if a mutation is used; do not silently keep mutations or count nondiscriminating ones as proof.

### True TUI: owed by this unit

Unlike S02, the unit adds a public local inspection command and menu. It cannot inherit the internal-only N/A. Use the final built/package-pinned candidate in a PTY with actual keys, following .codex/skills/test-tui/SKILL.md, RUST_LOG=trace and a private log directory with synthetic data. Send command text and Enter separately. Exercise direct-provider/signed-out entry, menu entry, actual summary-to-request-to-attempt reconciliation, unknown/unpriced states, narrow layout, scrolling/detail pages, cancel while loading, failure/timeout, retry, thread switch, deletion/reopen and restart/resume. Verify existing account/reset controls through isolated fixtures, without redeeming anything live. Record version, commit, binary/assets hashes, worktree, profile provenance, exact keys/timing, visible checkpoints, raw evidence and cleanup.

Collection OFF means ordinary fresh profiles should show unavailable/empty accurately. A separate setup operator may prepare synthetic installed-state fixtures through accepted APIs for the exact candidate; the tester uses only user controls and cannot repair state. These fixtures prove inspection, not provider collection or all application coverage. No public enable switch is added merely to make acceptance easier. The S04 live-provider check remains separately gated and cannot be faked by seeded rows.

### Independent code-blind functional gate: owed before unqualified handoff

Follow qa/code-blind-functional/README.md and qa/code-blind-functional/isolated-execution.md. This code-informed proposal is NOT that design.

1. Fresh-context designer receives only user intent, essential constraints and feature/transition screenshots. Do not provide this proposal, code, implementation tests, test counts or prior findings. Freeze unedited prioritized starting-state/action/observable-result cases and ambiguities before mapping to implementation evidence.
2. A different fresh-context, code-blind executor receives frozen cases, neutral navigation and exact read-only packaged assets. Enforce filesystem/tool/process/IPC/network restrictions denying repository/history/prior results and real operator credentials, with per-run isolated synthetic state and mediated inference. Record actual executor/child negative probes and positive package/PTY controls. A prompt instruction or binary-only cwd alone is insufficient. No tool-denial bypass.
3. Execute actual keys on representative fresh/existing profiles and applicable platforms. Preserve all attempts, timeouts and fixture failures. Map every frozen case to evidence or explicit disposition; product authority accepts exclusions. Missing prerequisites or basic failures block an unqualified human-test handoff, even when source may be internally integrated.
4. Reviewer independent of implementer and executor checks original cases, raw evidence, provenance and enforced isolation; original designer may review. Run qa/code-blind-functional/check.py on frozen design/results/exact binary. Schema-2 traceability is necessary and is not proof by itself.
5. Charge design/evidence reviews to the existing track budget; record execution costs separately. Named integrator may extend scoped review allowance with history preserved. Do not import another lane's isolation exception or current harness status as S03 approval.

Resolve TensorCash and Isometric Game input paths/bases from the plan before qualification and use disposable worktrees. Both are appropriate representative contexts for an inspector intended for systems and interactive multi-agent work; run applicable S03 journeys in both, or record a reasoned plan-level applicability decision. Record platform coverage honestly. Release-level both-repository, benchmark, documentation and human/release-authority rules remain unchanged. Named-human acceptance is pending; this design does not assert human-test or release readiness.

## 8. Ten-line summary

1. Allocate a first vertical S03 unit: local inspection of one thread's recorded requests for one UTC day.
2. The present TUI shows account token activity and reset credits, not request-accounting explanations.
3. Raw S02 rows retain attempts, revisions and immutable original prices, but expose no public drill-down reader.
4. Add a non-installing state inspection API; do not call open/maintain from the display.
5. Compact days lose request and provider attribution; complete historical/campaign claims need decisions and possibly new storage.
6. Display known subtotals plus unknown participation, missing price separately, and billed cost/collection coverage as unavailable or unknown.
7. Propose accounting-pf60-s03-inspection on bootstrap/acct-inspect-20260915, with exactly 21 existing worker files.
8. Target 2500 total/1000 non-test changed lines; STOP before 2800/1150, any extra path or changed collection/retention contract.
9. Freeze 34 future tests and require actual final combined-tree results, independent review and exact-package real-key TUI proof.
10. Archive S02 and serialize PF-83 overlap before dispatch; independent code-blind design/execution/evidence remain mandatory and collection stays OFF.
