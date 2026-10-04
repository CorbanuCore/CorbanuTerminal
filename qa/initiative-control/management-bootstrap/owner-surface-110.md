# RETURN — owner-surface-110

Runtime: gpt-6-astra / high. Allocation digest:
`d523169b60624efdfa8ef2280d6479293f5c218920913d5e8e49ca8da9410c5b`.
Claim: `27bcf789-24d8-4033-8f23-30f33044c2cc`.
Assigned base/starting HEAD:
`932d2e4be7f2eb85cad69357ba2c411f80e3f72f`.
The frozen brief hash matched before implementation.

Bounded reliability fix under exact product heading **Internal delivery
control — TO BUILD**: “durable event dispatch, acknowledgments and watchdog”
and “Show blockers, rendered sprints, human test plans, machines, run logs
and freshness”. Existing initiative-delivery-control / PF-80-S01 context;
this worker does not advance manager-owned plan/sprint status.

## Saved-snapshot read path

`decision_feed.slack_health` now bases the saved observation's effective state
on the reassessed supervisor health. A previously verified snapshot becomes
`stale` with condition `supervisor-stale` after the five-second supervisor
window, or `unknown` with condition `supervisor-unreadable` when the
observation is missing, invalid or unreadable. The display uses the distinct
condition. No snapshot rewrite, new projection or supervisor write is needed.

The regression goes through export, activation, pinned snapshot read and
publication at zero, five and six seconds, checks exact rendered labels and
state, and verifies that the saved bytes and in-memory snapshot are unchanged.
It separately covers absent/malformed/future-dated observation metadata.

The existing saved state enum remains compatible. Read-time conditions separate
`supervisor-stale` from `verification-expired` and `snapshot-expired`;
the latter two name the fifteen-minute windows. When supervisor uncertainty
and verification expiry coexist, the notice also names verification expiry.

## Qualification as a separate unit

`Transport.qualify` holds a store-wide, nonblocking `flock` on
`.qualification.lock` across both journal phases, authentication and cleanup.
A competing transport instance or spawned process refuses before credentials,
auth I/O or its own hold write. Ordinary transport callbacks retain their
short-lived transport lock; the network call does not hold the store or
transport journal locks.

After a persisted qualifying hold, a catchable failure restores the exact
previous hold under the existing store/transport locks. It changes only that
field and only while it still reads `qualifying`; a newer fault hold is
preserved. Cleanup waits for an admitted callback's journal lock rather than
abandoning restoration on ordinary lock contention. An intervening accepted
event and its ingress evidence survive a failed qualification without appending
a review or advancing verification.

The new test covers KeyboardInterrupt during auth, authentication failure and
failed final persistence, with both null and already-held starting states.
It reopens the journal and checks exact equality after each failure, then
qualifies successfully on retry. A separate test uses a spawned process plus
a second transport instance to prove single-flight refusal.

[Actual interrupted replay](owner-surface-110-interrupt.json):

```text
before_hold=null
during_hold=qualifying
interrupted_by=KeyboardInterrupt
after_hold=null
reopened_journal_unchanged=true
retry=qualified
```

**Boundary:** SIGKILL/other uncatchable process death cannot run Python cleanup.
An unavailable cleanup storage write can also leave a hold. Those cases still
need explicit owner recovery; automatic restoration for them is not claimed.
The flock is released by process exit, but release alone does not rewrite the
journal.

## Held notices and recovery

Waiting for route, quarantined intake and retained expired-undelivered evidence
now produce different text with wait, act and no-replay instructions.
An optional validated `expired` count carries retained expiry evidence into the
saved projection; it does not clear a hold or claim delivery. Mixed states keep
their separate disclosures. Reviewed old expiry records alone do not downgrade
a later healthy observation.

[Revised recovery step 2](owner-surface-110-recovery.md) names the exact
existing local operation:

```sh
python scripts/initiative_control/decision_manager.py drain --store <operator-store>
```

Supply one newline-terminated `{"binding": <exact pinned binding object>}`
on stdin and omit `--live`. Inspect the returned drained count and pending
events; each call is bounded to ten events. A refusal or stalled drain requires
investigation. Drain is local intake/disposition, not answer delivery, approval,
quarantine clearance or qualification. The complete revised procedure names
the subsequent exact-review and qualification steps too.

