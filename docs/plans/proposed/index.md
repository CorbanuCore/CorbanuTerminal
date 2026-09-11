# Proposed plans

Plans here are drafts with `status: draft`. They do not consume an active-plan
slot and do not authorize implementation.

| Proposal | Priority | Owner | Purpose |
| --- | --- | --- | --- |
| [Arbitrary-model Autoreview](arbitrary-model-autoreview.md) | P1 | Alex Good | Turns explicit second-model review into one secret-scanned, isolated, exact-runtime workflow across configured Corbanu providers |
| [Prompt-injection firewall and brokered authority](prompt-injection-firewall.md) | P0 | Jim Ricketts | Historical input reconciled into the active P0 security plan on 2026-08-28; no separate implementation authority |

## September 9 transcript portfolio

September 11: accounting is now [active workstream 2](../active/portfolio-agent-cost-accounting.md),
with S01 selected for kickoff. The other 15 portfolio initiatives remain draft;
[provider onboarding](unified-provider-auth.md) is deferred without changing its
completed evidence. [Current three-stream allocation](../main-workstreams-2026-09-11.md).

The [portfolio map](../portfolio-2026-09-09.md) covers Alex's product discussion
and adjacent/enabling ideas. After the [September 10 follow-up](../call-followup-2026-09-10.md),
the historical portfolio contains 16 initiatives and 52 sprints. The September
11 disposition above supersedes its earlier no-activation status.
The [September 10 scrum packet](../scrum-2026-09-10.md) records the receiving
`main` baseline and remaining activation decisions. No new active slot or worker
allocation is implied.

| Proposal | Suggested priority | Proposed accountable role | Bounded scope |
| --- | --- | --- | --- |
| [Unified agent cost and usage accounting](../active/portfolio-agent-cost-accounting.md) | P1 active | Codex accounting lane / Travis acceptance | PF-60; 4 sequential sprints; S01 kickoff selected |
| [Plan and model-serving backend reconciliation](portfolio-plan-backend-reconciliation.md) | P1 proposed | Jim Ricketts (proposed) | PF-61; 3 draft sprints; decision |
| [Compute referrals and provider onboarding discovery](portfolio-compute-referrals.md) | P1 proposed | Alex Good (commercial lead, proposed) | PF-62; 3 draft sprints; decision |
| [Repeatable cross-harness quality, runtime and cost benchmarks](portfolio-recurring-benchmarks.md) | P1 proposed | Jim Ricketts (proposed) | PF-63; 3 draft sprints; delivery |
| [Controlled rollout, support ownership and adoption](portfolio-rollout-support.md) | P1 proposed | Alex Good (adoption) / Jim Ricketts (support), proposed | PF-64; 3 draft sprints; pilot |
| [Private inference feasibility for Ambient and Corbanu](portfolio-private-inference-research.md) | P2 proposed | Privacy research lead (assignment pending) | PF-65; 3 draft sprints; decision |
| [Private enterprise Terminal opportunity qualification](portfolio-private-enterprise-terminal.md) | P2 proposed | Alex Good (commercial lead, proposed) | PF-66; 3 draft sprints; decision |
| [Real-time domain fine-tuning feasibility](portfolio-domain-finetuning.md) | P2 proposed | Domain-model research lead (assignment pending) | PF-67; 3 draft sprints; decision |
| [On-chain stock access and broker-wrapper feasibility](portfolio-onchain-stock-access.md) | P2 proposed | Alex Good (product/commercial lead, proposed) | PF-68; 3 draft sprints; decision |
| [Cash management and basis-strategy paper validation](portfolio-cash-basis-product.md) | P2 proposed | Alex Good (strategy owner, proposed) | PF-69; 3 draft sprints; decision |
| [Index-creation API, replayability and creator ecosystem](portfolio-acceleration-index.md) | P2 proposed | Alex Good (index owner, proposed) | PF-70; 5 draft sprints; gated API candidate / creator decision |
| [Tokenized options-basket feasibility](portfolio-options-basket.md) | P2 proposed | Alex Good (financialization lead, proposed) | PF-71; 3 draft sprints; decision |
| [Campaign Tracker and agent ROI integration](portfolio-campaign-roi.md) | P2 proposed | Alex Good (campaign owner, proposed) | PF-72; 3 draft sprints; pilot |
| [Text-improvement harness qualification](portfolio-text-improvement.md) | P2 proposed | Editorial workflow owner (assignment pending) | PF-73; 3 draft sprints; pilot |
| [Stock research and content distribution workflow](portfolio-research-content-distribution.md) | P2 proposed | Alex Good (research/editorial lead, proposed) | PF-74; 3 draft sprints; pilot |
| [Native agent management and independent acceptance pilot](portfolio-agent-management-pilot.md) | P1 proposed | Jim Ricketts (integration lead, proposed) | PF-75; 4 draft sprints; pilot |

Promote a proposal only after every activation field in
[`../index.md`](../index.md) is complete, product authority has authorized the
outcome, and `python3 docs/plans/check.py` confirms that an active slot is
available.
