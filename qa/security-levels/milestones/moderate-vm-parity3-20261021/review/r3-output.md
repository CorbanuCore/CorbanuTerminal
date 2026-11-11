I found no required corrections. All four optional points from round 2 are applied, and each one matches the evidence. Nothing else changed.

**The four round 2 points**
1. **T17-tart3, two "Remove managed crew" confirmations:** correct.
   - In `runs/T17-tart3/collected/evidence/actions.jsonl`, Status was entered at 20:44:55Z and again at 20:45:20Z, and each was followed by Esc (20:45:02Z and 20:45:29Z).
   - In `screens.jsonl`, the two `mod21-status-before-send` screens at 20:44:56Z and 20:45:21Z are the only ones that show "Remove the complete managed crew?".
   - `dispositions.md` and `results.json` now say the same thing.
2. **The T18-utm1 and T18-utm3 deviation rows:** correct.
   - **T18-utm1:** `result.md` line 36 (with lines 83 and 96) shows that MOD-07 step 1 at 18:17:47 came after action 1 was saved at 18:17:40. It drew a Moderate approval prompt, and the tester pressed Esc.
   - **T18-utm3:** `result.md` lines 32 and 57 show that in the redo, step 1 was typed at 19:20:43 while the Saved screen was still open. It never reached the agent and was sent again at 19:21:50.
3. **The latency row:** correct.
   - "Upper median" matches `tools/transport_check.py` line 44 (`secs[len(secs)//2]`).
   - Every Tart window does start with that clone's gate requests (a GLM 5.3 Flash POST and a GET). See `runs/T*-tart*/mediator-window.jsonl`.
   - The figures still match `transport.json`: UTM T17 is 36.2–160.3, Tart T17 is 24.0–39.2, UTM T18 is 6.3–7.2, Tart T18 is 5.6–7.1 and T09 is 5.3.
4. **Decision 2, how the key file was handled:** correct.
   - `harness/agms-probe` line 112 tries to open the key as the executor accounts (line 106 also tries through a symlink), and line 113 does the same for `auth.json`. Both are expected to be denied, and neither reads the file.
   - `harness/capture-policy.sh` line 8 lists only the files' metadata (`stat`).
   - `common/utm/policy-root-start.txt` lines 102–103 show `auth.json` and `zai_api_key` as `600 agent:staff`.
   - `auth.json` was already present in earlier UTM records, for example `moderate-20261014/common/policy-root-start.txt`.

**Nothing else changed.** `git diff HEAD~1` touches only `README.md`, `dispositions.md`, `results.json` and `review-ledger.md` (row 2), plus the review files. Nothing in `../moderate-amendment-v9-20261021/` changed. `results.json` still parses, and the only untracked file is `review/r3-start.txt`.

## Required corrections

None.

## Optional

- **Latency row:** the note says only Tart windows include gate requests, but T17-utm1's window also does. It holds UTM's gate requests (a GLM 5.3 Flash POST at 17:54:03Z and a GET at 17:54:04Z), which pull its upper median down from 65.9 s to 50.4 s. The reported 36–160 s range does not change. Possible wording: "the first window on each machine (every Tart clone, and UTM's T17-utm1) includes gate requests". Evidence: `runs/T17-utm1/mediator-window.jsonl` and `transport.json` (T17-utm1: 4 product requests, median 50.424).

VERDICT: APPROVE