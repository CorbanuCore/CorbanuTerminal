# P0 orchestration freeze incident

Date: 2026-07-26 UTC
Release candidate: `release/0.1.23-orchestration-rc1`

## User-visible incident

An assignment Manager running `claude-fable-5-plan` read
`PFUSDC-MAINNET-CAMPAIGN-HANDOFF-20260726.md`, then produced no visible
assistant response. PFTerminal reported that the previous turn had completed
successfully without visible output, attempted its recovery prompt, and then
appeared frozen while later user messages received no response.

The reported Manager and Worker thread IDs came from a different host
(`/home/postfiat`), so their rollout rows are not present in this worktree's
`/home/pfrpc` state database. The supplied transcript, the terminal-event
control flow, and a reproducing TUI regression nevertheless identify two
independent failures in the exact incident path.

## Diagnosis

### P0-A: an empty provider stream was classified as success

Anthropic Messages and chat-completions adapters could emit a terminal
`Completed` event after receiving no text, reasoning, tool call, server tool
call, or web-search result. Core therefore described the turn as successful
even though the model had produced nothing.

Commit `a23af6b4a` fixes the provider boundary. Both adapters now require
actionable model output before emitting `Completed`; an empty terminal stream
returns a stream error and enters the existing bounded retry/error lifecycle.
Tool-only completions remain valid.

Relevant code:

- `codex-rs/codex-api/src/endpoint/anthropic_messages.rs`
- `codex-rs/codex-api/src/endpoint/chat_completions.rs`

### P0-B: ordinary assignment threads were excluded from terminal processing

Commit `394d26c3f` correctly separated persistent crew identity from legacy
role labels, but it also narrowed `is_spawn_orchestration_thread` to persistent
crew members. An ordinary native thread attached as an assignment Manager or
Worker was no longer considered an orchestration thread.

`update_spawn_status_for_thread_notification` consequently returned before it
could:

- replace stale picker text with the current terminal state;
- invoke the assignment Manager empty-output retry;
- deduplicate the terminal notification;
- wake a Manager after Worker completion; or
- advance the assignment watchdog.

This is why the pane looked frozen after the empty completion: the provider
turn was no longer running, but PFTerminal discarded the lifecycle event that
would recover or visibly terminate the assignment.

The release candidate restores assignment participation as an independent
lifecycle criterion. It does not grant crew identity or crew permissions:

- `is_active_assignment_participant` recognizes the active assignment holder
  and target.
- `is_spawn_orchestration_thread` accepts either persistent crew membership or
  active assignment participation.
- the regression asserts that both an ordinary native Manager and Worker
  receive terminal lifecycle processing.

Relevant code:

- `codex-rs/tui/src/orchestrate.rs`
- `codex-rs/tui/src/spawn_orchestration.rs`
- `codex-rs/tui/src/app/thread_routing.rs`
- `codex-rs/tui/src/app/tests.rs`

## Consolidation status

The separate `fix/spawn-control-plane` worktree is not lost. Its tip
`7542ebb04` is an ancestor of this release candidate. The candidate contains
the native mailbox, resume, runtime persistence, provider authorization,
manager-saturation, model-economics, cache-stability, empty-stream, and
assignment-lifecycle changes in one line of history.

The prompt-cache repair keeps the Anthropic tool schema byte-stable across
turns. Structured edit/write fallbacks are advertised from request one, while
activation changes only turn-local dispatch state. Its evidence, scored spec,
and implementation report are stored beside this incident report.

## Qualification

Focused release gates:

| Gate | Result |
|---|---:|
| `codex-api` provider adapters | 164 / 164 passed |
| Assignment lifecycle | 13 / 13 passed |
| Spawn, hierarchy, runtime, and economics | 45 / 45 passed |
| Prompt caching | 8 / 8 passed |
| Structured edit/write fallback | 2 / 2 passed |
| Concurrent fallback activation | 1 / 1 passed |
| Workspace clippy/fix compile | passed with existing warnings |
| Formatting and `git diff --check` | passed |
| Debug candidate build | passed |

The broad `codex-core` suite attempted earlier on this host was not a usable
release gate: extreme parallelism produced unrelated helper-binary, migration,
and timeout failures. The focused failure surfaces above are green.

## Candidate

Binary:

`codex-rs/target/debug/pfterminal`

Version:

`pfterminal 0.1.23`

SHA-256:

`3f14546949f958142d8eee5f8e68c2efaab754010d85de7ab86476b16717268e`

This candidate has not been published or installed over the production
executable.

`pfterminal-debug --version` resolves to this candidate and reports 0.1.23.
`pfterminal-debug doctor` reports the current host's state, log, goals, and
memories databases as healthy with zero failed checks. It is degraded only by
three missing active-rollout rows and eight stale rollout rows; those warnings
do not block startup.

## Separate StakeHub symptom

The shown `stakehub agent unlock` traceback is a separate terminal-input
failure: Python `getpass` reached EOF while reading a vault passphrase from its
input stream. It is consistent with a command launched without a usable
controlling TTY. It is not evidence of the PFTerminal state database or native
agent scheduler causing the exception.
