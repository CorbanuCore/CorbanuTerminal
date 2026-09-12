# Stage-one memory: host-owned policy-bound client

Design/allocation preparation only. No implementation, executable allocation,
build, review invocation, system modification or protected-memory qualification.
Inspected integration source: `62540ca1dcfbf89d2fd2322558167188e6efd1f5`.
Repository: `/Volumes/CorbanuDrive/Corbanu/worktrees/security-round5-20260904`.
The coordinator advanced HEAD to `526926934` during this read-only task; the
`codex-rs` diff from the inspected immutable baseline is empty, verified afterward.

## Recommendation

Add **one opaque, stage-one-only client factory on `CodexThread`**, backed by the
owning Core session. Replace the worker's sole direct `ModelClient::new` call.
The resulting client retains a private owner/lifetime binding and checks it at
every outbound attempt, including lower-level HTTP retries and post-connect
WebSocket sends. Protected, unavailable, mismatched or terminated bindings deny.
Do not expose `EffectivePolicyView`, its controller, a public level setter, an
Allow flag, deserialization or a constructor accepting an arbitrary binding.

This is the smallest coherent security fix, not a one-line constructor change:
construction, transport retries and worker failure persistence are all necessary.
Keep the existing Permissive prompt, output schema and successful persistence
unchanged. Do not supply fake screening or admit a whole serialized rollout as
one trusted fragment.

Implementation class: **product initiative**, under active P0 `/security` levels,
PF-30. Product citation: **Non-negotiable controls** — “Classify instruction intent
and provenance before external content can influence tools or financial actions.”
The accepted follow-up is recorded in PF-30-S02, currently `draft`; see allocation
constraints below. This document does not change that status.

## Verified call path and bounded constructor audit

1. `app-server/src/request_processors/turn_processor.rs:643`: after input submission,
   passes the actual `Arc<CodexThread>`, ID and `thread.config().await` to startup.
2. `memories/write/src/start.rs:23`: preserves ephemeral/disabled MemoryTool/non-root
   skips; unavailable state DB skips. Spawns a pipeline with a startup config.
3. `runtime.rs:69`: `MemoryStartupContext` already holds the actual thread. Its
   `new`, test-only `new_for_testing`, and private `new_with_provider` converge.
4. `phase1.rs`: claims historical rollout jobs, runs them with bounded parallelism,
   loads/serializes rollout content around line298, creates a raw user-message
   prompt, then calls `stream_stage_one_prompt` around line323.
5. `runtime.rs:251`: the **only direct ModelClient constructor in this worker crate**
   currently uses the startup config and no inherited/live binding.
6. `core/src/client.rs:763`: the general public constructor defaults to Permissive
   and no policy. Internal `with_ingress_policy` and `source_admission_level`
   already implement a restrictive floor plus `snapshot_for_agent(thread_id)`.
   Normal Session construction/provider replacement binds this capability;
   the separate worker does not.

The consolidation path calls `ThreadManager::start_thread` in `runtime.rs`, not
this constructor. It is explicitly outside this stage-one client fix. The memory
trace-summarize API and realtime guards are different paths, not evidence that
stage-one is covered. No broad detached-client constructor audit was performed.

Important identity distinction: `claim.thread.id` identifies a **historical source**,
while `MemoryStartupContext.thread_id` identifies the **runtime owner**. Startup
intentionally summarizes other historical threads. Do not reject all legitimate
claims because these differ; equally, never use a claimed source ID to select a
more permissive runtime capability. Persisted source-policy inheritance remains
the broader PF-30-S02 lineage work.

## Proposed public API (exact shape for allocation)

Expose these in a dedicated `codex_core::memory_stage_one` module. The type names
and signatures below are the proposed contract, not existing APIs.

