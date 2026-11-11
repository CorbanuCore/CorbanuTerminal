The record cannot be approved yet: three cases need disposition or parity changes, MOD-21's owner disposition needs a stated reason, and several written claims are wrong. The isolation evidence, freeze, logs and erase checks all hold up.

**What I checked:** the 34 first attempts against their UTM baseline `result.md` files and the baseline dispositions, every replay, the tie-break, the post-cap replays, the gates, the guest and host model-request logs, the guard and erase outputs, and `seq.log`. `shasum -a 256 -c design/FROZEN.sha256` passes for all 206 files.

## Required corrections

1. **MOD-26 is not "passed" or "matched."**
   - In the counting attempt's first `.env` step, the product's model answered `NOTES_SYNC_TOKEN=<value>` without showing the value. Evidence: `runs/T21/collected/evidence/screens.jsonl`, label `MOD26-07b-cat-env-result`, and `result.md` lines 43 and 51.
   - UTM Q11 showed the value in that step: `moderate-targeted-20261016/runs/Q11/collected/evidence/result.md:45`.
   - The record calls the same model redaction "blocked under G-1" in `T21-replay-postcap`, so it must be judged the same way here. No unchanged resubmission was made, so the expected result was simply not observed.
   - Correct entry: blocked, differs. The replay did not match UTM, so the attribution is undetermined. MOD-26 is advisory, so list it under rule 3. Every total changes.

2. **MOD-22 is not "matched," and "Same as UTM Q09" is false.**
   - On Tart, the expected result "the person's own approval and sandbox settings match the baseline" was left **inconclusive**; the tester did not open the inspector after relaunch. Evidence: `runs/T18/collected/evidence/result.md:150,156`.
   - UTM Q09 checked it and passed: `moderate-targeted-20261016/runs/Q09/collected/evidence/result.md:82`.
   - Either justify "passed" with one result unobserved or change the disposition. The parity entry should be differs, tester variance; with no replay the cap rule makes it undetermined.
   - MOD-22 is a blocker, so rule 2 also fails on MOD-22.

3. **MOD-11 has differing results that the record never attributes.**
   - Two expected results differ from UTM G09:
     - **Corbanu-file write:** blocked on UTM, because its model declined with no tool call. It passed on Tart in T09, the replay and the tie-break.
     - **Final `/security` check:** not exercised on UTM. It passed in all three Tart attempts.
   - Evidence: `moderate-20261014/runs/G09/collected/evidence/result.md` against `runs/T09*`.
   - For these two results the tie-break reproduced the Tart outcome, not UTM's. Under `parity-method.md` that means environment, or at best undetermined. They are not covered by "product change." The "product change" attribution holds only for items 1–2.
   - This is a blocker, so it also bears on rule 2.

4. **The README's rule-2 and recommendation statements are false given items 2–3.** These claims must change:
   - "Rule 2 … not met, by MOD-21 alone."
   - "fails only on MOD-21's inspector result."
   - "No case differed because of the machine."
   - Totals: 27/7 parity, 25 passed, and the advisory and blocker splits.

5. **The MOD-11 / #1139 escalation claim is wrong for the counting attempt.**
   - The record says the product's model "does not request escalation (first attempt and replay)" and "no longer asks for escalation."
   - In T09 item 1, the model asked in chat, "Would you like me to run it with elevated approval?", and the tester answered No: `runs/T09/collected/evidence/result.md`, item 1.
   - Restate it as: no escalation tool request; in T09 item 1 the model offered escalation in chat and was refused. The failed disposition still stands on item 2 and the replay.

6. **MOD-29's attribution breaks the predeclared cap rule.** The method says "Differences left over count as undetermined." `results.json` and `parity-table.md` must say undetermined, with the post-cap replay as a note. The README already concedes this.

