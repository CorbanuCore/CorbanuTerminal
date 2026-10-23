**Conclusion: Supported with corrections.** Every number I recomputed matches exactly and every listed flow has a verdict. Two things need fixing before Travis signs off: the code-blind boundary was the executor's own description rather than something enforced, and the waiver (a) recommendation needs a narrower scope.

## Findings

### Major

1. **The code-blind boundary was self-declared, and wider than the sprint allows.** (README "Method", "Candidate"; sprint "Verification")
   - The sprint wants an executor "who reads only this record, the qualification and in-product help".
   - This executor also read the S03 and S05 acceptance READMEs and tools, `docs/*.md`, the `justfile`, `git log` titles and `codex-rs/core/config.schema.json`. It also built the binary itself from `git archive`, so it had the repository.
   - AGENTS.md (Sept 12 amendment) says a prompt-only restriction is not enough. It requires enforced filesystem, tool, process and network boundaries, with negative access probes recorded. None are recorded here.
   - The qualification says the isolated execution gate was not used. This README doesn't say so.
   - **Fix:** state this as a deviation needing Travis's waiver (as S05 waiver (b) did), not as an unqualified "code-blind" run. The verdicts themselves still stand on the evidence.

2. **Waiver (a): the lift is justified, but its scope is overstated.** (README criterion 14, "For Travis"; S05 rerun README lines 174–178)
   - **What the evidence proves:** API-key estimates are exact for `gpt-5.4`, `gpt-5.6-luna` and `gpt-5.6-terra`, on the Standard tier, short context, cache writes included. Fast tier shows "no price available" and is never mis-priced.
   - **The wrong part:** the "What remains" bullet blames the remaining "no price" models on #368. #368 only lists `gpt-6.1-sol` (plus the Sol promotion date). S05's concern that "most OpenAI models show no price", and the unpriced Fast/priority tier, are not #368 items in the qualification.
   - **Not exercised:** long-context band, Batch/Flex, `gpt-5.6-sol` and other catalogue models.
   - **Fix:** lift waiver (a) for estimate _correctness_, scoped to the three models and Standard tier. State that unpriced tiers and models show "no price", which is not a mismatch, and is a separate coverage limitation rather than #368.

### Minor

3. **Consolidation (criterion 3) has an unexplained request.**
   - `/cost` in session A shows 4 requests, 2 of them unreported (`h4-05`). The README only explains request 4.
   - `data/ledger-dumps.txt` (h4) shows the other one is `turn=assess:01a12468-a66d…` with `obs=none`. Neither the `assess:` label nor its unknown usage is mentioned.
   - Only one consolidation request completed (the lane saw 6). There is no count of consolidation requests started versus recorded, so completeness is not shown — only that the recorded ones are labelled and attributed correctly.

4. **Range totals match only after rounding (criterion 12).**
   - `h2-06` to `h2-08` show only the rounded $0.291501, $0.920787 and $1.001880. Range views have no exact-USD line.
   - "Each equals the ledger sums" should read "matches the ledger sums at the displayed rounding". Day views do show exact lines, and those match.

5. **Arithmetic error.** The Reconciliation `h6` row says "27,383 cache-write tokens". The correct figure is 13,643 + 15 + 13,723 = **27,381**, which is also what `h6-03` shows.

6. **The audit trail is incomplete.**
   - "Every command sent is in keys-sent.log" holds for TUI driving only. These steps are not logged:
     - the `accounting_demo_seed` run;
     - the retry-proxy launch and `zai-retry` config;
     - the ledger dumps, recompute and key scan.
   - The seeding time can only be inferred from a gap in the log (05:54:10–05:55:07, between `h2-03` and the resume).
   - Raw `codex-tui.log` frames are not archived. `recompute-all.txt` is an extract, so it can't be checked against them.
   - "13 created, 12 completed" (criterion 6) and the empty `git diff --stat` claim have no supporting data file.

7. **Key-scan claims are stronger than the tool shows.** (`tools/keyscan.sh`, `data/key-scan.txt`)
   - The README says each value was "confirmed non-empty". The script has no such check, and an empty helper output would give a false 0 hits.
   - The key-shaped-string scan only ran on the evidence directory, not "everywhere".
   - The scratch workspace `work/ws` was not scanned, even though agents with shell access had the keys in their environment.
   - I found no secret, real path, host or IP in the evidence directory (only `127.0.0.1` in the proxy tool). Key handling in `q`, `start.sh` and `ephem.sh` (stdin to environment only) looks plausible.

8. **Gap (ii) leftover.** The unpriced Fast-tier page (`h1-20`) still shows "Cache write cost: $0.000000; exact USD 0". The gap asks to show "plainly that it has no price". Observation 3 calls this cosmetic; it should be filed as a follow-up for Travis, not left as an observation.

9. **Duplicate retry: money dedup was not exercised (criterion 8).** The retry ran on an unpriced custom route. Token dedup (5,572+) and the predecessor link (`cc527f8d…`, confirmed in `h3-03` and `h3-04`) are verified; say in the verdict that the money side wasn't.

