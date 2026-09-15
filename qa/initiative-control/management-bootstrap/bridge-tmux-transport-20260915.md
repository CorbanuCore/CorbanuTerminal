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
monotonic capture time in memory. Retention compares the complete supplied record
against that issued witness, re-reads the same rollout inode and exact bytes,
recaptures the live pane byte for byte, and consumes the nonce once. After the
`owner-daemon-bridge-02` revision, ACK collection has an independent
`handoff_timeout` (default and maximum 300 seconds). The
`owner-daemon-bridge-03` revision measures this shared deadline from baseline
preparation, including baseline retries and subsequent ACK collection.
The maximum witness age remains `timeout` (default and maximum 20 seconds), now
measured from immediately before its ACK pane capture. Verification checks that
age both before and after its live identity/rollout/pane checks. Final eligibility
revalidates the same consumed witness; no refresh, import or resend is added.
A slow model can therefore spend 60 seconds producing the ACK and still leave
20 seconds for retention/unlock. Delaying unlock beyond the capture's freshness
bound still refuses; this is not an indefinite authorization lease.

During baseline preparation and ACK polling, an incomplete trailing rollout
record or a snapshot changing during an append is retried within the shared
collection deadline. No prefix or partial record is accepted as evidence. Stable
newline-terminated records still require strict JSON and full provenance;
malformed complete records remain terminal. Verification reads remain strict
and do not retry. A timeout,
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

## Revision — owner-daemon-bridge-02

- Action: `owner-daemon-bridge-02`; allocation digest:
  `93992203a4e962d82305ba7aef9bd1619edb8e15eb89e7de2446e25985d8b851`.
- Claim: `10b596dc-33ae-4644-a34b-2d5a7225ba55`.
- Base: `3ede7d18836ec02e31cd128b5b5c4c57968e2a93`; same worktree,
  branch and Astra High runtime as above.
- Frozen brief: `/private/tmp/fmgr.Q1SIYZ/briefs/owner-daemon-bridge-02.json`;
  `shasum -a 256` matched
  `e5e879c228b103b86e13ae77698a3503f4b367db5e53d2295d6e0bd96ab07a18`.
- Read independent review `/private/tmp/fmgr.Q1SIYZ/dbridge-review.json`
  before implementation. Both findings were P3; original verdict was
  “patch is correct.” This worker corrects both findings, without adding an
  independent review or claiming reviewer acceptance.
- Classification: bounded reliability fix to the existing internal transport,
  within the product authority **Internal delivery control — TO BUILD**,
  “durable event dispatch, acknowledgments and watchdog” and
  “actual Slack reply/decision/agent acknowledgment.” PF-80-S01 remains
  `in_progress`; allocation/plan reconciliation and all later qualification
  gates remain manager-owned as recorded above.

### Finding dispositions and discriminating proof

**P3 freshness test gap: corrected.** The stale-clock assertion now runs before
any pane changes or nonce consumption. Restoring the real clock accepts and
consumes that same witness, then changing the pane independently fails
verification. No production change was needed to demonstrate this finding.

The three-case pre-fix run below used unchanged production code from the base
commit with the new tests. Result: **3 tests in 4.390s, 1 error, 1 failure**;
the corrected freshness case passed. The actual mid-append receiver failed in
`rollout()` with `decisions.Invalid`; the 60-second simulated model turn
returned `work_ready=False`.

```sh
env -u CODEX_HOME -u CORBANU_HOME -u PFTERMINAL_HOME TMPDIR=/private/tmp PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts/initiative_control:/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/lib/python3.14/site-packages /Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B -m unittest -v test_owner_tmux.TmuxTests.test_bridge_mid_append_rollout_repolls_before_issuing_evidence test_decision_manager.ManagerTests.test_tmux_slow_model_turn_keeps_fresh_witness_and_unlocks test_owner_tmux.TmuxTests.test_bridge_live_pane_changes_and_stale_capture_refused
```

Using the structured edit tool, temporarily deleted exactly
`and time.monotonic() - started <= self.timeout` from the pre-fix
`BridgeReceiver.verify`. Ran:

```sh
env -u CODEX_HOME -u CORBANU_HOME -u PFTERMINAL_HOME TMPDIR=/private/tmp PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts/initiative_control /Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B -m unittest -v test_owner_tmux.TmuxTests.test_bridge_live_pane_changes_and_stale_capture_refused
```

