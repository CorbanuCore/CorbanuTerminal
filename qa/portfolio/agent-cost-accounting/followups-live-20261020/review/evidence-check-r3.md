**VERDICT: pass.** The freeze, packet identity, isolation, totals and all 25 dispositions hold up against the evidence. Every r1 and r2 correction is applied and checks out. I found nothing that must be corrected. A few optional clarifications are listed at the end.

## 1. Order and freeze: supported
- **Freeze hashes:** all 30 entries in `design/FROZEN.sha256` still match.
- **Timing:** `common/vm-start.txt` shows the lock and clone at 02:02:23Z and SSH at 02:02:35Z. That is after the stated freeze (02:01:19Z) and packet push (02:02:20Z). The push times themselves can't be checked, because the record has no git history.
- **Packet matches the freeze:**
  - `executor-packet/design-cases.md`, `amendments.md` and `answers.md` are byte-identical to `designer-output.md`, `amendments-final.md` and `answers-final.md`.
  - The packet screen hashes match `design/packet/screens`.
  - All four executors checked the hashes themselves.
- **Amendments unchanged:** the `## Amendments` section of `amendments-final.md` is identical to `review-output.md`'s (21,751 chars). Its JSON equals `review-amendments.json` (25 amendments).
- **`design.json`:**
  - The amendments attached to each case equal the reviewer's.
  - Case texts are in `designer-output.md`; the only exception is FUP-25, which comes from amendment 18.
  - All 28 receipts bind to its hash, b4cfb625….
- **`results.json`:** all 363 path and hash references resolve and match. Its summaries match `dispositions.md` and `tools/final.json`.

## 2. Cases

**The three dispositions that differ from the executor's verdict:**
- **FUP-10 (failed → passed): supported.**
  - Request 1's figures come from the O screens:
    - `FUP09-19` at 03:13:04 shows `Input: 14646` with every other count 0;
    - `FUP09-33` gives the 0.1 USD/M rate;
    - `FUP09-06` gives the subtotal 0.003695025, and 0.003695025 − 0.002230425 = 0.0014646.
  - Summing the 14 exact values gives 0.01630897, which equals the subtotal on screen.
  - Re-reads: R1 (first read 03:11:19), R2 (03:10:52), R3 (03:14:44) and R10 (03:34:48) were all re-read in the final audit. R4–R9 and R11–R13 first appear in that audit. The skipped intermediate audits are disclosed.
- **FUP-15 (failed → blocked): supported.**
  - `FUP15-06` shows "Thread forked from <T1>".
  - `FUP15-07` shows 1 request with `Reopen warm-up: 1` and "15 requests in 3 conversations" (11 + 2 + 2).
  - The ledger shows nothing sent at 03:43.
- **FUP-06 (failed kept): supported.**
  - The refusal is at byte 1483577 of `runs/Z/.../pty.raw.gz`, followed at once by a clear-screen (`ESC[2J`).
  - Screens from 02:31:16 to 02:37:55 sit at one pty offset and show only `■`.
  - X stays at 2 requests in the ledger.

**The other cases asked about:**
- **FUP-14 (failed): supported.** On `FUP14-20` and `FUP14-21`, the only notice sits above "continuing from this point" and belongs to the 03:38:56 restart. The new conversation has 2 requests and no Reopen line. The `prewarm:` label comes from the ledger and is labelled as such.
- **FUP-17, 18 and 19 (blocked): right.**
  - Down presses: 30, 30, 1 and 10 on K3 Request 1, and none on the other three request pages (`actions.jsonl`).
  - No M screen shows a technical page or any rate.
  - `→ Tech…ical details (attempt 1)` appears in `pty.raw` only, with K3 Request 1's ID.
  - Every figure matches the relay usage: 0.0404658, 0.01332051, 0.036201, 0.0042648, 0.01095155 and 0.00236896.
- **FUP-03 and FUP-05: supported.**
  - FUP-03: four technical pages show `code-review:` and Thread Y (01a15717), and the subtotal is 0.00351237.
  - FUP-05: `group-Z/fup05-r2-attempt-40.txt` shows Thread X and `Turn: side:01a15723…`.
- **FUP-21 to 23: supported.**
  - `realtime:ws:` appears with Thread TR.
  - FUP-22's screen shows "at least $0.029290 (1 attempt had no price)".
  - After the restart the Request ID and times are unchanged, and the ledger shows one prewarm.
- **Supplementary reads** (ledger, relay, `pty.raw`) are labelled as the integrator's in the dispositions. One exception is noted at the end.

## 3. Isolation: supported
- **Probes:** 57 + 52 + 57 per group, with no failures.
- **Receipts:** all 28 match on run, case, design, candidate, agent, policy, probes and relay window, and every receipt check is true.
- **Gate:** 109 probes passed, `AGMS-SMOKE-OK` appeared, and coordinator egress was blocked.
- **Guard:** 0 SecurityAgent prompts in 12,043 checks.
- **Caps:** the relay started once with realtime 2 and Moonshot 4. It forwarded 1 realtime session and 4 Moonshot POSTs, with no cap events.
- **End of run:** keys were shown absent, then the clone was deleted at 05:30:45Z (`vm-state-end.txt`). The lock and UTM state are covered by `host-end-check.txt`.
- **Not provable from the record:** the pf allowlist addresses (only `provision.sh` shows them).

## 4. README: matches the evidence
- **Totals:** 18 passed, 2 failed, 5 blocked.
- **Moonshot:** $0.05378631.
- **OpenAI:**
  - Group O: $0.025164.
  - Realtime: $0.004764.
  - Run total: about $0.059, and about $0.063 for D16 counting the shakedown.
- **Z.AI:** 446 requests, 33.17M input tokens, 32.59M cached, 133k output, about $1.132.
- **Shakedown:** the WebSocket opened about 70–75 ms after the refusal.
- **Key scan:** 347 = 339 + 8.
- **Times and Tart notes** match the logs.

## Optional clarifications
1. **FUP-15:** the fork ID `01a15763…` and its "03:38:05" time come only from the ledger; they appear on no screen and in no executor transcript. They should be labelled as the integrator's read.
2. **FUP-02:** the README's "(outside its history)" should say "outside its request-detail history". The same screen says daily totals are kept since 2025-10-21, which covers that date. Whether this is blocked or an advisory failure remains a call for the human.
3. **Moonshot link:** the README should say the `pty.raw` link was seen only on K3 Request 1's page.
4. **FUP-10:** the final audit actually ran 03:47:30–04:04:09Z, not "03:50–04:02Z". The claims hold either way.
5. **Before acceptance:** update `results.json` `evidence_check` and `review-ledger.md` for this round (5 of 5).

VERDICT: pass