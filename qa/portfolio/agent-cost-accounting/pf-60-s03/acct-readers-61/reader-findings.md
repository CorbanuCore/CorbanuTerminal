# Reader states — acct-readers-61

This is code-aware developer evidence from the exact debug developer-accounting
candidate, not an independent acceptance pass. All amounts below are **estimated
token cost**, never a provider bill. No settlement evidence exists in these
fixtures. Full viewport captures, selected rows, native request/attempt links and
keys accompany each named page in reader-run-05.

The fixed independent Sol tariff is input 5, cache-read 0.5, output 30 USD per
million tokens. Each priced sample emits inclusive input 100, cache-read 20,
cache-write 0, output 10 and reasoning 4. Therefore:

- Noncached input: (100 − 20) × 5 / 1,000,000 = **0.0004 USD**.
- Cache read: 20 × 0.5 / 1,000,000 = **0.00001 USD**.
- Output: 10 × 30 / 1,000,000 = **0.0003 USD**.
- Cache write: reported 0 contributes **0 USD**.
- Sum: **0.00071 USD**, rendered **$0.000710**. Reasoning is a subset of output;
  total tokens 110 is not separately billed.

The audit checks all captured numeric pages, component charges, six-decimal
displays, price rates and all seven token measures. Persisted snapshots are
cross-checked against those fixed inputs; stored subtotals never set expectations.

## 1. Historical sessions remain inspectable

Inputs: sample a native session on August 8, sample again on August 10, stop and
resume that same native thread, then type `/usage requests 2026-08-08`. Open
Request 1 and Attempt 1 with real keys.

Exact rendered text:

> Requested UTC day: 2026-08-08  
> Estimated token cost for recorded attempts: $0.000710  
> Known subtotal exact USD: 0.00071  
> Collection coverage: unknown; recorded root and resolved descendants only. Unknown parent population excluded.  
> Billed cost: unavailable — no settlement evidence  
> Logical requests may have attempts on other days; this UTC day is not their complete lifetime.

Attempt detail renders the four component charges above and the original
dispatch-time price. The later August 10 request is not included.

**Verdict:** the observed retained historical day is inspectable and reconciles.
Estimate versus measurement is explicit. “Known subtotal exact” means exact
decimal arithmetic, not a measured bill; copying that line alone would lose the
qualification. Day scope is stated, although epoch-ms and Some/None diagnostics
still burden the reader. This does not claim all historical retention states are
inspectable: compact-history refusals remain in the boundary evidence.

Evidence: [historical](reader-run-05/historical-selected.json),
[attempt](reader-run-05/historical-Request-1-Attempt-1-selected.json).

## 2. Missing-price and unavailable-backend states

### Missing price

Inputs: make one actual OpenAI-wire fixture request for gpt-6-astra, whose
dispatch-time accounting price is absent. Usage is present: 100 input, 20 cached,
10 output. No price row is fabricated.

Exact rendered text:

> Estimated token cost: unknown  
> Known estimated token cost: $0.000000 + unknown costs  
> Full recorded estimate: unavailable (1 of 1 attempts incomplete)  
> Known subtotal exact USD: 0

Attempt detail:

> Noncached input cost: unknown — rate unavailable  
> Cache read cost: unknown — rate unavailable  
> Cache write cost: $0.000000; exact USD 0  
> Output cost: unknown — rate unavailable  
> Price: unavailable — no dispatch-time price snapshot  
> Billed cost: unavailable — no settlement evidence

Visible actions are **Back**, **Refresh**, **Close**; there is no actionable
explanation of how to obtain or reconcile the missing price.

Independent arithmetic: no known positive price component exists. The known
subtotal is 0; the full cost is **unknown**, not 0. Reported zero cache-write is
the only zero-valued component. The Sol tariff must not be applied to Astra.

**Verdict:** the leading lines clearly deny a complete estimate, but the isolated
“Known subtotal exact USD: 0” line can be read as zero cost. The user-visible
next-step requirement is not satisfied by an unexplained Refresh button.

Suggested text: **“Cost unknown: this request has no dispatch-time price.
The zero known subtotal is not a zero-cost estimate. Compare this request with
your provider’s usage or billing record; Refresh only rereads saved evidence.”**

Evidence: [overview](reader-run-05/missing-price-selected.json),
[attempt](reader-run-05/missing-price-Request-1-Attempt-1-selected.json).

