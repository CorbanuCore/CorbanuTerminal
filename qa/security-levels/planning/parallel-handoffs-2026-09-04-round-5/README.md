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
| Broker | 5 / 5 | Earlier #1–4 retained; new service tests pass at 6bdc84195. #5 Astra High completed clean, final evidence cd7457da7. #6 Fable coverage still awaits the user's explicit budget decision |
| Provenance | 5 / 5 | Earlier #1–3 dispositions retained; bounded segmentation #4 Astra High and #5 Fable High both clean. Final lane e890ae4a9 integrated; no further review |
| Security UI | 3 / 5 | #1 CLI rejected; #2 Astra High clean; #3 Fable High clean; no further review planned |
| Memory dispatch (new PF-30-S04) | 2 / 5 | #1 Astra P2 provider/config mismatch fixed; #2 Fable confirms correctness with a documented non-blocking P3 snapshot-cloning cost. Final lane 1f86e81b3, tested runtime 343aae434; ready for combined integration |
| Local controller (new PF-20-S03) | 0 / 5 | New native PF20 dependency; consolidate source/tests before requesting Astra #1 and Fable #2. No old-lane ledger reset |

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

A thread heartbeat checks every five minutes, resumes actionable stalled work,
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

## Rolling implementation dispatch

The user's subsequent instruction authorizes filling agent slots continuously
until a substantial human-testable checkpoint. The design-only assignment above
has now graduated to PF-30-S04, a separate denial-only contract based solely on
completed PF-22-S02. It rejects protected raw-rollout sends independently of
unfinished ingress screening; S01/S02 full provenance gates remain unchanged.
The plan and sprint record define its exact memory-dispatch worktree/base/scope.
client.rs and session/mod.rs transfer exclusively from S01 to S04.

Three current implementation reservations: PF-27-S04 broker service composition,
PF-30-S01 remaining native ingress/segmentation, PF-30-S04 memory dispatch.
Root owns shared registration, sequential builds and combined integration.
Broker/provenance retain their prior review ledgers (4/5 and 3/5); do not
silently reset them. Memory is a new scope baseline (0/5). Agents consolidate
implementation/tests before requesting the next numbered review; if mandatory
review coverage would exceed a remaining budget, report it before invocation.
No privileged service installation or existing Vault migration is authorized.
The main-merge choice remains pending; keep checkpoints pushed to scoped branches.

Rolling coordination checkpoint: root registered the broker service workspace
in lane commit 2a6cfb3d0 and the memory public-host/TMUX suites in 72c210c5f.
Both workers resumed RTX qualification. The broker's new service stage needs
fresh review coverage while its original ledger has only one slot; a specific
request for two reviews (six total across both broker stages) is awaiting the
user. Do not exceed the existing cap without an answer. Tests proceed meanwhile.

Next queued technical boundary is the PF-20-owned durable IntegrityRootStore
and authenticated native bootstrap adapter. Current production implementations
are absent. A bare already-open File is not enough to establish controller
ownership, first install versus lost state, rollback rejection and durable CAS.
Do not hide this as a PF-27 fixture extension. Allocate it separately after
its exact native ownership/storage contract and dependencies are verified and
a slot opens; privileged setup remains a distinct approval.

The heartbeat now runs every five minutes to service shared-registration and
handoff requests promptly. Unchanged non-actionable status stays quiet.

Segmentation integrated and pushed at 0266c2db9. Its entire codex-rs tree is
identical to tested source 2a4fb5857 (git diff exit 0); no new compilation was
needed just for this merge. Latest scoped proof: Core provenance 27/27,
content-security 22/22, locked CLI/full formatting and actual-key smoke 1/1.
These results do not claim a fresh full-Core run or production classifier.
The freed provenance agent now prepares the protected audit-root design
read-only, including real PF20 anchor reuse and native ownership requirements;
no new implementation allocation or privileged actions yet.

Broker service source 6bdc84195 separately passes default denial 1/1, actual
synthetic subprocess 6/6, broker/Vault/proxy 338/338 and Core 6/6, plus lock/Bazel
parity. It is not yet merged or externally qualified; production remains
unavailable pending protected-root/bootstrap composition and measured isolation.

## Latest rolling checkpoint — 2026-09-04 evening

The current review ledger above supersedes the historical dispatch counts.
Broker service Astra #5 is clean; Fable #6 remains unapproved. Memory Astra #1
identified a phase-two provider/config pairing regression. Its agent is making
the scoped correction and rerunning final-candidate automated and actual-key
TMUX checks before authorized Fable #2. Neither stage is represented as a
completed sprint or merged final candidate.

