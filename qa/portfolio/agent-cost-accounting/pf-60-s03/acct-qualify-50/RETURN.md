# RETURN — acct-qualify-50

Allocation digest: e739ccccf7fb042bf74e7ea6e864866c54328e4240083d10934cc52e638f910c. Claim: 92b6b02c-a325-40c6-a384-a47ee471c515. Worker: gpt-6-astra/high.

Brief SHA-256 verified: 7d8920cd6bc471924cf4dc4fce4de2ff3f3fcb98d2d3abbdd2406f629fce33f0.
Base and tested source commit: 8b968c94b1384e14379b610249d3b88f41b684c2.
Candidate: Corbanu Terminal v0.1.42, debug CLI built with codex-core/developer-accounting.
Executable SHA-256: f78a1cc8a433030d3f50d91cfd415946e5fe3dd4e08e69d5c9644244c0d018bb.

Classification, product citation, active plan/sprint and coordinate discrepancy are in [PREFLIGHT](PREFLIGHT.md). This is qualification tooling and evidence only. No inspector, activation, never-distribute guard, pricing implementation, Rust test or production source changed. Nothing was pushed.

## Sampled data and independent arithmetic

The actual CLI sent four HTTP Responses requests to a loopback provider: two root Sol responses, one Terra child response through native spawn_agent, and one separate Sol response. The [server receipts](collect-04/requests.json) preserve emitted token counts, requested models, endpoint paths and fixture/real UTC timestamps; no request headers or prompt bodies were retained there.

The wall-clock-only fixture sampled those requests on August 10 around 12:15 UTC. Monotonic timers remained unshifted. The CLI reopened at real current time; one new current-day sample advanced the normal ledger checkpoint and is outside every August comparison. [Clock control](clock-control.json), [sandboxed clock control](sandbox-clock-control-02.json), [candidate manifest](collect-04/manifest.json).

Unknown ancestry was a deliberate metadata fault injection: after the independent root's real sampled response, its native thread source was changed to unknown. Its token observations, price binding and amounts were not manufactured or changed. This tests a nonempty unresolved population, not spontaneous loss of ancestry in a normal user flow.

Prices were frozen independently from the bundled catalogue and then compared with the [actual bound original snapshots](collect-04/bound-prices-and-attempts.json). Expectations never read stored estimates or contributions. USD per million tokens: Sol input 5 / cache read 0.5 / output 30; Terra input 2.5 / cache read 0.25 / output 15. The custom fixture model catalogue changes only tool presentation fields; accounting binds the bundled authority.

| Path | Independent calculation, USD | Exact rendered amount |
| --- | --- | --- |
| One root Sol response | ((100−20)×5 + 20×0.5 + 10×30)/1,000,000 | 0.00071 |
| Root's own attempts, two responses | 2×0.00071 | 0.00142 |
| Descendant Terra response | ((80−10)×2.5 + 10×0.25 + 6×15)/1,000,000 | 0.0002675 |
| Root plus resolved descendants | 0.00142 + 0.0002675 | 0.0016875 |
| Provider openai / model gpt-5.6-sol group | Two root responses | 0.00142 |
| Provider openai / model gpt-5.6-terra group | One descendant response | 0.0002675 |
| Unknown-parent attempt, excluded | ((40−5)×5 + 5×0.5 + 4×30)/1,000,000 | 0.0002975 in its attempt detail |

The day renders:

```text
Estimated token cost for recorded attempts: $0.001688 (rounded)
Known subtotal exact USD: 0.0016875
Input: 280 known + unknown in 0 attempts
Noncached input (derived for inclusive input): 230 known + unknown in 0 attempts
Cache read: 50 known + unknown in 0 attempts
Cache write: 0 known + unknown in 0 attempts
Output: 26 known + unknown in 0 attempts
Reasoning (subset, not separately billed): 10 known + unknown in 0 attempts
Total (not separately billed): 306 known + unknown in 0 attempts
Unknown parent population: 1 attempts, excluded from root total
```

Independent token arithmetic: input 100+100+80=280; cache read 20+20+10=50; noncached 280−50=230; output 10+10+6=26; reasoning 4+4+2=10; total 280+26=306. Cache write is explicitly reported zero. Reasoning and total are not added again to billed components.

Each root/descendant/provider-model group, all three logical requests and all four original-price attempt pages were opened with real keys. Actual full viewport captures and raw PTY streams are retained, alongside concise selected-row records. [Day rows](inspect-01/day-selected.json), [comparison result](inspect-01/comparisons.json), [frozen expectations](EXPECTED.md). Provider coverage is OpenAI with two models; this is not evidence of an Anthropic/OpenAI cross-provider run.

**No rendered monetary or token disagreement was found.** [Full rendered transcript](rendered-output.md) collects the actual reached screen rows for every data path, group, attempt, range, refusal and recovery. Lossless full-viewport and raw-PTY captures are included beside each run; JSON store dumps are not the rendering evidence.

