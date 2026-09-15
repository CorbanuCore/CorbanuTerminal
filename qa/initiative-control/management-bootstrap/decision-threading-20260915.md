# RETURN — decision-threading-04: retryable pre-admission refusal

Status: scoped correction of the accepted separate-thread design; offline
implementation return to the Fable manager. Earlier rejected designs and all
prior evidence remain below.

## Allocation and authority

- Worker: `gpt-6-astra`, effort `high`; action `decision-threading-04`.
- Allocation digest:
  `340316bd2a0657ebc68a0305aaee624a198fd6886b8463fa8654edd7da00f2d3`.
- Claim: `2a8d82ab-b50c-45b3-b212-548eb5a18408`.
- First read and SHA-256 verification:
  `/private/tmp/fmgr.Q1SIYZ/briefs/decision-threading-04.json`,
  `fb8ff89bdc9260e70765d56ff754213a3e7fc8b45ebbe563e2caf5bdb850cfd4`.
- Base: `776dac8dc2ae33e685a5a96e4b5301cf00364804`; branch:
  `bootstrap/decision-threading-20260915`; worktree:
  `/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-decision-threading-20260915`.
- Classification remains a correction within active initiative
  `initiative-delivery-control`, PF-80, sprint PF-80-S01 (`in_progress`).
  Product heading: **Internal delivery control — TO BUILD**; excerpts:
  “actual Slack reply/decision/agent acknowledgment” and
  “durable event dispatch, acknowledgments and watchdog”.
- The explicit allocation supplies worker coordinates and exact file scope.
  Shared plan/sprint coordinates still name the manager checkout; their
  reconciliation remains manager-owned and outside this worker's write scope.
  The sprint checker passed before editing.

## Independent review and dispositions

Read `/private/tmp/fmgr.Q1SIYZ/dthr3-review.json` before implementation.
The Opus 5.0 High review accepted the separate-thread design and found one
blocking P2 plus a P3 payload issue. This correction adds no new review pass or
approval claim.

**P2 — corrected.** The transport now reports `NotDispatched` only when a
pre-dispatch exception is followed by a locked journal read proving that this
attempt has no `posts` entry. Root/details and notice senders preserve
`pending` for that exception. The request remains stable and can be tried by
a later admitted send. A recorded attempt, failed post-write fence check or
unreadable journal retains the existing fail-closed uncertainty behavior.
Post-dispatch exceptions still require positive reconciliation and never cause
an automatic retry.

The reconcile branch deliberately retains ordinary send admission. It may
prepare the pointer and check admission after binding the follower's own thread,
but held/sessionless recovery makes no pointer POST and leaves the slot pending.
It does not widen `allow_hold`, assume a session or bypass either fence. A hold
arising between the initial gate and the locked dispatch check is handled the
same way. Parent rows and routes retain their accepted ownership semantics.

**P3 — corrected.** The native `message_mention` now has exactly `type`,
`channel_id` and `message_ts`; the unsupported custom `text` is removed.
Fixed surrounding text and the plain-text fallback remain. The loopback SDK
test asserts the actual transmitted element. This is schema/payload evidence,
not a live Slack acceptance claim.

The optional `invalid_blocks` reclassification was deliberately not taken:
this correction removes the malformed field and keeps all existing transport
error classifications identical, including auth/channel/rate-limit behavior.
An unexpected `invalid_blocks` response therefore still requires investigation
under the existing uncertainty policy; no new rejection or retry policy is
introduced here.

## Pre-fix failure proof

All five new cases were first run against unchanged production files at
`776dac8dc`: **5 tests, 5 failures, 5.375s, exit 1**. At that point
`git diff --stat` showed only the new manager tests.

A first corrected focused run was **8 tests, 3 failures, 7.801s, exit 1**.
All retry/payload assertions passed; the three recovery tests then reached an
incorrect test-only expectation of project status `active`. The existing
status contract calls the recovered state `last-verified`; the assertions
were corrected accordingly. This failed attempt is retained, not called a pass.

The final test definitions were then replayed with production definitions
loaded in memory from the exact base using `git show`: **5 tests, 5 failures,
5.354s, exit 1**. No working-tree production file was reverted or overwritten.

