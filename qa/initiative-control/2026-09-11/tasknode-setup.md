# Workstream 3: native Task Node setup and first-progress acceptance

Status: native account link and three proposed-task mappings verified; publisher setup pending;
live server posting OFF.

Observed runtime: installed Corbanu 0.1.36. Native `/tasknode` was opened in an
actual tmux PTY; Link GitHub / Task Node opened Safari. Travis approved continuing
as IridiumMaster and had completed the first browser authorization. That temporary
request expired before terminal collection. Recovery used only `/tasknode link`,
the approved GitHub account selection, and `/tasknode status` in the same TUI;
no credential-file repair was used.

At 2026-09-11 08:29 UTC, Corbanu and Safari matched the same account identity and
wallet. GitHub account IridiumMaster corresponds to Task Node @iridiumeagle.
A fresh native CLI process verified an active, non-expired session and no pending
link. The server reported two outstanding accepted tasks, neither corresponding
to the selected three workstreams. No mapping was guessed or task modified.
No wallet password or recovery phrase was needed, read or imported.

The production callback's historical `tasknodeofficial-dev.fly.dev` hostname is
documented in the [official source](https://github.com/postfiatorg/tasknode#task-node).
No authorization URL, pending token or credential value is published here.

Required gates, in order:

1. **Verified:** user confirmed the account/authorization; native linking completed.
2. **Verified:** terminal status, separate-process persistence and browser identity
   agree; the account-scoped task list is readable. This does not prove broad
   worker profile isolation or Campaign Tracker entitlement on this old runtime.
3. Verify current server Campaign Tracker/native progress contract and required
   entitlement. Do not purchase or change a subscription to bypass a failure.
4. **Verified:** user authorized three dedicated personal tasks; generated offers
   were inspected and [mapped](../../../docs/plans/tasknode-workstream-tasks-2026-09-11.md).
   They remain Proposed. Verify eligibility for progress without acceptance;
   if acceptance is required, ask Travis. No team access grants.
5. Confirm remote publisher destination and credential scope before provisioning
   owner-only storage. Never distribute a broad terminal session to workers.
6. Review the exact first goal-only progress payload and workspace enrollment
   together. No raw logs, transcripts, financial details or local paths.
7. Enable only after prerequisites and payload acceptance; verify a single
   post, retry deduplication, disabled behavior and stale/expired-auth recovery.
   If blocked, preserve OFF and a visible actionable reason.

Wallet unlocking or signing is not assumed necessary for progress. If a genuine
native gate requires it, hand off to the protected prompt; never use a Task Node
recovery phrase as a Corbanu trading-wallet credential. Transactions, reward
claims, verification and task completion are outside this setup.

Account authorization and three-task creation: Travis approved, 2026-09-11.
Exact generated-task mapping is recorded; remote credential permission,
progress entitlement/lifecycle eligibility and first delivery receipt: pending.
No credentials have been provisioned to the Linux publisher or worker agents.

September 11 task-target sync prepared local goal-only outbox records while
delivery stayed disabled. The observed records had zero attempts, and enrollment
was unverified. Review the entire queued batch, including older run observations,
before any first-delivery qualification; do not enable a bulk flush assuming
only one new event exists. No outbox event is a Task Node delivery receipt.

## Planned follow-on public beta

[PF-79](../../../docs/plans/tasknode-beta-program.md) adds two draft sequential
sprints: Desktop beta channel/test contract, then public assignment/evidence
triage pilot. Human review next: identify Desktop repo/channel owner and approve
public-board authority, evidence destination, cohort and any reward budget.
No beta branch or public tester assignment exists from this setup. Personal
coordination records must not be described as the public beta program.

Plan: [Task Node / delivery control](../../../docs/plans/active/initiative-delivery-control.md).
