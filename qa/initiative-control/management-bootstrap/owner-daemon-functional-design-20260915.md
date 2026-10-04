# Owner-daemon increment D: functional qualification proposal

Action: odsup-functional-design-01  
Allocation digest: 3eacfa9101b0ed950ad8e9ea15d725fdefc79f448ab27f71b249a9890db037ac  
Claim: b2d6d5f7-b89d-4006-88a2-5332892707c7  
Author: gpt-6-astra / high, September 15, 2026.  
Status: design proposal only; every case is unexecuted.

## 1. Scope, provenance and role limitation

This is routine private qualification design for the existing increment D bounded reliability fix. Product authority: **Internal delivery control — TO BUILD**, “durable event dispatch, acknowledgments and watchdog” and “Show blockers, rendered sprints, human test plans, machines, run logs and freshness”. Existing parent context is initiative-delivery-control / PF-80-S01; its evidence record reports in_progress. This proposal neither changes that status nor authorizes implementation, activation, repair or shipment.

The dispatched brief was read first and its SHA-256 matched `de1f154a975172c6bd26ad13217d18c3f0d8841856a209f4593e58ff81e8c485`. Then the isolation contract was read, followed by all five increment D records and mutation tables, then the DEC and SLK harness precedents. No product implementation or implementation test file was read. Harness code and implementation/results descriptions WERE read, as expressly required by the brief.

**Design independence limitation:** the same brief's required reading conflicts with the contract's fresh-designer rule: “Freeze all original cases and literal input formatting before any implementation/result disclosure.” This author is independent of implementation and execution, but is not results-blind and cannot truthfully supply a compliant blind-design attestation. Preserve this as a results-informed qualification proposal. Do not set `results_blind: true`, or represent it as the original blind design. The manager must obtain a fresh intent-only design before an unqualified handoff, unless an explicit applicable product-authority amendment resolves this conflict. Preserve both proposals and map every case; do not discard this matrix to reset review history. This task forbids subagents, so none were dispatched. The separate fresh executor and reviewer remain mandatory.

The assignment base exists as `73f262d8b2990ecd560e9466a6e0e96448d708f2`. The brief's received increment exists as `aba3707b0c014b9499b1de941d1fc90f48ac427f`. Read-only HEAD observation was `8f631e08cd2405cf8209fe018567fc32a8d1ade5`; these are different identities. No candidate selection is inferred from HEAD. The manager must pin the exact receiving commit, package and dirty-file manifest before execution and show that it contains the received increment. Old DEC/SLK pins do not qualify this candidate. No builds, tests, provider calls, credentials, live profiles, activation, pushes or repository edits occurred here.

## 2. Reuse and deliberate differences

Reuse DEC's fresh per-case model-only executor; exact literal case/manifest hashes; immutable phase packets; typed bounded actions; actual browser keys and reload; per-call exact prompts and raw outputs; independently measured runtime and package hashes; publication generation and retained sibling health binding; negative parent/child probes; live peer controls; owned-container cleanup by name AND immutable ID; and separate `functional_verdict`, `harness_outcome` and `reconciled` fields. Reuse SLK's separate SUT and actor containers, private persistent synthetic state, allowlisted diagnostic projections, independently captured synthetic transport effects, synthetic public documents, and real candidate export/publication.

Differences that must be explicit in the future harness manifest:

1. Add D-specific fault phases and process observations to SLK. Its current supported five cases and source-informed offline runner do not execute this design or supply independent functional verdicts.
2. Do not inherit SLK's automatic stop/start and synthetic gap-requalification helper. Here automatic recovery MUST be the candidate's bounded supervisor, and holds/unknown arrivals MUST remain. Harness-driven rescue would invalidate the very behavior being tested.
3. Supervision needs a continuing real candidate process, physical child exits and file-only readers, not prepainted static projections. The actor receives exact candidate outputs through fixed read/PTY operations. A diagnostic PTY is labeled diagnostic; it is not a shipped Corbanu TUI. Any claim about a shipped interactive path additionally needs the real packaged launcher/TUI and actual keys.
4. Missing D fault controls, opaque package routing, mediated actor execution and source-free publications are infrastructure prerequisites, not claims that the existing scripts already implement them. Do not patch product predicates, computed status, restart decisions or expected outputs. Implementers can supply fixtures, never new expectations.
5. Live Slack and actual human repair are separate follow-on lanes, never prerequisites to running the offline D matrix. They are not silently dispositioned out of scope or passed.

## 3. Packet, fixture and confinement contract

The manager creates a new private attempt root, never reusing an old attempt. All artifact names below are proposed future receipt members, not assertions that those files currently exist. The operational slack-operator store named in the brief is an exclusion target only: never read, copy, initialize, mount, hash, probe for content or mutate it. Use newly generated synthetic state; do not sanitize a copy of the real store.

### Common fixture F

One synthetic team TSYNTH, app ASYNTH, channel CSYNTH, human USYNTH, binding generation 1; two synthetic decisions D1/D2 with distinct identities; D2 stays pending as a cross-talk control. One explicit bounded foreground listener, with recurrence OFF and no owner-daemon activation. A disposable transport session can be initialized/qualified with clearly synthetic fixture provenance before action; this is not owner-daemon activation or human acceptance. Stop if the actual candidate requires forbidden activation to reach this boundary.