| Final new test in `test_decision_manager.ManagerTests` | Exact pre-fix failure | Corrected expectation |
| --- | --- | --- |
| `test_cli_reconcile_under_hold_leaves_pointer_pending_until_admitted_send` | `'uncertain' != 'pending'` | Pending, no pointer attempt, later qualified send posts once |
| `test_cli_reconcile_without_session_leaves_pointer_pending_until_admitted_send` | `'uncertain' != 'pending'` | Same behavior with no listener session and no hold |
| `test_cli_reconcile_held_and_sessionless_leaves_pointer_pending_until_admitted_send` | `'uncertain' != 'pending'` | Same behavior with both conditions |
| `test_pointer_pre_admission_refusal_never_writes_uncertain` | Captured writes were `['sending', 'uncertain', 'uncertain']` | A hold after the first gate never persists uncertain; later send posts once |
| `test_pointer_message_mention_contains_only_documented_fields` | Actual element had extra `text: Open follow-up thread` | Actual SDK request contains exactly the three documented fields |

Reproduction: use the isolated SDK interpreter/environment below with `-B -c`
and this driver, listing the five test names from the table:

```python
import importlib, subprocess, unittest, sys
base = "776dac8dc2ae33e685a5a96e4b5301cf00364804"
for name in ("decision_alerts", "slack_transport", "decision_manager"):
    module = importlib.import_module(name)
    path = "scripts/initiative_control/" + name + ".py"
    source = subprocess.check_output(["git", "show", base + ":" + path], text=True)
    exec(compile(source, base + ":" + path, "exec"), module.__dict__)
names = [
    "test_cli_reconcile_under_hold_leaves_pointer_pending_until_admitted_send",
    "test_cli_reconcile_without_session_leaves_pointer_pending_until_admitted_send",
    "test_cli_reconcile_held_and_sessionless_leaves_pointer_pending_until_admitted_send",
    "test_pointer_pre_admission_refusal_never_writes_uncertain",
    "test_pointer_message_mention_contains_only_documented_fields",
]
suite = unittest.defaultTestLoader.loadTestsFromNames(
    ["test_decision_manager.ManagerTests." + name for name in names])
result = unittest.TextTestRunner(verbosity=2).run(suite)
sys.exit(not result.wasSuccessful())
```

## Final-tree verification

Read `docs/development/test-isolation.md` before any test. Tests use disposable
private state, synthetic credential strings and local SDK HTTP fixtures.
No live profile or credential store was used, no native credential prompt
occurred, and no live Slack message or push was sent.

Exact full SDK command:

```sh
env -u CODEX_HOME -u CORBANU_HOME -u PFTERMINAL_HOME TMPDIR=/private/tmp PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts/initiative_control:/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/lib/python3.14/site-packages /Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B -m unittest discover -s scripts/initiative_control -p '*test*.py'
```

Full final-tree SDK result: **623 tests passed in 336.778s, exit 0**, no
failures, errors or skips. All five new cases passed, as did the existing
post-dispatch uncertainty, receipt reconciliation, auth/channel/rate-limit and
thread-attribution regressions. Synthetic HTTP 429/500 cleanup ResourceWarnings
were retained. Production and test files were unchanged throughout the run;
only this QA record was completed afterward.
Governance: `python3 docs/plans/check.py` passed, **3/3 active plans**;
`python3 docs/sprints/check.py` passed, **116 current / 126 archived**;
`git diff --check` passed. These validate manager records, not completed
reconciliation of this worker's coordinates.

## Handoff boundary

The Fable manager owns final independent review, receiving-tree integration
and shared ledger reconciliation. Applicable code-blind design, isolated
execution and independent evidence review, live Slack/link-navigation proof,
and human acceptance remain open before an unqualified functional handoff.
This worker has not declared an internal-only N/A approved, human-test
readiness, sprint completion, release qualification or benchmark success.
No Rust/TUI runtime changed; Rust tests and TensorCash/Isometric release
workflows were outside this correction allocation.

---

# Historical RETURN — decision-threading-03: separate threads with fixed pointers

Status: offline implementation candidate for manager review and integration.
The two earlier candidates below were **rejected**, and their shared-thread
design is removed. Historical attempts and their test results remain below;
they do not describe the adopted implementation or establish acceptance.

## Current allocation and authority

- Action: `decision-threading-03`; worker: `gpt-6-astra`, effort `high`.
- Allocation digest:
  `612ec9af0a896c18fbcca2ecd2aefc933e67201a890c2db61a2033abfa45d3d0`.
