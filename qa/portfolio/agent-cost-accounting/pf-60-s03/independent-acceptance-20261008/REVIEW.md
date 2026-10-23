# PF-60-S03 acceptance record: independent review

**Bottom line:** the record's conclusion holds, with corrections. I reran the cost arithmetic from the raw provider data and every reconciled total matches exactly. "Not accepted" is supported. "Three functional FAILs" is roughly right but needs fixing. The verdict table actually lists four FAIL rows, the no-usage test is set up in a way that blurs what it shows, and the clock-skew finding claims more than the evidence covers.

I only read files in this directory and changed nothing.

## Verdict-by-verdict

| # | Executor verdict | Review | Reason |
|---|---|---|---|
| 1 | PASS | **Agree, with a caveat** | My recomputed totals match every capture. Two things make totals harder to explain (finding 9): the attempt page says "Noncached input: unknown" but still charges a noncached cost, and request numbers are not in time order. |
| 2 | PASS | **Agree** | Root and descendant splits are correct for k1 and k3; rollout and provider usage match row for row. mac c3's root share matches the exec JSON. For mac conversation A, nothing kept in the evidence shows which requests the "independent" child total came from. It is the only 4-request subset that adds up, though (finding 7). b-day's 2/5 split was never checked against an independent source. |
| 3 | Mixed | **Agree** | No settlement data exists and Corbanu balance was not exercised, so NOT VERIFIABLE is correct. |
| 4 | PASS | **Agree, weak** | No capture from before the restart was kept. The PASS rests on `mac-01` (after restart) matching independent totals, which is enough. |
| 5 | PASS (P3) | **Agree, with caveats** | Start-time bucketing is proven by the day, month and week boundary requests. However, the hour range outside the detail window is accepted at range level and shows full coverage. It is refused only one level down (`mac-25`/`27`). The same page also says both "oldest daily total kept none" and "daily totals kept since 2025-10-09". For week and month, nothing shows the boundary request is missing from the next bucket, because those buckets reach the future and show as unavailable (finding 10). |
| 6 | PASS (P3) | **Agree, weak** | The "empty day" in `mac-10` (2026-10-07) is from before the store existed, yet the page claims retention back to 2025-10-09. A truly empty day with collection on (h-rtx2 10-02, 10-03 or 10-05) was available but not captured. |
| 7 | PASS | **Agree** | `mac-05` shows "no price available", "at least $…" and the zaiproxy next step. |
| 8 | FAIL | **Agree on the outcome; evidence partly confounded** | `zainousage` is a custom provider, so "no price available" is literally true there anyway. The defect the evidence does prove: the only next step on the no-usage conversation's own page names a different provider, zaiproxy (`mac-04`). The sprint record already lists this as open (finding 4). |
| 9 | FAIL | **Agree, but narrower** | `rtx-30`, `rtx-32` and `rtx-33` show "No other conversation recorded requests on this day" on days that had 7–18 real requests. That is false and is the defect. Removing deleted spend may be intended (the README itself calls it a product decision). The criterion has no cited source. |
| 10 | PASS (except after deletion) | **Agree** | `rtx-31` checks out: 7 requests, $0.012682. |
| 11 | PASS | **Agree; wording overclaims** | The product never shows provider response ids. Matching was by exact amounts per request, which I confirmed for conversation A, c3, k1, k3, k4 and b-day 10-06. "Provider ids reconcile 1:1" is not what was shown. |
| 12 | PASS | **Agree** | `mac-30` and `mac-03` support it. The capture method drops repeated lines (finding 8). |
| 13 | PASS (Linux only) | **Partly agree** | `/cost` is rejected in the default build. `/usage requests` still prints the developer syntax there. The command that was typed isn't captured, and a dated query was never tried. |
| 14 | PASS within retention | **Agree, weak** | "Historical" days are fake-clock rows only hours old. The 90-day refusal was only tested on an empty period. |
| 15 | FAIL | **Agree it is a real defect; scope overstated** | The evidence shows two failed turns. "Every model request" fails, and recovery once the clock catches up, were never tested. The failure is likely inherited fail-closed behaviour from S02. The criterion is the executor's inference, not a written S03 criterion (finding 3). |
| 16 | FAIL (outside S03) | **Disagree with FAIL** | The facts are right: usage shows zeros when unknown, and c3 reports 37,933 of 58,128 input tokens. But the record calls it outside S03 and likely inherited, so it should be an out-of-scope observation (finding 2). |
| 17 | PASS (executed) | **Agree** | Better stated as PASS for macOS and Linux, Windows not run. |
| 18 | NOT VERIFIABLE | **Agree** | — |

## Findings

1. **P2 – Failure, cancel and recovery paths were never inspected.** The sprint's Verification section requires "actual-key success, failure/cancel, recovery/resume". Two ready-made cases were left out:
   - `r-mid1` was killed after 1 request.
   - `r-skew60s` failed mid-turn.

   Neither was opened in `/cost`. These were also never exercised:
   - retry predecessor (always "none")
   - unknown-parent and unknown-attribution populations (always "none")
   - the busy-host refusal
   - stale-estimate and current-command

   The README doesn't list these as gaps.
2. **P2 – The FAIL count is inconsistent.** The headline says three FAILs; the table has four (8, 9, 15, 16). Row 16 should be an observation or out of scope.
3. **P2 – #287 is overstated and its origin isn't stated.** The evidence is one turn failure plus one month-backward failure. The 58 s figure is confirmed: r-day2's last usage arrived at 10:30:28.786 and the failure was at 10:29:30.149, about 58.6 s. Still:
   - Recovery after the clock passes the checkpoint was not tested, so "every model request fails" is not shown.
   - The error comes from `accounting_chat.rs` with "request stopped without a repair send". That looks like S02 fail-closed design and should be labelled likely pre-existing.
   - The Anthropic naming for a Z.AI request is real but P3.

   It is still a valid blocker for wider developer activation. Severity is P2 unless it is shown to persist.