10. **Coverage gaps against the qualification.**
    - Qualification flow 12 (basis per route) has no verdict row. Evidence exists in pieces: `h3-03` declared basis, plus the subscription and pay-per-use labels.
    - DeepSeek V4 Flash and GLM 5.2 were not re-run; Z.AI GLM 5.3 Flash was used instead, per the Oct 10 amendment. Say that the qualification's DeepSeek and GLM 5.2 recomputes were not independently re-confirmed.

11. **NOT VERIFIABLE items lack fresh evidence.** TensorCash has no recorded access probe (for example `git ls-remote` output). Image generation relies on S05 rather than a re-check. ChatGPT login and realtime are reasoned adequately.

### Nit

12. "Homes: `h1` to `h5`", but `h6` was used.
13. The README links `REVIEW.md` and says a reviewer "audited this record" before that file exists.
14. Criterion 5 says "those five captures" but cites six files (`h4-06` has req2 and req4).
15. `sec-common.md` in Method is not identified.
16. The candidate binary (`953802b1…`) is not the lane's recorded binary (`cb378c27…`). That's fine, but say plainly that this run qualifies its own recorded binary.

## What I checked and found correct

- **Turn labels and thread ids:**
  - `side:prewarm:` and `side:01a12457…` are both on `Thread: 01a12451-0c3e…`, which matches the resume id in `h1-15` and `launch-commands.txt`.
  - The `consolidation:` requests are on session A (`01a12468-99a5…`).
  - The children's thread ids match the spawned ids.
- **Request lists stay stable:** across `h1-03 → 09 → 13 → 16 → 18 → 19`, each earlier request list is unchanged, and only the expected requests are added (12–13, 14, 15, 16–17, 18–19).
- **Resume is clean:** `h2-02` and `h2-04` are identical on the provider lines, request rows and exact subtotal.
- **"Price: none recorded":** appears once per page (twice per file because of the cursor duplicate) in all six basis-only files. None of them has a Price ID, source, currency, observed date or rate line.
- **Refusals:** the future day, reversed range, future start, bad grouping and before-retention messages all match `h2-09` to `h2-15`. The malformed date and `yesterday` each add one usage line.
- **Ephemeral exclusion:** exactly one WARN line, and the `e2` control created one ledger row on thread `01a12471-1996`.
- **tmux suite:** 71 of 71 passed on `b1e20a8ec6`.

## Numbers I recomputed

Recomputed with Decimal from `recompute-all.txt` rows at the rates in `published-prices-20261010.txt` and `openai-pricing-20261010.txt`. Uncached input = in − cached − write. All 27 rows matched their own `usd=` figure.

| Check | My result | `/cost` capture |
| --- | --- | --- |
| h1 journey: 7 `gpt-5.4` rows | 0.1025795 | — |
| h1 journey: plus GLM 0.00113168 | **0.10371118** | `h1-03` exact 0.10371118; OpenAI 105,730 tokens matches |
| h1 after `/side` | 0.16364418 | `h1-09` matches |
| h1 OpenAI share after cancel | 0.1625125, 131,867 tokens | `h1-13` "at least $0.162512", 131,867+ |
| h1 after reopen | 0.18900918 | `h1-16` matches |
| h1 after kill and restart (auto 0.1002175 + default 0.1197035 + GLM 0.00113168) | **0.22105268** | `h1-18`, `h1-19` exact 0.22105268 |
| h1 Fast tier at $5 / $0.50 / $30 | 0.123971 | shown as no price |
| h2: 4 GLM 5.3 Flash rows | **0.00300274**, 34,660 tokens | `h2-02` exact 0.00300274 |
| h2 Claude at API prices: 4×4 + 5,172×0.2 + 5,867×8 + 8×20, per million | 0.0481464 | matches |
| h1 Claude at API prices: 4×4 + 15,114×8 + 8×20, per million | 0.121088 | matches |
| h6 luna (0.0021104 + 0.00341855 + 0.00028441) | 0.00581336 | — |
| h6 terra (0.021104 + 0.0343855) | 0.0554895 | — |
| h6 total | **0.06130286** | `h6-03` exact 0.06130286; tokens 37,871 / 24,284 match |
| h6 cache-write tokens | 27,381 | README says 27,383 (finding 5) |

Day and range totals, from the per-day rows in `day-totals.txt`:

| Check | My result | Product shows |
| --- | --- | --- |
| All days | 1.00188048 | — |
| Seeded days only | 0.99887774 | — |
| 2026-10-05 to 10-10 | 0.2915007 | $0.291501 (rounded) |
| 2026-09-28 to 10-11 | 0.92078666 | $0.920787 (rounded) |
| 2026-09-28 to 10-04 | 0.62928596 | — |
| Sep–Oct month range | 1.00188048 | $1.001880 (rounded) |
| Days 10-07, 09-28, 10-09 | 0.15569982, 0.161997, 0.03180032 | exact lines in `h2-05-*` match |

Token sums:

| Check | My result |
| --- | --- |
| Kimi, h1 | 7,895 + 436 = 8,331 |
| Kimi, h2 | 5,300 + 420 = 5,720 |
| Kimi, h4 session A | 7,908 + 14,946 = 22,854+ |
| Kimi, h4 session B | 7,892 + 412 = 8,304 |
| Retry, h3 | 5,538 + 34 = 5,572, which matches `retry-proxy.jsonl` |

Spend: $0.221 + $0.124 + $0.061 + under $0.02 ≈ $0.42, consistent with the README.