Each variant starts from its own hashed fixture image and fresh actor context, except explicitly paired phases within one case. Preserve state through same-case process restart. Existing-profile variants retain synthetic history and original IDs; fresh-profile variants start empty. Invalid/expired status fixtures never borrow live credentials. The isolated SDK/web/socket service captures physical requests, acceptances and replies outside the dying child. No request hits Slack. Ordinary cases use at most 80 synthetic callback envelopes and three decisions. Timestamp origins, actual monotonic elapsed time and any injected clock are separately labeled.

Only the trusted fixture controller can select a frozen fault ID; the actor can request `advance` for the next predeclared phase, never arbitrary paths, Python, shell, URLs, SQL, clock values, process IDs or repairs. All controls have entry/release acknowledgments independent of the candidate's reported health:

- Physical kill targets the controller's recorded child PID/start identity inside the SUT namespace, never a host PID. SIGKILL yields fixture return code -9; a dedicated child exit fixture also covers return code 17.
- Callback barriers release exact original queued bytes. In case 01, the controller permits real fence-mark admission but interrupts before journal ingress. It may not write expected gap or exit output into the journal.
- Parent/child admission race barriers stop execution immediately before the relevant check, then apply a supported synthetic lifecycle/binding operation. The barrier is an audited external seam around entry, never a replacement for either check. If the packaged interface cannot expose this window, the variant remains blocked pending a faithful fixture.
- Journal failures use a separate lock-holder process for contention and an audited path/FD-scoped filesystem fault facility returning EIO for journal replacement/fsync. Keep independent observation writes usable unless the case explicitly disables both. Fault evidence lists exact targeted synthetic objects, syscall/error/time and successful control writes. A trusted timeout facility delays wait/handshake completion only; it never decides restart eligibility. No general LD_PRELOAD or debugger capability is exposed to the actor.
- A fixed observation barrier after N completed supervisor passes permits exact failure-counter checks without racing a 10 Hz loop. Process exit/reap events are independently recorded by the confined controller. A killed process being absent from a simple PID list is insufficient proof of successful reaping.
- Retention fixtures may seed valid synthetic history before candidate launch, with full byte hashes and separately computed ordered expectations. Candidate code appends and prunes the final event. No post-action fixture write may repair output.
- Exact clock-boundary and write-budget cases require a separately hashed clock/syscall measurement facility confined to this SUT. Inject wall/monotonic clock readings only, never product results. If unavailable, record blocked rather than substituting sleeps or claiming exact 5/6-second boundary coverage. Real-clock backoff remains mandatory separately.

Freeze a neutral observation map before execution: actual CLI command, emitted JSON field path and raw file/publication source for each assertion. `returncode`, fence/ingress count labels and summary counter labels below name semantic quantities; do not guess undocumented command syntax or invent JSON keys. Literal values and the documented keys `fence_gap`, `listener_exits`, `last_listener_exit`, `pending_pointers`, `supervisor_health`, `pending_events`, `event_flush_failures`, `listener_events`, and `listener_events_pruned` must be preserved. Missing fields stay missing and fail/block the applicable claim; a harness must not synthesize them. Record the full original response alongside any extracted row. Phrases such as `state=held` below denote the actual relevant projected state, not necessarily a top-level field with that name.

### Enforced bounds

Retain the DEC image/policy/runtime identities after independent verification; do not silently update them. Each actor, SUT and live canary peer runs nonroot UID/GID 65534, network none, private PID/IPC/mount namespaces, read-only root/package, all capabilities dropped, no-new-privileges, pinned seccomp, 128-task/512-MiB/one-CPU limits and the existing bounded noexec tmpfs. SUT-only writable state remains private across same-case restart; actor mounts only its frozen case, neutral packet and actual allowlisted outputs/assets. It cannot read SUT Python/package source, fixture implementation, raw manager history or host mounts. The source-bearing Python package stays in the inaccessible SUT namespace; actor package identity is conveyed through attested executable/assets measurements. Missing actual process/IPC separation blocks admission.

No Docker socket, host automation socket, agent tools, privileged shell, debugger, ptrace/process-memory capability, cross-case TMUX socket, credential files or inherited secret environment reaches the actor. Use a dedicated short PTY/TMUX socket per run if needed; no default server. The only inference route is an already permitted pinned model-only mediator outside the actor. It admits fixed typed actions and observations, not arbitrary fetch, code or tools. Host transport authentication is outside the actor; its allowed credential access is explicitly distinguished from actor denial. No provider call is authorized or made by this design assignment.

Before EVERY case/variant group, preserve actual actor and fork-child denial probes for source read/list, history/prior findings, synthetic secret canaries, symlink escape, package write, cross-run sockets/process signals/memory, unapproved destination and host-tool escape. Test real access denials against existing synthetic canaries; ENOENT alone is not enough. Real stores/secrets are never probe contents. Verify effective mounts/policy and process identities, successful packet read, exact candidate launch and actual PTY text then separate Enter. The peer must be demonstrably alive before and after cross-namespace checks. Probe SUT children as well. Do not replace probes with declared booleans or sandbox policy queries alone.

## 4. Case index and budgets

Class (a): offline synthetic transports; mediated executor inference is separate from Slack network access. Class (b): actual Slack service, no human response. Class (c): actual human judgment/action. Classification describes the necessary lane, not current runnable status. All cases are currently `not_reached`; missing infrastructure is `blocked`, observed behavior violation is `failed`.

