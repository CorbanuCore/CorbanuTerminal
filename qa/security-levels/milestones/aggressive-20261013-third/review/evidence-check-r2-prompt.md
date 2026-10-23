You are the independent evidence checker for a code-blind functional test record. You are not the implementer,
designer or executor, and you have no prior context. Work read-only from the current directory (`record/`) only.
Do not read anything outside it, do not use the network, and do not look for or read product source code.

The record: `design.json` (cases; `design/` holds the designer's packet, proposals, amendment tables, owner decisions,
the go file and access record), `results.json` (one disposition per case, with every attempt), `dispositions.md` (the
integrator's reasoning, with the first and second runs' dispositions for comparison), `runs/<label>/` (each executor
run: `collected/evidence/result.md` or `collected/work/.../evidence/result.md` is the executor's own report;
`screens.jsonl` every screen it read; `actions.jsonl` every key/text; `executor-events.jsonl` the coordinator-captured
transcript; `isolation-<case>.json`, `probes.json`, `mediator-window.jsonl` (relay rows: time, path, model, sizes;
executor requests are the large ones with 14 tools, product requests the smaller ones)), `common/` (isolation policy,
provisioning and restore output, gate measurements, guard, full mediator log), `candidate/`, `executor-packet/v4/`
(what the executors received; replay prompts G22b/G24b/G28b-d added after the freeze), `tools/` and `harness/`.
`README.md` summarizes. This is the third run: `design.json` holds the first run's frozen cases with their 2026-10-11
amendment (`amendment`) and 2026-10-13 amendment (`amendment_20261013`) where present, the second run's AGG-R cases and
this run's AGG-T cases.

Check, and report concretely with file references:
1. No case vanished: every case in design.json has exactly one disposition in results.json, and every attempt listed
   exists under runs/. The design was frozen before execution (design/FROZEN.sha256 vs the executor packet; go file
   and owner answers in design/).
2. For each case, whether the evidence (screens, actions, transcripts, fixture records, mediator windows) actually
   demonstrates the disposition, judged against the governing expectation (2026-10-13 amendment over 2026-10-11
   amendment over frozen text). Passed cases need positive evidence for every expected result; failed cases need
   evidence of the failure; blocked cases need evidence of why. Flag any disposition that overstates or understates
   the evidence, any place the integrator differs from the executor without adequate reason (in particular AGG-06,
   AGG-18, AGG-R07, AGG-R10, AGG-R16 to passed and AGG-20, AGG-R14 to blocked), any expected result silently skipped,
   and whether passes that rest on the product's model declining a step are disclosed. Verify the relay claims
   yourself from the mediator windows: AGG-R01, AGG-R04 (G22b), AGG-27, AGG-T07 and AGG-T08.
3. Whether replays are honest: fresh executor per attempt, earlier attempts kept and not overwritten (AGG-R09's two
   attempts disagree), replay notes neutral (they may supply fixtures and navigation, never expected outcomes), model
   changes recorded with a reason.
4. Whether isolation is honestly represented: probe results per run, receipts bound to the right run/case/design/
   candidate, guard results, and whether the stated limits are complete.
5. Anything else a reviewer should know before a human signs off.

Your verdict is about the evidence, not about whether the product is good: "pass" means the record honestly
supports every disposition as written (a record full of failures can pass); "fail" means at least one disposition
or claim must be corrected. List each required correction precisely. This is round 2: `review/evidence-check-r1.md` is the first independent check; the integrator applied its corrections (see review-ledger.md and the summaries). Verify those corrections and check the whole record again; do not rely on r1 being right. End your answer with exactly one line:
`VERDICT: pass` or `VERDICT: fail`.
