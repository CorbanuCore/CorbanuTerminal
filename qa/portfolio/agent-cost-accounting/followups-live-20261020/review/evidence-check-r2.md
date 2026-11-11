**VERDICT: fail.** The freeze, isolation, totals and all 25 dispositions hold up, and so does every r1 correction I re-checked. Three record errors still need fixing: one wrong evidence citation in a D25 claim, one stale field in results.json, and one wrong ordering in the shakedown spend wording.

## 1. Order and freeze: supported
- **Hashes:** every entry in `design/FROZEN.sha256` still matches. `vm-start.txt` shows the lock and clone at 02:02:23Z and SSH at 02:02:35Z. That is after the stated freeze (02:01:19Z) and packet push (02:02:20Z), as `effective-policy.txt` also says. The push times themselves can't be checked: the record has no git history.
- **Packet matches the freeze:**
  - `executor-packet/design-cases.md`, `amendments.md` and `answers.md` match `designer-output.md`, `amendments-final.md` and `answers-final.md` byte for byte.
  - All packet digests match `candidate/packet-files.txt`.
  - All four executors checked the hashes themselves.
- **Amendments unchanged:** the amendments section of `amendments-final.md` is identical to `review-output.md`'s (21,736 chars), and its JSON equals `review-amendments.json` (25 amendments, CHANGES).

## 2. Cases
**Re-checks of the r1 corrections (all applied correctly):**
- FUP-01: the sum now includes the 0.00001728 cache read (screen 218).
- FUP-16: the sum through R13 is now 0.01611938 in all three files.
- FUP-02: now blocked.
- FUP-03: the wording now says the sum is over all 5 of Y's requests.
- FUP-14: the claims from reading code are labelled.
- FUP-17: the navigation facts and the pty.raw source are now right. I confirmed the Down presses (30, 30, 1, 10 on K3 Request 1, none on the other three request pages) in `runs/M/.../actions.jsonl`. I also confirmed that `→ Technical details (attempt 1)` appears only in pty.raw, never on a captured screen.

**The dispositions that differ from the executor's verdict:**
- **FUP-10 (failed → passed): defensible.**
  - R1, R2, R3 and R10 were first read at 03:11:19Z, 03:10:52Z, 03:14:44Z and 03:34:48Z, and re-read unchanged at 03:50–03:59Z.
  - R1's derivation holds: FUP09-19 shows `Input: 14646` with all other counts 0; 0.003695025 − 0.002230425 = 0.0014646; and the sum is 0.01630897.
  - Link costs for R1–R10 before 03:50Z are identical in the audit, and the 5 warm-ups are labelled.
  - Precision point: the audit after FUP-09 was also incomplete (R4's ID wasn't read until 03:54Z). The README limit names only the audits after FUP-12 and FUP-14.
- **FUP-15 (failed → blocked): right.** Screen 128 shows "Thread forked from <T1>" with 1 Reopen warm-up ($0.001465) and "15 requests in 3 conversations". In the ledger, only fork 01a15763's 03:38:05 warm-up fits, and nothing was sent at 03:43.
- **FUP-06 (failed kept): supported.**
  - The refusal is in `runs/Z/.../pty.raw.gz` at byte 1483577, followed at once by a clear-screen.
  - Eight screens at a frozen pty offset (02:31:16–02:37:55Z) show only `■`.
  - X stayed at 2 requests.

**The other cases asked about:**
- **FUP-14 (failed): supported.** On screens 115–116, the only notice sits above "continuing from this point" (it belongs to the 03:38:56 restart), and the new conversation has no Reopen-warm-up line. The ledger shows `prewarm:` for thread 01a15764.
- **FUP-17/18/19 (blocked): right.** I recomputed every figure, and all match:
  - Conversation level: 0.0404658 and 0.01332051.
  - Relay usage: 0.036201, 0.0042648, 0.01095155 and 0.00236896.
- **FUP-03:** all 4 technical pages for R2–R5 carry `code-review:` and Thread Y; the five requests sum to 0.00351237.
- **FUP-21 to 23:** every expected line is on R's screens. One point the human should see: the GPT-6 Sol warm-up is labelled only by the `Turn: prewarm:` line on its technical page. The executor's caveat about this ("passed with ambiguity") isn't carried into the summary, though the summary's wording is accurate.
- **Supplementary reads:** the ledger, relay and pty.raw reads are consistently labelled as the integrator's, and no disposition rests on them.

## 3. Isolation: supported
- **Probes:** each group passed 57 + 52 probes (109) with no failures; each executor ran 57 itself, all true.
- **Receipts:** all 28 are bound to the right run, case, design.json (b4cfb625…), candidate, policy, probes and relay window.
- **Gate and guard:** the gate passed and the smoke turn returned OK. The guard saw 0 SecurityAgent prompts in 12,043 checks.
- **Caps:** the relay started once with realtime 2 and Moonshot 4. One realtime session and 4 Moonshot POSTs were used, with no refusals.
- **Guest network:** provisioning added the hosts entries; the pf allowlist addresses are shown only in `provision.sh`, not in the root policy dumps.
- **End of run:** keys were checked absent (`erase-keys.out`), then the clone was deleted at 05:30:45Z. `host-end-check.txt` shows no lock and UTM stopped.
- **Stated limits:** complete.

## 4. README
These all match the evidence:
- the totals (18/2/5);
- Moonshot $0.05378631;
- Group O $0.025164;
- realtime $0.004764, giving about $0.034 for the run and about $0.063 with the shakedown;
- Z.AI: 446 requests, 33.17M input, 32.59M cached, ≈$1.13;
- the key scan: 344 files = 336 committed + 8 ignored;
- the issue references, the Tart notes and the times.

## Required corrections
1. **FUP-05's evidence file is wrong** (`dispositions.md`, `results.json`, `tools/final.json`). `fup05-r2-attempt-1.txt` shows only rates and `Close`. The screens with Thread X plus `Turn: side:01a15723…` are `group-Z/fup05-r2-attempt-40.txt` and `fup05-r2-cost-nav-1..8.txt`, the earliest at 02:30:16Z (screens.jsonl.gz entry 1243 shows the Turn line alone). Cite one of those.
2. **`results.json` budget count is stale.** `review_budget.used` is 2 (from `tools/review-budget.json`), but `review-ledger.md` records 3 rounds used.
3. **The shakedown ordering is wrong** (README Execution section and `effective-policy-addendum.txt` item 1). Both say the Responses WebSocket opened "before" the realtime refusal. The relay's connection times show it opened about 75 ms *after* the refusal: realtime 1792461251229649000, refusal …234312000, WebSocket …304500000. Group R shows the same order. The spend estimate is unaffected.

**Minor:** state that the audit after FUP-09 was also partial (R4 unread). The frozen `designer-session.md` says "eight" S06–S08 screens; there are nine, and the README's total of 13 is correct.

VERDICT: fail