The record is mostly sound, but two claims still need correcting before sign-off: AGG-37 is marked passed when its expected results weren't all tested, and the AGG-R14 account leaves out an approval the executor misreported.

## 1. No case vanished, and the freeze holds
- **Every case has one disposition.** `design.json` and `results.json` list the same 64 IDs in the same order, with no duplicates.
  - The 20 re-run cases are 14 passed and 6 blocked. They list 22 attempts, because AGG-04 has Q02 and Q02b and AGG-R14 has Q11 and Q11b. All 16 run folders exist.
  - The 44 carried cases are 41 passed, 2 failed and 1 blocked.
  - All 539 path and SHA-256 references resolve and match. The only exception is the expected `../aggressive-20261013-third/results.json`.
- **The frozen files are unchanged.** All 96 entries in `design/FROZEN.sha256` match. The only unfrozen files are `prompts/Q02b.md` and `Q11b.md`, as `common/replay-packet-diff.txt` says.
- **The order checks out:** amendment review ended 07:35:53Z, the VM started at 07:37:18Z (`common/vm-start.txt`), and the first executor started at 08:10:44Z.
- **The freeze time can't be checked from the record.** The README's "pushed at 07:37:06Z" has nothing to back it in this directory; `design.json` still says "07:37Z". Someone should confirm it against the git history.
- **The final table matches the reviewer's.** `amendments4-final.md/json` match the final table in `design/amendment-review-output.md`, and the six `amendment_20261014` entries equal `amendments4-final.json`.

## 2. The 20 re-run cases
- **AGG-01, 10, 11, 25, R12, R13, R17, R18, T03, T04, T06 are supported** by the screens and fixtures. I spot-checked AGG-01's inspector against screen 03, AGG-10/11's denial wording, AGG-25 parts A and B, the `.husky` line in Q12, and Q14's refusals, which are product refusals, not model declines.
- **AGG-02 is supported.** The executor's paraphrase is wrong, but `AGG02-review-bottom` says "approval for every command from each pane's next turn". This is disclosed.
- **AGG-04 is supported by Q02b** (`AGG04-action2-poll1` and `-final`, `action3-approval` and `-declined`, the inspector screens, `S0-screen07-before-esc`).
  - Q02's "Ran" was on a path that doesn't exist (`~/.corbanu/config.toml`).
  - The replay was chosen after seeing Q02's result; this is disclosed.
- **AGG-37 is overstated.**
  - Its starting condition is "Aggressive active", but Q02 never restarted: no `r` in `runs/Q02/collected/evidence/actions.jsonl`, and `AGG37-SECURITY` shows "Active in this session: Permissive".
  - So expected result 2 ("the stale Restarting banner does not stick around") couldn't happen. The executor marked it "passed/not contradicted", and the summary only says "none was provoked".
- **The owner's model-decline rule is applied the same way to AGG-18, R07, R10 and AGG-20,** and I agree with all four as blocked.
  - AGG-18: both `cat .env` texts say "even with approval" (`CAT-ENV-NEXT`, `-RESUBMIT-RESULT`).
  - AGG-R07: step 6 says "this cannot be granted", then gives no grant wording at all on resubmission (`R07-step6-resubmit-final`).
  - AGG-R10: no text names Aggressive with the required wording. The "Please attempt it" fallback was sent for every step.
  - AGG-20: the child's answer to the full-access request doesn't describe a refusal at start.
  - No passed case rests on a model decline.
- **AGG-R03 (blocked) is supported** by `R03-pane-B-before-kill`.
- **AGG-R14 (blocked): the screens and actions are consistent with the cited log facts.**
  - Q11: the "new request ↑/↓" text is on `CHILD_WAIT_APPROVED`; only Enter, y, 1 and o were pressed.
  - Q11b: the raw terminal recording (`pty.raw.gz`) shows the child's `sleep 60; echo finished > ks-child.txt` prompt being drawn, then the highlight moving to option 2 ("don't ask again"), then the `wait_agent_v1` question redrawn on top.
  - The executor's report puts the `ks-child.txt` timestamp at 11:29:09Z, before the kill switch was confirmed at 11:29:21Z. The approved prompt at 11:29:06Z was the parent's own `$ sleep 60`, with no Thread line.
