# Task Node live writeback, 2026-10-02/03: enrolled, first live event delivered once

Status (round 3, 2026-10-03): **one live event delivered; the same-key re-send
made no second write.** Travis accepted the five PF-80 tasks. `PF-80-S01` now maps
to `task_80628c7…`, and event `cc-57beba1c…52cc13` reached the server once
(revision 1, one item for the account). Evidence submissions are recorded in the
round 3 section below. No task stage was moved and no reward was claimed. Batch
posting, TUI recording and recurring writeback stay **OFF**. Round 1 (2026-10-02)
stopped before any write; round 2 enrolled the workspace and requested the tasks.

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

### Follow-up decisions (Travis, 2026-10-03, verbatim)

- A1: "We don't need to submit anything against a task that has been recognized as closed. If we erred in giving that task too much scope, we should create some new tasks that reflect the actual breakdown of our work and submit tasks and evidence against those. We should have a way of mapping this back to our actual work."
- A2: A reviewed CLI command that reads Travis's Task Node session from the vault at send time; nothing is written to disk.
- A3: "Our wallet address is 5oUcE2vkXK7ffn7pxJXCawq8UYD62hoadLLr6zGDRDtC, I've put the private key in the vault corresponding to CorbanuAPIKey. … Alternately, just copy the credentials from the RTX box."
- A4: Enroll `corbanu-initiative-control` through the delivery-control enroll command. No TUI recording.

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

## Round 2, 2026-10-03: decisions A1–A4 carried out

Worker: Task Node integration worker, Claude Code, round 2. Times 2026-10-03
09:20–10:10 UTC. All Task Node and vault reads used the real binary
`.codex-work/integration-build-20260922a/integration-package-dev/bin/corbanu`
(0.1.48) with `CORBANU_HOME`/`CODEX_HOME` set explicitly.

### A finding about scope probes

`~/.local/bin/corbanu` is a shell wrapper that sources the terminal's
`activate.sh`, which re-exports `CORBANU_HOME`/`CODEX_HOME` to the terminal work
home. Any probe made through it reads the terminal home, whatever `CODEX_HOME`
the caller sets. My first round-2 probes through the wrapper reported "no
session" in `~/.corbanu` for that reason. With the real binary, `~/.corbanu`
holds `tasknode/session` (212 bytes, the IridiumMaster session) and no lifecycle
marker, which means a legacy link. The new sender refuses wrapper scripts for this reason.

### Vault metadata (labels only, read through the TUI `/vault` → View credentials list)

| Home | Labels |
| --- | --- |
| `~/.corbanu` | `provider/claude-code-oauth-token`, `tasknode/session` |
| terminal home `.codex-work/corbanu-terminal/home` | `CorbanuAPI` (manual secret), `RTX-machine`, `neo-vm`, `provider/ai_gateway_api_key`, `provider/ambient_api_key`, `provider/claude-code-oauth-token`, `provider/deepseek_api_key`, `provider/openrouter_api_key`, **`provider/pfterminal_plan_api_key`**, `provider/zai_api_key` |
| RTX `~/corbanu-rtx/home` | `provider/ai_gateway_api_key`, `provider/claude-code-oauth-token`, `provider/deepseek_api_key`, `provider/openrouter_api_key`, `provider/zai_api_key` |

No vault holds a label `CorbanuAPIKey`. The terminal home's `CorbanuAPI` entry
matches Travis's description of the wallet private key. **It was not read.** It is
stored as a *manual secret*, and `vault auth-helper` refuses only the
private-key, seed-phrase and keystore types. That means any agent can read this
entry with the helper. **Recommendation:** re-store it as type "crypto private
key".

### A1: task breakdown

Five requests were created once each with `tasknode request create --body-file`.
Task Node generated five **Proposed** tasks, and nothing was accepted or refused.
The full mapping to commits, QA records and ledger entries is in
[tasknode-task-map.md](tasknode-task-map.md).

| Task | Title (as generated) | Request |
| --- | --- | --- |
| `task_9d4fc2d8b21c3fd3ab2bc5ce6b18a188` | Record PF-80-S01 Bootstrap Deliverables With Commit Evidence | `req_6551ea32…` |
| `task_512bafc249df308083f7acacbc22382e` | Restore the Receipted Slack Alert Path and Send One Owner Alert | `req_6952457b…` |
| `task_cf2cc91ff71a095a3baa93df1c3d91f8` | Install and Arm the Corbanu Owner Daemon Recurrence | `req_fdfa8a1d…` |
| `task_80628c7accf770686bdd04fc4cb78bb5` | Ship PF-80-S01 Live Progress Event With Idempotent Re-Send | `req_def84961…` |
| `task_6d472c45a76bcd6cb5e554eda863a9a1` | Register First Owner-Bridge Allocation and Record QA Evidence | `req_94b6270f…` |

