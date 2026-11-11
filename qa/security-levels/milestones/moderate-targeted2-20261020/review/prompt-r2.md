You are the independent evidence checker for a code-blind functional test record. You are not the implementer,
designer or executor, and you have no prior context. Work read-only from the current directory only. Do not read
anything outside it, do not use the network, and do not look for or read product source code.

The current directory holds three folders:
- `moderate-targeted2-20261020/`: the record under check, a SECOND TARGETED re-run of milestone "Moderate ships":
  MOD-15 (group Q01) and MOD-28 (group Q02) executed on this candidate under amendment v4, plus the owner's
  decisions of 2026-10-19 applied without a re-run (D39: MOD-09/21 known limits; D41: MOD-33 counts passed; D43:
  MOD-11 and MOD-06's git part accepted limits; D40: MOD-28's "no session approval exists to revoke" outcome; D42:
  sign-off after a passing MOD-15). All other cases are carried over from the previous record.
- `moderate-targeted-20261016/`: the previous targeted re-run's full record (17 cases; its combined 34-case column),
  for the carried cases and the amendment v3 in force. Only its README gained a one-paragraph pointer to the new record.
- `moderate-20261014/`: an excerpt of the first run's record (README, dispositions, results, design.json).

In the record under check: `design.json` (the 34 frozen cases and references); `design/` holds the designer packet
(owner decisions in `designer-packet/decisions.md`, `scope.md`, `tools.md`, the frozen cases, amendments v2 and v3,
host captures of this candidate `screens-v4/` and the previous candidate's `screens-v3/`), the designer's prompt and
output, the integrator's mechanical edits (`integrator-edits.diff`), the independent reviewer's prompt and output
(`review-output.md`, VERDICT: CHANGES), `amendments-v4-final.md`, `FROZEN.sha256`, `sessions.txt` and
`host-capture/` (the capture scripts and raw screens). `executor-packet/v4/` is what executors received. `runs/Q01`
and `runs/Q02` hold each executor run: `collected/evidence/result.md` (the executor's report), `screens.jsonl` (every
screen read), `actions.jsonl`, `fixture.jsonl`, `pty.raw.gz`, the executor's own screen files, 
`collected/product-logs/codex-tui.log` (the product's own log), `executor-events.jsonl` (transcript),
`isolation-<case>.json`, `probes.json`, `mediator-window.jsonl`; `runs/seq.log` is the host's run log. `common/` has
the isolation policy, provisioning/restore output, guard, VM state and the full mediator (relay) log; `gate/` the
isolation gate. `tools/final-t2.json`, `tools/owner-dispositions.json`, `dispositions.md` and `README.md` give the
integrator's reasoning and summary; `candidate/` the candidate.

Check, and report concretely with file references:
1. No case vanished: every case in design.json has exactly one disposition in results.json; the executed cases have
   their runs. Amendment v4, the packet and the harness were frozen before execution (design/FROZEN.sha256 vs
   executor-packet/v4 and harness/; README, common/vm-start.txt and runs/seq.log for the order); amendments-v4-final.md
   is the reviewer's final version (design/review-output.md) and the packet's amendments-v4.md is that file.
2. MOD-15 and MOD-28: does the evidence demonstrate each disposition against the governing expectation (amendment v4
   over v3 over v2 over the frozen text, with rules G-1..G-3 and R-1..R-3 as v4 changes them)? Passed needs positive
   evidence for every expected result. Flag any overstatement, understatement or silently skipped expected result.
   In particular: MOD-15's refusal text in the cells (pty and screens), "no prompt", item 4, Recent denials, the
   not-"Approval declined" check, and the product log; MOD-28's P1-P3 search, that no session option was offered, the
   revoke review wording vs screens-v4 06, `g` and the level after the revoke, that action 2 was correctly not run,
   and whether "accepted under D40" (not passed) is the right label.
3. Whether the owner dispositions (D39, D41, D43) are applied faithfully to MOD-09, MOD-21, MOD-11, MOD-33 and MOD-06,
   and the carried cases faithfully carried from moderate-targeted-20261016/results.json.
4. Whether isolation is honestly represented: probe results per run, receipts bound to the right run/case/design/
   candidate, guard, VM handling (erase, live check, clean shutdown, verification boot, power-off, the lock), and
   whether the stated limits are complete.
5. Whether the README's 34-case totals (30 passed, 4 accepted by owner decision) match results.json and
   dispositions.md; whether the README's "ready to be brought for sign-off (D42)" follows; the two new advisory issues
   (#1035, #1036) as described; and anything else a reviewer should know before a human signs off.

A previous independent check (`moderate-targeted2-20261020/review/evidence-check-r1.md`) failed the record on three
corrections and two recommendations; the integrator says they are applied (README named limits, `tools/final-t2.json`,
`common/effective-policy-addendum.txt`). Verify those, but check the whole record afresh; do not rely on the earlier
check's findings.

Your verdict is about the evidence, not about whether the product is good: "pass" means the record honestly supports
every disposition and claim as written; "fail" means at least one must be corrected. List each required correction
precisely. Be concise. End your answer with exactly one line: `VERDICT: pass` or `VERDICT: fail`.
