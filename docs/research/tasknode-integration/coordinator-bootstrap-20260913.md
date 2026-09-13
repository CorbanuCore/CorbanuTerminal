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
