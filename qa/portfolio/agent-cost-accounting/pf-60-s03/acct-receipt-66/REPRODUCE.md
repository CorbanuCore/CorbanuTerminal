# Reproduce round 66

Read `docs/development/test-isolation.md` first. Run from the repository root;
all paths below are under `qa/portfolio/agent-cost-accounting/pf-60-s03/`.

1. Run `python3 -B acct-inherited-64/page_controls.py acct-receipt-66/NEW-PAGES`.
   Require 74 rejections before success receipt; exactly 34 of the original 38
   accepted by the frozen round-62 auditor; all 36 additional cases accepted
   by both frozen earlier auditors. Require both old baseline successes.
   Do not edit the frozen nonmonetary page digests to fit a mutant.
2. Run `python3 -B acct-scope-62/coverage_controls.py acct-receipt-66/NEW-REMOVALS`.
   Require all 28 removals rejected; baseline 9 priced, 4 unknown-cost and
   2 zero-recorded subtotal pages.
3. Run `python3 -B acct-scope-62/audit_scope.py acct-scope-62/scope-run-01
   acct-receipt-66/NEW-SCOPE.json`. This rechecks the round-62 capture, not
   round 61 and not a new product execution.
4. `gate.py` runs one lane per invocation, from codex-rs with the shared
   dedicated target and four test threads. For a fresh campaign, copy it into
   another new allocation directory under the same QA parent so it writes
   fresh logs. Invoke `prerequisites`, inspect its exit/log, then
   `core-default`, `core-feature`, and `tui`, inspecting each before the
   successor. It refuses to overwrite lane logs. The three Rust commands are
   the exact guarded commands recorded in gates-01/*.json.
   Stop on a native credential prompt or live-profile access; no retry or
   successor is authorized on a contaminated run.
5. `freeze_evidence.py` exported the current fixed paths after all gates
   finished. It requires the assigned HEAD and exclusively creates artifacts;
   use distinct paths for any export replay. Check `capture-bindings.json`
   against both original viewport sequences and the quoted disposition;
   compare `next-step-source.json` with the exact source base.

Mutation directories are regenerable and ignored; recipes, old auditors,
baseline receipts, every mutant's stdout/stderr/exit, and gate logs (lossless
gzip) are retained. No prior capture or receipt JSON is overwritten.
The content digest contract is deliberately specific to reader-run-05; new
product captures require separately reviewed expectations. It is not a currency
parser or a blanket approval of arbitrary page wording.
