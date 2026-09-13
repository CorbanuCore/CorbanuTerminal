# PF-80-S01 — management bootstrap allocation

Product initiative under **Internal delivery control — TO BUILD**, September13
explicit five-part goal; [full coordinator contract](../../../coordinatorInstructions.md).
This allocation preserves the entire objective, not a replacement MVP. Parent
coordinates startup until the Fable manager is qualified. Other product work and
old automations remain paused; no main/release or live Task Node bulk operation.

## Ownership and code boundaries

Base eb01bf006eacbeef5be07174a3f37ad124abc74c. Parent canonical worktree/branch:
`worktrees/management-workstreams-20260911`, `integrate/management-workstreams-20260911`.
Launcher worker: `worktrees/fable-launcher-20260913`,
`bootstrap/fable-launcher-20260913`, same base. Paths are relative to Corbanu root.

- Fresh Astra High launcher worker: ONLY `scripts/initiative_control/fable_launcher.py`,
  `scripts/initiative_control/test_fable_launcher.py`,
  `qa/initiative-control/management-bootstrap/fable-launcher.md`.
  Own no credentials, shared registration, source state, service or policy files.
  Target 1200 total lines, hard 1500; stop for reallocation if required.
- Parent: `scripts/initiative_control/coordinator.py`, `coordinator_cli.py`,
  `integration.py`, `test_coordinator.py`, `test_integration.py`, plus existing
  `control.py`, `decision_alerts.py`, `decision_manager.py`, `decision_feed.py`,
  `slack_transport.py` and their exact associated tests as required for linkage.
  Parent owns product/plan/sprint updates, approved private state, live Slack/HTTPS
  configuration, mainline registration and rehearsal evidence. First durable
  coordination/integration unit target 1600 total/900 non-test; hard 2000/1200.
  Later UI/live bridges get an explicit scope update before implementation.

No Rust or dependency-cache campaign. Same-sprint disjoint workers are not extra
initiatives. Shared edits and receiving tests remain serialized.

## Sequential core owner-controls allocation

After accepted core84f16cceec87292296ff94a9d431a86d23a9fe52, a fresh Astra High
worker owns only `scripts/initiative_control/coordinator.py`, `coordinator_cli.py`,
`test_coordinator.py` and `qa/initiative-control/management-bootstrap/owner-controls.md`
in `worktrees/coordinator-bridge-20260913`, branch
`bootstrap/coordinator-bridge-20260913`, base84f16cceec87292296ff94a9d431a86d23a9fe52.
Parent relinquishes those four files until handoff. New unit target600 changed
lines/350 non-test, hard950/550; this is a new authorized bootstrap unit, not
unbounded extension of the completed first review. No live state/credentials,
launcher, integration.py, dashboard, Slack, policy or scheduler writes.

Deliver supported revision-bound owner controls to add/replace exact frozen
allocations and enable/pause an individual stream without reinitializing SQLite;
do not silently mutate a claimed allocation or leave prepared claims executable
against changed scope. Add explicit completion/archive and successor transition
operations that require verified receiving/action evidence and owner-checked
mandatory gate records; dependency predicates alone are not completion. Preserve
the three-reservation limit, owner-only authority, pause precedence, audit history,
wrong-revision/duplicate/restart refusal and the distinction between claims and
acceptance. Tests must prove these paths and denials through the CLI. No actual
sprint is completed/archived by this implementation or its synthetic tests.
The parent performs Fable material review and receiving integration, then owns
actual native tool/rehearsal wiring in a subsequent explicitly allocated unit.

## Bounded actual native handoff rehearsal

After owner-controls receiving `fc7656ee370e33277f093144ba1ccf773fb2afc4`, the
parent may exercise one actual fresh-manager decision and one fresh Astra High
acknowledgment/reconciliation worker using the existing owner CLI and private
operator artifacts. The worker has zero write scope: inspect only this canonical
checkout's branch/HEAD/clean status and confirm the launcher/owner-control entry
points exist. No product work, reviews, builds, source edits, services, credentials,
network, document archives, Task Node sends or additional workers. Bind its ACK
to the exact claim/allocation digest and its actual native identity before work.
Capture actual return and independent owner comparison, close the worker, retain
pending/history records and return global/delivery dispatch to paused afterward.
Both other streams remain paused throughout. This is one operator-driven bridge
rehearsal, not proof of a deployed automatic event controller or recurring loop.

## Read-only Slack reply collection allocation

