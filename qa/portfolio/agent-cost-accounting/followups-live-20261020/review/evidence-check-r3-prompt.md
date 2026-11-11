This is the third and last independent evidence check of this record within its review budget. Two earlier checks
(both fail, on statements) are in `followups-live-20261020/review/evidence-check-r1.md` and `evidence-check-r2.md`;
the integrator says every correction they list is applied (`review-ledger.md` rows 3-4, `tools/differences.md`,
`common/effective-policy-addendum.txt`, `common/host-end-check.txt`). Verify each r1 and r2 correction against the
evidence, then do the full check below independently; do not rely on earlier confirmations.

You are the independent evidence checker for a code-blind functional test record. You are not the implementer,
designer or executor, and you have no prior context. Work read-only from the current directory only. Do not read
anything outside it, do not use the network, and do not look for or read product source code.

The current directory holds `followups-live-20261020/`, the record of acc-tart1. This is an isolated code-blind live run
of Corbanu Terminal's developer-only `/cost` accounting, run on one fresh Tart virtual machine. It re-checks the gaps of
an earlier acceptance (PF-60-S10, owner decision D25), a real Moonshot cache read (D3/D54), one real OpenAI realtime
session under $0.05 (D16) and the move to Tart (D34).

Layout:
- `README.md`: the summary.
- `design/`: the intent packet and screens, the designer prompt, output and session, the integrator's answers to the
  designer's questions, the owner decisions, the independent design review and its 25 amendments as applied, and
  `FROZEN.sha256`.
- `design.json`: 25 cases with their amendments attached; amendments govern.
- `executor-packet/`: packet v1, used by every group.
- `tools/`: groups, notes, `final.json` (the integrator's dispositions), `differences.md` and the scripts that built
  results.json and dispositions.md.
- `harness/`: the relay `acc_mediator.py`, the `agms-*` tools, `tart-run.sh` (host driver), provisioning, and the
  realtime driver.
- `candidate/`.
- `common/`: the isolation policy, provisioning output, gate, guard, VM state at start and end, key erase, relay logs,
  and `seq.log.txt` (host run log).
- `shakedown/`: a harness shakedown on a separate clone before the freeze.
- `runs/<group>/` for Z, O, R and M:
  - `collected/evidence/result.md` is the executor's report;
  - `screens.jsonl` holds every screen read; also `actions.jsonl`, `pty.raw.gz` and the `executor-events.jsonl`
    transcript (any file over 400 KB is gzipped);
  - the executor's own screen-file folder is packed as a `.tar.gz`;
  - `isolation-<case>.json`, `probes.json`, `mediator-window.jsonl`;
  - `ledger-attempts.txt`, the integrator's read of the run's accounting ledger after the run;
  - `product-home.manifest.txt`.
- `dispositions.md`, `results.json`, `checker-output.txt`, `review-ledger.md`, `keyscan-record.txt` and
  `keyscan-run-folder.txt`.

Check each of the following and report concretely, with file references:
1. **Order and freeze.**
   - Was the design frozen and pushed before the VM started? See design/FROZEN.sha256, the README,
     common/vm-start.txt and common/effective-policy.txt.
   - Do the executor packet's design-cases.md and amendments.md match the frozen design and amendments?
   - Does amendments-final.md reproduce the reviewer's amendments without change?
2. **Each case.** Does the evidence in its final run demonstrate the disposition against the governing expectation
   (an amendment over the frozen text)?
   - A passed case needs positive evidence on a captured screen for every expected result. A blocked or failed case
     needs evidence of why. Flag any overstatement, understatement or silently skipped expected result.
   - Judge in particular the dispositions that differ from the executor's raw verdict:
     - FUP-10 (failed -> passed on an arithmetic correction, plus the claim that Request IDs were re-read after reopens);
     - FUP-15 (failed -> blocked: wrong conversation opened);
     - FUP-06 (failed kept, but for a reason found in the raw terminal output).
   - Also judge:
     - FUP-14 (failed; is the reason supported by screens and by the ledger read?);
     - FUP-17, 18 and 19 (blocked; is blocked rather than passed or failed right, and are the conversation-level
       arithmetic and the supplementary relay-usage claims correct?);
     - FUP-03 and FUP-05 (the D25 re-checks of ACC-06 and ACC-15);
     - FUP-21 to 23 (realtime);
     - whether the integrator's supplementary reads (ledger, relay log, pty.raw) are kept apart from what the executors
       saw.
3. **Isolation.**
   - Probes per run; receipts bound to the right run, case, design and candidate; the guard; the gate.
   - The relay's real-key paths and caps (realtime 2, Moonshot 4) and the guest network changes.
   - Key deletion before the clone was deleted, clone deletion, and the lock.
   - Whether the stated limits are complete.
4. **README.** Do its totals and claims (spend, the Tart-vs-UTM notes, the key scan, the issue references) match
   results.json, dispositions.md and the evidence? Also report anything else a reviewer should know before a human
   accepts.

Your verdict is about the evidence, not about whether the product is good. "pass" means the record honestly supports
every disposition and claim as written; "fail" means at least one must be corrected. List each required correction
precisely. Be concise. End your answer with exactly one line: `VERDICT: pass` or `VERDICT: fail`.