With the predicate deleted: **1 test, failed in 1.218s**, specifically
`AssertionError: Invalid not raised` at the stale-clock assertion. Restored
the exact predicate with the structured edit tool and reran that same command:
**1 test passed in 1.628s**. The mutation was restored before implementing the
separate collection/capture clocks.

**P3 benign-read/model-latency hold: corrected.** A private real TMUX receiver
writes its completed turn without the final newline and waits. The parent
observes those actual partial bytes, confirms no evidence is issued, and signals
the child to finish appending. Delivery re-polls, accepts only the completed
snapshot, and verifies it. The end-to-end manager regression advances only the
transport's monotonic clock by 60 seconds before ACK capture; the real fixture
pane and rollout still supply evidence. It proves retention, acknowledged state
and final unlock, without sleeping 60 seconds or claiming live inference.

Additional cases prove permanently incomplete data expires without evidence,
malformed complete JSON refuses without retry acceptance, an ACK capture past
the collection deadline issues no witness, and a witness expiring during live
verification is refused before nonce consumption. Existing replay, imported
capture, edited rollout, identity, cross-transport and native-path refusals
remain unchanged. The intentional liveness changes are bounded re-polling and
measuring witness freshness from capture; no partial/malformed evidence,
stale witness, new owner, replay or resend becomes valid.

The five-case corrected focused run passed **5 tests in 8.577s** (mid-append,
slow model, independent stale/pane, permanent partial/malformed, and expiry
during verification). The collection-deadline case was then added before the
full final suite. All runs use disposable fixture profiles and local SDK HTTP
servers. No Rust tests, credentials, Slack messages, push or release.

First revision full SDK run: **612 tests in 365.457s, 3 failures**, exit 1.
All failures were existing shell-fixture startup waits: blank pane instead of
initial `READY`, before ACK/bridge operations, in
`test_duplicate_launch_prompt_and_start_are_never_retried`,
`test_exit_at_pane_query_uses_a_post_exit_process_snapshot`, and
`test_exit_between_pane_query_and_snapshot_is_reobserved_by_close`.
Focused replay of those three exact tests, using the SDK environment above and
`-m unittest -v test_owner_tmux.TmuxTests.<method>` for each: **3 passed in
2.187s**, with no intervening code changes. This supports a transient startup
failure but does not erase the failed full run. Full fresh replay with the exact
SDK command above: **612 tests passed in 382.283s**, exit 0, no reported skips.
There were no intervening production or test code changes. Both runs emitted the
existing synthetic HTTP 500/429 ResourceWarnings. The final suite includes five
new regression methods; the strengthened existing freshness case is also covered.
Governance passed: **3/3 active plans, 116 current / 126 archived sprints**.
Final `git diff --check` passes. Production `decision_manager.py` is unchanged;
its native retention and final unlock gates have not been altered.

## Revision — owner-daemon-bridge-03

- Action: `owner-daemon-bridge-03`; allocation digest:
  `0f744b2d5dc9cde72f065536619ab04ad011cb51f8ea59e7b50fcab4fb100a91`.
- Claim: `c4125294-bb65-4d59-962f-92186fe99c47`.
- Base: `e800a3db1147443db2f5822bdacae467bde5dd21`; same worktree,
  branch and Astra High runtime as above.
- Frozen brief `/private/tmp/fmgr.Q1SIYZ/briefs/owner-daemon-bridge-03.json`:
  `shasum -a 256` matched
  `005ab4b5e2bf310d217afa1e1c88e730f3b5fd01847d1c67764bbc8aa91e6c1e`.
- Read independent review `/private/tmp/fmgr.Q1SIYZ/JrsRzA-review.json` first.
  Original verdict: “patch is correct,” with two P3 findings. This revision
  addresses those findings; it does not claim a new independent approval.
- Classification: bounded reliability fix under **Internal delivery control —
  TO BUILD**, “durable event dispatch, acknowledgments and watchdog” and
  “actual Slack reply/decision/agent acknowledgment.” PF-80-S01 remains
  `in_progress`; manager-owned allocation reconciliation and later functional
  qualification remain open as above.

### Baseline retry and duplicate-effect reasoning

