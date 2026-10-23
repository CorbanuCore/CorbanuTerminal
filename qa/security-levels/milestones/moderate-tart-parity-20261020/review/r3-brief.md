# Evidence check, round 3 of 5 — Tart parity run of the Moderate milestone (tart-par1)

You are an independent, read-only evidence checker. Work in the current folder (`qa/security-levels/milestones/`).
The record under check is `moderate-tart-parity-20261020/`. The UTM baseline records are `moderate-20261014/`,
`moderate-targeted-20261016/` and `moderate-targeted2-20261020/` (signed off). Run nothing that writes; reading files,
`zcat`, `python3 -c` for parsing and `shasum` are fine.

## What to check
1. **Dispositions** (`dispositions.md`, `results.json`, built from `tools/tart-dispositions.json`): for each of the 34
   cases, does the Tart disposition follow from the evidence of the attempt that counts (the first attempt;
   `runs/<group>/collected/evidence/result.md` or `.../work/evidence/result.md`, screens, `actions.jsonl`, `pty.raw.gz`)
   under the rules in `design/parity-method.md` (failed if an expected result was observed to fail; blocked if a starting
   condition was not met or under G-1; owner dispositions D39/D40/D41/D43 only when the accepted limit repeats)? Look hard
   at every place the executor's verdict was changed (MOD-23 failed -> blocked; MOD-33 failed -> passed by D41; MOD-28,
   MOD-09, MOD-21 accepted) and at MOD-11 (failed, not accepted).
2. **Parity classes and attributions** (`parity-table.md`): against each case's UTM baseline (`design/baseline.json`;
   the per-result outcomes are in the baseline run's `result.md` and the baseline record's `dispositions.md`), is
   "matched" true for every expected result, and does each "differs" have the attribution the method allows, with the
   positive evidence it requires (replay matched baseline; transport check for variance: `common/<clone>/`
   `mediator-final.jsonl.gz`, `tunnel.log`; tie-break for MOD-11: `runs/T09-tiebreak-df3d8c5d1c/`)? Find any "matched"
   case where a per-result outcome actually differs from its UTM baseline.
3. **Method adherence and disclosure:** frozen inputs (`design/FROZEN.sha256`, `shasum -a 256 -c` from the record root),
   the replay cap and the two post-cap replays (`runs/seq.log`), the tie-break staging (`tools/run-tiebreak.sh`), the
   per-clone gates (`common/*/gate/`, `harness/gate_ok.py`), guard counts, erase checks (`common/*/erase-test-data.out`),
   the MOD-24 relay check (`runs/T20/mod24-relay-check.txt`, `common/host-zai-relay-T20-window.jsonl`).
4. **Written claims:** every number and statement in `README.md` and `common/effective-policy.txt` (totals, counts,
   times, the environment table, the recommendation's factual basis).

List required corrections (wrong disposition, wrong attribution, unsupported or false claim) separately from optional
ones, each with the evidence path. End with exactly one line: `VERDICT: APPROVE` (no required correction) or
`VERDICT: CHANGES`.
