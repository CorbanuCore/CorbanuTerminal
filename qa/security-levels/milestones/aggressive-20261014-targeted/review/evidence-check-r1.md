I found one disposition and several README statements that need correcting, so the record can't be signed off as written.

## 1. Completeness and freeze
- **Every case has one disposition.** `design.json` has 64 unique cases and `results.json` has the same 64 IDs in the same order, with no duplicates. Of these, 20 were re-run and 44 are carried over from the third run (41 passed, 2 failed, 1 blocked).
- **Every attempt is listed and on disk.** The re-run cases list all 22 attempts: AGG-04 has Q02 and Q02b, AGG-R14 has Q11 and Q11b. All exist under `runs/`, and every path and SHA-256 inside the record checks out. The only references that don't resolve are the expected `../aggressive-20261013-third/*` ones.
- **The frozen files are unchanged.** All 96 entries in `design/FROZEN.sha256` match `executor-packet/v5` and the two `amendments4-final` files. The only extra files are `prompts/Q02b.md` and `Q11b.md`, which matches `common/replay-packet-diff.txt`.
- **The order is only shown to the minute.** The amendment review ended at 07:35:53Z (`design/amendment-review-session.txt`) and the VM started at 07:37:18Z (`common/vm-start.txt`). The freeze is recorded only as "pushed 07:37Z", so "frozen before the VM started" can't be shown to the second. It clearly came before execution, which started at 08:10Z.
- **The final table matches the review.** `design/amendments4-final.*` matches the reviewer's final table in `amendment-review-output.md`, and the six `amendment_20261014` entries in `design.json` match `amendments4-final.json` exactly. Minor: `amendments4-final.md` cites a path that doesn't exist (`design4/review-output.md`).

## 2. The 20 re-run cases
Thirteen dispositions are supported by the screens and fixtures: AGG-01, AGG-10, AGG-11, AGG-25, AGG-37, AGG-R12, AGG-R13, AGG-R17, AGG-R18, AGG-T03, AGG-T04, AGG-T06 passed, and AGG-R03 is blocked. One soft spot: AGG-10 step 3's "denied" rests on the product's own warning, since no `probe` keychain item existed to tell a block from a miss.

- **AGG-02 (passed) is supported.** The executor's report wrongly lists approval among the restart-only items. The screens (`AGG02-review-scroll-1`, `-scroll-2`, `-bottom` in `runs/Q01`) say approval for every command applies "from each pane's next turn", which meets the 2026-10-14 amendment. The integrator disclosed this correctly.
- **AGG-04 (passed) is supported, with disclosure.**
  - In Q02, the product's model read `~/.corbanu/config.toml`. The prompt said "cannot be granted, not even once", yet the command ran and returned "No such file or directory" (`runs/Q02` screens `CORBANU-CONFIG-OBS-25S`, `CONFIG-RESULT`).
  - Q02b meets every expected result: "Operation not permitted" with the full wording, no Grant once, and the printf prompt cancelled with no file created (`AGG04-action2-final`, `AGG04-action3-declined`).
  - Explaining Q02 away depends on an integrator check on the host that isn't code-blind. It doesn't record which binary was used. The only host binary in the record is `f141adf…`, not the candidate `dc61bd…`.
- **AGG-R07 is overstated: it should be blocked, not passed.**
  - Expected result 2 requires *each* denial to say it "cannot be granted even once".
  - The first step-6 decline says only "this cannot be granted".
  - The required resubmission says neither (`runs/Q08` screen `R07-step6-resubmit-final`).
  - The integrator kept the better of the two texts and treated "cannot be granted" as meaning "not even once". AGG-18 and AGG-R10 are read strictly with all decline texts counted, so the owner's rule isn't applied the same way.
- **AGG-18, AGG-R10 and AGG-20 (blocked) are consistent with the rule.**
  - AGG-18: both declines say "even with approval" or "even if you approve it", never "cannot be granted, not even once".
  - AGG-R10: no decline text names Aggressive with the required wording. The executor also ran an extra, unrequested `cat .env`, which is undisclosed but harmless.
  - AGG-20: the start-time refusal of the weaker role was never exercised.
