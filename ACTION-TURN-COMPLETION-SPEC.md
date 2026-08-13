# Action-Turn Completion Semantics

Status: Implemented and qualified for PFTerminal 0.1.22
Scope: PfTerminal core turn lifecycle and chat-compatible providers
Primary incident: Kimi Code K3 ends an unfinished action turn with
`finish_reason: "stop"`, causing PfTerminal to require a manual `continue`.

## 1. Problem

PfTerminal currently treats a chat provider's `finish_reason: "stop"` as proof
that the user's turn is complete. That assumption is false for Kimi Code K3.
Kimi sometimes emits an ordinary assistant message such as:

> Bit-identical. Now wire it into the bench behind `--fastuni` and measure
> against the library version.

It then returns `finish_reason: "stop"` without making the promised tool call.
PfTerminal emits `TurnComplete`, the TUI becomes idle, and the user must type
`continue`. The same failure has occurred after repository inspection,
implementation progress, test setup, and benchmark progress.

This is not a `/goal` failure. The defect reproduces in an ordinary user turn
when no goal is active. Goal state must not participate in the repair.

The failed boundary is **turn completion**:

- Provider protocol says generation stopped.
- Assistant semantics say the requested action is unfinished.
- PfTerminal currently treats the protocol signal as authoritative.

The repair must determine whether the **user's turn** is complete, not merely
whether one provider response ended.

## 2. Current behavior and code references

### 2.1 Provider stop becomes terminal