The pre-send baseline read now retries only `RolloutPending` within the same
`handoff_timeout` used for ACK collection. The clock starts before the baseline
read and is not reset after it; another deadline check before `worker.once()`
prevents a baseline/capture that finishes late from starting a durable attempt.
All these baseline retries precede `once()` and any buffer/key operation, so they
cannot duplicate a durable attempt or send. After the one durable attempt,
existing anti-resend behavior is unchanged. Partial evidence is never accepted;
complete malformed JSON and identity/provenance failures remain terminal.
A permanently pending baseline still times out and leaves dispatch uncertain;
this revision recovers transient appends, not timed-out deliveries.

`test_bridge_baseline_mid_append_repolls_before_attempt_or_keys` removes the
last newline from actual baseline rollout bytes, observes that incomplete read,
asserts no durable attempt, buffer/key operation or witness exists, then finishes
the append on the same inode. Delivery must retry and use the full baseline,
perform exactly one `once`, paste and Enter, and produce verifiable evidence
from the real private TMUX receiver. The second new test,
`test_bridge_baseline_partial_deadline_and_malformed_send_no_keys`, proves that
permanent partial bytes receive bounded retries and complete malformed JSON is
terminal after one read; neither path creates an attempt or sends keys.

Pre-fix discrimination used both new tests against unchanged production code
at the base commit: **2 tests in 0.956s, 1 error and 1 failure**, exit 1.
The mid-append test raised `owner_tmux.RolloutPending` at the baseline read in
`deliver`; the permanent-partial case failed because it observed only one read.
The malformed subcase passed. Exact command (using the SDK interpreter from the
full-suite command above):

```sh
env -u CODEX_HOME -u CORBANU_HOME -u PFTERMINAL_HOME TMPDIR=/private/tmp PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts/initiative_control /Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B -m unittest -v test_owner_tmux.TmuxTests.test_bridge_baseline_mid_append_repolls_before_attempt_or_keys test_owner_tmux.TmuxTests.test_bridge_baseline_partial_deadline_and_malformed_send_no_keys
```

Corrected focused run: **6 tests passed in 8.310s**, exit 0. It includes both
new baseline cases, the existing ACK mid-append and partial/malformed cases,
collection deadline refusal, and the manager's simulated 60-second model turn.

### Handoff timeout and lock cost

**Decision: retain the default/max `handoff_timeout=300` seconds.** This preserves
the existing allowance for a real model turn, including the simulated 60-second
case; that regression proves clock handling, not real inference latency.
The baseline and ACK phases share this budget rather than adding two waits.
Witness freshness remains at most 20 seconds from ACK capture.

The operational cost is explicit: `decision_replies.dispatch` holds both the
process-global store and feed exclusive locks across its receive callback,
including baseline preparation and ACK collection. The collection allowance
therefore extends their potential hold from roughly 20 seconds to roughly
**five minutes**, plus surrounding I/O/processing; the polling deadline is not
an OS-enforced wall-clock interrupt. Contending `lock_timeout=5` callers
(including `ResolutionStore`, the manager CLI and decision-feed operations)
can time out more often, and `decisions.save_fixture` uses an unbounded feed
`flock`, so it can block for minutes. This revision accepts and discloses that
cost to preserve model-turn headroom within the frozen scope; lock ownership
and the dispatch protocol are unchanged.

### Final validation

After the code/test edits and `git diff --check`, the full SDK command recorded
above, with `TMPDIR=/private/tmp`, passed **614 tests in 336.610s**, exit 0,
with no reported skips or failures. This includes the two new baseline tests
and all existing partial/malformed, stale, replayed, imported/edited witness,
identity-drift, cross-transport, anti-resend and native regressions. The run
emitted the existing synthetic HTTP 500/429 cleanup ResourceWarnings. No retry
of this full-suite run was needed; the pre-fix failures remain recorded above.

Both governance commands passed: **3/3 active plans** and **116 current /
126 archived sprints**. `git diff --check` passed. Only `owner_tmux.py`,
`test_owner_tmux.py` and this note changed; `decision_manager.py` and its tests
are unchanged. Safe test-isolation guidance was read before testing. Tests used
disposable synthetic profiles, private TMUX sockets and local SDK HTTP fixtures;
no live profile, native credential prompt, Rust test, Slack message, push or
release was involved. Existing internal-only qualification limitations below
remain open for the manager; this commit is a review handoff.

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
