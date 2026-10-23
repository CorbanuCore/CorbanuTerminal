# Independent evidence review: PF-60-S03 third targeted re-run

Reviewer: installed `corbanu exec`, `-m claude-opus-5-5-plan -c model_provider=claude-plan -c model_reasoning_effort=high -s read-only`, code-blind. It saw only a copy of this directory (README as first written) and `context/`: the earlier runs' README/REVIEW files, two first-re-run captures, `acceptance.md`, `zai-glm-52.md` and the texts of issues #286-#289 and #308. One pass. The report is verbatim; line numbers refer to the README before the corrections listed in its "Independent evidence review" section.

---

# PF-60-S03 third re-run (`ccc38bfc03`): independent audit

**Conclusion: Supported with corrections.** Every verdict holds and every displayed number recomputes exactly. The corrections are about scope and disclosure: the "For Travis" summary leaves out acceptance gates and one #287 residual that this record's own evidence reproduces, and a few smaller claims are unevidenced or slightly overstated.

## Verdicts

| Item | README | Evidence checked (file:line) | My call |
|---|---|---|---|
| #289 R1/R1b/13b, default build | PASS (signed-in menu NOT VERIFIABLE) | `mac-289-r1-01:11`, `-02:5-6`, `-03:4-6` show one line each, with no page and no next step. `-04a:9` shows the description. `-04:13` and `-05:4-5` show the signed-out hint. | **PASS, with two sub-items NOT VERIFIABLE**: the signed-in menu, and the "usage error" text, which was never reached (F5). |
| 13a `/cost` absent | PASS | `mac-289-13a:5` | PASS. This also corroborates that the default binary was the one running. |
| #289 R2 | PASS | `r2-01:8-9,13`, `r2-02-top:10`, `r2-02:2-4` (no next step), `r2-03:9,12,15`, `r2-04-raw:3,9`, `r2-05:13`, `r2-06:2,8-10` | PASS |
| #289 R3 | PASS | `r2-01:4`, `r2-03:4`, `r2-05:4`, `r2-04-raw:5`, `r2-02-top:15`, `r2-06:21`; the day pages are `mac-c9-02:23` and `rtx-308-01:22` | PASS |
| #308 delete with the clock behind | PASS (succeeded, consistent) | `rtx-308-del58s:3-6`, `del6d:3-9` (exit 0, rollout gone, faketime control); `rtx-308-03:6-7`; `rtx-308-01:6,10,21`; `rtx-308-02:4-5,15` | **PASS, with caveats**: the short case was about 57 s behind, not 58 (F4), and the listing of the untouched rollouts is not in the record (F3). |
| 8a | PASS | `mac-c8-01:5,11`; `mac-c8-02:3,7-8` | PASS |
| 9 | PASS | `mac-c9-01:2-11` (root and child removed, 2 kept); `mac-c9-02:6-10` | PASS |
| 15 | PASS | `accounting-log-lines-rtx.txt:2`; `rtx-sk58s.jsonl` ends in `turn.completed`; `rtx-c15-01:5,23`; `-02:6-7`; `-03:18` | **PASS**, but the same captures reproduce open #287 residual 2 (F2). |

## Findings

1. **Medium: "For Travis" leaves out the acceptance gates.**
   - `context/acceptance.md:2-15` says S03 "is not accepted". It also says the scope-zero P2 has "no fix or accepted waiver", and that shipping `/usage requests` "remains held".
   - It also lists the round-66 timeout as open (`:26-37`).
   - The README calls itself the "final confirming re-run" and says "every item this re-run covered passes". It should say these gates are outside its scope and are not cleared by it.
   - The scoped empty-day wording in `rtx-308-02:4-5,8,17` may bear on scope-zero, but this run did not target it.

2. **Medium: open #287 residual 2 is reproduced in this record but not reported.**
   - The WARN line says "recording at the ledger's time until it catches up" (`accounting-log-lines-rtx.txt:2`).
   - Yet the attempt was admitted at the faked process time, 2026-10-08T22:39:09.255Z (`rtx-c15-03:18`). That is about 49 s before the checkpoint, which `behind_ms=48950` puts at ≈22:39:58.134Z.
   - The price was also observed and made effective at that faked time (`:37-38`).
   - The README cites the faked admission time as PASS evidence without noting the conflict. "For Travis" lists only #287's >90-day item; residuals 2 and 3 (`issue-287:17-18`) are missing.

