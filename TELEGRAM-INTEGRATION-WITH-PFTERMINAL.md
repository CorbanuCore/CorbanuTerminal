# Telegram Integration with PFTerminal

## Executive summary

PR #59 contains a useful but experimental Telegram remote surface: private-chat
text, screenshots, commands, approvals, persistence, and recovery foundations
exist. It should merge only after packaging CI and review of the corrected dedup
routing. It should not be advertised for private-chat testing until the exact
package passes the three-session live gate, or as stable until mid-turn steering,
attachments, status, and the seven-day soak pass. Group and forum-topic support
remain explicitly unsupported until per-user authorization and topic isolation
land.

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

| Gate | Required scope | Pass criteria | Evidence | Decision owner |
| --- | --- | --- | --- | --- |
| Merge as disabled experimental code | Current PR plus P0 routing correction | Telegram tests and packaging CI pass; review has no open merge-blocking P0 in the enabled private-chat scope; connector remains disabled by default | CI links, commit, test transcript, reviewed diff | PR maintainer |
| Ship for private-chat testing | Phase 1 | Three fresh sessions on the exact packaged artifact; at least 50 accepted updates total; text, image, approval, cancellation, restart, replay, and network-loss cases exercised; zero lost accepted inputs, duplicate mutations, or silent deaths | Binary hash, redacted transcripts, state snapshots, log intervals, verdicts | Release owner, with one session driven by someone other than the implementer |
| Declare private-chat stable | Phases 1, 2, 4, and 5 | Every private-stability blocker in Known gaps is closed; three consecutive free-form sessions pass; seven-day soak with at least 100 turns and zero lost accepted inputs, duplicate mutations, stuck turns, unbounded storage, or manual restarts | Regression results, session evidence, soak ledger, storage measurements | Release owner after independent review |
| Declare group/topic support stable | Phase 3 | Per-user authorization and topic isolation pass adversarial callback, cross-topic, restart, and race tests; zero cross-talk or cross-principal approvals | Security review and adversarial test artifacts | Release owner plus independent security reviewer |

There are no calendar deadlines in this plan. Progress is landing-based: a
phase exits only when its gate evidence exists. Before execution, the release
owner assigns one integration owner, one independent reviewer, and one live-test
driver; the same person may not fill all three roles.

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
- inbound photos and image documents up to 10 MB as real local-image model
  inputs, including image-only messages;
- bot-token loading from a named environment variable or the encrypted vault;
- a setup script and a Linux systemd user-service template;
- bounded outbound calls, retry of idempotent reads, persisted update-ID
  deduplication, polling-conflict detection, and restart recovery.

Commit `990f1169f` corrected a merge-blocking defect in the reliability layer:
the deduplication function had been installed as a terminal dptree endpoint, so
it recorded and swallowed all updates. It is now filter middleware, with a
regression proving that the first delivery reaches its handler and a replay
does not.

## Known gaps

The current connector is useful for controlled private-chat testing, but it is
not yet a high-quality mobile interface.

### Qualification and security boundaries

1. **No release-level live qualification** *(blocks private testing and every
   later gate)*. Automated tests cover components,
   but the current head still needs a real bot-token session exercising the
   complete Telegram-to-app-server path.
2. **Group authorization is too broad** *(blocks group/topic stability only)*.
   Allowlisting a group authorizes every
   member to submit work and press approval buttons. Group use must support an
   explicit user allowlist before it is presented as a safe default.
3. **Conversation identity ignores Telegram topics** *(blocks group/topic
   stability only)*. State is keyed by chat
   ID. Two forum topics in the same group therefore share one PFTerminal thread,
   active turn, model selection, and approval state.
4. **Deduplication is not a crash-safe delivery transaction** *(blocks
   private-chat stability)*. The connector records a Telegram update before its
   handler durably commits the corresponding input. A crash at that boundary
   can suppress Telegram's replay and lose an accepted message. Moving the
   marker after dispatch would invert the failure into duplicate mutation.

### P1: everyday mobile workflow

1. **Mid-turn messages are rejected** *(blocks private-chat stability)*. A
   follow-up sent while a turn is active
   receives “A turn is already running.” Normal chat behavior requires
   `turn/steer` for steerable turns and a bounded queue for the narrow cases
   that cannot be steered.
2. **Non-image files are not ingested** *(blocks private-chat stability)*. A
   caption is forwarded with an honest
   warning, but the agent cannot read the attached PDF, text file, archive, or
   source file.