## Six situations — verbatim rendered text

Captured with disposable transport stores and the actual feed/attention
renderers. [Raw observations](owner-surface-110-observations-1.jsonl) include
the seventh saved-snapshot-aging regression.

| Situation | Verbatim rendered notice |
| --- | --- |
| Healthy | Slack observation: last-verified; assessed 2026-09-12T12:00:00Z; last verified 2026-09-12T12:00:00Z. This is a saved observation, not a live connection or work authorization. No transport action is needed for this observation. |
| Held-waiting | Slack observation: held; assessed 2026-09-12T12:00:00Z; last verified 2026-09-12T12:00:00Z. This is a saved observation, not a live connection or work authorization. Waiting for a route: wait for the manager to bind the reply route. Do not resend while it is held; arrival is not guaranteed. |
| Quarantined | Slack observation: held; assessed 2026-09-12T12:00:00Z; last verified 2026-09-12T12:00:00Z. This is a saved observation, not a live connection or work authorization. Quarantined reply: act by asking the manager to review its disposition. Delivery is not established. |
| Expired | Slack observation: held; assessed 2026-09-12T12:15:00Z; last verified 2026-09-12T12:00:00Z. This is a saved observation, not a live connection or work authorization. A reply expired undelivered while waiting for a route. Do nothing to replay that expired reply; if an answer is still needed, answer in the manager task. |
| Supervisor stale | Slack observation: supervisor-stale; assessed 2026-09-12T12:00:06Z; last verified 2026-09-12T12:00:00Z. This is a saved observation, not a live connection or work authorization. Supervisor observation is over five seconds old. Ask the manager to check the supervisor and refresh the observation. |
| Supervisor unreadable | Slack observation: supervisor-unreadable; assessed 2026-09-12T12:00:00Z; last verified 2026-09-12T12:00:00Z. This is a saved observation, not a live connection or work authorization. Supervisor observation is missing, unreadable or invalid. Ask the manager to restore a readable observation; supervisor health is unknown. |

**Duplicate pairs among these six: none**, including when ignoring timestamps.
The seventh saved-snapshot-aging row has the same supervisor-stale explanation
as the stale-supervisor row, appropriately; its assessed timestamp stays at
12:00:00 because no reprojection occurred.

## Verification and limits

Final targeted tests: **11 passed** (12.177s). Focused owner/feed/attention/control/
promotion/preflight tests: **299 passed** (102.898s). JavaScript renderer:
**1 program passed**. **7 observations** and **1 interrupted replay** captured.
Full discovery: **872 passed** (492.412s), **0 failures / 0 errors / 0 skips**,
exit 0; [final raw gate](owner-surface-110-suite-2.txt). Final failure names:
**none**.

Changed source/tests: **251 additions / 20 deletions** across eight files.
[Verification record](owner-surface-110-verification.md) contains exact commands,
isolation, failed attempts, file counts and SHA-256 hashes.

Developmental attempts: **3 nonzero test commands**. The first two each
reported the same **1 failure and 2 subtest errors**; full discovery attempt 1
reported **2 failures and 0 errors** across 872 tests (492.477s). All four named
assertions were corrected; raw attempts and exact failure names are preserved
in the verification record. Total: **4 failure occurrences / 4 subtest error
occurrences** in developmental runs.
Other nonzero operations: **1 read-only shell command** and **1 no-op edit
rejection**. Final targeted/focused/renderer/evidence gates have zero failures.
No raw attempt was overwritten.

This is a revise-worker return for manager review. Independent code-blind
design/execution/evidence review, packaged true-TUI proof, live-repository
qualification and named-human acceptance are not claimed or waived. No live
listener, journal, coordinator, profile or credentials were accessed. No
native prompt, Rust test, workspace formatter, commit or push occurred.

## Corrections to the brief

The saved-path false-assurance finding is correct. More precisely, the old
top-level notice said last-verified while nested supervisor health already
said unknown; it was not a nested healthy value. Waiting and quarantine were
byte-identical. Expiry used the same state/action wording but had a different
timestamp, so that entire string was not byte-identical. The interruption
demonstration covers catchable failure, not SIGKILL or failed cleanup storage.
All six rendered situations are now distinct; broader qualification remains
with the manager.
