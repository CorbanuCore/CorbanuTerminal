# Owner daemon Increment B — worker transport

A pane echo must not authorize work, and a lost key-delivery receipt must not
cause another START. This internal transport records separate intents, key
receipts, rollout observations and shutdown evidence. It does not enable a
recurring owner or accept a worker's results.

Classification: product initiative within active initiative-delivery-control,
PF-80-S01 (in_progress). Product heading: **Internal delivery control — TO BUILD**;
excerpt: “fresh Fable 5.1 High management through Corbanu/TMUX; durable event
dispatch, acknowledgments and watchdog”.
Allocation owner-daemon-impl-02, digest
`7f4824e31451b1791c1d62af19f5c2276c8a82d1a263a29dd7ed7190edbf07d3`.
Frozen base `a826527bbc0029db4f5e9e81a097e15d2a2215f6`,
branch `bootstrap/owner-daemon-20260914`, worktree
`/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-owner-daemon-20260914`.
The dispatch supplies this increment's exact mandate; manager-owned plan/sprint
allocation reconciliation and review remain outside these five writable files.

## Configuration and call contract

The original four daemon config fields remain required. Optional `transport`
has exactly `kind: "tmux"`, absolute `binary`, its `binary_sha256`, absolute
resolved `tmux`, a short private `runs_dir`, and absolute `auth_link` ending in
auth.json. The transport never opens, copies, hashes or prints that auth file;
it creates only the authorized per-worker native link. There is no token input.
The package digest now includes owner_tmux.py. Default invocation remains OFF.
An explicitly armed default tick uses FixedTestAdapter. Explicit TMUX selection
returns HOLD / tmux_lifecycle_routing_unavailable without launching anything.

An admitted owner may call TmuxAdapter(config).prepare(binding, assignment).
Binding contains exactly action_id, claim (UUID), allocation_digest (SHA-256),
model, provider, effort, worktree, sandbox and approval. The owner supplies the
frozen assignment and validates its authority; this adapter grants none.
prepare creates a unique private home, run manifest and short dedicated socket.
Worker.launch starts the pinned binary through a fresh TMUX server without user
TMUX config, with explicit runtime/policy argv and a restricted environment.
All home aliases point at the private worker home; no ambient credentials pass.
The private socket/session identity is durable before new-session. A private TMUX
configuration enables remain-on-exit before executing the binary. A client timeout
or failure is recorded and reconciled by bounded socket inspection; creation is
never retried. Worker(run).inspect can continue reconciliation after restart,
including when the pane exited before its process start could be sampled.
The parent calls prompt only after inspecting actual startup readiness. Known
auth/trust/approval pane text refuses key delivery; no prompt is auto-approved.

prompt sends the frozen assignment and exact ACK instructions using TMUX
bracketed paste, then a separate Enter. start requires the exact first completed
assistant ACK line, including action/digest/model/effort. The claim is bound by
the exact submitted startup prompt and fresh session, not a fabricated native ACK.
START is sent once. Successful send-keys is recorded as accepted=false; only the
matching rollout user-message and configured turn prove submission.
RETURN requires the second completed turn, exact START input, matching runtime,
provider response identity, and a bounded body beginning with a standalone RETURN.
No pane, tool output, partial JSONL line or incomplete assistant message qualifies.

Worker(run) reloads existing metadata; effect intents prevent launch/prompt/START
retries after receipt loss. inspect records actual host boot, UID, pane/PID/server,
process starts, process group, observed descendants, session/thread/turn IDs,
rollout path/digest, pane digest, liveness and fixed-deadline stall status.
Missing socket/server or malformed evidence stays unknown/invalid, never inferred
successful or safely replaceable. Changed observations get separate private files;
unchanged polls do not rewrite evidence. Descendant discovery scans same-UID
processes at most once per second per Worker instance; intervening polls refresh
only recorded PIDs. Owned-process evidence is written only when it changes.
close sends /quit once and waits boundedly, including through unknown observations
and crashed panes with transient surviving processes. Clean means the observed pane exited,
tracked descendants are gone, the recorded server exited, and its session probe
fails. A stale socket file is harmless evidence, not a new launch permission.
There is no forced termination or global-server cleanup in production code.
Pane death is queried before sampling process identities. An exit after the pane
query can still produce a temporary unknown result; close re-observes through its
deadline. Exit status can briefly be empty while TMUX processes SIGCHLD. Shutdown
receipts retain the final observation, delivery uncertainty and the sampled server
identity/session-probe result, including unclean outcomes.

