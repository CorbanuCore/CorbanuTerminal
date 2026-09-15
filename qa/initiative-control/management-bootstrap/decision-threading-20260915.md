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