Budgets are per variant, including all its phases, negative controls and finish call. Each row's variant count is fixed; any added replay is separately authorized/recorded by the manager. One fresh executor per variant unless a single paired observation group is explicitly named. Every listed variant retains its own ID (for example OD-D03-child-gap).

| Case | Priority | Purpose | Class | Variants | Hard wall / maximum model calls per variant |
| --- | --- | --- | --- | ---: | --- |
| OD-D01 | P0 | Actual 45/42 incident and durable disclosure | a | 1 | 6 min / 12 |
| OD-D02 | P0 | 1/2/4-second restart bound and one consumer | a | 2 (fresh, 200-history) | 6 min / 12 |
| OD-D03 | P0 | Parent and child admission races | a | 10 (2 windows x 5 hazards) | 5 min / 10 |
| OD-D04 | P0 | Stop, EOF, quiescence and finite deadline | a | 4 | 5 min / 10 |
| OD-D05 | P1 | Restart failure/refusal and legacy projection | a | 2 | 5 min / 10 |
| OD-D06 | P0 | File-only gap and unreadable counts | a | 3 (gap, equal, unreadable) | 5 min / 10 |
| OD-D07 | P0 | Pre-admission refusal remains retryable | a | 2 (pause, temporary hold) | 6 min / 14 |
| OD-D08 | P0 | Dispatched uncertainty and retained receipt | a | 3 | 6 min / 14 |
| OD-D09 | P0 | Pointer identity/cancellation/no-request | a | 4 | 5 min / 10 |
| OD-D10 | P0 | Persistent journal failure/reap/recovery | a | 3 (lock, EIO, invalid envelope) | 6 min / 14 |
| OD-D11 | P0 | Pending evidence cannot starve pointers | a | 1 | 6 min / 14 |
| OD-D12 | P0 | Independent observation and storage outage | a | 5 | 6 min / 12 |
| OD-D13 | P0 | Failed-start survivor exit is not lost | a | 2 (recovery before/after exit) | 6 min / 14 |
| OD-D14 | P1 | Count pruning and cumulative exit accounting | a | 1 | 5 min / 12 |
| OD-D15 | P1 | Byte pruning and irreducible full journal | a | 2 | 5 min / 12 |
| OD-D16 | P1 | Bounded heartbeat writes and immediate transitions | a | 1 paired group | 6 min / 14 |
| OD-D17 | P0 | Publication independently expires cached health | a | 1 paired group | 6 min / 14 |
| OD-D18 | P0 | Recurrence/authority remains OFF | a | 2 (fresh, existing) | 5 min / 10 |
| OD-L01 | Follow-on | Genuine Slack reconnect and one physical post | b | 1 | 10 min / 16; not dispatched |
| OD-H01 | Follow-on | Actual incident disposition/unknown-arrival repair | c | 1 | 10 min response window / 0 automated model calls; not dispatched |

For every offline attempt: no model call after the wall deadline; no per-phase budget reset; guest window <=450 seconds; action <=5 seconds; framed exchange <=10 seconds; bounded CLI <=20 seconds; individual fixture operation <=60 seconds; 65,536-byte model input, 8,192-byte action, 3-MiB observation, 32-MiB stdout, 256-KiB stderr, 128-MiB retained evidence. Failures or truncation of required evidence block completion. Exact 1/2/4 backoff is measured without model latency: the controller logs monotonic spawn boundaries continuously. Phase barriers preserve checkpoints until the actor observes them; they do not renew product deadlines. Do not carry an old healthy cache beyond its real age merely to accommodate inference.

## 5. Frozen proposed cases

Each negative control below is a discriminating measurement or contrasting fixture, not permission to mutate candidate source or replace an expected result. If a control's precondition is absent, the variant is blocked, not passed.

### OD-D01 — reproduce the incident first

Fixture/action: begin F with an established listener and 42 actual synthetic callback ingresses, then admit three additional actual fence marks whose callbacks are interrupted before ingress. Confirm fence=45, ingress=42 without supplying identities/content for the missing arrivals. Kill the owned real child with SIGKILL. Keep a second reader disconnected from the supervisor's foreground pipe throughout. Read standalone status, actual feed projection, resulting status cache and a newly published dashboard health payload. After event persistence, terminate the supervisor and read again from a fresh independent process over the same files. Request only the existing missing-fence inspection, not repair.

Literal observations: exactly one additional `child-exit`; fixture return code `-9`; observed fence `45`, ingress `42`, `fence_gap=3`; `listener_exits=1`; latest exit binds that same epoch and observation; restart disposition `held`; automatic restart count `0`. Status, feed, status file and dashboard health each expose `fence_gap=3` and held state, including aged-cache publication. Missing-fence inspection refuses because the fence exists. Ingress stays `42`; no three invented events, no counter advancement, no requalification, no repaired/healthy overall status. Reopening after durable recording preserves the exit exactly once. Unknown callback content stays unknown.

Negative control: retain independent callback admission/ingress counters and child death receipt so “never started” or a painted badge cannot pass. The paired baseline at 42/42 shows gap 0. Removing exit evidence or disclosing gap 0 would fail the recorded 45/42/3 comparison and reopen check.

