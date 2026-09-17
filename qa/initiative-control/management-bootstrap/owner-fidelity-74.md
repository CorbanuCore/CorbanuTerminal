# owner-fidelity-74 — answered revisions, omission fidelity, and safe status errors

Allocation: `owner-fidelity-74`; model/effort: `gpt-6-astra / high`.
Allocation digest: `47323a94c5579d67bd9381894fc154f88738c7dbff50a2420b038811a8701cb4`.
Claim: `b4cc1929-5743-4861-a483-a9a173dfd1b2`.
First file read: supplied brief, SHA-256 verified as
`08a72103e6643c942686fbf65748f86a890222261342efce09ff1df96652a4ea`.
Initial HEAD/base: `d778cba2aedc3dac21e71266e23c346895afac2c`.
Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/owner-recurrence-20260917`.

## Classification and handoff

Bounded fix within existing management-bootstrap PF-80-S01, in_progress,
active initiative-delivery-control. Product-spec heading:
**Internal delivery control — TO BUILD**; excerpt:
“actual Slack reply/decision/agent acknowledgment”.
This corrects the fidelity of existing projections and applies the existing CLI
error-redaction rule to partial status; it grants no new authority or disclosure.

Shared plan/sprint records remain manager-owned. Sprint checker: 115 current,
127 archived, exit 0. This is an implementation return to the Fable manager;
independent functional acceptance remains with that manager before an
unqualified human-test handoff. No TUI qualification, live-repository acceptance,
human sign-off, benchmark, deployment or release pass is claimed.

## Retained observations and omitted history

For each decision, projection retains the union of its latest revision,
**every revision referenced by any historical or current resolution's
answered_revision**, and revisions with unresolved Slack obligations.
The reference rule applies equally with Slack enabled and disabled.
Disabled projection still reads no Slack store and produces explicit off/zero
observations for the retained answered rows.

The renderer preserves the difference between:
- a retained observation (including definite off, cancelled, or genuine unknown);
- explicitly omitted older history (settled history when enabled; historical
  detail omitted while projection was off when disabled); and
- an unavailable observation without explicit omission metadata (unknown).

Each affected decision card now includes its own exact omitted-revision count,
and missing question observations carry the omission explanation beside their
revision. The existing page total remains. A card's count is calculated from its
own missing revision keys, not copied from the page total.
Older count-bearing projections remain readable, including the round-71 shape
that omitted answered rows. The renderer does not invent missing delivery
details for them. Projections without omission metadata do not gain a false
settled-history claim.

New separate Slack-on/off tests use a settled cancelled alert and a later
resolution citing revision 1, with revision 2 omitted and revision 3 current.
They assert retained keys {1, 3}, exactly one omission, and the current card's
Slack line: cancelled/zero with Slack on, off/zero with Slack off.
A second lineage test asserts {1, 4, 5} survive two resolutions separated by a
reopen, so retaining only the latest resolution's target cannot pass.
The renderer test covers enabled/disabled and explicit/missing omission metadata.

## Redacted partial status

Both owner and coordinator error dictionaries preserve LaunchError/Rejected
reason codes. Other caught errors use only their exception class name, matching
main(): e.g. OperationalError rather than a database statement, OSError rather
than a filesystem path. Successful component data and complete=false survive.

The CLI regression checks both components against five untrusted exception
types and two domain-code types: 14 subcases. It checks structured output,
absence of synthetic detail canaries, retained success fields, and exit 0.
The actual exclusive-SQLite contention test now expects OperationalError.

## What the bound protects and why this number

**5,109,609 bytes bounds the local saved Slack-status JSON consumed by the
dashboard and exported-snapshot reader. It is not a Slack API payload limit.**
The read limit rejects oversized input before JSON parsing; row, envelope and
total validation bound accepted observation size and row count. This protects
local ingestion and the renderer's observation input. It is not a proof of peak
Python memory usage, render duration, final HTML size, or Slack network safety.

The derivation is independent of the current 20-decision example:
- The admitted decision feed is capped at 1,048,576 bytes.
- Required revision keys/braces/colons/separators alone consume at least
  211 bytes, ignoring all values and surrounding feed overhead.
- Thus there can be no more than floor(1,048,576 / 211) = 4,969 revision rows.
  This deliberately overestimates the possible count.
- Each current-schema projection row has an ASCII ID of at most 80 characters,
  a 64-character digest, a revision ordinal bounded by that row count, fixed
  delivery enums, pending <=100 and reply counters <=1,000,000.
- The maximum-field row check measures 440 canonical bytes and a 588-byte
  indented row increment. The existing 1,024-byte allowance conservatively
  covers both representations; a further separator byte per row is reserved.
- With the existing 16,384-byte envelope allowance:
  16,384 + 4,969 * (1,024 + 1) = **5,109,609**.

This is a conservative schema-derived ceiling, not a uniquely minimal number
or a measurement that merely fits today's data. It allows all admitted revision
rows to remain when every row is referenced or unresolved, so status fidelity
does not depend on truncating history. Writer validation checks canonical and
actual indented cache encodings. The enclosing snapshot reader ceiling is
1,048,576 + 5,109,609 + 256 = **6,158,441** bytes.
[Recomputed bound](owner-fidelity-74-bound.txt).
Neither ceiling changed in this revision.

## Test campaign and preserved attempts

Read docs/development/test-isolation.md before testing.
Disposable venv: `/private/tmp/owner-fidelity-74.AzeMxz/venv`, built under
`env -i` from scripts/initiative_control/requirements.txt:
markdown-it-py 3.0.0, mdurl 0.1.2, slack-sdk 3.44.1.
Tests run from repository root with empty disposable HOME and profile directories,
all three profile aliases pinned to that disposable profile, TMPDIR=/private/tmp,
PYTHONDONTWRITEBYTECODE=1, CODEX_TEST_DISABLE_NATIVE_KEYRING=1, and
PYTHONPATH=scripts/initiative_control. No live profile or credential read,
native prompt, raw Rust test, workspace formatter, push or activation was used.

- [Initial red](owner-fidelity-74-red.txt): 6 tests, 14 failure records, 2 errors,
  exit 1. The two errors were fixture defects in the new Slack-on/off cases:
  save_fixture correctly rejects appending two revisions in a single save.
- [Initial corrected-code attempt](owner-fidelity-74-green.txt): 6 tests,
  4 passed, 2 fixture errors, exit 1. Preserved, not represented as green.
- The fixture was corrected to save one revision at a time.
- [Corrected base replay](owner-fidelity-74-red-corrected.txt): 5 test methods,
  15 failure records, zero errors, exit 1. All five new methods fail against
  the frozen base modules; no checkout files were replaced.
  [Replay script](owner_fidelity_74_base_repro.py).
  Failure names: test_resolved_decision_slack_line_survives_omission_with_slack_on;
  test_resolved_decision_slack_line_survives_omission_with_slack_off;
  test_historical_resolutions_also_retain_their_answered_revisions;
  test_omitted_settled_history_is_distinct_from_unavailable_status (2 subcases);
  test_partial_status_cli_redacts_untrusted_error_details (10 subcases).
- [Corrected focused green](owner-fidelity-74-green-corrected.txt): 6 passed
  in 1.433s, zero failures/errors/skips, exit 0; includes actual SQLite contention.

Final-tree commands (same sanitized environment above, sequential):

```text
python -m unittest discover -s scripts/initiative_control -p 'test_*.py' -v
python -m unittest -v test_owner_daemon test_decision_feed test_attention
```

[Full discovery](owner-fidelity-74-suite.txt): **800 passed in 438.777s**,
zero failures/errors/skips, exit 0.
Includes all 91 decision-manager ManagerTests, all 11 fable-launcher RealTmux
cases, and all 48 owner_tmux tests. In particular,
test_ack_start_return_resume_and_clean_shutdown and
test_bracketed_paste_echo_is_not_ack_and_start_is_denied both passed.
No load-sensitive failure was excluded, retried or relabeled.
Python HTTP fixture ResourceWarnings are retained in the log; these are not
test failures. Expected timeout fixture output is also preserved.

[Separate owner/feed/renderer run](owner-fidelity-74-focused.txt):
**166 passed in 59.934s**, zero failures/errors/skips, exit 0:
107 owner-daemon, 38 decision-feed, 21 attention tests.
Thus the specifically requested owner/feed total is **145 passed**.

[Final source hashes](owner-fidelity-74-source-hashes.txt) verify unchanged after
both final runs. No formatter was run; git diff --check passes.
Final git status shows only the six allocated source/test paths and this
allocation's new QA evidence under the permitted management-bootstrap directory.
No native credential prompt was observed.

## Changed lines and brief corrections

Six source/test files: 110 additions, 5 deletions, **115 changed lines**.
attention.py +17/-0; decision_feed.py +5/-2; owner_daemon.py +4/-2;
test_attention.py +23/-0; test_decision_feed.py +38/-0;
test_owner_daemon.py +23/-1. activate.py is unchanged.
New QA evidence is separate from this source/test count.

No substantive factual correction to the brief: both reported regressions were
present. The bound concerns local observation ingestion/rendering and snapshot
transport, not external Slack message transport. Historical round-71 evidence
is preserved and is not relabeled as acceptance of this revision.
