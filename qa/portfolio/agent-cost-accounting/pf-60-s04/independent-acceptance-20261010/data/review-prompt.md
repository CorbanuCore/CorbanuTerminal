You are a code-blind reviewer auditing an independent functional acceptance record for sprint PF-60-S04
(cost-accounting acceptance and handoff) in this repository. Read-only; do not edit files.

Read ONLY:
- docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s04-cost-accounting-acceptance-and-handoff.md (the sprint
  record and its mandate, which defines the criteria);
- qa/portfolio/agent-cost-accounting/qualification.md (the lane's own qualification);
- qa/portfolio/agent-cost-accounting/pf-60-s04/independent-acceptance-20261010/ (README.md, captures/, data/, tools/);
- for context only: qa/portfolio/agent-cost-accounting/pf-60-s05/independent-acceptance-20261009/rerun-20261009/README.md
  (waiver (a) origin).
Do NOT read implementation source (codex-rs/**), PR diffs, lane briefs, worker logs or other review files.

Audit:
1. Does every criterion in the sprint mandate (gaps i and ii, the AC10 waiver re-check iii, #368 limitations iv, and the
   cost flows the qualification lists: parent/child journeys, /side, consolidation, ephemeral exclusion, basis-only
   request, cancellation, missing price, duplicate retry, reopen, kill -9 restart, historical inspection, /cost date
   bounds, recompute, subscription never spent) have a verdict? Is anything missing or silently converted into a pass?
2. For each verdict, is it supported by the cited captures/data? Spot-check the numbers yourself: recompute at least the
   h1, h2 and h6 pay-per-use totals from data/recompute-all.txt rows and the published prices in data/, and compare
   with the /cost captures (exact USD lines). Check the retry attempt linkage, the cancellation and missing-price
   wording, the "Price: none recorded" line and absence of Price ID/source on basis-only pages, the side:/consolidation:
   turn labels and thread ids, the date-bound refusals, and the day/range totals against data/day-totals.txt.
3. Is the waiver (a) recommendation justified by the evidence, and correctly scoped?
4. Are NOT VERIFIABLE items honest and reasoned? Are safety claims (disposable homes, key handling, key scan, redaction)
   plausible from the tools and data? Any secret, real path or host left unredacted?
5. Any overclaim, wrong citation, arithmetic error or unclear wording.

Output a markdown review: a one-line conclusion ("Supported", "Supported with corrections" or "Not supported"), then
numbered findings by severity (Blocker / Major / Minor / Nit) with file references, then the numbers you recomputed.
Keep it under 120 lines.