3. **Recovery is operator-oriented** *(blocks private-chat stability)*.
   `/status` exposes IDs but does not clearly
   explain whether the bot is idle, working, awaiting approval, disconnected,
   recovering, or blocked.
4. **Installation is Linux-first** *(does not block Linux private-chat
   stability)*. The systemd path is reasonable for an
   always-on Linux host, but there is no equivalent managed service path for
   macOS or Windows.

### P2: polish and breadth

- No explicit topic/thread selector or recent-conversation list.
- No attachment retention controls or cleanup policy.
- No concise usage/cost summary in `/status`.
- No outbound artifact delivery for generated images or files.
- No first-class connector setup and health view in the TUI.

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

## Delivery plan

### Phase 1 — establish an honest baseline

Land the corrected dedup routing only after CI packaging passes. The integration
owner builds the exact candidate artifact and runs it with a dedicated test bot
and copied `CODEX_HOME`. Record the binary hash, commit, configuration shape,
Telegram update IDs, PFTerminal thread/turn IDs, and redacted logs. Do not use a
developer binary that differs from the merge candidate.

The three baseline sessions must collectively cover text, a long tool-using
turn, cancellation, approval, model change, image-only input, captioned image
input, connector restart, one deliberately replayed update, and temporary
network loss. A failure blocks the baseline; it is not converted into a
documentation caveat.

### Phase 2 — make conversation flow natural

Make delivery crash-safe before adding richer flow. Persist each authorized,
validated update to a bounded durable inbox keyed by bot identity and Telegram
update ID. Use a deterministic client user-message ID when submitting it to
`turn/start` or `turn/steer`, record app-server acceptance, and only then mark
the inbox item applied. On restart, reconcile pending items against the
PFTerminal thread before resubmission. The recent-update dedup window remains a
fast replay filter, not the correctness boundary.

Replace the active-turn rejection with app-server `turn/steer` when the current
turn accepts steering. Preserve Telegram order. If the app-server reports that
the active turn is not steerable, retain a small per-conversation FIFO with a
hard item and byte cap, show the queued state once, and start the queued input
after the active turn completes. Never retry an ambiguous mutating request.

### Phase 3 — close group authorization and isolation gaps

Add an explicit Telegram user allowlist for group and forum use. Authorization
must evaluate both the destination chat and the initiating Telegram user.
Approval callbacks must bind the request to the conversation and authorized
user; possession of callback data alone is insufficient. Keep private chat as
the setup default, and refuse group activation without an explicit user policy.

Use a conversation key containing both Telegram chat ID and message-thread ID.
Private chats naturally use an empty message-thread component. Route replies,
stream edits, approvals, cancellation, model settings, persistence, and restart
recovery through the same key.

Test unauthorized messages, callbacks copied into another conversation,
expired approvals, two users racing the same button, and callbacks delivered
after restart. An independent reviewer owns this verdict; the implementation
author cannot self-approve the group-security gate.

### Phase 4 — support useful attachments safely

Introduce one bounded attachment-ingestion path rather than per-extension
special cases. Classify Telegram media from metadata, enforce a configurable
size ceiling before download, store accepted files under a connector-owned
inbox, and attach a structured description to the turn. Images remain native
image inputs. Text and source formats may be read through normal workspace
tools. Archives and executable formats should be rejected initially with a
clear reason.

Every accepted file needs a stable name, content hash, original media type,
source conversation, and retention timestamp. Add cleanup by age and total
bytes so an always-on bot cannot fill the host disk.

### Phase 5 — make operation understandable

Change `/status` from raw identifiers into a compact state view:

- `Idle`, `Working`, `Awaiting approval`, `Queued`, `Recovering`, or `Blocked`;
- active model and workspace;
- current conversation/topic;
- queued-message count;
- last successful Telegram contact and last error;
- the one or two commands that are valid next.

Add a local health command suitable for systemd, launchd, and Windows service
managers. Setup should verify bot identity, authorization, writable state,
workspace access, provider credentials, and sandbox viability before enabling
an always-on service.

### Phase 6 — package and roll out

Ship the connector as experimental until the release-decision table above passes
on the packaged Linux artifact. Add macOS launchd and Windows service guidance
only after the same lifecycle tests run on those platforms. Graduate it from
experimental only at the quantitative private-chat stability gate above.

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

## Graduation rule

The release-decision table is authoritative. Until its private-chat stability
gate passes, describe PR #59 as a useful experimental connector with text,
image, command, approval, persistence, and reliability foundations—not as a
finished replacement for the terminal UI. Group and topic support remain a
separate claim even after private-chat stability is achieved.
