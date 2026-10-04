# RETURN — acct-qualify-55

Allocation c801ac87af9d13089ef897ebff05e0c0820ced3af3d858351ca5dcfaa83a7fdd;
claim 97e14251-5dd6-4f49-b1da-ce4e510a2e9c; worker gpt-6-astra/high.
Brief SHA-256 verified: 3f24895054501db362f71eb31ab2da6bdf52ab6702f3c1b594194076b33378f0.
Tested source commit c02865ef6c3e862f93ed687039e187ffe00355e0.
Candidate Corbanu Terminal v0.1.42, debug developer-accounting CLI;
SHA-256 f78a1cc8a433030d3f50d91cfd415946e5fe3dd4e08e69d5c9644244c0d018bb.
[Manifest](run-06/manifest.json), [classification and boundaries](PREFLIGHT.md).

The revised code-aware PTY qualification passed: **8 buckets, 4 breakdown groups,
1 partial-range case, 5 unknown-ancestry scenarios**. A separate exact Decimal
audit passed **48 rendered numeric pages**. This is supporting developer evidence,
not independent acceptance or an authorization to distribute the feature build.

## Boundary sampling and exact membership

Seven historical requests were actually sent by the CLI to the loopback provider:
six in the selected root tree and one independent root. A further Oct 2 synthetic
request resumes the selected root, advances the normal checkpoint, and is outside
every comparison. Its historical wire phase was mistakenly labelled orphan;
round 58 labels a fresh replay checkpoint and verifies the stored selected-root
identity. The historical wire receipt remains unchanged.
The clock leaves monotonic timers and OS wait deadlines real. Exact times are
sampled admission times, not post-hoc edits to accounting rows.

Each emission below has a unique model/input-count/time signature. Expectations
come from server-emitted counts and hardcoded Sol/Terra rates using Decimal.
The driver visits every rendered request and attempt in each bucket and compares
the complete membership multiset against those emissions, including cardinality
and duplicate-ID checks. Bound prices are checked against constants; they never
supply the expected amount.

| Emission | UTC admission | Role/model | Exact USD |
| --- | --- | --- | ---: |
| E0 | Aug 30 23:00:00.000 | Root / Sol | 0.00071 |
| E1 | Aug 30 23:00:00.000 | Root / Sol | 0.000715 |
| E2 | Aug 30 23:00:00.000 | Native descendant / Sol | 0.000535 |
| E3 | Aug 31 00:00:00.000 | Root / Sol | 0.00072 |
| E4 | Aug 31 23:00:00.000 | Root / Terra | 0.0003625 |
| E5 | Sep 1 00:00:00.000 | Root / Terra | 0.000365 |
| E6 | Sep 1 00:00:00.000 | Separate root / Sol | 0.0002975 |

All dates are 2026, all providers openai. E6 is a valid unrelated root during the
range tests and is excluded. It becomes the explicit metadata-fault subject later.

| Half-open UTC bucket | Exact included emissions | Exact USD |
| --- | --- | ---: |
| Hour Aug 30 23:00–Aug 31 00:00 | E0, E1, E2 | 0.00196 |
| Hour Aug 31 00:00–01:00 | E3 | 0.00072 |
| Hour Aug 31 23:00–Sep 1 00:00 | E4 | 0.0003625 |
| Hour Sep 1 00:00–01:00 | E5 | 0.000365 |
| ISO week Aug 24–Aug 31 | E0, E1, E2 | 0.00196 |
| ISO week Aug 31–Sep 7 | E3, E4, E5 | 0.0014475 |
| Calendar month Aug 1–Sep 1 | E0, E1, E2, E3, E4 | 0.0030425 |
| Calendar month Sep 1–Oct 1 | E5 | 0.000365 |

Every other root-tree emission is excluded from each row; the receipt lists them
explicitly alongside the actual attempt UUIDs. **E3 lands exactly on the ISO-week
boundary and E5 exactly on the calendar-month boundary**. Both are absent from
the earlier bucket and present in the later one, including the adjacent hours.
No comparison relies on “each range equals the single populated day.”