4. **P2 – The no-usage test can't separate its two causes.** Usage was stripped on an already unpriced custom provider. The test cannot show the "missing usage vs missing price" label error. A no-usage request on a priced provider would be needed. The wrong-provider next step remains a real defect, and the sprint record already flags the label as open, so this FAIL was expected.
5. **P2 – The deletion FAIL should be framed around the false statement.** The false "No other conversation recorded requests on this day" breaks the acct-scope-62 intent and is a defect. Removing deleted spend is a product decision, which the README concedes. The README's headline wording ("erases its spend everywhere") presents the decision as the defect.
6. **P3 – Some "independent" sources are product output.**
   - Built-in zai usage comes from the product's own trace log. It was checked against the proxy on only 2 custom-provider responses.
   - Bucket start times use the product's own `corbanu_call_metrics` latency figure.
   - The k1/k3 root/descendant split uses session rollouts.

   All of this is disclosed and the raw provider chunks agree. It means "independent" is semi-independent.
7. **P3 – Conversation A's independent child total has no retained source.** The TUI log extract carries no thread ids. My subset search finds exactly one 4-request set worth 0.04031996, and its timing fits, so it is consistent but unproven.
8. **P3 – Captures are reconstructed, not verbatim.** `scrollcap.sh` drops repeated lines (`awk '!seen[$0]++'`). In `mac-30`, every repeated "(rounded)" continuation line vanished, and the capture stops at Request 4 of 8. The README doesn't disclose the dedup, and a duplicated row could be hidden.
9. **P3 – Wording and usability issues not in the README.**
   - The noncached-input line contradicts its own cost line (`mac-08`, `rtx-15`).
   - "Request 1…8" is not in time order: Request 8 is the first request.
   - Week and month "admitted 23:59:5x" times are the independent estimate, not product admission times.
10. **P3 – Boundary exclusivity is shown only for days.** Day buckets 7+8=15 prove no double count. The week and month boundary requests were never shown absent from W41 or October (a 10-01 day view would have done it).
11. **P3 – Small disclosure gaps.**
    - `mac-c5` (1 request at 11:28Z, in the totals) is not explained or listed in `threads-mac.txt`.
    - The TUI root thread `01a11b1d-ecc9…` is not listed either.
    - The verifier run output is not kept.
    - REVIEW.md is referenced but absent.
    - `tools/execrun.sh` puts the key in variable `K` on its own line rather than attached to the consuming command. Nothing leaked, but it's a policy deviation.
12. **Redaction is clean.** I found no Z.AI-format keys, no `/Users`, `/home` or `/Volumes` paths and no host IPs; 127.0.0.1 is only the local proxy. Paths use `<scratch>`, `<corbanu-root>`, `<rtx-host>` or `~`.

## My recomputed numbers

Prices are USD per 1M tokens (input 1.40, cached 0.26, output 4.40), using exact decimals.

| View | Requests | Tokens | Exact USD |
|---|---|---|---|
| mac conversation A total | 8 | 107,759 (prompt 107,113, cached 56,128, out 646) | 0.08881468 |
| …root / descendants | 4 / 4 | — | 0.04849472 / 0.04031996 |
| …request 8 (`20261008184519b0…`) | 1 | 14,188 in, 0 cached, 141 out | 0.0204836 |
| mac other conversations (probe, c1, c2, c3) | 10 | 121,450 | 0.09893564 |
| mac c3 | 5 | 58,410 | 0.04161648 |
| …root (3) / descendants (2) | — | — | 0.0254966 / 0.01611988 |
| RTX b-day, 2026-10-06 (by request start) | 7 | 34,359 | 0.01268184 |
| RTX b-day, 2026-10-07 | 8 | 43,805 | 0.01665244 |
| RTX b-day range total | 15 | 78,164 | 0.02933428 |
| RTX September (b-month) | 4 | 27,258 | 0.01496616 |
| RTX W40 (b-week) | 3 | 20,617 | 0.00968756 |
| RTX k1 root / descendants (2 threads) | 4 / 4 | — | 0.01664260 / 0.01595560 |
| RTX k3 root / descendants | 5 / 24 | — | 0.01930652 / 0.14179384 |
| RTX k4 own | 2 | — | 0.01034596 |
| RTX k1+k2+k3 | 39 | 398,333 | 0.20378164 |
| QA spend: mac | 22 | — | 0.2361168 |
| QA spend: no-usage proxy | 2 | — | 0.01698848 |
| QA spend: Linux | 114 | — | 0.39893252 |
| QA spend: total | — | — | ≈ 0.652 |

Notes on the table:
- Bucketing September and W40 by completion time would give one fewer request each, so these boundary tests do tell start-time from completion-time bucketing.
- Every listed product request amount matches the provider-derived values for each conversation I checked.
- The total RTX W40 spend across all conversations is 41 requests, $0.10757616; the product view is per conversation.

## Overall

**Supported, with corrections.** "S03 not accepted" is well supported, and the sprint record's own open items would block acceptance anyway. "Arithmetic exact" is supported: I found no discrepancy anywhere. The deletion FAIL is real on the narrower false-statement ground. The no-usage FAIL is real but already known, and its evidence is confounded. The clock-skew FAIL is real, but its breadth is overstated and it is probably inherited from S02. The exec-JSON row should not count as an S03 FAIL. The record should also list the unexercised failure, cancel and retry paths as open gaps.