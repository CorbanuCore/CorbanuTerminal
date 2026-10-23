**VERDICT: fail.** All of r1's corrections hold, and isolation, the freeze order and the totals are supported. But four passed C2 dispositions (ACC-11, ACC-14, ACC-17, ACC-18) claim checks that no captured screen shows. ACC-17 needs a softer fix than the other three.

## r1 corrections: all made
- **ACC-15:** the corrected reason is supported. `runs/C2/collected/evidence/pty.raw.gz` holds `Turn: side:01a14382-90…` at offsets 1,632,506–1,637,929. That matches the Down keys at 07:03:26–31Z in `actions.jsonl` and ledger line 11. No line in `screens.jsonl` contains `side:`. README and `dispositions.md` now say this.
- **ACC-18:** the two residuals are confirmed in `runs/C2/ledger-attempts.txt`. Line 8 is 01a14381, line 12 is 01a14387, and each holds one plain `prewarm`.
- **ACC-36:** the conversation that opened was not ALPHA, and this is now disclosed.
- **Priorities:** ACC-17, ACC-34 and ACC-35 now carry their amended priorities in `dispositions.md` and `results.json`.
- **`make_packet.py`:** the drift is disclosed. Its hash is now 036a3c8c…; the frozen hash is 5863196e….
- **Gaps:** the provisioning runs, the :443 scope and the unlabelled "0" are explained in the addendum. The request count reads 1,132 and the spend scope is stated. `keyscan-record.txt` now exists, and the sprint list includes ACC-01, 29 and 34.
- **Harness for F:** `agms-product.v1-as-used-by-A-to-E` differs from the current script only in how an empty provider list is expanded, so using it for F is harmless.

## 1. Order and freeze: holds
- **Freeze timing:** `FROZEN.sha256` is stamped 02:53:41Z. The candidate was installed at 03:08:51Z and the lock taken at 03:10:03Z. Every frozen file still verifies except `make_packet.py`, which is disclosed. I could not check the pushes themselves: there is no git data in the copy.
- **Packet contents:** v1 `design-cases.md` is byte-identical to `designer-output.md`. `amendments.md` and `answers.md` are the same in all three packets, and all 28 screens match.
- **Packet diffs:** `tools/packet-v2-diff.txt` and `tools/packet-v3-diff.txt` reproduce exactly. C2 and E2 differ from C and E only in the assignment header and group note. `packet-v2-install.out` replaced only the navigation note and the E, F, C2 and D2 prompts.
- **Amendments:** `amendments-final.md` matches the reviewer's text from "### Amendments" onward; the only addition is a header.
- **`design.json`:** the attached amendments match the JSON block for all 22. Its `priority` fields still hold the unamended values; the attached amendments govern.
- **Hash references:** all 795 `path`/`sha256` pairs in `results.json` verify.