Sequential bootstrap unit from receiving `327eade129e186a5c66a3bacbf60c2342771de86`:
fresh Astra High worker in `worktrees/slack-reply-poll-20260913`, branch
`bootstrap/slack-reply-poll-20260913`, owns only
`scripts/initiative_control/slack_reply_poll.py`, `test_slack_reply_poll.py` in
the same directory, and `qa/initiative-control/management-bootstrap/slack-reply-poll.md`.
Target 650 total/350 non-test lines, hard 850/450. Parent retains all existing
modules, registrations, credentials, live state and scheduling. One Fable High
material review plus necessary in-scope corrections; prior review history remains.

Deliver one bounded owner-invoked collector using an injected authenticated Slack
WebClient and the existing durable Coordinator event API. Exact owner-pinned
team/bot/human/channel and tracked thread-to-decision mapping, never discovered
from untrusted replies. Read replies only; no Slack sends, canonical resolutions,
worker dispatch, service/automation enablement or credential loading in the worker.
Persist original bounded reply evidence with stable deduplication across repeated
polls/restarts and distinct edited revisions. Pagination must not silently skip
answers: bounded partial scans expose continuation/coverage, not a false caught-up
claim. Failure/429 and unknown/deleted/malformed evidence must stay explicit;
polling cannot prove unseen deletions or replace qualified Socket Mode ingress.
No approvals inferred from text and no message text in diagnostic stdout. Recording
events is allowed with product dispatch paused. Tests cover real API-shaped
fixtures, identity/thread/author rejection, duplication/edit/restart, pagination,
partial results, failure and persistence-before-observed semantics. No live API
or credential access by this worker. Parent owns actual bounded replay and later
monitor activation after the relevant launch gates; this is not recurring enablement.

## Event-to-manager driver allocation

Next bounded unit after collector receiving and actual read/dedup, base
`bca6485a2e60803393bbca0ee56d98953022ed12`: fresh Astra High worker in
`worktrees/manager-cycle-20260913`, branch `bootstrap/manager-cycle-20260913`,
owns only `scripts/initiative_control/manager_cycle.py`, its adjacent
`test_manager_cycle.py`, and `qa/initiative-control/management-bootstrap/manager-cycle.md`.
Target800 total/450 non-test lines; hard1050/600. Parent retains coordinator,
launcher, native tools, private credentials/state, policy and scheduling. One
Fable High material review plus necessary scoped corrections, retaining history.

Build the minimal owner-invoked one-cycle driver on existing Coordinator and
`fable_launcher.run_launcher`, not a new daemon/framework. No native tool API is
callable from this Python module: return accepted prepared actions for the actual
host coordinator; never synthesize spawn/ACK/return or integrate anything here.
Observe enabled/owned/pending state before inference; refuse duplicate cycles.
Claim the existing manager token, freeze an owner-only briefing/artifact record,
include the three workstreams and their last three actions plus original selected
event/result evidence, and call the actual fresh launcher once. Enforce64KiB
before inference, expose missing/oversized evidence as an owner hold without
silently truncating it or consuming events. Trusted owner context is bounded data,
not model-authored policy. Describe the seed as historical and allow separately
dated owner observations; no invented current facts. Prompt short rationales under
300 bytes while preserving the core's hard1000-byte limit and exact frozen inputs.

Bind successful receipt to this manager token, exact briefing digest, binary and
actual model/provider/high/session/turn/complete-response/shutdown evidence before
calling `accept_decision`. Retain unsuccessful/stale/invalid attempts and report
ownership/reconciliation requirements; never retry the model implicitly or clear
uncertain ownership based on elapsed time, a caller boolean or PID guess. An owner
uses existing reconciliation after actual shutdown inspection. No credential load,
network, tools or state writes at import/help/OFF; execution uses the existing
explicit private auth path only through the launcher. Test injected launcher
receipts, empty/paused/owned, evidence overflow/missing, wrong identity/digest,
stale revision, failed shutdown, crash boundaries and successful accepted action.
Those fixtures are not live-manager or native-dispatch qualification. Parent owns
actual fresh-manager replay, native bridge, future supervision and recurrence.

## Isolated-executor transport falsification allocation

Parent inspected the read-only Hypatia audit and verified the Astra catalog's
`code_mode_only` precedence in source. Its inspected default standalone0.1.36
is not our current0.1.41 candidate; preserve that limitation. RTX SSH timed out,
so no remote packaging, forwarding or runtime claim is established. The existing
646-line/12-test five-file boundary runner and all old evidence remain frozen.

Authorize only Stage A locally now: fresh Astra High worker may create exactly
four private files under `.codex-work/executor-route-audit.Nerv9u/model-only-v1/`:
`model_only_transport.py`, `model-only.json`, `test_model_only_transport.py`,
`README.md`. Target650 total/400 non-test; hard850/550. This is manager-owned
infrastructure within PF-80-S01, not PF81 activation or a fourth initiative.
No canonical source edits or second storage/proxy/daemon framework. Parent owns
the one Fable High boundary review and necessary corrections. The audit's
eight-file full executor proposal is not simultaneously allocated.

