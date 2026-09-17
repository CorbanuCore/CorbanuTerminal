# RETURN — acct-boundaries-58

Allocation digest 466d2ff0ce149802cc805c5803a8f68144fa2fb73d2215832b56f115427c2d8f;
claim bab66bb6-48f3-4258-b842-86d05c58a593; gpt-6-astra/high.
Frozen brief SHA-256 verified:
d9418de93cd80462d9ae75486547b20e462d247445b87f1e867b9e86674a0aa8.
Base/tested source: 32f237eddb4ddf94e23da5699dbb8148c3e4c289.
Candidate: Corbanu Terminal 0.1.42, debug developer-accounting CLI,
SHA-256 f78a1cc8a433030d3f50d91cfd415946e5fe3dd4e08e69d5c9644244c0d018bb.
[Manifest](boundary-run-03/manifest.json), [scope/classification](PREFLIGHT.md).

**Four P3 corrections have passing evidence. The next increment passes 26 boundary
cases plus independent arithmetic. Required Rust gates pass 342/342.**
This is code-aware developer evidence, not independent acceptance or sprint
completion. No production file, release, approval, commit or push is claimed.

## P3 corrections and what the checks now catch

1. **Audit membership:** audit.py selects emissions using driver-recorded windows
   in results.json and checks rendered bucket labels against those windows.
   It no longer reads membership from the page's Bucket header or gates on
   results.passed. Historical run-06 and fresh p3-run-01 each pass 48 exact numeric
   pages and 48 bucket-label checks. A synthetic bucket relabelled to its neighbour
   with the neighbour's internally consistent amount now fails “wrong bucket
   label”; a wrong amount also fails. Setting results.passed=false does not affect
   this audit. Arithmetic and expected windows are independent of product output;
   native identity/ancestry still use store reads. No storage-independent ancestry
   proof is claimed.
2. **Unknown ancestry:** each of five committed source/edge injections is read back
   immediately before its render, asserted against its cause, and recorded.
   Each case starts a new inspector process. Five distinct causes, five distinct
   readback hashes and five distinct PIDs are verified. Reusing the first
   readback fails each of the other four cause matches. All five still render
   root E5=0.000365 and unknown E6=0.0002975, excluding E6 from the root.
   Identical monetary screens are expected; cause evidence is now distinct.
3. **Historical receipt:** verify_utc_alignment.py writes
   acct-qualify-55/utc-alignment-regenerated.json and compares bytes against the
   round-50 receipt, failing on mismatch. Reproduction leaves historical bytes
   **and modification time unchanged**. A tampered synthetic historical receipt
   produces exit 1, “Historical receipt mismatch”. The round-50 RETURN again links
   its own tracked receipt; “orphan-artifact citation” wording is corrected.
   Original raw results remain untouched.
4. **Checkpoint role:** the new Oct 2 emission is labelled checkpoint and its
   stored attempt belongs to the selected root. There is exactly one unrelated
   orphan wire row. The checkpoint is outside every compared August/September
   range. Historical requests.json is preserved, with its old label explained
   in the historical RETURN.

Evidence: [negative controls](p3-negative-controls.json),
[fresh P3 numeric audit](p3-run-01/numeric-audit.json),
[historical numeric audit](historical-numeric-audit.json),
[cause readbacks and driver windows](p3-run-01/results.json),
[wire records](p3-run-01/requests.json).

## Boundary increment: independent arithmetic and reader output

Every amount uses server-emitted counts:
((input − cached) × noncached rate + cached × cache rate + output × output rate)
÷ 1,000,000. Sol rates are 5 / 0.5 / 30; Terra rates are 2.5 / 0.25 / 15.
Reasoning is a subset, not another charge. All seven token measures are checked
on numeric pages. No stored amount supplies an expectation.

