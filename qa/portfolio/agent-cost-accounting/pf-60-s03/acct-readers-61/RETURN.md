# RETURN — acct-readers-61

Allocation digest 87ab2378867e1154c110ec0aeb0345bcda08ffae469ce047b319b0fbd2f5d311;
claim 00e56549-2eaf-4220-b0d4-9a63ac5b4a5e; gpt-6-astra/high.
Brief SHA-256 verified:
18976e97928f0acd366ee8f5c7e7ac3c4269490ccdf88f913fe9488a53ed5c81.
Base/tested source: **90095a359599f055653d39eb3e6aa9831faeaaef**.
Candidate: **Corbanu Terminal 0.1.42**, debug developer-accounting CLI,
SHA-256 **f78a1cc8a433030d3f50d91cfd415946e5fe3dd4e08e69d5c9644244c0d018bb**.

**Three P3 checks corrected; 26 boundary cases, six rejecting controls and four
positive product-process timezone controls pass. All six reader categories have
actual-key evidence and explicit verdicts. Round-62 correction: **9 priced page
reconciliations, 4 unknown-cost pages and 2 zero-recorded pages**, not 15 priced
reconciliations. The [bound re-audit](../acct-scope-62/coverage-run-03/baseline.json)
supersedes the old receipt’s undifferentiated count. Rust gates pass 342/342.** Missing-price next steps and mixed-provider
completeness remain gaps; this is not a declaration that the reader-facing
requirements or sprint are complete.

**Round-64 receipt correction:** the never-prompted zero is a **P2
scope/coverage misread**, not a clean day-wide zero control. Round 62 reproduced
four priced attempts across three other roots on August 10, independently
totaling USD 0.00284, while the fresh root rendered “No recorded attempts in
this day; collection coverage unknown.” and “Known subtotal exact USD: 0”.
That zero describes only this session and its resolved descendants. The finding
survives this receipt; no product fix or approval is claimed.
[Reproduction and proposed wording](../acct-scope-62/reader-findings.md).

[Preflight/classification](PREFLIGHT.md), [reproduction](REPRODUCE.md),
[full reader text, arithmetic and verdicts](reader-findings.md).

## Three P3 fixes and new discrimination

1. **Whole-day unavailable response is bound to the requested day.**
   The driver requires the rendered `Requested UTC day: YYYY-MM-DD` header for
   every day. The independent audit reads the actual saved viewport, including
   that header, for both unavailable whole-day cases: **2026-11-01** mixed
   history and **2026-03-08** deleted history after retention. Replacing either
   header with November 2, or removing it, now fails. All four altered responses
   retain the compact-history explanation and absence of a subtotal, so the old
   assertions would have accepted them.

2. **Mixed compact hour positively asserts its rendered explanation.**
   Driver and independent audit now require the normalized exact sentence:
   **“Precision unsupported outside retained raw detail; compacted days lost
   request/provider attribution. No bucket total.”** A counterexample replacing
   that explanation with “History unavailable.” still has no subtotal and passes
   the old negative-only condition; the new assertion rejects it.

3. **Timezone family has a positive control inside the actual product process.**
   A QA-only DYLD clock constructor calls `tzset/localtime_r` inside each
   inspector PID. The driver binds its output to the exact launched PID and
   checks TZ, epoch and local timestamp independently with ZoneInfo. At the same
   March 10 12:00Z clock, the four processes report:
   **New York 08:00−0400, UTC 12:00+0000, Kolkata 17:30+0530,
   Phoenix 05:00−0700**. Their UTC March 8 bucket memberships/totals still agree.
   A deliberate invocation of the exact product with TZ removed is rejected by
   the same checker. This proves TZ reached and affected the process; it is not
   a Python-only wall-time conversion or a claim of local-time inspector output.
   Per-PID filenames prevent inherited child execution from replacing evidence.

[26-case independent audit](boundary-audit.json),
[6 counterexamples and 4 positives](p3-negative-controls.json),
[fresh boundary results](boundary-run-02/boundary-results.json),
[per-process controls](boundary-run-02/timezone-controls.json).
Historical captures and prior receipts were not overwritten or relabelled as
timezone-qualified.

## Reader-facing half

Each page also has adjacent selected JSON, lossless viewport/raw captures and
actual keys. [The detailed report](reader-findings.md) includes the exact text
and proposed replacement wording wherever the reader could be misled.

| State | Exact rendered text / arithmetic | Plain reader verdict |
| --- | --- | --- |
| Historical native session | “Requested UTC day: 2026-08-08”; “Estimated token cost for recorded attempts: $0.000710”; “Known subtotal exact USD: 0.00071”; “Billed cost: unavailable — no settlement evidence” | Reopen and request/attempt drill-down work. The retained day is correctly scoped and explicitly estimated, not billed. |
| Missing price / unavailable backend | Missing price: “Estimated token cost: unknown”; “Known estimated token cost: $0.000000 + unknown costs”; “Price: unavailable — no dispatch-time price snapshot”. Backend: “Unavailable — accounting evidence is corrupt, incompatible or could not be read. Refresh to retry; no repair performed.” | Missing-price total is unknown, not zero; isolated zero subtotal can mislead and there is no useful price-recovery next step. Backend failure withholds money and supplies an explicit retry action. |
| Narrow screen | 64×24 real resize retains “Estimated token cost for recorded attempts: $0.000710”, “Known subtotal exact USD: 0.00071” and “Billed cost: unavailable — no settlement evidence” | At this width, amount and qualifiers survive wrapping and request/attempt navigation. No new amount/scope ambiguity beyond the wide view. |
| Mixed providers | Actual OpenAI + reader_local requests in one root/day; page shows “Estimated token cost for recorded attempts: $0.000710”, “Provider: openai; Model: gpt-5.6-sol”, generic “Collection coverage: unknown” | Correct collected component, incomplete mixed-provider run. Custom request is absent with no specific omission warning: a reader could take the displayed amount for the whole day/run. |
| No usage at all | “Estimated token cost: unknown”; “Full recorded estimate: unavailable (1 of 1 attempts incomplete)”; “Input: 0 known + unknown in 1 attempts”; “Known subtotal exact USD: 0” | Missing usage is not measured zero. The overview warns clearly, but a copied zero subtotal can mislead. The never-prompted zero is now a **P2 scope/coverage misread**, independently reproduced in round 62; it does not prove a day-wide zero. The empty-day statement is scoped to this root. |
| Stale estimate | “Recorded totals unavailable — stored contributions need refresh. Retry rereads only; no repair performed.”; “Billed cost: unavailable — no settlement evidence” | No amount is shown, so this cannot be mistaken for a measured amount. “Need refresh” is confusing because the button only rereads; a clearer persistent-error next step is needed. |

