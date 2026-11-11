**VERDICT: fail.** Freeze order, isolation, the totals (19 passed, 2 failed, 4 blocked) and most dispositions hold up. But the record has two wrong figures, one wrong fact about how the executor navigated, an untested claim stated as fact, and a spend claim that the shakedown's own relay log contradicts. These need correcting before a human accepts.

## 1. Order and freeze: supported
- **Hashes:** every file in `design/FROZEN.sha256` still matches its hash.
- **Times:** design pushed `b53816429f` at 02:01:19Z, packet at 02:02:20Z, then the lock and clone at 02:02:23Z and SSH at 02:02:35Z (`common/vm-start.txt`, `common/effective-policy.txt`). The push times can't be confirmed from this folder because it has no git history.
- **Packet matches the freeze:** `executor-packet/design-cases.md`, `amendments.md` and `answers.md` are byte-identical to `designer-output.md`, `amendments-final.md` and `answers-final.md`. All four executors also checked these hashes themselves.
- **Amendments unchanged:** the amendments section of `amendments-final.md` matches the reviewer's text in `review-output.md` exactly, and its JSON equals `review-amendments.json` (25 amendments).
- **`design.json`:** its 24 designer cases match the frozen text exactly, and each attached amendment is identical to the reviewer's. FUP-25 comes from amendment 18. `design.json` itself isn't listed in `FROZEN.sha256`, and the run receipts bind to its hash (b4cfb625…).

## 2. Cases

**Supported as written:** FUP-03, 04, 05, 07, 08, 09, 11, 12, 13, 20, 21, 22, 23, 24 and 25.
- FUP-03: Thread Y, `Turn: code-review:` on all 4 review requests, sum 0.00351237.
- FUP-05: `Turn: side:01a15723…` with Thread X, at 02:30:16Z.
- FUP-21 to 23: every expected line is on R's screens.

**Corrections needed:**

1. **FUP-01:** the summary's sum "0.0011844 + 0.0000015 = 0.00120318" is false. It leaves out the cache-read cost of 0.00001728, which is shown on the 02:13:01Z screen. The pass itself stands.

2. **FUP-16:** "exact sum through R13 0.01611978" repeats the same $0.0000004 slip that FUP-10 corrects. It should be **0.01611938**. This appears in `dispositions.md`, `results.json` and `tools/final.json`.

3. **FUP-10:** passing is defensible, but two things are left out:
   - **Request 1's exact cost:** its technical page was opened only in FUP-09 (03:13Z). The final-audit screens labelled "R01-tech" show its request page, and 0.0014646 is on no screen. It can be derived from the product's 03:10Z subtotal 0.003695025 minus Request 2's 0.002230425; the disposition should say that.
   - **Skipped audits:** amendment 12 required a full Request-ID audit after FUP-12 and FUP-14. Neither was done; IDs for R4–R9 and R11–R13 first appear at 03:54Z or later. Only R1–R3 and R10 were read earlier. This should be listed as an executor deviation.

4. **FUP-17 (and 19):** "6 Down presses on the request page" is wrong.
   - On K3 Request 1 the executor pressed Down 30, 30, 1 and 10 times. The selection wraps around the page, so it overshot the link.
   - It pressed none at all on K3 Request 2 or on either K2.7 request page (`runs/M/collected/evidence/actions.jsonl`).
   - The proof that `→ Technical details (attempt 1)` exists under `—— Open ——` comes only from `pty.raw`, not from any captured screen. The summary must label it as the integrator's supplementary read.
   - Blocked rather than failed is still right. The conversation-level arithmetic and the relay-usage figures are all correct: 0.036201, 0.0042648, 0.0404658, 0.01095155, 0.00236896 and 0.01332051.

5. **FUP-14:** the failure is supported by the screens.
   - After "continuing from this point in a new conversation" no notice appears, and the new conversation's `/cost` page has no `Reopen warm-up` line. The notice visible higher up belongs to the 03:38:56Z restart. The ledger shows `prewarm:`.
   - But "The cut leaves no history, so the product treats it as a new conversation" (in `dispositions.md`) and "`skip_reopen_warmup` would not skip it" (in the README) come from reading the code or are untested. Both must be labelled as such.

6. **FUP-06 and FUP-15:** both are fine.
   - FUP-06: the refusal text is in `pty.raw` at byte 1483577, between screen captures at 1483497 and 1483857, and every captured screen for about 6.6 minutes shows only `■`.
   - FUP-15: the opened conversation shows "Thread forked from <T1>" with 1 request, and the picker row chosen was the 03:38 fork. Blocked is correct.

7. **FUP-02 (judgement call):** the expected text "says no requests were recorded" isn't on screen; the product says the detail is unavailable. Either mark it failed (it is advisory), or state explicitly why the different wording still meets the intent.

## 3. Isolation
- **Probes:** all groups passed 57 + 52 = 109 probes with no failures, and each executor's own 57 probes were all true. All 28 receipts are bound to the right run, case, design, candidate, policy, probes and relay window, and every receipt check is true.
- **Gate and guard:** the gate and the smoke turn passed. The guard saw 0 SecurityAgent prompts in 12,043 checks.
- **Caps:** the relay started once with realtime cap 2 and Moonshot cap 4. 1 realtime session and 4 Moonshot POSTs were used, with no refusals.
- **End of run:** keys were checked absent before the clone was deleted (`vm-state-end.txt`).
- **Not evidenced or not stated:**
  - **Lock release:** "lock released" is printed outside the captured log, so the record doesn't show it.
  - **UTM end state:** UTM's state is captured only at the start.
  - **Designer and reviewer isolation:** both were instruction-only, meaning reads were not restricted (`designer-session.md`). The README's named limits don't mention this.

## 4. README
1. **Shakedown spend claim is contradicted.** The README and the policy say the shakedown sent no OpenAI inference. But `shakedown/common/mediator-final.jsonl.gz` shows a real OpenAI Responses WebSocket forwarded at 01:54:12Z (101, 923 bytes, about 1 s). That matches the pattern of Group R's billed GPT-6 Sol warm-up ($0.029290). The OpenAI spend (about $0.059) and the claim must disclose this possible extra warm-up of about $0.03.
2. **FUP-03 wording:** "the 4 review requests'… exact sum equals the product's subtotal" is wrong. All 5 of Y's requests sum to the subtotal; the 4 review requests alone sum to 0.00223062.
3. **Moonshot explanation:** "(navigation, not the product)" rests on `pty.raw`; say so.
4. **Screen archives:** only Z and O have a screen `.tar.gz`. R's and M's captures exist only as labelled entries in `screens.jsonl`.
5. **Accurate as stated:** totals, Moonshot $0.053787, Group O's OpenAI items (about $0.0252), realtime about $0.0048, Z.AI 446 requests ≈ $1.132, the key-scan file count (339 − 8 + 1 = 332), and the Tart notes.
6. **Before acceptance:** `results.json` carries no issue references, and the repository's checker still exits 1 at FUP-06.

VERDICT: fail