# RETURN — acct-scope-62

Allocation digest **f7a74cc9debb78a92e2819f2d132b8d7bb87e16192e66074810ce0c0587862b8**;
claim **4e985c4f-1b1d-4f84-a029-0451f9fc8cc6**; **gpt-6-astra / high**.
Brief SHA-256 matched **1967a569a40666a0a7ac98f116c405b825694d621aa919c347fff39f76329e03**.
Assigned/tested base: **defa10a2dd00b6d4f8b91d54978b480c75a2b0e4**.
Candidate: **corbanu 0.1.42**, debug developer-accounting,
SHA-256 **f78a1cc8a433030d3f50d91cfd415946e5fe3dd4e08e69d5c9644244c0d018bb**.

**P2 scope-zero deliberately reproduced; audit coverage bound; counts corrected;
unevaluated TZ old-check claim removed; both residuals have exact prerequisites.
Rust gates: 342/342.** This is routine QA tooling/evidence work; production code
is unchanged. It is not product acceptance or an assertion that PF-60-S03 is done.

## Scope-zero finding and the four attempts

In the same synthetic store, four actual collected/priced requests on August 10
across three roots independently total **USD 0.00284**. A fresh session asking
`/usage requests 2026-08-10` renders exactly:

> No recorded attempts in this day; collection coverage unknown.
> Known subtotal exact USD: 0
> 90-day wall-clock detail cutoff: Some(1778587200000); aggregate day floor at checkpoint: 20311; oldest recorded day: None

| Attempt | Root | Exact estimated USD |
| --- | --- | ---: |
| cd95c387-e1c2-46ec-99fe-65ad38c17b2c | A | 0.00071 |
| de749048-449c-49c8-86fc-dc4f478663e7 | A | 0.00071 |
| 05a7c6e3-9589-475f-8481-dcc40695f8f2 | B | 0.00071 |
| 89ee3e05-2ad8-4b89-b38c-470e72432863 | C | 0.00071 |

Each is OpenAI/Sol, admitted at 2026-08-10T12:00:00Z, with reported input 100,
cache read 20, output 10: **80×5/1M + 20×0.5/1M + 10×30/1M = USD 0.00071**.
Before/during store readbacks match. Reopening A/B/C renders 0.00142/0.00071/0.00071;
all four attempt details reconcile too. [Seven-page independent audit](scope-audit.json).

**Verdict: P2 scope/coverage misread.** The title and generic scope qualifier
name root/descendants, but the primary empty-state and oldest-day conclusions can
be read as covering all sessions on the requested day. The zero is valid only
for the fresh root. It must not be presented as a clean day-wide zero control.

**Replacement:** “No recorded attempts for this session and its resolved
descendants on 2026-08-10. Other sessions are excluded. This is not your total
spend for the day.” Use “Known subtotal for this session: USD 0” and “Oldest
recorded day in this session: none”; retain collection/billing unknowns.

[Full finding, exact text, four thread identities and original-count correction](reader-findings.md),
[before store](scope-run-01/store-before-fresh-reader.json),
[during store](scope-run-01/store-during-fresh-reader.json),
[raw actual-key attempt](scope-run-01/fresh-scope-reader.raw.gz),
[keys](scope-run-01/keys.json). No product code changed.

## Bound coverage and corrected monetary counts

`acct-readers-61/audit_readers.py` freezes exact page-name sets before evaluating
amounts or writing a receipt: **9 priced + 4 unknown-cost + 2 zero-recorded = 15
subtotal-bearing pages**. Missing or unexpected numeric pages fail. Each expected
priced/unknown page must also contain exactly one displayed total. The counts
mean repeated page views, not 9 unique attempts or billed-dollar comparisons.

[Final baseline](coverage-run-03/baseline.json) reports those separate counts and
wording. [28 controls](coverage-run-03/results.json) remove each of 15 exact
subtotals and each of 13 displayed totals, keeping associated reader-case rows
consistent. The revised audit rejects all 28 before writing any success receipt.
The actually executed pre-revision auditor accepts **26**; its two special-case
narrow-overview checks already reject. This is a genuine discrimination test,
not an assumed `old_check_passes` field. The preliminary run that incorrectly
expected the old narrow check to pass remains recorded in [attempts.json](attempts.json).