| Emission | UTC timestamp | Role/model | Exact USD |
| --- | --- | --- | ---: |
| E0 | Mar 8 06:59:59 | Root / Sol | 0.00071 |
| E1 | Mar 8 06:59:59 | Root / Sol | 0.000715 |
| E2 | Mar 8 06:59:59 | Descendant / Sol | 0.000535 |
| E3 | Mar 8 07:00:00 | Root / Sol | 0.00072 |
| E5 | Oct 31 23:00:00 | Root / Sol | 0.000725 |
| E6 | Nov 1 05:59:59 | Root / Sol | 0.00073 |
| E7 | Nov 1 06:00:00 | Root / Terra | 0.0003675 |
| E8 | Nov 2 00:00:00 | Root / Terra | 0.00037 |

These dates are 2026. E4 (Mar 10), E9 (Nov 4), and E10 (Jan 30, 2027)
are selected-root maintenance requests outside every compared range.
Complete counts/rates/times are in [independent-arithmetic.json](boundary-run-03/independent-arithmetic.json).

| Case (half-open UTC) | Independent membership/arithmetic | Reader sees |
| --- | --- | --- |
| Spring 06:00–07:00, four TZ settings | E0+E1+E2 = 0.00196 | “Known subtotal exact USD: 0.00196” |
| Spring 07:00–08:00, four TZ settings | E3 = 0.00072 | “Known subtotal exact USD: 0.00072” |
| Mar 8 UTC day, four TZ settings | E0+E1+E2+E3 = 0.00268 | “Known subtotal exact USD: 0.00268” |
| Before child deletion, Mar 8 | 0.00268 | “Known subtotal exact USD: 0.00268” |
| After native child deletion, Mar 8 | 0.00268−E2 = 0.002145 | “Known subtotal exact USD: 0.002145” |
| Fall 05:00–06:00 | E6 = 0.00073 | “Known subtotal exact USD: 0.00073” |
| Fall 06:00–07:00 | E7 = 0.0003675 | “Known subtotal exact USD: 0.0003675” |
| Nov 1 UTC day before compaction | E6+E7 = 0.0010975 | “Known subtotal exact USD: 0.0010975” |
| Empty Nov 3 UTC day | No emissions = 0; all seven measures zero | “No recorded attempts in this day; collection coverage unknown.” |
| Oct 31 partial leading day (requested from noon) | E5 = 0.000725; excluded | “Partial bucket — excluded from totals” |
| Nov 2 partial trailing day (requested until noon) | E8 = 0.00037; excluded | “Partial bucket — excluded from totals” |
| Full Nov 1 middle of that range | E6+E7 = 0.0010975 | Exact bucket subtotal; range says “Range total unavailable — partial or unavailable buckets excluded; no partial total.” |
| Mixed compact/raw Nov 1 day | E6 compact 0.00073 + E7 raw 0.0003675 = 0.0010975 | “Request detail unavailable for this whole UTC day — compacted history lost request/provider attribution. No total shown.” |
| Mixed range, compact 05:00–06:00 hour | E6 = 0.00073; withheld | “Precision unsupported outside retained raw detail; compacted days lost request/provider attribution. No bucket total.” |
| Mixed range, raw 06:00–07:00 hour | E7 = 0.0003675 | Exact amount and request/attempt drill-down still available |
| Expired Jan 1, 2025 range | No fixture emissions; 0 arithmetic is **not** evidence of no historical spend | “Range total unavailable — partial or unavailable buckets excluded; no partial total.” |
| Deleted history after normal retention, Mar 8 | E0+E1+E3 = 0.002145, E2 still absent | Whole-day compact-history refusal; no total |

The three four-TZ rows represent 12 cases; the remaining rows represent 14,
for **26 separately recorded/audited cases**. Both partial end-day samples are
inside the requested clipped interval, so exclusion is not explained by missing
traffic. The hypothetical requested-range sum is 0.0021925, and is never shown
as a complete range total.

Maintenance at Jan 30, 2027 05:59:59Z yields cutoff Nov 1, 2026 05:59:59Z:
E6 at the cutoff is compacted; E7 one second later remains raw.
The final audit checks **four raw attempts and four compact-day aggregates**
against emitted membership, fixed-rate costs, counts and all seven measures.
Native thread/delete removes E2; no SQL accounting mutation is used.
The later compact March aggregate remains exactly 0.002145, with no child rows.

