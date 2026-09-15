# RETURN — decision-threading-01

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