3. **Low: one #308 claim has no evidence in the record.**
   - README:175-176 says the `vk`/`sk58s`/`cp1` rollouts are listed "in the `rtx-308-02` run log". That file contains only the TUI page, and `rtx-del.sh` lists only the deleted ID.
   - Indirect support exists: `rtx-308-01:6` shows 9 requests in 2 conversations, and `sk58s` was viewed after the deletes (`rtx-c15-01:21`).

4. **Low: the "58 s" delete was about 57.2 s behind.**
   - Faked 22:39:54 at a real start of ≈22:40:52, against a checkpoint of 22:40:51.235Z.
   - The README body says "about 57 s", but the verdict table, `threads.txt:7` and "For Travis" say 58 s. This doesn't change the result.

5. **Low: item 1 is slightly under-scoped.**
   - #318 also changed "the usage error" (`issue-289:45`). `/usage bogus` returned the signed-out hint instead (`r1-05:4-5`), so the usage error was never exercised.
   - The `ps` check that the default binary was running is not recorded; only `13a:5` corroborates it.
   - The "no 401 in stderr" claim is also unrecorded.

6. **Low: the example given for the clock offset doesn't show 58 s.**
   - README:229-231 compares a response-ID stamp (06:40:09+08, which is request start) with a log line at 22:39:20 (usage end). That pair gives 49 s.
   - Rows 2-7 of `recompute-rtx.txt:7-12` do show 58 s (for example, 22:39:23 vs 06:40:21+08).

7. **Low: the "same preload and offset failed before" argument covers only the 58 s case.**
   - The first re-run's captures show only 58 s deletes (`rtx-c15-08:3-4`, `-12:4-5`). There is no 6-day precedent.
   - Also, the README says it used only #322's title and merge state, but it read `issue-308`, which describes #322's internals. That is allowed under the code-blind rules, but the statement is imprecise.

8. **Low: the price source.**
   - `context/zai-glm-52.md` contains no prices. The rates come from `first-run-README:101-105` and the product's own technical page (`rtx-c15-03:39-42`).
   - I checked them independently: Z.AI's pricing page lists GLM-5.2 | $1.4 | $0.26 | Limited-time Free | $4.4 per 1M tokens (input, cached input, cached-input storage, output). These match.
   - The `zai` usage comes from the product's own trace lines; the raw `.err` files are not committed. Two outside cross-checks agree with it: the proxy log, and `exec --json` `turn.completed`.

9. **Low: security is credibly evidenced, with these deviations.**
   - Every wrapper sets `CORBANU_TEST_NO_NATIVE_KEYRING=1` and disposable homes (`execrun.sh:10,13`, `rtx-execrun.sh:7`, `rtx-del.sh:8`, `tuiview.sh:5`, `rtx-tui.sh:4`). A pattern scan of `record/` finds only placeholder keys.
   - **(a)** `rtxrun.sh:4-5` attaches the helper to a `bash`/`printf`/`ssh` relay, not to the consuming `corbanu`. This is disclosed, and `printf` is a builtin, so the key never appears in a process's arguments.
   - **(b)** The key scan substitutes the key into `grep`'s arguments (`key-scan.txt:1`), so it is briefly visible in the process table. Feeding it to grep with `-f` from process substitution would avoid that.
   - **(c)** The macOS `d1` delete (README:270) has no wrapper in `tools/`, so its environment (key, keyring flag, binary) is not shown. Only the disposable-home path is visible (`mac-c9-01:2`).
   - The code-blind boundary rests on the executor's statement. The record contains no source excerpts.

10. **Low: two capture-quality issues.**
    - `mac-289-r2-02-top:1` is labelled "first screen", but it shows the Details section under a pinned title (cursor at `:18`).
    - `mac-289-r2-02` is an incomplete union of the page's lines (incident 2). `r2-06` is complete, so the verdict is unaffected.