- Claim: `d46e56c1-c0c8-4b8b-8a30-658561903ef9`.
- Read the frozen brief first and verified its SHA-256 with `shasum -a 256`:
  `7d1524b98da0eb60799c2fadb3acb66d503c1a18b5125bfd261624d0c77f10d7`,
  at `/private/tmp/fmgr.Q1SIYZ/briefs/decision-threading-03.json`.
- Verified starting HEAD: `4aaf3cee183537119ac1fc36631d9373f8b9870e`.
- Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-decision-threading-20260915`.
- Branch: `bootstrap/decision-threading-20260915`.
- Classification: revision within product initiative
  `initiative-delivery-control`, feature PF-80, sprint PF-80-S01
  (`in_progress`). The initial commentary called this a bounded revision;
  the inherited initiative classification is authoritative because the
  relationship and projection contracts are persisted.
- Product heading: **Internal delivery control — TO BUILD**.
  Requirement excerpts: “actual Slack reply/decision/agent acknowledgment” and
  “durable event dispatch, acknowledgments and watchdog”.
- The explicit frozen allocation authorizes this worker's paths and coordinates.
  Shared plan/sprint records still name the manager checkout; the manager owns
  their reconciliation before integration. They are outside this write scope.

## Incident and both rejected attempts

Travis answered a decision in its Slack thread, saw no response there, and
asked, “You didn't reply to me in Slack?” The acknowledgment notice was skipped
without a visible outstanding signal, and the follow-up question appeared in
an unrelated thread with no pointer from the answered thread. A decision
relationship is useful; moving another question into the answered thread was
the wrong repair.

Both review JSONs were read before edits:

| Rejected candidate | Review | Why rejected |
| --- | --- | --- |
| `182818493ebca3fdf5aa3457be2bb6756b8b684d` | `/private/tmp/fmgr.Q1SIYZ/P1BuxK-review.json` | The follower copied its parent's root receipt. Between follower posting and route binding, including uncertainty/reconciliation, a reply to the follower could be silently recorded against the parent. Fallback provenance also lacked a status/projection surface. |
| `4aaf3cee183537119ac1fc36631d9373f8b9870e` | `/private/tmp/fmgr.Q1SIYZ/r7v5EI-review.json` | The new ingress fence required every alert in the shared thread to be sent. A terminally rejected follower could never satisfy that condition, permanently making the parent's posted question unanswerable. Reconciliation could also move the shared route backward to an older alert. |

## Adopted design

Every question alert, including a follower, now runs its own original two-phase
send: create a root message, then post its details under that root. No parent
request or receipt is copied. A chain therefore has distinct question threads.

For `follows`, enqueue pins the newest eligible alert for that parent decision:
same feed, same complete Slack identity, not cancelled, with confirmed root and
details receipts. It records `threading.mode=pointer` and `source_alert`, without
changing the parent's alert or feed revisions.

After the follower's own root and details are confirmed, its send invokes the
existing `notice` machinery with kind `follow-up` and the pinned source alert
as its basis. There is one durable notice slot per follower alert:
`digest([follower_alert, "follow-up", source_alert])`. Its only text is:

> A follow-up decision has been raised. Find it in its own thread: Open follow-up thread.

“Open follow-up thread” is a fixed-label Slack `message_mention` in a fixed
`rich_text` block. Its only variable fields are the validated pinned channel ID
and the follower's genuine root receipt timestamp. This uses the existing
`chat.postMessage` exchange; no additional API or transport is added. The notice
targets the **parent** thread. It contains no question, answer, manager text or
caller-supplied URL. Markdown and both unfurl flags remain false. Clarification
and acknowledgment notices still target their own question thread, with
unchanged fixed text.

During local review, the initial hand-built permalink was removed: Slack's
[permalink documentation](https://docs.slack.dev/reference/methods/chat.getPermalink/)
shows workspace-specific links, but this binding does not retain that domain.
The adopted native reference follows Slack's documented
[message mention element](https://docs.slack.dev/reference/block-kit/block-elements/message-mention-element/).
The SDK regression checks that the entire exact fixed block reaches the
loopback HTTP endpoint; actual Slack rendering/navigation remains a later
functional qualification gate.

The notice request, state and receipt are stored solely on the follower's
alert. Restarts do not repeat a sent, failed or uncertain pointer. Exact receipt
reconciliation can resolve uncertainty without reposting. A cancelled target
records a failed pointer without a POST. Identity or basis mismatch is refused.
The CLI details-reconciliation path binds the follower's own thread and then
runs the same send completion path to attempt its still-pending pointer.

If enqueue finds no eligible parent thread, it retains
`mode=new-thread`, `reason=followed-decision-has-no-eligible-slack-parent`.
The existing `new_thread_fallbacks` aggregate, per-question projection and
dashboard rendering remain. A late parent receipt does not silently change the
pinned choice.

### Why neither previous failure is reachable

- **No reply can be attributed to a different decision than the owner of the
  thread it arrived in.** Each root route names one alert; binding checks that
  alert's own root receipt and refuses to replace an existing different route.
  Callback attribution is a direct lookup by the incoming thread. The
  shared-thread selector and latest-follower routing logic are deleted.
  Reconciliation of an older alert can only reassert its own unchanged route.
- **No decision can be made unanswerable by another decision's state.**
  Callback admission no longer reads any alert rows or follower/pointer state.
  Pending, posting, uncertain, failed and unbound followers cannot gate
  admission on the parent thread. A pointer is just a fixed notice; it neither
  registers a question in the parent's thread nor changes that thread's route.
  Failed/refused pointer attempts leave parent replies attributable and
  interpretable through the manager's guarded resolution store.

Existing authentication, unknown-thread, ingress-loss and invalid-journal
refusals remain. A reply to an unbound new thread is held, never guessed to be a
parent reply. Superseded candidate rows that copied another alert's root request
are refused on alert validation rather than silently replayed or migrated.
No live store was inspected or converted.

## Retained sound behavior

- `unacknowledged_answers` stays outstanding until **both** real receiver ACK
  evidence and the corresponding acknowledgment-notice receipt exist.
  Read-only projection neither posts a notice nor creates receiver evidence.
- Strict `follows` ID, same-feed target, self-reference and cycle validation;
  immutable relationship; no parent revision rewrite or append when introducing
  a follower; existing CAS/history checks.
- Fallback visibility, legacy projection compatibility, receipt reconciliation,
  cancellation/identity fences and current-audit execution eligibility.

## Regression evidence

| Regression | What it proves |
| --- | --- |
| `test_follower_owns_thread_and_posts_one_fixed_pointer_without_parent_mutation` | Own root/details plus exactly one fixed parent-thread pointer; exact payload/flags; restart idempotency; parent alert/feed unchanged; arbitrary text and unbound basis refused. |
| `test_follower_chain_keeps_own_roots_and_preserves_send_refusals` | Chains keep separate roots; pointers target the immediate parent; identity/uncertain retry refusal survives; root reconciliation resumes correctly. |
| `test_rejected_shared_root_candidate_cannot_be_replayed` | A copied historical root request is not accepted as a follower's own request. |
| `test_follower_sdk_each_thread_attributes_only_its_own_question` | Real loopback SDK sends two distinct question roots and a fixed pointer, then each thread's reply resolves only its owner. |
| `test_pending_follower_leaves_parent_answerable`, `test_sent_unbound_follower_leaves_parent_answerable`, `test_follower_post_in_progress_leaves_parent_answerable` | Parent ingress and guarded interpretation work across pending/posting/receipt-before-binding windows. |
| `test_uncertain_follower_leaves_parent_answerable`, `test_failed_follower_leaves_parent_answerable`, `test_reconciled_unbound_follower_leaves_parent_answerable` | Lost response, terminal HTTP 429 and reconciled-but-unbound follower states cannot deadlock the parent. |
| `test_failed_pointer_leaves_parent_answerable`, `test_uncertain_pointer_leaves_parent_answerable_and_reconciles_without_reposting`, `test_refused_pointer_leaves_parent_answerable` | Pointer HTTP rejection, socket-response loss and identity refusal preserve parent ingress plus manager-guarded answer/resolution; no retry or invented receipt. |
| `test_unbound_follower_reply_is_never_attributed_to_parent` | Unknown follower thread retains the original hold; nothing is recorded against the parent. |
| `test_old_parent_answer_edit_stays_with_parent_after_follower` | Later edits remain in the parent's audit and deny outdated execution eligibility. |
| `test_cli_reconcile_binds_own_thread_posts_pointer_and_cannot_repoint_parent`, `test_route_binding_refuses_another_alert_on_an_existing_thread` | CLI recovery preserves both routes; older rebinding cannot steal the follower route; a colliding route is refused. |
| Existing `follows`, fallback and acknowledgment regressions | Malformed/unknown/self/cyclic relationships and parent changes refused; fallback rendered without journal mutation; receiver ACK alone does not clear the outstanding count; exact notice receipt does. |

Read `docs/development/test-isolation.md` before tests. All SDK work uses the
existing synthetic private fixtures and loopback endpoints, with live profile
aliases removed and `TMPDIR=/private/tmp`. No raw Rust tests, real credentials,
native credential prompts, Slack messages, pushes or release operations occurred.

Focused preliminary run: **63 tests passed in 40.005s**. After that run, parent
answer regressions were strengthened to use `ResolutionStore`, and comment
escaping was cleaned up. A full intermediate run then started. During that run,
local link-format review replaced the initial hand-built URL with the documented
native message mention. The intermediate run is not final-tree evidence.
The full final-tree run uses the exact SDK command retained in the historical
evidence below. Synthetic HTTP 429 cleanup warnings are retained as warnings,
not hidden.

Intermediate full SDK result: **618 tests passed in 328.227s**, exit 0.
This run began before the pointer payload correction; it is supporting evidence.
Corrected pointer focused result: **2 tests passed in 1.126s**, exit 0.
Full final-tree SDK result: **618 tests passed in 328.614s**, exit 0, no
failures or errors. The HTTP 500/429 cleanup ResourceWarnings were retained.
Production and test files were unchanged throughout this final run; only this
QA result was updated afterward. `git diff --check` passed.
Governance: both checkers passed, **3/3 active plans; 116 current and 126 archived
sprints**. They validate current manager records, not this worker allocation's
pending shared-ledger reconciliation.

## Handoff boundary

This is an implementation return to the Fable manager, not a human-test-ready
candidate. Independent review and current-candidate code-blind
design/execution/evidence review remain manager-owned and unclaimed. Live Slack
and actual interactive link/navigation qualification are excluded by this
allocation and remain open before functional handoff. No human acceptance,
internal-only N/A approval, sprint completion or release qualification is
invented. TensorCash/Isometric release workflows and benchmarks were not run.

---

# Historical RETURN — decision-threading-01 (rejected)

Status: implementation candidate for manager integration; no live Slack or human-test readiness claim.

## Allocation and authority

- Worker: gpt-6-astra, high, dispatched by Fable manager.
- Allocation digest: `5f16355a4955e4faa0a2ff13f1905b13bad41569375c21f1358bb34f79d039e0`.
- Claim: `d06ef2b9-a3c9-40f9-a484-be9453bfaf62`.
- Frozen brief SHA-256 verified with `shasum -a 256`:
  `4ec8dddfc8b3a8cec170d8805fda34231f5ea3e2a58c42638c2f9f85b5945155`.
- Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-decision-threading-20260915`.
- Branch: `bootstrap/decision-threading-20260915`.
- Verified starting HEAD: `34590180808b06c35234afe144c16b4b07b300a3`.
- Class: product initiative revision within active `initiative-delivery-control`,
  PF-80, sprint PF-80-S01 (`in_progress`), because the approved repair extends
  the persisted decision relationship and projection contracts.
