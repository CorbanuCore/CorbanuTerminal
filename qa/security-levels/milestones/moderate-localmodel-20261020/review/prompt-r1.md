You are the independent evidence checker for a code-blind functional test record. You are not the implementer,
designer or executor, and you have no prior context. Work read-only from the current directory only. Do not read
anything outside it, do not use the network, and do not look for or read product source code.

The current directory holds:
- `moderate-localmodel-20261020/`: the record under check. A follow-up re-run of two blocker cases of milestone
  "Moderate ships" (already signed off), MOD-09 (group R01) and MOD-21 (group R02), with the owner's local model as the
  product's model, under amendment v5, in fresh Tart clones. In earlier runs both cases were blocked because the hosted
  product model declined the steps; the owner then accepted them as known limits (D39).
- `excerpts/`: the signed-off record's README, dispositions and results (`moderate-targeted2-20261020/`), the first
  run's dispositions and its MOD-13 executor report (`moderate-20261014-run1/`, for precedent), the signed-off
  Aggressive record and an agent-cost-accounting record (README/dispositions/results only, for the "blocked the same
  way" list), the product's user docs on Moderate and on protected launch (`docs/`), and the text of issue #1077 filed
  from this run.

In the record under check: `design/` holds the designer packet (owner decisions in `designer-packet/decisions.md`,
`scope.md`, `tools.md`, the frozen cases, amendments v2-v4, screens incl. `screens-v5/` of this candidate on the local
model), the designer's prompt and output, `integrator-edits.diff`, `integrator-notes.txt`, the reviewer's prompt and
output (`review-output.md`, VERDICT: CHANGES), `review-packet-tools.md`, `amendments-v5-final.md`, `FROZEN.sha256`.
`executor-packet/v5/` is what the testers received. `runs/R01` and `runs/R02`: `collected/evidence/result.md` (the
tester's report), `screens.jsonl`, `actions.jsonl`, `fixture.jsonl`, `pty.raw.gz`, the tester's screen files,
`collected/fixture/` (the test project after the run), `executor-events.jsonl.gz` (tester transcript),
`mediator-at-end.jsonl.gz`, preflight, guard, wipe and memory files; `runs/R01/product-log-excerpt.txt`. `common/R01`,
`common/R02`: VM start, seal, provisioning, policy, gate (`gate/`), tunnel, guard, VM end state and test-data erase,
and the full guest mediator (relay) log; `common/host/`: the host-side local-model gate and Z.AI relay logs.
`harness/` the guest and host scripts (`post-freeze-host-driver.diff` a host-only change after the freeze);
`shakedown/` the pre-freeze check; `candidate/`, `tools/`. `dispositions.md`, `results.json` and `README.md` give the
integrator's reasoning and summary.

Check, and report concretely with file references:
1. Order and freeze: amendment v5, the packet and the harness were frozen (design/FROZEN.sha256 vs executor-packet/v5
   and harness/) before the first clone (README, common/R01/vm-start.txt); amendments-v5-final.md is the reviewer's
   final version and the packet's amendments-v5.md is that file; the prompts R01/R02 assign the right cases.
2. MOD-09: does the evidence support "failed" against the governing expectation (amendment v5 over v4 over v3 over
   v2 over the frozen text, with its rules and "How each outcome is recorded")? Check every expected result and the
   integrator's calls where they differ from the tester: item 7 (the approval and the ⚠ after it, the prompt's
   Reason), item 1 (refused without a prompt), items 2-4 and 8 (prompts, Esc), item 5 (the image tool; product log
   excerpt), item 6 (the test account has no ~/Library/Keychains; "not exercised"), result 8 ("names the security
   level": prompts beginning "Security level:"; compare run 1's MOD-13 precedent), the R-2 check, `creds-copy.txt`,
   and that no synthetic value appeared in the product's output. Also the combined request before item 6.
3. MOD-21: does the evidence support "passed"? Positive evidence is needed for every expected result: the model on
   both screens, the child's attribution (`Thread: Agent (...)`) and its three attempts, the outside write's failure
   after approval, the credential prompt's security reason, the config read denied after approval, outside-notes.txt
   unchanged, and the inspector's Agents row while the child was open.
4. Isolation and handling: probes per clone, the claim that no real key entered the guest (setup-agent, mediator,
   relay design), the tunnel bound to loopback and closed to the testers' accounts, guard, test-data erase before each
   clone was deleted, one VM at a time, no UTM VM, memory, and whether the named limits are complete and honest.
5. The README's claims: the combined status (31/1/2) against the signed-off record, "doesn't change the sign-off",
   issue #1077 as described (excerpts/issue-1077.md) and whether the record's statements about the docs match
   excerpts/docs, the list of Aggressive/accounting cases blocked the same way against the excerpts, and the review
   ledger.

Your verdict is about the evidence, not about whether the product is good: "pass" means the record honestly supports
every disposition and claim as written; "fail" means at least one must be corrected. List each required correction
precisely. Be concise. End your answer with exactly one line: `VERDICT: pass` or `VERDICT: fail`.
