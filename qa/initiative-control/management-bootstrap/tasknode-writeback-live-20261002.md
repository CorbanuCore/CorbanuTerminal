# Task Node live writeback, 2026-10-02: identity confirmed, send stopped before any write

Status: **stopped before enrollment or send.** No Task Node write happened. No task
stage changed, nothing was accepted, and no reward was claimed. Batch posting and
recurring writeback stay **OFF**.

- Class: product initiative, PF-80-S01 (`in_progress`), active plan
  `docs/plans/active/initiative-delivery-control.md`.
- Product citation: **Internal delivery control — TO BUILD**: “Task Node receives
  only explicitly mapped, supported progress; no automatic reward, signing,
  financial action or task-completion claim is implied.”
- Worker: Task Node integration worker, Claude Code. Build: installed
  `~/.local/bin/corbanu` → integration package `0.1.48`, commit `704650ccd`
  (`INSTALL.json`, binary SHA-256 `7b8c77a6…dd77c`).
- Times: 2026-10-03 05:2x–05:37 UTC (2026-10-02 evening, America/Phoenix).

## Decision `tasknode-writeback-20261002` (Travis, 2026-10-02, verbatim)

1. Account and destination: "IridiumEagle, use my existing login."
2. Enrollment: "Yes, go ahead and enroll it."
3. Lifecycle: "The Task Node itself moves task node tasks through their stages. Our
   job is to submit evidence of completion."
4. First live event: "Yes, of course."

Manager instructions that came with the decision: never create a new credential;
never print, copy or log the token; send one real PF-80 progress/evidence event
with a fixed idempotency key, then qualify an idempotent re-send; batch posting
stays OFF; never turn on recurring writeback; stop and report if Travis is needed.

## What was observed (read-only)

### Login scope

| Probe | Scope | Result |
| --- | --- | --- |
| `tasknode link status` | inherited session: `CODEX_HOME=…/corbanu-terminal/home`, profile `null` | no active session |
| `tasknode status` | same | `Task Node is not linked in this Corbanu profile` |
| `tasknode link status` | aliases unset, no profile | refused: profile not supplied (correct fail-closed) |
| `tasknode link status` | `CODEX_HOME=~/.corbanu`, profile `null` | active session, origin `https://tasknode.postfiat.org`, account `acct_oauth_73e1ab2f9f7cd01d3ee6230e`, GitHub `IridiumMaster`, no expiry, no pending link |

The existing login lives in the `~/.corbanu` default profile, not in the
terminal's work home. That is the login used for the
[September 14 acceptance](tasknode-acceptance-20260914.md) ("home variables unset"):
`~/.corbanu/tasknode/session-locks/default` was created at 22:49 UTC that day,
the minute of that receipt's status read. I used that scope only because Travis
named his existing login, and only for reads. The server then confirmed the
account, so this scope was not a guess. It is still a deviation from the
tasknode-usage skill's inherit-only rule, recorded here so the manager can decide
whether future runs should start from a terminal that already carries that home.

### Account

- `tasknode status`: `ok`, GitHub `IridiumMaster` linked, `terminalBridgeEligible`
  true, wallet linked, `signingRequiredForActions=false`, server
  `offchainTaskLifecycle` and `terminalTaskActions` true. Counts: outstanding 3,
  verification 0, refused 2, rewarded 3.
- Handle: the native TUI Campaign Tracker view (real PTY, keys sent:
  `/tasknode`, Enter, Down, Enter) showed
  `@iridiumeagle · 0 events · 0 pending · 0 capture gaps · recording OFF`.
  **Account confirmed as IridiumEagle.** I exited with Esc/Ctrl-C and selected
  nothing else.

### Tasks

| Task | Title | State |
| --- | --- | --- |
| `task_789a0f3bd75b41d1eca20cae698f04cf` | Corbanu workstream 3: Task Node integration and beta-test coordination (PF-80-S01 mapping) | **Rewarded** (terminal; `rewardOutcome` present; last event 2026-09-15 00:39 UTC) |
| `task_865f75c6911f953c6586cdc1f4531e4f` | Corbanu workstream 1: PF-13 security | Rewarded |
| `task_b3e8506327fb906173fd68b2f642221b` | Corbanu workstream 2: accounting | Rewarded |
| `task_f4bec7e…` | Find and Fix One Reproducible Bug in PostfiatL1V2 | Proposed |
| `task_05099d2…` | AI newsletter section | Accepted |
| `task_96e88f7…` | Private inference user flow spec for PF Terminal | Accepted |
| `task_a369683…`, `task_6c67422…` | (unrelated fixes) | Refused |

`task show` for the PF-80 task reports every action false, including
`canSubmitInitialEvidence` and `canSubmitVerificationEvidence`. **No open PF-80
task exists.** None of the outstanding tasks concerns PF-80.

### Workspace and enrollment (before = after)

