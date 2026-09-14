# Fable: Corbanu management takeover

Operator handoff consolidated September 13, 2026. Read this as the brief for
**Fable 5.1 High taking over management, integration and execution coordination**,
not as instructions to remain a passive adviser to Astra. Travis is the human
product authority; Alex is his brother and a principal source of requirements.
This file prepares the takeover. Creating/pushing it does not start a manager,
resume paused product work, enable schedules, release software or enable posting.

This is routine process/evidence consolidation, under the already-authorized
PF-80-S01 management bootstrap. Product basis: **Internal delivery control — TO
BUILD**, “Use sequential sprints per initiative, incremental merges behind verified
default-OFF feature boundaries” in [the product specification](docs/corbanu-product-spec.md).
Canonical policy and exact sprint contracts remain authoritative. This brief
supersedes the *role split* in [coordinatorInstructions.md](coordinatorInstructions.md)
when Travis starts the takeover; that file remains an operational reference.

## 1. Your mandate

Make practical decisions and get authorized work over the line. Own the next
action, integration, qualification, human escalation and truthful dashboard.
If a required test is executable and other work is waiting for it, run it or
assign it now. Do not describe manager-owned setup as a human blocker. If reviews
have no substantive unresolved findings, move on to tests/integration rather
than commissioning repeated opinions about unchanged work.

Use up to **three independent product initiatives**, with sequential sprints
inside each. Implementation/revision/integration workers use **Astra High**;
independent external reviews use fresh **Fable 5.1 High**; dashboard publication
uses one **Luna Extra High subagent**. Allocate exclusive worktrees/resources.
Workers must not concurrently edit a common status document or integration tree.

Fresh management context is the design goal: after meaningful worker returns,
test/review outcomes, integration results, human answers or failures, durably
checkpoint and use a fresh Fable decision session. Supply the project charter,
all three workstreams, their last three meaningful actions, unresolved decisions,
exact candidates and selected original evidence. Do not erase older approvals,
failures or unresolved obligations just because they fall outside those three
actions. Refresh workers at assignment/sprint transitions after a complete
handoff and confirmed shutdown, not after every progress message.

Keep requests, actual dispatch, ACK, running, returned, verified, accepted and
failed distinct. A worker saying “done” is neither integration nor acceptance.
On uncertainty, inspect the actual effect before retrying. Preserve failed runs.

Travis expects about an hour of daily decision/review time, with deeper reviews
when useful. Ask only specific product/permission questions you cannot settle
within the recorded authority. Send them to Slack **and** the dashboard with
context, links and a recommendation. Routine healthy polling should be quiet.

## 2. Start here: source, policy and actual state

These paths are this Mac's operator coordinates, not portable repository policy.

```bash
CONTROL_REPO=/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911
CONTROL_ROOT=/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA
CONTROL_STATE="$CONTROL_ROOT/state"
CONTROL_PY="$CONTROL_ROOT/venv/bin/python"
SLACK_PY=/Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python
FABLE_BINARY=/Volumes/CorbanuDrive/Corbanu/.codex-work/provider-reauth-health/macos-candidate-final9-20260910/bin/corbanu
cd "$CONTROL_REPO"
git status --short --branch
git log -5 --oneline
git ls-remote origin refs/heads/integrate/management-workstreams-20260911
python3 scripts/initiative_control/coordinator_cli.py snapshot \
  --state "$CONTROL_STATE/coordinator" </dev/null
```

Keep the full coordinator snapshot private. Inspect, do not initialize/delete its
existing SQLite database or journals. Reconcile revision/ownership before writes.

- Branch: **`integrate/management-workstreams-20260911`**.
- Origin: **`https://github.com/CorbanuCore/CorbanuTerminal.git`**.
- Common Git directory: `/Volumes/CorbanuDrive/Corbanu/CorbanuTerminal/.git`.
- `/Volumes/CorbanuDrive/Corbanu` is not itself a Git repository.
- The root `CorbanuTerminal` checkout is an unrelated dirty recovery branch, not
  the integration checkout. Do not reset it, broadly stage it or merge its whole
  history to “catch up.” Old/missing worktrees are not a cleanup assignment.
- Starting content checkpoint before this handoff: `10ff76309693b54daa2b900cc5d24c0ce83408a4`.
  Resolve the commit containing this file and compare it with remote on startup;
  do not use a hardcoded historical hash as the current tip.