The new scope reproduction separately adds **7 priced pages and 1 scoped-zero
page**, all audited; it is not folded into the original 15-page count. **Positive
measured-dollar comparisons remain 0.** Historical receipts/captures are unchanged;
the old report's misleading “15 monetary pages” wording now points to the re-audit.

**Round-64 correction:** the exact-USD predicate above did not reject every
unexpected numeric page: another dollar/USD amount could appear on a navigation
page without that line. The current audit binds all **24 page identities**:
15 subtotal pages and nine pages that forbid numeric money. Missing/unexpected
identities fail before receipt. [Round-64 controls](../acct-inherited-64/page-controls-01/results.json)
reject 38 altered-page cases (34 accepted by this round's auditor); all 28
original amount-removal cases still reject. Original receipts remain historical.

## TZ control

The brief permits evaluating the old predicate **or omitting its field**. This
revision chooses omission for TZ; `negative_controls.py` no longer supplies a
blanket `old_check_passes=true` default. A fresh invocation of the same built CLI,
PID **75037**, reports TZ empty, epoch **1773144000**, local
**2026-03-10T05:00:00-0700**. The process-bound assertion expecting New York rejects
it. The six negative cases pass; four preserved round-61 positive process
controls are checked, not represented as fresh executions.
[TZ receipt and exact rejection](p3-negative-controls.json).

## Two residuals

- **Two collected providers: blocked in this exact loopback CLI rig.** The
  developer selector collects built-in OpenAI and Anthropic only; custom
  `reader_local` is disabled. Top-level built-in Anthropic configuration ignores
  a base-URL override, retaining api.anthropic.com; the existing OS boundary
  permits loopback only. The earlier genuine Anthropic attempt timed out without
  reaching the fixture. A supported, properly admitted Anthropic fixture endpoint
  and newly qualified binary, or a separately authorized isolated live-provider
  lane, is required. Two OpenAI models/transports are not two providers.
- **Positive measured-dollar comparison: blocked by missing settlement support.**
  The inspector consumes token observations and priced estimates, has no settled
  dollar field, and always renders billed amount/difference unavailable. It needs
  an authorized settlement-input/binding/storage contract and UI comparison,
  followed by nonzero measured-dollar tests. Restored estimates are not bills.

[Detailed limits and required changes](residuals.md),
[frozen-source excerpts/hashes](residual-source-evidence.json). Neither residual
is marked passed, waived or inherently impossible in future implementations.

## Exact gates and changed lines

Prerequisites built first; all commands executed from `codex-rs`, shared dedicated
`acct-activation-33/feature/target`, `NEXTEST_TEST_THREADS=4`, guarded `just test`:

| Lane | Command | Run / passed | Exact failures |
| --- | --- | ---: | --- |
| Default core | `just test -p codex-core accounting` | 124 / 124 | None |
| Developer core | `just test -p codex-core accounting --features codex-core/developer-accounting` | 127 / 127 | None |
| TUI | `just test -p codex-tui usage` | 91 / 91 | None |

[Exact summaries](test-results.json), [ordered commands/exits](gates-01/lanes.json).
Total **342 run, 342 passed, 0 failed/timed out**. Prerequisite and final feature
binary builds also exit 0. No native credential prompt was observed.

Tracked edits: audit **+35/−4**, negative controls **+10/−5**, prior RETURN
**+5/−3**, prior reader findings **+6/−1**: **+56/−13**. New QA Python: **370
lines** (scope driver 93, scope audit 104, coverage controls 83, finalizer 90).
New reports and machine/raw evidence are separately enumerated in
[scope.json](scope.json) and [inventory.json](inventory.json). Product/Rust-test
lines changed: **0**. All changes remain under the allocated QA directory;
`git diff --check` passes. No commit, push, broad formatter or approval fabricated.

## What the brief got wrong

The original round-61 store has four attempts across **two days**; only three are
on August 10, and only one of those has both price and usage. Its fourth
August-10 emission is an uncollected custom provider. Also, “every” monetary check
being gated is too broad: the narrow overview already had explicit positive
amount assertions, as the old-auditor counterexamples prove. The primary P2 and
unbound general coverage findings remain valid. Full scope wording exists in the
old title/qualifier; its placement does not cure the misleading primary zero.
