# Slack malformed-attempt repair — September 15, 2026

## Mandate and classification

- Action: `slack-attempt-guard-01`; implement worker: GPT-6 Astra, High.
- Allocation digest: `b1ebdf2d14e74fc6acc7b0858ba36028cbb57bf831ed2767d1b18e7ca6edbfc9`.
- Frozen brief SHA-256 verified before inspection:
  `1aabb5e16258f706f22aa759edcdfc53f6bd0d38eacc1a3b432b5486518851fd`.
- Base: `34590180808b06c35234afe144c16b4b07b300a3`.
- Branch: `bootstrap/slack-attempt-guard-20260915`.
- Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-slack-attempt-guard-20260915`.
- Classification: **bounded fix** restoring explicit local reconciliation and
  truthful delivery status within the existing owner-controlled journal.
- Product authority: [Internal delivery control — TO BUILD](../../../docs/corbanu-product-spec.md#internal-delivery-control--to-build):
  “Slack connection, exact destination and delivery require verification; a
  question card or dashboard refresh is not proof the human was alerted.”
- No new product outcome or authorization boundary; no product sprint is
  implemented by this repair.

## Incident

The frozen manager brief reports that attempt
`pf83-vm-key-20260915T103614Z` was journaled with `receipt=None` before
`uuid.UUID(attempt[:32])` failed. No HTTP POST occurred. The same conversion
prevented reconciliation, leaving projected status permanently held.

The manager reports five inspected messages in thread `1789461307.096899`,
none matching the attempted payload. The inspection artifact is
`/private/tmp/fmgr.Q1SIYZ/slack/orphan-post-inspection.json`; its SHA-256,
independently computed by this worker, is
`7b51827d1c3850d3c9e1b3ff44d595dba3d9ef31174a225cd141e8e9f21245ba`.
The inspection's findings are manager-supplied; this worker did not access
live Slack or the live transport journal.

## Implementation

New `exchange` attempts must be Python strings matching
`[0-9a-f]{64}` in full: exactly 64 lowercase hexadecimal characters. Validation
and UUID derivation precede the gate and every durable write. Rejection raises
`decisions.Invalid` without touching state or credentials.

Repository caller inspection found:

- `decision_alerts.request_for`: parent/details attempts are
  `d.digest([key, phase])`.
- `decision_alerts.notice`: clarification/acknowledgment attempts are
  `d.digest([key, kind, basis])`.
- `decision_manager.main` send/interpret and `decision_manager.finish`
  acknowledgment paths pass `transport.exchange` through those producers.
- SDK fixtures either use those producers or replace attempts with
  `d.digest([index])`.

All existing production producers emit 64 lowercase hex characters. A 32-character
new-attempt rule would reject them. Existing UUID mapping remains the first
32 characters of the digest.

Recovery deliberately uses the **old** conversion, not the stricter new
admission rule. A 32-hex ID, uppercase hex ID, or valid first-32-hex prefix with
a suffix could have been sent previously and cannot be cleared as an orphan.

`Transport.reconcile(request)` returns a local `never-sent` classification
for an impossible legacy client ID without HTTP or journal mutation. This is
not a delivery receipt. All encodable IDs retain the existing exact positive
association rule; missing, ambiguous or incomplete history stays uncertain.

## Explicit owner recovery path

The manager may call:

```python
transport.reconcile_orphan(
    "pf83-vm-key-20260915T103614Z",
    request_digest=expected_request_digest,
    evidence="inspection-7b51827d1c3850d3c9e1b3ff44d595dba3d9ef31174a225cd141e8e9f21245ba",
)
```

Here `expected_request_digest` is `decisions.digest` of the exact retained
post request inspected by the owner. The transport must use the current pinned
binding, enabled local operation, fresh qualification, a valid ingress fence,
and an observed listener session. Existing holds are retained.

The equivalent owner-pipe CLI is
`decision_manager.py reconcile-orphan --store <owner-store> --live`, receiving
one JSON line with `binding`, `attempt`, `request_digest` and `evidence`.
This operation does not invoke the SDK or read credentials. `--live` preserves
the existing explicit enable gate; it does not cause a Slack send.

Evidence is a mandatory nonempty owner inspection reference matching the
existing 1–100-character alphanumeric/underscore/hyphen token contract.
The supplied reference should identify the preserved inspection artifact,
as the SHA-256-based reference above does. The tool does not independently
certify the owner's inspection.

Recovery refuses a digest mismatch, existing receipt, encodable legacy ID,
missing evidence, binding mismatch, broken fence or invalid session. It appends
`reconciliation={outcome, evidence, at}` inside exactly the selected post.
The outcome retains attempt, request digest, `never-sent` and
`invalid-client-msg-id`; the original request, receipt and retry metadata remain.
Exact replay is read-only; different replay evidence is refused.
No import/startup clearing, deletion, retry, notice or send is introduced.

Status excludes only a validated durable never-sent reconciliation from its
uncertain-post calculation. Another uncertain post still holds status; a
reconciliation copied onto an encodable post cannot clear it.

## Regression evidence

Eight added tests cover:

1. Malformed attempts (including wrong lengths/types/case) stop before the gate,
   writes and HTTP; durable transport bytes remain identical.
2. Digest attempts preserve UUID mapping, receipt lookup and duplicate suppression.
3. Malformed orphan classification is local; explicit recovery retains the
   original post and evidence, survives reopen and supports only exact replay.
4. Missing evidence, wrong request digest and an existing receipt refuse mutation.
5. Every legacy-encodable test ID refuses owner clearing; empty complete history
   remains uncertain.
6. Invalid sessions and ingress fences still refuse recovery.
7. Status clears only the reconciled orphan, remains held for genuine uncertainty,
   and rejects copied reconciliation metadata on an encodable post.
8. The owner-pipe CLI persists the exact evidence without SDK/credential access.

Initial focused invocation stopped at import with a test line-continuation
SyntaxError; no tests executed. The syntax was corrected before the final runs.
Focused regressions: **8 passed in 4.151s**.

Full SDK command (synthetic fixtures, inherited profile aliases and credential
variables excluded by the clean environment):

```sh
env -i PATH="$PATH" HOME="$HOME" TMPDIR=/private/tmp PYTHONDONTWRITEBYTECODE=1 \
  PYTHONPATH=scripts/initiative_control:/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/lib/python3.14/site-packages \
  /Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B \
  -m unittest discover -s scripts/initiative_control -p '*test*.py'