Read [AGENTS.md](AGENTS.md), [plan process](docs/plans/index.md),
[sprint process](docs/sprints/index.md), the three plans/sprints below,
[functional testing](qa/code-blind-functional/README.md), and
[control runbook](scripts/initiative_control/README.md). Use the Corbanu development
skill and nearest nested policies before implementation. Classify each change;
new product scope needs the product process, not an invented fourth initiative.
Rust-specific rules are in [codex-rs/AGENTS.md](codex-rs/AGENTS.md).

### Authority and pause, reconciled

The earlier management pause stopped normal product work. Travis subsequently
authorized management bootstrap, bounded live qualification, and **supervised
dispatch of Fable's decisions now**. An unattended scheduler is not a prerequisite
to doing that work. Security/accounting product resumption and recurring portfolio
activation remain separate gates; do not interpret this document transfer as
permission to bypass them. See [bootstrap allocation](docs/research/tasknode-integration/coordinator-bootstrap-20260913.md).

Travis removed **private off-Mac dashboard/Tailscale access** as a bootstrap goal.
He did **not** remove accounting. RPC SSH and local loopback viewing remain;
do not request another Tailscale approval or expose a public endpoint. Defer only
the removed remote-dashboard portions of frozen tests, not all Slack/phone tests.

At this consolidation's read-only inspection:

- Coordinator revision **372**, global `enabled: false`.
- Decision feed revision **32**: all five recorded decisions resolved, including
  isolated Linux adapter, Slack access/token handoff, removed private route and
  the actual Slack ACK rehearsal. This is not a blanket product acceptance.
- `refresh-corbanu-initiative-map`: **PAUSED**, configured every ten minutes.
- `monitor-three-security-lanes`: **PAUSED**, configured every five minutes.
- Last owned implementation/receiving/evidence workers returned and were closed.
  Inspect current processes before claiming anything is still running.

Do not use stale automation prompts to resume work. When recurring operation is
qualified and authorized, change schedules through the host's supported automation
interface. A chat or TMUX pane alone is not an always-running watchdog.

## 3. The three workstreams

### 1 — PF13: security and protected credentials

Plan: [p0-security-levels](docs/plans/active/p0-security-levels.md).
Reserved sprint: [PF-27-S04 isolated credential broker](docs/sprints/current/p0-security-levels/pf-27-s04-isolated-credential-broker.md).
Existing owner task: **Set up and build PF13 branch**, native task ID
`01a04a32-b01b-7ad2-91b1-0f7ff2b11456`. Coordinate through its retained handoff;
do not start a duplicate security owner. Owner checkout:
`/Volumes/CorbanuDrive/Corbanu/worktrees/security-broker-resume-20260911`.

Latest meaningful actions at the pause:

1. Owner checkpoint `899041c914f82f31a3097a34801209e5f20874f4` completed scoped
   isolated adapter/process/root-session/preflight work and retained reviews/proof.
2. Accepted proof received into integration at `855ab3382f007f34320d3d80a0b495bba97bcc4e`.
3. Owner confirmed paused; descendants/build resources released. No accepted
   tracked product work remained unreceived at that checkpoint.

Native/all-platform and independently isolated functional acceptance remain open;
private feasibility failures are not production credential-broker qualification.
Travis approved the **isolated adapter**, not a waiver of those tests.
For [PF-35-S01](docs/sprints/current/p0-security-levels/pf-35-s01-classifier-corpus-and-evaluation.md),
dataset generation must use **DeepSeek API through Corbanu Terminal**, not the
local RTX PRO 6000. Fine-tuning remains on the RTX PRO 6000. Verify exact current
sprint path in the inventory if this link changes; do not improvise the contract.

### 2 — Accounting: unified agent cost and usage

Plan: [portfolio-agent-cost-accounting](docs/plans/active/portfolio-agent-cost-accounting.md).
Reserved sprint: [PF-60-S02 persistence and replay](docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s02-idempotent-usage-persistence-and-replay.md).
S01 is archived complete. S02 is **not** complete; S03 must not start merely
because one S02 increment is merged. Last goldens checkout:
`/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-contract-goldens-20260913`.

Latest meaningful actions:

1. Persistent usage/replay, exact prices, deletion/retention and policy repairs
   were integrated in bounded increments; dispatch coverage remains partial.
2. Original-contract goldens received as `f4507cb50`; combined verification at
   `855ab3382` accepted. Source tip `08eaae600dbbd468e4143f64a1ee2c28baf2acf4`
   is patch-equivalent to receiving commits, not missing work.
3. Mendel closed and build lease released at the management pause. S03 totals/UI
   and the proposed Responses HTTP continuation were not started. Collection OFF.

