# Evidence check, round 1 of 5 — UTM vs Tart three-attempt check (D139), amendment v9 (D140), MOD-11 on Tart (D141) (utm-par3)

You are an independent, read-only evidence checker. Work in the current folder (`qa/security-levels/milestones/`).
The records under check are `moderate-vm-parity3-20261021/` (the runs) and `moderate-amendment-v9-20261021/` (the
amendment). References: `moderate-utm-samebuild-20261021/` (utm-par2: packet v4c, harness, its decision 1(a) settling
check), `moderate-tart-parity-20261020/` (the frozen Tart parity method and its MOD-21 attempts), the amendment records
`moderate-amendment-v6-20261020/` and `moderate-amendment-v7-20261020/` (the process v9 follows), `vm-parity-method.md`
(D97 same-build rule) and `tester-packet-rules.md` (D114). Run nothing that writes; reading files, `zcat`/`gzip -dc`,
`tar -tf`/`tar -xOf` to stdout, `python3 -c` for parsing and `shasum` are fine.

Owner decisions (verbatim from the brief): D140 "first amend MOD-21 so the tester sends the task from the Spawn status
screen (the path #1175 changed) — use the amendment process used for v4–v8 (fresh code-blind designer, separate
reviewer, freeze the reviewer's text), recording it as owner-approved D140." D139 "run MOD-22 (with amendment v6) and
the amended MOD-21 THREE times on each machine — UTM 'macOS 3' and Tart — with the SAME build on both: the installed
signed build (INSTALL.json commit 9750861c2a). One VM at a time; fresh Tart clone per run, deleted after; UTM restored
to its starting state after. Code-blind testers GLM 5.3 Flash on zai. Apply the frozen comparison rule; recommend
whether to switch milestone runs to Tart." D141 "one MOD-11 re-run (about 20 min) on Tart with the same build so item 8
is reached and the case can close." D142 "only if your recommendation is to switch AND the owner later confirms, the
real Z.AI key file in 'macOS 3' is deleted — do NOT delete it yourself; just report its path (never read it)."

## What to check
1. **Amendment v9** (`moderate-amendment-v9-20261021/`): does the process match v6/v7 (fresh read-only designer with
   the listed packet, separate reviewer, the reviewer's final text frozen word for word as `amendments-v9-final.md`,
   no integrator edits; `design/FROZEN.sha256`)? Was the designer packet code-blind and free of earlier results (the
   two screens the README says were changed or dropped)? Does the README describe it truly?
2. **Dispositions** (`dispositions.md`, `results.json`): for each attempt (`runs/<label>/collected/evidence/result.md`,
   `screens.jsonl`, `actions.jsonl`), does the disposition follow (G-1; D39 only where the model declined before any
   product protection; D43; executor verdicts corrected where the evidence contradicts them)? Check the six MOD-21
   attempts under v9, the six MOD-22 attempts under v6 (`mod22-rows-check.jsonl`, `tools/check_rows.py`), and
   T09-tart-d141. Are the tester deviations listed truly, and is "none changed an outcome" supported?
3. **Comparison and recommendation** (`design/parity-method-d139.md`, frozen before the first VM): is "alike" right for
   both cases, are no exclusions needed, does the transport check pass (`transport.json`, `common/*/mediator-final.jsonl.gz`,
   `common/host/host-zai-relay.jsonl.gz`, `common/*/tunnel.log`), and does the recommendation follow the rule? Are the
   named limits honest (in particular: MOD-21's expected results 1-3 were never exercised)?
4. **#1175 claims:** are they supported by `runs/T17-*/task-delivery.txt` and the mediator logs (`runs/T17-*/mediator-window.jsonl`,
   the `task_messages` flags; `harness/agms_mediator.py`'s `task_flags`)? Is the comparison with the Tart parity run's
   T17 right?
5. **Method adherence and disclosure:** frozen inputs (`shasum -a 256 -c design/FROZEN.sha256` from the record root),
   that packet v4d differs from v4c only as `tools/make_packet_v4d.py` says, the same build and stage on both machines
   (`candidate/`, the testers' reported SHA-256), gates (`common/*/gate/`), guard counts, erase and restore checks
   (`common/*/erase-test-data.out`, `common/utm/restore-end.out`, `common/utm/vm-state-end*.txt` vs
   `common/utm/guest-state-start.txt`), "macOS 2" untouched and one VM at a time (`runs/seq.log`, `common/*/vm-start.txt`),
   and that no real key entered a guest (the UTM key file left in place, not read).
6. **Written claims:** every number, time and statement in both READMEs and `common/effective-policy.txt`, and whether
   the numbered decisions follow from the evidence and state the predeclared rule's result honestly.

List required corrections (wrong disposition, wrong class, unsupported or false claim) separately from optional ones,
each with the evidence path. End with exactly one line: `VERDICT: APPROVE` (no required correction) or
`VERDICT: CHANGES`.
