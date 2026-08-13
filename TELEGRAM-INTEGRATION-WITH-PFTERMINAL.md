# Telegram Integration with PFTerminal

## Executive summary

**Current verdict:** PR #59 is a candidate for a disabled experimental merge.
It is not approved for private-chat testing or described as stable. Packaging
CI, exact-package live qualification, an independent group/topic security
verdict, and the durability soak are tracked separately in the release table.

## Product goal

Telegram should let a PFTerminal user continue useful work while away from the
terminal. A user should be able to ask a question, steer an active task, inspect
progress, send a screenshot or file, approve a sensitive action, and receive the
result from a phone. The Telegram connector is a remote surface for the same
PFTerminal agent and workspace—not a separate chatbot with different state or
capabilities.

The target experience is simple:

1. Connect a Telegram bot to one always-on PFTerminal host.
2. Authorize a private chat or selected users.
3. Send ordinary messages and attachments from Telegram.
4. Continue the same conversation across connector restarts.
5. See failures and recovery actions in Telegram without opening host logs.

“Stable” means more than starting successfully or passing unit tests. The
connector must survive normal, unscripted use: follow-up messages while work is
running, long responses, screenshots, approval prompts, network interruption,
process restart, and repeated Telegram delivery.

## Product boundaries

- PFTerminal remains the source of truth for models, tools, permissions,
  sessions, and workspace state.
- Telegram transports user input, agent output, status, and approvals. It must
  not implement a second agent loop or rewrite conversation history.
- One bot token may have only one active long poller.
- A private chat is the initial high-assurance deployment. Group support is not
  high assurance until authorization is per user, not merely per chat.
- The connector should fail closed for unauthorized actions and fail visibly
  for unavailable features. It must never imply that the agent saw an
  attachment that was discarded.

## Release decisions

The connector has four distinct release gates. Passing one must not be
described as passing the next.

| Gate                                | Required scope                                               | Pass criteria                                                                                                                                                                                                                                     | Evidence                                                                    | Decision owner                                                               |
| ----------------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| Merge as disabled experimental code | PR #59, including the corrected dedup route                  | Telegram tests and native packaging CI pass; review has no open merge-blocking P0 in private-chat scope; connector remains disabled by default                                                                                                    | CI links, commit, test transcript, reviewed diff                            | PR maintainer                                                                |
| Ship for private-chat testing       | Private-chat transport, recovery, approvals, and attachments | Three fresh sessions on the exact packaged artifact; at least 50 accepted updates total; text, image, approval, cancellation, restart, replay, and network-loss cases exercised; zero lost accepted inputs, duplicate mutations, or silent deaths | Binary hash, redacted transcripts, state snapshots, log intervals, verdicts | Release owner, with one session driven by someone other than the implementer |
| Declare private-chat stable         | All private-chat workstreams below                           | Private-chat testing gate passes, followed by a seven-day soak with at least 100 turns and zero lost accepted inputs, duplicate mutations, stuck turns, unbounded storage, or manual restarts                                                     | Regression results, session evidence, soak ledger, storage measurements     | Release owner after independent review                                       |
| Declare group/topic support stable  | Per-user authorization and topic isolation                   | Adversarial callback, cross-topic, restart, and race tests pass with zero cross-talk or cross-principal approvals                                                                                                                                 | Security review and adversarial test artifacts                              | Release owner plus independent security reviewer                             |

Before live qualification, the release owner assigns an integration owner, an
independent reviewer, and a live-test driver; the same person may not fill all
three roles.

## Existing integration (status at 2026-07-22)

