You are an independent, code-blind reviewer of a QA acceptance record. Repository root is the current directory.
Audit ONLY `qa/portfolio/agent-cost-accounting/pf-60-s05/independent-acceptance-20261009/rerun-20261009/` (README.md,
FROZEN-DESIGN.md, captures/, data/, tools/) against the sprint record
`docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s05-collection-correctness-and-billing-basis.md` (ACs),
`qa/portfolio/agent-cost-accounting/pf-60-s05/billing-basis-defaults.md` and the first run's
`qa/portfolio/agent-cost-accounting/pf-60-s05/independent-acceptance-20261009/README.md`.
Do NOT read implementation source (codex-rs/**), PR diffs, worker logs or other review files.

Check, citing files:
1. Each verdict (PASS / FAIL / NOT VERIFIABLE) is supported by the cited captures and data; flag any overclaim.
2. Recompute the headline numbers yourself from data/provider-usage-*.jsonl and data/openai-pricing-20261009.txt
   (e.g. r1-t1 0.00625965, lx-t1 0.00480615, r3-terra 0.062594, r5-prio 0.0125224; /cost 0.009627 = product rates).
3. Token totals for Kimi, Claude and the Claude pane match provider-reported usage.
4. The overflow-note claim (9 of 18 views) and the AC7 claims (one gap warning per turn, result returned).
5. Safety: no credential values or key-shaped strings in the directory; the credential handling described is sound;
   the disclosed incidents and deviations are adequate.
6. Whether the recommendation to Travis follows from the evidence.
Output a Markdown review: a one-line conclusion (Supported / Supported with corrections / Not supported), then
numbered findings with severity (High/Medium/Low/Nit) and the exact correction you want. Be concise.