Historical precision: the real listener died at 12:22 UTC on September 15 and stayed down approximately two hours. Those are the incident's facts, along with 45/42/3. Its actual death signal/return code is not established by the supplied incident narrative; -9 belongs to this controlled reproduction. Use historical timestamps only as labeled fixture data. A six-minute replay does not prove a two-hour soak, historical cause, delivery or recoverability of any missing arrival. A pass proves bounded incident detection, persistence when writable and independent disclosure for this exact synthetic candidate state. It does not repair the real incident.

### OD-D02 — bounded restart and exclusive consumption

Fixture/action: equal fence/ingress, unchanged active lifecycle, no hold. Kill each of four successive owned real children; continue observing beyond the final backoff window. Run separately with no history and with 200 valid retained historical sessions. Submit one uniquely numbered callback after each successful restart and drain it normally, keeping counts equal.

Literal observations: three automatic restarts maximum after the explicit start; minimum inter-attempt waits `1`, `2`, `4` seconds measured from the applicable exit observation, with no early spawn. With an uncongested 100-ms controller cadence, each eligible restart begins within one second after its due time; unexplained misses are retained failures/timeouts. Four distinct consumer sessions total, maximum simultaneously live consumers `1`; each callback has one physical admission/ingress. After fourth death: `listener_exits=4`, held, no fifth session throughout an additional 8-second observation. History variant: first restart takes history from `200` to `201`, preserves original 200 entries and succeeds through the actual bounded control channel.

Negative control: physical launch/process interval log detects both zero automatic restart and an extra fourth retry, even if status counters lie. Duplicate callback acceptance detects overlapping consumers. History hashes catch shrinking history to fit the frame.

### OD-D03 — reject unsafe recovery in both race windows

Fixture/action: separate variants for stopped lifecycle, changed binding, changed epoch, newly introduced gap 3, and missing fence. For each, kill a safely retryable child; pause before parent revalidation OR after parent admission but before the child's locked session admission; introduce exactly that hazard through the fixture's supported state operation; release. Observe admission requests, actual SDK connections, lifecycle snapshots, journal and all disclosure projections.

Literal observations: no newly admitted consumer/session for the unsafe state; no stopped epoch revival; no lifecycle rewrite undoing the injected change; latest event `restart-refused`, null return code, restart `held`; no later retry after 8 seconds. Original exit remains counted once. New gap variant reports exact `3`; unreadable/missing fence never reports invented zero. Child-side refusal may launch a process that refuses before becoming a consumer; do not count process creation alone as an unsafe consumer.

Negative control: paired unchanged-binding/lifecycle phase is admissible before the hazard; barrier receipts prove the mutation occurred in the designated window. A child that starts and then stops still fails if it admitted a session after the stop/rebind boundary. No-op hazard injection cannot pass.

### OD-D04 — explicit shutdown and unchanged deadline

Fixture/action: live no-gap listener; separate explicit stop, owner-pipe EOF, repair-quiescence WITHOUT invoking repair, and finite-run-deadline variants. The deadline variant has a 6-second original deadline; kill at second 1, allow a restart, then observe beyond second 6. Other variants observe 8 seconds after shutdown.

Literal observations: live consumers `0` after the shutdown boundary; no automatic revival, no second explicit start; intentional stop/EOF/quiescence is not labeled an unexpected child-exit. Deadline remains the original deadline, not six more seconds from restart; no active consumer beyond that boundary plus one controller cadence/cleanup allowance, all timestamps retained. Recurrence stays OFF.

Negative control: verify original consumer was actually alive and accepting a synthetic callback before shutdown. A restarted child with a renewed deadline fails independently of the printed status.

### OD-D05 — refusal stays held; legacy evidence is not rewritten

Fixture/action: (timeout) record a no-gap child death, allow retry, then withhold the synthetic handshake until the real start timeout. Fail the first refusal journal write, then restore it. (legacy) seed a valid prior-format synthetic `restart-refused/pending` event, no active consumer; only read/project/publish.

Literal observations: after successful flush, latest event `restart-refused`, null return code, restart `held`; `listener_exits` still counts only actual child exits. No further retry. The refusal cannot disappear behind an older `child-exit/pending`. Legacy status/feed/dashboard display `restart-refused/held`; the original journal bytes/hash do not change from reading, and no restart is performed. Do not label legacy projection normalization a repair.

Negative control: compare last-event identity against last-child-exit identity; an implementation projecting only the older exit fails. Hash comparison detects silently rewriting legacy history.

### OD-D06 — readers retain exact gap and honest unknowns

Fixture/action: three independent existing-profile fixtures: 45/42, 42/42, unreadable fence with ingress 42. No foreground pipe access. Read real standalone status and feed; read the generated status file; publish dashboard health once fresh and once with cached assessment older than 900 seconds, without changing gap input.

Literal observations: gap fixture on all four surfaces `held`, `fence_gap=3`, including stale outer cache; equal fixture `fence_gap=0` (not a blanket claim of overall healthy); unreadable fixture an explicit unknown/null rather than `0`, with no healthy claim based on those unavailable counts. All source fence/journal/unknown-arrival/qualification state is byte-identical before and after projection; only designated observation/output caches may change.

Negative control: baseline fixture must distinguish zero from unknown and three. Independent counts, unchanged source hashes and retained publisher bytes defeat a hardcoded held badge, fake gap and projection-as-repair.

### OD-D07 — pre-admission refusal does not poison a retry