## 2. Cases
**Supported:**
- **ACC-01–03, 07–09, 19–21, 23–33 and 37.** For ACC-37, branch (a) is OpenRouter GLM, branch (b) is Vercel and Corbanu API, branch (c) is mimo and the uncatalogued model. The missing counts are named as `unknown` on the technical pages.
- **ACC-06 and #898.** `runs/A/ledger-attempts.txt` shows six requests under thread 01a142f0-c302 with plain turn IDs (04:20:37–04:21:29Z). `/cost` showed "No other conversation" through 04:25:44Z. After the control exec it showed "1 request in 1 conversation", so the review is shown nowhere.
- **The blocked cases.** ACC-04 (the model's verbatim decline), ACC-22 ("not advertised"), ACC-35 (`d3.txt`: NOT GRANTED) and ACC-15 are all supported.
- **ACC-34.** E2's exact-case `grep` output was empty (`executor-events.jsonl`). Its "hit" came from case-insensitive reading of the fixture listing.
- **ACC-36.** The deviation is disclosed.
- **ACC-16.** C2's evidence matches C's. The verdict flipped because the integrator read "+1" as applying to the new conversation, not because of the replay. This is disclosed, but a human should note it.
- **The replays.** Both are legitimate: new executor thread IDs, profiles wiped first, raw verdicts of the first attempts kept. The reasons are confirmed in `runs/C/.../result.md` (request pages would not open) and `runs/E/.../result.md` (the corbanu route would not start).

**Overstated, which must be corrected:** after the restart at 06:47Z no earlier request page was opened again. In C2's `screens.jsonl` and `pty.raw.gz`, `Request: <id>` lines appear only at 06:39–06:49Z and for Request 8 at 07:02–07:03Z. ACC-18 also never opened a request page.
- **ACC-11:** the amended expected result is "C1…Cn and their Request IDs are unchanged". The summary says "earlier costs and IDs unchanged". The IDs were not re-read; only the costs were.
- **ACC-14:** the expected result is "Earlier request costs and IDs are unchanged; each ID appears once". No ID was on screen in this case. The summary claims IDs, and so does the executor's own "passed".
- **ACC-18:**
  - "Every cost and Request ID … identical": no IDs were viewed.
  - "No Request ID appears in two conversations": this result is silently dropped from the summary and was not observed.
  - "Each conversation's request costs sum to its total": shown only for ALPHA (Requests 1–9, which add up). The request lists for BRAVO, CHARLIE and the fork were never opened, and the "exact subtotals" the executor cites do not appear in any ACC-18 screen.
  - What does hold: no warm-up rise, ALPHA's costs unchanged, and the first-screen reconciliation. I checked that 0.003699 + 0.030794 equals the sum of all totals, and that 27 = 9 + 2 + 14 + 1 + 1.
- **ACC-17 (softer):** "each listed once" rests on aggregate counts only. Codex Auto Review went from 1 to 3 requests and GPT-6 Luna from 3 to 11; only Request 1 and Request 2 were ever seen as links. The summary should say so.

These passes conflict with the rule used to keep ACC-15 blocked ("dispositions rest on what the executor observed and judged").

**Minor:** ACC-05 and the README say "listed once as a descendant", but no screen shows a descendant count above "(none)". The sub-agent's request is counted in the root total under its own thread. The README's "(it would pass)" for ACC-15 is stronger than the "suggests" used in `dispositions.md`.

## 3. Isolation: holds
- **Receipts and probes:** 62 receipts (A 8, B 4, C 11, D 5, E 9, F 5, C2 11, E2 9; r1 said 64). Every one is bound to its own run, case, group and executor thread, design e6ac8f…, candidate 5b805b…, and hash-matched files, with every flag true. Every run's probes pass 105/105 (55 + 50), with 0 failures.
- **Gate:** attempt 1 failed on a read-only `config.toml`. Attempt 2 failed because the probe counted the keychain-off switch as a credential name. Attempt 3 passed.
- **Guard:** 0 SecurityAgent prompts in 16,192 checks.
- **Relay:** it started with the allowlisted paths and a cap of 12. Denied paths: 46 Moonshot, 92 OpenAI, 46 Z.AI. Moonshot forwards: 2 POSTs plus 1 `GET /v1/models` positive control. Z.AI: 1,132 requests.
- **Restore and shutdown:** the original pf anchor and `/etc/hosts` are back, no accounts or campaign keys remain, and the verification boot shows no Moonshot hosts entry. The VM stopped at 08:06:47Z and the lock was deleted at 08:06:54Z.
- **Limits not stated in the README:**
  - `harness/agms-product` was changed while group F was running; this is only in the addendum.
  - The pre-existing Z.AI key file stays on the guest (listed in `vm-state-end.txt`; mentioned only in the policy).

## 4. Totals and claims
- **Counts:** 32 passed, 1 failed, 4 blocked; this matches `results.json`, `dispositions.md` and `tools/final.json`.
- **Spend:**
  - Moonshot: about $0.047.
  - Z.AI: about $3.16, from 1.34 M uncached input, 94.1 M cached input and 0.27 M output tokens.
  - OpenAI: about $0.26 is plausible. Group D alone shows about $0.200 and C2 about $0.034.
- **Key scan:** `keyscan-record.txt` reports 957 files scanned, but this record has 945. Re-run the scan on the final record.

## Required corrections
1. **ACC-11, ACC-14 and ACC-18:** remove the claims that Request IDs were unchanged or unique. Then either mark these results not observed and dispose consistently with ACC-15 (blocked), or state explicitly why passing without them is allowed. For ACC-18, also say the per-conversation sums were checked only for ALPHA.
2. **ACC-17:** state that "each listed once" rests on aggregate counts only.
3. **README:** carry these through to the S08 claim "history not re-priced", and add the ID gap to the named limits. Also soften "as a descendant" (ACC-05) and "(it would pass)" (ACC-15).

VERDICT: fail