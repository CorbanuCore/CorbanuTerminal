# RETURN — owner-rearm-106

Runtime: gpt-6-astra / high. Allocation digest:
`1c4d0ab1849f3dfe8c4fed9aaf873f6e3433a78235d8061b9172c7b2ecceedbf`.
Claim: `58254855-ab4f-4180-90d3-c04cd1efa9f3`.
Starting HEAD matches assigned base `8ef2e1c070f78093b295c4fe734e347c992eee39`.
Brief SHA-256 verified before other reads:
`962dcc12bca7f957011fa4df0e3bb21ab82b05ed426ac07a8c636437a503d2ab`.

Bounded reliability repair under product-spec heading **Internal delivery control
— TO BUILD**: “durable event dispatch, acknowledgments and watchdog”; “The
matching Slack thread should log his answer and route it to the appropriate
agent through a verified identity/revision-aware manager handoff”.
Existing initiative-delivery-control / PF-80-S01 context. This worker does not
advance a plan/sprint, qualify a release or claim human-test readiness.

## Legacy disposition and no re-arming

Legacy journals without an outstanding summary now count only retained,
unreviewed, non-expired records as outstanding. The missing pruned prefix
has an **unknown disposition**, separately represented as `unknown`.
An old review is no longer compared with the moving journal ingress counter:
unrelated callbacks cannot turn missing history into outstanding quarantine.

Missing legacy dispositions are not reconstructed from a lifetime total,
even when a historical review is present. Consequently a legacy journal can
report unknown both before and after unrelated ingress, with zero outstanding
records in both cases. This is the brief's explicitly allowed unknown-history
alternative, not a claim that the missing entries were resolved or expired.

The unknown count survives a new quarantine append in the durable summary.
A successful existing exact ingress/session/epoch review writes
`outstanding = {count: 0, oldest_at: null}` in the same durable qualification
write, removing both current obligations and unknown legacy history.
A failed final write leaves both untouched. Reopening the store, receiving
unrelated callbacks and later receiving a fresh rejected event do not
resurrect the cleared prefix; the fresh event has its own count and age.
No review, transport hold, route or authorization boundary was relaxed.

With only missing history and an otherwise healthy observation, manager and
feed status report `unknown`; supervisor health reports `unknown` with reason
`quarantine-history-unknown`, zero outstanding count and no outstanding age.
The existing renderer can display its generic “Slack observation: unknown”
label. Actual transport holds remain held. Known outstanding intake, flush
failures and missing/stale supervisor observations retain their existing
precedence, with unknown history still disclosed separately.

Regression coverage includes legacy review followed by unrelated ingress;
manager, live supervisor and saved feed projection; stale observation
precedence; independent transport holds; persistence across new quarantine;
failed final review write; reopen after successful review; later fresh intake;
and invalid unknown-count encodings. Existing bounded-pruning review coverage
also passes in the targeted replay.

## Renderer allocation blocker and four-state answer

Reported immediately upon locating the renderer: `attention.py` owns the HTML
notice; `decision_feed.py` collapses unhealthy intake to held and uses the old
cached health when selecting the displayed Slack state. Both are outside the
frozen allocation. **Items 2 and 3 cannot be claimed complete.** Their files
were not changed and no in-scope rendering workaround was introduced.

The final-code fixture observations are preserved in
[the five observation rows](owner-rearm-106-observations.jsonl).
Here is what the operator actually sees in this return:

| Condition | Current rendered notice begins | Distinction |
| --- | --- | --- |
| Held reply awaiting a route | Slack observation: held | Still identical to quarantined |
| Quarantined reply requiring review | Slack observation: held | Still distinguished from held only by structured count fields |
| Stale supervisor, otherwise no blocker | Slack observation: last-verified | Still looks like the fine state until the broader Slack cache expires |
| Everything fine | Slack observation: last-verified | Fresh healthy supervisor exists in structured data only |
| Stale supervisor plus quarantine | Slack observation: held | Stale reason remains structured, not rendered |

Each notice still ends: “This is a saved observation, not a live connection or
work authorization. If held or stale, answer in the manager task.”
Therefore **I cannot confirm that no pair is distinguished only by a number**.
Held and quarantined remain visibly identical; stale and fine can also remain
visibly identical.

The required wider allocation is:

- `scripts/initiative_control/decision_feed.py` and `test_decision_feed.py`:
  preserve distinct held/quarantined states in projection and publication;
  select freshness from re-assessed supervisor health, not the cached original.
- `scripts/initiative_control/attention.py` and `test_attention.py`:
  render explicit, different notices and assert their visible text.
- The already allocated manager/transport tests can then exercise the shared
  state selection through all surfaces.

Proposed notices for the widened follow-up, **not implemented**:

| State | Required visible meaning |
| --- | --- |
| Held | Reply held: waiting for its matching route; delivery is not yet confirmed. |
| Quarantined | Reply quarantined: not delivered. Human review is required; answer in the manager task. |
| Supervisor stale | Supervisor observation is stale. Delivery is unverified; check the listener or answer in the manager task. |
| Last verified / fresh supervisor | Slack last verified; supervisor observation is fresh and no outstanding held or quarantined replies are reported. |
| Unknown legacy history | Legacy quarantine dispositions are unknown; missing history is not counted as an outstanding reply. |

Combined conditions need combined notices: a quarantine must not hide a stale
supervisor, and stale supervision must not hide a held reply. Display saved
observation times without treating them as live-delivery or work authorization.

## Brief corrections and qualification boundary

The re-arming and identical rendering findings are valid. One wording
correction: a held reply is waiting for a route, but “will arrive” is not an
unconditional guarantee. Retained unroutable replies can expire after the
existing 900-second limit; successful routing and delivery still need evidence.
Also, already-pruned legacy dispositions cannot be recovered from a numeric
lifetime total. Unknown is an explicit limitation, not an outstanding alarm.

This is an implementation return to the Fable manager. Independent code-blind
design/execution/evidence review, applicable packaged true-TUI proof and
named-human acceptance are not claimed. The visual distinction remains an
open functional gate requiring the allocation above.

Read `docs/development/test-isolation.md` before testing. Tests used disposable
stores/profiles and synthetic SDK fixtures only; no live listener or profile,
credential inspection, native credential prompt, Rust test, workspace formatter,
commit or push was used. Exact commands, counts, failed attempts and changed
lines are in [the verification record](owner-rearm-106-verification.md).