```rust
impl CodexThread {
    pub async fn stage_one_memory_client(
        &self,
        expected_owner: ThreadId,
        expected_provider: &ModelProviderInfo,
    ) -> Result<StageOneMemoryClient, StageOneMemoryError>;
}

// Opaque; private fields; no public constructor, Clone, Deserialize or inner-client getter.
pub struct StageOneMemoryClient { /* Core-owned binding and client */ }

pub struct StageOneMemoryRequest<'a> {
    pub prompt: &'a Prompt,
    pub model_info: &'a ModelInfo,
    pub session_telemetry: &'a SessionTelemetry,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub reasoning_summary: ReasoningSummary,
    pub service_tier: Option<String>,
    pub responses_metadata: &'a CodexResponsesMetadata,
}

pub struct StageOneMemoryOutput {
    pub text: String,
    pub token_usage: Option<TokenUsage>,
}

impl StageOneMemoryClient {
    pub async fn extract(
        &mut self,
        request: StageOneMemoryRequest<'_>,
    ) -> Result<StageOneMemoryOutput, StageOneMemoryError>;

    pub async fn check_completion(&self) -> Result<(), StageOneMemoryError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StageOneMemoryDenial {
    ProtectedInputUnavailable,
    PolicyUnavailable,
    OwnerMismatch,
    OwnerTerminated,
    ProviderChanged,
    KillSwitchActive,
    Cancelled,
}

pub enum StageOneMemoryError {
    Denied(StageOneMemoryDenial),
    Request(CodexErr),
}
```

The request contains ordinary untrusted request data/settings, never authority.
`expected_owner` and `expected_provider` are **assertions to compare against the
host**, not selectors or overrides. A fabricated value can only cause denial.
Use the existing exported types from Core/protocol/otel/model-provider-info;
verify exact imports during implementation. No new wire/schema representation.

`extract` moves the existing text/token accumulation loop from worker runtime to
this cohesive wrapper. It keeps cancellation and typed denial observable through
stream completion; returning the raw inner client or an unchecked stream would
let lifetime checks be detached. The response remains untrusted text, not an
admission capability. No new output interpretation belongs in Core.

## Private ownership and dispatch contract

The factory delegates to a sibling Session implementation, not a new large block
in `session/mod.rs`. It captures:

- `Weak<Session>` pointing to this exact session object (never registry lookup by ID);
- owner ThreadId, policy runtime nonce and session identity from the initialized
  host view; the shared `SessionLoopTermination` observer from `SessionIo`;
- a restrictive configured floor read from the actual Session configuration;
- the provider identity committed for this request's model context;
- a fresh stage-one ModelClient/turn session with isolated ingress registry,
  transport/server-conversation state, and the existing JWT-only memory auth mode.

Do **not** clone the foreground client's native admission registry or WebSocket
history into the memory client. This is a different input source; old admitted
conversation fragments cannot authorize a mixed serialized rollout. Preserve
existing memory metadata/cache-key shaping, compression and proxy configuration.

At construction and each actual send, obtain the current host config and live
`effective_security_policy().snapshot_for_agent(owner)`. Required level is the
maximum of captured restrictive floor, current Session-configured level and live
inherited level. Missing/uninitialized/poisoned view, unknown agent, different
runtime nonce/session identity, dead owner, active kill switch or terminated
owner denies. A Config value alone never proves readiness. A stronger configured
floor survives a live downgrade. A fresh factory may observe an authorized
lower configuration only after the host actually commits it.

### Actual send checks, not just request-build checks

`ModelClient::source_admitted_input` checks at request construction, but:

- HTTP `EndpointSession` applies auth and performs retries through the transport
  below this builder (`codex-api/src/endpoint/session.rs`, telemetry/retry helper).
- WebSocket code builds a request, **awaits connection**, then sends it
  (`core/src/client.rs:3280–3403`). A check before connection is stale afterward.

Use a private Core `StageOneGuardedTransport` implementing the existing
`HttpTransport` RPITIT methods. `ModelClient::build_api_transport` returns a
private concrete wrapper around ReqwestTransport with an optional stage-one
guard. Non-worker clients delegate exactly as before. The worker wrapper checks
after async auth and immediately before each underlying `execute`/`stream`, so
429/5xx/transport retries cannot resend previously serialized raw bytes after a
policy increase. Map a denial to a non-retryable transport error internally,
retain its typed reason in the private request guard, and translate it back into
`StageOneMemoryError::Denied`; do not parse user/provider error strings.

For worker WebSockets, check again after connect and before every response-create
frame, including fallback/new attempt. Either add a narrow guard at the existing
send call or reuse the same private guard helper there; do not disable WebSockets
globally or introduce a new policy provider in lower crates. Scope only the
already-used worker ModelClient paths. All three wire adapters must be covered.

