# Assessment of the Opus 5 / Extra architectural review

Date: 2026-08-28. Review only; no product code, plans, sprint states, commits or remote branches changed.

## Verdict

**Useful architectural review, not an adoption-ready change list. Keep the security goals; refine the boundaries, shared contracts and execution graph before broad implementation.** Several recommendations identify real gaps, but the report also contains incorrect source inference, overstatements and proposed changes that would weaken the approved contract.

The strongest recommendations are an explicit per-platform trusted-process design, protection of the authoritative policy state, early browser/classifier feasibility work, defined Codex integration seams, and separation of interface preparation from final qualification. They do not justify stopping all existing foundation reconciliation, automatically changing the deadline, or reducing Moderate/Aggressive guarantees.

The unmodified outside report is [OPUS_REVIEW.md](OPUS_REVIEW.md). Its SHA-256 is `c3b28a1d71229729d91c55149458d89c7b2beb95a385b5f4c362e69fbbdf6aa3`.

## Scope and evidence

- Opus 5 and Extra effort were selected and observed in Claude desktop through Computer Use. The session completed; this is not the earlier interrupted Fable review.
- The reviewer used the retained readings of all 63 security sprints and performed additional direct source inspection. No Corbanu runtime implementation was included in its packet. Its claims about Corbanu are planning observations, not verified vulnerabilities.
- All 127 snapshot checksums passed again after completion. The report remains unedited; this assessment records corrections separately.
- I read the entire report, checked the relevant sprint contracts and source paths, and independently checked the dependency graph. The graph has 63 nodes, no cycles or order violations, and a longest dependency chain of 34 nodes. Nodes are not elapsed time estimates.
- To check OR-20, I additionally inspected the underlying plugin-state store in the previously downloaded full OpenClaw checkout, verifying HEAD is the same pin: `13adff02ca3897768d80d2bca18f5acf08c55d91`. Those implementation files were absent from the review packet. No source or runtime test was executed as part of that additional inspection.
- I consulted primary Linux, Microsoft and Apple documentation to check the review's broad platform claims; links appear below.
- The report says **22 findings**, but contains **21 numbered finding sections**: OR-01–13 and OR-15–22. OR-14 is absent. Its coverage section says **18 source files** but enumerates 20 in that list plus four targeted-range files. These are recordkeeping errors; use the named paths and limits rather than the aggregate count. The seven supplementary Autoreview sprint records were not individually reviewed by Opus; its plan and index were read.
- No new platform qualification, live-network exploit reproduction, core suite, TUI run or release certification is claimed. The previous plan/sprint validators passed, which establishes structural consistency only.

## Findings: disposition