For **Aug 30 23:30–Aug 31 01:00**, the first hour is explicitly
“Partial bucket — excluded from totals”; the range total is unavailable and
the partial page offers no numeric subtotal. The following whole hour contains
only E3, exactly 0.00072 USD. The three 23:00 emissions remain excluded.

[Independent emissions/arithmetic](run-06/independent-arithmetic.json),
[wire receipts](run-06/requests.json),
[per-bucket membership, exclusions, groups and partial result](run-06/results.json),
[partial page](run-06/partial-bucket-selected.json),
[exact numeric audit](run-06/numeric-audit.json). Adjacent selected JSON,
viewport .txt.gz and raw PTY .raw.gz files preserve actual displayed values/keys.

## Breakdown no longer echoes ancestry

On the August month page:

| Group | Emissions | Exact USD |
| --- | --- | ---: |
| Root's own attempts | E0, E1, E3, E4 | 0.0025075 |
| Descendants | E2 | 0.000535 |
| openai / Sol | E0, E1, E2, E3 | 0.00268 |
| openai / Terra | E4 | 0.0003625 |

The native root and descendant share Sol; the same root also has Terra requests.
The separate root E6 uses Sol while the selected root E5 uses Terra at the same
September boundary. Both requirements therefore have real sampled counterparts.

Own + descendant and Sol + Terra each reconcile to 0.0030425, but neither
individual model subtotal equals its ancestry counterpart. Merely echoing
own/descendant now fails both model assertions. The earlier one-to-one fixture
could not detect that defect. Actual group counts and exact amounts were checked;
provider diversity beyond openai is not claimed.

## Reproducible UTC citation

The tracked historical receipt remains linked from acct-qualify-50/RETURN.md.
[verify_utc_alignment.py](verify_utc_alignment.py) regenerates its bytes from the
three preserved PTY selections into this round-55 directory and compares them
with the untouched round-50 receipt, failing on mismatch: 3/3 labels pass. It is described as a **label** check; the new multi-bucket evidence
above supplies actual boundary membership. Round 58 corrected the producer's former write to the closed receipt; the
historical bytes were identical, but that write path was inappropriate. New
reproductions leave the historical receipt untouched.

## Unknown ancestry

The sampled E6 request is reused across isolated metadata scenarios. Only its
native thread source/edge changes; its accounting attempt, usage and price are
untouched. Source/edge inputs are recorded in results.json and restored afterward.

| Cause | Injected condition | Observed Sep 1 result |
| --- | --- | --- |
| Malformed source | Unparseable source plus a surviving edge to the selected root | One unknown attempt |
| Absent ancestor | Source and edge agree on a nonexistent ancestor | One unknown attempt |
| Conflict | Source names the descendant, edge names the root | One unknown attempt |
| Cycle | Source and edge form a self-cycle | One unknown attempt |
| Missing edge | Source names the root, no matching spawn edge | One unknown attempt |

In **all five**, the rendered root total stays **one E5 attempt, 0.000365 USD**.
The separate unknown population contains **one E6 attempt**, whose drilled-down
thread identity and **0.0002975 USD** match its real sampled request. It is never
silently added to the root or its descendant total. The prior valid `unknown`
source scenario is preserved in acct-qualify-50; this revision also tests actual
unparseable source and source-without-edge instead of assuming those are identical.

## Reader verdicts

[Full per-finding assessment](reader-findings.md), retaining every prior finding:

- **Range too large:** scope explanation defect; blames the requested range for
  unrelated whole-store budget exhaustion. No false numeric amount is displayed.
- **Duplicate child-start messages:** can imply duplicate workers or chargeable
  work, so amount/scope reporting defect; no evidence of duplicate charging.
- **Epoch dates:** can mislead about covered dates, cutoff or freshness, hence
  scope/validity presentation defect rather than merely ugly text.
- **Some/None:** None leaves no-data versus unknown versus unconstrained coverage
  ambiguous; substantive scope explanation defect plus cosmetic Rust syntax.
- Rounded unknown-parent overview with exact value one click deeper: navigation
  polish on this evidence.
- Conservative freshness warning, disclosed component rounding, and explicitly
  unavailable settlement comparison: no false amount/scope claim established.

