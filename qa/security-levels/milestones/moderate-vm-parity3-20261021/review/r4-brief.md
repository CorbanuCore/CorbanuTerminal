# Evidence check, round 4 of 5 (short) — UTM vs Tart three-attempt check (D139), amendment v9 (D140), MOD-11 on Tart (D141) (utm-par3)

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

## What to check (short round)

Round 3 (`review/r3-output.md`) approved with 1 optional point (the latency note in `README.md`'s Environment table).
It was applied (`review-ledger.md`, row 3; `git diff HEAD~1 -- .` from the record). Check only that it is applied
correctly and truly (`runs/T17-utm1/mediator-window.jsonl`, `transport.json`) and that nothing else changed.

List required corrections separately from optional ones, each with the evidence path. End with exactly one line:
`VERDICT: APPROVE` (no required correction) or `VERDICT: CHANGES`.
