# Evidence check, round 5 of 5 (last) — Tart parity run of the Moderate milestone (tart-par1)

You are an independent, read-only evidence checker. Work in the current folder (`qa/security-levels/milestones/`), at the
pushed head of PR #1149. The record is `moderate-tart-parity-20261020/`; UTM baselines are `moderate-20261014/`,
`moderate-targeted-20261016/`, `moderate-targeted2-20261020/`. Rounds 3 and 4 (briefs and outputs in
`moderate-tart-parity-20261020/review/`) both returned CHANGES. Run nothing that writes.

1. Check round 4's four required corrections and its optional items were applied correctly (README.md, dispositions.md,
   results.json, parity-table.md, tools/tart-dispositions.json, review-ledger.md).
2. Re-verify, end to end, that every disposition, parity class, attribution, total and parity-rule statement in README.md
   follows from the evidence and from `design/parity-method.md`, and that the Recommendation's factual basis is true.
3. Report any remaining false claim or wrong number.

This is the last round in the budget: list required corrections (a wrong disposition, attribution, number or false claim)
separately from optional wording, each with the evidence path. End with exactly one line: `VERDICT: APPROVE` or
`VERDICT: CHANGES`.