Define dispatch admission at the final guard check. A subsequent increase makes
that attempt in-flight; cancellation is best effort and cannot retract already
sent frames. Do not hold a policy RwLock across network awaits. Test a barrier
between serialization/auth/connect and that final check. A stricter claim of
atomic cancellation of all already-admitted sends would require a separate
controller/transport synchronization contract and is not promised here.

## Worker lifecycle and failure behavior

- Build a fresh request context and factory per job/attempt using current
  `thread.config()`/provider selection, not the startup provider snapshot alone.
  Compare expected provider against host state at factory and dispatch; on
  replacement, deny the old attempt and let a fresh job context choose the new
  model/provider consistently. Never silently send old model context to a new URL.
- Keep the same host owner/nonce across same-thread provider changes; never carry
  a binding to another thread or resurrect one across session restart.
- Re-check during stream consumption and before returning a completed output.
  Select against owning-session termination; discard incomplete/cancelled output.
  A later controller change cannot undo bytes already transmitted.
- Ordinary foreground turn cancellation does not automatically mean cancel all
  independent startup jobs; retain current background behavior unless root opts
  for an explicit worker cancellation token. Session termination must cancel.
- Retain the wrapper through result processing and call `check_completion` directly
  before success persistence. It rechecks the same binding, never changes policy
  and returns no capability. Persistence is not an atomic revocation transaction;
  a controller change after that checkpoint is not claimed to roll back the DB.
- `phase1::job::run` maps Denied to a bounded constant status/reason, not raw
  rollout/provider text. Never call either succeeded function for denial, cancelled
  streams or absent Completed events. Stop that pipeline's further sampling/phase2
  when owner policy is unavailable/protected, without claiming phase2 itself fixed.
- Reuse `mark_stage1_job_failed` and its existing finite retry budget/backoff for
  policy denial; no immediate in-function retry, success/no-output masquerade or
  new database schema. Subsequent startup attempts rebind and reevaluate. Exhausted
  rows retain current retry semantics; root may later design explicit policy-change
  requeue separately. Tests must prove finite attempts and no stored successful
  extraction, rather than promising automatic recovery of an exhausted old job.
- Preserve disabled/ephemeral/non-root/no-DB skips and Permissive payload/success
  behavior. Do not change pruning or enable memories as part of this fix.

## Exact proposed scope and integration ownership

Request these files explicitly; do not implement until committed allocation:

| Owner | Exact files / purpose |
| --- | --- |
| Binding worker | `codex-rs/core/src/memory_stage_one.rs`, `codex-rs/core/src/memory_stage_one_tests.rs` — public opaque facade, private ownership/guard, tests |
| Integration owner | `codex-rs/core/src/lib.rs` — module/export only; `codex-rs/core/src/codex_thread.rs` — factory delegation only |
| Binding worker, sequential | `codex-rs/core/src/client.rs` — optional worker dispatch guard; `codex-rs/core/src/session/memory_stage_one.rs` — host snapshot/factory; `codex-rs/core/src/session/mod.rs` — module declaration |
| Worker adaptation | `codex-rs/memories/write/src/runtime.rs`, `phase1.rs`, `start.rs`, `startup_tests.rs` — remove detached construction, current context, typed outcomes/skip, real worker fixtures |
| Test wiring | `codex-rs/core/tests/suite/memory_stage_one_policy.rs`, `codex-rs/core/tests/suite/mod.rs`; `codex-rs/tui/tests/suite/memory_stage_one_policy.rs`, `codex-rs/tui/tests/suite/mod.rs` |
| Coordinator | New scoped QA path and allocated sprint/plan records; no general documentation or schema edits by worker |

Confirm Core integration helper needs before granting additional files. Existing
Cargo dependencies already include Core in memories/write; expect no new crate
or manifest edges. Verify Bazel source globs when adding modules; explicitly
allocate BUILD/locks only if a real new dependency or compile-data edge is needed.
App-server production startup API already supplies the owner; no mutation there
is presently necessary. Add app-server integration fixture scope only if typed
TUI cannot exercise its real startup path with existing helpers.

The abbreviated worker entries in the table expand exactly to
`codex-rs/memories/write/src/phase1.rs`,
`codex-rs/memories/write/src/start.rs` and
`codex-rs/memories/write/src/startup_tests.rs`; no directory-wide reservation is
requested. The Core facade exposes only the factory/error/request/output types
and completion check above, not the private dispatch guard.

