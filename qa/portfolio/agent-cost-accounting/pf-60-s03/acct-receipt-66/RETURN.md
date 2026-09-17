# RETURN — acct-receipt-66

Allocation digest **e29b037b6e3afc51ca36923db0f6c4b9aa9ff840742be26bbc204835cd5ffd4b**;
claim **6a072701-26e3-4e4d-a03a-fec7c65dd5e1**; **gpt-6-astra / high**.
Brief SHA-256 verified:
**be6fb351aa49bea6b0bbb7346367af664198842596a24595b954c577542589c8**.
Assigned/tested HEAD: **270a6644e01d780ef93f8715dd0051481643dc7d**.
Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/acct-activation-20260916`.

**Receipt corrections and checker controls completed; concrete next step
specified. Required Rust gates are not all green: 341 passed, one timed out
across 342 lane test counts.** No product implementation or inherited
qualification is claimed.

Routine QA tooling/evidence work under the corbanu-terminal-development skill.
Product context follows PF-60: exact heading **Product measurement**, excerpt
“No commercial performance numbers have been supplied.” Plan PF-60 is active;
PF-60-S03 is in_progress. Fable owns their reconciliation and new product
allocation. This receipt does not alter their execution coordinates or status.

## Each capture supports its own receipt

[Round 61](../acct-readers-61/RETURN.md) now describes its never-prompted
capture only as a fresh-session control: zero recorded attempts in that root
and resolved descendants, unknown collection coverage, no established day-wide
amount. Its detailed findings and verdict table agree. It cross-references the
separate reproduction without importing that reproduction's amount/population.

[Round 62](../acct-scope-62/RETURN.md) retains **its own** four priced attempts
across three roots, independently totaling **USD 0.00284**. Its scope-zero
page and seven priced pages are rechecked in [scope-recheck.json](scope-recheck.json).
The round-64 receipt now distinguishes these captures explicitly.

The [corrected disposition](../acct-inherited-64/inherited-disposition.md#existing-collection-cross-check-and-exact-reader-text)
quotes both headers and all 24 selected rows in page order, including:
“Collection coverage: unknown; recorded root and resolved descendants only.
Unknown parent population excluded.”
It identifies the scrolling sequence and omitted chrome, rather than calling
a two-line excerpt the complete page. Round 62 also labels its shorter quotation
as an excerpt in page order. [Capture hashes and rows](capture-bindings.json)
bind both original selected files to their unchanged viewport sequences.
No capture or historical receipt JSON was overwritten.

## Counterexamples and asserted asymmetry

[74 controls](page-controls-01/results.json) all reject before writing a revised
success receipt. The original 38-case group's **34 earlier acceptances are now
asserted**, independently of the extra cases. Four new forms are injected on
each of nine nonmonetary pages: `€9.99`, `999 cents`, a symbol-free
`Estimated token cost: 9.99`, and `$.99`.

All **36** evade the round-64 regex, and actual executions of **both** frozen
earlier auditors accept them. Assertions require these counts as well as
**70 total round-62 acceptances**. Both earlier baselines pass. Complete frozen
nonmonetary-page content is now bound by canonical JSON digests, rather than
adding currency spellings to a blacklist. This is a reader-run-05 evidence
contract, not a general parser or approval of future UI wording.

[28 prior removal controls](removal-controls-01/results.json) also reject.
Baseline remains **9 priced page reconciliations, 4 unknown-cost pages and
2 zero-recorded pages**, with all 24 page identities bound. All are re-audits
or deliberate QA mutations, not newly executed product observations.

## Smallest next step

[The proposed slice](next-step.md) selects **startup prewarm on direct OpenAI
Responses WebSocket**: bind a distinct startup lifecycle/attempt to its owner,
collect existing raw completion usage through the admitted WS observer, persist
the operation distinction, and display it with explicit unknown cost.
[Seven frozen source excerpts](next-step-source.json) show the caller, current
exclusion/drain, attempt contract and existing fixture.

The product-versus-QA split is explicit: product work comprises identity and
persistence compatibility, native collection plumbing and reader projection;
QA work adapts the warmup fixture and adds ownership, presence, failure/retry,
cached reuse, reopen/retention, default-OFF and packaged real-key evidence.
The first qualifiable slice is collected and inspectable startup usage with
unknown economics. A prewarm-specific authoritative economic source is still
required for a priced/billed claim. Removing noncollection is progress, not
completion of all inherited prewarm requirements. No product changes are made.

## Exact required gates

Prerequisites built first (exit 0); all lanes ran from codex-rs, shared dedicated
`acct-activation-33/feature/target`, `NEXTEST_TEST_THREADS=4`,
`CARGO_BUILD_JOBS=2`, debug assertions enabled and guarded `just test`.

| Command | Run | Passed | Failed / timed out | Exit |
| --- | ---: | ---: | ---: | ---: |
| `just test -p codex-core accounting` | 124 | 124 | 0 / 0 | 0 |
| `just test -p codex-core accounting --features codex-core/developer-accounting` | 127 | 126 | 0 / 1 | 100 |
| `just test -p codex-tui usage` | 91 | 91 | 0 / 0 | 0 |

Exact timeout name:
`codex-core::all suite::accounting_responses_ws_recovery::accounting_responses_ws_native_auxiliary_scope_and_event_parity`.
Feature TRY 1 timed out at **60.013s**; nextest's configured TRY 2 timed out at
**60.012s**. Both raw attempts remain in the lossless feature log.
The same test passed the default lane in **58.790s**. No assertion failure,
native credential prompt or live-profile access was observed. The timeout's
cause is not established; a default-lane pass does not qualify the feature lane.
No manual replay, timeout relaxation or production/test-source fix followed.

[Machine-readable results](test-results.json), [raw gate logs and commands](gates-01/),
[reproduction](REPRODUCE.md). Counts overlap across Core lanes; the configured
retry is not an additional distinct test. Zero positive inherited-item or
measured-dollar qualifications are added.

## Changed lines, brief accuracy and limits

Existing tracked QA changes: **+146/−41**, including checker source **+58/−10**.
New authored QA Python: **134 lines** (`gate.py` 41, `freeze_evidence.py` 93);
new nonmonetary digest manifest: **11 lines**. Production and Rust test changes:
**0 lines**. [Scope and per-file inventory](scope.json) separately enumerate
new prose, copied earlier auditors and generated receipts/logs, with hashes and
text line counts. All edits stay inside the assigned QA prefix.
Whitespace and final scope checks pass; no broad formatter, commit or push.

One precision correction to the brief: both preserved empty pages request
**2026-08-10**, with the same UTC admission interval. “Different days” is wrong
if it means their requested UTC day; actual wall-clock capture dates are not
established here. **Round-69 correction:** the earlier claim “They remain
distinct captures/attempt populations” is retracted. Both selected JSON files
and compressed viewport files are byte-identical, including the read time and
admission interval. The pages are indistinguishable and bind neither identical
nor distinct attempt populations or resolved roots. The separation rests on the
store read-backs alone; round 62's amount must not be imported into round 61.
See the corrected `capture-bindings.json` note for exact hashes and limits.

This internal QA revision changes no user-facing product path; no new
functional handoff is claimed. Fable retains the internal-stage N/A decision
and later independent code-blind PF-60-S03 design/execution/evidence review.
Live-repository/platform qualification, human acceptance, benchmark/release
qualification, the surviving P2 and every inherited item remain open. The
feature-lane timeout is an additional disclosed gate failure, not an approval.