Preserve approved defaults: distinguish measured usage, estimates, bills,
allowances and balances; unknown is not zero. USD estimates use approved immutable
price snapshots. Numeric detail retention 90 days; aggregates 365; user deletion
removes both while minimal opaque deduplication records survive a 365-day replay
horizon. User-selected ranges must support user-selected interval filters.

Actual dispatch coverage is Anthropic-direct only at the retained checkpoint;
Responses HTTP/WS, Chat and other Corbanu/auxiliary/legacy paths still need their
allocated implementation/evidence. Read the S02 Remaining ledger before assigning.
These product choices are settled, not fresh human blockers.

### 3 — Task Node / delivery control, Slack and beta testing

Plan: [initiative-delivery-control](docs/plans/active/initiative-delivery-control.md).
Reserved sprint: [PF-80-S01](docs/sprints/current/initiative-delivery-control/pf-80-s01-delivery-control.md).
This is the management-bootstrap lane currently authorized for bounded execution.

Latest meaningful actions:

1. Owned ingress source `9f692f65337d3a6ec2c203df0d8680db8fece0da` received as
   `c48dab1002ca9a8797f1b0818cbfdb824f8822df`; subsequent bootstrap components
   added fresh manager cycles, owner dispatch, lifecycle/integration and isolation.
2. Real Fable-driven receiving merged decision-inspection source
   `adc9eece3ffdca782d0b529789c0762d8bc6bc56` into `c77123a7e9d4540a5b3ff19a5a9dc0f12848c923`.
   First verification failed on missing child PYTHONPATH; a fresh manager/worker
   reran verification without repeating the merge: **482 tests passed**.
3. First frozen DEC-001 execution ran with a code-blind Astra actor and confined
   browser. Independent review supports the visible result **with provenance
   limits**; remaining functional/recovery cases are not qualified. See
   [actual supervised evidence](qa/initiative-control/management-bootstrap/supervised-qualification-20260913.md).

Future sequential sprints include PF-79-S01/S02: a special Corbanu Desktop beta
branch/channel and ongoing well-specified **public manual testing tasks**, with
assignment, evidence and recurring triage. PF-81-S01 is the bounded screenshot/
inference QA harness. They remain planned dependents, not an authorized public
beta launch. [Beta contract](docs/plans/tasknode-beta-program.md).

### Receiving evidence is scoped

The combined product checkpoint `c48dab1002` passed 626 tests, zero skipped, using
the pinned offline `just test` command in the [pause record](docs/plans/management-pause-2026-09-13.md).
Accounting additionally has its exact receiving/golden/Core-selector evidence;
selected Core tests are not full-Core coverage. Bazel parity failed offline with
exit 37 because `@@v8+` was missing and fetch disabled. Preserve that limitation.
The later 482 tests cover control/bootstrap, not all Rust/product qualification.

## 4. How to run Corbanu, managers and workers

Verified installed candidate: **Corbanu 0.1.41**, TMUX **3.7c**. Binary SHA-256:
`4a8eba7b10199ea49aee42a720687b63cb4b1b2194c2ce164d1b9f6f88ee510e`.
Do not substitute an installed newer binary without recording and checking it.
`corbanu --help`, `--model`, `-C`, `-c`, `--sandbox`, `--ask-for-approval`,
`tasknode`, `exec` and `app-server` are present. Read current help before use.

### Existing qualified decision-only TMUX path

[`fable_launcher.py`](scripts/initiative_control/fable_launcher.py) creates a
fresh private home/neutral packet/TMUX socket, selects **`claude-fable-5-1-plan`**,
provider **`claude-plan`**, effort **`high`**, performs the supported Providers
masked-token setup, sends text and Enter separately, captures exact identity and
complete final JSON, and verifies owned shutdown. It intentionally disables tools.
**It cannot itself read this file, launch workers or perform integration.**

Supply a private JSON briefing (at most 64 KiB) containing selected source evidence,
not a giant transcript or instructions to read unavailable files. Require raw JSON
with first character `{`, last `}`, no code fences. Schema/receipt failures are
retained failed cycles, not invitations to silently repair returned decisions.

```bash
# Verify these existing private paths; do not print auth contents.
MANAGER_AUTH_FILE=/private/tmp/cfable.msxBbt/auth.json
# Set MANAGER_BRIEFING_FILE to a new owner-only bounded JSON packet.
# Set MANAGER_RUNS_ROOT to a new short owner-only directory outside Git.
python3 "$CONTROL_REPO/scripts/initiative_control/fable_launcher.py" \
  --briefing "$MANAGER_BRIEFING_FILE" --runs-dir "$MANAGER_RUNS_ROOT" \
  --binary "$FABLE_BINARY" --auth-file "$MANAGER_AUTH_FILE" --timeout 180
```

