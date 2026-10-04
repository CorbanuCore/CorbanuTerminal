# RETURN — acct-inherited-64

Allocation digest **4dabf015e35d6e4430392badafd41d4ab1d83d295c8894978f3736f479410891**;
claim **3b6e073c-0452-424b-8176-b58c60751f60**; **gpt-6-astra / high**.
Brief SHA-256 verified:
**7ea041cd237dd36f18d3141297702fb9762adab5f667d98445c61250bc4aceec**.
Base/tested source: **49fac26d01fdc172d6300fd72cad8037d0c6e351**.

**Both P3 corrections completed. No inherited item qualified: all four lack
specific collection inputs. Required Rust gates pass 342/342 lane executions.**
This is routine QA tooling/evidence work with zero production changes. Product
context is the exact heading **Product measurement**, excerpt “No commercial
performance numbers have been supplied.” PF-60 remains the active plan and
PF-60-S03 remains in_progress; the manager owns their records. No new feature,
acceptance, release or sprint closure is claimed.

## Corrected records and failing counterexamples

1. [Round-61 RETURN](../acct-readers-61/RETURN.md) now records its never-prompted
   zero as a surviving **P2 scope/coverage misread**, including correction of
   the reader-verdict table. Round 66 corrects that amendment: round 61's
   fresh-session zero establishes no day-wide amount. The separate round-62
   capture supplies the four-priced-attempt reproduction; its amount cannot
   be attached to round 61's control.
2. [Reader audit](../acct-readers-61/audit_readers.py) now binds all **24 exact
   page identities** before writing a receipt: nine priced, four unknown-cost,
   two zero-recorded and nine nonmonetary pages. Every required subtotal is
   checked; navigation/refusal pages reject numeric dollar and both USD-prefix
   and USD-suffix amounts even without the exact-USD line. Missing or additional
   page identities reject. [Round-62 RETURN](../acct-scope-62/RETURN.md) has a
   dated-round correction beside its original coverage claim; original raw
   captures and receipts remain unchanged.

[38 controls](page-controls-01/results.json) inject three monetary spellings
on every nonmonetary page (27), remove each such page (9), and add an unexpected
navigation or amount page (2). **All 38 reject before success receipt; the
actually executed frozen base auditor accepts 34.** Four old rejections remain
visible. [28 original removal controls](removal-controls-01/results.json)
still reject all exact/displayed omissions. Baseline remains **9 priced page
reconciliations, 4 unknown-cost pages, 2 scoped-zero pages**; these are views,
not 15 distinct priced attempts. The mutants are QA copies, not product output.

**Round-66 correction:** the checker above used only three currency patterns.
The [new replay](../acct-receipt-66/page-controls-01/results.json) adds 36 amounts
outside that key set, asserts the original 34 acceptances separately, and binds
complete nonmonetary-page content. All 74 revised cases reject; both earlier
auditors accept all 36 new cases. The 38-case numbers above remain historical.

## Inherited qualification decision

**Chosen item: none.** Gateway economics lacks admitted gateway attempts and
a bound economic source; prewarm is explicitly excluded from observations;
auxiliary compaction is exercised but leaves the sampling records unchanged;
legacy display replay and the accepted compact importer do not acquire
authoritative legacy request evidence. Consequently, none of these four can
be qualified to the reader standard from this collection. The
[full disposition](inherited-disposition.md) names each missing input and cites
[13 exact hashed source excerpts](source-evidence.json), current passing
exclusion tests, and the accepted import receipt. No item is silently discharged.

The preserved direct-sampling collection independently reconciles:
**(80×5 + 20×0.5 + 10×30)/1,000,000 = USD 0.00071 per attempt;
four attempts = USD 0.00284.** [Re-audit](scope-recheck.json) binds the stored
evidence and seven priced pages. The actual round-62 fresh-root text includes “No recorded
attempts in this day; collection coverage unknown.”, “Collection coverage:
unknown; recorded root and resolved descendants only. Unknown parent population
excluded.” and “Known subtotal exact USD: 0”. See the corrected complete
transcription in the disposition. This is the surviving P2, not proof of zero
inherited cost or an amount belonging to round 61's capture.

**Unproven:** all four inherited workflows, complete application coverage,
two-collected-provider UI reconciliation, positive measured/billed comparison,
independent code-blind acceptance, native/platform/live-repository qualification
and human acceptance. This routine audit-only change adds no user-facing path;
a new functional handoff is not made. Applicable PF-60-S03 functional gates
remain with Fable. No inherited arithmetic is fabricated from missing inputs.

## Exact gates

From codex-rs, prerequisites built first, shared dedicated target
`acct-activation-33/feature/target`, `NEXTEST_TEST_THREADS=4`,
`CARGO_BUILD_JOBS=2`, guarded tests:

| Lane | Run / passed | Failed / timed out | Exact failure names |
| --- | ---: | ---: | --- |
| `just test -p codex-core accounting` | 124 / 124 | 0 / 0 | None |
| `just test -p codex-core accounting --features codex-core/developer-accounting` | 127 / 127 | 0 / 0 | None |
| `just test -p codex-tui usage` | 91 / 91 | 0 / 0 | None |

[Results](test-results.json), [ordered commands/exits](gates-01/lanes.json).
**342 executions passed**, with overlap between default/feature lanes; not
342 distinct test names. Both prerequisite and final feature builds exit 0.
One slow passing WS auxiliary case appears in each Core lane. No native
credential prompt observed. No raw cargo tests, live profile, formatter, push
or commit. The exact built binary's [hash](binary.json) matches preserved
round-61/62 captures; no new PTY run is claimed.

## Changed lines and brief accuracy

[scope.json](scope.json) records exact tracked additions/deletions and new QA
source counts; [inventory.json](inventory.json) hashes the deliverable artifacts.
All changes are within the allocated QA prefix. Production/Rust test lines: **0**.
Two evidence-export attempts had out-of-range excerpt bounds and stopped before
writing receipts; both are preserved in [attempts.json](attempts.json).

No factual error found in the brief's two P3 requests or its transfer of the
inherited block. Its “qualify one” path cannot be fulfilled with these inputs;
the explicitly allowed “none” branch applies. “Reader-facing half qualified”
is treated as the manager's bounded evidence disposition, not a claim that the
surviving P2 is fixed or all product/functional acceptance gates are passed.
