I found blocking problems in four areas: two contradictions between the brief and the packet testers will actually read, a baseline-candidate error in the attribution method, a parity method that cannot separate environment from product change, and missing pass and replay rules. The harness port itself is close to sound. Everything below comes from reading the files in this folder; I ran nothing.

## Findings

**(1) Packet faithfulness, groups and notes**
- The copied texts and the navigation edits are faithful. The only differences from v4 are in `navigation.md` and `prompts/`, as the brief says.
- The groups match run 1's groups, with MOD-28 split off. The six standing notes are navigation only, and each is the note the case's baseline run used.
- **The tester will be told not to run three things the brief says to run.** The tester reads amendments-v4, then v3, in that order of precedence:
  - **MOD-09 and MOD-21 (T07, T17):** v4 R-1 says R-1 "does not apply: MOD-09 and MOD-21 are not re-run (D39)", then "the product's model is GLM 5.3 Flash". A T07/T17 tester may skip the case or launch without `--model glm-5.2`. Nothing in the prompt says v3 R-1 is the text to follow.
  - **MOD-11 (T09):** v3's "Dispositions without execution" says "MOD-11: not re-run". Nothing in the prompt overrides it.
  - **MOD-06's git part (T04):** v3 replaces action 4. The git add/commit "is not run, in the baseline or under Moderate", and the `git log -1` result is "not judged". The brief's "MOD-06's git part … run as written" therefore contradicts the case text in force. Running it would need a group note that overrides case text, which breaks "no new case text".
- **The version in every prompt may be wrong.** All 26 prompts say "Corbanu Terminal 0.1.48 package". That came from the older template. If 8c73c8ea80's `--version` (in `build-head.txt`, not in this folder) is different, a tester may see a mismatch with the header and stop the case.
- Minor: the new navigation paragraph tells every tester that "This run repeats every Moderate case". It is only needed for T13 (MOD-15).

**(2) Case-specific choices**
- **MOD-09/21 on GLM 5.2:** sound. It is the text in force, and it compares with targeted1 Q04/Q08 (both blocked by the model declining). What's missing is a rule for when D39 applies. D39 accepted "never reached the product". If the product's handling is reached on Tart and an expected result fails, that must be recorded as failed, not accepted.
- **MOD-11:** comparing with run 1 and applying D43 only if the failure is the same `.git` read-only limit is sound. Run 1 also had other unmet or unexercised results: item 2's Reason, no "don't ask again" offered, and the final check. Those are not covered by D43 and must be judged separately. #926 makes a different raw outcome likely here.
- **MOD-33:** comparing the raw outcome with targeted1 Q14 and applying D41 only if the same note appears is sound.

**(3) Harness port**
- Isolation and method are preserved. The guest product only ever gets a placeholder key, on UTM as on Tart, and the real key now stays on the host. There are 99 probes, including the tunnel denials. pf is per account, DNS is sealed, the shakedown passed, and erasure was verified. Splitting the mediator into a guest mediator plus a host relay keeps one request per connection, and the effective upstream timeout is still 600 s.
- **Weak points:**
  - Evidence folders: `C=$Q/${PAR_COMMON:-common}` defaults to one shared folder. Without a separate `PAR_COMMON` per clone, each clone overwrites `seal.out`, `gate/`, `mediator-final.jsonl`, `erase-test-data.out` and `vm-state-end.txt`.
  - Gate: `group` never checks that this clone's gate passed. It checks the probe preflight, but not the smoke turn or the relay control.
  - Memory pause: the "pause while free% < 20" rule is not in `tart-run.sh`. It only logs `memory_pressure`, and `lib.sh`'s `guard_slot` is not in this folder, so I couldn't verify it.
  - Password-prompt guard: it compares against a login-window SecurityAgent baseline, as on UTM. Whether a headless Tart clone sits at the login window (or can show a system password prompt at all) isn't shown. This matters for the case-wide "no system password prompt" rule and for MOD-27.

