Conclusion: **Supported with corrections**

The four verdicts hold, and every headline number I could recompute matches the data and the captures exactly. No finding is blocking. Several claims go further than the evidence: two macOS points rest on evidence that was never captured, one PASS comes from a retry rather than a clean delete, one #289 "Fixed" was tested too narrowly, and the record cites a review that doesn't exist.

## Findings

1. **Correction: the README cites a review and corrections that don't exist.** `rerun/README.md:17` says the review "is REVIEW.md; the corrections it led to are listed at the end". There is no `rerun/REVIEW.md` and no corrections section. The line also says "Opus 5.5", which doesn't match the model this review is running on (claude-opus-5-5-plan, per `review-err.log`). Remove the line or replace it with this review once it is final. Separately, the link "[first independent run](../README.md)" doesn't resolve in this layout.

2. **Correction: on macOS, the #308 claim that rollouts were deleted has no evidence.** README §2 and issue #308 say the failed real-clock delete "removed the rollouts" (`mac-c9-03`, `-04`, `-05`). But `mac-c9-03` shows only the two errors and `exit=1`, with no rollout listing before or after. `mac-c9-04` and `-05` only show that the accounting rows were kept. The non-atomic delete is proven on Linux only (`rtx-c15-08` has the before/after listing). Either attach the macOS listings or cut the claim back to "same error, exit 1". #308's statement that the conversation is "already gone from `resume`" was not tested on either platform.

