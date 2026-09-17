# RETURN — owner-outstanding-104

Runtime: gpt-6-astra / high. Allocation digest:
`7139df5541a3279951c49e169bbc7907c8882a22aaea861d0ae233a8a4552bea`.
Claim: `159299cf-e49b-4892-bb62-ca206fb83761`.
Starting HEAD matches assigned base `6ed3300006a133c6ea1b862504109a5b924fd76a`.
Brief SHA-256 verified before other reads:
`6a8ce1473dccb100ee3e312933da513c8e18a78a93d8f27a968683eafb177226`.

Bounded reliability fix under product-spec heading **Internal delivery control
— TO BUILD**: “durable event dispatch, acknowledgments and watchdog”; “The
matching Slack thread should log his answer and route it to the appropriate
agent through a verified identity/revision-aware manager handoff”; “No arbitrary
Slack reply grants operational authority.” Existing initiative-delivery-control /
PF-80-S01 context; no plan/sprint advancement, deployment or acceptance claim.

## Outstanding intake and the path back

The manager and feed projection now count held envelopes plus quarantine
entries requiring review, rather than the lifetime audit total. New journals
keep a bounded `quarantine.outstanding` count and oldest timestamp, separately
from the unchanged lifetime total and latest 128 audit records. Audit pruning
cannot reduce the outstanding count. Explicit `unbound-expired` records remain
in history but do not count as live quarantine.

A successful existing exact ingress/session/epoch gap review clears the
quarantine summary in the same durable write as qualification. Failed writes
leave the outstanding obligation intact. Review does not discard held
envelopes: they remain counted until replay or durable expiry. The next fresh
projection can become healthy once outstanding intake, transport holds/fence
gaps, supervisor faults and other existing blockers have cleared. There is no
automatic transport hold clearance and no authorization inferred from a reply.

Projection age is based on the earliest outstanding held arrival or quarantine
timestamp. A reviewed historical incident does not affect a later incident's
age. No current intake means the optional quarantine projection is absent.
Saved observations recompute age without pretending the underlying observation
is fresh.

Legacy journals are read without rewriting them during projection. Retained
records are filtered by the existing review ingress boundary and expiry reason.
Old pruned records lack per-record dispositions: they conservatively remain
review obligations with unknown age until a review covering the journal, or
a new successful exact review, clears them. Their count is a conservative
legacy obligation count, not proof that every missing record is a live reply.
No migration can reconstruct which missing entries had already expired.
After review, new count/age tracking is exact and does not inherit that unknown
history.

## Supervisor precedence and route order

Quarantine no longer overwrites `observation-unavailable`,
`observation-stale`, `event-flush-failed` or `event-unflushed`. The reason
reports the supervisor fault while the separate quarantine object still
reports count, held count and age. Saved intake-only failures also undergo the
five-second supervisor freshness check; they cannot freeze a dead supervisor
at its previous observation. Existing flush failure counters remain visible.

Settlement now looks up the exact thread route first. At the deadline or
after it, a valid still-retained reply enters that route and drains once.
A full delivery queue retains it for retry, even after the deadline. Root/detail
messages still become invalid-message records, including at the deadline.
Only an unroutable reply expires. Previously committed expiry cannot be
resurrected, because its original content has already been removed.

The discarded-before case is a human reply held pending binding, with
`bind_alert` installing the matching route exactly 900 seconds after arrival
(or later), before any operation durably expires it. The old branch discarded
it as unbound-expired; the new branch enqueues and delivers it once. Failed
binding writes retain the original envelope for retry.

## Operator answer

In the structured manager/feed status, a held reply shows Slack held with
`quarantine.held > 0`; quarantined-only intake shows held with
`quarantine.count > 0` and `held = 0`, with the oldest outstanding age;
a stale supervisor shows `reason = observation-stale` even when quarantine
coexists, and unavailable observation similarly takes precedence; when
everything is fine the supervisor is healthy, quarantine is absent, and Slack
is last-verified. Those structured fields distinguish the four conditions,
which can coexist. **The current HTML renderer still shows only the coarse
Slack observation label**: held and quarantined look alike, stale plus
quarantine also looks held, and a stale supervisor alone can look last-verified
like the fine state until the broader Slack cache becomes stale. It does not
print supervisor reasons, held/count or age. `attention.py` and
`decision_feed.py` are outside this allocation's writable scope, so an
unqualified claim that the four are visually distinguishable would be false.

## Qualification boundary and brief corrections

The four code defects in the brief were present. The earlier “unresolved
quarantine total” description was wrong: it was a lifetime total. Exact current
classification of already-pruned legacy entries is unrecoverable without
additional evidence. The brief's requested four-state operator distinction is
available in structured data; making that distinction visible in HTML requires
a renderer allocation.

This is an implementation return to the Fable manager, not human-test readiness
or release qualification. Independent code-blind design/execution/evidence
review, applicable packaged TUI proof and named-human acceptance are not
claimed; those later functional handoff gates remain with the manager.
No live listener, transport journal, coordinator or personal profile was used.
No credentials were read, no native credential prompt occurred, and no commit
or push was made. No workspace formatter or Rust test was run.

Verification counts, raw attempts and changed-line accounting are recorded in
[the verification record](owner-outstanding-104-verification.md).
