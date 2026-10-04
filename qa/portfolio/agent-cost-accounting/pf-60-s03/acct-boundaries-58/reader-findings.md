# Reader findings — exact displayed text

These are evidence-backed reader assessments, not production fixes or approved
severity changes. New captures are from boundary-run-03; historical evidence is
explicitly labelled. Wrapped rows below preserve the rendered words.

## Empty versus unavailable

[Empty November 3](boundary-run-03/empty-day-selected.json):

> No recorded attempts in this day; collection coverage unknown.
>
> Known subtotal exact USD: 0

[Partial-day range](boundary-run-03/partial-days-overview-selected.json):

> Range total unavailable — partial or unavailable buckets excluded; no partial total.

Both [end days](boundary-run-03/partial-day-2026-10-31-selected.json) say:

> Partial bucket — excluded from totals

The [expired January 2025 range](boundary-run-03/expired-range-selected.json) says:

> Effective coverage (requested ∩ aggregate retention ∩ snapshot) for [2025-01-01T00:00:00.000Z, 2025-01-02T00:00:00.000Z): unavailable

Verdict: empty recorded population is distinguished from unavailable coverage.
Zero emitted fixture attempts in the expired range does not establish zero
historical expenditure. The refusal correctly supplies no monetary total.

## Mixed compact/raw history

[Whole-day refusal](boundary-run-03/mixed-day-selected.json):

> Request detail unavailable for this whole UTC day — compacted history lost request/provider attribution. No total shown. Store checkpoint: 1801288799000; aggregate day floor:
> 20484; oldest recorded day: Some(20520); 90-day wall-clock detail cutoff: Some(1793512799000) ms UTC.

[Compacted-hour explanation](boundary-run-03/mixed-compact-hour-selected.json):

> Precision unsupported outside retained raw detail; compacted days lost request/provider attribution. No bucket total. Whole UTC-day bounds offered: [2026-11-01T00:00:00.000Z,
> 2026-11-02T00:00:00.000Z); this does not restore attribution.

[Surviving raw hour](boundary-run-03/mixed-raw-hour-selected.json):

> Known subtotal exact USD: 0.0003675

Verdict: arithmetic is conservatively withheld, including the 0.0010975 whole-day
sum. The whole-day wording can suggest all drill-down is gone, although 06:00–07:00
remains available; the successful adjacent-hour case documents that scope nuance.
The suggestion of whole-day bounds expressly says it does not restore attribution.

## Epoch dates and Some/None

[New spring day](boundary-run-03/spring-America-New_York-day-selected.json):

> UTC admission interval: [1772928000000, 1773014400000) ms since Unix epoch
>
> 90-day wall-clock detail cutoff: Some(1765368000000); aggregate day floor at checkpoint: 20158; oldest recorded day: Some(20520)

[New spring range](boundary-run-03/spring-America-New_York-overview-selected.json):

> Oldest retained aggregate day (ledger): None; 90-day drill-down cutoff: Some(1765368000000) ms UTC (exclusive)

Verdict: scope/validity explanation findings remain. Readers must convert
milliseconds and day integers to know what dates an estimate covers. None
does not tell a reader whether there is no retained aggregate, no data, or no
retention constraint. Some(...) adds untranslated implementation syntax.
These are more than visual polish because they affect interpretation of coverage.

## Range too large — retained historical finding

[Round-50 empty-day budget refusal](../acct-qualify-50/limits-01/rows-571429-day-2026-08-08-selected.json):

> Range too large for this inspector. No total shown.

That preserved fixture crossed the whole-store work budget using unrelated rows.
The requested day was empty. Verdict: misleading cause/scope, because shrinking
the selected range need not permit the read. This round does not repeat the large
padding run and does not claim new threshold evidence. The actual 571,429-empty-day
and 571,428-one-attempt-day thresholds remain as previously recorded.

## Deleted history

[Before deletion](boundary-run-03/before-deletion-selected.json):

> Known subtotal exact USD: 0.00268

[After native child deletion](boundary-run-03/after-deletion-selected.json):

> Known subtotal exact USD: 0.002145
>
> Collection coverage: unknown; recorded root and resolved descendants only. Unknown parent population excluded.

Verdict: the exact decrease is correct, 0.000535. There is no deletion-specific
explanation on the post-deletion page; a reader comparing old screenshots must
understand that these are currently recorded totals, not immutable lifetime spend.
No false retained amount was observed. After normal compaction, even the remaining
0.002145 becomes unavailable in this request inspector.

## Timezone and DST

[Offset-input refusal](boundary-run-03/offset-input-result.json):

> Range refused: timestamps must use UTC Z

All tested range headers explicitly say “timezone: UTC”. UTC and date-only inputs
define UTC buckets regardless of process TZ. Local DST gaps and repeated hours
do not change UTC bucket boundaries. Offset-bearing local timestamps are refused,
not silently converted. This is clear for the tested input.