- Product citation: **Internal delivery control — TO BUILD**, requirement
  excerpt: “actual Slack reply/decision/agent acknowledgment”; also “durable
  event dispatch, acknowledgments and watchdog”.
- The frozen user allocation supplies this worker's exact coordinates and nine
  writable paths. Shared plan/sprint front matter still names the manager
  checkout. Manager owns reconciliation before integration; neither shared
  record is writable by this worker.

## Incident

On September 15, Travis answered a decision in its Slack thread. The manager
interpreted the answer and raised a follow-up as a new top-level thread, leaving
no response where Travis had replied. Travis asked, “You didn't reply to me in
Slack?” The receiver handoff was skipped, so no genuine receiver acknowledgment
or acknowledgment notice existed. The decision feed also lacked a relationship
that could retain the follow-up in the original conversation.

## Changes

### Outstanding acknowledgment

`decision_manager.project_status` and `decision-slack project-status` expose
`unacknowledged_answers`, a count of distinct interpreted decisions without both
retained receiver ACK and a confirmed acknowledgment-notice receipt. Recorded
intents count before and after resolution persistence, including the crash
boundary before the queue write. Delivered-but-not-acknowledged, missing-notice,
and failed/uncertain-notice cases remain outstanding. A held session does not
hide a count already read from intact local journals.

