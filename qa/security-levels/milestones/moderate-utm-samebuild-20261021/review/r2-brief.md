# Evidence check, round 2 of 5 — UTM same-build check (D93) and D94/D95 Tart re-run (utm-par2)

You are an independent, read-only evidence checker. Work in the current folder (`qa/security-levels/milestones/`).
The record under check is `moderate-utm-samebuild-20261021/`. Its reference is the Tart parity record
`moderate-tart-parity-20261020/` (signed-off case texts: `moderate-tart-parity-20261020/executor-packet/v4p/`; amendments
v6 and v7: `moderate-amendment-v6-20261020/`, `moderate-amendment-v7-20261020/`; same-build rule: `vm-parity-method.md`).
Run nothing that writes; reading files, `zcat`/`gzip -dc`, `python3 -c` for parsing and `shasum` are fine.

Owner decisions: D93 "re-run MOD-11, MOD-21, MOD-22, MOD-26, MOD-29 once on UTM 'macOS 3' with the same build Tart used
(8c73c8ea80) and the same case texts as the Tart run, code-blind; compare with Tart's results and recommend whether to
switch milestone runs to Tart (switch if they behave as on Tart)". D94/D95: re-run MOD-11 and MOD-21 on Tart with the
current installed build and the current case texts (amendments v6+v7) to confirm #1171 (#1139) and #1175 (#1146).

## What to check
1. **Dispositions** (`dispositions.md`, `results.json`): for each attempt (`runs/<label>/collected/evidence/result.md`
   or `.../work/evidence/result.md`, `screens.jsonl`, `actions.jsonl`, `pty.raw.gz`), does the disposition follow under
   the Tart method's rules (`moderate-tart-parity-20261020/design/parity-method.md`, "Dispositions on Tart": G-1 model
   declines, owner decisions D39/D40/D41/D43 only where the accepted limit repeats, executor verdicts corrected where the
   evidence contradicts them)? Look hard at MOD-11 part 2 (executor failed -> blocked), MOD-21 part 1 (blocked, D39 not
   applied) and MOD-22 part 1 (passed).
2. **Comparison classes** (`design/parity-method-utm.md`, frozen before the VM started): against the Tart attempts of
   the same build (`moderate-tart-parity-20261020/runs/T09*, T17*, T18, T21*` and its `dispositions.md`), is each
   "matched" true for every expected result, is MOD-29 "within Tart's range" right, and do MOD-21 and MOD-22 follow the
   predeclared replay rule (cap 2, blockers first)? Check the transport check (`transport.json`, `common/*/
   mediator-final.jsonl.gz`, `common/host/host-zai-relay.jsonl.gz`, `common/*/tunnel.log`).
3. **D94/D95 claims:** is #1139's expected result really met in `runs/T09-cur` (screens naming the level for items 1-2,
   #1171's warning text), and is "#1175's route not exercised" right for `runs/T17-cur` (see
   `runs/T17-*/task-delivery-excerpt.txt`)? Is the claim about how the task reached the child in `runs/T17-utm` supported?
4. **Method adherence and disclosure:** frozen inputs (`shasum -a 256 -c design/FROZEN.sha256` and
   `design/FROZEN-d94d95.sha256` from the record root), that packet v4u differs from v4p only by the 30-second line and
   that `candidate/package-files.txt` equals the Tart run's, the gates (`common/*/gate/`), guard counts, erase and restore
   checks (`common/*/erase-test-data.out`, `common/utm/restore-end.out`, `vm-state-end*.txt`), "macOS 2" untouched and one
   VM at a time (`runs/seq.log`, `common/*/vm-start*.txt`), and that no real key entered a guest.
5. **Written claims:** every number, time and statement in `README.md` and `common/effective-policy.txt`, and whether
   the recommendation and the numbered decisions follow from the evidence and state the predeclared rule's result
   honestly.

List required corrections (wrong disposition, wrong class, unsupported or false claim) separately from optional ones,
each with the evidence path. End with exactly one line: `VERDICT: APPROVE` (no required correction) or
`VERDICT: CHANGES`.

## Round 2 focus
Round 1 (`review/r1-output.md`, head `f470787768`) asked for 8 required corrections; the integrator applied them and the
optional ones (see `review-ledger.md`, `git diff f470787768 HEAD -- .` from the record root shows every change). Check
that each required correction is resolved correctly and introduced no new error; in particular the MOD-21 session-file
claims (`runs/T17-*/task-delivery-excerpt.txt`, `tart-reference/*.txt`, `tools/excerpt_tasks.py`; the file SHA-256 in each
excerpt should appear in the run's `withheld-files.txt`, for Tart in `../moderate-tart-parity-20261020/runs/T17*/`), the
recommendation and decision 1 against `design/parity-method-utm.md`, and `harness-tart/seal-assets.sha256`. Then re-check
anything else in the record you find wrong.