Use exact existing macos-candidate-final9 Corbanu0.1.41
SHA256 `4a8eba7b10199ea49aee42a720687b63cb4b1b2194c2ce164d1b9f6f88ee510e`.
Build the smallest fixed model-only command/profile on supported CLI/catalog
configuration, with a neutral direct-mode Astra entry preserving required wire
metadata. Explicit Astra/High, no fallback/session reuse, no operational tools,
hooks, host docs/memories or guest-selected paths/providers/commands. First test
only synthetic owner-private homes, file auth and loopback fixture servers; never
read/use real auth, Keychain, models or external network. Bound processes, output
and cleanup; inspect captured real request/response and hostile unadvertised tool
calls for shell/file/image/MCP/code-mode/collaboration/extension capabilities.
Exercise synthetic hook/notify/AGENTS/memory/plugin/config canaries and ordinary
text success/invalid configuration failure. Distinguish wire-content proof,
unsupported-tool responses, marker absence and filesystem-denial proof honestly.

Return exact package/profile/catalog/argv hashes, actual tests and failures,
remaining limitations and frozen four-file diff. If supported configuration
cannot remove a required capability, identify the exact remaining registration
and stop that admission path for a separately scoped source correction. No
prompt-only isolation, live credentials/inference, remote SSH/forwarding, namespace
changes, full mediator/action loop or product/browser/Slack acceptance is authorized
by Stage A. Parent chooses and allocates subsequent enforced executor work after
examining this falsification evidence and current machine reachability.

## Launcher interface for coordinator integration

Standalone CLI reads a bounded JSON briefing from `--briefing`, fresh private
`--runs-dir`, exact `--binary`, owner-only `--auth-file`, bounded `--timeout`.
No network on help/import/offline tests. Its stdout returns one redacted JSON
receipt containing run_id, status, model, provider, effort, session_id, decision
and artifact paths. Errors/timeouts return a nonzero exit and retained receipt.
An outer caller supplies credentials privately; never put credential values in
briefing, argv, stdout or exported artifacts. Exact model claude-fable-5-1-plan,
provider claude-plan, effort high; no fallback/resume/shared session history.
Use actual Corbanu interactive TMUX, separate text/Enter, full final response
capture and authoritative session identity. Fresh application home and neutral
packet CWD; tools restricted to required read-only manager work. A new home is
context isolation, not full functional-executor containment.

Manager decision JSON: `state_revision` integer, `actions` list. Each action has
stable `id`, `kind`, `workstream`, `sprint`, `rationale`, `inputs` object,
`timeout_seconds` and exact `expected_revision`. The coordinator validates the
allowed kind, policy authority, scope, dependency/receipt state and revision;
the launcher validates only framing, actual model/session and complete response.
Never treat pane quietness or a JSON snippet in tool output as final acceptance.

## Full completion evidence

1. Fresh Fable: two actual independent runs, correct model/effort/session IDs,
   complete structured decisions, bounded timeout/cancel and clean shutdown.
2. Coordination: durable event → fresh manager → validated action → actual native
   worker dispatch → ACK/result → new event; deduplication, stale decision denial,
   crash/restart and silent-stall watchdog tests. No fake dispatch receipts.
3. Integration: exclusive writer, actual temporary Git merge/receiving test and
   receipt; concurrent writer denial, conflict/failure recovery, branch/candidate
   checks and dependency-complete successor promotion. No unchecked worker claims.
4. Slack/private HTTPS: actual authenticated question, actual human reply,
   canonical decision, native-agent ACK, restart/recovery and stable links usable
   without this Mac; approved audience/access-denial tests. Preserve old journals.
5. Initialization/rehearsal: real three-lane baseline and approvals, aligned policy,
   complete handoff and failure/restart rehearsal, then qualified recurring enablement
   and final Slack completion ping. Until then no completion or running claim.

## Verification and limits

Use exact nonzero test receipts, all prior failures retained. Initial internal
launcher/queue engineering has reasoned internal-only N/A for GUI acceptance;
the later combined dashboard/Slack workflow requires independent code-blind
design/execution/evidence under root policy. Do not relabel smoke tests as that gate.
One fresh Fable material review per returned unit plus necessary correction;
extensions preserve prior PF80 usage and require purpose, not a reset.
Do not grow an unbounded process framework: implement these explicit operations
on existing Corbanu, native tools, private state and supported Slack machinery.