Fixture/action: one retained approved follow-up request P with parent/details already sent; no prior attempt/receipt for P. On first supervised pointer pass, introduce pause or temporary transport hold before dispatch. Observe, end/restart the foreground supervisor over the same files, then restore only the named temporary admissibility condition through its permitted synthetic operation. Never clear an uncertainty or the case-01 gap.

Literal observations at refusal: P remains `pending`, `pending_pointers=1`, physical P requests `0`, no dispatched attempt/uncertainty created. After legitimate readmission: one physical P request and acceptance; P becomes `sent`, pending count `0`. Additional supervised passes and another supervisor restart produce no second P request. Original request and client/request identity remain unchanged.

Negative control: remote transcript must show zero before admission and exactly one afterward. A slot incorrectly marked uncertain on refusal fails even if nothing duplicate is sent; a helper that never retries fails the eventual one-post checkpoint.

### OD-D08 — dispatched uncertainty is fail-closed across restart

Fixture/action: retained P as above; variants (1) remote accepts but response is lost before receipt, (2) dispatch is recorded but remote never accepts/no receipt, (3) durable exact receipt exists but slot still awaits sent bookkeeping. Interrupt the owned posting process at the named boundary, reopen same files, perform supervised retries and another restart.

Literal observations: variants 1/2 retain fail-closed uncertainty; physical P attempts remain exactly `1`, remote acceptance counts respectively `1`/`0`; no automatic retransmission or fabricated receipt, regardless of endpoint recovery. Variant 3 reuses the retained receipt, marks P sent, and leaves physical attempts/acceptances exactly `1`. Keep pending/uncertain user disclosure; never call uncertainty a retryable pre-admission refusal.

Negative control: distinguish physical requests, remote acceptances, durable attempts and local receipts independently. A zero-acceptance uncertain attempt still cannot be safely inferred retryable. Duplicate posts fail even if the service deduplicates the displayed message.

### OD-D09 — retry only the retained authorized pointer

Fixture/action: four variants: retained request absent, cancelled notice, changed binding, reconstructed request differs from retained bytes. A second properly retained pointer Pgood is the positive control in an otherwise admissible state; isolate it if the hazard is global.

Literal observations: bad target physical posts `0`, no request synthesized, no cancellation reversal and no unapproved pointer. Pgood posts exactly once when its own unchanged context is admissible. Preserve original bad-target request/history, with honest pending/held disclosure where applicable.

Negative control: independently compare sent request bodies/IDs to the retained approved request. A global no-op retry helper fails Pgood; iterating sent or requestless slots fails the zero-post constraint.

### OD-D10 — persistent journal failure must not starve reaping

Fixture/action: equal counts, healthy foreground supervisor. Kill its real child while holding the journal lock, injecting persistent journal EIO, or supplying a deliberately invalid synthetic journal envelope. Continue three completed supervisor passes; include an initial reap timeout then permit actual wait/reap. Read foreground status and file-only outputs. Restore the original valid storage bytes/availability without adding event evidence; keep the supervisor alive and resume.

Literal observations while blocked: owned dead child is reaped; no duplicate exit observation; `supervisor_health=unhealthy`, `pending_events=1`, and `event_flush_failures=3` at the barrier after exactly three failed flushes. No retry budget consumed and no new listener before the event becomes durable. The invalid-envelope variant may have unavailable ordinary journal projection; it must not invent a healthy journal. On recovery: one durable exit with original return code/counts/time, zero pending and failure count reset to 0; bounded restart resumes only if all safety checks remain valid.

Negative control: independently recorded completed passes, wait/reap receipt and process state detect early-return starvation. No killed-but-unreaped zombie can count as pass. Comparing recovery journal length catches repeated death records. Crash durability of an event whose first write never succeeds is expressly not claimed.

### OD-D11 — pointer progress behind unflushed refusal

Fixture/action: a failed-start surviving consumer with one pending refusal, journal-event writes persistently failing but pointer transport and pointer store writes usable; one retained P and one requestless follow-up. Three completed supervised passes; also request explicit start while evidence remains pending; then restore only journal availability.

Literal observations: a real pointer admission/check is attempted during the failure interval, observed through a fixed audited call-boundary receipt. If the existing safety gate admits P, exactly one physical post; if the retained refusal makes delivery unsafe, a real admission refusal and P stays pending. Never require bypassing a safety hold merely to prove progress. Requestless target posts 0. Pending refusal is not overwritten; explicit and automatic starts perform no new listener start and consume no additional retry while evidence is pending. Recovery records the original refusal and preserves honest held state.

Negative control: merely showing P pending fails; there must be an actual pointer check/admission receipt during continued flush failure. A helper-call counter without real candidate boundary provenance is insufficient. Deleting pending evidence to permit restart also fails.

### OD-D12 — failure reaches a reader without the pipe

Fixture/action: five independent phases/variants: first failed exit flush with initially no hold; missing observation; mismatched-binding observation; invalid observation; both journal and observation writes unavailable after one healthy observation. Actor never uses foreground status. Real independent interpreter reads standalone status/feed, the actual status cache and freshly published health.

Literal observations: first failure publishes `unhealthy`, reason `event-flush-failed`, `pending_events=1` across file-only surfaces; original durable `listener_exits=0`, `last_listener_exit=null`, journal bytes unchanged. Observation is not a fabricated durable exit. Missing/mismatched/invalid observations are `unknown`. During both-path outage, prior healthy observation is at most temporarily fresh; at age 6 seconds fresh projection/publication reports `unknown`, `observation-stale`. Previously recorded unhealthy observation stays unhealthy even when old. No raw synthetic exception payload is disclosed as a reason.

