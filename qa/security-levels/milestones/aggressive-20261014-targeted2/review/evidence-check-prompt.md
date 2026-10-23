You are the independent evidence checker for a code-blind functional test record. You are not the implementer,
designer or executor, and you have no prior context. Work read-only from the current directory (`record/`) only.
Do not read anything outside it, do not use the network, and do not look for or read product source code.

The record is a SECOND TARGETED re-run: 4 of 64 cases executed (AGG-04, AGG-37, AGG-R03, AGG-R14), 4 accepted by the
owner without execution (AGG-18, AGG-R07, AGG-R10, AGG-20; `out_of_scope` with a recorded approval), and 56 carried over
from the first targeted run's record (`carried_from`; that record is NOT in your directory, so take those as stated).
`design.json` holds all 64 cases with their earlier amendments and this run's `amendment_20261014b` /
`disposition_20261014b`. `design/` holds the owner decisions (`decisions/`, verbatim excerpts in
`review-packet/decisions.md`), the amendment-5 draft and final table, the keystroke-driver designer's packet, prompt,
output and session, the independent amendment/driver review (`amendment-review-*`), `driver-halt-check.txt` and
`FROZEN.sha256`. `executor-packet/v6/` is what executors received (cases, amendments part 6, `drivers/`, screens,
prompts V01-V04). `runs/V01..V04/` hold each executor run: `collected/evidence/result.md` (the executor's report),
`screens.jsonl` (every screen read, including the driver's), `actions.jsonl` (every key/text), `driver-<case>.log`
(V03, V04), `executor-events.jsonl` (transcript), `isolation-<case>.json`, `probes.json`, `mediator-window.jsonl`.
`common/` has the isolation policy, provisioning/restore output, gate, guard, VM state and mediator log;
`host-checks/` are integrator host checks (not code-blind); `harness/` includes `agms-drive`. `dispositions.md`
and `README.md` give the integrator's reasoning and summary. Owner rule (#447, 2026-10-14): when the product's model
declines a step, the case passes if the product's text still meets the expected wording, and is blocked otherwise.

Check, and report concretely with file references:
1. No case vanished: every case in design.json has exactly one disposition in results.json, and the four executed
   cases have their runs. The amendment table, drivers and packet were frozen before execution (design/FROZEN.sha256
   vs executor-packet/v6 and harness/agms-drive; README and common/vm-start.txt for the order); the final table and the
   AGG-R03 driver match the reviewer's final versions in design/amendment-review-output.md, and AGG-R14's driver is the
   designer's.
2. For each executed case, whether the evidence demonstrates the disposition against the governing expectation
   (part 6 over part 5 over 2026-10-13 over 2026-10-11 over frozen). Passed cases need positive evidence for every
   expected result; blocked cases need evidence of why. Flag any overstatement or understatement or silently skipped
   expected result. In particular judge: AGG-04 expected result 3 given the inspector's Sandbox/Network rows and
   screen 07 (see #816 and host-checks/); AGG-37's vault refusal being the product model's own text; AGG-R03's and
   AGG-R14's stops (did the drivers do what the frozen files say; is the integrator's correction of the AGG-R14
   approval time right; was anything continued by hand).
3. Whether the four owner-accepted cases are recorded faithfully to the decision (accepted, not passed; AGG-20 also a
   named limit), and AGG-T01/T02 (#739) and AGG-R06 left as stated.
4. Whether isolation is honestly represented: probe results per run, receipts bound to the right run/case/design/
   candidate, guard, VM handling (power-cycle at start; erase; verification boot; final shutdown), and whether the
   stated limits are complete.
5. Whether the README's combined 64-case status matches results.json and dispositions.md, and anything else a reviewer
   should know before a human signs off.

Your verdict is about the evidence, not about whether the product is good: "pass" means the record honestly
supports every disposition and claim as written; "fail" means at least one must be corrected. List each required
correction precisely. Be concise. End your answer with exactly one line: `VERDICT: pass` or `VERDICT: fail`.
