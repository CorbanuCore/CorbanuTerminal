I'm asking for changes. Round 1's nine required changes are applied and the packet is still clean. Two problems remain, one in the harness and one in the method: the per-clone evidence folder breaks as soon as a group runs a second time, and "product-model variance" has no check that would rule out the Tart network path.

## Check of round 1's required changes

1. **Scope: applied.**
   - The Scope paragraph in `navigation.md` overrides v3's "Dispositions without execution" (which covers MOD-11) and v4's R-1 line about MOD-09/21. It keeps MOD-06's git part not run, which matches v3's own MOD-06 action 4.
   - The T07 and T17 notes repeat v3 R-1 word for word in substance (`--model glm-5.2`, repeated at any relaunch).
   - The T13 note matches v4, which already says the tester has nothing to do.
   - `packet-vs-v4.diff` shows that only `navigation.md` and `prompts/` differ. The r1→r2 diff touches only the paragraph and the three notes.
   - No note adds an expected result or hints at an outcome.
2. **MOD-06: applied.** The baseline is targeted1 Q02 (passed, D43).
3. **Version: confirmed.** `build-head.txt` shows 0.1.48 for 8c73c8ea80.
4. **Evidence folder and gate: applied for a first run of each group, but see required change A.**
   - `PAR_COMMON=common/<G>` is set, and `seq.log` names the folder.
   - `group` checks the probes, `AGMS-SMOKE-OK` and `models 200`.
   - The memory pause fails safe: an empty reading counts as 0%, so it keeps waiting.
   - `guard_slot` counts every Virtualization guest and allows only one Tart VM.
5–8. **Method: applied.**
   - `baseline.json` has 34 cases on 3 baseline candidates: 17 + 15 + 2. My round-1 count missed MOD-32; the method's 15 is right.
   - Owner dispositions apply only when the same limit repeats, and MOD-11's results that D43 doesn't cover are judged on their own.
   - Attribution categories, the tie-break using the baseline candidate on Tart, the cap of 6 and the pass rule are all predeclared.
9. **Environment manifest: applied.** The new lines in `up` record tool versions, `who`, the loginwindow and SecurityAgent counts, and hardware. Per-group memory comes from `memory-before`/`memory-after`, and latency from the mediator logs.

## Required changes

**A. A second clone of the same group reuses `common/<G>`.** This affects both evidence and gate safety. The method runs replays "in a fresh clone", tie-breaks and re-gated groups, and `run-groups.sh T07` puts all of them in the same `common/T07`.
- **Lost evidence:** the second clone overwrites the first clone's `vm-start.txt`, `seal.out`, `mediator-final.jsonl`, `erase-test-data.out` and `vm-state-end.txt`, and appends to its `tunnel.log`.
- **Stale gate result:** `grep -q '"ok": true' "$C"/gate/gate-*.summary` (and the same `smoke-*.jsonl` test in `run-groups.sh`) passes if *any* earlier summary passed. A clone whose own gate failed could therefore start a group, which defeats round 1's change 4.
- **Fix:**
  - Make `PAR_COMMON` unique per clone, for example `common/<G>[-<label>]-<UTC stamp>`.
  - Have `up` refuse to start if `$C` already exists.
  - Have `gate` write its run id `R` to `$C/gate/current`, and have both checks read only that run's summary and smoke file.

**B. Attributing a difference to "product-model variance" needs a transport check, and the replay rule needs one clear meaning.**
- **The gap:** variance does not count against parity, and the main Tart-only mechanism is the network path (SSH reverse tunnel plus host relay). A dropped tunnel or a slow or erroring relay can look exactly like the model declining or not attempting.
- **Add a rule:** "product-model variance" (and tester variance caused by a timeout) may be assigned only if:
  - the guest mediator and host relay logs for that window show every upstream request completed normally (no non-200, no timeout, no `ssh exited` or reconnect in `tunnel.log`);
  - otherwise the difference is environment, or undetermined.
- **Fix the replay sentence.** "The first attempt counts" contradicts "matched only if the replay matches the baseline". State one rule. For example:
  - the first attempt is the one that counts;
  - a variance attribution stands only if the replay matches the baseline;
  - if the replay differs the same way, the difference is undetermined.

## Optional changes

- In the D43 line of `parity-method.md`, say that MOD-06's git part is not run on either machine, so its D43 disposition carries over and the comparison covers the rest of MOD-06.
- State that the baseline's outcome for each expected result comes from each `utm_run_id`'s `result.md` and evidence check. `baseline.json` only holds verdicts and summaries.
- Replays should use the group's frozen prompt, with the label only in the run name. Leave `make_packet_par.py`'s `REPLAYS` note unused, since it would change the prompt. Generate and hash the tie-break prompts and packages at freeze time, including the version string, which may not be 0.1.48 on run 1's candidate.
- Make the gate checks parse the JSON's top-level `ok`, and look for the smoke token in the agent's reply item, rather than grepping for the strings.
- In the tie-break, a change on Z.AI's side since 10-14 would show up as "environment". That errs on the safe side, but say so in the record.
- `make_packet_par.py`'s docstring still says "plus one test-machine paragraph".

VERDICT: CHANGES
EXIT:0