`decision_feed.project_slack` publishes the aggregate under `status` and the
per-question count under `decisions[].replies`. The existing dashboard renders
that field as **unacknowledged answers**, including for a resolved question.
Legacy schema-1 projections remain readable without inventing a historical
count. New count values must be nonnegative integers, never booleans; existing
unknown-field and projection identity/digest refusals remain.

Projection posts nothing and creates no ACK evidence. The manager must still
perform the receiver handoff and fixed acknowledgment notice. A receiver ACK
alone does not falsely claim that Travis received the notice.

### Decision relationships and thread provenance

Schema 1 accepts one optional decision-level field alongside the existing
required `id` and `revisions`: `"follows": "existing-decision-id"`. Revision
objects retain their existing exact schema. `follows` is a nonempty string matching the existing
80-character ID grammar. Null, wrong types, unsafe/malformed IDs, unknown IDs,
self-reference and cycles are rejected. The target must exist in the same feed;
array order does not define the relationship. The relationship is immutable
once saved, including adding it to an existing unrelated decision, removing it
or retargeting it. Adding a follower cannot rewrite or append to its existing
parent's revisions in the same feed transaction. The original history-prefix,
revision, time, private-file and compare-and-swap checks remain in force.

For a follower, enqueue selects the newest confirmed parent-thread receipt for
its followed decision under the exact same feed and Slack identity. Cancelled
or different-identity parent alerts are ineligible. The follower explicitly
retains the genuine parent request/receipt and records
`threading.mode=followed` with `source_alert`. Its own details post includes its
headline, dashboard link, follows ID and question/context, all through the
existing fixed builder. Its receipt, retries, replies and fixed notices remain
separate, while Slack uses the same root thread. Chained followers retain that
root. The existing transport route points subsequent replies at the newly bound
question; manager interpretation still binds the alert, question, audit and
allocation before resolving it. Once a thread is rebound to a follower, the
previous question cannot receive an execution permit: the manager requires the
current thread route to still name the handoff alert. Later edits now route to
the follower, so the prior question must stay held rather than treating its old
audit as current. This adds a refusal without modifying the parent decision.

