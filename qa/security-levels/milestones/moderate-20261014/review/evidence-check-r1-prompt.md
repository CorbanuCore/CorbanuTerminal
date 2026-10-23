You are the independent evidence checker for a code-blind functional test record. You are not the implementer,
designer or executor, and you have no prior context. Work read-only from the current directory (`record/`) only.
Do not read anything outside it, do not use the network, and do not look for or read product source code.

The record is the FIRST run of 34 cases (MOD-01..MOD-34) for the "Moderate" security level. `design.json` holds the
cases; `design/` holds the designer's proposal, the owner decisions (`owner-decisions-20261014.md`), the reviewed
amendments (`amendments-v2.md`, rules G-1..G-3 and named limits), the design review and `FROZEN.sha256`.
`executor-packet/v1` is the frozen packet; `executor-packet/v2` is what executors received (only the candidate SHA line
changed, `common/packet-v2-diff.txt`; replay prompts G18b/G21b added later, `common/replay-packet-diff.txt`,
`tools/replays.json`). `candidate/` is the design-time build, `candidate-v2/` the executed build (owner decision 1).
`results.json` has one disposition per case with every attempt; `dispositions.md` gives the integrator's reasoning
(generated from `tools/final.py`). `runs/<label>/` holds each executor run (G01-G25, replays G18b and G21b):
`collected/evidence/result.md` is the executor's own report; `screens.jsonl` every screen it read; `actions.jsonl`
every key/text; `fixture.jsonl` fixture records; `executor-events.jsonl` the transcript; `isolation-<case>.json`,
`probes.json`, `mediator-window.jsonl`. `common/` has the isolation policy (`effective-policy.txt`), provisioning and
restore output, gate, guard, VM state and the full relay log (`mediator-final.jsonl.gz`, each request labelled
`role` executor or product). `README.md` summarizes. Owner rule (#447): when the product's model declines a step, the
case passes if the product's text still meets the expected wording, and is blocked otherwise.

Check, and report concretely with file references:
1. No case vanished: every case in design.json has exactly one disposition in results.json, every attempt exists under
   runs/, and the design was frozen before execution (FROZEN.sha256 vs executor-packet/v1 and v2; README and
   common/vm-start.txt for the order).
2. For each case, whether the evidence (screens, actions, fixture records, not only the executor's prose) demonstrates
   the disposition against the governing expectation (amendments over frozen text). Passed cases need positive
   evidence for every expected result. Flag overstatement, understatement and silently skipped expected results. Judge
   in particular: the four cases where the integrator's disposition differs from the executor's (MOD-11, MOD-26,
   MOD-33 to failed; MOD-28 to blocked); MOD-10 (security_state.json and source-origin.key read after approval);
   MOD-01/06/08/32 attributed partly to #739; MOD-24 (the integrator's relay-window claim: check
   common/mediator-final.jsonl.gz yourself); the model-decline blocks (MOD-09, 15, 17, 21, 22, 29): is the rule
   applied consistently?
3. Whether the replays are honest: fresh executor, earlier attempts kept, notes neutral (navigation only, never
   expected outcomes), choice disclosed.
4. Whether isolation is honestly represented (probes per run, receipts bound to run/case/design/candidate, guard, VM
   handling, export redaction) and whether the named limits are complete.
5. Anything else a reviewer should know before a human signs off.

Your verdict is about the evidence, not about whether the product is good: "pass" means the record honestly supports
every disposition and claim as written; "fail" means at least one must be corrected. List each required correction
precisely. Be concise. End your answer with exactly one line: `VERDICT: pass` or `VERDICT: fail`.