- Campaign Tracker server view: 0 events ever recorded for this account. Recording
  is OFF for the TUI workspace of this worktree. The TUI keys that workspace by
  SHA-256 of its path, `6c8b10df…f969a`.
- Delivery-control workspace `corbanu-initiative-control` (live
  `control.json`): `tasknode.enabled=false`; no `enrollment.json`, no `outbox/`, no
  `send-receipts/`. Mapping `PF-80-S01 → task_789a0f3bd75b41d1eca20cae698f04cf`.
- After: unchanged. I did not enroll. Two empty lock files were created by the
  reads: `…/corbanu-terminal/home/tasknode/session-locks/default` and
  `~/.corbanu/campaign-tracker/<scope-hash>/outbox.lock`. No outbox, no tracker key
  and no vault entry were written.

Raw read outputs stay in a private temporary directory and are not committed,
because they contain full task projections. SHA-256: status
`38afa9eb…ba780b`, outstanding `9b31b5fd…52ae0`, rewarded `6d58fe65…afb`, refused
`d75e26ab…5bd1569`, verification `55f39f44…154a1a`, PF-80 `task show`
`c9f75b31…4308`, tracker screen `a3406b8a…b4ac35f`.

## Why nothing was sent

Each of these blocks the send by itself. I did not work around any of them.

1. **The target task is closed.** Decision 3 limits Corbanu to evidence "for tasks
   the Task Node already assigned". The only PF-80 task finished its lifecycle
   (Rewarded) on 2026-09-15, and the server rejects evidence for it. If I attached
   a new event to a settled task, it could be read as reopening or re-claiming
   that task. The sender's owner activation also requires a `target_lifecycle`
   attestation, which I cannot honestly give.
2. **No supported sender can do this without copying the token.** The installed
   CLI (`704650ccd`) has no Campaign Tracker, enrollment or event command. The
   native one-event sender (`tasknode-session` `delivery_send`) has no caller. The
   only implemented designed-event sender, `scripts/initiative_control/tasknode.py
   send --live`, and its `enroll` both need a 0600 credentials file holding the
   terminal session token. Writing that file means copying the token, which the
   decision forbids.
3. **Enrollment would turn on recurring writeback.** The native path, Campaign
   Tracker → "Enable recording for this workspace", records the workspace's prompts
   and summarized outputs and syncs them every 30 seconds. That is the recurring
   writeback the decision keeps OFF, and it would also enroll a different
   workspace from the delivery-control one.
4. **The entitlement credential is unverified.** Enrollment and events also need
   the Corbanu subscription API key, `apiKey`. It is not in the environment, and
   checking auth storage would mean reading the Keychain, which a worker must not
   do.

## Draft payload (not sent, not frozen, no event ID)

This text is prepared for review only. No event ID exists because no outbox record
was created.

```text
PF-80-S01 progress observation. Received on integrate/management-workstreams-20260911: owner-daemon increment D functional qualification, 13 of 13 cases, candidate 6f3c2d705; CLI recovery, retention and restart fences, 709 of 709 focused tests, bea83c9c6; launchd recurrence guard 93f60eee8, owner live fixture-only since 2026-09-17; Slack listener interpreter fix 08db99fff. Still open: live progress writeback and recovery qualification, independent final-tree review, named-human acceptance. Progress observation only; no acceptance or task-completion claim.
```

561 bytes; SHA-256 `220bb1ba01ad300ec272fb8b0f56808a2ccd5f9ae08c7432437a84b34fb0087f`.
Every claim cites the PF-80-S01 Done ledger. It contains no secrets, local paths,
URLs or completion claims.

## Receipts and recovery

- Send receipt: **none**. No live event was sent.
- Duplicate re-send and read-back by event ID: **not run**, because there is no
  event to re-send. Nothing was deleted.

## What Travis must decide, and what stays OFF

To unblock one live event, Travis has to choose one option for each of these:

1. **Target:** (a) Task Node assigns a new PF-80 task, which Corbanu does not
   accept itself; or (b) Travis states that progress events may cite the Rewarded
   task `task_789a0f3…` as informational activity, not evidence for reward.
2. **Sender without copying the token:** approve a reviewed native CLI command
   that reads the session from the vault at use time, for example a
   `tasknode tracker send/read` that wraps `delivery_send`. Or explicitly allow a
   short-lived 0600 credentials file that is deleted right after the send.
3. **Entitlement:** confirm that a Corbanu subscription API key exists for this
   account. If it does, provision it through `/vault`. Workers will not read the
   Keychain to find it.
4. **Enrollment target:** `corbanu-initiative-control` through the delivery-control
   `enroll` command, not TUI recording, so prompts are not recorded automatically.

Still OFF: batch posting (`tasknode.enabled=false`), TUI Campaign Tracker
recording, recurring writeback, enrollment, task acceptance and stage moves,
reward claims and PF-79 beta work.
