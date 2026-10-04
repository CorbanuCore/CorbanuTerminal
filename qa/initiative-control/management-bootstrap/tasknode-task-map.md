# Task Node task map: PF-80 work breakdown

This file links each Task Node task to the commits, QA records and PF-80-S01
ledger entries that make up its real work. Use it to check any evidence sent
against these tasks.

- Decision: `tasknode-writeback-20261002`, A1 (Travis, 2026-10-03): the closed
  workstream 3 task gets no further submissions; new tasks cover the actual
  breakdown and are mapped back to the work here.
- Account: GitHub `IridiumMaster` / `@iridiumeagle`, account
  `acct_oauth_73e1ab2f9f7cd01d3ee6230e`, `~/.corbanu` default profile.
- Created: 2026-10-03, one `tasknode request create --body-file` per task,
  source title "Corbanu PF-80 breakdown 2026-10-03". Each request was sent once.
  Task Node generated the tasks itself.
- Task state at 2026-10-03T09:58Z: all five **Proposed**. Travis accepted all
  five (2026-10-03, "I have accepted the tasks"); at 10:32Z each read
  **Accepted** with `canSubmitInitialEvidence` true. Corbanu did not accept,
  refuse or move any of them.
- 2026-10-03 10:52Z: all five read **Rewarded** (16.3 PFT total: 3a 2.5, 3b 2.8,
  3c 4, 3d 4, 3e 3).
- Round 3, 2026-10-03 10:45–10:47Z: initial evidence was submitted once for each
  of the five tasks, and each then read **Verification requested**. Receipts and
  verification responses are in the
  [writeback record](tasknode-writeback-live-20261002.md), round 3.
- Ledger references are the opening words of each bullet in
  [PF-80-S01](../../../docs/sprints/current/initiative-delivery-control/pf-80-s01-delivery-control.md).
  Line numbers change when the ledger is edited, so they are not used.

## Closed parent

| Task | State | Use |
| --- | --- | --- |
| `task_789a0f3bd75b41d1eca20cae698f04cf`, Corbanu workstream 3: Task Node integration and beta-test coordination | Rewarded 2026-09-15 (final) | Historical only. Nothing more is submitted against it (A1). `PF-80-S01` was repointed to `task_80628c7…` on 2026-10-03 (round 3). `PF-79-S01/S02` still point here; PF-79 beta work is paused. |

## New tasks

### 3a. `task_9d4fc2d8b21c3fd3ab2bc5ce6b18a188`: Record PF-80-S01 Bootstrap Deliverables With Commit Evidence

- Request: `req_6551ea328a15e5207852167a246fc4abfe2662854a1a7b0a552ae501e32a78f5`.
  Reward shown at generation: 2.5.
- Scope: bootstrap deliverables 1–3 (coordinator, owner transitions, native owner
  bridge).
- Commits: `c77123a7e9` (supervised fresh-Fable loop), `ca0474fe4` received at
  `ee93206c0` (owner transactions), `2093475e8` received at `d3f742ac3` (native
  owner bridge).
- QA: [supervised-qualification-20260913.md](supervised-qualification-20260913.md),
  [full-lifecycle-rehearsal-20260913.md](full-lifecycle-rehearsal-20260913.md),
  [native-owner-receiving-20260913.md](native-owner-receiving-20260913.md),
  [bootstrap-deliverables-audit-20260916.md](bootstrap-deliverables-audit-20260916.md).
- Ledger: Done "Supervised fresh-Fable receive/failure/reverification loop", "Owner
  transitions ca0474fe4", "Native owner bridge2093475e8". Remaining "Complete all
  five bootstrap deliverables" (audit: 1–3 satisfied).

### 3b. `task_512bafc249df308083f7acacbc22382e`: Restore the Receipted Slack Alert Path and Send One Owner Alert

- Request: `req_6952457b7a776f017f022f3eefdb28edeff35f89bd42481c1182c99816d9735f`.
  Reward shown: 3.5.