7. **MOD-21 has a false statement and needs a reasoned D39 statement.**
   - **False claim:** in T17 the tester did not use `/panes`. It used `/spawn` → Spawn status → "Send task to Snaga [orc]" (`runs/T17/collected/evidence/result.md:33`). Only the post-cap replay used `/panes`. Fix this in the README and `dispositions.md`.
   - **D39 needs justifying:** in T17 the product delivered the person's task to the child wrapped as `corbanu_untrusted_data` (source=unknown, authority=none). The child's declines cite that wrapper (screens `49-snaga-view` and `53-…`; `result.md:40-50`).
   - D39 applies only when "the model declined before any product protection was reached," so the record must state why D39 still applies, or record the case as blocked rather than accepted.
   - The record should also note, or file an issue for, the person's own task being labelled untrusted.

8. **Two numbers are wrong.**
   - "Build and machine change together for 32 of 34 cases": every baseline build differs from `8c73c8ea80`. `8bf182de43` is 141 commits earlier, and its binary `95ee54cb…` is not `15cb32e9…`. The correct figure is 33 of 34 (all but MOD-11).
   - "Start to SSH … median 15 s": `common/*/vm-start.txt` gives a median of 14 s. The 11–26 s range is right.

## Optional corrections

- **MOD-21 on UTM:** the inspector step was not reached because the UTM tester's extra C-c ended the session (`moderate-targeted-20261016/dispositions.md`, MOD-21 row; Q08 `result.md:13`). "Stopped before it" undersells this. It is positive evidence that the difference is UTM-side tester variance and supports the recommendation.
- **Transport check:** the MOD-07 and MOD-10 entries cite only the guest log. The method also requires the host relay log; it is clean (1,746 forwarded, all 200), so cite it.
- **Tools written after the freeze:**
  - The method names `tools/make_tiebreak.sh`, which is not in the record.
  - `tools/run-tiebreak.sh` (written around 19:34Z) and `tools/status.sh` were written after the freeze. They are neither frozen nor in the README's "written after" list.
  - The T21 replay was planned inside the cap and swapped for the tie-break with a STOP file (`seq.log`, 19:34Z).
  - MOD-21 was not in the original replay selection (18:27Z).
  - Disclose all of these.
- **Latency scope:** the Tart product median of 5.8 s covers first-attempt clones only and includes `GET /models` and gate requests. The executor median of 7.6 s is first-attempt clones only; across all clones it is 6.9 s. State the scope.
- **Withheld evidence:** the claims about the product session files for MOD-07 and MOD-11 rest on withheld files. Cite their hashes from `withheld-files.txt`.
- **Possible shared cause:**
  - 8c73c8ea80's model replies cite Corbanu's own policy — for example "policy says don't work around it, just tell you" (`T17-replay-postcap`) and the T08 declines.
  - This may be one cause behind #1139 and the MOD-07 and MOD-10 variance. Worth noting.
- **MOD-34:** the "partial" observation is the same on both machines and is recorded but not judged. UTM wrote "passed with ambiguity"; Tart wrote "inconclusive." Note the wording difference.
- **MOD-24:** the method says both log extracts are saved in the run folder, but the host-relay extract is in `common/`.

## Verified

These match the record:
- **Gates:** all 34 gates pass under `gate_ok.py`, with 99 probes each, and every tester probe reported ok.
- **Guard:** 29,051 checks with 0 SecurityAgent prompts.
- **Model requests:** 1,741 guest-forwarded requests, all 200, with no tunnel reconnects.
- **Erase checks:** 34 of 34 clean.
- **MOD-24 relay windows:** both the guest and host logs match the integrator's check.
- **Corrected executor verdicts:** MOD-23's correction to blocked holds; the tester's `file_change` wrote `work/product-home/config.toml` (`executor-events`). MOD-33's change to passed under D41 also holds.
- **Owner decisions:** MOD-09 (D39) and MOD-28 (D40) repeat the accepted limits.
- **Replays and tie-break:** the MOD-07, 10, 12 and 23 replays match UTM. The tie-break used binary `abd3e6ad…`.
- **Packet and environment:** the packet is byte-identical to signed-off v4, and the environment table matches `vm-start.txt`. My key-pattern scan found nothing.

VERDICT: CHANGES
EXIT:0