### Unavailable backend

An initial `sqlite=false` fixture did not close an already available native
database: reader-run-02/unavailable-backend rendered a valid $0.000710 estimate.
That is a failed fixture premise, not unavailable-backend evidence.

The final case temporarily renames the synthetic backend's estimate table while
the TUI is running. Rows, token observations and monetary values are unchanged.
The inspector cannot read the table. Refresh is pressed while that fault is
still active; the TUI is then stopped and the exact pre-fault SQLite backup is
restored for a fresh-process control. This is an injected
backend **read failure**, not proof of an absent database, remote backend or
spontaneous corruption.

Exact failure text:

> Unavailable — accounting evidence is corrupt, incompatible or could not be read. Refresh to retry; no repair performed.  
> Collection coverage: unknown; recorded root and resolved descendants only. Unknown parent population excluded.  
> Billed cost: unavailable — no settlement evidence

The requested UTC day remains August 8. Visible actions: **Refresh**, **Close**.
There is no monetary amount to reconcile in the failure state. The control
amount, when readable, is the independently calculated **0.00071 USD**.

**Verdict:** unavailable is not presented as blank or zero; the page gives an
explicit next step. Refresh correctly keeps refusing while the fault is active.
The exact fixture restore permits a fresh process to show the original estimate.
This does not prove user repair or automatic recovery from real corruption.
The [retry result](reader-run-05/backend-refresh-result.json) and
[fresh-process output](reader-run-05/backend-reopened-selected.json) keep those
claims separate.

Earlier runs 03 and 04 renamed the table back and still failed Refresh/reopen.
SQLite had changed the table's stored schema text by quoting its name; that
cleanup was not an exact pre-fault restore. Those attempts are preserved as
fixture failures, not attributed as a product recovery defect. The final control
uses a backup instead. No repair instructions are inferred for real user data.

Evidence: [failure](reader-run-05/unavailable-backend-selected.json),
[table fault](reader-run-05/backend-fault.json),
[restoration](reader-run-05/backend-restored.json).

## 3. Narrow screen

The same retained historical session is inspected after a real PTY resize to
**64 columns × 24 rows**. Home, down-arrow, Enter and Escape traverse the overview,
request and attempt pages; all wrapped lines remain navigable.

Exact selected rows include:

> Estimated token cost for recorded attempts: $0.000710  
> Billed cost: unavailable — no settlement evidence  
> Known subtotal exact USD: 0.00071

The scope qualifier wraps as:

> Logical requests may have attempts on other days; this UTC  
> day is not their complete lifetime.

**Verdict:** at this tested width the amount and estimate/billing/scope qualifiers
are preserved, with no observed truncation or arithmetic change. Scrolling is
necessary. This is evidence for 64×24, not all narrow terminal dimensions.
The same **0.00071 USD** arithmetic applies.

Evidence: [overview rows](reader-run-05/narrow-selected.json),
[attempt rows](reader-run-05/narrow-Request-1-Attempt-1-selected.json),
[viewport capture](reader-run-05/narrow.txt.gz).

## 4. Mixed-provider session

The same native root session successfully makes an OpenAI/Sol request and a
custom `reader_local/reader-local-model` request on August 10 through the local
fixture. Both emit 100 input, 20 cached, 10 output. The custom Responses endpoint
is supported, but this developer-only accounting selector collects only the
eligible built-in provider lanes. No custom-provider accounting row is present.

Exact rendered text:

> Estimated token cost for recorded attempts: $0.000710  
> Known subtotal exact USD: 0.00071  
> Collection coverage: unknown; recorded root and resolved descendants only. Unknown parent population excluded.  
> Root total = own attempts + resolved descendant attempts. Provider/model groups partition the same root total. Compare exact USD, not rounded displays.  
> Provider: openai; Model: gpt-5.6-sol

Only **Request 1** and the OpenAI provider group appear. There is no explicit
notice that the successful custom-provider request was uncollected.

Independent arithmetic: the OpenAI component is **0.00071 USD**. No custom
provider tariff was supplied; its cost and therefore the complete mixed-provider
cost are **unknown**. The inspector's number is correct for its recorded subset;
it does not reconcile the whole two-provider day.

**Verdict:** a reader could mistake this for the complete run/day amount. The
generic collection-coverage warning mitigates but does not identify the omitted
provider/request. Estimates remain labelled; the risk is **scope/coverage**, not
a numerical error in the recorded component.