Account counts afterwards: outstanding 8, verification 0, refused 2, rewarded 3.

### A2: vault-backed send and enroll

- Commits `ec88be0a06` (feature) and `b0222784b9` (review fixes) on
  `integrate/management-workstreams-20260911`.
- `tasknode.py send --live` and `enroll --confirm-live` accept `--corbanu-bin`,
  `--vault-session-home`, optional `--vault-api-key-home` and `--tasknode-profile`.
  After every local gate passes, the sender runs `corbanu vault auth-helper` with
  every home alias pinned to the given home. It reads the lifecycle marker
  (refused unless absent or `"linked"`, as in Rust), then the session record
  (origin and expiry checked), then `provider/pfterminal_plan_api_key`. Values stay
  in memory and are never logged, echoed or written. Replays and dry runs never
  read the vault, and a vault failure leaves no durable intent behind.
- Independent review: `corbanu exec -m claude-opus-5-5-plan`, prompt plus diff only.
  - Pass 1 on `ec88be0a06`: two P2s (inherited debug-home aliases could redirect
    a `*-debug` binary; the unlink marker was ignored) and two P3s (enroll read the
    vault before its local checks; missing tests). All four were fixed in
    `b0222784b9`, and both P2 fixes were confirmed by mutation.
  - Pass 2 on `b0222784b9`: **VERDICT: CLEAN**.
  - Artifacts: `.codex-work/tasknode-round2-review/`.
- Tests, synthetic values only:
  - `test_tasknode` 43/43 OK.
  - `test_control` 41/41 OK.
  - Full discovery in a clean worktree at `b0222784b9` (fresh venv from
    `requirements.txt`, disposable HOME, keyring disabled): 897 tests, 866 pass,
    30 failures and 1 error, all in `test_owner_daemon` worker-lifecycle/handoff
    cases. The identical 31-case failure set reproduces at the untouched base
    `4da42a9b10` in the same environment (133 tests, same 31), so it is
    pre-existing and environmental, not caused by this change. It is retained,
    not counted as a pass. Outputs: `.codex-work/tasknode-round2-suite2.txt` and
    `tasknode-round2-base-owner.txt`.
- **Disclosed incident:** before `enroll`'s transport was late-bound, one new CLI
  test's patch missed the default `post`. That test reached the production
  enrollment endpoint once with the synthetic values `synthetic-terminal-token-…`
  and `synthetic-plan-key-…` and was rejected with HTTP 401. No real credential
  was involved. The fix is in `ec88be0a06`, and later runs show no network access.

### A3: Corbanu API key

The RTX copy was not needed, and was not possible: the RTX vault has no Corbanu
key (labels above; `provider/pfterminal_plan_api_key` and the other candidates
exit 1, not found). This Mac's terminal home already holds
`provider/pfterminal_plan_api_key`: the helper exits 0 with a 47-character value.
The sender reads it from that home (`--vault-api-key-home`) instead of copying it,
so no second copy exists. The server accepted it during enrollment, below. The
wallet private key was not touched.

### A4: enrollment

Command (clean worktree at `b0222784b9`, venv Python):
`tasknode.py enroll --state <live state> --confirm-live --corbanu-bin <real binary>
--vault-session-home ~/.corbanu --vault-api-key-home <terminal home>`.

- Run 2026-10-03T09:58:13Z, exit 0: "Enrollment verified; live event delivery still
  requires enabled: true".
- New `state/enrollment.json` (0600, 117 bytes): `workspace_id
  corbanu-initiative-control`, `verified true`, `verified_at
  2026-10-03T09:58:15+00:00`.
- `control.json` is unchanged (SHA-256 `0ee45fb7…6ce1` before and after) with
  `tasknode.enabled=false`. No outbox or receipts exist.

### Step 5: live send, not run

At 09:58Z all five new tasks are **Proposed** (`canAccept` true,
`canSubmitInitialEvidence` false), so no new task is assigned. Per the round-2
instruction I stopped after A1–A4. No event was enqueued or sent, so no send or
re-send receipt exists.

To send the first event:

1. Travis accepts one task in Task Node, most naturally `task_80628c7…`. Corbanu
   does not accept tasks.