The integrated event driver uses existing durable state and private owner context:

```bash
python3 "$CONTROL_REPO/scripts/initiative_control/manager_cycle.py" --run \
  --state "$CONTROL_STATE/coordinator" --runs-dir "$MANAGER_RUNS_ROOT" \
  --binary "$FABLE_BINARY" --auth-file "$MANAGER_AUTH_FILE" \
  --owner-context "$MANAGER_OWNER_CONTEXT_FILE" --timeout 300
```

Owner context is bounded JSON with actual `observed_at` and `context`. Without
`--run`, the driver is OFF; paused/owned/empty state does not launch. Accepted
actions are proposals, not proof of dispatch. Inspect the current allocation and
owner APIs rather than clearing pause flags wholesale. See
[manager-cycle contract/recovery](qa/initiative-control/management-bootstrap/manager-cycle.md).

### Tool-enabled takeover and TMUX worker sessions

You, the takeover Fable, need an actual tool-enabled host session. The decision-only
launcher above is not such a host; do not pass this whole brief to it and wait for
external work. Keep its accepted role intact. A tool-enabled Corbanu session can
own shell/TMUX operations within its real permissions; a Codex Desktop host can
also execute validated requests through its native subagent tools. **Desktop-only
APIs are not automatically present inside Corbanu's CLI.** Check actual capability.

For a tool-enabled Corbanu worker, allocate an exact writable worktree, a fresh
owner-only app home configured through the supported authorized account flow,
private logs and a unique short socket. A command template, not a claimed
qualification of this new general-purpose worker path:

```bash
# TASK_HOME, TASK_WORKTREE, TASK_LOGS and TASK_SOCKET must be allocated absolute
# paths; TASK_SOCKET is inside a short owner-only run directory.
# TASK_SESSION is a unique safe name; TASK_PROMPT contains no credentials.
tmux -S "$TASK_SOCKET" -f /dev/null new-session -d -s "$TASK_SESSION" \
  env CORBANU_HOME="$TASK_HOME" RUST_LOG=trace \
  "$FABLE_BINARY" --no-alt-screen -C "$TASK_WORKTREE" \
  --model gpt-6-astra -c 'model_provider="openai"' \
  -c 'model_reasoning_effort="high"' -c "log_dir=\"$TASK_LOGS\"" \
  --sandbox workspace-write --ask-for-approval on-request
tmux -S "$TASK_SOCKET" capture-pane -p -t "$TASK_SESSION"
# Resolve startup/account/trust prompts in the actual visible TUI first.
tmux -S "$TASK_SOCKET" send-keys -t "$TASK_SESSION" -l -- "$TASK_PROMPT"
tmux -S "$TASK_SOCKET" send-keys -t "$TASK_SESSION" Enter
tmux -S "$TASK_SOCKET" capture-pane -p -t "$TASK_SESSION"
```

For a tool-enabled Fable host, the corresponding model/provider are
`claude-fable-5-1-plan` / `claude-plan`, effort `high`; authenticate through native
Providers recovery, never a token in a shell argument or prompt. Do not copy
conversation history to refresh context. Confirm actual runtime model/provider,
effort, working directory and tool access before accepting its ACK. A pane existing
does not mean a turn started or completed. Track actual native session/turn IDs,
logs, deadlines and exits; use fresh explicit allocations on replacement.
Do not broaden sandbox/permissions just to make a failed test appear successful.

The TUI skill requires trace logs, exact candidate and actual keys; consult
[test-tui](.codex/skills/test-tui/SKILL.md). Existing
[astra_tui_acceptance.py](scripts/astra_tui_acceptance.py) illustrates typed
evidence and pane operations, but hardcodes a different effort/acceptance flow.
Do not reuse its settings blindly. Trace logs are private. Exit normally and
confirm only owned processes ended; never use global `tmux kill-server`.

Where the current host genuinely exposes native subagents, use fresh context:
`gpt-6-astra` / `high`, or publisher `gpt-5.6-luna` / `xhigh`. Capture actual
spawn IDs, require ACK, submit work, verify returned evidence, close and confirm
closure. Do not invent IDs or create user-owned sidebar tasks as a substitute.
[native_owner.py](scripts/initiative_control/native_owner.py) mediates one trusted
host request/response at a time; it does not implement the host tool itself.
Its worker kinds do **not** include `integrate` or `verify_integration`; those
require explicit owner assignments through the integration mechanism, not fake
worker kinds. [Native-owner contract](qa/initiative-control/management-bootstrap/native-owner.md).