```

Full suite: **601 tests passed in 312.165s**, exit 0, no skips reported.
HTTP 500/429 fixture cleanup ResourceWarnings were emitted; no failures or retries.
No native credential prompt was observed.
Plan checker: passed, active 3/3, available slots 0.
Sprint checker: passed, current 116, archived 126.
Whitespace check: passed.

## Handoff limits

This is an implementation return to the Fable manager, not a human-test-ready
candidate or release. No true-TUI, independent code-blind functional execution,
live-repository qualification, human acceptance or benchmark result is claimed.
The manager retains any applicable dashboard/functional qualification gate.
Rust is untouched; no Rust tests were run.

The manager must perform and record the live orphan reconciliation afterwards.
This worker did not mutate live Slack state, send Slack messages, push or claim
that the live projected status is already repaired.

## Parity gap found while clearing the live orphan — September 15, 2026

The guard and the owner reconciliation worked: the live orphan
`pf83-vm-key-20260915T103614Z` was reconciled as `never-sent` against retained
read-only inspection evidence, and `decision_manager.project_status` correctly
returned `last-verified` afterwards.

The **dashboard did not clear**, because it does not read `project_status`. The
published projection is `decision_feed.project_slack`, which carries its own copy
of the same check and still read `post["receipt"] is None` without the
`reconciled_never_sent` exemption. So a reconciled orphan kept the operator
surface wedged at `held` even though the state was resolved.

Fixed by mirroring the manager check. A regression
(`test_published_projection_clears_once_an_orphan_is_owner_reconciled`) asserts
`held` before reconciliation and `last-verified` after; with the exemption removed
it fails `'last-verified' != 'held'`, so it discriminates.

Worth noting for future duplicated predicates: the review verified the manager
path, the tests covered the manager path, and the second copy went unnoticed
until the live state was actually exercised.