| Finding | Assessment | What should be retained or corrected |
| --- | --- | --- |
| OR-01 — schedule/WIP | **Valid risk; overstated conclusion** | One active sprint per plan prevents concurrent implementation, and 34 dependency stages are real. But 63 bounded execution units in 41 days is not *arithmetically impossible* without duration/capacity estimates. Estimate work and qualify the date; do not label this a proven critical security defect or block PF-15 reconciliation on that arithmetic alone. |
| OR-02 — platform containment | **Accept the design gate; reject universal OS claims** | Name the trusted controller/broker, untrusted workers, OS mechanisms, permitted IPC/handles, signing requirements and capability probes on each OS. A separate same-user process alone is not a sufficient argument. However, the report's assertions that all Linux mechanisms require elevation, Windows realistically requires a separate account, and macOS containment exists only after notarization are not established and are too categorical. Unsupported protected-mode activation is already required to fail visibly in the plan. |
| OR-03 — mutable security state | **Accept as an explicit missing mechanism, not a proven exploit** | Specify protection of authoritative level, grants, revocation/kill generations and recovery state against agent file writes, deletion, replacement and rollback. An authenticated UI event alone does not protect persisted state. The packet does not establish that the real config path is currently agent-writable; the example exploit is hypothetical. |
| OR-04 — moving upstream baseline | **Accept dual-baseline testing; do not replace independent evidence** | Add an upstream-aligned control and a reviewed upstream-drift ledger. Keep the independently captured pre-feature baseline. Building the same modified code with a feature off is useful, but both modes can share the same regression; it is not sufficient to demote the original baseline to an informational check. |
| OR-05 — upstream seams | **Accept specificity; correct absence claims** | Require an enumerated hook/owner/contract-test map and requalification after Codex merges. The product spec already assigns upstream parity and regressions to the lead developer, and the plan already requests small Codex hooks. The missing part is executable detail, not all ownership. In-place hook edits are sometimes necessary. A side table is an option, not a blanket improvement: it creates identity, persistence and missing-record risks of its own. |
| OR-06 — browser parallelism | **Accept preparation/integration split** | Move image/dependency pinning, platform feasibility and pure destination-policy work earlier. Keep actual agent/broker connection enforcement and protected retrieval dependent on qualified launch/network contracts. Do not simply delete live integration edges. Use valid stable sprint identifiers, not the proposed S01a/S01b notation without a schema/process decision. |
| OR-07 — classifier critical path | **Accept early feasibility and interface work** | Bring licensed corpus, evaluator-owned holdouts, CPU baseline and detector artifact feasibility forward. Define segment/verdict contracts so quarantine/facade preparation can use fixtures. Keep real classifier quality and end-to-end gates mandatory before enabled protected ingestion; fixture-based construction is not feature completion. Multi-week estimates remain estimates, not measured facts. |
| OR-08 — review milestone blocking construction | **Accept conditionally** | Some implementation can proceed against completed interface contracts while independent qualification is pending, with protected activation unavailable. Do not remove gates without re-auditing all required interfaces and tests. PF-13-S05 also owns a planned canary harness, so the report's claim that it produces no code is inaccurate. |
| OR-09 — credential-use bounds | **Accept explicit quantitative enforcement; correct the premise** | PF-13-S01 already binds purpose/operation and reuses BoundedGrant; PF-17 includes quantitative limits; PF-13-S04 requires atomic consume/replay rejection. It is wrong to describe the entire contract as confidentiality-only. Still, explicit per-request/aggregate usage reservations, authoritative metering and adjacent-operation tests at the credential transport are worthwhile refinements. Do not duplicate the existing authority engine. |
| OR-10 — taint usability | **Accept a usability/action matrix; reject automatic clean-value inference** | PF-30-S01 already specifies source/lineage envelopes, and PF-30-S03 explicitly allows unchanged narrow grants to satisfy their existing scope. Thus not every future action necessarily needs fresh approval. A model exposed to hostile content can alter control flow while selecting apparently human-origin values. Merely tagging argument values as clean cannot prove an action uninfluenced. Require trusted reconstruction or an exact bounded human mandate; preserve conservative ancestry. Measure realistic Moderate workflows without weakening this rule. |
| OR-11 — unknown financial effects | **Accept joint recovery tests; reject blocking emergency restriction** | Preserve uncertain submitted effects across policy changes and never label them cancelled. But a kill/restrictive transition must not wait for an unknown transaction to settle. Revoke future authority immediately, retain receipt/status reconciliation separately, and show the irreversible/unknown effect honestly. |
| OR-12 — descendant authority requests | **Useful UX addition** | A typed, rate-limited, deduplicated request may surface the exact child/actor/scope to the human. It is untrusted data, not a grant. The reviewed grant TUI lacks an explicit request/escalation interaction, but the packet cannot prove that existing orchestration has no notification path. |
| OR-13 — closed-world ingress | **Retain as a sharpened acceptance test** | Unknown origin and missing/corrupt provenance already fail conservatively in PF-30. Add a synthetic newly introduced ingress route and prove it cannot bypass registration/screening. This is not a newly absent security principle. |
| OR-15 — debug/capture sinks | **Retain a named integration test; correct the novelty claim** | The fetch guard passes headers/body to a capture subsystem, so trace its final scrubbing and persistence. That alone does not prove raw capture persistence. PF-13-S05 already explicitly lists proxy/request capture; PF-28 covers traces/diagnostic artifacts and PF-27 inventories flags/launch paths. Name the exact capture adapter rather than claiming no inventory exists. |
| OR-16 — Done wording | **Optional clarity improvement, not a demonstrated policy violation** | A commit can be observed to contain code while behavioral acceptance remains pending. The plan already says presence is not acceptance. Reword if it helps readers; do not build a checker that treats every historical “added” statement as false completion. |
| OR-17 — stale TLS cache after re-registration | **Accept as a strong, source-supported adoption concern** | The cache key omits a run generation and its handler captures the prior registration. Add the specific new-connection-after-revoke/re-register test, including cases where the outer traffic policy still admits the host. This refines already-required same-run replacement coverage. It is not an executed exploit or proof Corbanu has this bug. |
| OR-18 — empty allowlist polarity | **Source behavior confirmed; proposed policy requires qualification** | Empty normalized lists impose no hostname restriction in this helper; public/private address checks still exist. Distinguish absent configuration, explicitly empty deny-all, wildcard public scope and authorized exceptions. Do not silently impose a new per-host whitelist on every Moderate public retrieval request. Aggressive's explicit grants and the resolved profile policy remain authoritative. |
| OR-19 — exact-host private DNS exceptions | **Source behavior confirmed; qualify the remedy** | The code intentionally permits private provider destinations for trusted exact hosts. Keep that separate from public-fetch permission and revalidate actual connections. Define an approved address/identity set and change policy; a single forever-static operator IP is one implementation choice, not the only valid solution. TLS identity and destination authorization are distinct controls. |
| OR-20 — provenance capacity/paths | **Reject the capacity exploit as stated; retain path/hash tests** | The underlying reject-new store throws before the wrapped file commit (detailed below). Filename-based system classification and absent read-time content-hash comparison are genuine source observations already covered in the source review/PF-30-S02. Do not relabel all three as newly discovered gaps. |
| OR-21 — response streaming | **Accept latency targets; keep protocol-specific proof** | Bounded carry is a useful technique for known literal encodings. Request-sentinel substitution does not prove arbitrary response decoding or reflected-secret protection. Required ingestion classification still forbids releasing unexamined content. Define which streams may be incremental and test each encoding/decoder/resource bound; do not promise universal first-token streaming. |
| OR-22 — optional hosted vendor gate | **Reject the supposed commercial blocker** | PF-36-S02 explicitly permits an evidenced disabled-no-qualified-vendor disposition. Completing its interface/failure/fallback evidence does not require a commercial vendor decision. Keep completion/disposition evidence; there is no need to delete the dependency merely because hosted activation is optional. |

