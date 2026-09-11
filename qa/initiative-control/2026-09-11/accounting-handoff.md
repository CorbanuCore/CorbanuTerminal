# Workstream 2: accounting contract acceptance

Status: contract review pending; runtime human testing is not ready.

Sequence: PF-60-S01 contract/fixtures → S02 persistence/replay → S03 inspectable
totals → S04 acceptance. S01 is allocated for post-merge Astra High kickoff;
S02–S04 remain draft. Allocation alone is not a running-agent claim.

For S01, Travis or a named delegate must accept:

1. Separate measured tokens, estimated cost, invoiced cost and subscription
   allowance where a provider supplies it, plus the distinct Corbanu API balance.
   Do not restore legacy Corbanu Plan entitlements. Missing price or incomplete usage must remain unknown, not zero.
2. Count retries, cached/reasoning tokens and interrupted requests explicitly.
   Verify a parent with two children is not double counted.
3. Approve price provenance/effective date, currency precision, retention and
   the historical-unknown policy. No customer rebilling or live price change.
4. Independently hand-calculate the synthetic fixture totals, including replay
   twice and process restart. No private prompts or billing records in reviews.
5. Freeze the accepted contract and independent implementation worktree before
   any schema/runtime sprint; new controls remain hidden until qualified.

Later user testing must let a human explain each aggregate through constituent
requests, cancel/reopen, and recover through supported controls. A model's
agreement or successful document check does not satisfy this gate.

Named acceptance, date, fixture digest and actual results: pending.

Plan: [accounting](../../../docs/plans/active/portfolio-agent-cost-accounting.md).
