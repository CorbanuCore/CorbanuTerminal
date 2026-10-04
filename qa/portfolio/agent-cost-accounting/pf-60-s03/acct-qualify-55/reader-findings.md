# Reader findings retained from acct-qualify-50

These are interpretations of preserved rendered output, not production fixes.
No amount below is newly billed, and no financial action was performed.

| Finding | Could mislead about amount or scope? | Verdict |
| --- | --- | --- |
| “Range too large” on an empty day refused because unrelated store rows exhaust the reader budget | **Scope.** It identifies the chosen range as the problem, although shrinking that range cannot necessarily admit the read. It does not display a false number. | Explanatory defect, not merely polish. Expose the actual work-budget cause and a useful next step. |
| Two “Started /root/qualify_child” messages for one child and one child HTTP response | **Amount and scope by implication.** A reader can infer two workers or two chargeable executions. The inspected descendant amount is correct; this is not evidence of double charging. | Activity-reporting defect, not merely polish. |
| Raw epoch milliseconds / epoch-day integers in freshness and retention | **Scope and validity.** A reader may mistake the covered date, cutoff, or snapshot age and apply the amount to another period. Explicit units make the numbers technically interpretable but do not make this only cosmetic. | Date/coverage presentation defect; no demonstrated arithmetic defect. |
| Some(...) / None in retained cutoff and oldest-aggregate facts | **Scope.** None can be read as no retention constraint, no data, or unavailable evidence. These meanings imply different population completeness. Some(...) also makes the time hard to read. | Missing-value explanation defect plus presentation polish; the ambiguity about coverage is substantive. |
| Unknown-parent overview rounds cost; its exact value requires opening the attempt | **No false scope/amount claim observed.** It is explicitly excluded and labelled rounded; exact amount is reachable. | Navigation/presentation polish for this fixture, not an arithmetic defect. |
| “Snapshot is not current” with a 624 ms maintenance gap | It can suggest the inspection failed, but it accurately discloses uncertainty and does not claim a different covered population or exact amount. | Conservative status wording/polish on the recorded evidence; no false-amount finding established. |
| Rounded Terra components sum to 0.000267 while subtotal rounds to 0.000268 | A casual reader may perceive disagreement, but rounded labels and exact values disclose the operation correctly. Exact sum is 0.0002675. | Rounding-reading caveat, not a defect on this evidence. |
| Billed cost and estimate-versus-billed difference unavailable | No: the explicit labels prevent treating estimates as settlement. | Honest limitation, not a rendering defect. |

Original captures: ../acct-qualify-50/limits-01/rows-571429-day-2026-08-08-selected.json;
../acct-qualify-50/collect-04/root-and-child-complete.txt.gz;
../acct-qualify-50/inspect-01/hour-overview-selected.json;
../acct-qualify-50/inspect-01/original-price-request-1-attempt-1-selected.json.