## Review correction — owner-daemon-impl-03

Bounded fix of existing Increment B behavior, within the same product heading and
requirement excerpt above. Frozen base `652435d7a35548e6bfe114a09c5885e828c762d4`;
allocation digest `af0facb692fd987f20f5368ba1eaaa763df8ebd345bd6a69d37c7ab36b097c04`;
claim `3d7c2e13-b80c-4449-aaed-bd491be8c324`. Branch/worktree unchanged.
Review input: `/private/tmp/frev.D98OWQ/review.log`.

- P2 inspection/shutdown race: corrected query ordering and bounded re-observation.
  Real harmless-shell regressions inject exits on both sides of the pane query,
  replay transient unknown/stale-survivor observations, and verify final uncertain
  evidence when the deadline expires.
- P2 launch identity gap: pre-recorded socket/session and restartable reconciliation.
  Real private TMUX regressions inject a client timeout after successful creation,
  temporarily hide socket probes, reload the Worker, and run an immediately exiting
  shell. Every created worker remains observable and closeable through its record.
- Harness load: same-UID discovery is throttled, shutdown polls use selected PIDs,
  unchanged observations/ownership skip durable writes, and poll cadence is 100ms.
  A regression checks repeated stable inspection performs no evidence writes.

The fixed default adapter and explicit-TMUX HOLD remain covered. This revision
does not enable a live tick or claim functional handoff. The internal-only N/A
proposal and later independent combined functional gate below remain unchanged.

Revision validation: the initial focused command (same environment and interpreter
as below, `-B -m unittest test_owner_tmux test_owner_daemon`) passed 42 tests in
17.612s. After adding the eighth regression, the documented SDK discovery command
below passed all 539 tests in 296.402s, including all 43 owner tests (25 TMUX,
18 daemon). No intermittent failures reproduced in these runs; the previous
Fable/version-probe and concurrency failures remain historical unresolved attempts,
not erased or relabeled. Only HTTP-fixture cleanup ResourceWarnings occurred.
`python3 docs/sprints/check.py` passed (116 current, 126 archived), and
`git diff --check` passed. No native credential prompt occurred.

## Evidence and remaining gates

Tests use real private TMUX servers running only the checked-in harmless shell
fixture, synthetic rollout records and a dangling synthetic auth link. They test
actual bracketed-paste bytes, exact ACK/START/RETURN correlation, restart intent
fences, wrong runtime/session/claim/return, stalls, crashes, a surviving child and
clean/refused shutdown. Increment A fixtures now live in resolved system temp;
their Python children also receive private home aliases and no inherited secrets.
Initial focused run: 32 tests, two failures and one error. It exposed stale socket
shutdown logic plus fixture assumptions about SIGSTOP and socket removal.
Focused replays passed 33 then 34; concurrent 35 failed 13 startup timeouts; isolated owner case passed.
Intermediate SDK 530 passed (294.607s); serial 531 failed nine existing Fable version probes (308.898s), while all 35 owner tests passed.
Isolated Fable replays failed then passed unchanged (6.742s). Cause remains unresolved; these attempts are retained.

This is internal engineering evidence, not actual-model/TUI acceptance. Proposed
internal-only N/A requires integrator acceptance; the later combined functional
gate remains. Before a live tick: Increment C must bind coordinator claims,
admission/pause checks, durable operations/processes, replay, verification and
lifecycle routing. Qualify the exact Corbanu package's startup and rollout format.
Same-UID metadata and sampled descendants are not enforced worker isolation or
proof against a child that escapes observation. Provision the design's enforced
boundary and independent confined acceptance before functional handoff.
Fresh manager routing, receiving/integration, typed Slack ACK/activation notice,
publication and packaged supervision remain C/D/E work. No live model, credentials,
Slack, Git transport, launchd, TensorCash/Isometric workflow, human acceptance,
benchmark or release qualification was exercised by this increment.
Final serial validation: 531 tests passed in 300.850s, including 35 owner tests; HTTP-fixture cleanup warnings only. Sprint checker and staged diff checks passed.
Command: `PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts/initiative_control:/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/lib/python3.14/site-packages /Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B -m unittest discover -s scripts/initiative_control -p '*test*.py'`.