If no eligible confirmed parent exists at enqueue, the record says
`threading.mode=new-thread` and
`reason=followed-decision-has-no-eligible-slack-parent`. It uses the existing
parent/details send flow. This choice is retained across restart; late parent
receipts do not silently reroute queued or uncertain work. Unrelated decisions
keep their original thread behavior. No original alert row or decision revision
is modified by enqueueing or sending its follower.

## Regressions and evidence

- Graph validation: legacy shape and valid chains accepted; malformed, unknown,
  self and two-/three-node cyclic links rejected.
- Persistence: follower creation leaves the parent identical; attempted parent
  rewrite or appended revision fails without changing saved bytes; relationship
  removal and retargeting are rejected.
- Same-thread send: exactly one follower details exchange under a genuine root
  receipt; full new question identity and fixed rendering flags retained; parent
  alert and feed unchanged; restart does not duplicate; notice stays in-thread.
- Missing-thread fallback: explicit reason and normal two-phase send; later
  parent delivery cannot change the pinned choice.
- Chains/refusals: same root across generations; changed identity, cancellation
  and uncertain-send retry refusals retained; cancelled/different-identity roots
  are ineligible.
- SDK loopback: follower posts to root, callback/drain routes the human reply to
  the follower, and interpretation/resolution leaves the original decision and
  original alert unchanged.
- Shared-thread execution fence: an earlier handoff with retained receiver ACK
  stays held after follower rebinding, including after a later edit to the old
  answer; the original decision stays unchanged.
- Outstanding acknowledgment: resolved-but-skipped handoff appears in direct
  status, CLI status, cached projection and rendered dashboard without posting
  or mutating alerts/replies; delivery and receiver ACK alone do not clear it;
  wrong receiver evidence is refused; uncertain notice remains outstanding;
  exact receipt reconciliation clears the count.
- Crash/held state: retained recorded intent after resolution and loss of the
  listener session keep the count visible.
- Transfer/schema: follows and new count survive a pinned publication transfer;
  legacy projections still validate; invalid count types/values are refused.

Read `docs/development/test-isolation.md` before tests. All tests use synthetic
private fixtures and loopback SDK endpoints, with live profile aliases removed.
No native credential prompt, live profile access, live Slack operation, push,
Rust test or release operation was performed.

Initial focused run: **39 tests passed in 4.918s**. Initial full SDK run:
**603 tests passed in 314.842s**; retained HTTP-fixture cleanup ResourceWarnings
for synthetic HTTP 500/429 responses, no test failures. While that suite was
running, local review found the shared-thread execution fence needed tightening.
The added fence regression passed separately (**1 test in 1.256s**). A fresh full
suite on the final implementation follows; the initial run is not claimed as
final-tree evidence.

