# RETURN — acct-inspect-impl-03

Status: implementation committed for scoped receiving review; all 34 named tests pass. Full TUI regression failed. No functional/human-test qualification claimed. The action's final RETURN supplies the commit hash; the immutable source diff digest is recorded below.

## Frozen provenance and authority

- Worker: gpt-6-astra, high; action acct-inspect-impl-03.
- Allocation digest: d5d5210e3466ca2982800639e53413e98ab6cba35b3aa6ffe81d6c1d997f30a0.
- Claim: 7e7ac0de-eb81-4be6-8409-7e6466c78f14.
- Brief: /private/tmp/fmgr.Q1SIYZ/briefs/acct-inspect-impl-03.json.
- Verified SHA-256: f049548e5e5d1a7db009314ed65b3f6130bee71b30cd1858a229a1d2498ac945. The brief retains the historical allocation label acct-inspect-impl-01; the dispatch envelope and this receipt identify action 03.
- Clean launch HEAD: 40442cc751dec6e0bc91c5ba7364d7c1aa0e0c00; branch bootstrap/acct-inspect-20260915.
- Worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/acct-inspect-20260915.
- Product initiative, active plan docs/plans/active/portfolio-agent-cost-accounting.md, in-progress sprint PF-60-S03.
- Product authority: **Measurement targets**, docs/corbanu-product-spec.md: “No commercial performance numbers have been supplied. The following metrics must be instrumented, with targets set through the decision rights defined above.”
- Frozen unit: docs/research/agent-cost-accounting/s03-inspection-allocation-20260915.md. PF-83's three overlapping TUI reservations were explicitly released and absent from its live write_scope at launch.
- The plan/sprint branch-base is 5105e3ce44d31fd16a35f6cb6900ef6abf6168a8; the dispatch explicitly distinguishes it from the launch HEAD above. Both governance checks passed before editing.
- The predecessor patch was reviewed and adapted; none of its compilation/test claims were trusted. Only allocated source paths changed. The full suite generated 57 transient pending snapshot artifacts; these were relocated into the allocated QA directory without accepting or modifying their source baselines. The oversized app/tests.rs exceeded structured_edit's 524288-byte input limit, so its bounded insertion used apply_patch.
- No subagents, external messages, pushes, public activation switch, live profile, credential reads or native credential approvals.

## Implemented contract

The local /usage requests [YYYY-MM-DD] argument precedes the account-auth guard. It validates one exact UTC date before dispatch, defaults to today, rejects future/pre-epoch/invalid/trailing arguments, and appends a third menu action while retaining account/reset order and predicates.

AccountingStore::inspect_day accepts an existing StateRuntime and reads one SQLite transaction without open/install/maintenance/repair/import/repricing. It validates schema/version/checksum, native owner, observation identities, original price bindings and old estimates; reconciles the complete DayTotals with grouped latest quotes; rejects stale contributions and corrupt evidence. Admission timestamps determine day membership. NULL original binding remains unpriced; missing binding is an error.

The embedded-only app route has no app-server client parameter. It uses the existing optional database and current native owner, with a 15-second timeout and bounded error copy. A fresh generation/thread/day correlates results. Cancel, refresh, account reset, thread change/redraw and widget clearing invalidate the local view. Page selection survives Back. The snapshot is rendered into scrollable, sanitized 26-column text fragments, with full identifiers and exact values reachable at 40/80 columns; no packet is inserted into chat/model history.

Collection remains OFF. This inspector does not establish collection coverage or billed costs.

## Exact unknown, rounding and retention copy

For the literal Inclusive fixture input=100, read=20, missing write, output=40 with rates 1/1/2/4 USD per million:

```text
Known estimated token cost: $0.000180 + unknown costs
Full recorded estimate: unavailable (1 of 1 attempts incomplete)
Cache write: unknown — no retained numeric evidence
Noncached input (derived for inclusive input): unknown — no retained numeric evidence
Collection coverage: unknown; recorded attempts only. Descendants excluded.
Billed cost: unavailable — no settlement evidence
```

A zero known subtotal with incomplete work leads with “Estimated token cost: unknown”; it is never headlined as zero spend. A complete all-zero recorded estimate can show $0.000000 while absent original price remains explicit. Missing usage and missing rate have separate bucket reasons. Reported zeros say “0 (reported)”; derived values say “(derived)”. Half-even rounding uses the existing Decimal formatter; rounded values and every nonzero sub-micro amount are labeled, and canonical exact decimal strings are separately reachable.

Whole days touching the wall-clock 90-day cutoff, compact-only/mixed days and expired aggregate days carry no displayed amount. The view says “Request detail unavailable for this whole UTC day”, identifies compacted attribution loss where applicable, and reports the actual cutoff/checkpoint/oldest-day metadata. It never invents constituents or displays a partial day's total. Checkpoint lag remains visible as “Snapshot is not current; newer activity is unverified.”

