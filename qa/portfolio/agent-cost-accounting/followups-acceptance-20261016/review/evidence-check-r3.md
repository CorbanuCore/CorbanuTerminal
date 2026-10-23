I checked the record afresh, and it fails. The case dispositions and totals hold up and every r2 correction has been made. But four written statements contradict the evidence or leave out facts that matter for isolation, and one file count needs explaining.

## r2 corrections: all made
- **ACC-11, ACC-14 and ACC-18** are now blocked because the Request IDs were not observed. C2's `screens.jsonl.gz` shows no `Request:` line after the 06:47Z restart except Request 8 (5b00d270, 07:02–07:03Z). ACC-14 shows no ID at all, and ACC-18 opened no request page.
- **ACC-18:** sums are stated for ALPHA only, which is true: links 1–9 appear only for ALPHA.
- **ACC-17:** "listed once rests on aggregate counts" is stated. The ledger agrees: 3 Codex Auto Review and 11 GPT-6 Luna requests (`runs/C2/ledger-attempts.txt` lines 14–27).
- **README:** the Request-ID gap, the harness change during F and the guest's Z.AI key file are now in the limits. ACC-05 and ACC-15 are worded more softly.
- **Totals:** 29 passed, 1 failed, 7 blocked. This matches `results.json`, `dispositions.md` and `tools/final.json`, including summaries and priorities.

## 1. Order and freeze: holds
- **Freeze timing:** `FROZEN.sha256` is stamped 02:53:41Z. The candidate was built at 03:08:51Z and the lock taken at 03:10:03Z (`common/vm-start.txt`). Every frozen hash verifies except `tools/make_packet.py`, which is disclosed. I could not check the pushes because there is no git data in the copy.
- **Packets:**
  - v1 `design-cases.md` equals `designer-output.md`. `amendments.md`, `answers.md` and the 28 screens are identical in v1, v2 and v3.
  - Both packet diffs reproduce exactly. Between versions only the navigation note changes, plus the C2/D2/E2 prompts, which differ from their originals only in the group header and note.
  - `packet-v2-install.out` shows that only E, F, C2 and D2 were replaced on the guest.
- **Amendments:** `amendments-final.md` matches `review-output.md` exactly from "### Amendments" on. All 22 amendments are attached to the right cases in `design.json`.
- **Hashes:** all 795 path/hash pairs in `results.json` verify.

## 2. Cases: dispositions supported
- **ACC-06 (#898):** six requests sit under thread 01a142f0-c302 with plain turn labels (04:20:37–04:21:29Z, `runs/A/ledger-attempts.txt`). `/cost` showed "No other conversation" through 04:25:44Z, and after the control run only 1 request in 1 conversation. The description is supported.
- **ACC-04 and ACC-22:** blocked correctly. ACC-04 has the model's verbatim decline; ACC-22 has the "not advertised" warning.
- **ACC-34:** passed is supported. Run E2's exact-case `grep` of the workspace printed nothing (its `executor-events` log), so its "hit" was the executor misreading the uppercase `TOKEN=`.
- **ACC-15:** `side:01a14382-90…` is in `pty.raw.gz` at offsets 1,632,506–1,637,929, during the 07:03:28Z Down presses. No captured screen contains `side:`. Blocked is right.
- **ACC-16:** C2's evidence matches C's. The flip from C's raw failed to passed reinterprets "+1" as counting in the new conversation. That is disclosed, but a human should note it. A fresh ⚠ notice follows "continuing in a new conversation", and the abandoned Request 5 ($0.000189) appears unchanged on ALPHA in ACC-18.
- **ACC-18:** the residuals are 01a14381 and 01a14387, one plain prewarm each in the ledger. The first screens reconcile: four conversations each show $0.034493 when added to their other-conversations figure, and the counts sum to 29.
- **ACC-37:** all three branches are shown (GLM: an estimate; Vercel and Corbanu API: "at least …"; mimo and the uncatalogued model: no price).
- **Replays C2 and E2:** legitimate. They had new executor threads, wiped profiles and identical packets, and the first attempts' raw verdicts are kept. The reasons are confirmed in C's and E's `result.md`.
- **Spend:** consistent with the evidence.
  - Moonshot: 0.036213 + 0.010991.
  - OpenAI: C 0.0303 + D 0.1999 + C2 0.0345 ≈ $0.26.
  - Z.AI: 1.34 M uncached input, 94.09 M cached and 0.27 M output tokens at 0.15/0.03/0.5 per M comes to about $3.16; 1,132 forwards.

## 3. Isolation: receipts hold, but the written account is incomplete
- **Receipts and probes:** 62 receipts, each bound to the right run, case, group, executor thread, design (e6ac8f…) and candidate (5b805b…). All flags are true and all referenced hashes match. Every run's probes pass 105/105 (launcher 55 + child 50) with 0 failures.
- **Relay:** cap 12. Moonshot had 2 POSTs plus 1 positive-control GET; the other denials are logged.
- **Guard:** 0 SecurityAgent prompts in 16,192 checks.
- **Restore and shutdown:** restore, verification boot, VM stop at 08:06:47Z and lock deletion at 08:06:54Z are all confirmed.
- **Gaps:** see corrections 1–3 below.

## Required corrections
1. **Harness fix 1 timing contradicts the addendum.**
   - The fix check's Corbanu stand-in request is at 06:11:56Z (`runs/F/mediator-window.jsonl`; 1,530 tokens, matching `common/harness-fix1-check.txt`). F's first product start was 06:12:46Z (`runs/F/collected/evidence/actions.jsonl`).
   - So F's product starts did **not** use the v1 script, and "06:2xZ" is wrong. Correct `common/effective-policy-addendum.txt`.
   - Also disclose that this check ran on lane agmx0 at the same time as F's executor (06:10:39–06:32:10Z), against "lanes run one at a time". Its request sits in F's receipt-bound relay log.
2. **Packet v2 was installed while group D was running.** D ran 04:58–05:27Z, and D's own directory listing shows `prompts` changed at 22:01 local (05:01Z). D's transcript never contains the v2 note. Say so alongside the F change.
3. **The failed gate attempts each sent a real Z.AI smoke turn.**
   - `gate.sh` does not stop when probes fail, so a smoke turn ran each time: relay POSTs at 03:26:22Z and 03:28:02Z, and `AGMS-SMOKE-OK` in both attempts' smoke output files.
   - Attempt 1 ran no probes at all; attempt 2 had a failing probe. "Attempt 1 failed before any probe ran" is incomplete. Disclose this in the README or policy.
4. **The README's S08 bullet is misleading.** "reopen warm-up on a new conversation and an edit, labelled and announced" reads as the opposite of ACC-10 and ACC-13 (a new conversation shows no label and no notice). Reword it.
5. **The key-scan file count doesn't match the record.** `keyscan-record.txt` reports 964 files; this record has 948. The 16 extra are probably the uncompressed JSONL beside their `.gz` (8 runs × 2). State that, or re-run the scan on the exact record tree. My pattern scan of this copy found 0 hits.

## Worth noting before acceptance (not blocking)
- **`auth.json` left on the guest:** a pre-existing `/Users/agent/.pf83-auth/auth.json` stays on the guest along with the Z.AI key file (`vm-state-end.txt`), but the README names only the key file.
- **Receipts written after the run:** the receipts bind a policy file that describes the 08:06Z shutdown, so the integrator assembled them afterwards. Their flags rest on `probes.json`.
- **Judgement calls for the human:** ACC-36 passed on the edit branch rather than ALPHA, and ACC-16 passed by reinterpretation. Both are disclosed.

VERDICT: fail