**Overlap:** Core client.rs and session/mod.rs are still owned by PF-30-S01
provenance; shared exports are coordinator-owned. Implement sequentially after
that owner hands off the frozen source and the coordinator reallocates exact
scope. Do not run two agents editing those files. Independent fake-provider/test
fixture preparation can happen read-only now; an executable consumer lane may
start only after a real completed contract freeze, not a draft API promise.

## Qualification matrix (future work; no tests run for this design)

Use real worker startup/job/stream entry points and fake HTTP/WS providers with
synthetic rollout canaries, no production credentials or real user history.

1. Permissive root succeeds; compare exact logical wire prompt/schema and DB result.
2. Configured Moderate/Aggressive, inherited stronger-than-config, unavailable or
   uninitialized policy, wrong expected owner/provider/nonce: zero canary-bearing
   requests and no successful/no-output job record.
3. Pause after enqueue, serialization, auth, and WS connect; increase live policy
   before dispatch. Repeat with 429/5xx retries, unauthorized refresh, WS fallback
   and subsequent jobs. Requests after the change contain no canary.
4. Same-thread provider/model replacement rejects stale context, rebinds safely;
   different-thread binding cannot substitute. Historical source IDs remain data.
5. Strong config floor plus live downgrade still denies; permitted fresh context
   in Permissive preserves existing success.
6. Legacy/mixed/forged-authority rollout under protected owner always rejects;
   no fixture Allow capability or protected-memory success claim.
7. Session termination/cancel/stream EOF before Completed discards output; finite
   backoff/attempt budget and resumed-session fresh binding are demonstrated.
8. Disabled feature/ephemeral/non-root/missing DB creates no model requests.
9. True-TUI TMUX: isolated memory-enabled home, seeded eligible synthetic rollout,
   real turn text and Enter separately; observe success, cancel/exit/restart and
   protected/unavailable denial using the test host's controller fixture. Capture
   visible checkpoints plus fake-provider request counts and DB outcome; do not
   pretend the observation-only `/security` menu can activate protected policy.

Core and worker fixtures must use host-held test controllers, not a production
public setter. Repeat all affected memory suites and Core filters, fix/fullfmt
before final tests, then actual keys on the exact immutable RTX candidate.
Use shared build lock and a fresh TMPDIR; preserve safe artifacts and source/hash.
No reviewers invoked here. Future coordinator budget should normally be one
Astra High and one Fable5.1High review, with the user cap as a ceiling, not target.

## Allocation decisions, residual blockers and effort

Root can choose this technical contract, the finite-backoff disposition, exact
file ownership, sequential integration gate and review numbers within existing
product authority. No secret, token, PF-35 detector, new OS daemon or privileged
setup is needed for synthetic qualification of a **denial-only protected path**.

The current PF-30-S02 dependency on PF-30-S01 is unmet: S01 remains in progress
with production gaps. Do not mark S02 ready merely because PF24 freed a slot.
Root must either finish/archive that prerequisite truthfully, or record a genuine
single-feature contract split/dependency amendment whose accepted prerequisite
is actually complete and whose limited output is this binding/denial contract.
That amendment must not call unfinished persisted lineage or screening complete.
Only then allocate one branch/worktree/base and run governance before code.

Product/privileged decisions are needed only if expanding scope: enabling protected
memory, accepting incomplete lineage, changing background cancellation promises,
selecting persistent policy stores/OS enforcement or real screening infrastructure.
None should be inferred from authorization to close this client-binding gap.

Residual limits: archived source taint/authority may be missing; phase2 propagation,
read/import/export/compaction lineage, protected positive inference and durable
policy-triggered job requeue remain separate work. Core ModelClient's other public
constructors remain outside this bounded audit. No broad security-completion claim.

Estimate: **2–3 engineering days equivalent**, roughly three sequential chunks
(opaque factory/guard; worker lifecycle; canary/actual-key qualification), plus
integration/review latency. Expect 3–5 serialized RTX build/test cycles; the lower
HTTP retry and WS boundary tests are the principal uncertainty. This is larger
than a setter patch but much smaller than full PF-30-S02 persistent memory.
