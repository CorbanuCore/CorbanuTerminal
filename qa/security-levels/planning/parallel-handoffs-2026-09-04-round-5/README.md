# Security round five — execution and monitoring

Authorized by the user's 2026-09-04 instruction to launch three parallel agents,
own integration, build on RTX and monitor ongoing. This is execution allocation,
not a completed-feature or release claim. Product initiative:
active `docs/plans/active/p0-security-levels.md`; product citations and literal
scope are in each sprint.

## Dispatch

Source base: `07791288b6feeccfaee5a57c12452359cc666957`.
Coordinator branch: `integration/security-round5-20260904`, worktree:
`/Volumes/CorbanuDrive/Corbanu/worktrees/security-round5-20260904`.
Worker branches start with the committed allocation on top of that source base.

| Sprint | Agent | Branch | Worktree |
| --- | --- | --- | --- |
| PF-27-S04 | /root/broker | feat/security-round5-broker | /Volumes/CorbanuDrive/Corbanu/worktrees/security-round5-broker |
| PF-30-S01 | /root/provenance | feat/security-round5-provenance | /Volumes/CorbanuDrive/Corbanu/worktrees/security-round5-provenance |
| PF-24-S01 | /root/security_ui | feat/security-round5-ui | /Volumes/CorbanuDrive/Corbanu/worktrees/security-round5-ui |

## Worker handoffs

Broker: restore only allocated leaves from
`recovery/p0-security-isolated-broker-2026-09-02` at `cdb821289`,
preserving provenance of the recovered implementation in a separate commit.
Then implement the PF-41 digest-bound journal adapter and actual native Linux
peer/framing/death coverage. Preserve shared audit minimization. Full provider
data-plane, service isolation and macOS/Windows qualification remain explicit
remaining work until measured. Do not mark PF-27 completed from leaf tests.

Provenance: implement immutable validated envelopes, host-owned admission and
closed-world ingress registry, then protected-only native ingestion and
provider projections within the allocated session/client/tool hooks. Preserve
Permissive byte/behavior compatibility. Unknown origin cannot mint authority.
Record route coverage explicitly; pure projection fixtures do not prove every
native route. Do not mark PF-30 completed with unconnected contracts.

Security UI: implement observation-only /security, requested configuration
versus unavailable/unverified effective protection, three readable profiles,
keyboard navigation, inert Enter, Esc and matching /status.
Do not add activation, grants or downgrade. Capture narrow/error/cancel and
status flows using the real typed TMUX harness with separate text/Enter sends.

Workers may implement and author tests immediately after governance passes.
Request shared registrations from coordinator; never edit outside literal scope.
Keep commits recoverable and push scoped commits with truthful evidence status.
Do not archive incomplete sprints. Source-reference files lost in recovery are
not evidence of a fresh local source inspection or executed reference tests.

## Remote execution

Host: authorized RTX machine; login details stay outside tracked files.
Preflight: 1.7 TB free disk, approximately 80 GiB available memory; no shutdown
needed at dispatch. Existing remote worktrees/processes remain untouched.
Each lane gets a new mirror worktree under /home/travis/worktrees/security-round5-*.
Authenticated remote mirrors are created at allocation
`4f263ca73a2860031be37945204f4676cbb347d4`. Toolchain 1.95.0, just 1.58.0 and
nextest 0.9.143 are installed. Set PATH to include /home/travis/.cargo/bin,
CARGO_TARGET_DIR=/home/travis/repos/CorbanuTerminal-harness/codex-rs/target,
TMPDIR=/home/travis/security-round5/tmp and CARGO_BUILD_JOBS=8.
Use `flock /home/travis/security-round5/locks/build.lock` around every
compile/fix/fmt/test command. Push/fetch exact checkpoints into each clean
disposable mirror; preserve other worktrees. Copy each tested binary into
/home/travis/security-round5/evidence/<lane> before releasing the build lock.
Bring intended remote formatting back with apply_patch before final tests.
No local compilation; never conflate artifacts from different candidates.

Run scoped fix and formatting before final affected tests. Use just test, not
direct cargo test. Record version, commit, commands, counts, artifacts, actual
TMUX keys and outcomes. Synthetic fixtures only in traced TUI sessions.