- Scope: bootstrap deliverable 4. Not claimed: the reply half of the round trip.
- Commits: `730577f1d` received at `9877c058d` (offline Slack delivery/replies),
  `92d6eb261` (owned listener quiescence), `08db99fff` (records a listener
  child's startup death; slack-listener-38). **Correction, 2026-10-03:** earlier
  versions of this map called `08db99fff` the "listener interpreter fix". It
  contains no interpreter change. The slack_sdk failure was resolved operationally
  by running the listener under the pinned-requirements venv interpreter, as
  [slack-requalified-20260917.md](slack-requalified-20260917.md) records.
- QA: [slack-requalified-20260917.md](slack-requalified-20260917.md); alert
  `owner-recurrence-domain-20260917`, `state: sent`.
- Ledger: Done "Exact reviewed supervisor c7a1b9690 received92d6eb261", "Offline
  recovery730577f1d". Remaining "Complete all five…" item **(4) CLOSED 2026-09-17**.

### 3c. `task_cf2cc91ff71a095a3baa93df1c3d91f8`: Install and Arm the Corbanu Owner Daemon Recurrence

- Request: `req_fdfa8a1dde13e235ad6695059fcf7badec53d3302e4ebc1360240afc40f0cc62`.
  Reward shown: 4.
- Scope: increment D, its functional qualification, the launchd guard, install and
  arming (generation 1, fixture-only). Not claimed: arming tmux-workers or enabling
  the manager.
- Commits: `aba3707b0` (increment D), `e04df14fb` (restart regression pin),
  `c782a340f` and `6f3c2d705` (qualification fixes), `93f60eee8` (launchd guard).
- QA: [owner-daemon.md](owner-daemon.md),
  [owner-live-activation-20260917.md](owner-live-activation-20260917.md).
- Ledger: Done "Owner-daemon **increment D…", "**Retraction…", "**Functional
  qualification RAN…", "**Four of the five defects fixed…", "**Owner-daemon
  increment D functional qualification is COMPLETE…", "**Recurrence is LIVE,
  2026-09-17…".

### 3d. `task_80628c7accf770686bdd04fc4cb78bb5`: Ship PF-80-S01 Live Progress Event With Idempotent Re-Send

- Request: `req_def849619e606fdb1c76d1b1aca64432d579b5b21fc58ac8a0203d1368052eeb`.
  Reward shown: 4.
- Scope: the writeback pipeline, the vault-backed sender, enrollment, and one live
  event with a duplicate re-send. **Done 2026-10-03:** event
  `cc-57beba1c9d8a83e20fef572689d9a6d9d705c6daedb09583e0ce5dcf0152cc13` delivered
  once, same-key re-send replayed with no network write, server read-back shows one
  item at revision 1. `control.json` maps `PF-80-S01` to this task.
- Commits: `2b281975a` received at `c33d47f6c` (native preparation), `7e25982b5`
  received at `486d2fb94` (one-event engine), `d7bf73a52` received at `56295f668`
  (adapter), `177ec93fc` received at `3898eaa65` (reconciliation), `bea83c9c6` (CLI
  fences), `ec88be0a06` and `b0222784b9` (vault-backed send/enroll, decision A2).
- QA: [pf-80-s01/native-preparation/handoff.md](../pf-80-s01/native-preparation/handoff.md),
  [pf-80-s01/native-one-event/receipt.md](../pf-80-s01/native-one-event/receipt.md),
  [pf-80-s01/native-adapter/receipt.md](../pf-80-s01/native-adapter/receipt.md),
  [pf-80-s01/native-reconciliation/receipt.md](../pf-80-s01/native-reconciliation/receipt.md),
  [pf-80-s01/cli-fences-20260916.md](../pf-80-s01/cli-fences-20260916.md),
  [pf-80-s01/pf76-discrimination-receipt.md](../pf-80-s01/pf76-discrimination-receipt.md),
  [tasknode-writeback-live-20261002.md](tasknode-writeback-live-20261002.md)
  (vault sender, review and enrollment receipt).
- Ledger: Done "Pure native immutable-goal preparation", "Bound one-event engine",
  "Native adapter", "Exact-goal reconciliation", "**Historical PF-76 reconciliation
  closed…", "**CLI recovery, input, retention and restart fences…", "**Owner
  writeback decisions recorded…". Remaining "Under Travis's September12
  progress-writeback authorization…".

### 3e. `task_6d472c45a76bcd6cb5e554eda863a9a1`: Register First Owner-Bridge Allocation and Record QA Evidence

- Request: `req_94b6270ffb2e714bb54ef6811df34d309c9deb8f34e45141d2a70109f8825946`.
  Reward shown: 4.
- Scope: `prepare_worker` and registering `owner-first-check-01`. Not claimed: the
  manager cycle, the isolated worker test or promotion.
- Commit: `66a5b66840`.
- QA: [owner-manager-enabled-20261002.md](owner-manager-enabled-20261002.md), "Item 1:
  PASS" (886 OK discovery, 103 OK owner QA, revision 2911 → 2912, audit seq 3413).
- Ledger: none yet. The manager records the owner-enable rounds outside the
  PF-80-S01 Done list.

## Network task (not PF-80)

### `task_f4bec7ef5c4454dacbe0f7494ed35f16`: Find and Fix One Reproducible Bug in PostfiatL1V2

- Reward shown: 100. Travis accepted it 2026-10-03 (Accepted at 10:32Z).
- Work: `account_tx` reported `truncated: true` when results exactly filled the
  limit (scan and index paths, `crates/node/src/block_finality.rs`).
- PR: https://github.com/postfiatorg/postfiatl1v2/pull/55 (IridiumMaster fork,
  branch `fix/account-tx-exact-limit-truncation`, commit `b7a54b95`, base
  `ce25aaa8`). Ready for review 2026-10-03 14:20Z. Regression test fails before,
  passes after; full `postfiat-node` suite passes on Linux.
- Merged 2026-10-04T09:37:09Z by goodalexander (approved after re-running the
  regression test), merge commit `b1d1928c091ed66836ce69d49cf8bc192a969281`.
- Evidence: submitted once at 2026-10-04 ~09:40Z (PR URL + summary), receipt
  `task_evt_3ecb66ac-78f1-4cf1-a0b9-2f56182d98c0`. Body:
  `.codex-work/workers-20261002/tasknode-f4bec7e-evidence.md`.
- Worker record: `.codex-work/workers-20261002/l1v2-bug.result.md`.

## Rules for evidence against these tasks

1. Submit evidence only after Task Node shows the task as accepted and
   `actions.canSubmitInitialEvidence` is true. Corbanu never accepts tasks itself.
2. Cite only the commits and records listed for that task, plus anything added to
   this file later with its own commit.
3. A Campaign Tracker progress event cites a task ID through the `control.json`
   mapping. Repoint `PF-80-S01` from the Rewarded parent to the accepted task first.