**(4) Parity and attribution method**
- **The baseline candidate is wrong for most cases.** "Product change since 8bf182de43" only fits MOD-15 and MOD-28, the only cases run on 8bf182de43 (targeted2).
  - 16 cases go back to run 1's candidate (2026-10-14, before #813): MOD-02/03/04/05/07/12/13/14/16/18/19/23/25/27/31/34.
  - 14 go back to targeted1's candidate (before #900): MOD-01/06/08/09/10/17/20/21/22/24/26/29/30/33.
  - MOD-11 goes back to run 1.
- **Build and machine change together.** A difference between Tart on 8c73c8ea80 and UTM on an older build can't be attributed from the record alone.
- **Two kinds of model variance are merged.** The tester model and the product's own model are different sources and need separate categories.
- **No rule decides whether Tart "matches UTM" (D34/D68).** There is also no replay policy.

## Required changes
1. **Scope notes for T07, T17 and T09**, marked navigation only and reviewed in round 2:
   - **T07, T17:** run the case per amendments-v3 R-1 (`agms-tui start --model glm-5.2`; relaunches repeat it). v4's R-1 lines describe the second re-run only.
   - **T09:** MOD-11 is assigned. Run it per `design-cases.md` and `amendments.md`. v3's "Dispositions without execution" describes the first re-run only.
2. **MOD-06:** follow v3 (git part not run, compared with targeted1 Q02) and correct the brief. If the git part is wanted, treat it as an extra owner-approved observation outside the parity comparison.
3. **Version string:** check `build-head.txt`. If the version isn't 0.1.48, fix the version text in the prompt template and regenerate the prompts.
4. **One evidence folder per clone:** require a unique `PAR_COMMON` per group (e.g. `common-T01`), named in `seq.log`. Require each clone's gate result (probes ok, `AGMS-SMOKE-OK`, relay control 200) to be recorded before its group starts.
5. **Per-case baseline columns in the parity table:**
   - baseline record and attempt (run1 Gnn / targeted1 Qnn / targeted2 Qnn);
   - baseline candidate;
   - amendment set it was judged under (v2 for run-1-only cases, v3, or v4);
   - product model;
   - raw outcome **per expected result** (not just the verdict);
   - integrator disposition and owner decision.

   Tart is judged on the same text as each case's baseline. Known baseline deviations, such as run 1's MOD-03 README skip, carry over.
6. **Attribution categories:** environment / product change since **that case's baseline candidate** / tester (executor) variance / product-model variance / undetermined. Each needs positive evidence: a named PR whose described change matches the observed text, a deviation by the tester from the case text, or a Tart-specific mechanism.
7. **A tie-break for every difference that isn't clearly product change:**
   - Rerun that case on UTM with 8c73c8ea80, or on Tart with the baseline candidate if it is still available.
   - Otherwise mark it "undetermined", which counts against parity.
   - Predeclare the replay policy: at most one fresh-executor replay with an unchanged prompt, both attempts kept, and which one counts.
8. **Predeclare the parity pass rule**, e.g.: no difference attributed to environment or left undetermined among blockers; variance differences resolved by the replay rule.
   - Also state that D39, D40, D41 and D43 apply only when the raw outcome repeats the accepted limit. A newly observed failure is "failed".
9. **Environment manifest for both machines:**
   - macOS version and build, and virtualization;
   - guest `python3`, `git` and `bash` versions;
   - network path (direct vs tunnel + host relay);
   - console/login-window state and guard evidence (`guard-final`);
   - per-group memory pressure and model latency;
   - run dates (Z.AI may have changed its models since 10-14).

## Optional changes
- Make `tart-run.sh group` refuse to start unless this clone's gate summary and smoke passed. Put the memory pause into the script.
- For MOD-24, cross-check the guest mediator log against the host relay log for the group's time window, and save that slice into the run folder.
- Move the "UTM macOS 3" paragraph from the shared navigation into T13's note only, and drop "repeats every Moderate case".
- Keep running `keyscan.sh` on collected evidence, as on UTM.
- List, for each group, which notes its baseline run used (including "none" for T13/T23 and the targeted1 groups without notes), so it's visible that nothing was dropped.

VERDICT: CHANGES
EXIT:0