Exact full SDK command (both runs):

```sh
env -u CODEX_HOME -u CORBANU_HOME -u PFTERMINAL_HOME TMPDIR=/private/tmp PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts/initiative_control:/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/lib/python3.14/site-packages /Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B -m unittest discover -s scripts/initiative_control -p '*test*.py'
```

Final full SDK: **604 tests passed in 313.052s**, exit 0, no failures or
errors. The same synthetic HTTP 500/429 cleanup ResourceWarnings were retained.
This run includes all **11 new regression methods** and the final execution
fence. Plan and sprint checks passed: **3/3 active plans; 116 current and 126
archived sprints**. `git diff --check` passed.
These checkers validate existing manager records, not this allocation's pending
shared-ledger reconciliation.

## Integration and qualification boundary

The assigned repair is implemented offline. No existing operational refusal was
relaxed; no free-form notice route, automatic posting, automatic handoff,
execution unlock or fabricated acknowledgment was added. The new graph and
parent-preservation checks only tighten accepted writes.

This worker return does not qualify the affected user workflow or mark PF-80-S01
complete. Manager-owned independent review, code-blind design/execution/evidence
review and applicable interactive/live Slack qualification remain required
before an unqualified human-test handoff. Live execution is explicitly excluded
by this allocation. No acceptance, N/A approval, recurrence or release approval
is inferred. TensorCash/Isometric release qualification and benchmarks were not
run; this is not a release candidate.

# RETURN — decision-threading-02 revision

Status: offline revision for manager review/integration; no live or human-test
qualification claim. This section supersedes the earlier claim that no ingress
refusal was relaxed: independent review found a real shared-thread attribution
gap in candidate `182818493ebca3fdf5aa3457be2bb6756b8b684d`.

## Allocation and reviewed findings

- Action: `decision-threading-02`; worker: gpt-6-astra, high.
- Allocation digest: `880a736fb4cd80f5611166b806e199cd9d0d90cea0315163ecdfd816208d8814`.
- Claim: `6c216406-8b85-4b49-86d0-e741b68dd228`.
- Frozen brief: `/private/tmp/fmgr.Q1SIYZ/briefs/decision-threading-02.json`.
  SHA-256 verified before review/code reads:
  `c71d6e3ba66b5debba7b7f5839eff7a6237161827a6c11ef1f13f948dccc186e`.
- Review read first after the brief:
  `/private/tmp/fmgr.Q1SIYZ/P1BuxK-review.json`, overall “patch is incorrect”.
  P2: shared-thread ingress could name the parent before follower binding,
  including after reconciliation. P3: fallback provenance was not projected.
- Verified branch/worktree remain those above; starting HEAD exactly
  `182818493ebca3fdf5aa3457be2bb6756b8b684d`.
- Classification remains a revision within the approved product initiative,
  active `initiative-delivery-control`, PF-80, PF-80-S01 `in_progress`.
  Product citation: **Internal delivery control — TO BUILD**, “actual Slack
  reply/decision/agent acknowledgment”. Manager still owns shared allocation
  ledger reconciliation; this worker has no write scope for plan/sprint records.

## P2: ingress admission closes every send/bind gap

The SDK callback now checks retained alerts under the transport lock before it
records a shared-thread event. It reads atomic alert snapshots without taking
the offline store lock: send holds that lock across the network call, so taking
it in the callback would block ingress. The transport lock also serializes
outbound request reservations. Enqueue durably records a follower before its
send can post; send durably records `sending` before exchange. Thus a callback
before either reservation cannot observe a posted follower, and a callback
during/after its POST sees the follower even if receipt persistence or route
binding has not completed.

For multiple alerts sharing the root and identity, admission requires every
details phase to be confirmed sent, and the route to exactly name the alert
with the latest confirmed details timestamp. Pending, sending, uncertain,
failed and current sent-but-unbound questions hold ingress through the existing
durable `ingress-held` fence. No event is acknowledged or inserted against the
parent as a guess. This also covers direct/offline reconciliation and restart:
the fence depends on retained rows, not on remembering to call a send wrapper.
Cancelled attempts are not assumed unseen.

