# Independent evidence review: PF-60-S03 second targeted re-run

Reviewer: installed `corbanu exec`, `-m claude-opus-5-5-plan -c model_provider=claude-plan -c model_reasoning_effort=high -s read-only`, code-blind. It saw only a copy of this directory and `context/`: the first run's and first re-run's README, the first re-run's REVIEW and captures, `acceptance.md`, `zai-glm-52.md` and the texts of issues #286–#289.

There were two passes, with the same reviewer and settings. Pass 1 ran on a copy that was missing README.md, because of an executor copy error. Pass 2 audited the complete record. The reports are verbatim. The README's response is in its "Independent evidence review" section.

## Pass 2 (complete record)

**Conclusion: Supported with corrections.** Every verdict follows from its captures, and every number in the README matches my own recomputation. Three fixes are needed before #288 is commented on and before the record is relied on: the "#288 fixed" summary goes too far, the key-scan file it cites is missing, and one product observation cites a capture that doesn't show it.

## Verdicts

| Item | README | Evidence | My call |
|---|---|---|---|
| 8a | PASS | Its own provider is named on every page of the no-usage conversation: `c8-01:13`, `c8-02:19`, `c8-03:7`, `c8-04:7`, `c8-05:7`. Other conversations name it too: `c8-10:13`, `c8-13:13`. Pages read at 18:05:26 (`c8-01:24`), before incident 2 (18:08:58). | **Supported** |
| 8b | NOT VERIFIABLE | Custom provider, so "no price" is literally true (`c8-04:21`). A priced provider can't be pointed at the proxy (`repoint-probe-error.txt:1`). | **Supported** |
| 8c | NOT VERIFIABLE | The closest case, `zaipartial`, is described accurately (`c8-10:13,30-36`, `c8-11:7-8`, `c8-12:22-28`). | **Supported**. Not underclaimed: "priced only in part" needs a priced route. |
| #289 R1 | FAIL (unchanged) | `289-01:4-5`, in a home where a turn had completed (`mac-def-zai.jsonl` ends in `turn.completed`). Binary confirmed with `ps` (`build-receipts.txt:30`). | Supported |
| R1b | FAIL | `289-02:7`; today's page is the same (`289-08:4`). The 401 on that page doesn't affect it, since the default build keeps no ledger. | Supported |
| R2 | FAIL | `289-05:1,11` and `289-07:1,11`. Coverage quotes match `289-05:8` and `289-07:8`. | Supported |
| R3 | FAIL | "oldest daily total kept none" (`289-04:4`) vs "daily totals kept since 2025-10-09" (`c8-01:25`). | Supported (see 8) |
| 13a | PASS (macOS) | `289-03:7` | Supported |
| 13b | NOT VERIFIABLE | Behaviour observed in `289-01` and `289-08`. | Fine in substance; the label is wrong (see 7) |

## Findings

1. **Major: the #288 summary overclaims.**
   - README:238 says "**#288: the defect as reported is fixed.**" Only item 2 (the next step) is shown to be fixed.
   - The issue title says no-usage requests are "labelled 'no price available'". On the original reproduction that label is unchanged: `c8-01:5` and `c8-04:21` ("no applicable price").
   - Item 3 ("Pay per use" on custom providers) is unchanged (README:181-182), and nothing says where it is tracked.
   - The last comment on #288 says "still FAIL" (`issues-286-289.txt:62`).
   - **Commenting on #288 and leaving it closed is justified only if the comment:**
     - says item 2 is fixed;
     - says item 1 is unchanged on the custom-provider setup and can't be verified on a priced provider;
     - says item 3 is unchanged, and gives where it is tracked;
     - explicitly supersedes the "still FAIL" comment.
   - **Keeping #289 open is justified:** R1–R3 reproduce, and 13b is still waiting on a decision.

2. **Major: the key scan is cited but missing.**
   - README:62 cites `data/key-scan.txt`, but `data/` has no such file.
   - The scan matters here. `execrun.sh:8` ran exec with `RUST_LOG=codex_api=trace`, and scratch logs and homes sat outside this bundle.
   - My own grep of the bundle found nothing beyond `127.0.0.1`: no `/Users`, `/Volumes`, Bearer tokens, keys or IP addresses. Key handling as scripted is correct: the helper is the first prefix on the consuming command (`execrun.sh:10,13`) and TUIs use a placeholder key (`tuiview.sh:5`). The claim of a clean scratch area still has no evidence.

