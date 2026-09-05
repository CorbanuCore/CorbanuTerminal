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
| Broker | 0 / 5 | Implementation first; no review started |
| Provenance | 0 / 5 | Implementation first; no review started |
| Security UI | 0 / 5 | Implementation first; no review started |

## Integration and follow-through

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

## PF-35 external handoff

User explicitly removed PF-35 from the engineering main path.
Its engineering reservation returns to draft under the existing sprint process.
The external dataset campaign is independent and untouched; this is not a
cancellation or completed qualification. Preserve all prior merged evidence.
Corpus, evaluator-custodian report, production signature and N100 measurements
remain required; PF-35-dependent work stays gated until honest archival.
The external operator must reconcile/reallocate before future repository writes;
do not merge external changes over these active lanes without a scope audit.