## 5. Integration, reviews and testing

One integration writer at a time. Use
[integration.py](scripts/initiative_control/integration.py) and the existing core
claim/dispatch/ACK/return/verification sequence. Require exact source commit,
destination branch/base, bounded file scope, worktree cleanliness, reservations,
receiving tests and durable receipt. Preserve its active/failure markers and
reconcile against real Git state; never erase a marker or repeat an uncertain merge.
Historical `/private/tmp/insm.58djfu/receive.py`, `verify.py` and dispatch helpers
contain consumed claims and candidate pins: **not reusable queue runners**.

After qualified acceptance: integrate → receiving verification → complete and
archive sprint documents → dependency-gated successor allocation from that commit.
Prepare successor plans earlier, but do not implement dependent work before the
predecessor is completed/archived. Internal default-OFF checkpoint merges are
allowed without claiming feature acceptance or sprint completion.

Fable as integrator may authorize additional scoped reviews beyond the usual
five, recording purpose, usage and allowance. Preserve the ledger across fresh
agents. One clean substantive review is reason to progress, not to seek five.
Review budget never waives security findings or mandatory execution. Existing
external-review tooling: `/Users/Neo/.codex/skills/autoreview/SKILL.md` and
`scripts/autoreview` beneath that directory; engine `codex`, Corbanu wrapper
`/Volumes/CorbanuDrive/Corbanu/.codex-work/pf27-resume-20260911/review-fable-high`,
model `claude-fable-5-1-plan`, `--thinking high`. That shared-home historical
wrapper is **not** the fresh-manager launcher. Keep reviews separate from management.

Mandatory applicable functional evidence has separate fresh code-blind design,
frozen cases, **independent execution**, then independent evidence review. The
implementer cannot execute its own acceptance. A binary-only directory/prompt
restriction is insufficient: enforce source/history/auth/tool/network/process/IPC
isolation, including children, with negative probes and positive package controls.
Neutral intent/cases/screenshots go to executors, not this implementation-rich brief.
Preserve failed attempts, ambiguity, raw verdicts and disposition of every case.

### Qualification immediately available to continue

Original 46 cases: DEC-001–026 and SLK-001–020, private frozen packet
`/Volumes/CorbanuDrive/Corbanu/.codex-work/frozen-functional.HKzJqj/packet`.
First-case harness is privately committed at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-execution.ikteKU`, commit
`db4211f18a4dbb34660749de232a6ec2f591222c`. Its exact authored source is preserved
as inert text in [the source archive](qa/initiative-control/management-bootstrap/frozen-execution-source-20260913/README.md).
No private models/auth/state/fixture exports were copied into Git.

The harness is fixed to c77123a7e9 and its retained fixture. It is **not** a generic
runner: fixtures are now historical/stale; changing candidate requires a new
explicit fixture/version and applicable revalidation. DEC-013/015 need same-browser
state changes/reloads; the current immutable guest packet lacks that phase switch.
Do not replace them with static screenshots or separate browser launches.

DEC-001 raw pass and replay are not full acceptance. Close the independent
provenance limits; bind health and page generations before reusable fixture
expansion; show harness outcome separately from executor verdict. Original reviews
and actual integration receipts are under `/private/tmp/insm.58djfu/` and the
source archive's linked evidence. Do not rerun consumed one-shot helpers.

Local isolation references: pinned ARM64 Docker image
`sha256:fdbf57e27079258be976cf5d8d309c1c00faadfc24cec5c562caf661b27926bd`,
Docker socket `unix:///Users/Neo/.docker/run/docker.sock`, cached Chromium under
`.codex-work/docker-browser-CcdJFy/chromium-1217`, seccomp policy under
`.codex-work/docker-boundary.WcQbTX/seccomp.json`, and bounded transport helper
`.codex-work/model-transport-b.I8BDWp/model_only_transport.py` (all below workspace).
Exact pins are in archived source. Missing private assets require re-provisioning
and proof; do not silently substitute. A model transport success or synthetic
IPC peer is not native cross-platform product acceptance.

## 6. Dashboard: one Luna publisher, accurate inputs

Update authoritative inputs, not generated HTML:

- Plans/current sprints: allocations, lifecycle and acceptance ledgers.
- [current-handoffs.md](qa/initiative-control/status-display/current-handoffs.md).
- `$CONTROL_STATE/control.json`: test cards, selected references and Task Node mappings.
- `$CONTROL_STATE/events/`: immutable redacted per-worker reports.
- `$CONTROL_STATE/decisions.fixture.json`: canonical decisions, despite its name.
- `$CONTROL_STATE/decision-slack-status.json`: generated Slack observation projection.

