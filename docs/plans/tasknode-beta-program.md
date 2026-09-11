# PF-79 — Public Corbanu Desktop beta-testing program

Status: **planned / not executed**. Authority: Travis's September 11 request,
confirming Alex's call requirement. This is an amendment to workstream 3, not a
fourth initiative. Two sequential draft sprints follow the existing Task Node
integration sprint. No beta branch, release, public assignment or schedule has
been created. The three new personal workstream tasks are coordination records.

## Outcome and limits

Give real users small, reproducible manual tests of an isolated Desktop beta,
and give Travis a short evidence-backed decision queue. Agent and human results
are labeled separately. Passing tests reduce uncertainty; no program can promise
bulletproof coverage, and a tester's pass does not authorize a release.

Reuse existing Desktop packaging, native `/tasknode`, Task Node lifecycle and
the private management projection. Do not build another scheduler, wallet, bounty
marketplace, evidence cloud or Desktop application inside these two sprints.
If a prerequisite is absent, stop at the contract and re-slice implementation
with Travis. PF-64 rollout and PF-75 tester-calibration drafts are references,
not newly activated initiatives or hidden implementation dependencies.

## Sequence and bounded deliverables

| Order | Sprint | Deliverable / timebox estimate |
| --- | --- | --- |
| 1 | PF-80-S01 | Main tooling port allocated for kickoff; native progress and internal delivery acceptance still gated |
| 2 | [PF-79-S01](../sprints/current/initiative-delivery-control/pf-79-s01-beta-channel-and-test-contract.md) | One isolated Desktop candidate/channel configuration plus versioned public test/evidence contract; 3–5 working days after gates |
| 3 | [PF-79-S02](../sprints/current/initiative-delivery-control/pf-79-s02-public-beta-pilot.md) | One native public-assignment adapter and a two-week bounded pilot with triage; 3–5 setup days plus observation |

Estimates are provisional, not deadlines. Both drafts have UNALLOCATED execution
coordinates. Assign one builder, a separate reviewer/tester and one manager per
active sprint; allocate exact paths before ready. The manager alone updates
shared plan/dashboard records; workers emit separate redacted progress reports.

## S01: branch, channel and feature boundary

- Resolve the **Corbanu Desktop** repository, package identity, supported pilot
  OS, release owner and existing installer/updater before allocation. The inspected
  CorbanuTerminal remote had no matching beta/Desktop branch; this does not prove
  a separate Desktop repository is absent. CLI/TUI tests cannot substitute for it.
- Proposed branch name: `beta/desktop`, **not created or approved as final**.
  Pin its reviewed main base; only reviewed, traceable forward-merges/cherry-picks.
  Name the owner and review checks; no autonomous merge to main or stable channel.
- Reuse existing signing/packaging: prerelease artifacts and a separate update
  feed; never `latest` on the stable feed. Version, source SHA, digest, platform,
  installer signature and enabled flags form the immutable candidate manifest.
- Side-by-side app identity and separate configuration/auth/storage home. Verify
  stable remains unchanged after beta install, update, rollback and uninstall.
  Do not copy production credentials into beta. Fixtures use disposable accounts.
- Reuse the native feature registry/delivery contract for beta discovery and
  execution. Default OFF hides invitations and blocks new assignment/publication
  at UI and service boundaries. Recovery/help/export and existing test data remain
  reachable when OFF. Toggling OFF is not deletion of already-public tasks.
- Exercise launch, update, failed update, cancel, restart, auth expiry/relink,
  rollback and cleanup through documented user controls. A helper-only repair or
  database/config-file edit is not acceptable recovery evidence.

## Native public Task Node contract: verify before build