[26-case driver evidence and exact rendered rows](boundary-run-03/boundary-results.json),
[final independent audit](boundary-run-03/independent-audit-final.json),
[raw/compact readback](boundary-run-03/mixed-store-readback.json),
[native deletion RPC](boundary-run-03/delete-rpc.json),
[deletion readback](boundary-run-03/deleted-store-readback.json).
Selected JSON plus adjacent .txt.gz/.raw.gz and keys.json preserve real-key reads.

## Timezone and DST behavior

TZ=America/New_York, UTC, Asia/Kolkata and America/Phoenix produce identical
spring UTC-hour memberships/totals and a 24-hour UTC day.
New York jumps from 01:59:59−05:00 to 03:00:00−04:00 at spring 07:00Z,
and repeats from 01:59:59−04:00 to 01:00:00−05:00 at fall 06:00Z.
The product still uses 3,600-second UTC hours and 86,400-second UTC days.
There is no local 23/25-hour rebucketing. Explicit local-offset timestamps are
rejected with **“Range refused: timestamps must use UTC Z”**.
[Conversions](boundary-run-03/dst-conversions.json), [input refusal](boundary-run-03/offset-input-result.json).

## Reader findings

[Exact text and assessments](reader-findings.md) retain the “Range too large”
whole-store-budget scope finding from round 50, and reproduce epoch-day/epoch-ms
and Some/None scope ambiguities in the new run. The large-store threshold is not
rerun or reclassified. The mixed whole-day refusal may obscure that a smaller
raw hour remains inspectable; its exact wording and successful neighbour are
recorded. Deletion correctly reduces recorded cost but adds no deletion-specific
explanation. No arithmetic defect was observed.

## Required lanes and preserved attempts

Prerequisites built first; all lanes used the same dedicated CARGO_TARGET_DIR
from codex-rs, guarded just test, NEXTEST_TEST_THREADS=4 and two build jobs.

| Lane | Exit | Run/pass | Nonzero tests | Exact failure names |
| --- | ---: | ---: | ---: | --- |
| just test -p codex-core accounting | 0 | 124/124 | 0 | None |
| same with --features codex-core/developer-accounting | 0 | 127/127 | 0 | None |
| just test -p codex-tui usage | 0 | 91/91 | 0 | None |

**342/342; no failed or timed-out tests.** Build prerequisites and final feature
binary both exit 0. [Commands/exits](lanes.json), [counts](test-results.json),
full lossless lane logs adjacent. No full-parallelism attribution was attempted.

[All attempts](attempts.json): P3 replay exit 0. Boundary run-01 deliberately
interrupted (130) to correct fixture expectations before compact/deletion work;
run-02 exit 1 on the case-sensitive scrolled-header startup predicate;
run-03 exit 0. No earlier result is erased. The supplemental audit first hit
a Python regex-escaping error, corrected without changing product/capture data;
its final execution passes and its failure is retained in audit-attempts.json.

## Changed lines and brief corrections

**0 production Rust lines and 0 Rust test lines.** New QA programs: **571 lines**.
Existing QA programs: **+96/−52**; existing explanatory records: **+20/−11**
(combined existing diff **+116/−63**). New reports, selected JSON and lossless
captures are separately enumerated with line counts, sizes and SHA-256 in
[inventory.json](inventory.json); the inventory excludes itself.
All changed paths are inside the assigned QA prefix. No broad formatter used.

No substantive error found in the brief. Its historical full-parallelism
deadline observation is neither tested nor disputed by this four-thread campaign.
The important behavior clarified by execution is that retained compact dollar
aggregates do not make a complete request-inspector total available: attribution
loss is disclosed and the numeric total is withheld.

[Reproduction](REPRODUCE.md). Independent/code-blind acceptance, other platforms,
live-repository qualification, named-human acceptance and the overall sprint
handoff remain with the manager.