Negative control: deny the observer any foreground channel and record its separate process identity and opened artifacts. A live response copied into a file does not qualify as candidate file-only projection. Ages 0 and 5 healthy versus 6 unknown distinguish real freshness assessment from always-unknown output. Already published static bytes do not change magically without publication; test a new publication and retain old bytes.

### OD-D13 — failed-start survivor death is retained behind refusal

Fixture/action: original child exit recorded, retry spawns a real harmless child but fails handshake; cleanup waits time out while that owned child survives. Retain the refusal. Kill survivor with SIGKILL and allow real reaping. Variant A restores journal before survivor dies; variant B leaves refusal unflushed through death and restores afterward.

Literal observations: final ordered sequence contains original child-exit, one restart-refused, one survivor child-exit; survivor return code `-9`, exact observed fence/ingress counts and original epoch. Exactly two child exits total, refusal not counted as exit; no lost/overwritten event or duplicate survivor observation; reaping succeeds while journal unavailable. Restart remains held and no additional retry is consumed after refusal.

Negative control: verify exact surviving PID/start identity and that it was alive after handshake failure. A fixture that actually killed it during cleanup fails its starting condition. Ordered journal comparison detects dropping the queued exit or replacing refusal with exit.

### OD-D14 — newest 128 and cumulative discarded evidence

Fixture/action: preseed 131 valid, independently enumerated chronological events E001..E131, with child exits only at E001 and E003, and zero prior pruning counters. Trigger one real child-exit E132. Read journal and all exit/discard-count projections, then perform two ordinary supervisor reopens without adding new events.

Literal observations: retained sequence exactly E005..E132, length `128`, newest E132 present; cumulative discarded events `4`, discarded child exits `2`, most recently discarded timestamp equals E004's exact timestamp. `listener_exits=3`, including both discarded exits and E132. Reopens preserve those totals rather than resetting them. Other journal sections and original 127 retained events are unchanged. Full discarded per-event details are intentionally unavailable; summary is not reconstructed history.

Negative control: fixture count is literal 131, independent of the product retention constant. Hash/order/counter comparisons fail both no-pruning and dropping exits from cumulative totals. Treat rejection of an impossible initial fixture as blocked, not a successful pruning test.

### OD-D15 — byte budget and irreducible storage pressure

Fixture/action: byte-pressure variant seeds fewer than 128 events and valid unrelated journal data such that appending a known newest event exceeds the pinned package's independently recorded envelope byte limit. Calculate exact serialized bytes including wrapper/checksum before execution. Ensure deleting oldest events will fit. Second variant makes unrelated sections plus the newest event and required summary alone too large.

Literal observations: first variant succeeds atomically, retains the longest allowable suffix under the actual byte limit including newest event, updates exact cumulative discarded/exit counters and last-discarded timestamp, and leaves unrelated sections unchanged. No intermediate truncated journal is observable. Second variant retains the event as pending and reports unhealthy; no silent dropping of newest event, unrelated data or partial journal write. Byte limit is taken from the frozen package contract, not a newly enlarged harness setting.

Negative control: boundary artifact proves unpruned append exceeds the limit and pruned suffix fits; a tiny ordinary journal cannot pass this case. A trusted concurrent reader records every observed journal hash to detect invalid intermediate states.

### OD-D16 — heartbeat has bounded writes and immediate transitions

Fixture/action: no listener started, recurrence OFF. At a supplied fixed clock, drive 300 real supervisor passes at 10 Hz over seconds 0..29. Count only physical observation-file writes and their file/directory fsyncs with a confined syscall observer. Within second 29, create pending evidence, fail its flush, then recover; perform an identical pass. At second 30 fail one heartbeat write, restore ability within the same second, retry, then duplicate. Finally stop heartbeat and read at age 6.

Literal observations: unchanged idle interval performs exactly `30` observation writes and `60` fsyncs; each second's observation carries that second, not the first timestamp. Pending, failed and recovered transitions each publish immediately: three more writes despite equal second. Duplicate adds `0`. Failed publication is retried at the same timestamp; subsequent identical pass adds `0`. Age 6 is `unknown/observation-stale`. New failure evidence is allowed immediate writes and is not subject to the idle-only count.

Negative control: underlying syscall counts detect 300 writes, a frozen timestamp and hidden duplicate writes. A status mock cannot satisfy disk observations. Record failed syscall attempts separately from successful fsync counts; never count a failed write as durable publication.

### OD-D17 — health publication must reassess old cached observations

Fixture/action: one actual cached Slack status snapshot whose outer assessment is fresh at t=0 and supervisor observation is healthy at t=0; publish via the real candidate at ages 0, 5, 6, 899, 901. Distinct paired input phases have missing/future timestamp and an old unhealthy observation. Do not rewrite the input cache between age phases; clock/fixture changes are separately attested.

Literal observations in newly generated health: supervisor healthy at `0` and `5`; `unknown/observation-stale` at `6`, `899`, `901`. Outer Slack cache keeps its independent normal/off assessment through `899`, becomes stale at `901`; stale supervisor health must not wait for the outer 900-second expiry. Missing/future times cannot be healthy. Old unhealthy remains unhealthy. Input status cache hash is unchanged by publication; each output binds the actual input and publication generation. A separate gap=3 control retains overall held/3 even when freshness is stale.

