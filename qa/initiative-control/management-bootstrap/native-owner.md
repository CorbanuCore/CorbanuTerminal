# PF-80-S01 native owner bridge — reviewed lifecycle checkpoint

Product initiative: **Internal delivery control — TO BUILD**, “Use sequential
sprints per initiative”. Plan: `docs/plans/active/initiative-delivery-control.md`;
sprint PF-80-S01 remains `in_progress`. Parent's canonical allocation is
`worktrees/management-workstreams-20260911/docs/research/tasknode-integration/coordinator-bootstrap-20260913.md`,
heading **Native owner bridge — next bootstrap unit, September13**. Its allocation
and plan coordinates were inspected read-only, including the parent's uncommitted
records. The Corbanu development skill/root policy informed the scope and gates.

Implementation checkout: `/Volumes/CorbanuDrive/Corbanu/worktrees/native-owner-20260913`;
branch `bootstrap/native-owner-20260913`; base
`a3368e443e09a9acfd1dd8c74432d9924ed8cd69`. Only the three assigned files are authored.
At the original worker handoff, changes were uncommitted. Parent owns freeze,
Fable review, integration and actual native lifecycle qualification. The worker
ran no independent review/autoreview or nested
agents were run. No native/model calls, Slack, credentials, services, dependency,
automation, Git integration, plans or operational state were changed.

## Executable interface

Run from this checkout; `--state` must identify an already initialized private
Coordinator directory. Each process executes at most one host operation:

```sh
python3 -B scripts/initiative_control/native_owner.py spawn --state /owner/private/state --timeout 30
python3 -B scripts/initiative_control/native_owner.py poll --state /owner/private/state --timeout 30
python3 -B scripts/initiative_control/native_owner.py start-work --state /owner/private/state --timeout 30
python3 -B scripts/initiative_control/native_owner.py close --state /owner/private/state --timeout 30
```

The trusted owner connects stdin/stdout pipes and writes one JSON line:
`{"action_id":"accepted-action-id"}`. The process emits a `native_request` JSON
line and waits for one response line, then emits a redacted `owner_receipt` line.
`inspect` emits only an owner receipt. This is a working stdio exchange, exercised
with subprocess fixtures; it does not load an adapter named by a model/event.

Map `spawn` to the host's actual native spawn tool with the startup instruction,
scope and binding only; no assignment yet. Deliver its binding/request ID to the
worker. Map `poll` to read-only native status/message inspection. Map `start-work`
to actual native send/start with the exact assignment and binding/request ID.
Map `close` to actual native close. Host tool routing/model selection are trusted
owner configuration; copy the real tool-returned agent ID, never invent one.
The host must independently associate worker messages with that native sender.

Every host response has exactly `request_id`, `binding`, `agent_id`, `status`,
and nonempty `receipt` (actual native tool evidence object). Echo request ID and
binding verbatim. Effects require `spawned`, `submitted`, or `closed`, respectively.
Poll accepts `running`, `timed_out`, `ready`, `returned`, or `failed`. A `ready`
response additionally requires exactly this `ack` object from the actual worker:

```json
{"action_id":"accepted-action-id","claim":"exact-core-claim","allocation_digest":"exact-frozen-digest","agent_id":"actual-native-id","ack":"ready_no_work"}
```

`returned`/`failed` additionally require a nonempty `result` object. Never turn a
timeout, missing message or quiet pane into `failed` or `returned`. ACK is a
message observation; some native hosts finish a turn when emitting it. The host
must distinguish a finished ACK turn from termination of the reusable agent.
`submitted` proves native send receipt, not successful execution or acceptance.

API equivalent (trusted host function implements the same request/response):

```python
owner = NativeOwner(Coordinator(existing_private_directory), host_native_call)
owner.run(action_id, "spawn", timeout=30)
owner.run(action_id, "poll", timeout=30)  # repeat read-only inspection as needed
owner.run(action_id, "start-work", timeout=30)  # only after strict ACK
owner.inspect(action_id)
```

`reconcile` takes `{"action_id":...,"effect":"spawn|start-work|close",
"response":<original-request-bound actual receipt>,"evidence":<owner lookup evidence>}`.
API: `owner.reconcile(action_id, effect, response, evidence)`. Inspect the actual
native tools/history first. This records a proven effect; it never resends it.
Duplicate effects return existing state. Conflicting receipts/identities fail.

## Durability, authority and limits

