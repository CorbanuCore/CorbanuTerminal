# Alex's product portfolio — proposed sprint map

Planning snapshot: 2026-09-09. **Draft only. No new initiative, sprint, worker, automation, external message, account action or spend has been activated.**

Publication update, 2026-09-10: these drafts are being integrated onto the
receiving `main` baseline recorded in the [scrum packet](scrum-2026-09-10.md).
The user approved a direction of up to three independent initiatives, with
sequential sprints within each. Main's existing policy and active allocations
are preserved by this planning-only publication; their transition needs a
separate reconciled change. The private delivery-control implementation is
outside this merge and is not an active plan on this receiving baseline.

## Read this first

The original recording yielded **17 workstreams**: security/recoverable credentials already has active owners. With the [September 10 follow-up](call-followup-2026-09-10.md), the other **16 proposals contain 52 single-feature draft sprints (PF-60 through PF-75)**. Two added PF-70 drafts cover the index API candidate and a later creator decision; research tickets do not create more initiatives.

These are not 52 promises to implement. Delivery plans specify bounded implementation and qualification; uncertain ideas specify the research, paper validation and human decisions needed to make implementation plannable. A no-go is a valid research outcome. An unknown provider, buyer, instrument, data right or existing harness is a hard gate, not permission for an agent to invent one.

The proposed priorities and effort ranges are planning hypotheses. Existing P0 security and market-data commitments retain precedence. Current source state must be reconciled before dispatch.

## Source and attribution

- Source ID: `SRC-ALEX-20260909-01`, a private product-discussion recording retained outside this repository. Its transcript and source-location mapping are private evidence, not published documentation.
- Supporting management research: `MGMT-RESEARCH-20260909`, retained outside this repository. Only the curated product topics and bounded workflow recommendations are reproduced here.
- Transcript speaker clusters are automatic and not fully human-audited. Timestamps locate discussion; they do not establish binding commitments or precise speaker identity.
- The source is a conversation, not an approved product spec. Several topics are Alex's brainstorming; management/QA is the user's explicit enabling request.
- This planning pass does not reverify current vendor terms, financial mechanics, prices or research claims. Relevant sprints require dated primary-source research and qualified review where needed. No investment or legal recommendation is made here.
- Only curated product context is sent to the requested reviewer, not the complete private recording/transcript.

## Coverage and bounded deliverables

| Feature | Plan | Group | New sprints | Planned endpoint | Recording anchor |
| --- | --- | --- | ---: | --- | --- |
| Existing security/auth | [P0 security](active/p0-security-levels.md) plus newer provider-auth work | Terminal | 0 | Existing implementation and qualification; no duplicate plan | 01:57:29–02:02:41 |
| PF-60 | [Unified agent cost and usage accounting](proposed/portfolio-agent-cost-accounting.md) | Terminal | 4 | Bounded delivery | 01:58:15–02:02:41 |
| PF-61 | [Plan and model-serving backend reconciliation](proposed/portfolio-plan-backend-reconciliation.md) | Terminal | 3 | Evidence / go-no-go | 01:58:15–02:02:41 |
| PF-62 | [Compute referrals and provider onboarding discovery](proposed/portfolio-compute-referrals.md) | Terminal | 3 | Evidence / go-no-go | 01:58:15–02:02:41 |
| PF-63 | [Repeatable cross-harness quality, runtime and cost benchmarks](proposed/portfolio-recurring-benchmarks.md) | Terminal | 3 | Bounded delivery | 01:49:00–01:51:57 and 01:58:15–02:02:41 |
| PF-64 | [Controlled rollout, support ownership and adoption](proposed/portfolio-rollout-support.md) | Terminal | 3 | Evidence / pilot decision | 01:39:36–01:41:00; 01:51:57–01:57:29 |
| PF-65 | [Private inference feasibility for Ambient and Corbanu](proposed/portfolio-private-inference-research.md) | Research | 3 | Evidence / go-no-go | 02:05:49–02:12:56 |
| PF-66 | [Private enterprise Terminal opportunity qualification](proposed/portfolio-private-enterprise-terminal.md) | Research | 3 | Evidence / go-no-go | 02:10:03–02:14:53 |
| PF-67 | [Real-time domain fine-tuning feasibility](proposed/portfolio-domain-finetuning.md) | Research | 3 | Evidence / go-no-go | 02:14:53–02:18:00 |
| PF-68 | [On-chain stock access and broker-wrapper feasibility](proposed/portfolio-onchain-stock-access.md) | Financial research | 3 | Evidence / go-no-go | 01:00:25–01:03:37 |
| PF-69 | [Cash management and basis-strategy paper validation](proposed/portfolio-cash-basis-product.md) | Financial research | 3 | Evidence / go-no-go | 01:24:37–01:31:00; 02:26:45–02:28:15 |
| PF-70 | [Index-creation API, replayability and creator ecosystem](proposed/portfolio-acceleration-index.md) | API / financial research | 5 | Gated API candidate / creator decision | Earlier 02:28:15–02:31:12; latest 02:09–06:16 |
| PF-71 | [Tokenized options-basket feasibility](proposed/portfolio-options-basket.md) | Financial research | 3 | Evidence / go-no-go | 01:41:00–01:46:46 |
| PF-72 | [Campaign Tracker and agent ROI integration](proposed/portfolio-campaign-roi.md) | Adjacent | 3 | Evidence / pilot decision | 01:31:00–01:38:00 |
| PF-73 | [Text-improvement harness qualification](proposed/portfolio-text-improvement.md) | Adjacent | 3 | Evidence / pilot decision | 01:38:00–01:39:36 |
| PF-74 | [Stock research and content distribution workflow](proposed/portfolio-research-content-distribution.md) | Adjacent | 3 | Evidence / pilot decision | Research/content discussion across the recording; 01:51:57–01:57:29 adoption context |
| PF-75 | [Native agent management and independent acceptance pilot](proposed/portfolio-agent-management-pilot.md) | Adjacent / enabling | 4 | Evidence / pilot decision | User's management request; Alex 01:46:46–01:49:00 and 02:25:25–02:26:45 |