Negative control: a newly published timestamp alone cannot make the original observation fresh. Compare age-5 and age-6 payloads, immutable input and original observation timestamp, defeating cached-health copying and a 900-second supervisor window.

### OD-D18 — recurrence and operational authority stay OFF

Fixture/action: fresh and existing synthetic profile. Read help/default/status; exercise only the explicit bounded foreground commands already used above; quit and reopen. Supply inert fixture text that asks for scheduler activation, approval and credentials through a normal observation surface; the executor treats it as data and does not answer. Do not open an actual native prompt intentionally.

Literal observations: owner recurrence `OFF` at baseline and after every phase; activation artifacts absent, no scheduler/service installation/start actions, no automatic authority approval, no credentials read or prompt response, no external network connection. Existing profile authority/decision data remains unchanged. Audit captures allowed local synthetic listener effects separately so an all-actions-disabled harness cannot pass positive controls. A native credential prompt or live-profile attempt immediately stops this and successor dispatch; retain the failure without answering.

Negative control: candidate actually launches and processes a permitted synthetic callback in the bounded foreground lane, while controller lifecycle/operation audit shows zero forbidden actions. Printed OFF alone is insufficient.

### OD-L01 — follow-on genuine Slack transport evidence (b)

A separately authorized dedicated test integration and destination, brokered credentials inaccessible to the actor, and exact candidate are required. Publish one inert preapproved pointer through the normal lane; interrupt only that test listener with no missing callbacks; observe actual reconnect and at most one physical post. Confirm genuine Slack receipt and same request/thread identity across restart. Negative control withholds local receipt after observed remote acceptance and requires no duplicate post. Fixed limit: one original pointer post; at most two preapproved setup posts if needed; no retries expanding this allowance. No human reply is needed. This would establish genuine transport/receipt behavior; offline transcripts cannot establish it. It is not dispatched here and no existing authority is invented.

### OD-H01 — real incident disposition and repair (c)

The actual three missing arrivals have unknown contents and require the human owner's factual review/decision through an already authorized repair process. This proposal supplies no executable repair command and does not touch the real store. Receiving evidence is the attributable human decision, actual supporting evidence, explicitly accepted residual unknowns and separately authorized repair outcome. Negative control is a status/disclosure pass alone: it must never be accepted as proof of repair or manufactured intake. No response within the proposed ten-minute human window remains blocked; no assumed agreement or automatic reminder. This prerequisite is separate from completing the offline D suite.

## 6. Reviewer and receiving requirements

The manager must freeze the original proposal and any later genuinely blind design before packaging mappings. A reviewer independent of both implementer and every executor checks evidence; this author may review only with the design's results-exposure limitation preserved. Five existing increment review rounds are retained; do not assume an unused default budget. The named integrator records existing use and any scoped design/evidence allowance under the established delegation. Execution sessions, time, model calls and actual cost are separate from review opinions.

Receiving packet, using existing schema-2 conventions (future artifact names):

1. `intent.json`: action/digest/claim; selected full candidate commit and dirty-source map; actual manager/designer/implementer/executor/reviewer identities; machine/profile/session/launcher/PTY/socket; case and variant ID; literal case/hash; full manifest and phase-plan hashes; fixed time/call/effect budgets and authority references. No fabricated identities.
2. `build-attestation.json`, `runtime-measurements.json`, package manifest: independently compare every included production blob to selected candidate, bind exact executable, SDK/runtime, wrapper, fixture and asset hashes; freeze fault seams separately. Use DEC's tree hash (SHA-256 of UTF-8 `json.dumps(files, sort_keys=True)`) where that format is required; do not substitute another canonicalization. Exact source/runtime verification occurs outside the actor and is not copied as source into its packet.
3. Case-bound `isolation.json`, effective policy/tool inventory, actor/SUT/child probes, synthetic canary provenance, peer-before/after and actual PTY positive controls. Bind actual run/agent/case/design/candidate digests. Include negative access attempts and expected denial identities, not just exit codes. All schema booleans are populated from evidence only.
4. Exact `model-input` and mediated request/stdin records; raw output/error/event bytes, warning classifications, actions requested/executed with sequence/timestamps, actual PTY transcript, screenshots/rendered text with truncation flags, phase transition acknowledgments and raw terminal verdict. CLI transport capture is not upstream provider HTTP/SSE capture; preserve that known gap.
5. Per-phase original standalone CLI response, supervisor response when allowed, safe journal projection, independently hashed synthetic state snapshot, actual status-file bytes, independent supervisor observation bytes, feed output, retained dashboard health, publication attestation/generation/current page identity. Keep raw underlying fields; do not publish only summary booleans. File-only reader access record proves no foreground pipe use.
6. `transport.jsonl`, process lifecycle/wait-reap receipts, bounded syscall/clock/fault logs, admission barrier receipts, physical post attempt/acceptance/receipt ledger, exact before/after request identities and ordered event/pruning calculations. These are collected by the controller and independently checked against actor-observed output, never generated from expected verdicts.
7. `receipt.json`: raw `functional_verdict` separately from `harness_outcome`, `reconciled`, actual budgets/cost and cleanup. Preserve setup failures, timeout, denied controls, incomplete phases and corrected replay ancestry. A successful mechanics run without a real independent actor stays `not_reached` functionally. No automatic inference retries or reused actor context after corrected fixtures.
8. Cleanup: retain last state before teardown; remove only owned named/labeled immutable container IDs and private process groups; successful empty queries by name AND ID establish absence. Record volume/PTY/socket lifecycle. Daemon/query failure is unknown cleanup, never absence. Do not reuse unresolved resources.
9. `design.json` and `results.json` using the existing record template: one entry per original case and separately bound variant; original proposals/access transcripts retained. For THIS proposal `results_blind` cannot be true. Every failed/blocked case remains in the matrix; product-authority scope exclusions need attributable acceptance. `evidence-check.md` contains independent findings and the integrator's final disposition is separately attributable. Later run the existing handoff checker against the exact package; no checker was run here.

