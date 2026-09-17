# owner-projection-71 — partial activation status and retained Slack questions

Allocation: `owner-projection-71`; model/effort: `gpt-6-astra / high`.
Allocation digest: `78c359ae2b3f631656c9ada15c8e3e785f4b29389dee4c75e8e058a6de1116e8`.
Claim: `7276df3a-435d-4e17-9243-ab2f8d941424`.
The brief was read first and its SHA-256 matched
`b0b7d8a0d999a4156e565a664b79134da681129b493eb36d0d5932f8b74a6d14`.
Base/initial HEAD: `940347db65066051ad9caebcad05cd872838b51a`.
Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/owner-recurrence-20260917`.

## Classification and limits

Bounded reliability correction under existing management-bootstrap PF-80-S01,
in_progress, active initiative-delivery-control. Product heading:
**Internal delivery control — TO BUILD**; excerpts: “durable event dispatch,
acknowledgments and watchdog”; “preserve the last good publication on failure”;
“dashboard and alerts are projections”. This restores observation of admitted
state and questions, without granting activation or Slack delivery authority.

The manager owns shared plan/sprint records. The supplied base remains newer
than the plan's recorded base; this worker does not update manager-owned records.
Sprint checker passed: 115 current, 127 archived.
This is an implementation return. Independent functional design/execution/evidence
acceptance remains with the manager before an unqualified human-test handoff.
No release, live recurrence, TUI, live-repository qualification, named-human
acceptance or benchmark pass is claimed.

## Activation status

Owner and coordinator reads are independent. Successful fields remain available;
an unreadable component is null with its exception type/reason under
`unavailable`, and `complete=false`. A readable empty coordinator is distinguishable
from an unreadable coordinator. Owner failure leaves state/generation/stored null,
never a fabricated OFF result. Successful reads explicitly report `complete=true`.
The warning states that these are advisory independent reads, not an atomic
snapshot or arming clearance. The coordinator read uses a 250 ms busy timeout.

Status never invokes journal recovery. Arming still uses the strict checks and
refuses unavailable coordinator state. Disarm and admission rules are unchanged.
The existing `recovery_required` field describes pending activation transactions;
a coordinator journal is separately disclosed as `coordinator_recovery_required`.

Actual disposable-fixture example, excerpt from
[examples](owner-projection-71-examples.txt):

```json
{
  "state": "off",
  "generation": 0,
  "coordinator": null,
  "complete": false,
  "unavailable": {
    "coordinator": {
      "error": "LaunchError",
      "reason": "coordinator_recovery_required"
    }
  }
}
```

The CLI contention case holds an actual SQLite EXCLUSIVE transaction and verifies
exit 0, readable owner state, null coordinator, and `database is locked`.
The recovery case verifies every fixture file is byte-for-byte unchanged.
The owner-lock case verifies coordinator visibility despite an unavailable owner.

## Discoverable red-to-green reproduction

The two original cases now run in normal unittest discovery:
[ProjectionRegressionTests](../../../scripts/initiative_control/test_decision_feed.py).
Their names are:

- `test_twenty_decisions_fifty_three_revisions_projects_within_bound`
- `test_bound_never_drops_open_or_acknowledged_question_after_closed_history`

The latter has separate `assertIn` checks naming the missing open versus
acknowledged question. Latest-revision checks also identify the decision ID.
The [standalone reproduction](owner_visibility_68_feed_repro.py) remains runnable,
and the [round-68 evidence](owner-visibility-68.md) links this correction.

[Failing-first run](owner-projection-71-red.txt), against the original implementation:
10 tests, 4 passed, 1 failure, 5 errors, exit 1. Both projection tests raise
`decisions.Invalid` at the old size validator. Other error names:
`test_busy_owner_still_reports_coordinator_and_unknown_owner`,
`test_complete_status_is_explicit`, and
`test_status_does_not_recover_coordinator_journal`.
The failure is `test_busy_coordinator_returns_partial_status_through_cli`
(old CLI exits 2). These are genuine behavioral failures, not fixture defects.

[First green run](owner-projection-71-green.txt): 17 tests passed, exit 0.
[Standalone run](owner-projection-71-standalone.txt): 3 passed, exit 0.
Its third diagnostic now forces a 100-byte cap to verify that refusal preserves
the previous cache; the historical round-68 logs preserve the original 16 KiB
diagnostic and its 53-row measurement.

## Retention and bound

Keep the latest revision of **every decision**, including closed, open and
acknowledged decisions. Keep an older revision when its alert has undrained
ingress, an unacknowledged answer, unfinished reply processing or notices, or
still lacks a resolved answer/delivery. Cancellation alone does not hide pending
ingress or handoff. Fully settled older history without these obligations may
be omitted. A missing/different transport binding conservatively retains the
older alert as unknown without attributing another binding's delivery/replies.

The cache records exact `omitted_revisions`; the dashboard displays that count
and explains that full decision history remains present. With Slack disabled,
no store is inspected: latest rows remain, and the text explicitly conditions
older-alert retention on enabled projection. The validator rejects missing latest
rows, duplicate revisions, wrong digests, invalid counters and incorrect omission
counts. Legacy complete projections without the optional count remain accepted.

The admitted decision feed is at most 1,048,576 bytes. Each revision uses at
least 211 bytes for required JSON keys, colons, separators and braces alone;
therefore at most floor(1,048,576 / 211) = **4,969** revision rows can exist.
Each projection row has an 80-character maximum ID, a 64-character digest,
a bounded revision number, fixed enum fields and bounded counters. Its 1,024-byte
row allowance is conservative: the [maximum-field encoding check](owner-projection-71-bound.txt)
measures 440 canonical bytes and a 588-byte indented per-row increment.
Add one separator per row and a 16,384-byte
envelope allowance: the explicit cache limit is **5,109,609 bytes**.
The complete feed transport limit becomes **6,158,441 bytes**.

This permits retaining all feed revisions if every alert remains unresolved;
neither a 16 KiB truncation nor a 100-row truncation can guarantee this.
Writer validation checks canonical and actual indented disk encoding, and both
cache/transport readers use the corresponding limits. Envelope/row/total bounds
still reject malformed or excessive input before replacing the cache.
This is a bounded increase in local observation/export memory, not an unbounded
Slack payload or a new external transmission. Delivery permissions are unchanged.
Deploy writer and reader from the same source export; old readers reject the new
optional metadata/large projections, so mixed-version deployment is not qualified.

Synthetic 20-decision/53-revision result: 20 rows, 33 omitted older revisions,
6,676 canonical bytes, 9,716 disk bytes. Additional tests cover:

- 130 open questions, exceeding both the old row and byte caps, preserved through
  cache read and a pinned exported-snapshot read;
- an older unanswered alert plus latest/open questions surviving above 16 KiB;
- a missing binding preserving the older alert as unknown;
- omission disclosure and validator rejection of a missing latest row;
- forced overflow preserving the last cache and reporting invalid on read.

## Published-history answer

**No silent decision drop was found in the retained published history. The three
retained failing publications fail closed for Slack status, while preserving all
20 decision cards. “Always” cannot be proved from the pruned history.**

This answer uses actual published artifacts, not just source inspection.
The read-only [audit script](owner_projection_71_history.py) read only exported
decision-feed snapshots, release manifests and index pages. It was run locally
and via the existing SSH wrapper against the remote publication root; it did not
read credentials, live profiles, live Slack journals or mutable feed state.

[Remote audit](owner-projection-71-history-remote.json):

| Publication | Time UTC | Feed revision | Feed decisions/revisions | Missing cards | Slack |
| --- | --- | ---: | --- | ---: | --- |
| build-4l3n9egy | 2026-09-17 10:45:20 | 160 | 20 / 53 | 0 | unknown |
| build-qwock15g | 2026-09-17 11:01:00 | 161 | 20 / 53 | 0 | unknown |
| build-eibs42we | 2026-09-17 11:34:20 | 162 | 20 / 53 | 0 | unknown |

Each manifest's exact feed SHA-256 pin matches its retained incoming snapshot.
Each snapshot has `slack_status=invalid` and no projected Slack object; none
presents a truncated object as valid. The page IDs were compared with **every**
decision ID in the matching snapshot. Manifest/page hashes are preserved.
The [local exports](owner-projection-71-history-local.json) match all three
remote snapshot hashes. A local September-10 page predates these decision records.

[Thirteen older publication receipts](owner-projection-71-history-receipts.json)
were also inspected and hashed. Four report Slack stale; the September-15
revision-39 receipt reports last-verified. Eight do not carry a parseable Slack
health object. These receipts do not contain complete historical projection rows,
so they cannot establish absence of a silent row loss in the deleted generations.
Publisher/export retention keeps three generations, explaining the gap.

The initial audit outputs are preserved with `-initial.json` suffixes. They
matched on source tree rather than exact feed pin and incorrectly derived missing
cards from missing Slack rows. They are superseded by the corrected audit above;
their empty missing-card lists are not used as evidence.

## Test campaign

Read docs/development/test-isolation.md before testing. Disposable venv:
`/private/tmp/owner-projection-71.OhInA4/venv`, created under `env -i`,
with only requirements.txt's markdown-it-py 3.0.0, mdurl 0.1.2, slack-sdk 3.44.1.
All tests run from the repository root under `env -i`, disposable HOME and all
three profile aliases, TMPDIR=/private/tmp, PYTHONDONTWRITEBYTECODE=1 and checkout
scripts/initiative_control on PYTHONPATH. No raw Rust test, native credential
prompt, live profile, broad formatter, push or activation was used.

Commands:

```text
python -m unittest discover -s scripts/initiative_control -p 'test_*.py'
python -m unittest -v test_owner_daemon test_decision_feed
python qa/initiative-control/management-bootstrap/owner_visibility_68_feed_repro.py
```

Initial [owner/feed run](owner-projection-71-focused.txt): 140 passed, exit 0.
The binding follow-up passed its [single test](owner-projection-71-binding.txt).
Initial [full discovery](owner-projection-71-suite.txt): 794 tests passed in
439.486s, zero failures/errors/skips, exit 0. This run loaded before the binding
follow-up and physical-cache check, so it is preserved as an intermediate run.
Final [owner/feed run](owner-projection-71-focused-final.txt): 141 tests passed in
56.219s, zero failures/errors/skips, exit 0.
Final [standalone run](owner-projection-71-standalone-final.txt): 3 tests passed in
0.084s, zero failures/errors/skips, exit 0.
Final [full discovery](owner-projection-71-suite-final.txt): **795 tests passed**
in **437.169s**, zero failures/errors/skips, exit 0. ManagerTests (including TMUX),
RealTmux, and the owner_tmux ACK/return and bracketed-paste cases passed; no
load-sensitive failure was relabeled or excluded. The full runs emitted Python
ResourceWarnings for fixture HTTP 429/500 cleanup; these are preserved in the
logs. Expected TimeoutExpired fixture output is not an unexpected test failure.
Final source hashes below match the tested files. git diff --check and the final
scope audit pass. No native credential prompt was observed.

## Changed source and brief corrections

activate.py is unchanged. Source/test diff: owner_daemon.py +26/-8;
test_owner_daemon.py +32/-2; decision_feed.py +47/-10;
test_decision_feed.py +138/-0: **243 additions, 20 deletions, 263 changed lines**.
The two pre-existing QA files add 15/delete 5 lines; new evidence is separately
listed by git status. All changed artifacts are inside the assigned scope.
No formatter was needed; git diff --check passes. Source SHA-256:

- owner_daemon.py: `816af4f01e25b657c175e856177f08f7d8d9c4248cb1130827b2902daf5b0791`.
- test_owner_daemon.py: `ca66ce1019cd58f3ed743ea60ca21bef8e10105eaa9981d73017e21a9d96f2a1`.
- decision_feed.py: `8f904d8c8b89fe0f5aab00a4d8c2c551a6f20838fbf50ce104169aa313a5d4f6`.
- test_decision_feed.py: `21b66fb3a2ad1fc2cefd0e923ea76b61cf7e82d8da22ec5a6053f125c7b7634e`.

Owner package digest: `c98c052877c7b584aecca5189c4c046d936d3577a9f0e3f43d1082bdeaf0bbe5`.
Deployment must re-pin deliberately.

The live 20/53 shape is confirmed in retained published snapshots, rather than
claimed from mutable live state. The 339-byte estimate is not universal:
the earlier synthetic all-history projection measured 16,850 canonical bytes;
actual cache encoding is larger than canonical encoding. “Permanently unknown”
describes failure while oversized/revision-invalid state persists, not an
irreversible transport condition. No historical evidence supports an unqualified
“always failed closed” claim across deleted generations.