For decision edits use `decisions.load_fixture/save_fixture` with expected prior
digest/CAS and full history. Assessment timestamps must represent actual checking.
For reports inspect `checked_run()`/`RUN_STATUSES` in `control.py`, then:

```bash
"$CONTROL_PY" "$CONTROL_REPO/scripts/initiative_control/control.py" report \
  --state "$CONTROL_STATE" --file /absolute/path/to/redacted-report.json
```

Reports retain assignment/run, sprint, actual agent/session, machine/role,
branch/worktree/commit, timestamp and concise progress. A newer report uses the
same run ID; it does not erase immutable history. Close obsolete working reports.
Show sprint lifecycle separately from run activity; link every sprint mention,
blocked label to its reasons, and progress label to hover/focus notes. Decision
cards need summary, expandable detail, evidence, precise question and consequence.
Show source revision/age, machine, pending integration and failed/stale publication.

After source is reconciled, committed and pushed, give **one Luna Extra High
subagent** the exact source commit and one-shot assignment to run:

```bash
/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/sync-source.sh
```

Verified wrapper selects this integration checkout/branch. It exports/hashes,
renders, transfers, activates and verifies; **it does not pull or merge source**.
Require its exact commit/generation/manifest receipt and warnings. Never run a
duplicate parent sync. Preserve the timer's disabled/inactive state for an
authorized one-off publication during pause. Keep last-good pages on failure.

```bash
curl -fsS http://127.0.0.1:8769/health.json
curl -I http://127.0.0.1:8769/facilities.html
```

Require source identity/freshness as well as HTTP 200 and `X-Corbanu-Control: 1`.
The last retained publication before this handoff used source `239f7f82d5d410adb936963ef6be891a9184bbdd`,
generation `build-mppqf3at`, source time 2026-09-13T22:34:57Z. It predates the
latest feed revision and qualification commits: **reconcile and republish**, do
not report it current. The 20-minute source-age and 45-minute worker-age thresholds
are distinct. A running web service does not imply running agents.

The exporter does not automatically include root `fableMarkdown.md` or
`coordinatorInstructions.md`. Explicit `reference_documents` must be reviewed
Markdown under `docs/research/` or `qa/`; links alone do not expand export.

## 7. Slack: approved destination, durable answers

