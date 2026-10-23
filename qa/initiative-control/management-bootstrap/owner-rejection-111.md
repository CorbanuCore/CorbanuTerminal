# RETURN — owner-rejection-111

Runtime: gpt-6-astra / high. Allocation digest:
`909c28f1cd880619bb890510415d2be3701df4a0cce1b1e7ae762340880da6bb`.
Claim: `d7d7b63a-9872-4b8e-9935-15d14d960594`.
Assigned base and starting HEAD:
`f152fcef66a3259aec977cea9b6f7284d8a6f1a1`.
The frozen brief was the first file read; its verified SHA-256 was
`70d13c2ff712dfdfb2e0585ffee609b7449ce99db20209990cfe2d4e95674238`.

Bounded reliability fix under exact product-spec heading **Internal delivery
control — TO BUILD**: “durable event dispatch, acknowledgments and watchdog”
and “Show blockers, rendered sprints, human test plans, machines, run logs
and freshness”. This restores qualification fault reporting and review
disposition in the existing initiative-delivery-control / PF-80-S01 context.
The worker does not change manager-owned plan/sprint records or claim acceptance.

## Could not tell versus Slack said no

Qualification now distinguishes positive rejection with a fixed-vocabulary
`QualificationRejected` exception:

- Recognized authentication rejection codes on HTTP 200/401/403 leave
  `qualification-auth-rejected`.
- Explicit permission rejection codes or an explicit scope list missing required
  permissions leave `qualification-scopes-rejected`.
- A successful authentication response with a different workspace/bot identity
  leaves `qualification-identity-rejected`.
- Interrupt, timeout/I/O failure, rate limit, server failure, malformed response,
  unknown error or absent scope metadata restore the exact prior hold.
  A failed final persistence also restores the prior hold.
  An already-recorded rejection therefore survives an inconclusive retry.

Neither direction advances verification or commits a gap review on failure.
Cleanup only changes a hold still equal to `qualifying`; a newer callback
fault wins. The per-store, cross-process single-flight guard remains intact.
Raw SDK errors, response bodies and credential material are not journaled.

Both manager and saved-feed projections carry the fixed rejection reason.
The renderer tells the operator to repair credentials/scopes/binding, review
the hold, and requalify. Repair alone cannot clear an established qualified
journal's rejection: an exact current gap review and successful qualification
are required. A pristine never-qualified journal can retry its rejected
bootstrap after repair under the existing zero-ingress/no-prior-evidence checks.

Tests use real Slack SDK requests to a loopback fixture for **46 positive
rejection cases** (23 responses × two prior holds) and **21 inconclusive
response cases** (seven responses × three prior holds). They compare reopened
journals, assert work gating, require explicit review after repair, and prove
successful recovery. Existing interrupt/final-write restoration, callback
race, and cross-process contention checks remain.

An uncatchable process death or failed cleanup-storage write can still leave
`qualifying`; explicit owner recovery remains necessary. This revision does
not claim automatic recovery when cleanup cannot execute or persist.

## Outstanding expiry and its clearing path

Expiry is counted in the durable outstanding quarantine summary separately
from active intake. Disposition increments that count; exact review, committed
with successful qualification, resets it to zero. Audit records remain.
Projection reads this summary instead of counting every retained expiry.

Legacy journals lacking the field derive the initial count from expiry
dispositions not covered by a strictly later gap review. Both ingress and
disposition time matter: reviewing an arrival before its eventual expiry does
not review the later expiry. Equal legacy timestamps cannot establish ordering;
the expiry stays outstanding until a new exact review durably writes zero.
Newly tracked expiries use the durable count, including an expiry at the same
clock tick as a prior review. The counter survives pruning of the 128-record
audit tail.

Tests cover failed review persistence, reopening, unrelated arrivals after
review, legacy migration before/after review, new expiry after review,
same-timestamp review/expiry ordering, bounded-tail pruning and invalid counts.
An expiry remains undelivered; clearing disclosure never replays it.

## Reachable guidance and end-to-end rendering

Incomplete-history and expired-undelivered guidance now render independently
of the top-level held state. The incomplete-history action requests an exact
gap review to resolve unknown history; it does not imply a hold exists.
Expiry guidance explains no replay and names review as the clearing path.

The regression path is:

`disposable callback/journal → project_slack → persisted projection →
read_slack validation → slack_health → attention.render_decisions`.

No constructed final health dictionary is passed to the renderer. The fixture
supplies a fresh supervisor observation, as the existing transport tests do.
The incomplete-history fixture models a real legacy truncated journal.
The expiry fixture drives an unbound callback through actual session renewal
and expiry. At verification +901 seconds the real top-level state is stale,
so the non-held rendering branch is exercised. Its durable journal remains
held for expiry until exact review; afterward both the journal and projection
recover and retained expiry stops being disclosed.

[Raw eight observations](owner-rejection-111-observations-final.jsonl) and the
[reproducible capture](owner_rejection_111_observe.py) contain complete status,
health, outstanding counts, durable hold and exact rendered notice.

Verbatim rendered action excerpts:

- Unknown history, durable hold null: “Quarantine history is incomplete. Ask
  the manager for an exact gap review to resolve the unknown history.”
- Non-held/stale expiry: “A reply expired undelivered while waiting for a route.
  Do nothing to replay that expired reply; if an answer is still needed, answer
  in the manager task. Ask the manager to review the expiry to clear this notice.”
