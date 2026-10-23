You are the security plan worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md first;
its decisions are binding.

Inputs:
- PF-83 VM result: /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/pf83-codeblind.result.md and the record
  qa/initiative-control/management-bootstrap/pf83-codeblind-35/ (commit cc1af43542 on main via integration).
- docs/plans/active/p0-security-levels.md (1,600 lines), docs/sprints/current/p0-security-levels/, docs/plans/index.md,
  docs/plans/check.py, docs/sprints/check.py, AGENTS.md.

Do, in one PR to main:
1. Close PF-83-S01: record the per-case results honestly (F01/F02/F06 pass, F05 fail, others blocked, pinned 0.1.42
   package), the GLM-route results from Oct 2, and the product defects found (typed text in an approval prompt can
   approve it with "don't ask again"; "Enable full access?" defaults to Cancel with no message; declined request
   renders both "You canceled" and "Ran ... (no output)"; zai-anthropic route has no max-output limit for glm-5.2).
   Mark them as handed to the TUI lane. Archive the sprint per the repo's archive rules.
2. Rewrite the active plan to be short (target under 300 lines): product intent, the three levels, decisions 2-5,
   the 19-sprint core graph with lanes, the tiered gate (per-sprint incl. videos; milestone gates), and links to
   history. Move historical prose to an archive file rather than deleting it.
3. Create the "P1 security hardening" plan (docs/plans/active/ or wherever check.py expects) and move the deferred
   sprint files to it with their dependencies intact. Rewire PF-41-S01 as stated.
4. Add the new picker sprint file: "Flagged /security picker: Permissive and Aggressive" in the TUI lane, with
   scope, feature-flag name proposal, Aggressive mapping onto existing controls, acceptance, and the per-sprint
   gate. Put it first in the TUI lane.
5. Replace the "one executable security sprint" rule with the 3-lane rule. If check.py enforces the old rule,
   change the checker in the same PR with tests.
6. Update the index tables. Both checkers must pass.
Merge when checks pass. Report the PR, line counts before/after, and anything you could not reconcile.