The CLI details-reconciliation branch binds the confirmed alert, allowing
subsequent new replies to reach the follower. The interval before that route
write remains protected by the same callback check. Reconciliation of notices
does not repoint the active question. A reply predating the newest details post,
or an edit/deletion of a retained answer belonging to an older alert, holds
instead of becoming a follower answer. Existing prior-work fences remain;
the old-answer edit test now requires an ingress hold as well.

A hold is conservative and durable: binding later does not silently clear it
or replay a lost event. Existing owner recovery/gap-review requirements remain.
No retry loop, free-form posting, fabricated receipt/ACK, parent-row mutation,
or change to `follows` validation was introduced.

## P3: operator-visible fallback

`decision_manager.project_status` (including the status CLI) now exposes the
aggregate `new_thread_fallbacks` count. `decision_feed.project_slack` publishes
that count under `status` and a per-question indicator under
`decisions[].replies.new_thread_fallbacks`. The existing dashboard displays
**new thread fallbacks: 1** on the affected decision. The exact fixed fallback
reason remains in its retained alert provenance.

All reads use `row.get("threading", {}).get("mode")`, so older and unrelated
alert rows remain readable. Older schema-1 status/projections may omit the new
field; new values must be nonnegative integers, with the existing per-question
bound. Projection only writes its own cache; it does not post or alter alerts,
replies, receiver ACKs or notices.

## Discriminative regression evidence

Eight new methods in `test_decision_manager.ManagerTests`:

| Test suffix | Pre-fix result | Revised expected result |
| --- | --- | --- |
| `shared_thread_unbound_follower_holds_during_post` | FAIL: listener stayed active | Hold while accepted POST still has `sending` alert state |
| `shared_thread_pending_follower_holds` | FAIL: listener stayed active | Hold before follower send |
| `shared_thread_sent_unbound_follower_holds` | FAIL: listener stayed active | Hold between receipt persistence and route write |
| `shared_thread_uncertain_follower_holds` | FAIL: listener stayed active | Hold after synthetic socket drop following acceptance |
| `shared_thread_bound_follower_holds_delayed_old_reply` | FAIL: listener stayed active | Hold delayed reply predating follower details |
| `shared_thread_reconciled_but_unbound_follower_holds` | FAIL: listener stayed active | Hold after positive history reconciliation without binding |
| `cli_reconcile_binds_follower_and_attributes_reply` | FAIL: reply alert was parent key | CLI reconciliation binds; callback/drain names follower; parent unchanged |
| `fallback_surfaces_without_mutating_legacy_alerts` | ERROR: missing `new_thread_fallbacks` | Status/projection/render visible; legacy accepted; invalid counts rejected; no posts/journal mutation |

Initial four-case red run against unchanged production code: **4 tests in
3.715s; 3 failures, 1 error**. Its during-post assertion initially ran inside the
exchange callback and was caught as uncertainty by send. The assertion was
moved outside that callback before the definitive red replay below.

Definitive red replay: current eight tests with production definitions loaded
from `git show 182818493ebca3fdf5aa3457be2bb6756b8b684d:scripts/initiative_control/<module>.py`
for `slack_transport`, `decision_manager` and `decision_feed`, executed into
their imported module namespaces in the test process. No tracked files were
reverted. **8 tests in 8.330s; 7 failures, 1 error**, each as listed above.

Initial focused revised run: **26 tests in 38.787s, all passed** (four initial
regressions, existing successful follower routing, full feed module). Final
focused revised run: **9 tests in 9.565s, all passed** (all eight new methods
plus strengthened prior-answer edit/work fence).

Read `docs/development/test-isolation.md` before tests. The exact SDK command
and isolated environment are the same as recorded above, including
`TMPDIR=/private/tmp`. Only private synthetic stores, synthetic credentials
and loopback SDK HTTP fixtures are used. No live profile, native credential
prompt, Slack message, Rust test, push or release operation occurred.

Final full SDK result: **612 tests passed in 317.362s**, exit 0. This includes
all eight new regressions and the strengthened old-answer edit test. Synthetic
HTTP 500/429 cleanup ResourceWarnings were retained; no failures or errors.
`git diff --check` passed on the final tree. Only the QA result was updated
after the full run; production and test files were unchanged.
Governance: **3/3 active plans; 116 current and 126 archived sprints**, both
checkers passed before the full suite. These validate the existing manager
records, not pending reconciliation of this worker's exact allocation.

Manager-owned independent review and functional/live qualification remain
open as recorded above. This return does not mark the sprint complete or
claim a human-test-ready candidate, human acceptance, benchmark results or
release qualification.