Reviewer must verify actual conditions rather than trust an all-green table: physical child death and successful reap; both race windows; real 1/2/4 timing; no overlapping consumers; zero-versus-unknown counts; exact first-unflushed file-only health; ordered refusal/survivor evidence; unchanged original deadline and store; pointer pre-admission versus post-dispatch distinction; separate physical attempts and acceptances; counter accumulation after reopen; observation at original timestamp rather than new publication time; source/credential/network/process denial with positive controls; and no forbidden activation/repair. Check all negative controls were meaningful. A fixture failure is not a feature pass. An evidence reviewer cannot repair missing actions by interpreting expected behavior.

## 7. What qualification establishes and does not establish

A complete, independently confined offline pass can establish these observable supervision, disclosure, persistence, refusal, pointer and resource-bound behaviors for the selected exact package, synthetic fixtures, platform and profiles. It does not require opening or repairing the real operational store or asking the human to perform infrastructure work.

It does not establish historical crash cause, recovery of the three unknown arrivals, a two-hour soak, crash durability while all writes fail, native Keychain/launcher behavior, genuine Slack authentication/delivery, actual human decision/ACK, semantic manager correctness, scheduler/recurrence readiness, TensorCash/Isometric live-repository qualification, benchmark results, named-human acceptance or release readiness. A diagnostic terminal is not true-TUI proof. The source/results exposure of this design is an explicit gate limitation, not an erased fact. No required later gate is waived by passing the offline subset.

## 8. Verified reference inventory

These existing references were opened or checked for existence; no private credential or real-store path is offered as evidence. Repository paths are relative to the assigned worktree. Runtime artifact names elsewhere in this proposal are prospective names only.

- `AGENTS.md`; `.codex/skills/corbanu-terminal-development/SKILL.md`; `docs/corbanu-product-spec.md` (heading above).
- `qa/code-blind-functional/isolated-execution.md`, SHA-256 `f71f21b62c802ffd5a1fb5cace5c84432a0bdc4f65c8a3b0c9f2cd4ea7399e98`.
- `qa/code-blind-functional/README.md`; `qa/code-blind-functional/RECORD_TEMPLATE.md`; `qa/code-blind-functional/check.py`.
- `qa/initiative-control/management-bootstrap/owner-daemon.md`, SHA-256 `4f5c0c69adca12ca0e681220ec9321711af7dd99b8c58a1d32ec38d1d2716a1e` (all five D rounds and mutation evidence read).
- `docs/plans/active/initiative-delivery-control.md`; `docs/sprints/current/initiative-delivery-control/pf-80-s01-delivery-control.md`; `docs/development/test-isolation.md` (existence checked; must be read before any later test campaign).
- `/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-execution.ikteKU/README.md`, SHA-256 `cc40dd3371060b97688eb8b4cb540ff0f4535e0adb1488817389264b57b901e0`.
- `/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-execution.ikteKU/run.py`, SHA-256 `c59d73f5d01df78cf5bec15431a05dd9ce34939588ed0ed36066c6d565d6f3b2`.
- `/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-execution.ikteKU/cases.json`, SHA-256 `e22833935332a63c11b55de07ad2d124452f8694d1190cae1753177a5ce6071e`.
- `/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-slk.2Sn5C2/README.md`, SHA-256 `babfe789d6ce0d1554d30863f93bc73618fad59cb56a1892e221a6e9cc1e5071`.
- `/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-slk.2Sn5C2/cases.json`, SHA-256 `e0688e5b1aafa05334a49505ed13e60c96220981fb690d3ae53f8e9cb6745681`.
- `/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-slk.2Sn5C2/PROPOSAL.md`; `/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-slk.2Sn5C2/run_slk.py`; `/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-slk.2Sn5C2/sut_guest.py`; `/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-slk.2Sn5C2/actor_guest.py` (proposal and targeted harness sections inspected).

## 9. Ten-line summary

1. Brief SHA-256 matched; this task produced design only.
2. First case recreates the real 45-fence/42-ingress/3-unknown incident.
3. SIGKILL -9 is a fixture fact; historical death cause remains unproven.
4. Eighteen offline cases cover restart, disclosure, pointers, failures and pruning.
5. File-only readers must expose failed writes and expire stale healthy observations.
6. Retries preserve refusal-versus-uncertainty and cannot duplicate physical posts.
7. Reuse DEC/SLK confinement, exact-package attestations and honest verdict receipts.
8. Separate live Slack and human repair lanes remain undispatched and unproved.
9. Required results reading makes this a results-informed proposal, not a blind-design attestation.
10. Recurrence stays OFF; fresh execution and independent evidence review remain required.
