I checked the record only, as scoped. **r3's four corrections are applied correctly, and results.json matches dispositions.md and design.json. But the new README "Before sign-off" section misstates one point, and one token-exposure claim is broader than the record supports.**

## 1. r3's four required corrections

1. **AGG-R08: fixed.**
   - The new summary is identical in `results.json` and `dispositions.md`.
   - The screen `26-r3-after-15s` (22:32:26Z) shows "List the files in P" answered by the model's own text, with no "Explored" line and no product denial.
   - In `runs/G25/mediator-window.jsonl` there is a single product-sized request at 22:32:08–15Z (43,710 bytes, 14 tools). It has no follow-up; the next request is executor-sized (165 KB).
   - Step 3 of the design case does include "List the files in P".
2. **Replay reasons: fixed.**
   - `tools/replays.json` now gives a `reason` for all five replays. G24b's reason agrees with the AGG-R07 summary.
   - Each `note` appears word for word in its prompt in `executor-packet/v4/prompts/`. Comparing each replay prompt with its original group prompt shows only the label, the case list and the appended note changed.
   - The README now says the reasons were added after the replays ran, which is accurate.
3. **Token-exposure limit: fixed, with one overstatement.** I searched the whole record, including gzipped files, for the synthetic token.
   - The README's numbers are right:
     - 27 of the 37 executor transcripts;
     - 26 `fixture.jsonl` files;
     - the three named fixture files;
     - the G08 and G19 reports.
   - G13 and G30 got the token in their own command output.
   - The token is not in any screen, PTY log, mediator file or product log.
   - **Overstatement:** it is in the r1 and r2 checker transcripts only, not r3's. The README says "the evidence checkers' transcripts", which includes r3.
4. **Shutdown: fixed honestly.**
   - `common/vm-state-end.txt` says plainly that the stop output was not saved. It records the integrator's account of the exit status and a 02:29:04Z re-capture showing both VMs stopped.
   - The ~01:41Z time is consistent with `restore-end.out`, whose guest clock reads 01:40:18Z.
   - The README describes this accurately.

## 2. Consistency

- **Cases:** `design.json` has 64 unique cases, and `results.json` has one disposition for each, in the same order.
- **Counts:** 51 passed, 6 failed, 7 blocked.
- **Hashes:** `results.json:design_sha256` matches `design.json`. All 1,555 path/SHA-256 pairs resolve and match.
- **dispositions.md:** all 64 rows match `results.json` on disposition and summary, word for word.
- **README lists match the dispositions:**
  - the failed and blocked lists;
  - the cases the integrator changed;
  - the comparison with run 2, against the Run 2 column;
  - the model-decline disclosures.
- **Pending after r4 (not errors):**
  - `results.json` still records r3 under `evidence_check` and 5 of 5 under `review_budget`.
  - Ledger row 6 points to a "row 7" that does not exist yet.
  - Both need updating once r4 is recorded.

## 3. Edits since r3

- The "Evidence check" section is accurate. I confirmed in r1 and r2 that they also verified the hashes, freeze, relay claims and isolation receipts.
- Ledger rows 5 and 6 are accurate, apart from the pending row 7.
- **Misstatement in "Before sign-off":** AGG-19's roughly 10-minute wait is listed under "Executor deviations reported only in the executors' reports". It is neither.
  - The cause is two relay `session_error` timeouts at 18:46:26–27Z in `runs/G07/mediator-window.jsonl`.
  - The G07 report only gives the send times, 18:36:24Z and 18:46:39Z. It doesn't mention a wait or the relay.
- **Minor omission:** the replay-selection bullet drops r3's point that G24b *did* replay a model-declined step, while AGG-18, R10 and AGG-20 were not replayed. The asymmetry is less visible as a result.

**Required corrections**
1. README "Before sign-off": give AGG-19's wait its own bullet, citing the relay timeouts in `runs/G07/mediator-window.jsonl`, not executor reports. Ideally, also restore that G24b replayed a declined step.
2. README token limit: change "the evidence checkers' transcripts" to "the r1 and r2 evidence-checker transcripts".

VERDICT: fail