3. **Major: new observation 5 cites a capture that doesn't show it.**
   - README:185-186 quotes the "Z.AI (zai) credential was rejected…" warning from the "`mac-c8-07` last screen". That last screen (`c8-07:48-54`) shows only the cost popup.
   - The warning appears only in `289-08:13`. That is a default-build session on the built-in `zai` provider, where naming Z.AI is correct.
   - Remove the observation, or capture it in the `zainousage` session.

4. **Minor: new observation 1 names a cause the evidence doesn't support.** It was also posted to #289.
   - It says noncached input is unknown "when output is missing". That case is unpriced as well as partial, so the two causes are confounded.
   - Where the value is derived, the product says the derivation depends on the price: "cache writes were not reported, and this price charges nothing for cache writes" (first re-run `mac-289-03:23`). With cache writes unknown and no price, inclusive arithmetic can't derive noncached input.
   - The README's contrast also mixes page types. In this run, the complete-usage priced day page also says "not reported (2 attempts)", with a costing note (`c8-13:27`).
   - A `zaiproxy` technical page (complete usage, no price) would settle the cause. Until then, reword the observation here and on #289.

5. **Minor: README:21-23 points to a review that isn't there.** It cites `REVIEW.md` and an "Independent evidence review" section "at the end". Neither exists; the README ends at line 245.

6. **Minor: build provenance is mostly fixed, with some gaps.**
   - Fixed since the first pass:
     - Ancestry of both #305 (`f0d71e62b6`) and #291 is recorded with exit 0 (`build-receipts.txt:3-6`).
     - The copy step is recorded.
     - The binaries that ran are hashed, matching README:47-48.
     - The `ps` check is recorded.
     - The proxy and recompute tools are byte-identical to the earlier copies.
   - Remaining gaps:
     - The mapping from commit to PR is asserted, with no `gh` output.
     - There is no `git status` showing the worktree was clean.
     - The `.err` traces aren't retained, so neither `product_seen_usage.py` nor `recompute.py` can be re-run from the bundle.

7. **Nit: verdict labels.**
   - R1–R3 are wording residuals of #289, not acceptance criteria. "REPRODUCES" reads better than "FAIL".
   - 13b's behaviour was observed. The open item is a decision, so "DECISION NEEDED" fits better than "NOT VERIFIABLE".

8. **Nit: R3 says "same home and conversation".** `289-04` doesn't show which conversation it belongs to. It is linked only indirectly:
   - its coverage ends at 18:04:40.946Z, the 18:04:40.945Z ledger time of `c8-01:24` plus 1 ms;
   - the incident 2 narrative.

9. **Nit: README:152 is ambiguous.** "7 after incident 2" is how `zaipartial` and `zai` see the other conversations (`c8-10:6`, `c8-13:6`). From the `zainousage` conversation the count stays 6 (`c8-07:6`).

10. **Nit: README:19 says every number was recomputed "from provider-reported usage".** The `nu-zai` and `zaipartial` values come from the product's own trace. Only the proxy logs are captured outside the product. §Method states this correctly; line 19 doesn't.

11. **Nit: one proxy path is untouched.** The `zaipartial` proxy's tail-buffer path (`usage_proxy_partial.py:91-98`) doesn't drop keys. It had no effect here: the product saw no `completion_tokens` in either usage chunk (`product-seen-usage.txt:9-10`). Otherwise the diff against `usage_proxy.py` is sound: `--drop-keys` is added and the logged usage is deep-copied, so the log keeps the original.

**Incidents.** None of them changes any evidence a verdict rests on:
- **Incident 1:** the first home was set aside (`threads-mac.txt:10-12`).
- **Incident 2:** 8a rests on captures taken before it. The later captures used for 8a, 8c and R2 don't depend on the `zainousage` count. R1 rests on `289-01`.
- **Incident 3:** that home was set aside, and the default binary was confirmed with `ps`.

## First-pass findings

| # | Finding | Status |
|---|---|---|
| 1 | README missing | Resolved |
| 2 | Verdict calls (8b/8c NOT VERIFIABLE, R1 resting on `def-zai`, 13b as a decision) | Resolved |
| 3 | 401 incident taint and filing | Resolved |
| 4 | Build provenance | Mostly resolved (finding 6) |
| 5 | `zaipartial`: state that output was derivable | Resolved; the narrow test is still unstated (finding 11) |
| 6 | Key scan not evidenced | **Not resolved**: the README now cites a file that is missing (finding 2) |
| 7 | Product observations | Partly: recorded, but observation 1 now names an unsupported cause (finding 4) |
| 8 | #288 reconciliation | **Not resolved** (finding 1). The #289 part is resolved |
| 9 | Capture-ID reuse | Resolved (README:89-90) |