## Reader limits and evidence limits

- At most 10,000 retained attempt rows, 512 selected attempts, 4,096 observations per attempt and a 4 MiB projected quote packet, with conservative header/map/row overhead.
- The inherited whole-store validator visits unrelated history and old estimate versions. Before materialization, the implementation also caps their combined serialized input at 4 MiB and applies the observation cap store-wide. Thus a small selected day can return TooLarge due to unrelated retained evidence. This conservative behavior is disclosed for receiving review; no schema/index changes were made.
- The boundary fixture proves a valid 512-attempt result and rejection at 513, and a valid 4,096-observation result and rejection at 4,097. The 10,000-row and exact 4 MiB input edges use deliberately invalid payloads to prove the cap runs before decoding (at the edge validation rejects; above it TooLarge). They do not prove a valid 10,000-row result or a maximal valid projected packet.
- The app timeout case exercises the exact production timeout wrapper with a held future and controlled Tokio time, alongside actual missing/closed database event routing. It is not an OS-lock qualification.
- The concurrent state case establishes the facade's read snapshot with schema validation, then releases a second-connection writer and invokes the same reader in the held transaction. The old packet remains internally consistent and the next facade read observes new evidence.
- The app thread-switch case holds an actual read result before delivery, then switches owner; it does not suspend the SQLite engine mid-query.
- Original failed attempts, compiler errors, nextest retries and correction runs are retained in distinct logs. Intermediate passes are not final-tree qualification.

## Verification and 34-case map

The [34-case map](impl-03-case-map.json) verifies one matching passing JUnit testcase for each frozen runnable function: 10 state-private, 6 state normal-library, 12 widget/render and 6 app/event. It does not silently convert the evidence limitations above into acceptance.

The [source manifest](impl-03-source-manifest.json) records all 21 source/test/snapshot paths, each file's additions/deletions, final line count and SHA-256. Source diff: [impl-03-source.patch](impl-03-source.patch), SHA-256 d92a7ed578b55b67cea043f89cf6d51d333612d3d2975a4ec221daafbbbfa092. Code/test/snapshot delta is **2,382 lines / 863 production lines**, below target 2,500/1,000 and STOP 2,800/1,150; remaining STOP headroom 418/287. tokens.rs is 797 physical lines including its test-module declaration, below the frozen 800-line module ceiling. The [complete receipt manifest](impl-03-receipt-manifest.json) separately counts governance and generated evidence, including raw logs, JUnit, unaccepted snapshots and this receipt, with per-file line counts. Those artifacts remain part of receiving review; they are not hidden inside the source-only budget.

All Rust commands below ran from codex-rs through this checkout's guarded just test. The wrapper reported a disposable profile and denied native keyring access. No native prompt or live-profile access was observed. Source formatting used rustfmt --edition 2024 --config skip_children=true on changed files only; no just fmt/fix or workspace formatter ran. Snapshot updates were limited to the four allocated existing snapshots and reviewed before final regression.

| Log | Exact command after just test | Exit | Result |
| --- | --- | ---: | --- |
| impl-03-state-preflight.log | -p codex-state --lib accounting_inspect_ --locked --offline | 4 | Compiled initial reader; 0 selected tests (before authoring), 297 skipped; not a pass |
| impl-03-state-private-01.log | -p codex-state --lib accounting_inspect_ --locked --offline | 100 | 7/10 passed; 3 fixture assertions confused native noncached and inclusive input |
| impl-03-state-private-02.log | -p codex-state --lib accounting_inspect_ --locked --offline | 0 | 10/10 passed; intermediate tree |
| impl-03-state-private-03.log | -p codex-state --lib accounting_inspect_ --locked --offline | 100 | 9/10 passed; newly added overflow setup was rejected by the writer, then changed to explicit corrupt fixture rows |
| impl-03-state-public-01.log | -p codex-state --test accounting_store accounting_inspect_ --locked --offline | 101 | Compile error: RetentionCoverage is not Clone; corrected assertion |
| impl-03-state-public-02.log | -p codex-state --test accounting_store accounting_inspect_ --locked --offline | 100 | 5/6 passed; fixture exceeded the existing 256-observation write batch |
| impl-03-state-public-03.log | -p codex-state --test accounting_store accounting_inspect_ --locked --offline | 0 | 6/6 passed, 4 skipped; valid 4096-observation fixture used 256-row batches |
| impl-03-tui-01.log | -p codex-tui --lib accounting_inspect_ --locked --offline | 0 | 12/12 then-authored widget tests passed, 4019 skipped |
| impl-03-tui-02.log | -p codex-tui --lib -E 'test(accounting_inspect_) or test(usage_command_) or test(usage_menu_) or test(rate_limit_reset_)' --locked --offline | 101 | App fixture error-type/private-method compile errors; INSTA_UPDATE=always, no passing test claim |
| impl-03-tui-03.log | -p codex-tui --lib -E 'test(accounting_inspect_) or test(usage_command_) or test(usage_menu_) or test(rate_limit_reset_)' --locked --offline | 100 | 45/46 passed, 3991 skipped; INSTA_UPDATE=always accepted four intended snapshots; remaining assertion wrongly required wrapped copy on one line |
| impl-03-state-tasknode-final.log | -p codex-state -p codex-tasknode-session --locked --offline | 0 | 406/406 passed, 0 skipped, 1 leaky marker on existing extract::tests::turn_context_sets_model_and_reasoning_effort |
| impl-03-tui-focused-final.log | -p codex-tui --lib accounting_inspect_ --locked --offline | 0 | 18/18 passed, 4019 skipped, no leaky marker |
| impl-03-tui-full-final.log | -p codex-tui --locked --offline | 100 | 4120 run: 3980 passed (3 slow, 5 flaky), 93 failed, 47 timed out; 8 skipped |

