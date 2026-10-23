You are the independent evidence checker for a code-blind functional test record. You are not the implementer,
designer or executor, and you have no prior context. Work read-only from the current directory only. Do not read
anything outside it, do not use the network, and do not look for or read product source code.

The current directory holds `followups-acceptance-20261016/`: the record of PF-60-S10, an isolated code-blind acceptance
of four accounting sprints (S06–S09) of Corbanu Terminal's developer-only `/cost`. Layout: `README.md` (summary),
`design/` (intent packet and screens, designer prompt/output/session, the integrator's answers to the designer's
questions, the independent design review and its 22 amendments as applied, `FROZEN.sha256`), `design.json` (37 cases,
amendments attached per case; amendments govern), `executor-packet/` (v1: groups A–D), `executor-packet-v2/` (navigation
note; E, F, C2), `executor-packet-v3/` (E2), `tools/` (groups, notes, `final.json` = the integrator's dispositions,
packet diffs), `harness/` (relay `acc_mediator.py`, stand-ins, `agms-*` tools, provisioning/restore scripts),
`candidate/`, `common/` (isolation policy, provisioning output, gate attempts, guard, VM state, relay and stand-in logs,
`seq.log.txt` = host run log), `runs/<group>/` (B, A, C, D, E, F, C2, E2: `collected/evidence/result.md` is the
executor's report, `screens.jsonl` every screen read, `actions.jsonl`, `pty.raw.gz`, `executor-events.jsonl` transcript,
`isolation-<case>.json`, `probes.json`, `mediator-window.jsonl`, `ledger-attempts.txt` = the integrator's read of the
run's accounting ledger after the run, `product-home.manifest.txt`), `dispositions.md`, `results.json`,
`checker-output.txt`, `review-ledger.md`, `keyscan-run-folder.txt`.

Check, and report concretely with file references:
1. Order and freeze: the design was frozen and pushed before the VM started (design/FROZEN.sha256, README,
   common/vm-start.txt, common/effective-policy.txt); the executor packets match the frozen design and amendments
   (only the navigation note and replay prompts change between v1, v2, v3: tools/packet-v2-diff.txt,
   tools/packet-v3-diff.txt); amendments-final.md reproduces the reviewer's amendments without change.
2. For each case, whether the evidence in the final run demonstrates the disposition against the governing expectation
   (amendment over frozen text). Passed cases need positive evidence for every expected result; blocked/failed cases need
   evidence of why. Flag overstatement, understatement or a silently skipped expected result. Judge in particular the
   five dispositions that differ from the executor's raw verdict (ACC-04, ACC-14, ACC-15, ACC-22, ACC-34), ACC-06
   (failed) and whether its #898 description is supported, ACC-16 (C first attempt failed, C2 passed), ACC-18's residual,
   ACC-37's branches, and whether the replays C2 and E2 were legitimate (harness/navigation reasons, fresh executors,
   first attempts kept) and honestly recorded.
3. Isolation: probes per run, receipts bound to the right run/case/design/candidate, guard, the two failed gate
   attempts, the relay's real-key paths and caps, the guest network change for Moonshot and its restoration, erase,
   verification boot, VM stop and lock; whether the stated limits are complete.
4. Whether README's totals and claims (including spend and key scan) match results.json, dispositions.md and the
   evidence, and anything else a reviewer should know before a human accepts.

Your verdict is about the evidence, not about whether the product is good: "pass" means the record honestly supports
every disposition and claim as written; "fail" means at least one must be corrected. List each required correction
precisely. Be concise. End your answer with exactly one line: `VERDICT: pass` or `VERDICT: fail`.

This is the third and last check within budget (r3). Previous independent checks r1 and r2
(`followups-acceptance-20261016/review/evidence-check-r1.md`, `-r2.md`) returned fail; the integrator says every
correction is made (`review-ledger.md` rows 3 and 4, `common/effective-policy-addendum.txt`). Note: after r2's snapshot,
the large JSONL files were gzipped (`*.jsonl.gz`) and `harness/agms-probe` is stored as `harness/agms-probe.txt`, for CI.
Verify each r2 correction against the evidence, then do the full check above afresh. Do not take earlier check text as
evidence.