No production fix or severity approval is invented.

## Required gates

One dedicated shared target; CLI/rmcp prerequisites built first. All test commands
run from codex-rs through guarded just test, with two build jobs and one nextest
thread. No raw cargo test/nextest, broad formatting, live profile, native
credential prompt/authorization, or push.

| Lane | Exit | Run | Passed | Failed / timed out | Skipped |
| --- | ---: | ---: | ---: | ---: | ---: |
| just test -p codex-core accounting | 0 | 124 | 124 | 0 / 0 | 3545 |
| same, --features codex-core/developer-accounting | 0 | 127 | 127 | 0 / 0 | 3545 |
| just test -p codex-tui usage | 0 | 91 | 91 | 0 / 0 | 4078 |

**342/342 pass. Exact failure names: none.** Both Core lanes have one slow passing
test: `suite::accounting_responses_ws_recovery::accounting_responses_ws_native_auxiliary_scope_and_event_parity`.
No failure attribution or retry of these lanes. Prerequisite/final feature builds
also exit 0. [Commands and exits](lanes.json), [test counts/names](test-results.json);
all full logs preserved losslessly as adjacent .log.gz.

## Preserved fixture attempts

1. run-01: exit 1, `RuntimeError: Timed out: first-startup`; no provider request.
   In-process thread/start and skills/list timed out after 30 seconds.
2. run-02: exit 1, same startup failure after replacing file clock reads with
   immutable environment time. The process sample shows parking_lot's timed
   waits reading the interposed gettimeofday. Keeping gettimeofday real removed
   the startup problem; this is a fixture repair, not an attributed product regression.
3. run-03: exit 130, deliberately interrupted after three responses because the
   driver's user-message-based classifier did not recognize the same-model child.
   Changed classification to the native parent-thread header; no credential
   headers were recorded.
4. run-04: exit 130, deliberately interrupted after four responses because resume
   retained the prior Sol model despite a changed config default. The fixture now
   supplies the explicit supported resume --model override.
5. run-05: exit 1, assertion expected textual `0.001960`, while the correct
   displayed exact value was `0.00196`. Decimal normalization fixes the fixture;
   the separate numeric audit now requires numeric equality, not substring equality.
6. run-06: exit 0; all revised cases pass. Exact numeric audit exit 0, 48 pages.
   Historical UTC-label regeneration exit 0, 3/3.

Every attempt has separate raw captures and driver/helper versions. Earlier clock
sources were reconstructed only after matching their contemporaneous manifest
SHA-256, then preserved as clock.c.gz; the process sample is also preserved.
The unsuccessful sample command against already-exited run-01 returned 255;
the run-02 process sample succeeded. A structured-edit exact-match failure made
no edit and was corrected after reading the line. No attempt is erased or
reclassified as a successful replay.

## Changed lines and brief corrections

- **0 production Rust lines; 0 existing Rust test lines.**
- **473 added QA program lines:** qualify.py 277, fixture_clock.c 38,
  campaign.py 34, audit.py 52, verify_utc_alignment.py 17, summarize.py 55.
  Of these, 418 implement/run/check qualification and 55 assemble evidence.
- Historical RETURN citation: **1 line added / 1 removed**.
- New reports, ignore rules, selected JSON, captured keys and compressed evidence
  are itemized by line count/size and SHA-256 in [inventory.json](inventory.json);
  the inventory excludes itself. No file outside the writable QA prefix changed.
- No commit or push. [Reproduction instructions](REPRODUCE.md).

No substantive error was found in this revision brief. One precision point:
the prior source value was the valid `unknown` variant, not malformed JSON.
The implementation also distinguishes a source-named parent with a missing
edge, so that case is explicitly covered in addition to the requested four
representative causes. The accepted 571,429 empty-day / 571,428 one-attempt-day
threshold correction is retained; this revision does not repeat that large-store test.

Independent acceptance, other platforms/providers, live-repository qualification,
named-human acceptance and the sprint's overall handoff remain outside this
evidence-only worker's claim. Plan/sprint coordinate reconciliation remains with
the manager. No production amount or bucket-membership defect was observed in
the revised cases.
