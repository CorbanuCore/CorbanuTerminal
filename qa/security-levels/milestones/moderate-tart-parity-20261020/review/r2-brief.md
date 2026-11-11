# Review round 2 of 5 (packet/plan re-review) — tart-par1, Moderate milestone parity run on Tart

You are an independent, read-only reviewer. Read only files in this folder; run nothing. Round 1's brief is
`r1-brief.md` and its review (VERDICT: CHANGES) is `r1-review.md`. Check that each required change was applied
correctly and that nothing new is wrong. This is the last review before the freeze.

## How round 1's required changes were applied
1. Scope: `packet-v4p/navigation.md` now ends with a Scope paragraph (every assigned case is run; v3's "Dispositions
   without execution" and v4's R-1 line on MOD-09/21 don't apply; MOD-06's git part stays not run per its v3
   amendment). Group notes (`standing-notes.json`): T07 and T17 run per v3 R-1 with `--model glm-5.2`; T13's
   test-machine sentence (moved out of the shared navigation, optional change). The shared "repeats every case"
   paragraph is gone. Diff vs round 1: `packet-r1-to-r2.diff`; vs the signed-off v4 packet: `packet-vs-v4.diff`.
2. MOD-06: follows v3 (git part not run); compared with targeted1 Q02.
3. Version: `build-head.txt` shows 0.1.48 for 8c73c8ea80, so the prompts' "0.1.48" is right; unchanged.
4. One evidence folder per clone: `run-groups.sh` sets `PAR_COMMON=common/<group>` per group and logs it; `tart-run.sh
   group` now refuses to start unless this clone's gate summary is ok, the smoke turn printed `AGMS-SMOKE-OK` and the
   relay positive control returned 200 (`tart-run-r1-to-r2.diff`). The memory pause (free% < 20) is in `run-groups.sh`
   before each clone; `lib.sh` (guard_slot: one Tart VM, no third macOS guest) is included.
5-8. Predeclared method: `parity-method.md` (per-case baseline from `baseline.json`, built by `make_baseline.py`;
   dispositions incl. when D39/D40/D41/D43 apply; classes and attribution categories with required evidence; replay and
   tie-break rules with a cap of 6; the parity pass rule).
9. Environment manifest: `tart-run.sh up` now records python3/git/bash versions, who, loginwindow and SecurityAgent
   process counts and hardware per clone; the UTM side comes from the earlier records (`parity-method.md`).
Optional changes taken: MOD-24 host-relay cross-check, keyscan on exports, notes listing per group (all in
`parity-method.md`), the T13 note. Not taken: none of the others.

## Your task
Confirm each required change is correctly applied and that the packet still adds no expectation or bias (the notes are
scope/navigation only), the method is sound and bounded, and the harness changes are safe. List required changes
separately from optional ones. End with exactly one line: `VERDICT: APPROVE` or `VERDICT: CHANGES`.