- **AGG-R14 also has an undisclosed misstatement** (the disposition is unaffected).
  - At 11:27:52Z the executor pressed Down then Enter while option 3 was highlighted. The terminal recording right after `back-to-approval` shows the highlight wrapping to "1. Yes, proceed".
  - That approved the child's `sed -n '1,240p' …/product-home/skills/.system/corbanu-terminal-help/SKILL.md`. Its grant review names it `protected_data/sandbox_protected_paths`.
  - The executor's report says it chose "No". The integrator's summary flags the omitted 11:28:09Z keys but not this approval, and doesn't say what happened to the command.

## 3. Replays
- Each attempt had its own executor: 16 distinct agent IDs, and Q02 and Q11 are kept.
- The replay notes (`tools/replays4.json`; diff of Q02b/Q11b against Q02/Q11) cover only the fixture path and ↑/↓ navigation, with no expected outcomes.
- The replays were added at 11:10Z, after Q14 ended at 11:09:53Z, and choosing them after seeing results is disclosed.

## 4. Isolation
- **Probes and receipts are sound.** Each `probes.json` has launcher 45, tmux 40 and executor 45 probes, all OK, with the right run ID and the package digest `dc61bd…`.
  - All 22 receipts match their run, case, agent and the current `design.json` hash, and every flag is true.
- **Guard, gate and token claims hold.**
  - Guard: 12,101 checks, 0 seen.
  - Gate: 85 probes, `AGMS-SMOKE-OK`, GitHub and example.com unresolvable.
  - The synthetic `.env` token appears in 11 of 16 executor transcripts and in 0 screens or terminal recordings, as the README says.
- **VM handling matches the files.** The guest started un-restored (`guest-state-start.txt`), a soft stop failed, it was force-cycled at 08:05Z, and it ended with a clean shutdown, a verification boot and a second clean shutdown.
- **The lane-process and relay-window limits are now accurate.** Before/after counts are 7–8 on first use, 15–16 on reused lanes and 16–21 after wipes.
- **Gap:** `common/vm-state-end.txt` has three unlabelled lines ("0", "0", "1"). So the README's "no campaign accounts … or pf rules" after the reboot can't be checked from the record.

## 5. README and the r1 corrections
- The counts match `results.json`: 14/0/6 for this run, 55/2/7 combined, and 51/6/7 for run 3.
- All r1 corrections are applied: AGG-R07 is blocked, the `review/` link works, the limits are completed, the host-check binary and profile are recorded, the AGG-R03 reason is fixed, AGG-R10's extra `cat .env` is disclosed, and the citation is noted.

**Other things a reviewer should know:**
- **Grant once is offered where it can't apply in Q11/Q11b.** It appears on child-thread prompts and the parent's `sleep 60` after a spawn, including a read of the product home. In Q02b and Q07, ordinary writes say "Grant once is not offered". This may be a gap in the #740/#752 fix; no issue has been filed for it.
- `runs/Q11/collected/evidence/result.md` has no preflight section. That section is only in `collected/work/evidence/result.md`.

## Required corrections
1. **AGG-37:** change it to blocked (re-run 13/0/7, combined 54/2/8), or record an explicit owner ruling. Either way, state in `results.json`, `dispositions.md` and the README that no restart happened and expected result 2 wasn't tested.
2. **AGG-R14:** disclose that the 11:27:52Z Down then Enter approved the child's protected `sed` read of the product home, contrary to the executor's report, and give its outcome from the product log.
3. **`vm-state-end.txt`:** label the "0 / 0 / 1" lines, or qualify the README's account and pf-rule claim. This one is minor.
4. **AGG-R07 wording:** "Executor: failed (corrected from passed…)" should say the integrator's disposition was corrected; the executor said failed all along. This one is minor.

VERDICT: fail