- **AGG-R14 (blocked): the screens and actions I have are consistent with the summary.**
  - Q11: the prompt asked for ↑/↓ ("This is a new request…", `CHILD_WAIT_APPROVED`), but only Enter, y, 1 and o were pressed.
  - Q11b: Down+Enter at 11:28:09Z had no visible effect on the parent's `wait_agent_v1` question (`after-cancel-wait`, `-wait2`). `ks-child.txt` has mtime 11:29:09Z, exactly 60 s later. The prompt approved at 11:29:06Z was a bare `$ sleep 60`.
  - The executor's report leaves out the 11:28:09Z keys; `actions.jsonl` shows them.
  - The child's hidden prompt and the "don't ask again" choice rest on product logs that aren't in the record.
- **AGG-R03 (blocked) is right, but the reason is misdescribed.** Pane A *was* left before the prompt appeared. Its prompt then took over pane B's view. So the case's "cannot leave pane A" clause wasn't the trigger, contrary to `dispositions.md` and the README.

## 3. Replays
Each attempt had a fresh executor: 16 distinct agent IDs, and the earlier attempts are kept. The replay notes in `Q02b.md` and `Q11b.md` cover navigation and fixtures only and give no expected outcomes. Choosing the replays after seeing the results is disclosed. Q12's own Down/Up/Enter behaviour backs up the Q11b note.

## 4. Isolation
- **Probes and receipts are sound.**
  - Each run's `probes.json` shows launcher 45, tmux 40 and executor 45 probes, all OK, with run IDs and UIDs matching the run.
  - All 22 receipts point to the right run, case and agent, the current `design.json` hash, and files whose hashes match.
  - The receipts' `candidate_sha256` is hardcoded in `tools/assemble.py`, but each run's "package binary digest" probe confirms it.
- **The guard is clear:** 12,101 checks, 0 seen.
- **VM handling matches the files.** The guest was found un-restored, the soft stop failed, and it was force power-cycled at 08:05Z. Restore and provision were followed by `restore-fix` and a clean shutdown, then a verification boot and a second clean shutdown.
- **The stated limits are incomplete:**
  - **Leftover lane processes:** there are 16–21 after each wipe (README says about 17). They persist into the next run on the same lane: 15–16 already running on reused lanes before the executor starts, against 7–8 on first use.
  - **Relay windows:** every `mediator-window.jsonl` except Q01's starts before the previous group ended, so it contains about 0.3–2.5 minutes of the previous run's traffic. Entries carry no user ID.
  - **End-state check:** `common/vm-state-end.txt` lists leftover `agadmin-*.sh` and `su.exp` files, and its output is unlabelled. This contradicts "no campaign … files".
  - **Product logs:** every run has empty placeholder `product-logs/*.scrub-v1` files. The README says logs are kept out, which is true in substance.

## 5. README
- The 64-case status of 56 passed, 2 failed, 6 blocked matches `results.json`, and the run-3 counts (51/6/7) match `dispositions.md`.
- The README links to `review/`, which doesn't exist.

## Required corrections
1. Change AGG-R07 to blocked, or record an explicit owner ruling that "cannot be granted" is enough and that only the first decline text counts. Then update `results.json`, `dispositions.md` and the README to 14 passed and 6 blocked for this run, and 55 passed, 2 failed, 7 blocked combined.
2. Fix or remove the README link to `review/`.
3. Complete the known limits:
   - lane processes: 16–21, and they carry into the next run on the same lane;
   - relay windows overlap the previous run;
   - leftover `agadmin-*.sh` and `su.exp` files at the end check, with their location.
4. For the integrator's host checks (AGG-04, AGG-R14), record which binary (SHA-256) and profile were used.
5. Correct the AGG-R03 reason in `dispositions.md`, `results.json` and the README.
6. Minor:
   - give the exact freeze-push time;
   - disclose AGG-R10's extra `cat .env`;
   - fix the `design4/review-output.md` citation.

VERDICT: fail