## Review allocation ledger

Maximum five invocations per lane, not a target. Normal initial pair: Astra High
autoreview and Fable 5.1 High external review through Corbanu Terminal TMUX.
Coordinator assigns the next number before a reviewer starts. Follow-ups require
actual findings; exceed five only when critical findings continue.

| Lane | Invocations | Evidence / next action |
| --- | ---: | --- |
| Broker | 4 / 5 | #1 CLI rejected; #2 Astra findings repaired; #3 Fable timeout finding repaired; #4 no blockers, one deferred P3 EINTR robustness follow-up (helper exit 1, not a clean exit) |
| Provenance | 3 / 5 | #1 Astra provider-switch/lock findings repaired; #2 Fable realtime omission repaired; #3 verifies fixes, accepts adjacent stage-one memory policy-binding gap as follow-up; no overall clean review |
| Security UI | 3 / 5 | #1 CLI rejected; #2 Astra High clean; #3 Fable High clean; no further review planned |

Successful Astra runtime: installed app-bundled Codex 0.153.1; 0.145.0 was
rejected by the backend before any review verdict. Keep these attempts visible.
Fable wrapper forces claude-fable-5-1-plan at High and runs via private TMUX;
credentials never enter the committed packet.

Remote formatter prerequisite: uv 0.11.3, installed in the broker lane's
tools-venv and reused by all lanes. Bazel 9.0.0 matches .bazelversion; isolated
remote install SHA-256 c44a93f25398c68f904fa1d19b61d321de6c0d2f09dca375d7bc0dc9b9428403,
verified against releases.bazel.build's matching checksum.
Combined registration source 71afdd426 passed a remote broker/network-proxy/Vault
compile check in 14.66 seconds. Generated Cargo.lock adds only the intended
13 dependency lines. Both just bazel-lock-update and just bazel-lock-check pass;
MODULE.bazel.lock unchanged. Existing platforms/rules_cc version warnings remain.
These are preliminary combined checks, not final post-review qualification.

### Integration checkpoint and known test environment effects

Source candidate dd2adb72b includes all three lanes, scoped dependency locks,
broker timeout fixes, provenance provider-switch fixes and realtime guards.
The earlier combined source f60d15f16 plus the committed module sort passed
protocol 285/285, Core 26/26, broker/Vault/proxy 338/338, UI 235/235 and
actual-key TMUX 3/3 after fix/format. Realtime lane source e592cf75a passed
88/88 provenance/realtime cases and actual-key TMUX. The final combined
affected-Core rerun passed 94/94 and all three actual-key TMUX journeys passed
at dd2adb72b, after fix/full formatting with no changes. The immutable RTX
candidate and exact run IDs are in [combined qualification](combined-qualification.md).
No merge to main or release is claimed.
The integration branch also contains the earlier, unmerged provider/onboarding
reconciliation inherited at its source base. Main has no new divergence as of
the final rerun dispatch. The coordinator has asked product authority whether
to merge this combined staged branch or retain it for now; do not interpret
successful tests as an answer or repeat the question on every heartbeat.

The accepted memory finding is outside the current source allocation:
memories/write stage one constructs an unbound ModelClient and serializes
rollouts. The coordinator verified its caller in app-server and that neither
path differs from the source base. It is an existing incomplete protection
boundary, not a newly introduced permissive behavior. Defer to PF-30-S02's
explicit inherited/live policy-binding contract; a configured-floor-only
patch would not establish that contract. This disposition does not waive the
finding or permit protected memory activation. PF-30-S01 stays in progress,
and the source subset is not a completed provenance feature.

Broker review 4's non-blocking EINTR robustness issue is recorded with its
next-service substage, including signal-handler tests. No fifth review is
needed without further source changes or blocking findings.

PF-30 full Core run using a fresh isolated temporary root: 3,455 passed,
five request-permission grant cases failed, eight skipped. All five failures
reproduce at allocation base 4f263ca73 (module: seven passed, five failed),
with extra "Failed to create stream fd: Operation not permitted" output.
They are existing failures, not a clean suite and not newly introduced passes.
Missing test_stdio_server and shared temporary-root contamination caused earlier
additional failures; required fixtures and a fresh per-run temporary root
resolved those. Never delete shared leftover test data to conceal failures.