The implementation is on branch `feat/telegram-connector-hardened`, under
`codex-rs/telegram`, and is proposed in
[PfTerminal PR #59](https://github.com/agtico/PfTerminal/pull/59).
`PFTerminal` is the product name; `PfTerminal` is the existing repository name
and URL casing.

Today it provides:

- `pfterminal telegram`, backed by the same in-process app-server used by other
  PFTerminal surfaces;
- Telegram long polling with a default-deny chat allowlist;
- persistent chat-to-PFTerminal thread mapping in
  `$CODEX_HOME/telegram/state.json`;
- streamed agent replies, Telegram-safe HTML rendering, and message splitting;
- command and file-change approvals through inline buttons;
- `/new`, `/cancel`, `/stop`, `/status`, `/model`, `/approvals`, `/compact`,
  `/diff`, and `/skills`;
- inbound photos and image documents as real local-image model inputs,
  including image-only messages, plus bounded text/source/PDF/XML/YAML
  ingestion with configurable size, age, and total-storage limits;
- bot-token loading from a named environment variable or the encrypted vault;
- setup and health checks plus Linux systemd, macOS launchd, and Windows
  Scheduled Task templates, all carried in PFTerminal package archives;
- `pfterminal telegram --setup`, which launches the packaged interactive setup
  path without requiring a source checkout;
- bounded outbound calls, retry of idempotent reads, persisted update-ID
  deduplication, a bot-keyed durable inbox, deterministic app-server message
  IDs, polling-conflict detection, and restart reconciliation;
- per-user group authorization and conversation isolation by Telegram chat and
  forum-topic ID;
- mid-turn `turn/steer`, a bounded per-conversation FIFO fallback, and
  plain-language `/status` recovery guidance.

Connector logs use a stable redacted conversation identifier. Authorization
and media events do not emit raw Telegram chat/user IDs or paths containing
them.

The corrected update route has a regression proving that the first delivery
reaches its handler and a replay does not.

### Evidence ledger

- [Implementation report](TELEGRAM-INTEGRATION-IMPLEMENTATION-REPORT.md):
  candidate hash, 107-test Telegram result, package-helper results, setup and
  health dry runs, and explicit unrun gates.
- [PR #59](https://github.com/agtico/PfTerminal/pull/59): review history and
  the exact candidate commits.
- [Native non-publishing package run](https://github.com/agtico/PfTerminal/actions/runs/29929282255):
  Linux x64/ARM64, macOS x64/ARM64, and Windows x64 package construction and
  packaged-resource smoke tests. Record its final verdict in the implementation
  report; an in-progress run is not evidence of a pass.
- [Independent review request](https://github.com/agtico/PfTerminal/pull/59#issuecomment-5047392566):
  group/topic adversarial checks and a non-implementer live session. The request
  is not itself a verdict.

## Deferred product breadth

- No explicit topic/thread selector or recent-conversation list.
- No concise usage/cost summary in `/status`.
- No outbound artifact delivery for generated images or files.
- No first-class connector setup flow in the TUI; setup is command-driven.

## User stories

The following stories define the product rather than a list of protocol calls.

### Connect and trust

- As a first-time user, I can connect a BotFather token without placing it in
  `config.toml` or shell history.
- As an operator, I can see which chat and user IDs are authorized before the
  bot starts accepting work.
- As a private-chat user, I can approve a command from my phone and know that a
  different chat or user cannot reuse the button.
- As an operator, I receive a clear, actionable error when another poller owns
  the same bot token.

### Work on the go

- As a user, I can send a task in ordinary language and receive progressive,
  non-duplicated output.
- As a user, I can add “focus on the failing test first” while the agent is
  working; that message steers the active turn rather than being rejected or
  starting duplicate work.
- As a user, I can cancel work and then immediately start another task without
  stale “turn already running” state.
- As a user, I can send a screenshot with or without a caption and the model
  actually receives the image.
- As a user, I can send a supported source or document file and see whether it
  was accepted, rejected, or truncated before the turn begins.

### Resume and recover

- As a user, I can restart the connector and continue the same conversation
  without replayed prompts or duplicate agent output.
- As a user, a brief Telegram or provider outage produces one useful status
  message and automatic recovery, not notification spam.
- As a user, `/status` tells me what is happening in plain language and gives
  the next valid action.
- As a forum user, each Telegram topic maintains an independent PFTerminal
  conversation and approval boundary.

## Execution status and next gates

The table below is the implementation traceability record. “Implemented” means
the boundary exists in PR #59 and has scoped automated coverage; it does not
mean that a later live or stability gate has passed.

| Workstream                         | Status                              | Evidence in the candidate                                                                                                                                                                                               | Remaining gate                                                                                                                                        |
| ---------------------------------- | ----------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| Baseline routing                   | Implemented                         | The dedup middleware routes a first delivery once and suppresses replay; the regression covers both outcomes.                                                                                                           | Packaging CI and the exact-package live sessions below.                                                                                               |
| Crash-safe delivery and follow-ups | Implemented                         | A bot-keyed bounded inbox persists authorized updates; deterministic client message IDs support reconciliation; active work uses `turn/steer` with a bounded per-conversation FIFO fallback.                            | Exercise restart, replay, ambiguous failure, rapid follow-ups, and queue saturation through the real Bot API.                                         |
| Group and topic isolation          | Implemented; security claim pending | Group use requires allowed chat and actor; the conversation key includes chat and topic; approvals bind to conversation and initiating actor.                                                                           | Independent adversarial review of unauthorized messages, copied callbacks, expired approvals, actor races, cross-topic traffic, and restart delivery. |
| Attachments                        | Implemented                         | Images remain native inputs; bounded text/source, JSON, PDF, XML, and YAML ingestion records stable names, hashes, media type, conversation, and retention metadata; cleanup enforces age and byte caps.                | Real image-only, captioned-image, supported-file, unsupported-file, oversized-file, and cleanup cases.                                                |
| Status, health, and setup          | Implemented                         | `/status` reports plain-language state and recovery actions; `telegram --health` checks the operational boundary; `telegram --setup` uses packaged resources; systemd, launchd, and Windows templates ship in archives. | Native packaged execution and real bot-identity/provider/workspace checks.                                                                            |

Live qualification must use a dedicated test bot, copied `CODEX_HOME`, and the
exact package. Appendix A defines the cases and evidence. A product failure is
fixed with a regression and restarts the affected session count; it is not
converted into a documentation caveat.

## Appendix A — acceptance and QA plan

Component tests remain necessary, but the release decision comes from observed
end-to-end behavior.

For these gates, an **accepted update** is one the connector acknowledges for
processing after authorization and validation. A **lost input** is an accepted
update absent from both the intended turn and its bounded queue. A **duplicate
mutation** is one Telegram update causing the same state-changing action more
than once. A **stuck turn** remains active for more than 60 seconds after the
app-server reports completion. A **silent death** is an unreachable connector
without a Telegram-visible terminal error or service-manager restart signal.

### Deterministic regression suite

- first delivery routes once; replay routes zero times;
- crashes before inbox write, after inbox write, after app-server acceptance,
  and before the applied marker reconcile without loss or duplicate mutation;
- a failed or timed-out read follows the bounded retry policy;
- a mutating request is never blindly retried;
- state and dedup markers survive restart and corrupt state is isolated;
- output splitting preserves valid Telegram HTML and UTF-16 limits;
- an approval is consumable once and only by its bound principal;
- independent conversation keys never share thread, turn, queue, or approval
  state;
- attachment caps, hashes, paths, and cleanup are enforced.

### Stable-candidate free-form qualification

Run at least three fresh sessions against the exact packaged candidate:

1. **Mobile coding session:** ask the agent to inspect a repository, interrupt
   its direction twice with ordinary follow-ups, approve one command, reject
   another, request `/diff`, and cancel/restart work.
2. **Evidence session:** send multiple screenshots, an image-only message, a
   supported document, an unsupported file, and an oversized attachment. Verify
   the agent refers only to content it actually received.
3. **Recovery session:** work in two independent conversations, disconnect the
   network, restart the connector during activity, inject a duplicate update,
   and resume. Verify no input loss, cross-talk, duplicate mutation, stale
   approval, or notification flood.

The driver should use plain objectives, not an implementation checklist. An
observer tails connector and app-server logs, records every unexpected pause or
manual recovery, and compares Telegram-visible behavior with persisted state.
Any product failure is fixed with a regression and restarts the affected
qualification count.

### Durability soak

After the interactive sessions pass, run the packaged connector for seven days,
including at least 100 turns, two controlled process restarts, one network-loss
interval, and a provider-auth refresh boundary. Sample media-directory bytes,
queue depth, and process state at least daily. The soak fails on any lost
accepted input, duplicate mutation, silent death, required manual restart,
cross-conversation delivery, stale active-turn state lasting more than 60
seconds after completion, or growth beyond the configured storage and queue
caps.

## Appendix B — observability and evidence

Logs must be useful without containing prompts, bot tokens, provider keys,
attachment contents, or approval payloads. Structured events should include a
hashed or redacted conversation key, update ID, PFTerminal thread and turn IDs,
operation, attempt, latency, result class, queue depth, and recovery action.

The release report should bind each acceptance claim to:

- candidate commit and binary hash;
- sanitized configuration and platform;
- test transcript or artifact path;
- relevant structured log interval;
- pass/fail verdict and any defect commit.