Independent arithmetic for every priced Sol request:
**(100−20)×5/1,000,000 + 20×0.5/1,000,000 + 10×30/1,000,000 =
0.0004 + 0.00001 + 0.0003 = 0.00071 USD**.
Reported cache-write 0 contributes 0; reasoning 4 is an output subset, not another
charge. All seven token metrics are checked. The two priced OpenAI emissions
belong to different requested days, so each viewed day shows one component.

Astra's price is absent: the known sum 0 is not a zero tariff or complete cost.
The no-usage response contains no usage object, so handler constants must not be
used as measured tokens. The custom provider has no supplied tariff and is not
collected by this developer selector; the complete mixed-provider cost is unknown.

Backend failure is an explicit synthetic table-availability fault. Refresh while
the fault remains correctly refuses. After stopping the TUI and restoring an
exact pre-fault SQLite backup, a fresh inspector renders 0.00071 again. That is
fixture cleanup/control, not a proposed user database repair. The stale case
separately changes only the evidence marker to `[]`; readback proves Refresh
does not repair it. Restoring the original marker recovers the same estimate.

[Historical reader audit: 15 subtotal-bearing pages, only 9 priced](reader-audit.json),
[final reader cases](reader-run-05/reader-cases.json),
[emitted requests](reader-run-05/emissions.json),
[exact binary/driver manifest](reader-run-05/manifest.json).

## Required nonempty gates

Prerequisites built first; all commands ran from codex-rs with the same dedicated
CARGO_TARGET_DIR, NEXTEST_TEST_THREADS=4, CARGO_BUILD_JOBS=2 and guarded just test.

| Lane | Exit | Run | Passed | Failed/timed out | Exact failure names |
| --- | ---: | ---: | ---: | ---: | --- |
| `just test -p codex-core accounting` | 0 | 124 | 124 | 0 | None |
| `just test -p codex-core accounting --features codex-core/developer-accounting` | 0 | 127 | 127 | 0 | None |
| `just test -p codex-tui usage` | 0 | 91 | 91 | 0 | None |

**342 tests run, 342 passed, zero failed or timed out.**
Prerequisite and developer CLI builds both exit 0.
[Commands/exits](gates-01/lanes.json), [counts](test-results.json);
full lossless logs are in gates-01. Sprint checker: current 115, archived 127.
Final scope check and whitespace check pass. No full-parallelism run or paired
load attribution was attempted.

## Preserved failed and exploratory attempts

[All attempts](attempts.json) distinguishes process exit from product acceptance:

- Boundary run 01: PID mismatch catches a child overwriting the original probe.
  Run 02 is the fresh successful replay with per-PID files.
- Reader run 01: Anthropic transport targets its built-in external endpoint
  despite the configured override; the loopback sandbox denies it and the
  fixture times out. No Anthropic fixture emission or billed amount is claimed.
- Reader run 02 exits 0, but sqlite=false did not produce backend unavailability.
  Its “unavailable-backend” filename is an incorrect fixture expectation, not a
  passed backend-failure case.
- Reader runs 03/04 preserve failure after an inexact table-rename restoration.
  The stored schema text changed; no exact original-state restoration occurred.
  Final run 05 instead restores a pre-fault backup and completes.
- The first reader-audit attempt incorrectly required unknown monetary text on a
  request navigation page that has no amount. The corrected numeric-page
  predicate passes without changing any product output or capture.

## Changed lines and brief corrections

**0 production Rust lines, 0 Rust test lines. Existing QA source +27/−0
(boundaries.py +12, audit_boundaries.py +15); new QA source 544 lines**
(readers 245, reader audit 119, negative controls 68, evidence finalizer 62,
P3 checks 28, timezone clock/probe 22). Total QA source additions: **571**.
New prose, receipts and lossless captures are separately enumerated by size,
SHA-256 and text line count in [inventory](inventory.json).
[Scope and source counts](scope.json). All writes stay inside the assigned QA
prefix. No commit, push, broad formatter or product fix.

No error found in the three P3 requests. Two limits matter when interpreting the
reader assignment: this developer build does not collect arbitrary providers,
and it retains no positive billed/settlement amount to contrast with an estimate.
Successful two-collected-provider OpenAI+Anthropic reconciliation remains
unproven here; the real custom-provider case demonstrates incomplete coverage.
A numeric measured-dollar comparison cannot honestly be manufactured. The brief's
full-parallelism timeout observation remains untested and is not disputed.

Fable retains sprint reconciliation, independent code-blind qualification,
live-repository/platform coverage, product follow-up and acceptance. This return
does not claim those gates, human sign-off or shipment.
