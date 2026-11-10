use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::Ordering;
use std::time::Instant;

use sha2::Digest;
use sha2::Sha256;
use tokio::sync::RwLock;
use tokio::task::JoinError;
use tokio_util::either::Either;
use tokio_util::sync::CancellationToken;
use tokio_util::task::AbortOnDropHandle;
use tracing::Instrument;
use tracing::info;
use tracing::instrument;
use tracing::trace_span;

use crate::function_tool::FunctionCallError;
use crate::function_tool::OrCancelToolExt;
use crate::session::session::Session;
use crate::session::step_context::StepContext;
use crate::tools::call_trace;
use crate::tools::context::AbortedToolOutput;
use crate::tools::context::SharedTurnDiffTracker;
use crate::tools::context::ToolCallState;
use crate::tools::context::ToolPayload;
use crate::tools::lifecycle::notify_tool_aborted;
use crate::tools::registry::AnyToolResult;
use crate::tools::registry::ToolArgumentDiffConsumer;
use crate::tools::router::ToolCall;
use crate::tools::router::ToolCallSource;
use codex_direct_tool_timing::ToolAdmission;
use codex_direct_tool_timing::ToolBatchTiming;
use codex_direct_tool_timing::ToolGateTiming;
use codex_history::ResponseItemEnvelope;
use codex_protocol::error::CodexErr;
#[cfg(test)]
use codex_protocol::error::CodexErrorDetails;
use codex_protocol::models::ResponseInputItem;
use codex_protocol::models::ResponseItem;
use codex_protocol::models::ToolResultMetadata;

const MAX_IDENTICAL_TOOL_CALLS_PER_TURN: u8 = 3;
/// Times in a row one exact direct call may run across the model requests of a
/// turn before further repeats are refused without running.
const MAX_CONSECUTIVE_IDENTICAL_TOOL_CALLS: u32 = 3;
/// Times one exact direct call may return the same result within a turn before
/// further repeats are refused. Catches loops that cycle through several calls,
/// which never repeat one call twice in a row.
const MAX_IDENTICAL_TOOL_RESULTS: u32 = 3;
/// Refused repeats after which the turn stops: the model has ignored the
/// refusals and is stuck.
const STOP_TURN_AFTER_REFUSED_REPEATS: u32 = 8;

/// Direct tool calls of a turn whose output may not be recorded yet. When an
/// interrupt hard-aborts a turn that missed the grace period, the abort path
/// records each call's abort output from here, so the model still sees how
/// long the call ran instead of the bare `aborted` placeholder that history
/// normalization would insert (#1467). A call whose tool already finished keeps
/// its real output, so a finished-then-killed call (for example a patch that was
/// applied) is not reported as aborted (D244).
#[derive(Default)]
pub(crate) struct PendingToolCalls {
    calls: std::sync::Mutex<Vec<PendingToolCall>>,
    /// The running turn's file-change tracker, so the abort path can send the
    /// turn's file-change summary as a regular turn end does (D282).
    turn_diff_tracker: std::sync::Mutex<Option<SharedTurnDiffTracker>>,
}

impl std::fmt::Debug for PendingToolCalls {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingToolCalls")
            .field("calls", &*self.lock())
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
struct PendingToolCall {
    call: ToolCall,
    started: Instant,
    finished_output: Option<ResponseItem>,
}

impl PendingToolCalls {
    fn register(&self, call: &ToolCall, started: Instant) {
        self.lock().push(PendingToolCall {
            call: call.clone(),
            started,
            finished_output: None,
        });
    }

    /// Keeps the output a call's tool returned until the turn records it.
    fn finish(&self, call_id: &str, output: &ResponseItem) {
        if let Some(pending) = self
            .lock()
            .iter_mut()
            .find(|pending| pending.call.call_id == call_id)
        {
            pending.finished_output = Some(output.clone());
        }
    }

    /// Forgets a call once its output is in history.
    pub(crate) fn complete(&self, call_id: &str) {
        self.lock()
            .retain(|pending| pending.call.call_id != call_id);
    }

    /// Takes the outputs of every remaining call: the tool's own output when it
    /// finished, else its abort output timed up to `cancelled_at`.
    pub(crate) fn take_unrecorded_outputs(&self, cancelled_at: Instant) -> Vec<ResponseItem> {
        std::mem::take(&mut *self.lock())
            .into_iter()
            .map(|pending| {
                pending.finished_output.unwrap_or_else(|| {
                    let secs = cancelled_at
                        .saturating_duration_since(pending.started)
                        .as_secs_f32()
                        .max(0.1);
                    ToolCallRuntime::aborted_response(&pending.call, Some(secs))
                        .into_response()
                        .item
                })
            })
            .collect()
    }

    /// Remembers the running turn's file-change tracker (D282).
    pub(crate) fn set_turn_diff_tracker(&self, tracker: &SharedTurnDiffTracker) {
        *self
            .turn_diff_tracker
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::clone(tracker));
    }