2. The manager repoints `control.json` `task_mappings["PF-80-S01"]` from the
   Rewarded `task_789a0f3…` to that task.
3. Enqueue one report, then write an owner activation for that exact event.
4. Run `send --live` with the vault options, then re-send with the same event ID
   to prove the replay.

Still OFF: batch posting, TUI recording, recurring writeback, task stage moves,
reward claims.

## Round 3, 2026-10-03: tasks accepted, mapping repointed, first live event sent

Worker: Task Node integration worker, Claude Code, round 3. Times 2026-10-03
10:30–11:00 UTC. Travis, 2026-10-03: "I have accepted the tasks." Reads used the
real binary (0.1.48) with `CORBANU_HOME`/`CODEX_HOME` = `~/.corbanu`, profile `null`.
`tasknode status` confirmed account `acct_oauth_73e1ab2f9f7cd01d3ee6230e`, GitHub
`IridiumMaster`; counts outstanding 8, verification 0, refused 2, rewarded 3.

### Task state, read-only, 10:32Z

All five tasks read **Accepted** (`statusKey accepted`, `canSubmitInitialEvidence`
true, `canSubmitVerificationEvidence` false). Corbanu did not accept, refuse,
cancel or move any of them.

### Mapping repoint (reviewed edit)

No CLI command edits `task_mappings`; earlier repoints (PF-76-S01, 2026-09-16)
were direct edits of the live `control.json`. This one is a single-line byte
replacement, checked by parsing before and after:

- Backup: `state/control.json.bak-20261003` (byte copy of the previous file).
- `PF-80-S01`: `task_789a0f3bd75b41d1eca20cae698f04cf` (Rewarded) →
  `task_80628c7accf770686bdd04fc4cb78bb5` (Accepted). Every other key and byte
  is unchanged (`diff` shows one line).
- SHA-256 `0ee45fb7…6ce1` → `7c10ef45d52869149155deb10788fd994737113f3b0449f5559182a12859784a`.
- `tasknode.enabled` stays `false`. `PF-79-S01`/`PF-79-S02` still point at the
  Rewarded parent; PF-79 beta work is paused, so they were left alone.
- No timer runs `tick.py` on this Mac, so the new mapping does not enqueue
  anything by itself.

### Live event and same-key re-send

Source: `scripts/initiative_control` exported from `577526daab` (`git archive`),
`test_tasknode` 43/43 OK on that export. Python: the delivery-control venv (3.14.4).

1. `enqueue` of one run report (`run_id tasknode-round3-20261003`, status
   `working`, commit `577526daab36f31055ba3cd678dc4cc503d22d62`) → event
   `cc-57beba1c9d8a83e20fef572689d9a6d9d705c6daedb09583e0ce5dcf0152cc13`,
   `taskIds ["task_80628c7accf770686bdd04fc4cb78bb5"]`, workspace
   `corbanu-initiative-control`.
2. `send --dry-run`: blockers only `posting_disabled` and
   `live_authority_entitlement_owner_and_target_lifecycle_unverified` (the two
   the live sender allows), local enrollment true, request digest
   `fffede1f1ae07f0d2a29c23484eeff64e2e7da754398b3c82c47b1fbb37682ce`.
3. Owner activation (0600, private, not committed): owner
   `travis-decision-tasknode-writeback-20261002`, bound to that event ID, digest,
   workspace, task and origin, 30-minute expiry. Gates set from evidence: identity
   (status read above), entitlement and enrollment (server-verified enrollment,
   round 2), target lifecycle (task Accepted with evidence actions), payload review
   (content checked against the ledger: no secrets, paths, URLs or completion claim).
4. `send --live` with `--corbanu-bin <real binary> --vault-session-home ~/.corbanu
   --vault-api-key-home <terminal home>`, 10:40:52Z, exit 0:
   `http_status 200`, `outcome delivered`, `network_writes true`, response
   `ok true`, `event_id_matches true`, `deleted false`,
   `idempotency_key` = the event ID.
5. Same command again, same event ID and key, exit 0: the stored receipt came
   back with `replayed true`, `network_writes false`, same `created_at`
   `2026-10-03T10:40:52+00:00`. No vault read and no POST happen on this path.
6. Server read-back, `GET …/campaign-tracker/activity` (session from the same
   reviewed vault reader, held in memory, nothing printed): by event ID **1 item**;
   whole account **1 item**, the same ID, `revision 1`, `receivedAt
   2026-10-03T10:40:52.246Z`, `handleAtExecution iridiumeagle`, `taskIds
   [task_80628c7…]`. **No duplicate exists on the server.**