Available agents are doing bounded coordination work: provenance identifies
the next dependency-ready sprint, and broker prepares an exact synthetic Linux
installation proposal. These are design assignments, not secretly activated
implementation sprints or additional reviews. The initial dependency inventory
found no unallocated ready security sprint outside the external PF-35 campaign.
Do not weaken a prerequisite merely to occupy an implementation slot.

The [protected audit-root proposal](protected-audit-root-design.md) records the
missing native dependency, reuse of PF-20/PF-41 protocols, enrollment and durable
CAS requirements, and qualification limits. It is not an approved allocation.
A local controller can protect against worker-owned state rollback but cannot
detect rollback of the entire controller disk. Product authority has been asked
whether to investigate hardware-backed protection or build an explicitly staged
local-only dependency while retaining the full protected-mode gate. No TPM
state, privileged principals, permissions, services or real credentials were
changed. Exact privileged actions require separate approval.

## Resolved local-data rollback scope and refill

Travis Good selected protection against restored Corbanu data, not restoration
of an entire machine. The product specification and architecture acceptance now
record that decision. A separately protected local controller checkpoint is the
selected root; no TPM/off-host witness is required. Loss/corruption still denies,
and privileged setup/real credential migration require separate authority.

PF20S03 is allocated to /root/provenance at
/Volumes/CorbanuDrive/Corbanu/worktrees/security-local-anchor on
feat/security-local-anchor, base 601602fa7e53fcb5b41753a0b3607addd45d4415.
PF30S01 returns to draft with frozen integrated source, remaining gates and
5/5 review ledger retained. This is not completion or a prerequisite waiver.
The new sprint owns protected-state and narrow Core anchor adapters only;
root owns workspace/export registration, PF27 owns native broker composition.
All builds remain on RTX under the shared lock.

Memory source 343aae434 passed write 44/44, read 3/3 and actual-key TMUX
four scenarios plus four restarts; unchanged Core source has 11/11 evidence.
Astra's P2 is fixed and Fable confirmed it. The non-blocking P3 allocation cost
is deferred without weakening per-event policy checks. Final pushed lane
1f86e81b3 is merged into integration source 6a6bb029d. Combined RTX qualification
is running after scoped fix/full formatting with zero diff. Core 110/110,
memory-write 44/44 and memory-read 3/3 have passed; immutable CLI and final
actual-key TMUX/UI gates remain pending. No further review requested.

PF20 shared registration is staged in its lane at 695704d8e: workspace member
and alias, Core dependency and Linux-gated Core anchor/test modules. Root
delegated only resulting Cargo.lock/MODULE.bazel.lock generation to the PF20
worker under the shared RTX lock, including Bazel parity checks and shutdown.
The implementation's coherent modules/checkpoints are staged for review; total
diff size must be recorded honestly. This adds no review invocation or privileged
deployment authority.

## Completed memory checkpoint and human-test handoff

PF30S04 is completed and archived after exact combined source 6a6bb029d passed
Core110/110, memory-write44/44, read3/3, UI235/235 and actual-key TMUX4/4.
The latter contains four memory scenarios plus four restarts and the security/
status journeys. Scoped fix/fullfmt, immutable binary hash, governance and clean
source checks passed; [final combined evidence](../../sprints/PF-30-S04/combined-qualification.md)
records the run IDs and synthetic request/output counts.

humanTest.html now points to the exact RTX binary and retains all previous
checkbox keys, adding checks21–23 for assisted disposable memory sessions.
Those fixtures need preparation before human testing; ordinary chat is not
background-memory proof. No human check is marked accepted. The Mac Apps
shortcut and earlier provider baseline are explicitly distinguished.
securityProgress.html reports27/78 archived (35% rounded), two active
reservations and the completed memory lane. The freed slot has no currently
eligible new implementation; PF20 proceeds while broker waits its root and
the already-asked review-budget choice.

Browser QA used an isolated external-drive profile, verified23 checkboxes,
the new memory section layout and updated progress display; no user acceptance
state was changed. Screenshots stay in .codex-work/security-round5.
This documentation-only closeout does not change the qualified codex-rs tree.

## PF-35 external campaign (unchanged)

User explicitly removed PF-35 from the engineering main path.
Its engineering reservation returns to draft under the existing sprint process.
The external dataset campaign is independent and untouched; this is not a
cancellation or completed qualification. Preserve all prior merged evidence.
Corpus, evaluator-custodian report, production signature and N100 measurements
remain required; PF-35-dependent work stays gated until honest archival.
The external operator must reconcile/reallocate before future repository writes;
do not merge external changes over these active lanes without a scope audit.