## User-chosen ranges

| Group | Rendered half-open UTC bucket | Result against standalone August 10 |
| --- | --- | --- |
| Hour | 2026-08-10T12:00:00.000Z → 2026-08-10T13:00:00.000Z | Exact USD 0.0016875 and all seven token measures equal |
| ISO week | Monday 2026-08-10T00:00:00.000Z → Monday 2026-08-17T00:00:00.000Z | Same |
| Calendar month | 2026-08-01T00:00:00.000Z → 2026-09-01T00:00:00.000Z | Same |

All historical sampled traffic lies within the same hour; there was no traffic on the other days of the complete week/month. Each overview and bucket page was traversed with real keys. Every bucket is whole and available, and its exact subtotal and token measures match the independent arithmetic above. The excluded unknown-parent attempt remains separate. [Original UTC label receipt](inspect-01/utc-alignment.json), [non-destructive reproduction and comparison](../acct-qualify-55/verify_utc_alignment.py), [hour](inspect-01/hour-bucket-selected.json), [week](inspect-01/week-bucket-selected.json), [month](inspect-01/month-bucket-selected.json).

Recovery passes after padding removal: the resumed TUI renders exact 0.0016875 again. Additional unaligned starts (hour 12:15, week Monday 12:00, month August 10) produce the full UTC hour/Monday-week/calendar-month boundaries and label each bucket partial. All three withhold both range and bucket monetary totals despite containing sampled attempts. [Recovery and alignment result](recovery-01/recovery-result.json); actual viewport and selected-row captures are under recovery-01/.

## Refusals and the measured whole-store threshold

The load fixture inserts explicitly artificial, unselected attempt rows on another day. They contain no fabricated usage, prices or estimates and are not claimed to be fully admitted accounting records or sampled traffic. This measures the global row-budget preflight through the real TUI; monetary qualification uses only actual sampled requests.

| Whole-store attempt rows; compact rows = 0 | Empty August 8 | Current day with one sampled attempt |
| ---: | --- | --- |
| 571,427 | Complete view | Complete view |
| 571,428 | Complete view | TooLarge; no total |
| 571,429 | TooLarge; no total | TooLarge; no total |
| 666,666 | TooLarge; no total | TooLarge; no total |
| 666,667 | TooLarge; no total | TooLarge; no total |

The measured empty-day boundary is therefore **571,429 rows**, with the one-attempt fixture refusing already at **571,428**. The roughly 666,000 estimate misses the extra whole-store scan in the exposed ancestry-aware day path. Its empty-day work is at least 7N+1: 7×571,428+1=3,999,997 fits; 7×571,429+1=4,000,004 exceeds the 4,000,000 budget. Additional selected work consumes more. The denominator counts attempt plus compact-day rows, not every SQLite table; this measurement uses zero compact rows.

The actual refusal says:

```text
Range too large for this inspector. No total shown.
Collection coverage: unknown; recorded root and resolved descendants only. Unknown parent population excluded.
Billed cost: unavailable — no settlement evidence
Logical requests may have attempts on other days; this UTC day is not their complete lifetime.
```

No known subtotal or estimated token-cost amount appeared in any refusal. [Measurements](limits-01/measurements.json), [empty-day refusal rows](limits-01/rows-571429-day-2026-08-08-selected.json). Recorded timings include key sending, waiting and page traversal; they are not query-only benchmarks. Padding removal restored the fixture to its five actual sampled attempts.

The separate **513 real sampled attempts** cap test passes (exit 0): 513 loopback Responses replies, 513 stored attempts, and the same TooLarge refusal above with no monetary total. Every response reported input 10, cache read 0 and output 1; independent hypothetical full cost is 513×(10×5+1×30)/1,000,000 = 0.04104 USD. The inspector shows none of that amount and no truncated substitute. [Cap result](cap-01/cap-result.json), [wire receipts](cap-01/requests.json), [rendered refusal](cap-01/cap-refusal-selected.json). First-to-last provider response took 21m45.763s; this is an observed fixture duration, not an attributed performance cause. This cap uses actual sampling, unlike the separately disclosed whole-store padding.

## Things a reader could misunderstand