## Recomputation

Prices are USD per 1M tokens: input 1.40, cached input 0.26, output 4.40. I computed with exact decimals.

| Item | Data (prompt/cached/output) | Mine | README / product | Match |
|---|---|---|---|---|
| `nu-zai` req 1 / 2 | 12704/5248/21; 12989/12672/50 | 0.01189528 / 0.00395852 | 0.011895 / 0.003959 (`c8-13:38-39`) | ✓ |
| `nu-zai` total | 2 req | 25,764 tok; in 25,693, cached 17,920, out 71; **0.0158538** | same (`c8-13:5,25-31`) | ✓ |
| `nu-proxy` | 9975/0/21; 10262/0/46 | 20,304 tok; 0.0286266 | 20,304 tok; 0.028627 | ✓ |
| stripped (`zainousage`) | 9975/9920/30; 10261/9984/56 | 20,322 tok; 0.00601824 | 20,322; 0.006018 | ✓ |
| `zaipartial` (proxy original) | 9975/9920/21; 10262/9984/50 | 20,308 tok; 0.00595364 | 0.005954 | ✓ |
| `zaipartial` as the product saw it | totals 9,996 + 10,312 | 20,308; in 20,237; cached 19,904; reasoning 10; derivable output 21/50 | `c8-10:30-36`; README:142 | ✓ |
| Main spend | 8 req | 86,698 tok; 0.05645228 | 0.056452 | ✓ |
| `def-zai` (mh-default / mh-default2) | 2 + 2 | 0.0163002 / 0.02502996 | — | ✓ (mh-default2 equals `turn.completed` 26,111/10,496/100) |
| Aborted `nu-zai` | 2 req | 0.03047608 | per-source 0.030476 | ✓ |
| Other spend | 6 req | 78,143 tok; 0.07180624 | 0.071806 | ✓ |
| Total | 14 billed + 2 unbilled 401s | 164,841 tok; 0.12825852 | ≈0.128 | ✓ |
| Other conversations | 2+2+2 / 2+3+2 | 6 req, 4 unpriced / 7 req, 5 unpriced | `c8-01:6,10` / `c8-10:6,10` | ✓ |
## Pass 1 (README missing from the copy)

**Conclusion: Not supported as delivered.** `rerun2-20261008/README.md` is missing, so I couldn't check the verdicts, wording or claims it makes. Most of the evidence in `captures/`, `data/` and `tools/` holds up, but the verdicts below need corrections. Every number in `data/` recomputes exactly.

## Findings

1. **Blocker: the README is not in the bundle.** `rg --files` lists only `captures/`, `data/` and `tools/` under `rerun2-20261008/`. `data/threads-mac.txt:10` points to "see README incidents", which doesn't exist. Because of this I could not check:
   - the verdict wording itself;
   - the numbers as the README states them;
   - the key-handling and incident write-ups;
   - the text proposed for #288 and #289.

   Everything below is checked against the captures, not against README claims. Please re-run this review with the README included.