Final state/TaskNode run UUID: 92598bba-e6f2-48b7-9eb6-ebe9c57ed06c. Focused final TUI UUID: c0c87928-8044-4172-8679-f06dc8dca35c. Matching JUnit files are preserved beside their raw logs. Full TUI UUID: 099247bb-798d-4070-a5d2-a957625c9c94. Its JUnit confirms all 18 inspection cases passed again; the complete suite is still failed. Selector counts overlap the complete suites and are not summed as unique tests.

The [full-suite failure list](impl-03-tui-full-failures.json) preserves 140 final failing/timed-out case identities. Observed classes include embedded-app/session/mailbox timeouts, stale version/model/status snapshots, missing codex executable at some integration launches, and TUI initialization failures. One explicit inherited mismatch is v0.1.35 expected versus the unchanged package's v0.1.42; no blanket baseline-cause claim is made for other failures. Fixing these existing workflows/baselines is outside the frozen unit, so further correction dispatch stopped at scope. The [57-artifact index](impl-03-pending-snapshots.json) records every unaccepted pending snapshot's original path, preserved QA path and hash; no unrelated snapshot was accepted. Full-suite automated PTY cases are supporting regression evidence, not the required independent exact-package inspection workflow. The suite also left an untracked tui/target/tmux-artifacts directory; its opaque fixture/viewport/reproduction artifacts were moved intact to full-suite-tmux-artifacts under this receipt before final cleanup. No fixture credential contents were inspected.

The executing TUI debug test binary was observed via process executable paths (no argument/environment inspection): /Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/target/debug/deps/codex_tui-73b7918ba310303d, SHA-256 5622f5962b7407d46b92a4ef417ea881c70df18f7b3b015e697d9e4a6da79d36. Package version is 0.1.42. The inherited shared build target was used; artifact-lock waits are preserved in logs. This is a test executable, not a qualified release package.

Governance checks python3 docs/plans/check.py and python3 docs/sprints/check.py passed at launch and after ledger editing. Source/docs diff checks passed, including git diff --cached --check -- codex-rs docs. The full staged git diff --cached --check exits 2 on whitespace preserved verbatim in raw patch/log/XML evidence; those raw attempts were not rewritten to make the check green. These are process results, not functional evidence.

## Required receiving and functional work

Fable owns literal scope/size review, independent material code review and combined receiving-tree checks. This implementation return is not an unqualified human-test handoff.

Still owed: a fresh-context code-blind design frozen from intent/constraints/screenshots only; a separate code-blind executor with enforced filesystem/tool/process/IPC/network isolation, synthetic fresh/existing profiles, negative access probes and positive package/PTY controls; actual keys through the exact packaged candidate; independent evidence review; true-TUI failure/cancel/recovery/refresh/resume and account/reset regressions. Preserve raw attempts and schema-2 receipts, run the handoff checker, and record named-human acceptance when obtained. No part of this code-informed suite substitutes for those gates.

The setup operator will need an installed synthetic state fixture through accepted APIs (with partial/unpriced/full/empty/compact cases), an untouched fresh profile, and the exact read-only binary/assets. Collection OFF makes fresh-profile unavailable/empty behavior expected. No public enable switch is needed for the fixture.

Resolve TensorCash and Isometric Game paths/base commits and use disposable worktrees for applicable qualification. No live-repository run, packaged binary/asset hash, human acceptance, benchmark or release claim is made here.

Later S03 units still own subtree/campaign membership, root/descendant and provider/model aggregate views, custom ranges/intervals, mixed raw/compact aggregation and any separately authorized persisted dimensions. All inherited S02 obligations remain on the sprint ledger. The sprint remains in_progress.