## Important corrections with direct evidence

### OR-20: capacity rejection prevents the normal wrapped write

The full pinned source establishes this order:

1. `createMemoryWriteProvenanceObserver.write` awaits `recordMemoryArtifactWriteProvenance` **before** invoking `commit` ([writer](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/agents/memory-write-provenance.ts#L124)).
2. The provenance function calls the synchronous keyed store's `update` ([provenance](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/memory/memory-artifact-provenance.ts#L150)).
3. The sync adapter forwards to `pluginStateUpdate` without swallowing its exception ([adapter](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/plugin-state/plugin-state-store.ts#L254)).
4. For a new key, `pluginStateUpdate` calls `assertCanInsertPluginStateEntry`; the latter throws `PLUGIN_STATE_LIMIT_EXCEEDED` at capacity. The outer catch rethrows a wrapped error ([capacity check](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/plugin-state/plugin-state-store.sqlite.ts#L561), [update](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/plugin-state/plugin-state-store.sqlite.ts#L896)).
5. An existing upstream test explicitly expects new-key `update` to reject at capacity ([test inspected, not run](https://github.com/openclaw/openclaw/blob/13adff02ca3897768d80d2bca18f5acf08c55d91/src/plugin-state/plugin-state-store.test.ts#L303)).

Therefore the report's capacity → unrecorded successful wrapped write → agent-origin promotion chain is not supported. Unwrapped writers, special paths and read-time identity remain separate concerns. This is a source-level correction, not an assertion that every upstream writer has been proven safe.

### OR-02: mechanisms must be tested, but elevation is not universally mandatory

Linux documentation explicitly permits unprivileged seccomp filters with `no_new_privs`; this refutes the blanket claim that every listed mechanism entails installation-time elevation, without establishing that seccomp alone satisfies Corbanu's whole threat model. [Linux kernel documentation](https://cdn.kernel.org/doc/html/latest/userspace-api/no_new_privs.html)

Microsoft documents AppContainer/LPAC process, file, network and credential isolation, using package/capability identities in addition to user identity. These are alternatives to assess, not grounds to assert that a distinct service account is the only realistic Windows design. No Corbanu configuration was qualified here. [Microsoft AppContainer implementation guide](https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer)

Apple documents hardened-runtime protections and the security consequences of `get-task-allow`, as well as their relationship to notarization. Code signing, runtime entitlements, sandboxing and notarization must be evaluated separately; the review does not prove notarization itself is the necessary-and-sufficient boundary. [Apple notarization guidance](https://developer.apple.com/documentation/security/resolving-common-notarization-issues)

### Do not adopt the proposed weaker “Moderate v1”

Section 9.3 suggests a classifier-less Moderate that can leave retrieval at Permissive behavior. That conflicts with the current approved protected-mode contract. Keep affected operations unavailable until the required isolation, screening, provenance and authority checks pass. A separately approved UI-only milestone may truthfully show unavailable modes, but does not satisfy the existing full security deliverable or waive release evidence. I do not recommend changing the meaning of Moderate to meet a date.

### Keep an independent baseline and conservative provenance

Feature-on/off comparison is supplemental, not an independent oracle when both builds share changed code. Likewise, clean-looking action arguments are not an oracle for lack of prompt-injection influence. These two recommendations need stronger designs than the review proposes.

## Additional gaps from my validation

1. **Browser login has an undeclared screening dependency.** PF-37-S01 requires output through PF-28/34/35, but its graph prerequisites are PF-31-S03, PF-28-S02 and PF-30-S03. PF-34/PF-35 are not even transitive prerequisites. Listed execution order currently hides this; parallel scheduling exposes it. Add the required interface/integration gates before treating the lane as independent. [Sprint contract](../../../../docs/sprints/current/p0-security-levels/pf-37-s01-origin-bound-browser-login.md)
2. **Shared audit/durable-state contracts arrive too late.** PF-34-S02 already needs a durable audit chain, PF-38-S03 needs tamper-linked receipts, and PF-40-S01 needs integrity-chained events; PF-41-S02 later unifies the chain. Define event IDs, durable commit/failure semantics and ownership early, while keeping inspector/export presentation later. This need not mean one shared monolithic database. [Audit sprint](../../../../docs/sprints/current/p0-security-levels/pf-41-s02-tamper-evident-security-audit.md)
3. **Parallel plans must remain executable under the actual checker.** The review proposes 13 lanes, letter-suffixed sprint IDs, interface dependencies and an uncomputed reduction to the low 20s. It does not supply a revised validated graph or staffing estimate, and even says both “nothing else starts before stage 0” and “dependency-free day-one lanes.” Treat it as an idea map, not a ready execution schedule. Prefer completed single-feature contract sprints as freeze points; no new dependency kind is necessary merely to get started.

## Recommended bounded parallel plan

After a deliberate process/worktree amendment, allow **at most three independent active implementation sprints**, each with a named owner, separate worktree/branch, completed prerequisites, bounded file ownership and an explicit integration gate. This is a recommendation, not a policy change made by this review.

| Stream | Work that can be moved forward | What remains blocked |
| --- | --- | --- |
| Authority/platform foundation | Reconcile PF-15–22; specify broker/controller trust and protected state; define IPC, revocation and durable event contracts | Protected activation until complete containment, output, migration and qualification evidence |
| Browser/retrieval preparation | Artifact/runtime selection and pins, existing-engine reuse, platform capability probes, pure destination policy and resource/isolation fixtures | Live protected retrieval/login until broker/launch/connection/screening integration is complete |
| Ingress/classifier preparation | Source/segment/verdict contracts, licensed corpus, blind holdouts, hardware feasibility, sanitizer/quarantine fixtures | Enabled protected ingestion until actual detector, lineage and deterministic-authority tests pass |

Later, rotate one slot to provider adapters or financial integration. Exa, Brave and SearXNG are naturally independent after a completed common facade/auth/screening contract. Avoid assigning separate teams to overlapping Core/protocol hooks without one integration owner. Preserve PF-13-S05 and final PF-26 qualification as non-optional gates; enabling concurrency is not permission to mark stubbed or unqualified work complete.

For Codex upgrades, require a per-hook table recording the upstream symbol, Corbanu-owned implementation, owner, expected policy/provenance contract, regression tests and last upstream revision checked. Re-run seam, ingress/egress, inheritance, persistence and Permissive comparison tests after each relevant merge. Never resolve a security-hook conflict solely by accepting either side.

## Tests to prioritize

| Priority | Test family | Required result |
| --- | --- | --- |
| P0 | Forced detector false negatives, missing verdicts, corrupted envelopes and hostile child/memory content | No unapproved disclosure/action, no trust promotion; unavailable screening pauses affected ingestion |
| P0 | Actual agent-context process, filesystem, IPC, inherited-handle and network probes on all three OSes | Broker secrets and authoritative policy state inaccessible; unsupported containment prevents activation visibly |
| P0 | Policy-file overwrite/delete/rename/symlink/rollback, then restart/resume | No unauthorized downgrade or restored revoked authority; tamper/failure visible |
| P0 | Revoke open channels, uploads and queued work; same-run re-registration and stale pooled/TLS state | No new protected dispatch after the revocation fence; unrelated authorized runs continue; irreversible effects remain honestly tracked |
| P0 | Reflected secrets in body/headers/trailers/SSE/error/debug capture, short/encoded/split/rotated values and capacity pressure | No managed-secret escape; unsupported decoding/resource conditions deny without raw fallback |
| P0 | Real DNS/redirect/connection pinning, proxy/NO_PROXY/direct socket/QUIC escape, private-service exceptions | The authorized destination is the actual peer; no public-fetch private-network bypass or stale-connection authority |
| P0 | Multi-turn compaction/memory/import/export/child lineage, capacity failure, special filenames and out-of-band content edits | No lineage loss, filename trust promotion or write-on-provenance-failure |
| P0 | Migration and financial crash boundaries; unknown submission plus immediate kill/restriction | Durable ownership-aware recovery; no plaintext recovery copy, stale approval reuse or duplicate financial effect |
| P1 | Upstream merge regression and dual-baseline Permissive tests; synthetic new ingress/egress route | New/changed paths cannot silently omit the security hooks or change baseline policy |
| P1 | Realistic Moderate research workflow and all affected true-TUI success/cancel/failure/recovery/resume flows | Measured approval/latency costs without taint laundering; final candidate proof in both live repositories and human acceptance |

These are planned acceptance recommendations, not passing tests from this review. Many are already in the sprints; the improvement is concrete fixtures, common ownership and integration timing.

## Suggested next decision

Authorize a **planning-only refinement**, retaining all current protected-mode guarantees: define the platform/state and shared-interface gates, allow a small number of independent active sprints, add the missing semantic dependencies, and replace the unassessed date with a measured capacity/scope decision. Keep this assessment alongside the raw review so its rejected or qualified claims are not accidentally promoted into requirements.