Shared target caches also reused stale workspace artifacts across lane checkouts.
Under the shared build lock, invalidate the affected source mtime and rebuild;
if necessary clean only the identified generated workspace package, never the
entire cache or source. Copy exact candidate binaries before releasing the lock.
Use fresh TMPDIR roots outside /home/travis/security-round5/tmp for every run.
Run normal bazel shutdown before leaving a locked Bazel operation: its persistent
server can inherit the lock descriptor. One such inherited lock was safely
released this way; no process was forcibly killed.

## Integration and follow-through

PF-24-S01 is completed and archived after the final combined run; 26/76 P0
sprints are archived (34% rounded). The two other engineering sprints remain
in progress, not silently completed from staged evidence. Worker tasks have
finished their allocated checkpoints; none is still compiling or reviewing.
Next broker service work and the provenance/memory follow-up require their
recorded scope/contract decisions. PF-35 remains external.
All current source and evidence are preserved on the integration branch.
Main merge still awaits the already-asked product decision, including the
inherited provider reconciliation. No additional reviews are running.

Documentation closeout: plan/sprint checks pass (57 current and 114 archived
across all plans; P0 alone is 50 current and 26 archived). Both HTML files pass
unique-ID and local-link checks, and humanTest retains 20 independent checkboxes.
An isolated browser profile visually verified the 34% progress display, three
lane cards and RTX candidate instructions; no human test was marked accepted.
Screenshots are retained on CorbanuDrive under
`.codex-work/security-round5/security-progress-final.png`,
`security-lanes-final.png` and `human-test-final.png`.

Allocation is committed and pushed as 4f263ca73. All three workers were dispatched
with implementation authority and remote build instructions. Broker's first
recovery checkpoint 90ae3a0cf is pushed; this is not new final-tree qualification.

Coordinator serializes shared Core/protocol/TUI exports, Cargo/Bazel/locks and
navigation. Audit each actual diff against declared scope before handback.
Use at least 35% capacity for integration, remediation and reruns; reforecast
at each handback. Combined-tree testing precedes any completion claim.
Update humanTest.html and securityProgress.html with verified results, honest
percentages and pending human/platform gates. No release is implied by dispatch.

A thread heartbeat checks every 15 minutes, resumes actionable stalled work,
and reports meaningful progress, failure, completion or a needed user decision.
No unchanged-status notifications. Pause the monitor when the requested
integration and handoff are finished.

## Freed-agent design allocation

After PF-24-S01 archival, the user requested another lane for that agent.
The coordinator checked every draft P0 sprint's archived dependencies: only
PF-35-S01 is dependency-ready, and it remains the external campaign.
No other complete implementation sprint can be dispatched without first
finishing a dependency or explicitly restructuring the plan.

The freed `/root/security_ui` agent is therefore running a bounded,
read-only **memory-worker policy-binding design/allocation lane**, using
source `62540ca1d` and the accepted PF-30 follow-up. Output:
`.codex-work/security-round5-ui/memory-policy-binding-design.md` on CorbanuDrive.
It must propose one minimal host-owned binding API, actual call path, exact
scope, inherited/live policy semantics and fake-provider/TMUX regression plan.
No source changes, additional reviews, builds or privileged setup are authorized
by this design assignment. It does not consume a third executable sprint
reservation or waive PF-30-S02's dependency on completed PF-30-S01.
The coordinator will reconcile overlapping Core ownership and record the
implementation allocation before any code changes. Existing review counts and
the outstanding main-merge choice are unchanged.

## PF-35 external handoff

User explicitly removed PF-35 from the engineering main path.
Its engineering reservation returns to draft under the existing sprint process.
The external dataset campaign is independent and untouched; this is not a
cancellation or completed qualification. Preserve all prior merged evidence.
Corpus, evaluator-custodian report, production signature and N100 measurements
remain required; PF-35-dependent work stays gated until honest archival.
The external operator must reconcile/reallocate before future repository writes;
do not merge external changes over these active lanes without a scope audit.
