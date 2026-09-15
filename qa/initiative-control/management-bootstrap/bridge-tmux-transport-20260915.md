# TMUX decision bridge — owner-daemon-bridge-01

## Allocation and status

Implementation candidate for manager review, September 15, 2026. No live Slack
message, live Slack state operation, credential read, push, release, or human-test
acceptance is part of this allocation.

- Action: `owner-daemon-bridge-01`.
- Allocation digest: `eb7d754d6624693f93898e9816399f99b664c032b19ee51678c2bd2448f195f7`.
- Claim: `127f2eb4-0895-4cb2-9790-15c901ba8552`.
- Frozen brief SHA-256 verified before work:
  `17e1dddedb1b8b0756306a274e3191a491a2f167ad55c0bf3a13c0b0c31f6ffb`.
- Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-owner-daemon-c-20260915`.
- Branch: `bootstrap/owner-daemon-c-20260915`.
- Base: `ac35ee1706c602f15409286e3d540904b742fe8d`.
- Runtime: `gpt-6-astra`, high.
- Classification: product-initiative increment, internal delivery control.
- Product authority: **Internal delivery control — TO BUILD**, “fresh Fable 5.1
  High management through Corbanu/TMUX; durable event dispatch, acknowledgments
  and watchdog” and “actual Slack reply/decision/agent acknowledgment”.
- Active plan: `docs/plans/active/initiative-delivery-control.md`.
- Selected sprint: `PF-80-S01`, `in_progress`, remaining bootstrap delivery and
  final combined-tree review/evidence. The explicit manager allocation above
  supplies this worker's coordinates; the checked-in plan still has older lane
  coordinates. Plan/sprint allocation reconciliation and acceptance remain
  manager-owned, outside this worker's five writable files.

## Contract and owner API

A manager explicitly selects `transport_kind="tmux"` and supplies an
`owner_tmux.BridgeReceiver(worker, owner)` to `decision_manager.dispatch` or
`Bridge`, with `channel=None`. The default stays native. Transport selection
is persisted before delivery; neither evidence fields nor message content select
or change the transport. There is no capture-file import, stdin TMUX receipt
format, native-shaped fallback, or new live CLI registration.

The receiver is an already-launched `owner_tmux.Worker` with a completed prior
turn and exactly one private rollout. Its owner agent is the frozen worker
`action_id`; its allocation is the frozen `allocation_digest`. Native Core
agent identifiers are not silently mapped onto TMUX identities. The real rollout
session/thread identifiers are retained separately.

The receiver checks the worker's frozen metadata, boot and UID, exact socket
path/device/inode and private permissions, TMUX executable digest, exact named
session and session ID, pane ID, pane/server PIDs and recorded process start
identities. It checks that both processes are live and that the pane is not dead.
It rechecks this identity around each capture and before work eligibility.

The collector takes a complete baseline rollout and live pane capture, rejects
a preexisting exact ACK or a visible credential/approval prompt, writes a durable
one-attempt intent, and generates a fresh nonce. It pastes the exact bounded
request using a unique TMUX buffer, then sends Enter separately. Question and
answer remain data with an acknowledgment-only instruction. The prompt is
limited to 16,384 UTF-8 bytes and the existing secret-pattern check.

Only an appended, correlated completed assistant turn counts: exact nonce-bearing
user prompt, turn ID, runtime/model/provider/effort/worktree/policy context,
completed model response, and exact final ACK. The ACK must equal the UTF-8
canonical serialization of `expected_ack(request, receipt)`, including byte
spacing. Equivalent reformatted JSON is refused for TMUX. The exact ACK must also
appear in a fresh direct capture of the pinned pane. Pane text by itself, including
an echoed prompt, cannot qualify.

The collector keeps the exact issued record, rollout bytes, capture bytes, and
monotonic start time in memory. Retention compares the complete supplied record
against that issued witness, re-reads the same rollout inode and exact bytes,
recaptures the live pane byte for byte, and consumes the nonce once. The maximum
evidence age is 20 seconds from send preparation (a smaller owner timeout is
allowed). Final eligibility revalidates the same consumed witness. A timeout,
uncertain key send, missing/malformed/truncated/edited rollout, changed pane,
identity drift, reused nonce or collector restart leaves the handoff held.
A new collector cannot import old evidence or resend the durable attempt.

## Exact retained TMUX evidence shape

`transport.bridges[handoff]` retains the existing request, receipt, submission ID,
ACK, watermark and unlock flag, plus `transport_kind: "tmux"`, `receiver`, and
`tmux_evidence`. The submission ID is the freshly generated nonce. The evidence
record has exactly these fields (angle-bracket values denote types/placeholders):

```json
{
  "type": "tmux-completed",
  "transport": "tmux",
  "nonce": "<32 lowercase hex characters>",
  "handoff": "<exact handoff>",
  "agent": "<frozen action_id>",
  "allocation": "<frozen allocation_digest>",
  "payload_digest": "<SHA-256 of canonical payload>",
  "request_digest": "<SHA-256 of canonical full request>",
  "expected_ack": "<exact canonical JSON string, no trailing newline>",
  "receiver": {
    "run": "<absolute private worker run directory>",
    "socket": "<absolute socket path>",
    "socket_device": "<integer>",
    "socket_inode": "<integer>",
    "session": "<exact session name>",
    "session_id": "<TMUX session ID>",
    "pane": "<TMUX pane ID>",
    "pid": "<integer>",
    "start": "<recorded pane process start>",
    "server": "<integer>",
    "server_start": "<recorded server process start>",
    "boot_id": "<host boot identity>",
    "uid": "<integer>",
    "binding_digest": "<SHA-256 of canonical frozen worker binding>",
    "tmux_digest": "<SHA-256 of TMUX executable>"
  },
  "prompt_digest": "<SHA-256 of exact submitted prompt bytes>",
  "capture_digest": "<SHA-256 of exact direct pane capture bytes>",
  "rollout": {
    "path": "<exact private rollout path>",
    "device": "<integer>",
    "inode": "<integer>",
    "session_id": "<rollout session UUID>",
    "thread_id": "<rollout thread UUID>",
    "before_bytes": "<integer>",
    "before_digest": "<SHA-256 of baseline bytes>",
    "after_bytes": "<integer>",
    "after_digest": "<SHA-256 of completed bytes>",
    "turn_id": "<correlated completed turn ID>"
  }
}
```

The ACK string encodes exactly `handoff`, `agent`, `allocation`,
`payload_digest`, and `receipt_id` using `decisions.canonical`.
Digests alone do not authenticate a caller's capture. Acceptance requires the
live, owner-created collector and its unconsumed in-memory witness. Complete
pane/rollout contents are not copied into the Slack journal.

## Refusal regressions and what they prove

- Wrong agent, allocation or payload digest: refuses before any delivery intent
  or keys; cannot reroute a valid decision to another allocation.
- ACK differs by one byte; semantically identical reformatted JSON: completed
  assistant output is refused unless its bytes match the expected string.
- Echo without completion, missing response, wrong user prompt or wrong runtime:
  visible text is insufficient; the final turn must correlate to this request.
- Changed evidence fields, extra/native-shaped records, absent collector:
  callers cannot assert or normalize a TMUX acceptance.
- TMUX record on a native bridge, native record on a TMUX bridge, attempted kind
  change: neither transport can satisfy the other transport's evidence checks.
- Consumed nonce, saved evidence given to a new collector, different handoff:
  an old completion cannot authorize another acceptance.
- Earlier real pane capture substituted during a later handoff: a fresh rollout
  does not make an old pane capture sufficient.
- Evidence from a second real private TMUX receiver/socket/session, changed
  socket/session record, and actual session rename: receiver identity is pinned.
- Truncated, changed or atomically replaced rollout and changed live pane:
  retained evidence cannot be edited or reduced to an old valid prefix.
- Expired monotonic witness: stale evidence is refused even with valid fields.
- Preexisting exact ACK and visible synthetic sign-in prompt: sends no keys.
- Uncertain Enter followed by receiver reload: durable intent prevents resending.
- End-to-end decision bridge: valid receiver output reaches agent-acknowledged
  and one work permit; duplicate retention/unlock and restart import fail.
- Reply edit during acknowledgment: actual ACK can be retained while the shared
  watermark/feed/ingress gate denies work.

No native evidence requirement was relaxed. The original accepted
`multi_agent_v1.send_input` shape, exact agent/handoff/payload checks, stable
submission ID, final `multi_agent_v1.wait_agent` shape, matching agent/submission,
and exact parsed expected ACK checks remain. Historical native journals without
a transport field remain native. Existing reconciliation and one-permit behavior
remain covered by the unchanged native regressions.

## Validation and preserved attempts

The safe automated-test guidance was read before tests. All new process fixtures
use disposable private homes, nonexistent fixture auth-link targets, harmless
local receivers and private TMUX sockets. No operator profile or native credential
store is read. Slack SDK tests target their local HTTP fixtures.

- Initial seven-case focused run: 7 errors in 22.765s. The TMUX target used
  `=session` with a pane-target command, which returned empty pane fields.
  Corrected to `session:` and retained exact returned session checks.
- Corrected seven-case focused run: 7 passed in 17.566s.
- First five-case integration/additional-refusal run: 1 failure and 1 error among
  5 tests in 49.485s. The harmless Python fixture used canonical terminal input,
  truncating larger requests. It now uses `tty.setcbreak`, like a TUI input
  reader; production checks were not weakened.
- Corrected two-case end-to-end run: 2 passed in 6.782s.
- Final-tree full SDK suite: **607 tests passed in 359.121s**, exit 0, with
  `TMPDIR=/private/tmp`. No failures or skips were reported. The existing
  synthetic HTTP 500/429 fixture cleanup emitted ResourceWarnings.
- Final governance: plans **3 active / 3 allowed**, sprints **116 current /
  126 archived**, both exit 0. `git diff --check` passes.
- Fourteen new regression methods are included in the full-suite count; native
  regressions also remain green. No Rust tests were needed for these Python-only
  changes. Only this result is final-tree evidence; earlier failures above remain
  part of the record.

Exact SDK command:

```sh
env -u CODEX_HOME -u CORBANU_HOME -u PFTERMINAL_HOME TMPDIR=/private/tmp PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts/initiative_control:/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/lib/python3.14/site-packages /Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B -m unittest discover -s scripts/initiative_control -p '*test*.py'
python3 docs/plans/check.py
python3 docs/sprints/check.py
git diff --check
```

## What this does not establish

**Same-UID private homes are not worker isolation.** The manager, TMUX executable
and server, host OS, and receiver rollout files remain trusted. A hostile
same-UID process can tamper with them, impersonate local output or modify Python
objects; this is not cryptographic agent attestation or protection against a
malicious owner. The native owner pipe likewise relies on authentic owner-side
tool evidence. This transport closes the caller-supplied capture bypass within
that trusted local boundary; it does not introduce a stronger security boundary.

The tests prove actual keys/captures and local completed-turn correlation with a
harmless synthetic receiver. They do not prove a real model response, production
TUI rendering, live Slack delivery, human approval, worker isolation, recurring
enablement, or successful execution of acknowledged work. Inference, a protected
credential prompt and external delivery were not exercised.

Internal-only code-blind/TUI/live-repository applicability is proposed N/A for
this SDK increment: no shipped TUI flow is changed and there is no human-test
handoff. Integrator acceptance of that disposition remains outstanding. The later
PF-80-S01 exact-candidate independent functional/real-Slack/TMUX qualification
gate remains open; this note does not close it or relabel earlier pane-by-eye
handoffs as contract-compliant. There is no named-human acceptance or benchmark/
release qualification claim.