- “Range too large” blames the selected range even when it is an empty day and unrelated whole-store growth causes refusal. The screen does not disclose the exhausted work budget or useful recovery.
- The sampling transcript prints “Started /root/qualify_child” twice for the one native child and one child HTTP response. That can suggest duplicate work or cost even though the inspector's descendant total is correct. The duplicate lines are preserved in collect-04/root-and-child-complete.txt.gz.
- Epoch milliseconds, epoch-day integers and Rust-style Some(...) / None appear directly in freshness/retention explanations. These are hard to interpret as dates.
- The unknown-parent population overview offers a rounded dollar amount, while its exact amount requires opening the individual attempt; the own/descendant/provider-model group pages expose an exact amount directly.
- The observed day warns “Snapshot is not current” with only 624 ms of maintenance lag. This is an honest conservative disclosure, but the prominent warning can be read as failure of a just-completed historical inspection.
- Rounded components can differ from a rounded subtotal by USD 0.000001: the Terra displayed components 0.000175 + 0.000002 + 0.000090 sum to 0.000267, while its rounded subtotal is 0.000268. Exact values reconcile to 0.0002675 and the rounding labels are correct; this is a reading caveat, not an arithmetic defect.
- Price is explicitly an estimate; billed cost and estimate-versus-billed difference remain unavailable. No settlement comparison was tested or claimed.

## Required gates and exact failures

All commands ran from codex-rs, with one shared dedicated CARGO_TARGET_DIR reused across lanes, codex and rmcp-client binaries built first, and guarded just test. CARGO_BUILD_JOBS=2, CARGO_INCREMENTAL=0, NEXTEST_TEST_THREADS=1. No raw cargo test/nextest, live profile, native credential authorization, workspace formatter or fixer was used.

| Lane | Exit | Tests run / passed / failed | Skipped |
| --- | ---: | --- | ---: |
| Core default: just test -p codex-core accounting | 0 | 124 / 124 / 0 | 3,545 |
| Core feature: same plus --features codex-core/developer-accounting | 0 | 127 / 127 / 0 | 3,545 |
| TUI: just test -p codex-tui usage | 0 | 91 / 91 / 0 | 4,078 |

**342/342 passed. Exact failing test names: none.** Both Core lanes report one slow passing case: suite::accounting_responses_ws_recovery::accounting_responses_ws_native_auxiliary_scope_and_event_parity. No retry or failure attribution is made. Prerequisite and final feature CLI builds exit 0. [Commands, targets and wall times](lanes.json).

Preserved nonzero fixture attempts:

1. Sandbox clock control 01: exit 65, sandbox profile syntax rejected literal 127.0.0.1; platform requires localhost or *. Corrected localhost-only control 02 exits 0.
2. collect-01: exit 1, RuntimeError: Timed out: root-startup at provider onboarding; cleanup also raised OSError: [Errno 5] Input/output error after the child had exited. Synthetic file-backed login and robust cleanup were added.
3. collect-02: exit 1, StopIteration in database(): fixture lookup omitted the pfterminal_ filename prefix. Three real root/child responses had completed; that attempt was not relabeled as a completed qualification.
4. collect-03: exit 1, root-startup timeout. The actual screen reports thread/start and skills/list in-process app-server request timeouts after 30 seconds. No cause is assigned. The next replay initializes a separate disposable Git repository inside the fixture work directory.
5. collect-04, inspect-01, limits-01, recovery-01 and cap-01: exit 0. All owned execution sessions have finished.

Raw prior attempts, timeouts and screenshots are retained separately. Later runs include a compressed copy and hash of their exact driver. The first two attempts predate that driver-copy addition. One scope-audit helper assertion also failed because line-based parsing treated Git's quoted filenames containing spaces as outside the prefix; the NUL-delimited audit corrected the parser and confirmed no outside-scope paths. That was not a source-scope violation.

## Brief corrections and limits

- The claim that no populated inspector data path had ever been shown is too strong: acct-activation-20-pty-run-02 already captured a real loopback-sampled day and exact subtotal 0.000125. It did not cover this assignment's ancestry/range/cap breadth.
- The roughly 666,000-row exposed-day threshold is corrected by the adjacent-count measurement above.
- Plan/sprint coordinates still name acct-inspect-20260915; this frozen qualification allocation names the current checkout. Record reconciliation is outside the worker's writable scope.
- This is code-aware developer evidence on macOS with synthetic API-key/default-tier traffic, not a fresh-context permission-isolated executor or independent evidence review. Independent acceptance, other platforms/providers, live-repository qualification and named-human acceptance are not claimed. The debug feature build remains prohibited from distribution.

## Change accounting

Changed existing Rust test lines: **0**. Changed production/configuration lines: **0**. Added QA test-program lines: **559** (campaign.py 34, qualify.py 481, clock_probe.c 13, fixture_clock.c 31); removed test lines: **0**. Added evidence-assembly program lines outside tests: **64** (summarize.py); no production behavior is implemented there. Total new QA program lines: **623**.

Other new changes are the local ignore rules, reports, fixture receipts, rendered captures and lossless compressed logs. [Artifact inventory](artifact-inventory.json) enumerates each reviewable file's category, line count or compressed size, and SHA-256 rather than hiding generated evidence in a production-line count. It excludes its own self-referential hash/count. Raw originals and synthetic profiles remain local and ignored; compressed copies preserve the original attempts. No code outside the allocated QA directory changed, no existing source diff remains, and no commit or push was made.