Suggested text: **“Known cost of collected requests: $0.000710. This is not a
complete run total. Some providers may not be collected; reconcile their usage
separately.”** A provider-specific omission count would require retained evidence
and is not proposed as already available.

A separate Anthropic attempt in reader-run-01 failed: its built-in endpoint
override was ignored, and the loopback-only sandbox denied external transport.
No request reached the fixture; no actual Anthropic monetary amount or successful
OpenAI+Anthropic reconciliation is claimed. The frozen binary's supported config
override path cannot provide that additional positive control here. No denial was
bypassed, product patched, or stored provider renamed to manufacture coverage.

Evidence: [emissions](reader-run-05/emissions.json),
[overview](reader-run-05/mixed-provider-selected.json),
[OpenAI group](reader-run-05/mixed-provider-Provider-openai-selected.json),
[native bindings](reader-run-05/native-bindings.json).

## 5. No usage at all

One native session receives a completed fixture response **without a usage
object**. Its handler's numeric constants are not sent and cannot be treated as
measurements. The inspector renders:

> Estimated token cost: unknown  
> Known estimated token cost: $0.000000 + unknown costs  
> Full recorded estimate: unavailable (1 of 1 attempts incomplete)  
> Known subtotal exact USD: 0  
> Input: 0 known + unknown in 1 attempts  
> Total (not separately billed): 0 known + unknown in 1 attempts

All seven measures have one unknown attempt. Attempt detail says:

> Input: unknown — no retained numeric evidence  
> Output cost: unknown — no retained numeric evidence

Independent arithmetic: an empty known contribution sum is 0; the amount is
**unknown**, even though a Sol tariff exists. No usage means there is nothing
to multiply. This differs from a never-prompted fresh session and a requested
empty day, both actually rendered as:

> No recorded attempts in this day; collection coverage unknown.  
> Known subtotal exact USD: 0

Those pages contain no Request link and zero known/unknown counts **within the
selected root**. This round's never-prompted capture is a fresh-session control;
it establishes no day-wide amount. The P2 scope/coverage wording concern remains
open. The [separate round-62 reproduction](../acct-scope-62/reader-findings.md)
provides its own other-root population and arithmetic, not values measured by
this capture. Both captures request August 10; they are distinct executions.
The original captures remain unchanged.

**Verdict:** no-usage is distinguished from no recorded attempts. Neither is a
measured zero bill. The standalone zero subtotal can still mislead; suggested
text is **“No usage was reported for this request. Cost cannot be estimated;
a zero known subtotal does not mean the request was free.”**

Evidence: [no-usage](reader-run-05/no-usage-selected.json),
[attempt](reader-run-05/no-usage-Request-1-Attempt-1-selected.json),
[never prompted](reader-run-05/never-prompted-selected.json),
[zero-attempt day](reader-run-05/zero-attempt-day-selected.json).

## 6. Stale estimate versus measured amount

For a previously priced historical request, the fixture changes only its
contribution evidence marker to `[]`, preserving the original marker and stored
amount. The exact stale row is read back before inspection. It renders:

> Recorded totals unavailable — stored contributions need refresh. Retry rereads only; no repair performed.  
> Collection coverage: unknown; recorded root and resolved descendants only. Unknown parent population excluded.  
> Billed cost: unavailable — no settlement evidence

No dollar amount or exact subtotal appears. Refresh leaves the injected evidence
unchanged. Restoring the original marker is a disclosed **fixture cleanup and
positive control**, not a supported user repair: a fresh inspector again renders
the independently calculated **0.00071 USD** estimate.

**Verdict:** the stale estimate cannot be mistaken for a measured amount on this
page because the amount is withheld and billing absence is explicit. “Need
refresh” next to “Retry rereads only” is confusing: the offered action does not
rebuild the stale contribution. Suggested text: **“Stored cost evidence is stale;
no amount is shown. Refresh only checks for updated evidence and does not repair
it. Report the issue if it persists.”**

Evidence: [stale page](reader-run-05/stale-estimate-selected.json),
[Refresh](reader-run-05/stale-refresh-selected.json),
[before](reader-run-05/stale-before.json),
[injected](reader-run-05/stale-injected.json),
[after Refresh](reader-run-05/stale-after-refresh.json),
[restored control](reader-run-05/stale-restored-control-selected.json).