- Authentication rejection: “Slack rejected authentication. Ask the manager to
  repair the app credentials, review the hold and requalify.”
- Scope rejection: “Slack rejected the required permissions. Ask the manager
  to restore the required app scopes, review the hold and requalify.”
- All four reviewed/recovered observations: “No transport action is needed for
  this observation.”

## Gate and isolation

Read `docs/development/test-isolation.md` before testing. Created a disposable
venv under `env -i` with `/opt/homebrew/bin/python3 -m venv`; installed exactly
`scripts/initiative_control/requirements.txt`.
[Installation output](owner-rejection-111-venv.txt).

All Python/Node test and capture commands ran from the repository root under
`env -i`, with only:

```text
HOME=/private/tmp/owner-rejection-111.0Q8o3Y/home
CODEX_HOME=/private/tmp/owner-rejection-111.0Q8o3Y/home
CORBANU_HOME=/private/tmp/owner-rejection-111.0Q8o3Y/home
PFTERMINAL_HOME=/private/tmp/owner-rejection-111.0Q8o3Y/home
CORBANU_TEST_NO_NATIVE_KEYRING=1
TMPDIR=/private/tmp
PATH=/opt/homebrew/bin:/usr/bin:/bin
PYTHONPATH=scripts/initiative_control:qa/initiative-control/management-bootstrap
PYTHONDONTWRITEBYTECODE=1
```

Python executable:
`/private/tmp/owner-rejection-111.0Q8o3Y/venv/bin/python -B`.

Commands:

```sh
python -m unittest discover -v -s scripts/initiative_control -p 'test_*.py'
python -m unittest -v test_owner_daemon test_owner_tmux test_decision_feed test_attention test_control test_owner_promotion_94 test_owner_preflight_98
node scripts/initiative_control/test_facilities_js.cjs
python qa/initiative-control/management-bootstrap/owner_rejection_111_observe.py
git diff --check
git status --short
```

Targeted run 1: **13 passed**, 12.247s, exit 0.
Targeted run 2: **2 passed**, 3.075s, exit 0.
Final targeted run 3: **3 passed**, 3.727s, exit 0.
These are overlapping executions, not additional unique tests. After run 2,
its pruning test was strengthened to create a second genuine expiry rather
than inject a summary count. Self-review also identified ambiguous equal-time
legacy reviews; the final targeted run covers that correction.
Final focused owner/feed/attention/control/promotion/preflight: **299 passed**,
107.956s, exit 0. The earlier focused run passed **299** in 103.452s.
Renderer: **1 program passed**, exit 0, repeated on the final tree.
Capture: **8 observations**, exit 0, repeated on the final tree.

First full discovery: **880 passed**, 505.489s, exit 0. This is preliminary
evidence because the legacy timestamp correction landed while it was running.
Final full discovery: **880 passed**, 501.647s, exit 0, **0 failures /
0 errors / 0 skips**. [Final raw gate](owner-rejection-111-suite-final.txt).
Final focused count breakdown: owner-daemon 133, owner-TMUX 52, feed 39,
attention 24, control 39, promotion 8, preflight 4. Full discovery includes
113 manager and 83 transport tests.

[Machine-checked verification](owner-rejection-111-verification.json) records
each run's exact counts, module breakdown, failure names, final source hashes
and eight rendered observations. The evidence consistency checker passed
once, exit 0; it checks traceability and the captured transitions, not
independent functional acceptance.

Nonzero test/capture/renderer/checker commands: **0**; failure names: **none**.
Other nonzero operations: **2 read-only shell searches** with no matches and
**1 JavaScript orchestration syntax error**, before any edit in that call ran.
All are disclosed; none touched live state or rejected an approval.
The logs also contain deliberate fixture ERROR/timeout messages with passing
assertions; those are not failed test commands.

No live journal, profile, credentials, listener or coordinator was accessed.
No native prompt, Rust test, workspace formatter, commit or push occurred.
Only assigned paths changed. Ordinary SDK ResourceWarnings from synthetic
HTTP 401/403/429/500 responses are retained in the raw logs.

## Changed source/test lines

Against the assigned base:

| File under scripts/initiative_control | Added | Deleted |
| --- | ---: | ---: |
| attention.py | 15 | 8 |
| decision_feed.py | 2 | 1 |
| decision_manager.py | 7 | 3 |
| slack_transport.py | 71 | 14 |
| test_decision_manager.py | 95 | 0 |
| test_slack_transport.py | 139 | 4 |
| Total | 329 | 30 |

Additional report/capture/log files are confined to
`qa/initiative-control/management-bootstrap/`.

## Plain verdict and corrections to the brief

**This surface is not yet finished as an independently qualified operator
surface.** The assigned code revisions and regression evidence are complete,
and the final gate passed. What remains is manager-owned
independent review and functional acceptance of this changed candidate.
This worker did not perform or claim that acceptance, and did not touch the
live healthy transport to obtain it.

The substantive defects in the brief are correct. Two precision corrections:
the retained audit tail was already capped at 128 records; its disclosure
lifetime was unbounded by review. Incomplete-history guidance was reachable
in mixed held cases, but absent from the normal quiet unknown-history path.
The old “auth failure” restoration test exercised OSError, an inconclusive
failure; it did not prove correct handling of an explicit Slack rejection.

No independent code-blind functional acceptance, packaged interactive proof,
live qualification or named-human acceptance is claimed by this worker.
Those remain manager-owned gates; these regression tests and rendered
captures do not replace them.