11. **Info: other notes on "For Travis".**
    - **#310:** it is listed as open, but `zai-glm-52.md:80-83` says the key is removed from model commands. Its status should be reconciled.
    - **#308:** the issue's remaining window (`issue-308:10`) and the untested fail-intact branch are not mentioned.
    - **#288 item 3:** confirmed still present (`c8-01:5`, `c8-02:5`).
    - **#289:** the 401-counted-as-"billed" item and the "Partial — excluded" follow-up match `issue-289:60`.
    - **Possible support for #308 (not stated in the README):** on macOS the ledger's current-to time (22:45:11.964Z) is later than `d1`'s end at 22:45:00. A real-clock delete may have advanced it. The RTX behind-clock deletes left it at 22:40:51.235Z, which fits the README's inference, but the macOS delete time isn't recorded.

## Recomputation

Prices used, USD per 1M tokens: noncached input 1.40, cached 0.26, output 4.40. I used exact fractions over the per-request usage in `data/`.

| View (capture) | Displayed | Recomputed | Match |
|---|---|---|---|
| `dev-zai`, own (`r2-02:4`, `r2-06:10,22-29`, `c9-02:5,24-31`) | 2 req; 25,819 tok (in 25,751 / cache 17,920 / out 68 / reasoning 8); $0.015922; exact 0.0159218 | 7,831×1.40 + 17,920×0.26 + 68×4.40 = 0.0159218 | ✓ |
| `dev-zai` Req 1 / Req 2 | 0.011934 / 0.003987 | 0.01193448 / 0.00398732 | ✓ |
| `zainousage` (`c8-01:5`) | 2 req, tokens not reported, no price | proxy: 2 responses, 20,366 tok (list 0.01734664, not shown) | ✓ |
| macOS deleted (`c8-01:10`, `c9-02:10`) | 6 attempts | `d1`: 6 req, 72,368 tok, 0.05430952 | ✓ |
| `vk`, own (`rtx-308-01:5,23-30`) | 2; 13,429 (13,364 / 13,056 / 65 / 10); $0.004112; 0.00411176 | 0.00411176; Req 1 0.00180368, Req 2 0.00230808 (shown 0.001804 / 0.002308) | ✓ |
| `vk`, others (`:6-8`) | 9 req, 2 convs, 64,338, $0.022646 | `sk58s` + `cp1`: 9, 64,338, 0.02264604 | ✓ |
| RTX deleted (`:10`) | 4 attempts | `a1` 2 + `a2` 2 (27,328 tok, 0.02382536, not shown) | ✓ |
| 2026-10-02 (`rtx-308-02`) | 0 | 0 | ✓ |
| `sk58s`, own (`c15-01:5,23-30`) | 7; 50,427 (49,993 / 47,488 / 434 / 63); $0.017763; 0.01776348 | 0.01776348 | ✓ |
| `sk58s` Req 1-7 (`c15-01:36-42`) | .002083 .002144 .002555 .002221 .002621 .002852 .003288 | .0020832 .00214408 .0025554 .00222064 .00262064 .002852 (exact, correctly shown without "rounded") .00328752 | ✓ |
| `sk58s`, others (`:6-8`) | 4 req, 2 convs, 27,340, $0.008994 | `vk` + `cp1`: 4, 27,340, 0.00899432 | ✓ |
| Req 1 components (`c15-03:22-33`) | 136 / 6,400 / 52 → 0.0001904 + 0.001664 + 0.0002288 = 0.0020832 | same | ✓ |
| Spend (README:245-247) | 12 / 0.112201; 15 / 0.050583; 27 / ≈0.163 | 0.11220092; 0.05058316; 0.16278408 | ✓ |
| Cross-checks | `exec --json` `turn.completed` (only `d1` is root-only: 51,831, which is #290); `product-seen-usage` lines; read-at − current-to gaps (39,768 / 133,311 / 62,577 / 88,027 / 159,210 / 22,358 / 189,241 ms) | all consistent | ✓ |

No arithmetic mismatch was found.