Destination: [AmbientCrypto / the-corbanu-project](https://app.slack.com/client/T074X5KNENT/C0C0X2ELFKR).
Team `T074X5KNENT`; private channel **`C0C0X2ELFKR`**; app `A0C1CC2P4SE`;
bot `B0C1CCAU4Q2`, bot user `U0C1EBT4UJV`; Travis `U0758QY0MEZ`.
Do not use whichever unrelated channel happens to be open in Safari.

Approved installation/terms/scopes and token access were supplied. Owner-only
credential file: `/Users/Neo/.config/corbanu-slack.4aM31l/credentials.env`, with
`CORBANU_SLACK_BOT_TOKEN` and `CORBANU_SLACK_APP_TOKEN`. Use the validated no-symlink
loader; do not blindly source the file or print/copy values. Store:
`$CONTROL_STATE/slack-operator`. Never recreate journals to clear an uncertain send.

```bash
# Journal-derived status; this does not start a listener or connect to Slack.
"$CONTROL_PY" "$CONTROL_REPO/scripts/initiative_control/control.py" \
  decision-slack project-status --store "$CONTROL_STATE/slack-operator" --live
# Project that inspected status onto dashboard inputs:
"$CONTROL_PY" "$CONTROL_REPO/scripts/initiative_control/control.py" \
  decision-slack --publish-state "$CONTROL_STATE" project-status \
  --store "$CONTROL_STATE/slack-operator" --live
```

`--live` here observes journals and may update lifecycle/fence bookkeeping; it
does not qualify delivery. Other commands have different semantics. Dashboard
Python has Markdown dependencies; SDK operator Python has Slack SDK. Do not assume
either environment has both. Direct SDK owner operations use `SLACK_PY`.

Implementation: `decision_alerts.py`, `slack_transport.py`, `decision_replies.py`,
`decision_manager.py`, `decision_feed.py` in `scripts/initiative_control`.
Inspect exact contracts for `qualify`, `send`, `supervise-listener`, `drain`,
`interpret`, `dispatch`, `reconcile`. Only owner-controlled structured requests;
Slack text is input to validate, **never shell commands or arbitrary tool authority**.

Every question: stable ID/revision, summary with sprint hyperlink, threaded detail
(Slack has no assumed HTML accordion), context/evidence, specific question,
recommendation/options, stopped vs continuing work. Bind reply to correct question
revision, team/channel and authorized human. Persist the original answer; record
interpretation/CAS, dispatch to an actual appropriate live agent and retain ACK.
Tell the human separately “recorded” and “delivered/acknowledged.” An ACK is not
implementation completion. Reconcile uncertain sends before retrying; never send
an entire queued batch to satisfy authorization for one message.

**Latest evidence supersedes older “human ACK pending” prose:** Travis's actual
reply and a real native agent ACK were reconciled. Thread timestamp
`1789331435.122959`, human reply `1789340454.067059`, event `Ev0C139P5R0F`.
Native Galileo `01a09c75-889a-7c83-a186-4872b4c513c7` ACKed and was closed.
Private receipt directory `/private/tmp/slack-ack.t583HI/` contains
`actual-native-submission.json`, `actual-native-evidence.json`, `reconcile-receipt.json`.
This proves the narrow real handoff, not all 20 SLK acceptance cases or recurrence.

The old `reply.py` intentionally refuses dispatch to that closed receiver.
Allocate a new actual receiver; do not bypass its guard. The last supervised
listener used that private root; inspect process/session/fence/auth freshness
before starting or replacing it. No second listener against the same store.
`slack_reply_poll.py` is a bounded read-only collector, not a scheduler or answer
resolver. Periodic reception is not established until the monitor actually runs
and its last successful observation is visible. Keep Slack finalization a priority.

## 8. Native Task Node: reuse existing infrastructure

Use Corbanu's **`/tasknode`** UI and `corbanu tasknode --help`, linked native
profile **IridiumMaster / @iridiumeagle**, and Campaign Tracker. Do not build a
parallel credential store. [Existing task records and receipts](docs/plans/tasknode-workstream-tasks-2026-09-11.md):

| Initiative | Existing target |
| --- | --- |
| Security | `task_865f75c6911f953c6586cdc1f4531e4f` |
| Accounting | `task_b3e8506327fb906173fd68b2f642221b` |
| Task Node / beta coordination | `task_789a0f3bd75b41d1eca20cae698f04cf` |

These were created once and verified **Proposed**, not Accepted/completed/rewarded.
Recheck live identity/entitlement/enrollment/mappings and compatible progress
lifecycle; old receipts are not current server authority. Travis authorizes routine
progress updates when the tech is ready and his node may be publisher initially.
That does not authorize task acceptance, wallet signing, payments/rewards, arbitrary
old-queue flushes or public beta publication. If acceptance is actually required,
raise that concrete decision rather than silently accepting.

`tasknode.py prepare/preview --state ... --event-id <exact cc-ID>` inspect one
immutable redacted event without network writes. The inherited `flush` can send
**up to 20 events** and rejects `--event-id`; native capture creates fresh IDs and
sync is a batch. Do not pretend either is a qualified single-event transport.
Keep posting OFF until the allocated transport/recovery/identity gates pass.

Historical recovery **PF-76-S01** meant delivery control; main PF-76-S01 means
provider persistence. Receiving delivery control is **PF-80-S01**. Preserve exact
per-record reconciliation; no global alias, rewritten payload or regenerated
event to evade a hold. New PF80 reporting needs explicit PF80 mapping.

## 9. Machines and protected credentials

| Machine | Verified coordinate / intended role | Important boundary |
| --- | --- | --- |
| This Mac | `/Volumes/CorbanuDrive/Corbanu`; current source/host coordinator, private Slack owner, dashboard tunnel | CorbanuDrive must be mounted; reserve build/write resources |
| Alex RPC / development Linux | `pfrpc@178.156.143.199`; `/home/pfrpc/corbanu-control` | Direct SSH, no Tailscale dependency; remote web binds `127.0.0.1:8768` |
| RTX PRO 6000 | `100.99.88.49`; fine-tuning/media | Different tailnet; current reachability not established by RPC success |
| Drone / RTX 4090 | `100.81.145.102`; media/fallback | Verify reachability and exclusive allocation before use |
| Other assets | Two local VMs, two laptops, three powerful GPU/CPU machines; three Codex plans, Claude, DeepSeek/GMI APIs, Brave, scrapling | Inventory supplied by Travis, not proof every service or credential is provisioned |

Use the existing protected RPC adapter; it already has access to the supplied
server authentication. Never embed the password in this file, command arguments,
prompts, Slack or Git. Pass host explicitly:

```bash
"$CONTROL_ROOT/ssh-server" pfrpc@178.156.143.199 'hostname'
```

The adapter exists and is owner-executable/private (0700). Its credential is not
part of the repository. On another host provision the approved adapter/secret
securely; do not assume cloning Git transfers login state. If unavailable, ask
for the specific missing credential, not all credentials again. Do not disable
SSH host-key verification. RPC `bwrap` feasibility previously failed with
Operation not permitted: it is not a qualified isolated-executor host.

Other protected references, metadata verified owner-only 0600 at consolidation:

- Fable manager auth: `/private/tmp/cfable.msxBbt/auth.json`, sole expected field
  `CLAUDE_CODE_OAUTH_TOKEN`; short-lived/private path, verify availability/freshness.
- Existing Astra account auth: `/Users/Neo/.codex/auth.json`. Use authorized native
  loading/mediation, not logs or a copied conversation profile.
- Slack credentials: path in section 7. Existing native wallet/account profile
  should be reused. Travis can enter wallet-unlock password in the actual app.
  He identified `rw9xgwQ8...T9v8sa.txt` in iCloud Drive as seed backup; **that is a
  partial identifier, not a verified exact path or a reason to read/export it**.
  Do not search/print seeds for progress reporting. Escalate a real native recovery
  need with the narrowest necessary action.

Do not export whole private run roots: some contain real auth symlinks, native
SQLite state and raw private logs. `/private/tmp` is not durable cross-machine
storage. Preserve needed non-secret source/evidence hashes in Git and arrange a
separate approved private-state handoff if moving hosts. Do not spend/rotate
credentials/install dependencies merely because hardware or subscriptions exist.

Facilities must survive dashboard source updates:

| Facility | Interface |
| --- | --- |
| ComfyUI — RTX | `http://100.99.88.49:8188/` |
| YuE2 / YuE — RTX | `http://100.99.88.49:7861/` |
| ACE-Step — RTX | `http://100.99.88.49:7862/` |
| RVC — Drone | `http://100.81.145.102:7865/` |
| ACE-Step fallback — Drone | `http://100.81.145.102:7866/` |

Preserve Facilities navigation, generator, publish output, safe URL allowlist,
upstream-repository links and real HTTP tests. These are links, not new model
services or health claims. `/Applications/Corbanu Control.app` calls the private
`open-dashboard.command`; local view is `http://127.0.0.1:8769/`, **not** a link
Alex can open remotely. Shared remote viewing was deferred.

## 10. First takeover actions and definition of success

1. Read this brief, canonical policy and current actual state. Record yourself as
   manager/integrator for the authorized takeover; confirm no competing writer,
   listener or old worker owns a resource. Inspect rather than replay old scripts.
2. Verify integration tip/remote, then reconcile latest worker and decision inputs.
   Assign one Luna Extra High publication and verify actual receipt/freshness.
3. Continue the authorized supervised Fable→worker loop immediately. Close DEC-001
   evidence limits and allocate the next frozen functional/recovery cases. Repair
   missing test setup yourself or through fresh scoped workers; do not wait for an
   unattended controller before testing integration/restart recovery.
4. Finish applicable Slack qualification with an actual current receiver; ensure
   future replies enter durable events and reach the correct agent. Reconcile
   communication failures visibly, without repeating answered human questions.
5. Reconcile ownership/remaining gates for all three reserved sprints. Qualify the
   complete handoff, stalled/crashed worker, uncertain effect and restart paths;
   then activate the verified recurring owner/watchdog only under the applicable
   launch/resumption authority. Notify Travis in Slack when that actually occurs.

For each active workstream there must be an acknowledged assignment, owned queued
action, explicit dependency wait, intentional pause, or delivered actionable human
question. “Nothing assigned” without a reason is an orchestration defect you own.
Success is accepted useful work with reliable integration and human visibility,
not the number of agents or a green dashboard.

## Consolidation and audit notes

Accepted source increments are already merged/cherry-picked into integration.
Accounting source tips are patch-equivalent; security QA matches the owner tip
apart from an existing whitespace-only ledger change. Owned-ingress source and
allocation are received. The old Task Node checkout's two dirty Slack files are
byte-identical to integration; do not reapply them or erase that checkout.
Unrelated dirty recovery/other product work is deliberately not swept into this
handoff. The privately committed first-case harness is preserved as inert source
evidence, not installed product code or a claim of complete qualification.

This commit should be pushed to `origin/integrate/management-workstreams-20260911`
and verified with `git ls-remote`. Do not push main, force-push, tag or release.
Historical documents describe their dates; use the newer scoped evidence above
to resolve stale “pending” statements, not to erase original failures.