2. **Major: what each verdict can claim, based on the captures.**
   - **8a, a no-usage request gets a next step naming its own provider: PASS is supported.**
     - The provider is named on its own pages: `c8-01:13`, `c8-02:19`, `c8-03:7`, `c8-04:7`, `c8-05:7`.
     - Other conversations now name `zainousage`: `c8-10:13`, `c8-13:13`.
     - The main captures were read at 18:05:26 (`c8-01:24`), before the 401 incident in finding 3.
   - **8b, missing usage told apart from a missing price: should be NOT VERIFIABLE.** A FAIL would overclaim.
     - `zainousage` is a custom provider with no price, so "no price available" is literally true there. `c8-04:21` gives the same "no applicable price" reason it would give for `zaiproxy`.
     - A priced test case can't be built: the built-in `zai` provider can't be pointed at the proxy (`data/repoint-probe-error.txt:1`, "reserved built-in provider IDs").
     - Positive partial evidence: "Tokens: tokens not reported" (`c8-03:8`), and each component says "not reported" (`c8-01:27-33`).
   - **8c, partial usage: PASS only for the next step and for how the tokens are kept.**
     - Next step: `c8-10:13`, `c8-11:7`.
     - Input, cache read, reasoning and total are kept, and output is shown as unknown, not zero (`c8-12:22-28`, presence patch on line 35).
     - The "usage incomplete" label (from #291) applies only to priced requests, so it is NOT VERIFIABLE for the same reason as 8b. A blanket PASS would overclaim.
   - **#289 R1, default-build hedge "if this build records costs": the residual still reproduces.** See `mac-289-01:5`. The same home already had 2 successful default-build requests (`exec-json/mac-def-zai.jsonl` reaches `turn.completed`; `product-seen-usage.txt:12-13`).
   - **R1b, "Run /usage requests for today" dead end: still reproduces.** See `mac-289-02:7`. Today's page then says "ledger not installed" (`mac-289-08:4`).
   - **R2, "Bucket unavailable" title: still reproduces.** See `mac-289-05:1,11` and `mac-289-07:1,11`.
   - **R3, retention wording disagrees: still reproduces in the same home.** The range page says "oldest daily total kept none" (`mac-289-04:4`), while the day page says "daily totals kept since 2025-10-09" (`c8-01:25`). Both are read at the same ledger time, 18:04:40.94x.
   - **13a, `/cost` absent from a default build: PASS on macOS.** See `mac-289-03:7`. Linux passed in earlier runs.
   - **13b, `/usage requests` opens the accounting page in a default build (`mac-289-01`, `-08`): this needs an owner decision, not a PASS or FAIL.**
     - Criterion 13 as written covers only `/cost` being absent (`first-run-README.md:190`).
     - The acceptance record says shipping `/usage requests` "remains held" (`acceptance.md:13-14`).
     - Call it FAIL only if the criterion is amended to cover this page.

3. **Major: the 401 incident affects some later captures but none of the verdict evidence.**
   - **What happened.** At 18:08:58 a turn was typed into a view-only TUI that held a placeholder key. Z.AI returned 401 (`proxy-stripped.jsonl:3`). The product recorded it as `zainousage` Request 3 (`c8-08`, `c8-09`; admission at 18:08:58.224Z).
   - **What it affects.** Everything captured afterwards counts this unbilled attempt: `c8-07`, `c8-10`, `c8-13`, `mac-289-06` and `mac-289-07`. For example "3 requests", "7 requests in 3 conversations" and "5 attempts had no price" (`c8-10:6-10`). Any claim that product request counts equal the provider's billed counts must leave these captures out or subtract 1.
   - **Product observation that should be filed, not written off as harness noise.** A rejected, unbilled request is described as "3 of 3 billed attempts incomplete" (`c8-07:18`) and the user is told to "check the bill".
   - **`mac-289-08` also contains a 401 turn (lines 13–14).** It can't serve as "sent a turn, page still unavailable". R1 should rest on the successful `def-zai` exec run instead.

4. **Minor: build provenance is a transcribed receipt, not raw output.**
   - `build-receipts.txt:1-2` states commit 886f19c and that `f0d71e62b6` is an ancestor (exit 0), so #305 is claimed as included. There is no raw git or cargo output, and #291's ancestry isn't recorded.
   - Both builds share one `CARGO_TARGET_DIR` (`build-mac.sh:6`), so the second build overwrites the first. The step that copies binaries to `<scratch>/bin/corbanu-acct` (or the default build's name) isn't recorded, and the binaries actually run aren't hashed. Behaviour reduces the concern: `/cost` works in the developer-accounting captures and is unrecognized in `289-03`.
   - `build-receipts.txt:9-11` hashes `extract_usage.py` and `buckets.py`, but neither file is in `tools/`, so `product-seen-usage.txt` can't be regenerated.
   - The three tools that are present match their 16-character hash prefixes against my full sha256: `usage_proxy.py`, `recompute.py`, `home-config.toml`.

5. **Minor: the `zaipartial` proxy variant is sound but narrow.**
   - **What the diff changes.** It adds `--drop-keys`. The usage is deep-copied before keys are removed, so the log keeps the original (line 79). It also logs a `dropped` field.
   - **What the product saw.** `completion_tokens` was absent, but `total_tokens` and `reasoning_tokens` were kept (`product-seen-usage.txt:9-10`). Output was therefore derivable as total − prompt (21 and 50). Leaving output unknown is a conservative choice, and the README should state that output was derivable.
   - **An unmodified path.** The tail-buffer path (lines 91–98) doesn't drop keys. It had no effect here, because the product saw completion_tokens missing from both usage chunks.
   - **Only one shape tested.** Only a missing output count was tested.

6. **Minor: key handling and redaction look sound; one gap.**
   - The vault helper is attached to the consuming command (`execrun.sh:10,13`). TUIs use a placeholder key (`tuiview.sh:5`), and both 401s confirm it. The proxy never logs headers.
   - My grep for `/Users`, `/Volumes`, Bearer, keys and IP addresses found only `127.0.0.1`.
   - The gap: nothing in `data/` records a key-value scan, unlike the earlier runs.

7. **Minor: product observations the README should record.**
   - **Noncached input on unpriced partial usage.** It reads "not reported" or "unknown" even though input and cache read are both known (`c8-10:30-32`, `c8-12:22-24`). This is the #289 item-2 pattern coming back for unpriced providers.
   - **#288 item 3 is unchanged.** Custom providers are still labelled "Billing: Pay per use" (`c8-02:17`, `c8-11:5`).
   - **`exec --json` reports unknown usage as zero.** `mac-nu-nousage.jsonl` has `input_tokens` and `output_tokens` 0, and `mac-nu-partial.jsonl` has `output_tokens` 0. This belongs to the out-of-scope `exec --json` usage-gaps issue (#290).

8. **#288 and #289.**
   - **#288: commenting is justified; closing is justified with conditions.** Item 2 (the next step) is fixed. Closing should say plainly that item 1 can't be verified here (finding 2, 8b) and that item 3 is still open and tracked somewhere. #288 is already CLOSED, but its last comment says "still FAIL" (`issues-286-289.txt:52,62`), so the new comment must reconcile the two.
   - **#289: keeping it open is justified.** R1, R1b, R2 and R3 all reproduce on macOS, and 13b still needs a decision.

9. **Nit.** Capture IDs were reused with new meanings. In the first re-run, `mac-c8-06` was the evidence for other conversations' pages; here it is `/cost help`. The README shouldn't carry over the old mapping.

## Recomputation

Prices are input 1.40, cached input 0.26 and output 4.40 USD per 1M tokens, computed with exact decimals.

| Item | Data | Mine | Product capture | Match |
|---|---|---|---|---|
| nu-zai req 1 / 2 | 12704/5248/21; 12989/12672/50 | 0.0118953 / 0.0039585 | 0.011895 / 0.003959 (`c8-13:38-39`) | ✓ |
| nu-zai total | 2 req, 25,764 tok | 0.0158538; input 25,693, cache 17,920, out 71, reasoning 10 | 0.0158538 exact (`c8-13:25-31`) | ✓ |
| nu-proxy | 9975/0/21; 10262/0/46 | 20,304 tok, 0.028627 | 20,304 tok, no price (`c8-01:9`) | ✓ |
| stripped (nu-nousage) | 9975/9920/30; 10261/9984/56 | 20,322 tok, 0.006018 | tokens not reported | ✓ |
| partial (full usage from log) | 9975/9920/21; 10262/9984/50 | 20,308 tok, 0.005954 | 20,308 tok; input 20,237, cache 19,904, reasoning 10 (`c8-10:30-36`) | ✓ |
| partial as the product saw it (no output) | — | would be 0.005641 | no price | n/a |
| partial req 1 | 9975/9920, total 9996 | — | 9,996 tok; input 9975, cache 9920 (`c8-12:22-28`) | ✓ |
| other conversations (`c8-01`) | 2+2+2 | 6 req in 3 conversations; priced 0.015854; 4 unpriced | same (`c8-01:6,10`) | ✓ |
| other conversations (`c8-10`, after the incident) | 2+3+2 | 7 req; 5 unpriced (includes the 401) | same | ✓, but includes an unbilled attempt |
| `recompute-mac` ALL | 8 req | 86,698 tok, 0.05645228 | 0.056452 | ✓ |
| spend-other: def-zai / nu-zai(aborted) | 4 / 2 req | 0.041330 / 0.030476; 78,143 tok, 0.07180624 | 0.071806 | ✓ |
| def-zai (mh-default2) vs `turn.completed` | 26,111 / 10,496 / 100 | 0.02502996 | matches exec JSON | ✓ |
| total spend | 14 billed requests, 164,841 tok | 0.12825852 (plus 2 unbilled 401s) | — | — |