The chat-completions adapter maps `stop` to `end_turn: Some(true)` in
[`codex-rs/codex-api/src/endpoint/chat_completions.rs`](codex-rs/codex-api/src/endpoint/chat_completions.rs#L832-L854):

```rust
let end_turn = match finish_reason.as_ref() {
    Some(CompletionFinishReason::Stop) => Some(true),
    // ...
};
```

This mapping accurately describes the provider response. It does **not**
establish that the user's action is complete. The adapter should continue to
report the provider fact; policy belongs in the core turn lifecycle.

### 2.2 Core trusts the provider fact

[`codex-rs/core/src/session/turn.rs`](codex-rs/core/src/session/turn.rs#L2910-L2961)
only requests another sample when `end_turn` is `Some(false)`:

```rust
if let Some(false) = end_turn {
    needs_follow_up = true;
}
```

[`run_turn`](codex-rs/core/src/session/turn.rs#L320-L425) then exits when neither
the provider nor pending user input requests a follow-up:

```rust
if !needs_follow_up {
    // stop hooks, then TurnComplete
}
```

Tool calls already set `needs_follow_up = true` in
[`codex-rs/core/src/stream_events_utils.rs`](codex-rs/core/src/stream_events_utils.rs#L420-L443).
Plain assistant messages do not
([same file](codex-rs/core/src/stream_events_utils.rs#L445-L485)). Therefore,
Kimi can stop immediately before a promised tool call and bypass the existing
continuation path.

### 2.3 A regression test preserves the defect

[`kimi_text_stop_is_terminal_without_extra_inference`](codex-rs/core/tests/suite/chat_provider_turn_lifecycle.rs#L97-L123)
uses this unfinished response:

```rust
const PROGRESS_RESPONSE: &str =
    "I found the relevant code. Next I will add the regression tests.";
```

The test asserts exactly one provider request. That test encodes the broken
product behavior and must be replaced.

### 2.4 Existing continuation limit solves a different problem

[`MAX_SERVER_SIDE_MODEL_CONTINUATIONS`](codex-rs/core/src/session/turn.rs#L149)
caps five consecutive provider-declared continuations, including
length-limited responses. It does not evaluate semantic completion and must
remain separate from the action-completion circuit breaker specified here.

### 2.5 Provider identity is already centralized

Kimi provider identity is defined in
[`codex-rs/model-provider-info/src/lib.rs`](codex-rs/model-provider-info/src/lib.rs#L97-L98),
with provider helpers near
[`ModelProviderInfo::is_kimi_code`](codex-rs/model-provider-info/src/lib.rs#L1193-L1195).
Completion behavior must be exposed as a provider capability there, not as
scattered `is_kimi_code()` branches.

## 3. Required user-visible behavior

### 3.1 Terminal answer

When the user asks for information and the model gives a complete answer,
PfTerminal ends the turn normally:

- No automatic “continue” request.
- No completion-check spinner after the answer.
- No warning.
- The prompt becomes available immediately.

### 3.2 Unfinished action

When the user asked PfTerminal to perform work and a provider stops after an
unfinished progress report:

- PfTerminal keeps the same user turn active.
- PfTerminal asks the model to continue exactly once for that completion
  decision.
- The continuation has the same tools, permissions, working directory, model,
  and turn context.
- PfTerminal does not emit `TurnComplete` between the progress report and the
  continuation.
- The user does not need to type `continue`.
- Productive action may pass through any number of progress boundaries; a
  fixed “three attempts” limit must not terminate productive work.

### 3.3 Completed action

When the model states the requested work is finished and provides the result,
PfTerminal ends the turn:

- A prior tool call does not force an unnecessary continuation.
- A final summary is not followed by a redundant “continue working” request.
- The TUI does not display `Working` after the accepted final response.

### 3.4 Genuine blocker or user decision

When the model cannot proceed without user input or an external state change,
PfTerminal accepts the stop:

- The blocker or question remains visible.
- The input prompt becomes available.
- PfTerminal does not fabricate work or repeatedly ask the model to continue.

### 3.5 Stalled continuation

When repeated continuations make no measurable progress:

- PfTerminal stops the automatic loop.
- It emits one visible warning explaining that the model repeatedly stopped
  without progress.
- It leaves the latest assistant output visible.
- The user may retry, steer, change model, or type `continue`.
- It does not emit a flood of warnings or provider requests.

### 3.6 User steering and cancellation

User input always outranks an automatic completion continuation:

- If input arrives before the continuation request is issued, PfTerminal
  drains that input and does not inject the automatic continuation.
- If the user cancels, the continuation is cancelled with the turn.
- If input arrives while the semantic decision is running, the decision is
  discarded and the user input is processed.
- No late classifier result may restart a cancelled or superseded turn.

### 3.7 Provider and transport failures

- A failed continuation is reported as the real provider/transport error.
- A failed semantic check follows the bounded fallback in Section 7.
- Neither failure is silently converted into `TurnComplete`.
- No error path may leave the TUI spinner active after the core turn has
  stopped.

## 4. Design

### 4.1 Preserve protocol facts; add a core completion decision

Do not change `stop -> end_turn: Some(true)` in the chat adapter. Instead, add
a core decision between `run_sampling_request()` and the `!needs_follow_up`
exit in `run_turn`.

The decision receives:

```rust
struct TurnCompletionInput<'a> {
    provider_semantics: ChatStopSemantics,
    finish_reason: Option<&'a CompletionFinishReason>,
    user_objective: &'a str,
    assistant_text: Option<&'a str>,
    tool_activity: &'a ToolActivitySummary,
    pending_user_input: bool,
    progress_state: &'a CompletionProgressState,
}
```

It returns:

```rust
enum TurnCompletionDecision {
    Accept {
        reason: CompletionReason,
    },
    Continue {
        reason: CompletionReason,
    },
    AwaitUser {
        reason: CompletionReason,
    },
    StopStalled {
        reason: CompletionReason,
    },
}
```

`Accept`, `AwaitUser`, and `StopStalled` end the active turn. `Continue` sets
the model follow-up path without emitting `TurnComplete`.

The concrete Rust names may change, but the inputs, decisions, and ownership
boundary are required.

### 4.2 Centralize provider stop semantics

Add a capability to `ModelProviderInfo`:

```rust
pub enum ChatStopSemantics {
    ReliableTerminal,
    AmbiguousForActionTurns,
}
```

The initial capability assignment is:

| Provider                                       | Stop semantics            |
| ---------------------------------------------- | ------------------------- |
| Kimi Code                                      | `AmbiguousForActionTurns` |
| Existing providers without reproduced evidence | `ReliableTerminal`        |

This is a provider capability, not a hard-coded model slug check. A gateway or
custom provider may opt into the same behavior later without copying Kimi
branches through core.

Reliable providers retain their current lifecycle and incur no additional
requests.

### 4.3 Fast deterministic paths

The arbiter handles facts before semantic classification:

1. Pending user input: do not auto-continue; process the input.
2. Tool/function call or `end_turn: Some(false)`: use the existing follow-up
   path.
3. Content-filtered or unknown finish reason: preserve the existing explicit
   error behavior.
4. Reliable provider stop: accept it.
5. Ambiguous provider stop with no assistant text: continue once; a second
   empty stop is stalled.
6. Ambiguous provider text stop: run the semantic decision in Section 4.4.

String matching such as `"Next I will"`, `"Let me"`, or `"Now..."` must not be
the product decision. Such phrases may appear in tests and diagnostics, but
must not be a production allowlist, regex router, or prompt patch.

### 4.4 One structured semantic decision per ambiguous stop

For an ambiguous provider text stop, use a small structured classifier. It
must evaluate both the user's requested outcome and the latest assistant
response, not isolated keywords.

Required output:

```json
{
  "state": "complete | incomplete | awaiting_user | blocked | uncertain",
  "reason": "short machine-readable explanation"
}
```

Definitions:

- `complete`: the requested answer or action has been delivered.
- `incomplete`: the assistant describes work it has not yet performed, omits
  an explicitly requested deliverable, or stops at a progress checkpoint.
- `awaiting_user`: the assistant asks for information, authorization, or a
  choice required to continue.
- `blocked`: an external condition prevents further useful work and is
  clearly reported.
- `uncertain`: available context cannot distinguish the above.

Mapping:

| Classifier state | Core decision         |
| ---------------- | --------------------- |
| `complete`       | `Accept`              |
| `incomplete`     | `Continue`            |
| `awaiting_user`  | `AwaitUser`           |
| `blocked`        | `AwaitUser`           |
| `uncertain`      | Fallback in Section 7 |

Constraints:

- At most one classifier request per provider stop.
- No “classify, reprompt, classify again” loop.
- The classifier result is not shown as assistant speech.
- It must not mutate conversation history.
- It must not receive secrets, raw tool output, images, or the entire rollout.
- The objective and response are independently length-bounded.
- Structured decoding failure is a classifier failure, not an implicit
  `complete`.
- The classifier should use the provider's lowest-latency supported reasoning
  setting. It must not silently switch to a separately billed provider.

### 4.5 Continuation request

`Continue` adds one request-local control fragment to the next model sample:

> Continue executing the current user request. The previous response was a
> progress checkpoint, not a completed result.

The fragment must:

- Be represented by a typed `ContextualUserFragment`, consistent with the
  context-fragment rule used by
  [`codex-rs/core/src/context`](codex-rs/core/src/context).
- Be request-local and absent from the visible transcript.
- Not be appended repeatedly to durable conversation history.
- Be discarded after the next sample.
- Be cancelled when user steering wins the race.

This is a general turn-lifecycle control message. It must not mention Kimi,
games, mining, repositories, test commands, or the incident wording.

### 4.6 Progress-aware circuit breaker

Do not cap semantic continuations at three or any other small fixed count.
Long implementation turns legitimately cross many progress boundaries.

Track a turn-local `CompletionProgressState` containing:

- Count and identities of completed tool calls.
- Count of newly recorded tool outputs.
- Turn-diff digest or changed-file summary when available.
- Normalized digest of the latest assistant response.
- Count of consecutive incomplete decisions without new progress.

Progress resets the stall counter when at least one of these changes:

- A new tool call completes.
- A new tool output is recorded.
- The tracked diff changes.
- The assistant reaches a materially new checkpoint rather than repeating the
  same response.

Stop as stalled after **two consecutive incomplete decisions with no
measurable progress**. This threshold protects against self-sustaining loops;
it does not limit productive continuations.

Text similarity alone must not establish success. Tool and diff activity are
stronger signals. Similarity is used only to detect repetition.

The existing five-response cap for provider-declared length continuations
remains unchanged and separate.

### 4.7 TUI state

The TUI must distinguish these states:

| Core state                   | TUI behavior                                      |
| ---------------------------- | ------------------------------------------------- |
| Primary provider sample      | Existing `Working` state                          |
| Semantic completion decision | `Checking whether the action is complete…`        |
| Automatic continuation       | `Continuing unfinished action…`                   |
| Accepted final response      | Spinner removed; input prompt active              |
| Stalled                      | One warning; spinner removed; input prompt active |
| Failed                       | Existing error surface; spinner removed           |

The semantic check state must only render while a check is actually in
flight. A completed conversational response must never sit under `Working`.

Prefer existing lifecycle events where they express the state accurately. If
a new event is required, define it in `codex-protocol` and ensure app-server,
TUI, and headless clients all receive a terminal event on every exit path.

## 5. Proposed code structure

Keep the implementation small and separate from the already large
`turn.rs`.

### New module

`codex-rs/core/src/session/turn_completion.rs`

Responsibilities:

- Deterministic routing.
- Structured classifier request and result parsing.
- Completion decision.
- Progress-state updates and stall detection.
- Unit tests for decision behavior.

Target: under 500 lines excluding tests. If classifier transport makes that
impossible, split transport from decision logic rather than growing
`turn.rs`.

### Existing files to change

1. [`codex-rs/model-provider-info/src/lib.rs`](codex-rs/model-provider-info/src/lib.rs)

   - Add `ChatStopSemantics`.
   - Add the capability to `ModelProviderInfo`.
   - Assign Kimi the ambiguous capability.
   - Test built-in provider assignments.

2. [`codex-rs/core/src/session/turn.rs`](codex-rs/core/src/session/turn.rs)

   - Create turn-local progress state.
   - Invoke the arbiter after a provider text stop and before stop hooks.
   - Respect pending input and cancellation.
   - Keep the integration thin; no classifier prompt or phrase list here.

3. [`codex-rs/core/src/session/mod.rs`](codex-rs/core/src/session/mod.rs)

   - Register the new module.

4. `codex-rs/core/src/context/turn_completion_continuation.rs`

   - Define the typed, request-local continuation fragment.

5. [`codex-rs/core/tests/suite/chat_provider_turn_lifecycle.rs`](codex-rs/core/tests/suite/chat_provider_turn_lifecycle.rs)

   - Delete or invert
     `kimi_text_stop_is_terminal_without_extra_inference`.
   - Retain the tool-call lifecycle coverage.
   - Add the integration cases in Section 8.

6. TUI status handling, only if existing events cannot represent the lifecycle.
   Do not add provider-specific logic to the TUI.

## 6. Performance and context budgets

The repair must not recreate the removed multi-round completion guard.

Required budgets:

- Reliable provider: zero new inference requests.
- Tool/function continuation: zero new classifier requests.
- Ambiguous text stop: at most one classifier request.
- Completion classifier timeout: 15 seconds by default. This is an end-to-end
  network deadline: observed Kimi requests can spend 7–8 seconds reaching HTTP
  200 before producing their small structured result. A failed assessment may
  continue again only when a new tool-result boundary proves fresh progress;
  without fresh progress, the turn stops with a visible warning.
- Persistent conversation growth: zero classifier messages and zero repeated
  continuation prompts.
- Raw classifier input: at most 4,000 characters of user objective and 6,000
  characters of assistant response.
- Logging: no prompt contents, tool contents, credentials, environment
  values, or classifier payloads.

The primary provider connection must not be discarded merely to run the
classifier. Reuse provider session state where supported, but do not allow a
classifier failure to corrupt the primary stream.

## 7. Failure fallback

A classifier can time out, return malformed output, or be unavailable.
Fallback must be conservative and bounded:

1. If the turn has tool activity and the latest stop made no new tool/diff
   progress, continue once.
2. If the response is a repeated or empty checkpoint, stop as stalled.
3. Otherwise accept the provider stop and emit one non-fatal warning:
   “PfTerminal could not verify whether this action was complete; review the
   result before relying on it.”

This fallback may use structural facts such as empty output, repeated output,
tool counts, and diff changes. It must not use a list of English progress
phrases as the primary decision.

The warning is deduplicated per user turn.

## 8. Required automated tests

### 8.1 Decision-unit tests

Test the arbiter without HTTP:

1. Reliable provider stop is accepted without classification.
2. Tool call continues through the existing path.
3. Ambiguous incomplete result continues.
4. Complete result is accepted after prior tool activity.
5. Awaiting-user and blocked results stop for user input.
6. Pending user input suppresses automatic continuation.
7. Cancellation invalidates a late classifier decision.
8. Classifier failure uses the bounded fallback.
9. Two no-progress incomplete decisions stop once with one warning.
10. Ten productive incomplete decisions do not trip the stall breaker.
11. Empty-stop repetition cannot loop.
12. Classifier content never enters durable history.

### 8.2 HTTP lifecycle tests

Replace the current broken expectation with scripted provider sequences:

| Scenario               | Provider sequence                              | Expected requests                                                    |
| ---------------------- | ---------------------------------------------- | -------------------------------------------------------------------- |
| Informational answer   | complete text stop                             | 1 primary + completion check only if provider capability requires it |
| Premature action stop  | progress text stop, tool call, final text stop | automatic continuation; one `TurnComplete` at the end                |
| Tool then final        | tool call, final result                        | no redundant work continuation                                       |
| Explicit blocker       | blocker text stop                              | prompt returned to user                                              |
| Repeated checkpoint    | same incomplete stop twice                     | bounded stop and one warning                                         |
| Productive long action | 10 distinct progress/tool cycles, final result | completes without a fixed-attempt failure                            |
| User steer race        | incomplete stop while user input arrives       | user input wins; no stale continuation                               |
| Classifier timeout     | ambiguous stop, classifier timeout             | fallback behavior and terminal lifecycle event                       |
| Non-Kimi provider      | text stop                                      | byte-for-behavior current lifecycle; no classifier request           |

Tests must assert:

- Primary-provider request count.
- Classifier request count.
- Exactly one final `TurnComplete` or one final `Error`.
- No `TurnComplete` at intermediate checkpoints.
- No duplicate warning.
- No request after cancellation.
- No continuation fragment in saved user-visible history.

### 8.3 Adjacent-language coverage

Use semantically equivalent checkpoints with different wording, including:

- A future action.
- An unfinished list.
- A claimed completed result.
- A request for user input.
- A genuine external blocker.

The test must prove the structured semantic route handles these classes.
Adding each sentence to a regex or literal table is not an acceptable
implementation.

## 9. Live qualification

Automated tests are necessary but not the product acceptance instrument.

Run a fresh TUI session on the exact candidate binary using Kimi Code K3:

1. Open a real repository with non-trivial code.
2. Give one plain user objective that requires inspection, editing, tests, and
   a measured result. Do not coach the model about completion behavior.
3. Let the session run for at least 20 minutes and at least 20 tool
   operations.
4. Trail the rollout and application logs during the run.
5. Steer the model once while it is working.
6. Include one task that ends in a real user decision or blocker.
7. Follow with one ordinary informational question.

Pass criteria:

- Zero manual `continue` prompts during productive work.
- Zero redundant continuations after a completed result.
- Zero fixed-count failure during productive work.
- Zero stale `Working` indicator after the turn ends.
- Zero repeated classifier warnings.
- User steering takes precedence.
- Blocker returns control to the user.
- Informational answer ends without a visible completion delay beyond the
  classifier budget.
- Logs contain a decision and reason for every ambiguous stop, with no prompt
  or secret content.

Any failure restarts qualification after the corresponding regression test is
added. A scripted smoke test alone does not satisfy this section.

## 10. Observability

Emit structured tracing fields for ambiguous stops:

- `turn_id`
- `provider_id`
- `finish_reason`
- `completion_decision`
- `decision_source` (`deterministic`, `classifier`, or `fallback`)
- `classifier_latency_ms`
- `semantic_continuation_count`
- `consecutive_no_progress_count`
- `progress_reset_reason`

Do not log assistant text, user text, tool output, request bodies, or
credentials.

These fields must make the following production questions answerable:

- Did the provider stop or did PfTerminal stop?
- Why did PfTerminal continue?
- Did the classifier fail?
- Was the circuit breaker triggered by repetition or by a transport error?
- Did user input supersede an automatic continuation?

## 11. Non-goals and prohibited repairs

- Do not depend on `/goal`, goal status, or goal continuation.
- Do not add Kimi-specific progress prose to the global agent prompt.
- Do not require a literal completion marker from the model.
- Do not restore the removed three-attempt completion-check loop.
- Do not hard-code the incident sentences or a list of English future-tense
  phrases.
- Do not classify every tool call; tool calls already have continuation
  semantics.
- Do not change completion behavior for all chat providers without capability
  evidence.
- Do not hide a provider or classifier failure behind an idle prompt.
- Do not keep the TUI busy after core has emitted a terminal lifecycle event.
- Do not solve the problem with a fixed maximum that kills long productive
  turns.

## 12. Rollout

1. Land unit and mock-provider tests first.
2. Enable `AmbiguousForActionTurns` for Kimi Code only.
3. Run the live qualification in Section 9.
4. Inspect decision telemetry for false continuation, false completion, stall,
   and latency.
5. Expand the capability to another provider only after a reproduced stop
   semantic failure and provider-specific qualification.

Rollback is a single capability change from
`AmbiguousForActionTurns` to `ReliableTerminal`; it must not require reverting
the shared turn lifecycle.

## 13. Definition of done

The defect is fixed only when all of the following are true:

- The current unfinished-progress regression automatically continues.
- A completed Kimi action ends promptly without redundant work.
- An ordinary Kimi answer does not remain under `Working`.
- A genuine blocker returns control to the user.
- Repeated no-progress responses stop safely and visibly.
- Productive long turns are not limited by an arbitrary continuation count.
- User steering and cancellation win all races.
- Non-Kimi providers preserve current behavior.
- The classifier and continuation do not pollute durable conversation
  context.
- The live 20-minute action session passes with logs reviewed.
- The implementation remains centralized, provider-capability-driven, and
  free of domain- or phrase-specific patches.
