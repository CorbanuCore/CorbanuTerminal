# Code-blind review of the re-run record (first draft)

Reviewer: installed `corbanu exec -m claude-opus-5-5-plan -c model_provider=claude-plan -c model_reasoning_effort=high -s read-only`, prompt in [data/review-prompt.md](data/review-prompt.md). Verbatim output below; the corrections applied are listed at the end of [README.md](README.md).

**Conclusion: Supported with corrections.** The headline numbers, token totals, overflow-note count and AC7 gap warnings all check out against the archived data. The corrections are about how a few results are framed, some citations, and the recommendation to Travis. No verdict needs to flip, except that the recommendation needs explicit waivers.

## Recomputation (done independently from `data/provider-usage-*.jsonl`)

| Run | Mine | Record |
|---|---|---|
| r1-t1 (luna, Standard) | 0.00625965 | ✓ |
| lx-t1 | 0.00480615 | ✓ |
| r3-terra at $2 / $0.20 / $2.50 / $12 | 0.062594 | ✓ (but see finding 3) |
| r5-prio (Fast) | 0.0125224 | ✓ |
| `/cost` | 9,594×$1 + 3×$1 + 5×$6 = 0.009627 | ✓ |
| Kimi M / L / custom | 10,428 / 6,527 / 7,429 | ✓ |
| Claude exec | 4 + 8,347 write + 4 = 8,355; API equivalent $0.167180 at the ledger's 10/20/50 rates | ✓ |
| Claude pane | 2 + 26,035 + 4 = 26,041 | ✓ |
| Overflow note | present in 9 views; absent in the other 9, none of which has a subscription line | ✓ |
| AC7 | `r7-search-inj`, `r7-sws-inj` and `r7-sws-post` each have exactly one accounting gap item and a final answer; h-r7 ledger 6 → 6 → 10 | ✓ |

The key scan found no key-shaped strings in the directory.

## Findings

1. **High: the recommendation hides two gates that aren't met.**
   - **AC10 fails as written.** AC10 requires pay-per-use estimates to "match an independent recomputation … to the micro-dollar", and that clause failed. Calling it "price data, not a blocker" is the executor dropping part of an AC on its own say-so.
   - **AC11's isolation gate wasn't used.** AC11 requires the isolated execution gate; the Method section says the run used "self-discipline". Observation 7 is a real isolation breach: the model read and ran a skill from the real user's `~/.local/share`.
   - **Correction:** change "For Travis" to "Accept only if Travis explicitly (a) waives AC10's estimate-match clause for OpenAI API-key routes pending #361, and (b) accepts AC11 without the enforced isolation gate." Move observation 7 into Safety as an isolation deviation.

2. **Medium: the price errors are described in the wrong direction.**
   - **Luna:** "5x high" is true of the rates only. The totals are 1.54x high on macOS and 2.0x high on Linux. The real problem is that "at least $0.009627" is a lower bound that is *above* the true cost, so the claim is false.
   - **Terra:** "25% high" is wrong for the total. `/cost` shows $0.024068 against a recomputed $0.062594, about 62% *low*, because the cache write has no price.
   - **Correction:** say "rates 5x / 25% high; luna lower bounds exceed the true cost; terra totals are understated about 2.6x because cache writes are unpriced."

3. **Medium: the price extract doesn't cover every rate used.**
   - `data/openai-pricing-20261009.txt` lists only `gpt-5.6-luna` and `gpt-6-luna`.
   - The terra recompute ($2 / $0.20 / $2.50 / $12) and the `gpt-5.4-mini` and `gpt-5.2` rows in `recompute-mac.txt` use rates with no archived source.
   - **Correction:** add those models' rows from the same fetch, or mark those recomputes as unsourced.

4. **Medium: the AC4 priority-tier evidence is mis-cited, and a deviation is undisclosed.**
   - **Mis-cited:** the cited `provider-usage-mac.jsonl` has no `service_tier` field, because `usage_extract.py` drops it. Only the derived `tier=priority` column in `recompute-mac.txt` shows the served tier.
   - **Override not recorded:** `configs/mac-h-r5.toml` says `service_tier = "default"`, so the override actually used isn't on record.
   - **Undisclosed deviation:** the frozen design (R5a) says `service_tier="fast"`; the run used `"priority"`.
   - **Correction:** archive the `service_tier` field (extend the extractor), cite it, record the exact override, and list fast → priority as a deviation from the frozen design.

5. **Medium: AC7 status on the new binary is incomplete.**
   - The re-run binary `5d283fde18` includes the #351 fix ("Keep turns going when the state DB is busy or read-only"). The first run's AC7 turn-level FAILs (read-only DB, busy DB) weren't re-run and aren't mentioned.
   - The R7 cases `sws-post` and the baselines weren't in the frozen design (R6 covered only refusal before the response). They are post-freeze additions and should be labelled as such.
   - **Correction:** state "first-run AC7 PARTIAL stands; read-only and busy modes not re-run on `5d283fde18`" (or re-run them), and label the R7 additions as amendments.

6. **Medium: no key rotation after the bearer leak.**
   - The OpenAI bearer was written to disk in plaintext. Deleting the file doesn't prove it's gone.
   - **Correction:** record that `openai-api-key` was rotated, or tell Travis to rotate it.

7. **Low: AC7 overclaims.**
   - **Headline:** it says *both* search paths survive failure "before or after the paid response". Only the standalone tool (`r7-sws-post`) was tested after the response.
   - **"Result returned":** the answers (lobste.rs, slashdot.org) are well-known domains and could come from model memory. The exec JSON shows a completed `web_search` item, but no result payload.
   - **Logs:** the cited "sent unrecorded" and `alpha/search` log lines aren't archived.
   - **Correction:** narrow the headline; reword to "the turn completed with an answer after a completed web_search call"; archive redacted `.err` excerpts.

8. **Low: wrong citations.**
   - AC12 Kimi Code cites `mac-r3-day`, which shows only `kimicode-custom`. The built-in `kimi-code` evidence is in `mac-r1-day-after-t2` and `mac-r1-t2-kimi-conv`.
   - The pane's token total comes from Claude Code's own usage report in `rtx-r4p-pane-cost`, not a wire trace. Say so.
   - AC4 cites the ledger basis (`PlanEquivalent`), but Method says the ledger was used "only to name turns". Reconcile the two.

9. **Low: path not redacted.** `rtx-r4p-pane-cost.txt` contains an unredacted RTX scratch path, while every other path is redacted (`<rtx-scratch>`, `<rtx-user>`). Redact it.

10. **Nit: small fixes.**
    - The Spend section lists Linux `lx-t1` as "untraced", but it is traced in `recompute-rtx.txt`.
    - The post-injection counts "6 / 5" in `r7-injections.txt` have no labels.
    - The README links `REVIEW.md`, which doesn't exist in `rerun-20261009/`.
    - The record doesn't say that the first run's verdicts were on `d7846e29d5`, not this binary, as AC1 requires one recorded binary.