The main Terminal themes are security, accounting, backend coherence, referrals, benchmarking and rollout/support. The three research bets are private inference, enterprise private Terminal and domain adaptation. PF-68/PF-69/PF-71 stay research/paper-only; PF-70 now plans an explicitly gated, non-trading API candidate. Campaign ROI, text improvement, research/content distribution and management/QA are adjacent/enabling.

PF-74's workflow is an inferred packaging, while its latest-call research topics have explicit meeting owners in the follow-up register. PF-75 primarily responds to the user's workload problem. Validate product priority with Alex before either receives a lane.

## Existing work: reuse and reconciliation, not duplication

| Existing boundary | Evidence inspected | How the portfolio treats it |
| --- | --- | --- |
| Security, protected credentials and human/TUI qualification | [Active P0 plan](active/p0-security-levels.md); PF-13, PF-26, PF-27, PF-35, PF-37 and related current sprints | Remains the first implementation initiative. Expiry/recovery tests belong to its owning auth boundary, not a new manager workaround. |
| Provider lifecycle and reauthentication | [Receiving main's active provider-auth plan](active/unified-provider-auth.md), including PF-57/PF-58 evidence and subsequent release records | Reuse the integrated record before opening an auth fix. A historical worker result alone does not establish installed behavior. |
| Historical large-image gateway/TUI work | The original recovery planning checkout carried a different active-plan ledger | Not an active plan on the receiving main baseline. Backend audit may reconcile historical artifacts; this merge does not resurrect or close that work. |
| Exact-model native review | [Existing PF-14 proposal and seven drafts](proposed/arbitrary-model-autoreview.md) | No second native autoreview implementation. Current external review workflow is a development tool, not proof PF-14 ships. |
| Agent hierarchy, mailbox, resume, Task Node | Existing core agent control/registry and tasknode-session modules; product spec's live MVP | PF-75 tests/reuses the native lifecycle first. A second scheduler requires a separate product decision and evidence of a concrete native gap. |
| Campaign Tracker | [Product spec, “Campaign Tracker — LOCAL PILOT CANDIDATE”](../corbanu-product-spec.md) and receiving main's `qa/campaign-tracker/2026-09-06/README.md` | PF-72 must inspect/reuse the existing implementation and distinguish unqualified ROI additions from existing capture/replay. No duplicate tracker or live write-back qualification is implied. |
| Text harness | Discussed in transcript; authoritative external system still needs owner confirmation | First sprint requires owner-provided location and permissions; no guessed replacement. |

**Repository provenance:** drafts originated on `recovery/corbanu-drive-2026-09-02`
at `6f8f4ce46446e951437e1ce7e7c4a526e5b9e546`. The selected receiving main baseline
is `3cec54d9917b776bedaffb586247d7cd7c633df6`, with security and unified provider
auth active. Only the planning packet is imported; root policy, product spec,
runtime, existing release records and active allocations are preserved.
Seven pre-existing sprint-link/ID errors on that baseline must be reconciled
before new dispatch. Their exact baseline and no-new-error checks are recorded
in `qa/portfolio/2026-09-10-main-integration/`. Do not merge the recovery tree wholesale.

PF-60–PF-75 were unused in the readable local Corbanu worktree/doc inventory at
planning time. The publication checks also compare the new sprint IDs with the
receiving main ledger. Recheck against any later integrations before activation;
a draft inventory is not a global allocation service.

## Dependency and integration map

Every plan is serial by default. Each current sprint links one feature, a predecessor where needed, a deliverable, ordered work, tests, exclusions and exit evidence.

| Prerequisite | Enables | Why |
| --- | --- | --- |
| Reconciled canonical branch, active slots and existing auth/security state | Any executable draft | Prevent stale baselines, duplicate work and allocations beyond the receiving policy |
| PF-60-S01 accounting contract | PF-72-S02 spend-derived attribution fixture | Agree measured/estimated/unknown costs before computing ROI; not a prerequisite for S01 or PF-75 operational visibility |
| PF-65-S03 private-inference decision | PF-66-S02 enterprise demonstration brief | Do not sell privacy stronger than the evaluated boundary |
| PF-70-S03 API product decision | PF-70-S04 API candidate → S05 optional creator decision | Freeze actual repository, contract, replay claims, flags and acceptance before implementation |
| PF-75-S01 authority contract | PF-75-S02 tester calibration → S03 rehearsal → S04 operating decision | Prove delegation and acceptance behavior before increasing concurrency |
| Approved S01 contract and completed predecessor | Each plan's later sprints | Resolve unknown paths, inputs, thresholds, budget and test commands before readiness |

PF-68-S03 diligence gates future tradability/distribution claims, not PF-70
API design or non-trading replay experiments. No hard dependency is added merely because two subjects are related:
accounting does not require rebuilding the Plan backend; the ROI fixture can use
approved synthetic costs; paper finance research does not require enabling live
trades; the management rehearsal does not require activating all portfolio plans.
If later code changes touch shared state migrations, provider/auth contracts,
module registration, lockfiles or task transport, assign one owner and serialize
the seam. Integration tests run on the combined receiving tree, not only each worker branch.

## Proposed queue, not a start order authorization

| Queue | Candidate work | Rationale / exit |
| --- | --- | --- |
| Protect current commitments | Reconcile and finish the selected active security/auth/release work; Alex continues existing data-rights responsibility | No new initiative displaces the 2026-10-08 P0 commitments by implication |
| First enabling option, after a slot decision | PF-75 native-management/QA contract and seeded tester calibration | Tests whether more parallelism would reduce or increase the user's burden |
| First product option | PF-60 accounting; alternatively PF-61 backend audit if current defects make accounting ambiguous | Make spend and existing backend truth inspectable |
| Then commercial basics | PF-62 referrals and PF-64 support/rollout readiness | Resolve terms and support load before wider marketing |
| Existing release obligation | Required benchmark cycle from current canonical ledger; PF-63 improves repeatability later | Current benchmark gates are not deferred behind a new proposal |
| Evidence-first queue | PF-65–PF-74, selected one at a time when ownership and budget exist | Run cheap kill tests; PF-70's API delivery remains blocked behind its explicit S03 decision |

There is no deadline commitment for the new proposals. Research sprints propose
0.5–2 analyst-days after required inputs arrive, code sprints 2–4 builder-days
plus testing, and QA sprints 1–2 tester-days after candidate readiness. These are
**timeboxes for re-slicing**, not effort estimates backed by completed discovery.
They exclude waiting for owners, partner terms, data rights, counsel, credentials
and release gates. Do not sum them into a delivery date.

## One-hour human operating budget

The user's stated availability is about one hour daily, with deeper reviews.
A proposed split is 10 minutes on triage, 30 on evidence packets, 20 on consequential
decisions. Deeper product/legal/security reviews are additional scheduled work;
do not hide them in a sixty-minute promise.

The bounded PF-75 calibration starts with **one builder and one independent
tester** and may test two builders after acceptance rehearsal and a slot check.
This experimental ceiling does not replace the user's desired operating model
of up to three independent initiatives with sequential sprints. Receiving main
currently permits two active plans and three reserved in-progress/blocked
sprints globally, including opt-in within-plan concurrency. Transition that
policy with explicit handoffs before using a third initiative; this merge does
not deallocate existing workers. A tester is a role within a sprint, not an extra
untracked implementation lane. A separate implementation sprint consumes a slot.

- At most two completed packets waiting for human acceptance; when full, stop starting new implementation and resolve/review existing work.
- At most three new consequential decisions presented per day. Urgent security incidents still escalate immediately; reduce ordinary work rather than suppress urgent alerts.
- The coordinator summarizes state but cannot grant authority, certify its own work, dismiss critical findings or change acceptance criteria after seeing results.
- Unknowns become a question with evidence, recommendation and consequence. They do not become heuristics that silently choose money, security or public commitments.
- Per-run spend and wall-time caps are approved before dispatch. Account subscriptions, machine ownership and model availability are not unlimited budgets.
- A failed worker may be retried only under an explicit bounded policy with failure evidence retained; no silent fallback to another requested model.

## Acceptance packet: the unit the human reviews

Each submitted packet must contain:

1. User outcome and acceptance criteria frozen before building.
2. Exact receiving branch/base, implementation commit and final binary or artifact digest.
3. Changed scope and authoritative decision; no hidden adjacent work.
4. Independent user-only success, failure/cancel, recovery and resume evidence.
5. Test commands, nonzero counts, expected/actual results, recording/checkpoints and known limitations.
6. Exact reviewer model/provider and findings with accepted/rejected rationale.
7. Security/disclosure/spend status and any human action still required.
8. Rollback/recovery plan, integration evidence and release gates if applicable.

A reviewer can say “changes look correct”; it cannot mark a feature shipped.
Missing/stale artifacts or a changed final SHA invalidate affected proof.
Successful snapshot/unit tests cannot override a failed user journey.

## The regression that should change how we test

For an expired login, the test user starts in the actually expired state,
observes the error, finds a supported recovery control, reauthenticates or chooses
an allowed alternative, resumes the intended task, then restarts and checks the
result persists. Repeat cancel, timeout, failed replacement and stale callback.

The fixture operator may create the expired state; the tester may not repair it
through shell/profile edits, inherit a secretly refreshed token, or ask the
implementer where the hidden workaround is. If no reachable recovery control
exists, the feature fails regardless of unit tests or a model's explanation.

PF-75 adds calibration: seed unrecoverable login, wrong-model substitution,
double-counted usage and stale-binary evidence. The tester must reject all four
before its acceptance signal can justify additional parallel work. This reduces
false confidence; it is not a claim of bulletproof testing.

## Hardware and tools: staged use

- This machine / Codex Desktop: human decision and integration cockpit; existing Terminal agent lifecycle remains execution authority.
- One available VM: isolated black-box acceptance with a fresh home and no developer repair shortcuts.
- Second VM or laptop: restart/install or independent reproduction where the selected flow benefits from it.
- Powerful CPU/GPU machines: allocated builds, full benchmarks or explicitly budgeted research experiments, not more speculative workstreams.
- Claude/Fable, DeepSeek and GMI: explicit per-task routes with permissions, privacy and cost recorded; no automatic credentials/config changes.
- Task Node: task organization and evidence projection through existing support, after its current usability and write permissions are verified. No Task Node records were created by this planning pass.
- Computer use: supplemental browser/OS acceptance where a real user journey needs it; actual-key tmux remains required for Terminal flows. No claim is made here about a new CLI integration being available.

## Decisions for our next workload discussion

1. Main is the chosen publication lineage. Who owns reconciliation of its inherited sprint IDs and the transition to three independent, sequential initiatives?
2. Should the next available initiative slot go to the management/QA pilot or accounting/backend clarity?
3. Who besides the user can accept research/commercial packets, and which decisions must remain with Travis/Alex/Jim under the product spec?
4. What initial per-run spend cap and maximum experiment budget should govern the pilot?
5. Which two speculative ideas would Alex be willing to park until another idea produces evidence?

Recommendation: do not add a permanent master-agent service yet. First run the
bounded native capability, tester-calibration and rehearsal sprints. If that
cannot reliably reject obvious failures, adding machines and agents will increase
the review backlog. If it works, expand one lane at a time from measured human load.

## Validation and independent review

Historical QA: `qa/portfolio/2026-09-09/` records review on the original recovery
baseline; its manifests are not claims about the receiving main tree.
Publication QA: `qa/portfolio/2026-09-10-main-integration/` records the scoped
integration checks and independent review. These repository records are not
promised as public-site links.
Latest-call amendment QA: `qa/portfolio/2026-09-10-call-followup/`; the earlier
50-sprint receipts remain historical, not evidence for these 52-sprint amendments.
No model review of these documents is product acceptance or a release qualification.
