# Reproduce acct-inherited-64

Read `docs/development/test-isolation.md` first. From the repository root,
prefix paths below with `qa/portfolio/agent-cost-accounting/pf-60-s03/`.
Use fresh output names; never overwrite an earlier receipt.

1. `python3 -B acct-inherited-64/page_controls.py acct-inherited-64/NEW-PAGE-CONTROLS`
   now runs the preserved round-61 baseline and 74 identity/content mutants.
   It asserts 24 baseline identities, all 74 revised rejections with no success
   receipt, and exactly 34 prior acceptances in the original 38-case group.
   The additional 36 cases evade every round-64 currency pattern and must be
   accepted by both frozen earlier auditors (rounds 62 and 64). Total round-62
   acceptances must be 70. Expected page digests bind the preserved capture's
   complete nonmonetary content; this is not a general money parser. Round-66
   replay artifacts are separate; the original 38-case receipt is historical.
2. `python3 -B acct-scope-62/coverage_controls.py acct-inherited-64/NEW-REMOVALS`
   requires all 28 prior amount-removal counterexamples to reject.
3. `python3 -B acct-scope-62/audit_scope.py acct-scope-62/scope-run-01
   acct-inherited-64/NEW-SCOPE.json` repeats independent arithmetic on preserved
   source evidence, not a fresh TUI execution.
4. `python3 -B acct-boundaries-58/campaign.py acct-inherited-64/NEW-GATES`
   builds prerequisites, runs the three guarded Rust lanes with the shared
   dedicated target and four test threads, then builds the feature CLI. Stop
   successor dispatch if any native credential prompt occurs.
5. Compare the 13 source entries in `source-evidence.json` to the exact base
   using their line bounds and full-file SHA-256. Read `inherited-disposition.md`
   against the quoted source and the actual named passing tests. Exclusion
   cannot be relabelled collection.

`collect_evidence.py` exported this allocation's fixed `gates-01` results.
It uses exclusive output creation and intentionally refuses to overwrite them;
a replay should use new exporter paths, not remove the original evidence.
Mutation subdirectories are regenerable and gitignored; mutation recipes,
prior auditor, baseline, every exit/error and consolidated results are retained.
Gate logs are retained losslessly as gzip. No credential store or live profile
is an input to any step.
