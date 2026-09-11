# Main workstream integration — 2026-09-11

Authority: Travis requested merging the planning changes to main, then two
`gpt-6-astra` / `high` subagents for accounting and Task Node integration. PF13
remains untouched. This authorizes bounded development, not public beta launch,
spending/rewards, task acceptance, remote credential provisioning or release.

## Receiving source and preservation

Start from verified `origin/main` at `295aed26e53b17f919f7199ae1c9748b1b1250ba`.
Import only the reviewed workstream/task/beta planning amendments and the
three-initiative governance delta, not the recovery branch wholesale. Preserve
main's 52 portfolio drafts, newer security IDs/archives, Corbanu API balance
semantics, runtime and root release policy. Main's existing provider-auth plan
has zero current sprints; defer it under Travis's three-stream selection without
claiming completion or modifying any shipped implementation/evidence.

## Identity and runtime source transition

| Record | Current meaning | Treatment |
| --- | --- | --- |
| Main PF-76 | Provider profile persistence | Unchanged |
| Recovery PF-76-S01 | Private delivery control | Receiving name **PF-80-S01**; same logical work, remaining main port explicit |
| PF-79-S01/S02 | Public Desktop beta program | Draft, after PF-80-S01; no branch or public assignments |
| Personal Task Node task IDs | Three Proposed coordination offers | Preserve exact IDs; no acceptance or completion |

The live private dashboard still runs the recovery-source export at this merge.
Do not point its sync job at main before the tooling port is reviewed and
available there. Existing reports and outbox events retain original PF-76 IDs.
PF-80's worker prepares a migration/cutover manifest; manager reviews the entire
queue and aliases before cutover. Never remap main's unrelated PF-76 provider
sprint to the delivery-control task. The new main task mapping table names
receiving sprint identities, not a claim that server state has already changed.

## Security reservation handoff

Main previously reserved PF-27-S04 and PF-35-S01 concurrently. The recorded
PF-27 checkout at `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-isolated-broker`
fails `git status` and `rev-parse`: it is not a Git repository. Its original
`feat/p0-security-isolated-broker` ref is invalid. The clean backup worktree
`/Volumes/BackupBook/RecoveredCorbanuDrive-XID48590/Corbanu/recovery-worktrees/p0-security-isolated-broker`
and local `recovery/p0-security-isolated-broker-2026-09-02` ref preserve commit
`cdb82128988a63ad30cef97494a3b048505b5ddf`.

Return only that unusable allocation to **draft**, retaining its source pins,
unfinished checklist and historical qualification requirements. No process was
killed, branch deleted or checkout repaired. The visible active task “Set up
and build PF13 branch” is not interrupted. PF-35-S01 retains the security
reservation; the security owner must reconcile it with PF13 before further
security dispatch. Restoring PF-27 requires a fresh exact allocation and review;
draft is neither cancelled nor completed.

## Two bounded kickoff lanes

| Stream | First sprint | Worker write boundary | Stop/next decision |
| --- | --- | --- | --- |
| 2. Accounting | PF-60-S01 contract/golden fixtures | `docs/research/agent-cost-accounting/`, `qa/portfolio/agent-cost-accounting/pf-60-s01/` | Propose semantics with synthetic fixtures; no runtime/migration, pricing change or private customer logs; Travis accepts before S02 |
| 3. Task Node | PF-80-S01 internal tooling/main port | `scripts/initiative_control/`, `docs/research/tasknode-integration/`, `qa/initiative-control/pf-80-s01/` | Offline qualification and one-event preparation; no live API mutation, credential access, remote deploy or public testing |

After the planning merge, allocate distinct worktrees/branches from its exact
commit, record them in plans/sprints and set the selected sprints in progress
before dispatch. Workers report changed paths, commit, tests, blockers and next
human decision; only the manager edits shared plans/dashboard/CI. Subagents do
not launch additional agents or models, silently accept tasks, push main or
broaden scope. Reviewer/tester work remains part of each sequential sprint.

## Verification and publication status

The integration QA record owns exact commit/test/review/push receipts. Planning
and kickoff are not product acceptance. Native account setup is historical
operator evidence; no broad human credential is delegated. Public beta tasks
remain future work; proposed pilot caps are still human approval decisions.
