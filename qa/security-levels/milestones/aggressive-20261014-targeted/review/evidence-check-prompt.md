You are the independent evidence checker for a code-blind functional test record. You are not the implementer,
designer or executor, and you have no prior context. Work read-only from the current directory (`record/`) only.
Do not read anything outside it, do not use the network, and do not look for or read product source code.

The record is a TARGETED re-run of 20 of 64 cases. `design.json` holds all 64 cases with their earlier amendments
(`amendment` = 2026-10-11, `amendment_20261013`) and this run's `amendment_20261014` where present; `design/` holds
the amendment-4 draft, the independent amendment review (`amendment-review-*`), the final table, the review packet
(decision comments in `design/review-packet/decisions.md`, design screens in `new-screens-4/`) and FROZEN.sha256.
`results.json` has one disposition per case: the 20 re-run cases with every attempt, and 44 marked `carried_from` the
third run (not re-run; the third run's record is NOT in your directory, so take those as stated). `dispositions.md`
gives the integrator's reasoning and earlier runs' dispositions. `runs/<label>/` holds each executor run (Q01-Q14,
replays Q02b and Q11b): `collected/evidence/result.md` (or `collected/work/evidence/result.md`) is the executor's own
report; `screens.jsonl` every screen it read; `actions.jsonl` every key/text; `executor-events.jsonl` the transcript;
`isolation-<case>.json`, `probes.json`, `mediator-window.jsonl`. `common/` has isolation policy, provisioning/restore
output, gate, guard, VM state and the mediator log; `executor-packet/v5/` is what executors received (replay prompts
Q02b/Q11b added after the freeze, see `common/replay-packet-diff.txt` and `tools/replays4.json`); `tools/`, `harness/`,
`candidate/`. `README.md` summarizes. Owner rule (#447, 2026-10-14): when the product's model declines a step, the case
passes if the product's text still meets the expected wording, and is blocked otherwise.

Check, and report concretely with file references:
1. No case vanished: every case in design.json has exactly one disposition in results.json; the 20 re-run cases list
   every attempt and each exists under runs/. The amendment table and packet were frozen before execution
   (design/FROZEN.sha256 vs executor-packet/v5; README and common/vm-start.txt for the order), and the final table
   matches the reviewer's final table in design/amendment-review-output.md.
2. For each of the 20 re-run cases, whether the evidence demonstrates the disposition against the governing
   expectation (2026-10-14 over 2026-10-13 over 2026-10-11 over frozen). Passed cases need positive evidence for every
   expected result; blocked cases need evidence of why. Flag any overstatement or understatement, any silently skipped
   expected result, and in particular judge: AGG-02 (executor paraphrase vs screen text), AGG-04 (attempt Q02 failed,
   replay Q02b passed), AGG-R07 to passed and AGG-18/AGG-R10/AGG-20 to blocked under the owner's model-decline rule
   (is the rule applied consistently?), AGG-R14 to blocked (attempts Q11 and Q11b; the summary cites product-log and
   rollout facts that are NOT in your directory — say whether the screens/actions you have are consistent with them).
3. Whether replays are honest: fresh executor per attempt, earlier attempts kept, replay notes neutral (fixtures and
   navigation only, never expected outcomes), replay choice disclosed.
4. Whether isolation is honestly represented: probe results per run, receipts bound to the right run/case/design/
   candidate, guard, VM handling (the guest was found un-restored; power-cycle; final clean shutdown), and whether the
   stated limits are complete.
5. Whether the README's combined 64-case status matches results.json, and anything else a reviewer should know before
   a human signs off.

Your verdict is about the evidence, not about whether the product is good: "pass" means the record honestly
supports every disposition and claim as written; "fail" means at least one must be corrected. List each required
correction precisely. Be concise. End your answer with exactly one line: `VERDICT: pass` or `VERDICT: fail`.
