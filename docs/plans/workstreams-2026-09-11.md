# Three selected workstreams — 2026-09-11

**Historical operations-source snapshot.** The [main integration/handoff](main-workstreams-2026-09-11.md)
supersedes the receiving IDs and allocations below. Main delivery control is
PF-80-S01, not its unrelated provider-persistence PF-76. Runtime cutover and old
event reconciliation remain separate; no old receipt is relabeled.

Authority: Travis explicitly selected PF-13 first, accounting second and Task
Node integrations third. One sequential sprint per initiative; active selection
is not proof of a running agent, accepted feature or permission to spend.

| Slot | Initiative / sequence | Observed machine and checkout | Next gate |
| --- | --- | --- | --- |
| 1 | [PF-13 security](active/p0-security-levels.md); preserve S01–S06 archives, then dependency-gated S07 | Alex Linux: `/home/pfrpc/repos/CorbanuTerminal-pf13-s02`, `feat/pf-13-s02-scoped-vault-resolver` | Resolve branch freshness and existing PF-35 reservation before a new security dispatch |
| 2 | [PF-60 accounting](active/portfolio-agent-cost-accounting.md): S01 contract/fixtures → S02 persistence/replay → S03 totals → S04 acceptance | This Mac: management records in `/Volumes/CorbanuDrive/Corbanu/CorbanuTerminal`; independent implementation worktree not allocated | Accept cost vocabulary, unknown/estimated/billed distinctions, retention and source ownership; allocate S01 without shared-doc writes |
| 3 | [Task Node integration](active/initiative-delivery-control.md): link and task targets verified → bounded progress acceptance → PF-79-S01 beta channel/contract → S02 public testing pilot | This Mac: Corbanu setup; Alex Linux: `/home/pfrpc/corbanu-control`, live posting disabled; beta execution unallocated | Entitlement, proposed-target eligibility, remote credential permission, enrollment and first progress payload; beta remains draft |

## Inspected source boundary

Operations source: recovery branch `recovery/corbanu-drive-2026-09-02`, committed
base `6f8f4ce46446e951437e1ce7e7c4a526e5b9e546`, plus explicitly uncommitted
management changes. Its dashboard export is not claimed to be current main.

Main was fetched and verified at `295aed26e53b17f919f7199ae1c9748b1b1250ba`.
It has newer provider/auth ledgers and 52 portfolio drafts versus the older
operations source's 50. Main and the recovery source must not be wholesale
merged. PF-60 is the same accounting feature in both; the operations feature
called PF-76 collides with main's provider-persistence PF-76 and needs an explicit
ID migration before integration. Main's lifecycle policy is not changed here.

PF-13 remote tip is `c25e2825a2fe3fe63c8e1d58cc1e3aa82b0d1d04`; its clean Linux
checkout is `434635dd23b7a35944524cf9fa2b069312a94236`, behind by 28 commits.
Historical base is `1bdc515bff48a4d9048dae7d06c6214e884265bc`. No checkout
update, merge, reset or claim about a running PF-13 agent follows from inspection.
An old S05 in-progress label must not resurrect the receiving ledger's completed
S05. Existing PF-35 work and owner records remain unchanged pending handoff.

## Slot disposition and readiness

Large-image work moves to the proposed/deferred directory under Travis's explicit
three-stream selection. Neither completion nor cancellation is asserted; prior
archives and missing human/release evidence remain visible. No fourth initiative
is activated. Accounting consumes initiative slot 2 but no sprint/worker slot.

The manager owns plan, sprint-allocation and dashboard metadata. Workers write
only their assigned implementation/evidence scopes and independent reports.
Feature contracts must enforce OFF/ON and recovery before runtime changes land;
an active plan is not a feature-enable or release decision.

## Native Task Node setup boundaries

Use Corbanu's existing `/tasknode` or `corbanu tasknode link start/poll/status`
flow. The installed runtime inspected here is 0.1.36, not main's newer builds.
Verify durable linking, profile/account identity and task ownership before
provisioning a remote publisher. A Safari session is not a terminal session.

The Task Node wallet and Corbanu trading wallet must not be conflated. Do not
import recovery material merely to link GitHub or post task progress. Password
entry belongs to the user's protected native prompt. No recovery phrase,
credential values, authorization URLs or raw terminal traces belong in this
publication, a worker prompt or a review bundle.

Live posting remains OFF. No guessed task IDs, transaction,
reward claim, verification/completion submission, or account-permission expansion
is authorized by this selection. The subsequent September 11 request explicitly
authorized three personal coordination tasks; [verified Proposed targets](tasknode-workstream-tasks-2026-09-11.md)
are now mapped. No public beta tasks were created. Remote credential provisioning needs explicit
destination/account confirmation and owner-only storage. A goal event is progress
metadata, not task acceptance; compare native `/tasknode` support and the pinned
Campaign Tracker adapter before selecting the final transport.

## Follow-on beta planning amendment

Travis requested ongoing public human testing from a special Corbanu Desktop
beta branch. [PF-79's two draft sprints](tasknode-beta-program.md) stay within
workstream 3 and depend on its existing integration acceptance. They do not
create a branch, launch recruitment, promise rewards or start an automation.
Desktop source/channel and public-board authority must be verified before ready.
