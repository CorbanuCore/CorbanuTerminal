# Independent fixture expectations (before PTY execution)

The loopback fixture emits numbers below. These are provider-reported synthetic token counts, not tokenizer measurements of the prompt. The real product sampler must retain and render them. Expected money is computed with Python Decimal and these frozen constants, never from stored estimates or contributions.

The bundled catalogue prices selected by API-key/default-tier accounting are USD per million tokens:

| Model | Noncached input | Cache read | Output |
| --- | ---: | ---: | ---: |
| gpt-5.6-sol | 5 | 0.5 | 30 |
| gpt-5.6-terra | 2.5 | 0.25 | 15 |

Price-binding payloads must independently match these constants. A custom model catalogue changes tool presentation only; the sampler still binds the bundled price authority.

- Each root Sol response: 100 inclusive input, 20 cached, 10 output, 4 reasoning. Noncached input = 100 − 20 = 80. Cost = (80×5 + 20×0.5 + 10×30)/1,000,000 = **0.00071 USD**.
- Each child Terra response: 80 inclusive input, 10 cached, 6 output, 2 reasoning. Noncached = 70. Cost = (70×2.5 + 10×0.25 + 6×15)/1,000,000 = **0.0002675 USD**.
- Separate root later given unknown source metadata: 40 input, 5 cached, 4 output, 1 reasoning. Noncached = 35. Cost = (35×5 + 5×0.5 + 4×30)/1,000,000 = **0.0002975 USD**.
- Cache write is reported zero; reasoning is a subset of output and total tokens are not separately charged.
- Intended initial root/child sequence: two root responses (spawn then completion), one child response. Root-own **0.00142**, descendant **0.0002675**, combined **0.0016875 USD**; provider/model groups partition that combined amount. Input 280, noncached 230, cache read 50, cache write 0, output 26, reasoning 10, total 306. The orphan's amount is separate.
- Any additional actual wire responses remain in the server receipt and are summed using the same fixed arithmetic; no response may be silently dropped to fit the intended sequence.

Traffic is sampled around **2026-08-10 12:15 UTC** through a wall-clock-only interposition fixture. At real-time reopen, a new current-day response advances the normal checkpoint and does not belong to August queries. The original sampled attempts and price bindings must confirm that the clock fixture worked.

Nonempty complete buckets:

| Group | Half-open UTC bounds | Equality |
| --- | --- | --- |
| Hour | 2026-08-10 12:00 to 13:00 | All historical traffic occurs here, so amount/tokens equal August 10 standalone day |
| ISO week | Monday 2026-08-10 to Monday 2026-08-17 | Same; no sampled traffic on other days of this week |
| Calendar month | 2026-08-01 to 2026-09-01 | Same; no other August sampled traffic |

A separate cap fixture requests 513 real sampled attempts, each 10 input, no cache, 1 output. Expected hidden total would be 513×(10×5+1×30)/1,000,000 = 0.04104 USD. The inspector must refuse and show **no** amount; 0.04104 is a checker's expectation, not a total to put into product output.

Whole-store work-budget padding is explicitly artificial and is never used as monetary evidence: unselected attempt rows on another UTC day, without fabricated observations/prices/contributions. This tests the row-budget preflight. Measure actual empty-day and one-attempt-day results around 571,428 and 666,666 rows. Do not describe these padding rows as sampled or fully validated ledger history. Restore the synthetic profile after measuring.