Receipts, `state/send-receipts/` (0600, retained, not committed):
`<id>.intent.json` SHA-256 `05ed028be4a7a83ef75bc627a9a3a0dba44fb5ab4527bc156ea9f412876311f1`,
`<id>.result.json` SHA-256 `00fad5d6b4f019a9cf5f2608a1c3b7c409bba69d28e928bc48fd253864eef6d3`.
`tasknode.py status`: the one outbox record reads `delivered`, 0 attempts, no error.
Worker copies of the report, dry run, both send outputs and the read-back are in
`.codex-work/tasknode-round3/`.

Still OFF: batch posting (`enabled=false`), TUI recording, recurring writeback,
task stage moves, reward claims.

### Initial evidence, one submission per task

Command: `corbanu tasknode task evidence <task> --body-file <body> --json` (real
binary, `~/.corbanu` scope; the helper reads the session from its own vault).
A one-shot wrapper read `task show` first, required `canSubmitInitialEvidence`
true and no local receipt, and recorded a fixed local submission key
(SHA-256 of task ID plus body). **Disclosure:** the installed helper sets its own
per-call `idempotencyKey` (`pfterminal-cli:initial_submission:<pid>:<nanos>`) and
has no option to fix it. Duplicate protection for evidence is therefore the
local one-shot guard plus the helper's server-state preflight, tested below. The
fixed server idempotency key applies to the Campaign Tracker event.

| Task | Submitted (UTC) | Receipt (`offchainLifecycle.eventId`) | Local key (first 16) | Scope |
| --- | --- | --- | --- | --- |
| `task_80628c7…` | 10:46:41 | `task_evt_24bfb396-384e-4d89-959e-7859868a0703` | `eba565edf907b658` | live event, replay, read-back, component commits and test counts |
| `task_9d4fc2d8…` | 10:46:51 | `task_evt_aa2a03ad-6fea-4e66-99b6-f3b655986040` | `06d8293f764ab6e1` | deliverables 1–3 record, ancestry, verbatim counts; 4, 5, writeback, beta excluded |
| `task_cf2cc91f…` | 10:46:54 | `task_evt_95dd8c2e-1ba3-4b77-a717-bbd0cd4dc825` | `70147484a14e4f78` | install and arm, 13/13 and 39/39, fixture-only; manager not claimed |
| `task_6d472c45…` | 10:47:01 | `task_evt_1dbdaac9-3d60-4c25-bbe4-ecde5d12050c` | `619c5517811a6291` | `66a5b66840`, ordering, 886/103, seq 3413; manager cycle, worker test, promotion excluded |
| `task_512bafc2…` | 10:46:48 | `task_evt_739efbcc-04d4-4bd2-9f42-b86dc266534a` | `eaa4c84c75c94e0a` | gate, alert receipt, commits; **reply half stated as NOT PROVEN** |

Every receipt: `ok true`, `phase submitted`, `pfterminalLifecycle.phase
awaiting_verification`, `completionConfirmed false`. A later `task show` read
**Verification requested** for all five (`canSubmitInitialEvidence` false,
`canSubmitVerificationEvidence` true, no `rewardOutcome`).

Duplicate checks, after each task read `canSubmitInitialEvidence` false:
re-running the wrapper refused locally (exit 3, no network). Re-running the
helper's `task evidence` with the same body was refused before any POST
(`task_evidence_mode_mismatch`, exit 1) for all five.

Leak scan: an in-memory comparison of 18 files (live `control.json`, outbox,
index, enrollment, both send receipts, every worker output) found 0 containing
the session token or the plan key.

### Correction: `08db99fff` is not an interpreter fix

The verifier for `task_512bafc2…` asked for the pinned-interpreter hunk in
`08db99fff`. There is none. That merge (of `c28238d87d`, `061cb26034`) makes a
listener child's startup death leave a durable child-exit record. The listener
still spawns `[sys.executable, …]` (`decision_manager.py` line 482 at the tip).
The slack_sdk failure was resolved operationally by running the listener under
the pinned-requirements venv, per
[slack-requalified-20260917.md](slack-requalified-20260917.md); the
`slack-alert.sh` recipe it names is not in the repository. The round 2 task
request, the task map and the live event text all carried the wrong label. The
map is corrected; the immutable event (`cc-57beba1c…`) is not edited, and this
paragraph is its correction.