    /// The tracker [`Self::set_turn_diff_tracker`] stored, if any.
    pub(crate) fn turn_diff_tracker(&self) -> Option<SharedTurnDiffTracker> {
        self.turn_diff_tracker
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<PendingToolCall>> {
        self.calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

struct ToolCallTimingGuard {
    started_at: Option<Instant>,
    execution_started_at: Arc<OnceLock<Instant>>,
    conversation_id: String,
    turn_id: String,
    call_id: String,
    tool_name: codex_tools::ToolName,
    extensions: Arc<codex_extension_api::ExtensionRegistry<crate::config::Config>>,
}

#[derive(Clone)]
pub(crate) struct ToolCallRuntime {
    session: Arc<Session>,
    // Tool calls may run later, so retain the step whose tool list advertised them.
    pub(crate) step_context: Arc<StepContext>,
    tracker: SharedTurnDiffTracker,
    parallel_execution: Arc<RwLock<()>>,
    tool_call_counts: Arc<RwLock<HashMap<String, u8>>>,
    timing: Option<DirectToolTiming>,
}

#[derive(Clone, Default)]
struct DirectToolTiming {
    gate: ToolGateTiming,
    batch: ToolBatchTiming,
}

// Remove this boundary when CodeMode and subagent dependencies have runtime capture.
fn direct_path_supported(tool_name: &codex_tools::ToolName, source: &ToolCallSource) -> bool {
    !(matches!(source, ToolCallSource::CodeMode { .. })
        || (tool_name.is_default_namespace()
            && matches!(tool_name.name.as_str(),
                crate::tools::code_mode::PUBLIC_TOOL_NAME | crate::tools::code_mode::WAIT_TOOL_NAME))
        // Multi-agent tool namespaces can be overridden by the tool catalog.
        || matches!(tool_name.name.as_str(), "spawn_agent" | "send_input"
            | "close_agent" | "resume_agent" | "wait_agent" | "list_agents"
            | "followup_task" | "interrupt_agent" | "send_message"))
}

pub(crate) trait IntoToolStepContext {
    fn into_tool_step_context(self) -> Arc<StepContext>;
}

impl IntoToolStepContext for Arc<StepContext> {
    fn into_tool_step_context(self) -> Arc<StepContext> {
        self
    }
}

#[cfg(test)]
impl IntoToolStepContext for Arc<crate::session::turn_context::TurnContext> {
    fn into_tool_step_context(self) -> Arc<StepContext> {
        StepContext::for_test(self)
    }
}

impl ToolCallRuntime {
    pub(crate) fn new(
        session: Arc<Session>,
        step_context: impl IntoToolStepContext,
        tracker: SharedTurnDiffTracker,
    ) -> Self {
        let step_context = step_context.into_tool_step_context();
        let timing = step_context
            .turn
            .config
            .features
            .enabled(codex_features::Feature::CriticalPathTiming)
            .then(DirectToolTiming::default);
        Self {
            session,
            step_context,
            tracker,
            parallel_execution: Arc::new(RwLock::new(())),
            tool_call_counts: Arc::new(RwLock::new(HashMap::new())),
            timing,
        }
    }

    pub(crate) fn create_diff_consumer(
        &self,
        tool_name: &codex_tools::ToolName,
    ) -> Option<Box<dyn ToolArgumentDiffConsumer>> {
        self.step_context
            .tool_router
            .create_diff_consumer(tool_name)
    }

    #[instrument(level = "trace", skip_all)]
    pub(crate) fn handle_tool_call(
        self,
        call: ToolCall,
        cancellation_token: CancellationToken,
    ) -> impl std::future::Future<Output = Result<ResponseItemEnvelope, CodexErr>> {
        let call = self.step_context.tool_router.resolve_flat_tool_call(call);
        let error_call = call.clone();
        let source = call.direct_source();
        let pending_tool_calls = Arc::clone(&self.step_context.turn.pending_tool_calls);
        pending_tool_calls.register(&call, Instant::now());
        let recorder = self.session.services.executed_tool_calls.clone();
        let recorded_call = recorder.prepare_direct_call(&call, &source, &self.step_context);
        let step_context = Arc::clone(&self.step_context);
        let call_state = Arc::new(ToolCallState::default());
        let future = self.handle_tool_call_with_source(
            step_context,
            call,
            source,
            cancellation_token,
            Arc::clone(&call_state),
        );
        async move {
            let call_id = error_call.call_id.clone();
            let result = future.await;
            let mut recorded_call =
                recorded_call.filter(|(_, recording)| recording.strong_count() > 0);
            let mut response = match result {
                Ok(result) => {
                    if let Some((call, _)) = recorded_call.as_mut()
                        && let Some(metadata) = result.result.tool_result_metadata()
                    {
                        call.set_tool_result_metadata(ToolResultMetadata::new(metadata));
                    }
                    result.into_response()
                }
                Err(FunctionCallError::Fatal(message)) => return Err(CodexErr::Fatal(message)),
                Err(other) => {
                    ResponseItemEnvelope::new(Self::failure_response(error_call, other).into())
                }
            };
            if let Some(text) = call_state.delivered_assistant_message.get() {
                response
                    .metadata
                    .get_or_insert_default()
                    .delivered_assistant_message = Some(text.clone());
            }
            recorder.attach_direct_call_to_output(&mut response.item, recorded_call);
            pending_tool_calls.finish(&call_id, &response.item);
            Ok(response)
        }
    }

    #[instrument(level = "trace", skip_all)]
    pub(crate) fn handle_tool_call_with_source(
        self,
        step_context: Arc<StepContext>,
        call: ToolCall,
        source: ToolCallSource,
        cancellation_token: CancellationToken,
        call_state: Arc<ToolCallState>,
    ) -> impl std::future::Future<Output = Result<AnyToolResult, FunctionCallError>> {
        let call = step_context.tool_router.resolve_flat_tool_call(call);
        if let Err(err) =
            call_state
                .invocation
                .reserve(&self.session, &step_context.turn, &call.call_id)
        {
            return Either::Left(std::future::ready(Err(err)));
        }
        let message_admission =
            super::user_messaging::admit_code_mode_send(&self.session, &source, &call.tool_name);
        self.session
            .services
            .executed_tool_calls
            .record_tool_call(&call, &source, &step_context);
        let router = &step_context.tool_router;
        let supports_parallel = router.tool_supports_parallel(&call);
        let tool_runtime = router
            .registered_tool(&call.tool_name)
            .map(|tool| Arc::clone(&tool.runtime));
        let finishes_on_cancellation = tool_runtime
            .as_ref()
            .is_some_and(|runtime| runtime.finishes_on_cancellation());
        let router = Arc::clone(router);
        let session = Arc::clone(&self.session);
        let turn = Arc::clone(&step_context.turn);
        let tracker = Arc::clone(&self.tracker);
        let lock = Arc::clone(&self.parallel_execution);
        let invocation_cancellation_token = cancellation_token.clone();
        let started = Instant::now();
        let tool_call_counts = Arc::clone(&self.tool_call_counts);
        let tool_signature = tool_call_signature(&call);
        let guards_repeats = matches!(source, ToolCallSource::Direct)
            && !tool_runtime
                .as_ref()
                .is_some_and(|runtime| runtime.repeated_identical_calls_are_polling());
        if turn
            .config
            .features
            .enabled(codex_features::Feature::CriticalPathTiming)
            && !direct_path_supported(
                &tool_runtime
                    .as_ref()
                    .map_or_else(|| call.tool_name.clone(), |runtime| runtime.tool_name()),
                &source,
            )
        {
            turn.turn_timing_state.invalidate_critical_path();
        }
        let work_timing = self.timing.and_then(|timing| match &source {
            ToolCallSource::Direct | ToolCallSource::DirectPlaintextMessage => {
                let work = turn
                    .turn_timing_state
                    .direct_tool_timings
                    .start_turn_direct_call(
                        &call.call_id,
                        &turn.sub_id,
                        timing.batch,
                        started,
                        turn.turn_timing_state.snapshot_path(started),
                    );
                Some((timing.gate, work))
            }
            ToolCallSource::CodeMode { .. } => None,
        });
        let mut tool_call_timing_guard = ToolCallTimingGuard::capture(
            started,
            &session.thread_id,
            &turn.sub_id,
            &call,
            &source,
            Arc::clone(&session.services.extensions),
        );
        if let Some(guard) = tool_call_timing_guard.as_mut() {
            for observer in session.services.extensions.tool_lifecycle_contributors() {
                observer.on_tool_dispatch(codex_extension_api::ToolDispatchInput {
                    thread_id: &guard.conversation_id,
                    turn_id: &guard.turn_id,
                    call_id: &guard.call_id,
                    tool_name: &guard.tool_name,
                });
            }
        }
        let execution_started_at = tool_call_timing_guard
            .as_ref()
            .map(|timing| Arc::clone(&timing.execution_started_at));
        let abort_session = Arc::clone(&session);
        let abort_source = source.clone();
        let abort_turn = Arc::clone(&turn);
        let dispatch_call_state = Arc::clone(&call_state);
        let dispatch_call = call.clone();
        let thread_id = session.thread_id;
        let trace_source = match &source {
            ToolCallSource::Direct | ToolCallSource::DirectPlaintextMessage => {
                call_trace::Source::Direct
            }
            ToolCallSource::CodeMode { .. } => call_trace::Source::CodeMode,
        };
        let dispatch_tool_name = call.tool_name.clone();
        let dispatch_call_id = call.call_id.clone();

        // Code-mode callbacks can resume outside the turn's local span ancestry.
        let dispatch_span = trace_span!(
            "dispatch_tool_call_with_code_mode_result",
            otel.name = %call.tool_name,
            tool_name = %call.tool_name,
            thread.id = %session.thread_id,
            conversation.id = %session.thread_id,
            turn.id = %turn.sub_id,
            turn_id = %turn.sub_id,
            tool_source = trace_source.as_str(),
            call_id = call.call_id.as_str(),
            codex.tool.phase = "dispatch",
            aborted = false,
        );
        let abort_dispatch_span = dispatch_span.clone();

        let mut dispatch_handle = AbortOnDropHandle::new(tokio::spawn(
            async move {
                let _message_admission = message_admission?;
                let identical_count = {
                    let mut counts = tool_call_counts.write().await;
                    let count = counts.entry(tool_signature.clone()).or_insert(0);
                    *count = count.saturating_add(1);
                    *count
                };
                if identical_count > MAX_IDENTICAL_TOOL_CALLS_PER_TURN {
                    return Err(FunctionCallError::RespondToModel(format!(
                        "repeated identical tool call stopped after {MAX_IDENTICAL_TOOL_CALLS_PER_TURN} attempts: {}. Change approach; do not retry the same tool payload again.",
                        dispatch_call.tool_name
                    )));
                }
                if guards_repeats {
                    let repetition = turn.record_direct_tool_call(&tool_signature).await;
                    if let Some(refusal) = repeated_call_refusal(&dispatch_call, repetition) {
                        let refusals = turn.record_repeated_tool_call_refusal();
                        if refusals >= STOP_TURN_AFTER_REFUSED_REPEATS {
                            return Err(FunctionCallError::Fatal(format!(
                                "stopped the turn after refusing {refusals} repeated tool calls; \
                                 the model kept repeating calls whose results could not change \
                                 (last: `{}`)",
                                dispatch_call.tool_name
                            )));
                        }
                        return Err(FunctionCallError::RespondToModel(refusal));
                    }
                }
                // Readiness and admission remain cancellable even when the handler must finish.
                let (guard, admission_started) = async {
                    if let Some(tool_runtime) = tool_runtime
                        && let Some(readiness) = tool_runtime.wait_until_ready(&session)
                    {
                        readiness
                            .instrument(trace_span!("tool_dispatch.wait_until_ready"))
                            .await;
                    }

                    let admission_started = Instant::now();
                    let guard = if supports_parallel {
                        Either::Left(
                            lock.read()
                                .instrument(trace_span!("tool_dispatch.parallel_lock"))
                                .await,
                        )
                    } else {
                        Either::Right(
                            lock.write()
                                .instrument(trace_span!("tool_dispatch.exclusive_lock"))
                                .await,
                        )
                    };
                    (guard, admission_started)
                }
                .or_cancel_tool(&invocation_cancellation_token)
                .await?;
                // Publish the completed path before releasing the admission lock,
                // including when cancellation drops the handler future.
                let _admitted_timing = work_timing.map(|(gate, work)| {
                    gate.admit(
                        work,
                        if supports_parallel {
                            ToolAdmission::Parallel
                        } else {
                            ToolAdmission::Exclusive
                        },
                        admission_started,
                    )
                });
                // Admission through the parallel-execution gate marks the end
                // of dispatch waiting and the start of handler execution.
                if let Some(execution_started_at) = execution_started_at {
                    let _ = execution_started_at.set(Instant::now());
                }

                let result = router
                    .dispatch_tool_call_with_state(
                        session,
                        step_context,
                        invocation_cancellation_token,
                        tracker,
                        dispatch_call,
                        source,
                        Arc::clone(&dispatch_call_state),
                    )
                    .await;
                drop(_admitted_timing);
                drop(guard);
                if guards_repeats && let Some(identity) = result_identity(&result) {
                    turn.record_direct_tool_result(tool_signature, identity)
                        .await;
                }
                let result = match result {
                    Err(FunctionCallError::MalformedToolCall {
                        diagnostic,
                        message,
                    }) => {
                        let signature = malformed_tool_call_signature(&diagnostic);
                        let count = turn.record_malformed_tool_call(signature).await;
                        if count > 1 {
                            Err(FunctionCallError::Fatal(format!(
                                "stopped the turn after {count} equivalent malformed tool calls \
                                 for `{}` (category={}); the model did not correct the tool payload \
                                 after one retry",
                                diagnostic.tool, diagnostic.category
                            )))
                        } else {
                            Err(FunctionCallError::MalformedToolCall {
                                diagnostic,
                                message,
                            })
                        }
                    }
                    other => other,
                };
                // The sampling loop collects results in order only after its stream ends.
                // Record readiness here, before either caller encodes or collects the result.
                // A fatal error still propagates to the caller instead of producing a tool
                // result; unlike a normal tool failure, it has no readiness event.
                if !matches!(&result, Err(FunctionCallError::Fatal(_))) {
                    call_trace::result_ready(
                        thread_id,
                        &turn.sub_id,
                        &dispatch_tool_name,
                        &dispatch_call_id,
                        trace_source,
                        started.elapsed(),
                    );
                }
                // Finished calls release their IDs even if their results have not been collected.
                // Otherwise the outer task may still need the ID for its abort callback.
                if dispatch_call_state
                    .terminal_outcome_reached
                    .load(Ordering::Acquire)
                {
                    drop(dispatch_call_state.invocation.take());
                }
                result
            }
            .instrument(dispatch_span.clone().or_current()),
        ));

        let completion = async move {
            let mut tool_call_timing_guard = tool_call_timing_guard;
            let join_error = |err| {
                drop(call_state.invocation.take());
                Self::tool_task_join_error(err)
            };
            let mut result = tokio::select! {
                res = &mut dispatch_handle => res.map_err(join_error)?,
                _ = cancellation_token.cancelled() => {
                    // Dynamic handlers observe the token and publish their terminal item themselves.
                    if finishes_on_cancellation
                        || call_state.terminal_outcome_reached.load(Ordering::Acquire)
                        || dispatch_handle.is_finished()
                    {
                        dispatch_handle.await.map_err(join_error)?
                    } else {
                        let secs = started.elapsed().as_secs_f32().max(0.1);
                        abort_dispatch_span.record("aborted", true);
                        dispatch_handle.abort();
                        match dispatch_handle.await {
                            Ok(result) => result,
                            Err(err) if err.is_cancelled() => {
                                let _reservation = call_state.invocation.take();
                                let response = Self::aborted_response(&call, Some(secs));
                                call_trace::result_ready(
                                    thread_id,
                                    &abort_turn.sub_id,
                                    &call.tool_name,
                                    &call.call_id,
                                    trace_source,
                                    started.elapsed(),
                                );
                                call_state.terminal_outcome_reached.store(true, Ordering::Release);
                                notify_tool_aborted(
                                    abort_session.as_ref(),
                                    abort_turn.as_ref(),
                                    call.call_id.as_str(),
                                    &call.tool_name,
                                    abort_source.clone(),
                                )
                                .await;
                                Ok(response)
                            }
                            Err(err) => Err(Self::tool_task_join_error(err)),
                        }
                    }
                },
            };
            let _reservation = call_state.invocation.take();
            // Cancellation outside the handler can return before the registry reports an outcome.
            if cancellation_token.is_cancelled()
                && !call_state
                    .terminal_outcome_reached
                    .swap(true, Ordering::AcqRel)
            {
                notify_tool_aborted(
                    abort_session.as_ref(),
                    abort_turn.as_ref(),
                    call.call_id.as_str(),
                    &call.tool_name,
                    abort_source,
                )
                .await;
                result = Ok(Self::aborted_response(
                    &call,
                    Some(started.elapsed().as_secs_f32().max(0.1)),
                ));
            }
            // Use one completion measurement for logging and response formatting.
            // Measuring inside a handler would omit routing and output processing.
            if let Some(timing) = tool_call_timing_guard.as_mut()
                && let Some(handler_duration_ms) = timing.finish()
                && let Ok(result) = &mut result
            {
                result.result.set_handler_duration_ms(handler_duration_ms);
            }
            result
        }
        .instrument(dispatch_span.or_current());
        Either::Right(completion)
    }
}

/// Returns the refusal for a direct call that cannot tell the model anything
/// new: it already ran `MAX_CONSECUTIVE_IDENTICAL_TOOL_CALLS` times in a row, or
/// it already returned the same result `MAX_IDENTICAL_TOOL_RESULTS` times this
/// turn. Re-running a command after an edit that changes its result is
/// unaffected.
fn repeated_call_refusal(
    call: &ToolCall,
    repetition: crate::session::turn_context::ToolCallRepetition,
) -> Option<String> {
    if repetition.consecutive_calls > MAX_CONSECUTIVE_IDENTICAL_TOOL_CALLS {
        return Some(format!(
            "Not run: this exact `{}` call has already run {MAX_CONSECUTIVE_IDENTICAL_TOOL_CALLS} \
             times in a row and its result is in the conversation above. Repeating it will not \
             produce new information. Use that result, or take a different action.",
            call.tool_name
        ));
    }
    if repetition.identical_results >= MAX_IDENTICAL_TOOL_RESULTS {
        return Some(format!(
            "Not run: this exact `{}` call has already returned the same result \
             {MAX_IDENTICAL_TOOL_RESULTS} times in this turn without any new result in \
             between, so running it again will not produce new information. Use the result in \
             the conversation above, or take a different action.",
            call.tool_name
        ));
    }
    None
}

/// Identity of a dispatched call's result for repeat detection, or `None` when
/// the outcome carries no model-visible result.
fn result_identity(result: &Result<AnyToolResult, FunctionCallError>) -> Option<String> {
    let identity = match result {
        Ok(result) => result.result.result_identity(&result.payload).to_string(),
        Err(FunctionCallError::RespondToModel(message)) => format!("error:{message}"),
        Err(_) => return None,
    };
    let digest = Sha256::digest(identity.as_bytes());
    Some(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn tool_call_signature(call: &ToolCall) -> String {
    let mut hasher = Sha256::new();
    hasher.update(call.tool_name.to_string().as_bytes());
    hasher.update(b"\0");
    hasher.update(call.payload.log_payload().as_bytes());
    let digest = hasher.finalize();
    let payload_hash = digest
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("{}:{payload_hash}", call.tool_name)
}

fn malformed_tool_call_signature(diagnostic: &codex_tools::MalformedToolCallDiagnostic) -> String {
    let normalized_excerpt = diagnostic
        .excerpt
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let mut hasher = Sha256::new();
    hasher.update(diagnostic.tool.as_bytes());
    hasher.update(b"\0");
    hasher.update(diagnostic.category.as_bytes());
    hasher.update(b"\0");
    hasher.update(normalized_excerpt.as_bytes());
    let digest = hasher.finalize();
    digest
        .iter()
        .take(16)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

impl ToolCallRuntime {
    pub(crate) fn batch_timing(&self) -> Option<ToolBatchTiming> {
        self.timing.as_ref().map(|timing| timing.batch.clone())
    }

    fn tool_task_join_error(err: JoinError) -> FunctionCallError {
        FunctionCallError::Fatal(format!("tool task failed to receive: {err:?}"))
    }

    fn failure_response(call: ToolCall, err: FunctionCallError) -> ResponseInputItem {
        let message = err.to_string();
        match call.payload {
            ToolPayload::ToolSearch { .. } => ResponseInputItem::ToolSearchOutput {
                call_id: call.call_id,
                status: "completed".to_string(),
                execution: "client".to_string(),
                tools: Vec::new(),
            },
            ToolPayload::Custom { .. } => ResponseInputItem::CustomToolCallOutput {
                call_id: call.call_id,
                name: None,
                output: codex_protocol::models::FunctionCallOutputPayload {
                    body: codex_protocol::models::FunctionCallOutputBody::Text(message),
                    success: Some(false),
                },
            },
            _ => ResponseInputItem::FunctionCallOutput {
                call_id: call.call_id,
                output: codex_protocol::models::FunctionCallOutputPayload {
                    body: codex_protocol::models::FunctionCallOutputBody::Text(message),
                    success: Some(false),
                },
            },
        }
    }

    pub(crate) fn aborted_response(call: &ToolCall, elapsed_secs: Option<f32>) -> AnyToolResult {
        AnyToolResult {
            call_id: call.call_id.clone(),
            payload: call.payload.clone(),
            result: Box::new(AbortedToolOutput {
                message: Self::abort_message(call, elapsed_secs),
            }),
            post_tool_use_payload: None,
        }
    }

    fn abort_message(call: &ToolCall, elapsed_secs: Option<f32>) -> String {
        let Some(secs) = elapsed_secs else {
            return "aborted before dispatch".to_string();
        };
        if call.tool_name.is_default_namespace() && call.tool_name.name == "exec_command" {
            format!("Wall time: {secs:.1} seconds\naborted by user")
        } else {
            format!("aborted by user after {secs:.1}s")
        }
    }
}

impl ToolCallTimingGuard {
    fn capture(
        started_at: Instant,
        conversation_id: &impl std::fmt::Display,
        turn_id: &str,
        call: &ToolCall,
        source: &ToolCallSource,
        extensions: Arc<codex_extension_api::ExtensionRegistry<crate::config::Config>>,
    ) -> Option<Self> {
        // Code-mode calls are nested within a direct code-mode tool call whose
        // timing already includes them. Suppress nested guards so consumers do
        // not mistake overlapping events for independent tool-call latency.
        if !matches!(
            source,
            ToolCallSource::Direct | ToolCallSource::DirectPlaintextMessage
        ) {
            return None;
        }

        Some(Self {
            started_at: Some(started_at),
            execution_started_at: Arc::new(OnceLock::new()),
            conversation_id: conversation_id.to_string(),
            turn_id: turn_id.to_string(),
            call_id: call.call_id.clone(),
            tool_name: call.tool_name.clone(),
            extensions,
        })
    }

    /// Log call timing once and return the handler duration in milliseconds.
    fn finish(&mut self) -> Option<u64> {
        // Taking the start instant keeps normal completion and Drop from logging twice.
        let started_at = self.started_at.take()?;
        let completed_at = Instant::now();
        // Snapshot once so a concurrently-starting dispatch cannot make one
        // event internally inconsistent.
        let execution_started_at = self
            .execution_started_at
            .get()
            .copied()
            .filter(|execution_started_at| *execution_started_at <= completed_at);
        for observer in self.extensions.tool_lifecycle_contributors() {
            observer.on_tool_timing(codex_extension_api::ToolTimingInput {
                thread_id: &self.conversation_id,
                turn_id: &self.turn_id,
                call_id: &self.call_id,
                boundary: codex_extension_api::ToolTimingBoundary::Handler,
                duration: execution_started_at
                    .map(|start| completed_at.duration_since(start))
                    .unwrap_or_default(),
            });
        }
        let duration_ms = |duration: std::time::Duration| u64::try_from(duration.as_millis()).ok();
        let total_duration_ms = duration_ms(completed_at.duration_since(started_at));
        let dispatch_duration_ms = execution_started_at.map_or_else(
            || total_duration_ms,
            |execution_started_at| duration_ms(execution_started_at.duration_since(started_at)),
        );
        let handler_duration_ms = execution_started_at.map_or(Some(0), |execution_started_at| {
            duration_ms(completed_at.duration_since(execution_started_at))
        });

        macro_rules! log_tool_call {
            ($dispatch_duration_ms:expr, $handler_duration_ms:expr, $total_duration_ms:expr) => {
                info!(
                    event.name = "codex.tool_call",
                    trace_id = %codex_otel::current_span_trace_id().unwrap_or_default(),
                    conversation.id = %self.conversation_id,
                    turn_id = %self.turn_id,
                    tool_name = %self.tool_name,
                    call_id = %self.call_id,
                    tool_source = "direct",
                    execution_started = execution_started_at.is_some(),
                    dispatch_duration_ms = $dispatch_duration_ms,
                    handler_duration_ms = $handler_duration_ms,
                    total_duration_ms = $total_duration_ms,
                    "tool call completed"
                );
            };
        }

        match (dispatch_duration_ms, handler_duration_ms, total_duration_ms) {
            (Some(dispatch_duration_ms), Some(handler_duration_ms), Some(total_duration_ms)) => {
                log_tool_call!(dispatch_duration_ms, handler_duration_ms, total_duration_ms);
            }
            _ => {
                log_tool_call!(
                    tracing::field::Empty,
                    tracing::field::Empty,
                    tracing::field::Empty
                );
            }
        }
        handler_duration_ms
    }
}

impl Drop for ToolCallTimingGuard {
    fn drop(&mut self) {
        // Cancellation can drop the outer future before its normal completion path runs.
        let _ = self.finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::time::Duration;

    use crate::session::step_context::StepContext;
    use crate::tools::context::FunctionToolOutput;
    use crate::tools::context::ToolInvocation;
    use crate::tools::registry::CoreToolRuntime;
    use crate::tools::registry::ToolExecutor;
    use crate::tools::registry::ToolRegistry;
    use crate::tools::router::ToolRouter;
    use crate::turn_diff_tracker::TurnDiffTracker;
    use codex_extension_api::ToolCallOutcome;
    use codex_protocol::models::FunctionCallOutputBody;
    use codex_protocol::models::FunctionCallOutputPayload;
    use codex_protocol::openai_models::ToolMode;
    use pretty_assertions::assert_eq;
    use tokio::sync::Notify;
    use tokio::sync::oneshot;
    use tracing_test::internal::MockWriter;

    #[test]
    fn unsupported_handlers_remain_incomplete_after_namespace_resolution() {
        let handler = crate::tools::multi_agent_tool::multi_agent_v2_handler(
            crate::tools::handlers::multi_agents_v2::WaitAgentHandler::default(),
            Some("custom_collaboration"),
            /*description_override*/ None,
            /*parameters_override*/ None,
        );
        let registry = ToolRegistry::from_tools([handler]);
        let registered = registry
            .registered_tool(&codex_tools::ToolName::namespaced(
                "custom_collaboration",
                "wait_agent",
            ))
            .unwrap();
        assert!(!direct_path_supported(
            &registered.runtime.tool_name(),
            &ToolCallSource::Direct
        ));
        for name in [
            crate::tools::code_mode::PUBLIC_TOOL_NAME,
            crate::tools::code_mode::WAIT_TOOL_NAME,
        ] {
            assert!(!direct_path_supported(
                &codex_tools::ToolName::plain(name).with_default_namespace(),
                &ToolCallSource::Direct
            ));
        }
        assert!(direct_path_supported(
            &codex_tools::ToolName::namespaced("mcp", "exec"),
            &ToolCallSource::Direct
        ));
    }

    #[test]
    fn tool_call_timing_guard_ignores_code_mode_source() {
        let buffer: &'static std::sync::Mutex<Vec<u8>> =
            Box::leak(Box::new(std::sync::Mutex::new(Vec::new())));
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .with_max_level(tracing::Level::INFO)
            .with_writer(MockWriter::new(buffer))
            .finish();
        // Keep callsite interest independent of untraced parallel test threads.
        let _untraced = tracing::Dispatch::new(tracing::subscriber::NoSubscriber::default());
        tracing::subscriber::with_default(subscriber, || {
            let call = ToolCall {
                tool_name: codex_tools::ToolName::plain("test_tool"),
                call_id: "call-1".to_string(),
                payload: ToolPayload::Function {
                    arguments: "{}".to_string(),
                },
                encrypted_function_args: None,
                native_call: None,
            };
            let direct_guard = ToolCallTimingGuard::capture(
                Instant::now(),
                &"conversation-id",
                "turn-id",
                &call,
                &ToolCallSource::Direct,
                codex_extension_api::empty_extension_registry(),
            );
            assert!(
                direct_guard.is_some(),
                "direct tool calls should create a timing guard"
            );
            drop(direct_guard);

            let code_mode_guard = ToolCallTimingGuard::capture(
                Instant::now(),
                &"conversation-id",
                "turn-id",
                &call,
                &ToolCallSource::CodeMode {
                    cell_id: "cell-1".to_string(),
                    runtime_tool_call_id: "runtime-call-1".to_string(),
                },
                codex_extension_api::empty_extension_registry(),
            );
            assert!(
                code_mode_guard.is_none(),
                "nested code-mode calls should not create overlapping timing events"
            );
        });

        let logs = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
        assert_eq!(
            logs.lines()
                .filter(|line| line.contains("event.name=\"codex.tool_call\""))
                .count(),
            1,
            "only direct tool calls should emit a timing event; logs:\n{logs}"
        );
    }

    #[test]
    fn output_timing_works_without_logs_and_excludes_dispatch_waiting() {
        tracing::subscriber::with_default(tracing::subscriber::NoSubscriber::default(), || {
            let call = ToolCall {
                tool_name: codex_tools::ToolName::plain("exec"),
                call_id: "call-1".to_string(),
                payload: ToolPayload::Custom {
                    input: "text('ready')".to_string(),
                },
                encrypted_function_args: None,
                native_call: None,
            };
            let started_at = Instant::now() - Duration::from_secs(/*secs*/ 5);
            let mut timing = ToolCallTimingGuard::capture(
                started_at,
                &"conversation-id",
                "turn-id",
                &call,
                &ToolCallSource::Direct,
                codex_extension_api::empty_extension_registry(),
            )
            .expect("model-visible timing must not depend on INFO logging");
            timing
                .execution_started_at
                .set(started_at + Duration::from_secs(/*secs*/ 4))
                .expect("execution start is unset");
            let handler_ms = timing.finish().expect("completed handler duration");
            assert!(u128::from(handler_ms) <= started_at.elapsed().as_millis() - 4_000);
            assert!(handler_ms >= 1_000);
            assert_eq!(timing.finish(), None, "completion is consumed exactly once");
        });
    }

    #[test_case::test_case(false, false; "ordinary_capture_disabled")]
    #[test_case::test_case(false, true; "ordinary_capture_enabled")]
    #[test_case::test_case(true, false; "dynamic_capture_disabled")]
    #[test_case::test_case(true, true; "dynamic_capture_enabled")]
    #[tokio::test]
    async fn cancellation_before_dispatch_admission_logs_dispatch_only_timing(
        dynamic: bool,
        critical_path_timing: bool,
    ) -> anyhow::Result<()> {
        // Create and drop instrumented session resources under the same subscriber.
        // A prior traced test may have installed a different global subscriber.
        let buffer: &'static std::sync::Mutex<Vec<u8>> =
            Box::leak(Box::new(std::sync::Mutex::new(Vec::new())));
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .with_max_level(tracing::Level::INFO)
            .with_writer(MockWriter::new(buffer))
            .finish();
        let _subscriber_guard = tracing::subscriber::set_default(subscriber);

        let (session, mut turn_context) = crate::session::tests::make_session_and_context().await;
        Arc::make_mut(&mut turn_context.config)
            .features
            .enable(codex_features::Feature::RuntimeMetrics)
            .unwrap();
        Arc::make_mut(&mut turn_context.config)
            .features
            .set_enabled(
                codex_features::Feature::CriticalPathTiming,
                critical_path_timing,
            )
            .unwrap();
        if critical_path_timing {
            turn_context.turn_timing_state.enable_critical_path();
        }
        turn_context
            .turn_timing_state
            .mark_turn_started(Instant::now())
            .await;
        let session = Arc::new(session);
        let turn_context = Arc::new(turn_context);
        let tool_name = codex_tools::ToolName::plain("test_tool");
        let handler: Arc<dyn CoreToolRuntime> = if dynamic {
            Arc::new(
                crate::tools::handlers::DynamicToolHandler::new(
                    &codex_protocol::dynamic_tools::DynamicToolFunctionSpec {
                        name: tool_name.name.clone(),
                        description: "test".to_string(),
                        input_schema: serde_json::json!({"type": "object"}),
                        defer_loading: false,
                    },
                )
                .expect("dynamic handler"),
            )
        } else {
            Arc::new(ImmediateHandler {
                tool_name: tool_name.clone(),
            })
        };
        let step_context = StepContext::for_test(Arc::clone(&turn_context));
        let router = Arc::new(ToolRouter::from_parts(
            ToolRegistry::from_tools([handler]),
            Vec::new(),
            ToolMode::Direct,
            BTreeMap::new(),
            /*tool_namespaces_info*/ None,
            &[],
        ));
        let step_context = step_context.with_tool_router_for_test(router);
        let tracker = Arc::new(tokio::sync::Mutex::new(TurnDiffTracker::new()));
        let runtime = ToolCallRuntime::new(session, step_context, tracker);
        let batch = runtime.batch_timing();
        assert_eq!(batch.is_some(), critical_path_timing);
        let execution_gate = Arc::clone(&runtime.parallel_execution);
        let execution_gate_guard = execution_gate
            .try_write_owned()
            .expect("execution gate should be available before dispatch starts");
        let (release_execution_gate_tx, release_execution_gate_rx) = std::sync::mpsc::channel();
        let execution_gate_task = tokio::task::spawn_blocking(move || {
            let _execution_gate_guard = execution_gate_guard;
            release_execution_gate_rx
                .recv()
                .expect("test should release the execution gate");
        });

        let cancellation_token = CancellationToken::new();
        let call = ToolCall {
            tool_name,
            call_id: "call-1".to_string(),
            payload: ToolPayload::Function {
                arguments: "{}".to_string(),
            },
            encrypted_function_args: None,
            native_call: None,
        };
        let response_task =
            tokio::spawn(runtime.handle_tool_call(call, cancellation_token.clone()));
        cancellation_token.cancel();
        tokio::time::timeout(Duration::from_secs(1), response_task)
            .await
            .expect("timed out waiting for cancelled tool response")
            .expect("cancelled tool response task should join")
            .expect("cancelled tool call should produce a response");

        assert_eq!(
            batch.and_then(|batch| batch.completed()).is_some(),
            critical_path_timing,
            "capture requires its own gate and must finish before cancellation returns"
        );
        let logs = String::from_utf8(
            buffer
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        )?;
        let timing_events = logs
            .lines()
            .filter(|line| line.contains("event.name=\"codex.tool_call\""))
            .collect::<Vec<_>>();
        assert_eq!(
            timing_events.len(),
            1,
            "cancelled tool call should emit exactly one timing event; logs:\n{logs}"
        );
        let timing_event = timing_events[0];
        assert!(
            timing_event.contains("execution_started=false"),
            "tool cancelled before admission should not report execution started: {timing_event}"
        );
        assert!(
            timing_event.contains("handler_duration_ms=0"),
            "tool cancelled before admission should report zero handler duration: {timing_event}"
        );
        let duration_field = |name: &str| {
            timing_event.split_whitespace().find_map(|field| {
                field
                    .strip_prefix(&format!("{name}="))
                    .and_then(|value| value.parse::<u64>().ok())
            })
        };
        let dispatch_duration_ms = duration_field("dispatch_duration_ms")
            .expect("timing event should include dispatch_duration_ms");
        let total_duration_ms = duration_field("total_duration_ms")
            .expect("timing event should include total_duration_ms");
        assert_eq!(
            dispatch_duration_ms, total_duration_ms,
            "tool cancelled before admission should attribute all elapsed time to dispatch: {timing_event}"
        );
        release_execution_gate_tx
            .send(())
            .expect("execution gate task should remain available");
        execution_gate_task
            .await
            .expect("execution gate task should join");

        Ok(())
    }

    struct ImmediateHandler {
        tool_name: codex_tools::ToolName,
    }

    impl ToolExecutor<ToolInvocation> for ImmediateHandler {
        fn tool_name(&self) -> codex_tools::ToolName {
            self.tool_name.clone()
        }

        fn spec(&self) -> codex_tools::ToolSpec {
            codex_tools::ToolSpec::Function(codex_tools::ResponsesApiTool {
                name: self.tool_name.name.clone(),
                description: "Immediate test tool.".to_string(),
                strict: false,
                defer_loading: None,
                parameters: codex_tools::JsonSchema::default(),
                output_schema: None,
            })
        }

        fn handle<'a>(&'a self, _invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'a>
        where
            ToolInvocation: 'a,
        {
            Box::pin(async {
                Ok(
                    Box::new(FunctionToolOutput::from_text("ok".to_string(), Some(true)))
                        as Box<dyn crate::tools::context::ToolOutput>,
                )
            })
        }
    }

    impl CoreToolRuntime for ImmediateHandler {}

    struct BlockingFinishContributor {
        records: Arc<std::sync::Mutex<Vec<ToolCallOutcome>>>,
        finish_started: std::sync::Mutex<Option<oneshot::Sender<()>>>,
        allow_finish: Arc<Notify>,
    }

    impl codex_extension_api::ToolLifecycleContributor for BlockingFinishContributor {
        fn on_tool_finish<'a>(
            &'a self,
            input: codex_extension_api::ToolFinishInput<'a>,
        ) -> codex_extension_api::ToolLifecycleFuture<'a> {
            let records = Arc::clone(&self.records);
            let allow_finish = Arc::clone(&self.allow_finish);
            let finish_started = self
                .finish_started
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            let outcome = input.outcome;
            Box::pin(async move {
                if let Some(finish_started) = finish_started {
                    let _ = finish_started.send(());
                }
                allow_finish.notified().await;
                records
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(outcome);
            })
        }
    }

    #[tokio::test]
    async fn cancellation_after_handler_finishes_preserves_completed_lifecycle()
    -> anyhow::Result<()> {
        let (mut session, turn_context) = crate::session::tests::make_session_and_context().await;
        let records = Arc::new(std::sync::Mutex::new(Vec::new()));
        let (finish_started_tx, finish_started_rx) = oneshot::channel();
        let allow_finish = Arc::new(Notify::new());
        let mut builder =
            codex_extension_api::ExtensionRegistryBuilder::<crate::config::Config>::new();
        builder.tool_lifecycle_contributor(Arc::new(BlockingFinishContributor {
            records: Arc::clone(&records),
            finish_started: std::sync::Mutex::new(Some(finish_started_tx)),
            allow_finish: Arc::clone(&allow_finish),
        }));
        session.services.extensions = Arc::new(builder.build());

        let session = Arc::new(session);
        let turn_context = Arc::new(turn_context);
        let tool_name = codex_tools::ToolName::plain("test_tool");
        let handler = Arc::new(ImmediateHandler {
            tool_name: tool_name.clone(),
        }) as Arc<dyn CoreToolRuntime>;
        let step_context = StepContext::for_test(Arc::clone(&turn_context));
        let router = Arc::new(ToolRouter::from_parts(
            ToolRegistry::from_tools([handler]),
            Vec::new(),
            ToolMode::Direct,
            BTreeMap::new(),
            /*tool_namespaces_info*/ None,
            &[],
        ));
        let step_context = step_context.with_tool_router_for_test(router);
        let tracker = Arc::new(tokio::sync::Mutex::new(TurnDiffTracker::new()));
        let runtime = ToolCallRuntime::new(session, step_context, tracker);
        let cancellation_token = CancellationToken::new();
        let call = ToolCall {
            tool_name,
            call_id: "call-1".to_string(),
            payload: ToolPayload::Function {
                arguments: "{}".to_string(),
            },
            encrypted_function_args: None,
            native_call: None,
        };

        let response_task =
            tokio::spawn(runtime.handle_tool_call(call, cancellation_token.clone()));
        tokio::time::timeout(Duration::from_secs(1), finish_started_rx)
            .await
            .expect("timed out waiting for lifecycle notification to start")
            .expect("lifecycle notification should start");
        cancellation_token.cancel();
        tokio::time::sleep(Duration::from_millis(10)).await;
        allow_finish.notify_waiters();

        let response = tokio::time::timeout(Duration::from_secs(1), response_task)
            .await
            .expect("timed out waiting for tool response")
            .expect("tool response task should join")?;
        let expected_response = ResponseInputItem::FunctionCallOutput {
            call_id: "call-1".to_string(),
            output: FunctionCallOutputPayload {
                body: FunctionCallOutputBody::Text("ok".to_string()),
                success: Some(true),
            },
        };
        assert_eq!(
            ResponseItemEnvelope::new(expected_response.into()),
            response
        );

        let actual = records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .drain(..)
            .collect::<Vec<_>>();
        assert_eq!(vec![ToolCallOutcome::Completed { success: true }], actual);

        Ok(())
    }

    struct MalformedHandler {
        tool_name: codex_tools::ToolName,
    }

    impl ToolExecutor<ToolInvocation> for MalformedHandler {
        fn tool_name(&self) -> codex_tools::ToolName {
            self.tool_name.clone()
        }

        fn spec(&self) -> codex_tools::ToolSpec {
            codex_tools::ToolSpec::Function(codex_tools::ResponsesApiTool {
                name: self.tool_name.name.clone(),
                description: "Malformed-call test tool.".to_string(),
                strict: false,
                defer_loading: None,
                parameters: codex_tools::JsonSchema::default(),
                output_schema: None,
            })
        }

        fn handle<'a>(&'a self, _invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'a>
        where
            ToolInvocation: 'a,
        {
            Box::pin(async {
                let diagnostic = codex_tools::MalformedToolCallDiagnostic {
                    tool: "structured_write".to_string(),
                    byte_len: 71,
                    category: "Eof".to_string(),
                    excerpt: "{\"path\":\"runtime/engine.js\",\"mode\":\"overwrite\"".to_string(),
                    finish_reason: None,
                };
                Err(FunctionCallError::MalformedToolCall {
                    message: format!("{diagnostic}"),
                    diagnostic,
                })
            })
        }
    }

    impl CoreToolRuntime for MalformedHandler {}

    fn test_router(registry: ToolRegistry) -> Arc<ToolRouter> {
        Arc::new(ToolRouter::from_parts(
            registry,
            Vec::new(),
            ToolMode::Direct,
            BTreeMap::new(),
            /*tool_namespaces_info*/ None,
            &[],
        ))
    }

    fn test_step(
        turn_context: &Arc<crate::session::turn_context::TurnContext>,
        router: &Arc<ToolRouter>,
    ) -> Arc<StepContext> {
        StepContext::for_test(Arc::clone(turn_context))
            .with_tool_router_for_test(Arc::clone(router))
    }

    #[tokio::test]
    async fn repeated_identical_tool_calls_are_stopped_after_budget() -> anyhow::Result<()> {
        let (session, turn_context) = crate::session::tests::make_session_and_context().await;
        let session = Arc::new(session);
        let turn_context = Arc::new(turn_context);
        let tool_name = codex_tools::ToolName::plain("test_tool");
        let handler = Arc::new(ImmediateHandler {
            tool_name: tool_name.clone(),
        }) as Arc<dyn CoreToolRuntime>;
        let router = test_router(ToolRegistry::from_tools([handler]));
        let tracker = Arc::new(tokio::sync::Mutex::new(TurnDiffTracker::new()));
        let runtime = ToolCallRuntime::new(session, test_step(&turn_context, &router), tracker);
        let cancellation_token = CancellationToken::new();

        for idx in 0..MAX_IDENTICAL_TOOL_CALLS_PER_TURN {
            let response = runtime
                .clone()
                .handle_tool_call(
                    ToolCall {
                        tool_name: tool_name.clone(),
                        call_id: format!("call-{idx}"),
                        payload: ToolPayload::Function {
                            arguments: "{\"cmd\":\"pwd\"}".to_string(),
                        },
                        encrypted_function_args: None,
                        native_call: None,
                        native_call: None,
                        native_call: None,
                    },
                    cancellation_token.clone(),
                )
                .await?;
            let ResponseItem::FunctionCallOutput { output, .. } = response.item else {
                panic!("expected function call output");
            };
            assert_eq!(output.success, Some(true));
        }

        let response = runtime
            .handle_tool_call(
                ToolCall {
                    tool_name,
                    call_id: "call-blocked".to_string(),
                    payload: ToolPayload::Function {
                        arguments: "{\"cmd\":\"pwd\"}".to_string(),
                    },
                    encrypted_function_args: None,
                    native_call: None,
                    native_call: None,
                },
                cancellation_token,
            )
            .await?;
        let ResponseItem::FunctionCallOutput { output, .. } = response.item else {
            panic!("expected function call output");
        };
        assert_eq!(output.success, Some(false));
        let FunctionCallOutputBody::Text(message) = output.body else {
            panic!("expected text output");
        };
        assert!(
            message.contains("repeated identical tool call stopped"),
            "{message}"
        );

        Ok(())
    }

    struct PollingHandler {
        tool_name: codex_tools::ToolName,
    }

    impl ToolExecutor<ToolInvocation> for PollingHandler {
        fn tool_name(&self) -> codex_tools::ToolName {
            self.tool_name.clone()
        }

        fn spec(&self) -> codex_tools::ToolSpec {
            ImmediateHandler {
                tool_name: self.tool_name.clone(),
            }
            .spec()
        }

        fn handle<'a>(&'a self, _invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'a>
        where
            ToolInvocation: 'a,
        {
            Box::pin(async {
                Ok(Box::new(FunctionToolOutput::from_text(
                    "still running".to_string(),
                    Some(true),
                ))
                    as Box<dyn crate::tools::context::ToolOutput>)
            })
        }
    }

    impl CoreToolRuntime for PollingHandler {
        fn repeated_identical_calls_are_polling(&self) -> bool {
            true
        }
    }

    /// One runtime per call, as `run_sampling_request` builds for each model
    /// request of a turn.
    async fn call_in_new_sampling_runtime(
        router: &Arc<ToolRouter>,
        session: &Arc<Session>,
        turn_context: &Arc<crate::session::turn_context::TurnContext>,
        tool_name: &codex_tools::ToolName,
        call_id: String,
        arguments: &str,
    ) -> Result<ResponseItemEnvelope, CodexErr> {
        let tracker = Arc::new(tokio::sync::Mutex::new(TurnDiffTracker::new()));
        ToolCallRuntime::new(
            Arc::clone(session),
            test_step(turn_context, router),
            tracker,
        )
        .handle_tool_call(
            ToolCall {
                tool_name: tool_name.clone(),
                call_id,
                payload: ToolPayload::Function {
                    arguments: arguments.to_string(),
                },
                encrypted_function_args: None,
                native_call: None,
            },
            CancellationToken::new(),
        )
        .await
    }

    fn output_text(response: ResponseItemEnvelope) -> (Option<bool>, String) {
        let ResponseItem::FunctionCallOutput { output, .. } = response.item else {
            panic!("expected function call output");
        };
        let FunctionCallOutputBody::Text(text) = output.body else {
            panic!("expected text output");
        };
        (output.success, text)
    }

    #[tokio::test]
    async fn consecutive_identical_calls_are_refused_across_sampling_runtimes() -> anyhow::Result<()>
    {
        let (session, turn_context) = crate::session::tests::make_session_and_context().await;
        let (session, turn_context) = (Arc::new(session), Arc::new(turn_context));
        let tool_name = codex_tools::ToolName::plain("exec_command");
        let router = test_router(ToolRegistry::from_tools([Arc::new(ImmediateHandler {
            tool_name: tool_name.clone(),
        })
            as Arc<dyn CoreToolRuntime>]));
        let grep = r#"{"cmd":"grep -n kept src/queryforge/__init__.py"}"#;
        let call = |idx: usize, arguments: &'static str| {
            call_in_new_sampling_runtime(
                &router,
                &session,
                &turn_context,
                &tool_name,
                format!("call-{idx}"),
                arguments,
            )
        };

        for idx in 0..3 {
            assert_eq!(
                output_text(call(idx, grep).await?),
                (Some(true), "ok".to_string())
            );
        }
        let (success, message) = output_text(call(3, grep).await?);
        assert_eq!(success, Some(false));
        assert!(
            message.starts_with("Not run: this exact `exec_command` call"),
            "{message}"
        );

        // A different call ends the run, so the same command may run again,
        // as when tests are re-run after an edit.
        let edit = r#"{"cmd":"sed -i s/a/b/ src/queryforge/__init__.py"}"#;
        assert_eq!(output_text(call(4, edit).await?).0, Some(true));
        assert_eq!(output_text(call(5, grep).await?).0, Some(true));
        Ok(())
    }

    #[tokio::test]
    async fn turn_stops_when_refused_identical_calls_continue() -> anyhow::Result<()> {
        let (session, turn_context) = crate::session::tests::make_session_and_context().await;
        let (session, turn_context) = (Arc::new(session), Arc::new(turn_context));
        let tool_name = codex_tools::ToolName::plain("exec_command");
        let router = test_router(ToolRegistry::from_tools([Arc::new(ImmediateHandler {
            tool_name: tool_name.clone(),
        })
            as Arc<dyn CoreToolRuntime>]));
        let arguments = r#"{"cmd":"python3 check.py"}"#;
        let refusals_before_stop = STOP_TURN_AFTER_REFUSED_REPEATS - 1;
        for idx in 1..=(MAX_CONSECUTIVE_IDENTICAL_TOOL_CALLS + refusals_before_stop) {
            let response = call_in_new_sampling_runtime(
                &router,
                &session,
                &turn_context,
                &tool_name,
                format!("call-{idx}"),
                arguments,
            )
            .await?;
            let expected_success = idx <= MAX_CONSECUTIVE_IDENTICAL_TOOL_CALLS;
            assert_eq!(
                output_text(response).0,
                Some(expected_success),
                "call {idx}"
            );
        }
        let Err(err) = call_in_new_sampling_runtime(
            &router,
            &session,
            &turn_context,
            &tool_name,
            "call-final".to_string(),
            arguments,
        )
        .await
        else {
            panic!("the turn must stop after repeated refusals");
        };
        let CodexErrorDetails::Fatal(message) = err.details() else {
            panic!("expected a fatal repeated-call stop, got {err:?}");
        };
        assert!(
            message.contains("refusing 8 repeated tool calls"),
            "{message}"
        );
        Ok(())
    }

    #[tokio::test]
    async fn cycles_of_calls_with_unchanged_results_are_refused() -> anyhow::Result<()> {
        let (session, turn_context) = crate::session::tests::make_session_and_context().await;
        let (session, turn_context) = (Arc::new(session), Arc::new(turn_context));
        let tool_name = codex_tools::ToolName::plain("exec_command");
        let router = test_router(ToolRegistry::from_tools([Arc::new(ImmediateHandler {
            tool_name: tool_name.clone(),
        })
            as Arc<dyn CoreToolRuntime>]));
        let check_a = r#"{"cmd":"python3 check_a.py"}"#;
        let check_b = r#"{"cmd":"python3 check_b.py"}"#;
        let mut idx = 0;
        let mut call = |arguments: &'static str| {
            idx += 1;
            call_in_new_sampling_runtime(
                &router,
                &session,
                &turn_context,
                &tool_name,
                format!("call-{idx}"),
                arguments,
            )
        };

        // A, B, A, B, ...: never twice in a row, and results never change. B's
        // first result was new information, so A's count restarts once; B is
        // refused on lap four and A on lap five.
        for _ in 0..MAX_IDENTICAL_TOOL_RESULTS {
            assert_eq!(output_text(call(check_a).await?).0, Some(true));
            assert_eq!(output_text(call(check_b).await?).0, Some(true));
        }
        assert_eq!(output_text(call(check_a).await?).0, Some(true));
        for arguments in [check_b, check_a] {
            let (success, message) = output_text(call(arguments).await?);
            assert_eq!(success, Some(false));
            assert!(
                message.contains("already returned the same result 3 times"),
                "{message}"
            );
        }

        // A call with a new result is progress, so the checks may run again.
        assert_eq!(
            output_text(call(r#"{"cmd":"apply fix"}"#).await?).0,
            Some(true)
        );
        assert_eq!(output_text(call(check_a).await?).0, Some(true));
        Ok(())
    }

    /// Output that alternates between two states, as when a model toggles an
    /// edit back and forth and re-runs the same test.
    struct TogglingHandler {
        tool_name: codex_tools::ToolName,
        calls: std::sync::atomic::AtomicUsize,
    }

    impl ToolExecutor<ToolInvocation> for TogglingHandler {
        fn tool_name(&self) -> codex_tools::ToolName {
            self.tool_name.clone()
        }

        fn spec(&self) -> codex_tools::ToolSpec {
            ImmediateHandler {
                tool_name: self.tool_name.clone(),
            }
            .spec()
        }

        fn handle<'a>(&'a self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'a>
        where
            ToolInvocation: 'a,
        {
            // Each edit flips the state; the test reports the current state.
            let is_edit = invocation.payload.log_payload().contains("toggle");
            let state = if is_edit {
                self.calls.fetch_add(1, Ordering::SeqCst) + 1
            } else {
                self.calls.load(Ordering::SeqCst)
            } % 2;
            let text = if is_edit {
                "edited".to_string()
            } else {
                format!("state {state}")
            };
            Box::pin(async move {
                Ok(Box::new(FunctionToolOutput::from_text(text, Some(true)))
                    as Box<dyn crate::tools::context::ToolOutput>)
            })
        }
    }

    impl CoreToolRuntime for TogglingHandler {}

    #[tokio::test]
    async fn oscillating_results_are_not_progress() -> anyhow::Result<()> {
        let (session, turn_context) = crate::session::tests::make_session_and_context().await;
        let (session, turn_context) = (Arc::new(session), Arc::new(turn_context));
        let tool_name = codex_tools::ToolName::plain("exec_command");
        let router = test_router(ToolRegistry::from_tools([Arc::new(TogglingHandler {
            tool_name: tool_name.clone(),
            calls: std::sync::atomic::AtomicUsize::new(0),
        })
            as Arc<dyn CoreToolRuntime>]));
        let test_run = r#"{"cmd":"python3 -m unittest"}"#;
        let edit = r#"{"cmd":"toggle the join rename"}"#;
        let mut outcomes = Vec::new();
        for idx in 0..16 {
            let arguments = if idx % 2 == 0 { edit } else { test_run };
            let response = call_in_new_sampling_runtime(
                &router,
                &session,
                &turn_context,
                &tool_name,
                format!("call-{idx}"),
                arguments,
            )
            .await?;
            outcomes.push(output_text(response).0 == Some(true));
        }
        // The first edit/test pairs report states 1 and 0 for the first time;
        // after that the test only returns results it has already returned.
        assert!(outcomes[..4].iter().all(|ran| *ran), "{outcomes:?}");
        assert!(
            outcomes.iter().any(|ran| !ran),
            "toggling between two known results must eventually be refused: {outcomes:?}"
        );
        Ok(())
    }

    #[tokio::test]
    async fn polling_tools_may_repeat_identical_calls() -> anyhow::Result<()> {
        let (session, turn_context) = crate::session::tests::make_session_and_context().await;
        let (session, turn_context) = (Arc::new(session), Arc::new(turn_context));
        let tool_name = codex_tools::ToolName::plain("write_stdin");
        let router = test_router(ToolRegistry::from_tools([Arc::new(PollingHandler {
            tool_name: tool_name.clone(),
        })
            as Arc<dyn CoreToolRuntime>]));
        for idx in 0..(MAX_CONSECUTIVE_IDENTICAL_TOOL_CALLS + STOP_TURN_AFTER_REFUSED_REPEATS + 2) {
            let response = call_in_new_sampling_runtime(
                &router,
                &session,
                &turn_context,
                &tool_name,
                format!("poll-{idx}"),
                r#"{"session_id":7,"chars":""}"#,
            )
            .await?;
            assert_eq!(output_text(response).0, Some(true), "poll {idx}");
        }
        Ok(())
    }

    #[tokio::test]
    async fn equivalent_malformed_calls_stop_across_sampling_runtimes() -> anyhow::Result<()> {
        let (session, turn_context) = crate::session::tests::make_session_and_context().await;
        let session = Arc::new(session);
        let turn_context = Arc::new(turn_context);
        let tool_name = codex_tools::ToolName::plain("structured_write");
        let handler = Arc::new(MalformedHandler {
            tool_name: tool_name.clone(),
        }) as Arc<dyn CoreToolRuntime>;
        let router = test_router(ToolRegistry::from_tools([handler]));
        let tracker = Arc::new(tokio::sync::Mutex::new(TurnDiffTracker::new()));
        let call = |call_id: &str| ToolCall {
            tool_name: tool_name.clone(),
            call_id: call_id.to_string(),
            payload: ToolPayload::Function {
                arguments: "{\"path\":\"runtime/engine.js\",\"mode\":\"overwrite\"".to_string(),
            },
            encrypted_function_args: None,
            native_call: None,
        };

        let first_runtime = ToolCallRuntime::new(
            Arc::clone(&session),
            test_step(&turn_context, &router),
            Arc::clone(&tracker),
        );
        let first = first_runtime
            .handle_tool_call(call("call-1"), CancellationToken::new())
            .await?;
        let ResponseItem::FunctionCallOutput { output, .. } = first.item else {
            panic!("first malformed call should return a corrective tool result");
        };
        assert_eq!(output.success, Some(false));

        // A new runtime represents the next model sampling request in the same user turn.
        let second_runtime =
            ToolCallRuntime::new(session, test_step(&turn_context, &router), tracker);
        let second = second_runtime
            .handle_tool_call(call("call-2"), CancellationToken::new())
            .await;
        let Err(err) = second else {
            panic!("second equivalent malformed call must stop the turn: {second:?}");
        };
        let CodexErrorDetails::Fatal(message) = err.details() else {
            panic!("expected fatal malformed-call guard, got {err:?}");
        };
        assert!(message.contains("after 2 equivalent malformed tool calls"));

        Ok(())
    }

    #[test]
    fn abort_message_reports_wall_time_for_shell_tools() {
        let call = |tool_name| ToolCall {
            tool_name,
            call_id: "call-1".to_string(),
            payload: ToolPayload::Function {
                arguments: "{}".to_string(),
            },
            encrypted_function_args: None,
            native_call: None,
        };
        let secs = Some(1.5);

        assert_eq!(
            "Wall time: 1.5 seconds\naborted by user",
            ToolCallRuntime::abort_message(
                &call(codex_tools::ToolName::plain("exec_command")),
                secs
            )
        );
        assert_eq!(
            "aborted by user after 1.5s",
            ToolCallRuntime::abort_message(
                &call(codex_tools::ToolName::namespaced("mcp", "exec_command")),
                secs
            )
        );
        assert_eq!(
            "aborted by user after 1.5s",
            ToolCallRuntime::abort_message(&call(codex_tools::ToolName::plain("other")), secs)
        );
        assert_eq!(
            "aborted before dispatch",
            ToolCallRuntime::abort_message(&call(codex_tools::ToolName::plain("other")), None)
        );
    }
}

#[cfg(test)]
#[path = "parallel_observation_tests.rs"]
mod observation_tests;

#[cfg(test)]
#[path = "parallel_trace_tests.rs"]
mod trace_tests;