First-party source inspected at
[`40d2df72710a644f33a2b30831061e8265716db0`](https://github.com/postfiatorg/tasknode/tree/40d2df72710a644f33a2b30831061e8265716db0):

- [`scripts/bm/writes.mjs::taskCreate`](https://github.com/postfiatorg/tasknode/blob/40d2df72710a644f33a2b30831061e8265716db0/scripts/bm/writes.mjs)
  checks board scope, a configured budget, candidate identity, routing constraints
  and bounded reward bands before network-task generation. Its default is dry-run.
  A zero reward argument is not proof of a zero-cost offer: defaults can apply.
- [`server/hive-routes.js`](https://github.com/postfiatorg/tasknode/blob/40d2df72710a644f33a2b30831061e8265716db0/server/hive-routes.js)
  exposes read-only public Hive task detail; the
  [projection](https://github.com/postfiatorg/tasknode/blob/40d2df72710a644f33a2b30831061e8265716db0/server/repositories/hive-projects.js)
  limits public description/summary fields and excludes raw evidence plaintext.
- The [scoped board command contract](https://github.com/postfiatorg/tasknode/blob/40d2df72710a644f33a2b30831061e8265716db0/docs/wiki/architecture/board-manager.md)
  uses expiring board grants and durable command receipts. Existing human terminal
  login is not evidence that this account can issue public board assignments.

Inference: reuse a permitted native network/Hive path, not personal task requests
masquerading as public tests. At implementation, re-pin code and verify production
capabilities, board ownership, eligible tester routing, grants, budget and precise
public fields. Source inspection is not a live permission check. If supported
native primitives cannot satisfy the contract, stop and propose a bounded change.
Do not modify or compete with the production board manager without its owner's
approval; use its approved intake or an explicitly delegated beta board lane.

Public task description must contain an immutable link to the full public test
card if native description limits would truncate it. Verify that link and the
actual generated assignment signed out; ensure wording preserves the approved
build, steps and safety limits. Generated offers are not accepted assignments.
Human approval must precede any publication that itself commits reward terms.

## Public test card v1 (required fields)

One card means one case × exact build × platform × tester slot. Include:

1. Case/version, campaign, candidate manifest URL and SHA/digest, platform, required
   feature settings, estimated minutes, due window and eligibility.
2. Plain objective and synthetic prerequisites; install/reset instructions and
   proof of the displayed version. No real funds, seed, paid key or private account.
3. Numbered actions with expected result at each checkpoint. Include cancellation
   or recovery and final cleanup, not just the happy path.
4. Evidence requirements: observed results, safe screenshots with capture points,
   environment/version, reproducible steps and pass/fail/blocked reason. Minimum
   sufficient evidence only; no home directories, raw logs or prompt history.
5. Explicit public fields, approved evidence destination/access, consent, retention
   and redaction instructions. Warn that public artifacts may persist. Never ask
   users to post credentials or security-exploit details; name a private escalation
   route. It must exist and be reachable before publication.
6. Stop rules, help link, reviewer/response window, acceptance rubric and appeal
   path. Reproducing a defect can satisfy a tester's assignment; do not pay for a
   predetermined pass. Any reward terms require a separately approved budget.

Template example, **not publishable until candidate and destinations are real**:
`AUTH-RECOVER-01 / <candidate> / <one pilot OS>`: install the isolated beta;
confirm displayed version; sign into a disposable test identity; use supported
test revocation/expiry; restart; observe a clear expired state with a usable relink
action; cancel once, then relink; verify beta resumes and stable remains unchanged;
sign out/clean up. Capture redacted version, error and recovered-state checkpoints.
If deterministic expiry cannot be triggered safely through approved controls,
mark the case blocked; never improvise by editing credential files.

## S02: recurring assignment and review

Proposal for approval: two-week pilot, one frozen candidate per week, one OS,
three test cases per candidate, two independently assigned humans per case:
at most **12 assignments**, 10–20 minutes each. No open-ended public recruitment
or parallel implementation sprint. After the pilot, cadence/caps continue only
with an explicit operating decision; no automation is installed by this plan.

Manager selects cases from sprint acceptance plans and unresolved user journeys;
build/reviewer agents draft cards. Human approves each public payload and reward
envelope during pilot. A single trusted publisher uses the supported native path;
workers never receive its broad credentials. Prefer narrowly scoped board grants.
Persist idempotency before sending: campaign/candidate/case/platform/tester slot;
reconcile request → generated task → accepted assignee without creating duplicates.
Two tester slots are deliberate replication, not accidental duplicate tasks.

Read actual service lifecycle. Track offered/accepted/submitted/needs-evidence/
reviewed separately from product pass/fail/blocked and from rewards. Claim races,
no-shows, stale builds and superseded candidates preserve history and require a
reviewed reassignment; do not silently cancel, recreate or carry a pass forward.
Before going live, verify whether task creation triggers downstream automatic
review/reward behavior; obtain approval for the whole lifecycle or remain blocked.

Evidence is untrusted. Quarantine attachments/URLs; no instruction execution or
automatic raw upload. Human reviews privacy and adequacy before any public
summary. Version/signature checks, independent reproduction and seeded broken
fixtures calibrate reviewers. LLM judgments are recommendations, not acceptance.
Inaccessible artifacts are blocked evidence, not grounds for punishing testers.

Agent triage groups duplicate defects and produces a daily top-three decision
queue: severity, exact candidate/case, reproduction, suggested owner and next
action. Reserve up to 20 minutes of Travis's approximately one-hour daily review
budget; escalate security/data-loss immediately through the agreed channel.
Pause new assignments if review is over one business day late or the budget is
exhausted. Record deeper weekly review separately. A reviewer cannot self-approve
their own implementation; Travis remains final product authority.

The existing half-hour private dashboard shows campaign/candidate, public card
links, offered/accepted counts, evidence awaiting review, blockers, triage owner,
machine/run progress and next human decision. It must not republish raw evidence
or turn private dashboard documents into public test assets.

## Acceptance and launch decisions

- Verify public visibility signed out and tester eligibility signed in; prove the
  full card remains available despite native projection limits.
- Seed wrong-build/false-pass evidence, duplicate sends/claims, offline retries,
  expired or wrong-board credentials, secret attachments and a broken auth flow.
  Each must fail safely or become a visible blocked review; retain the same IDs.
- Prove OFF prevents new sends, in-flight outcomes reconcile, and restarting or
  relinking needs no hidden repair. Already-public tasks need explicit lifecycle
  handling; disabling does not retract a committed offer or its obligations.
- Pilot exit: 12 assignments reconciled (including declined/no-show/blocked),
  critical cases independently reviewed, zero open critical security/data-loss
  defects, and missing coverage explicitly blocked. Human approve/hold decision
  records review load, defect yield, retries and next cadence; not a pass vote.
- Stable promotion still requires the existing exact-candidate true-TUI,
  applicable Desktop GUI, live-repository, named human and release gates.

Open decisions: Desktop repo/branch/OS/owner; public board and scoped publisher;
private support/evidence destination and retention; tester cohort and disclosure
terms; approved reward/zero-reward feasibility; cadence/budget; launch approver.
Travis approves scope/spend/launch; nominate an independent technical reviewer
and public-task operator before ready. No remote credentials provisioned by this plan.