3. **Correction: the macOS criterion 9 PASS comes from a retry after a partial delete.** The deletes behind `mac-c9-07` and `-08` are the second attempt (`mac-c9-06`). The first attempt (`mac-c9-03`) had already failed partway through (#308). So macOS never shows a clean single delete. The disclosure text and counts (5 and 6) are correct, but they come from that recovery path. The verdict row should say "PASS (Linux: clean delete; macOS: via retry after #308 partial delete, clock faked ahead)". Linux (`rtx-c9-01`…`07`) is clean and fully supports the verdict.

4. **Correction: blaming the macOS hang on the harness is likely but not proven.** `data/mac-libfaketime-behind-probes.txt` shows three timeouts at `thread/start`. A ledger-free fresh home rules out accounting, but not other date-sensitive startup code in this binary. No control was run, such as another program or a default build, with the clock behind under DYLD libfaketime. The probe file also doesn't record each probe's faked time. Wording such as "NOT VERIFIABLE (blocked at `thread/start`; cause not isolated, most likely libfaketime on macOS)" would be accurate.

5. **Correction: the Linux criterion 15 PASS should carry its residuals in the verdict, and they need issue numbers.**
   - **Turns completing is proven.** All `rtx-sk58s`, `sk6d` and `sk95d` `.jsonl` files end in `turn.completed`. The log's `behind_ms` values (57999, 518378007, 8207957009) all point back to the same checkpoint, 16:07:49.155Z, so the test condition really was hit.
   - **Two behaviours are reported only as wording issues.**
     - Requests are written at the faked time, behind the ledger's checkpoint. That changes days the ledger had already covered, such as 10-02 and the 16:06 part of today. The product's own log says it should record "at the ledger's time".
     - `sk95d` sent 11 unrecorded requests, and `/cost` gives no sign of a gap. Since #286 added a disclosure for deleted spend, the lack of one for unrecorded spend is the same kind of gap.
   - **Missing issues.** These two points and the "in this fixture" log text (`data/accounting-log-lines-rtx.txt`) appear only under "For Travis", with no issue numbers.
   - **"exit 0" is not in the evidence.** Exit codes from `execrun` were not kept. Only `turn.completed` is.

6. **Correction: "Request numbering … Fixed" was tested too narrowly.** The first run's misordering was in a conversation with subagents ("Request 8" was first). The re-run checks only root-only conversations: `mac-289-02` (4 requests), `rtx-c15-02` (8) and `rtx-c15-05` (4). The subagent runs m1, m2 and d1 never had their request lists captured. Better: "not reproduced in root-only conversations; the subagent case was not retested."

7. **Correction: the default build now shows the accounting page.** In `rtx-289-01`, `/usage requests` in a default build opens a "Cost — this conversation" page with retention and coverage lines. The acceptance record says "shipping `/usage requests` remains held". The README files this only as a #289 wording residual. It should be flagged for a criterion 13 (developer-only activation) review. Also, `rtx-289-02` tells a default-build user to "Run /usage requests for today", which has the same pointless-next-step problem the README flags for `rtx-289-01`.

8. **Correction: the "Paths are redacted" claim has three exceptions.** `rtx-289-01`, `-02` and `-03` show `<rtx-dir>/…/repos/ansi-regex` in the TUI header box, while the rest of the record uses `<rtx-dir>`. No username, host or IP leaks: my grep for `/Users`, `/home`, `/Volumes`, IPs and key or Bearer patterns found only `127.0.0.1` for the proxy.

9. **Note: the criterion 8 FAIL is supported, but one capture is incomplete.**
   - **Supported quotes.** Both are in `mac-c8-01`: "zainousage · … 2 requests, tokens not reported. Estimated cost: no price available." and "Next step for requests with no price: check the bill from zaiproxy". `mac-c8-06` names only zaiproxy.
   - **Incomplete capture.** `mac-c8-02` (the provider page) starts at "Day covered". The capture script starts at the first line matching its header pattern, and "Day " matches. Anything above that line was cut, so "its own provider page has no next step" isn't fully shown. The FAIL still stands on `mac-c8-01`, `-03` and `-04`.
   - **Confound.** As the README says, a custom provider has no price either way.

10. **Note: some "independent" sources are partly product output.**
    - **Usage.** For built-in zai rows, usage comes from the product's own `codex_api=trace` log. Only the 4 proxy responses come from outside the product, and the 2 pass-through ones match `nu-proxy.err` exactly.
    - **Bucketing.** Day buckets use the product process's log timestamps, which libfaketime fakes.
    - **Supporting check.** The Z.AI response IDs encode the provider's real time (UTC+8). They confirm every faked offset: sk58s −55 to −57 s, sk6d −6.000 d, sk95d −95.000 d, m1 +0.73 d, mk +1.81 d.
    - **The README should say this.** It no longer labels these channels as semi-independent.

11. **Note: some claims can't be reproduced from the retained files.**
    - `data/anthropic-hits.txt` contains only `0`, and the `.err` files it scanned weren't kept. In what was kept, my grep finds "anthropic" only in README.md. The Anthropic-naming failure path wasn't triggered, so its absence is weak evidence.
    - The build hashes, the merge commit and the timing of PR #306 have no build logs or receipts.
    - "Unchanged copies" of the tools can't be checked against the first run, because its tools aren't in this directory.

12. **Note: key handling.**
    - **Scripts.** `execrun.sh` and `tui.sh` attach the helper call to the consuming command. `rtxrun.sh` pipes the key through `printf` over ssh stdin, which is disclosed. `tui-ft.sh` uses a placeholder key. The proxy never logs headers.
    - **Unscanned raw logs.** The key scan covered only committed evidence. The raw `RUST_LOG=codex_api=trace` stderr files and the TUI log DB stay in scratch on both hosts. The README doesn't say whether they were checked for request headers or deleted.
    - **Environment leak.** `LD_PRELOAD` reached the model's shell commands, so environment variables are inherited. Nothing shows whether `ZAI_API_KEY` was filtered out of tool calls.

13. **Note: smaller items.**
    - Captures are de-duplicated, which is now disclosed.
    - `rtx-c9-03` lacks the command line and exit code.
    - Pages for days with no deletions still use the old wording, "No other conversation recorded requests on this day" (`rtx-c15-02`, `mac-289-02`).
    - The #308 issue and README quotes match `rtx-c15-08` and `mac-c9-03`.
    - The quotes I checked from `rtx-c9-04`, `mac-c9-07`, `-08`, `mac-289-01`, `-03`, `rtx-289-01`…`07` and `rtx-c15-07` all appear in their captures.

## Recomputations
Prices: input 1.40, cached input 0.26, output 4.40 USD per 1M tokens, using exact decimals. Every result matches the README and the captures.

| Check | Result |
| --- | --- |
| sk0 + sk58s + sk-after | 8 req, 55,484 tok, 0.04154992 ($0.041550, `rtx-c15-01`/`03`) |
| sk0 + sk-after | 4 req, 27,736 tok, 0.01717748 (`rtx-c15-05`) |
| + sk-predel / sk-after + sk-predel | 5 req, 34,276 tok, 0.02473436 / 3 req, 20,363 tok, 0.01241908 (`rtx-c15-09`/`11`) |
| sk58s | 4 req, 27,748 tok, 0.02437244; input 27,563, cache 13,184, output 185, reasoning 36 (`rtx-c15-05`) |
| sk58s request 1 | 0.0091574 + 0.0001936 = 0.009351 (`rtx-c15-07`) |
| sk6d | 8 req, 57,308 tok, 0.01963924; reasoning 88; per-request amounts in provider order (`rtx-c15-02`) |
| sk95d | 11 req, 79,478 tok, 0.02831728 |
| d1 / d2 | 7 req, 36,646 tok, 0.02134052 / 7 req, 36,605 tok, 0.01309060 (`rtx-c9-01`/`02`/`05`) |
| m1, m2, m3, m2 + m3 | 0.05377672, 0.03420384, 0.00835228, 0.04255612 (98,306 tok) (`mac-c9-01`/`02`/`08`) |
| mk | 0.00807088 |
| nu-zai | 2 req, 25,889 tok, 0.02459416 (`mac-c8-06`) |
| nu-proxy | 20,344 tok; the proxy log and the trace log agree on both responses |
| Stripped proxy | 2 req, 20,368 tok, 0.00599768 |
| TUI turn | 4 req, 59,017 tok, 0.04706756; requests in order 0.0189336, 0.00406064, 0.0193298, 0.00474352 (`mac-289-02`) |
| Request 3 components | 13,497 × 1.4 = 0.0188958; 1,280 × 0.26 = 0.0003328; 23 × 4.4 = 0.0001012; total 0.0193298 (`mac-289-03`) |
| Spend | Linux 44 req, 0.13663476; macOS exec 22 req, 0.18062012; `recompute-mac` "ALL" 24 req, 0.1866178; overall 72 req, 0.37032012 |
| Clock offsets (from response-ID times) | All runs are where the README says they are |
| Duplicates | No duplicate response IDs across the usage files |
