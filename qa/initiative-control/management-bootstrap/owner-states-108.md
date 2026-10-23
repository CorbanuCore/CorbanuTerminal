# RETURN — owner-states-108

Runtime: gpt-6-astra / high. Allocation digest:
`8d9add414c7ab77fde6f11d0980bc30034425365202f9cb649cb36bc3bf47dbb`.
Claim: `25f6270a-41ae-4e50-a4fb-fb441d83fcc1`.
Starting HEAD matches assigned base `e839c466a376cfaf684cb7f11cb9ec143a8eafd4`.
Brief hash verified before other reads:
`cfe1eef3e93161a1e8575c87027535a1d4eabbbcb5f35f041939c65cc07a59c5`.

**Partial implementation; renderer requirements blocked by contradictory
allocation.** The brief says attention.py, control.py and their tests were
added, but the dispatch's explicit "Writable scope (only these paths)" excludes
all four. That narrower explicit list was respected. No edits outside the
listed files and management-bootstrap evidence directory were made.

Bounded reliability repair under product-spec heading **Internal delivery
control — TO BUILD**: “durable event dispatch, acknowledgments and watchdog”;
“The matching Slack thread should log his answer and route it to the appropriate
agent through a verified identity/revision-aware manager handoff”.
Existing initiative-delivery-control / PF-80-S01 context. This worker does not
advance a plan/sprint or claim product, release or human-test acceptance.

## Durable quiet-journal clearing

The existing owner-controlled manager `qualify` operation forwards
`ui_evidence` and `gap_review` to transport qualification. It now validates an
explicit supplied review even when the previously reviewed journal is quiet,
unheld and has no uncovered ingress. Previously that review was silently
ignored unless a fresh fault first caused a hold or ingress gap.

The operator supplies the existing supported-UI evidence and exact review:
current watermark, binding digest, ingress count, listener session and epoch,
plus an evidence identifier. Session/liveness checks, locking, binding checks,
undrained-event refusal and final atomic persistence remain in place. No
automatic clearing occurs on ordinary qualification without an explicit review.

Successful persistence appends the review and writes
`outstanding = {count: 0, oldest_at: null}` in the same qualification write.
The audit history remains; this acknowledges uncertainty rather than recovering
missing dispositions or claiming delivery. Failed writes preserve the unknown
history. Reopening, unrelated ingress and a later rejected event cannot
resurrect the cleared prefix.

Three new test methods cover successful quiet clearing, renewal without
clearing, reopen and new ingress, failed final persistence with retry, and six
invalid exact-review fields. These are fixture tests, not live qualification.

## Four actual rendered notices, verbatim

These were captured from the final source with disposable journals and the
existing feed and attention renderers. Dates are the literal fixture values.
Items 1, 2 and 4 of the brief remain incomplete; these are not proposed fixes.

**Waiting for a route:**

> Slack observation: held; assessed 2026-09-12T12:00:00Z; last verified 2026-09-12T12:00:00Z. This is a saved observation, not a live connection or work authorization. If held or stale, answer in the manager task.

Actual action implied: act by answering in the manager task. It does not
communicate the requested wait action.

**Quarantined and needing a human:**

> Slack observation: held; assessed 2026-09-12T12:00:00Z; last verified 2026-09-12T12:00:00Z. This is a saved observation, not a live connection or work authorization. If held or stale, answer in the manager task.

Actual action implied: act by answering in the manager task. It does not explain
quarantine or request the required review.

**Expired undelivered:**

> Slack observation: held; assessed 2026-09-12T12:15:00Z; last verified 2026-09-12T12:00:00Z. This is a saved observation, not a live connection or work authorization. If held or stale, answer in the manager task.

Actual action implied: act by answering in the manager task. It does not disclose
expiry or distinguish it from waiting/quarantine. The different observation time
is not a state distinction.

**Nothing wrong:**

> Slack observation: last-verified; assessed 2026-09-12T12:00:00Z; last verified 2026-09-12T12:00:00Z. This is a saved observation, not a live connection or work authorization. If held or stale, answer in the manager task.

Actual action implied: no action for this state, only inferred from the
conditional. No explicit “nothing to do” message is rendered.

Waiting, quarantine and expiry still require structured facts to distinguish:
respectively held=1; outstanding count=1; and held=0/outstanding count=0 with a
retained expired record. **The number-free distinction requirement is not met.**
Nothing-wrong and unavailable/stale supervisor also remain visibly conflated.

None of these rendered notices promises arrival; no arrival promise was added.

## Unknown sources and brief corrections

The separated fixture sources are preserved in
[the eight observation rows](owner-states-108-observations.jsonl):

- Legacy pruned history: supervisor reason `quarantine-history-unknown`, count
  zero, unknown=128. Renders “Slack observation: unknown” without a clearing
  instruction. The quiet clearing path above is implemented.
- Missing supervisor observation: `observation-unavailable` in structured
  supervisor health. The actual top notice still reads “Slack observation:
  last-verified”, indistinguishable from the healthy case.
- Stale supervisor observation: `observation-stale` in structured health.
  The actual top notice still reads “Slack observation: last-verified”.
- No saved Slack observation: renders “Offline manager assessment. Slack not
  connected. Answer in the manager task. No operational approval or agent
  activity is established here.”

Thus separate rendered notices for the unknown sources are **not implemented**.
The brief's literal statement that all sources render the same “unknown” notice
is inaccurate for this base: supervisor failures may be silent behind
last-verified, and absent Slack observation uses the offline notice. The
underlying requirement to disclose each source remains valid.

The scope-widening statement is also inconsistent with the dispatch allowlist.
At minimum attention.py and test_attention.py must be explicitly added to
implement and test the notices. Correcting saved feed state selection may also
need decision_feed.py and test_decision_feed.py, which are not in either the
explicit allowlist or the brief's asserted widening. Control files mentioned
by the brief remain unmodified.

## Verification and qualification limits

Final discovery: **862 passed**. Focused owner/feed/attention/control gates:
**296 passed**. New regressions: **3 passed**. JavaScript renderer: **1 program
passed**. Eight disposable observations captured. Test failures/errors/skips:
**0**; failure names: **none**. One read-only search exited 2 for a nonexistent
filename; no test gate returned nonzero. Source/tests: **66 added / 1 deleted**
lines in two files. Exact commands and accounting are in
[the verification record](owner-states-108-verification.md).

This is a partial implementation return to the Fable manager. Independent
code-blind design/execution/evidence review, packaged true-TUI proof,
live-repository qualification and named-human acceptance are not claimed.
The rendered-state requirement remains an open functional gate.

No live listener, journal, coordinator, profile or credential was accessed.
No native credential prompt, Rust test, workspace formatter, commit or push
was used. Only disposable stores, isolated test profiles and synthetic SDK
fixtures were used.
