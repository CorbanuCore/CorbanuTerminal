You are the independent evidence checker for a code-blind functional test record. You are not the implementer,
designer or executor, and you have no prior context. Work read-only from the current directory only. Do not read
anything outside it, do not use the network, and do not look for or read product source code.

The current directory holds two folders:
- `aggressive-20261015-targeted3/`: the record under check, a THIRD TARGETED re-run of milestone "Aggressive ships".
  4 of 64 cases executed on this candidate (AGG-T01 in two groups, part A in W01 and part B in W02; AGG-T02 in W01;
  AGG-R03 in W03; AGG-R14 in W04 and W04-a2), 2 re-judged without re-running on the previous record's screens by owner
  decision (AGG-04 on run V01, AGG-37 on run V02), and 58 carried over from the previous record (`carried_from`; take
  those as stated).
- `aggressive-20261014-targeted2/`: an excerpt of the previous record (its README, dispositions, results, amendment 5,
  executor packet v6 and runs V01 and V02 only), so that AGG-04 and AGG-37 can be checked on the screens they rest on.

In the record: `design.json` holds all 64 cases with their earlier amendments and this run's `amendment_20261015` /
`disposition_20261015`. `design/` holds the owner decisions (`decisions/`; verbatim excerpts in
`review-packet/decisions.md`), the amendment-6 drafts and final table, the keystroke-driver designer's packet, prompt,
output and sessions (its first attempt died with no reply), the independent amendment/driver review (`review-*`), the
regex lists, `harness-smoke.txt` and `FROZEN.sha256`. `executor-packet/v7/` is what executors received (cases,
amendments part 7, `drivers/`, screens, prompts W01-W04). `runs/<group>/` hold each executor run:
`collected/evidence/result.md` or `collected/work/evidence/result.md` (the executor's report), `screens.jsonl` (every
screen read, including the driver's; a `"term": 2` field marks AGG-R03's second terminal), `actions.jsonl`,
`driver-<case>.log` (W03, W04, W04-a2), `fixture.jsonl`, `executor-events.jsonl` (transcript), `isolation-<case>.json`,
`probes.json`, `mediator-window.jsonl`; `runs/seq.log` is the host's run log. `common/` has the isolation policy,
provisioning/restore output, gate (including a first gate attempt that failed), guard, VM state and mediator log;
`harness/` includes `agms-tui` and `agms-drive`. `tools/final6.json`, `dispositions.md` and `README.md` give the
integrator's reasoning and summary.

Check, and report concretely with file references:
1. No case vanished: every case in design.json has exactly one disposition in results.json, and the executed cases
   have their runs. The amendment table, drivers, harness drivers and packet were frozen before execution
   (design/FROZEN.sha256 vs executor-packet/v7, harness/agms-tui and harness/agms-drive; README and
   common/vm-start.txt for the order); the final table and the AGG-R03 driver match the reviewer's final versions in
   design/review-output.md (the reviewer's "as quoted for AGG-04" stands for the full quote); AGG-R14's driver is the
   previous record's, byte for byte.
2. For each executed or re-judged case, whether the evidence demonstrates the disposition against the governing
   expectation (part 7 over part 6 over part 5 over 2026-10-13 over 2026-10-11 over frozen). Passed cases need positive
   evidence for every expected result; blocked cases need evidence of why. Flag any overstatement, understatement or
   silently skipped expected result. In particular judge: AGG-04's amended results 3 and 4 on V01's inspector and screen
   07 (and the nested-agents row); AGG-37's amended result 1 on V02's screen 07; AGG-T01 part A, where the executor
   reported "failed" on an advisory result the integrator says amendment 6 removed from part A; AGG-T01 part B and its
   setup flags; AGG-T02's "Permissive active" without a /security screen after the relaunch; AGG-R03 (did the driver do
   what the frozen file says, on the right terminal; is "stopped by the kill switch" shown); AGG-R14 (was a second
   attempt legitimate and honestly recorded; are result 1 passed and result 2 blocked right; was anything continued by
   hand).
3. Whether the owner-accepted cases (AGG-18, AGG-R07, AGG-R10, AGG-20) and AGG-R06 are carried faithfully.
4. Whether isolation is honestly represented: probe results per run, receipts bound to the right run/case/design/
   candidate, guard, VM handling (the refused first start, the gate's stale expected digest, erase, two verification
   boots, final power-off), and whether the stated limits are complete.
5. Whether the README's combined 64-case status matches results.json and dispositions.md, and anything else a reviewer
   should know before a human signs off.

Your verdict is about the evidence, not about whether the product is good: "pass" means the record honestly supports
every disposition and claim as written; "fail" means at least one must be corrected. List each required correction
precisely. Be concise. End your answer with exactly one line: `VERDICT: pass` or `VERDICT: fail`.