The existing SQLite database gains one `native_owner` journal table. Core claim,
dispatch, ACK, return and failure reconciliation use existing Coordinator APIs.
Intent commits before spawn/send/close. A crash before effect or before receipt
leaves reconciliation required; no retry is authorized. Durable receipts replay
into the core after a receipt/core-update crash. Even a pre-journal core claim
cannot respawn; the owner can import actual recovered spawn evidence. Proven
absence/no-effect disposal uses the existing core owner's reconciliation controls;
this bridge deliberately provides no reset/retry operation.

Pause, allocation digest, dependencies, sprint reservation and resource ownership
are checked before every effect, including close, and again before host handoff.
Poll works while paused and after restart. The owner must serialize authority
changes with the actual host tool invocation: SQLite cannot atomically fence an
external native tool after a request has left this process. Close requires an
observed terminal result and valid authority; emergency close while paused or
after allocation/sprint retirement remains an explicit host-owner operation.
Close does not accept work. Returns await owner verification/integration; the
bridge has no verify, integrate, complete-sprint or successor operation.

Frames are capped at 64 KiB; each stdio read/write has a 0–60 second bounded
deadline (strictly positive). The supplied API adapter must honor its timeout.
Transport errors omit exception text; uncertain effects and unavailable/timeout
observations exit 2 and preserve journal state. Successful receipts exit 0.
Private protocol frames include frozen assignment inputs and native evidence;
only the final summary omits those contents. Do not put secrets in assignments
or expose this owner-authority interface to workers, Slack or an arbitrary RPC.
Journals retain full bounded receipts; no retention/compaction policy is added.

## Evidence and remaining qualification

Fixture-only tests cover real SQLite reopening, subprocess stdio restart, both
effect/receipt crash windows, core-update replay, duplicate/concurrent effects,
wrong ACK/binding/identity, pause before work, stale/dependency/resource denial,
running/timeout/transport failure, native failure and return without acceptance.
Final command: `PYTHONPATH=scripts/initiative_control python3 -B -m unittest
test_native_owner test_coordinator -v`: **63 passed** (20 bridge, 43 core), 3.548s.
Both plan/sprint checkers pass (3 active plans; 115 current/126 archived sprints).
Diff whitespace checks include all three new files against `/dev/null`.
Sizes: implementation344, tests429, this receipt122 = **895 total / 466 non-test**.
Internal-only browser/TUI and TensorCash/Isometric qualification: N/A for this
transport increment, subject to integrator acceptance. Actual native lifecycle
proof is mandatory before operational acceptance; later combined dashboard/Slack
flows retain the independent functional gate. Human acceptance, benchmarks,
release readiness and whole-sprint completion are not claimed.

## Parent review and actual lifecycle, September 13

Fable High review01, session `01a09c3e-c8e2-73b0-bc09-4d30c33a53bd`, found
no blocking defects. Four P3 follow-ups remain: pre-work failure projects as a
return needing verification; unclaimed/archived API calls can raise KeyError;
failed-native successful close/inspect exit-code wording is imprecise; fixture
temporary directories live under the checkout. Owner inspected these paths:
none grants acceptance or duplicate effects. Close before action archival and
inspect native status as well as core state. Preserve the findings; no code
correction or repeated opinion review was required for this internal increment.

Actual fresh Fable cycle `949de5ae-08a4-4710-8d7f-a90dd92d66c1` accepted the
frozen `native-owner-qualification-01` action. The stdio bridge launched fresh
Astra High `01a09c4f-fafd-7cf1-9bee-6dc8383b3312`, recorded its exact binding ACK,
then delivered work. The agent verified all three frozen hashes and ran 63
tests successfully in 2.545s; parent had independently run 63 in 2.612s. A new
process repeating start-work emitted no second native request. Return produced
an event and awaited owner verification, without completing the sprint.

Retained real failure: canonical PTY input could not deliver the large return
frame. Parent interrupted that read-only exchange and recovered with noncanonical
input; no work was resent. Production owners must use pipes or noncanonical input.
Parent then deliberately interrupted receipt recording after the actual close.
A fresh process held the duplicate close; actual native lookup returned not_found.
Importing the original request-bound close proof reconciled the gap without retry.
Owner verified the internal return at coordinator revision118. Worker is closed;
all product modes/global dispatch are paused. Private full receipts are in
`/private/tmp/no.nqvEuK/`; implementation SHA256 starts `ae8c050040ed`, tests
`f7d0a125d83c`. Receiving integration, Slack and combined recurring qualification
remain separate gates. Internal-only browser/TUI and live-repository N/A accepted
for this bridge; this is not isolated functional acceptance of the final workflow.
