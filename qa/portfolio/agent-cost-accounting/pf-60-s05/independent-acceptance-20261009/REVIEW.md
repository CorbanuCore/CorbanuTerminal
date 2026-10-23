# Independent evidence review of the PF-60-S05 acceptance record

Reviewer: separate code-blind session, installed `corbanu exec -m claude-opus-5-5-plan -c model_provider=claude-plan -c model_reasoning_effort=high -s read-only`, run 2026-10-09 in a packet holding only a copy of this directory (first draft) plus the sprint record, defaults table, options memo, coverage audit, ledger-format check and the S03 body review. It was told not to read source. The prompt is in `data/review-prompt.md`. Report below is verbatim apart from path redaction; the executor's responses are in README.md, "Review corrections applied".

# PF-60-S05 code-blind evidence review

**Conclusion: Supported, with corrections.** I recomputed every headline number and none disagrees with the record. Three verdicts overstate what the evidence shows: AC6, AC7 and AC11. So do the README's top-line and "For Travis" summaries.

## Findings by severity

### High

**H1. AC7 "PASS (compaction)" is too generous.**
- **The body review's Major 4 scenario was never injected.** That scenario is an observation that fails *after* a paid response. Every failure injected here hit before the request was sent:
  - the trigger failed at `step="admit attempt"`;
  - the busy and read-only cases failed at `step="open sampling"` (`data/accounting-log-lines-*.txt`).
- **The "failing validation" mode was not exercised.** The corrupt-estimate run (`c7f`) produced no accounting failure and no gap warning (there are no `c7f` lines in the log file). The README's "in each failure mode" is therefore wrong for that mode.
- **The executor's own frozen expectation was not met.** `FROZEN-DESIGN.md` C7 says "Compaction completes and the turn finishes". In the 2-minute lock run (`c7c-t2`) and the read-only run (`c7d-t2`) the turn failed:
  - each shows a gap warning, then a "Heads up … compactions" item, then `turn.failed` in the throttle check;
  - the frozen design was dropped without disclosure.
- **Is the #351 attribution fair?** On the evidence, yes. The logs show accounting stepped aside ("request sent unrecorded step=resolve sampling") before the error, and the error names the provider-request throttle check.
- **Other gaps:**
  - Only local compaction over Chat Completions on Z.AI was tested. No remote/Responses compaction was run (no OpenAI credential).
  - `c7a-compact` ran on a new thread (`d060`, not the warm thread `c3e5`) and did not compact. For macOS, the insert-refusal evidence is `c7b-t3`.
- **Suggested verdict:** PARTIAL.
  - PASS for compaction under admission/open failures: insert refusal on both platforms, and the 40 s lock.
  - Turn-level FAIL (#351) for read-only and the ~2-minute lock.
  - NOT VERIFIED: validation or post-response failure, web search, image generation.

**H2. AC6: real ledgers read only partly, and this is not disclosed.**
- The real-ledger views report conversations that could not be read in full:

  | View | Conversations not read in full |
  | --- | --- |
  | `mac-c6-real-*-2026-09-23` | 1 |
  | `mac-c6-real-a63-2026-10-02` | 3 |
  | `mac-c6-real-live-2026-10-02` | 22 |
  | `mac-c6-real-a63-2026-10-03` | 34 |
  | `mac-c6-real-live-2026-10-03` | 44 |

- The injected corrupt estimate (`mac-c7f-corrupt-estimate-day.txt`) produces exactly this message. The executor must explain the cause or qualify the verdict.
- The executor's own pre-`5bae` ledger has no bound price records (`ledger-dumps-rtx.txt` `h-old-view`: `{}`; `/cost` shows "no price available"). It therefore never re-serializes a pre-`5bae` snapshot or estimate, which is the Major 3 defect. Major 3 coverage rests on the lane's fixture test.
- No real ledger from before 09-21 exists. AC6 asks for that absence to be recorded explicitly; the README instead implies "real ledgers" were the old case.

**H3. The top line and "For Travis" overclaim.**
- "The Blocker and Majors 2–5 are fixed on real runs on both platforms" is wrong for the Blocker: Kimi Code ran only as a 401 placeholder attempt.
- Majors 2, 3, 4 and 5 are only partly verified (see the table below).
- "Every criterion that could be run … passed" conflicts with H1.

### Medium

- **M1. AC8 / Major 5: the reviewer tested was the persisted trunk, not an ephemeral fork.**
  - The reviewer has its own resumable thread (`mac-c8-reviewer-thread-conv.txt`), so it is the trunk reviewer.
  - Major 5 named the ephemeral parallel forks (`fork_config.ephemeral = true`). That path was not shown.
  - On Linux, the `review:` label is evidenced only by the ledger dump, not by a Technical-details screen.
- **M2. AC3 invalid-value evidence is missing.** `captures/exec-json/mac/c3-bad*.jsonl` are 0 bytes, and the error text exists only in the README. Linux has no evidence of the invalid-value case at all.
- **M3. The independent channel is not archived.**
  - The raw SSE trace stderr is not in the packet. The guardian reviewer's usage (3,940/113 on macOS, 3,913/195 on Linux) rests only on derived output: it is not in the exec turn usage.
  - Also missing: the Z.AI pricing-page extract, and the `config.toml` files for the custom providers (`zaicoding`, `zaiproxy`, `bigmodel-*`, `moonshot`, `kimicode-custom`, `openai-apikey`) and for the Bedrock command `auth`. Without the configs, base URLs can be checked only from 401 error strings.
- **M4. Safety gaps.**
  - `data/key-scan.txt` covers only this directory. The `codex_api=trace` logs and TUI logs in scratch on both hosts were not scanned, and there is no record that they were deleted.
  - `env "KEY=$(…)" cmd` (in `execrun.sh` and `rtx-execrun.sh`) puts the value briefly in `env`'s argv. This is a small variant of the prescribed `KEY="$(…)" cmd` form and should be disclosed.
- **M5. AC12 and AC2 coverage gaps.**
  - The AC12 NOT VERIFIABLE list omits real requests on the Kimi Code route and the OpenAI API-key route.
  - The built-in `openai` with an API key never recorded an attempt: the WebSocket 401 at the handshake. AC2's openai evidence is a custom provider at `api.openai.com/v1` plus the lane's unit test. Say so.
- **M6. The `zai-anthropic` and coding-URL "subscription" result confirms the label only.**
  - The defaults table flags `zai-anthropic` as needing a real-provider check (AC10).
  - Nothing shows whether Z.AI debited the balance, or whether the vault account holds a GLM Coding Plan.
- **M7. AC11 "Done" is premature.**
  - It cites `REVIEW.md`, which does not exist yet.
  - AC11 requires the isolated execution gate, and that was not met (host self-discipline). Mark it "Pending review; isolation deviation".

### Low

- **L1. "Every verdict rests on the `/cost` screens" is false for several verdicts:**
  - AC9 rests on log lines;
  - AC7 rests on exec JSON;
  - the AC1 openai row rests on a unit test;
  - the Ollama and LM Studio rows rest on the ledger dump, because `/cost` labels both "gpt-oss".
- **L2. Display anomalies that should be listed against #352 (or filed):**
  - "gpt-oss" as the provider name for both local providers;
  - "OpenRouter · Ambient GLM 5.2" for `z-ai/glm-5.2`;
  - `zai-anthropic` shown as "1+ tokens" although 9,908 input tokens are known;
  - OpenRouter rows say "No money is stated" next to "Billed cost: $…", and are counted in "had no price".
- **L3. Smaller wording corrections:**
  - "three weeks of `claude-plan`" is really 09-22..10-03, and those plan rows are priced, so they don't bear on Major 2.
  - The hash timestamp is 12:15:40Z, but `FROZEN-DESIGN.md` says "~12:25Z".
  - AC6 says "$0.00302848, exact", but the capture shows only "$0.003028 (rounded)".
  - Per-AC platform coverage is not stated. AC2, the busy/read-only/corrupt cases of AC7, and the AC12 placeholder routes are macOS only; `claude-plan` is Linux only.
  - The DeepSeek pricing extract lacks the column header that identifies `deepseek-flash`.
  - The step that copied the real ledgers is not in `tools/`.

**Checked and sound:**
- Every run used `CORBANU_TEST_NO_NATIVE_KEYRING=1` and a disposable `CODEX_HOME`, `CORBANU_HOME` and `PFTERMINAL_HOME`.
- Vault helper calls sit on the consuming command; the Linux key goes over ssh stdin.
- Real ledgers were opened read-only (`mode=ro`).
- `claude-plan` used a fake helper, and the vault block was not bypassed.
- The overflow note appears on all 61 views exactly where subscription work is shown; I re-checked each one.
- AC9: one `accounting.excluded` warning per ephemeral run and none for the control; the ledger confirms (`h-c9`, `h-l-c9`).
- AC6 future format: one warning per session, nothing recorded, ledger format `[1, 99]`.
- AC1: 19 attempts on macOS and 20 on Linux, every basis matching the table (ledger dumps and `*-c1-day`). The "11 no price / 4 incomplete" count is exact.

### Body review findings

| Finding | Status | Basis |
| --- | --- | --- |
| Blocker 1 (Kimi Code counted as spend) | Partly verified | Placeholder 401 recorded as `PlanEquivalent` and shown as subscription on both platforms; no real tokens (AC5 NOT VERIFIABLE) |
| Major 2 (priceless plan work shown as "Pay per use") | Partly verified | Priceless subscription routes (`zai-anthropic`, `zaicoding`, the override, Kimi Code) show Subscription and are left out of the no-price count. The named ChatGPT, priority-tier, image/realtime and pane-bridge paths are not verified |
| Major 3 (older ledgers stop reading) | Partly verified | Fixture test and future-format gate pass; executor's own old ledger has no price records; real ledgers have unexplained unreadable conversations (H2); the reverse direction (older build on a newer ledger) was not tested |
| Major 4 (best effort) | Partly verified | Admission/open failures pass; post-response and validation failures not tested; turn fails in 2 modes (#351); web search and image NOT VERIFIABLE |
| Major 5 (uncollected sessions) | Partly verified | `exec --ephemeral` shown on both platforms; trunk guardian attributed; ephemeral fork not exercised |
| Minor 10 (command `auth` = subscription) | Verified (macOS) | Bedrock with command `auth` is pay per use (`h-c2`) |

## Recomputation

Prices used: Z.AI GLM-5.2 1.40 / 0.26 / 4.40 and DeepSeek flash off-peak 0.15 / 0.003 / 0.60 per 1M tokens, in Decimal.
- I re-priced all 183 priced rows in `recompute-*.txt` and re-summed every TOTAL line: 0 mismatches.
- I cross-checked each pay-per-use turn against the turn usage in `captures/exec-json/`, which is cumulative per thread.
- Token counts are input / cached / output.

| View | README | Mine | Source | Match |
| --- | --- | --- | --- | --- |
| macOS `zai` T1 | 0.01759704 | 12612/64/3 → 0.01759704 | exec-json; screen exact USD | ✓ |
| macOS T2 (override) | not spent | API equivalent 0.01708632; day $0.017597 | `c3-day` | ✓ |
| Linux T1 / T2 | 0.00307788 | 6431/5248/13 → 0.00307788 | | ✓ |
| `zaiproxy` | 9,898 / 3,715 tokens | 9883+15 per turn; 3702+13 | `proxy-*.jsonl` | ✓ |
| macOS C10 `zai` | 0.01170668; 0.00697936 | 0.01170668; 0.00352508 + 0.00345428 = 0.00697936 | | ✓ |
| macOS `deepseek` | 0.001753362; 0.000170376 | same | | ✓ |
| macOS `openrouter` | $0.001702; $0.008359 | stated 0.0017016; 0.00691306 + 0.00144554 = 0.0083586 | | ✓ (rounded display) |
| Linux mixed day | $0.005304 | 0.00498788 + 0.000315918 = 0.005303798 | | ✓ |
| Linux pay-only day | 0.00501388; 0.000233664; OR $0.001803 | same; day "at least" 0.005247544 = $0.005248 | | ✓ |
| Guardian, macOS | 0.0448102 | 0.0201958 + 0.0060132 + 0.0186012; parent turn 0.038797 + reviewer 0.0060132 | | ✓ |
| Guardian, Linux | 0.012911 | 0.00448888 + 0.0063362 + 0.00208592; parent turn 0.0065748 + reviewer 0.0063362 | | ✓ |
| Pre-`5bae` ledger | 25,781 tokens + new turn 0.00302848 | 25738 + 43 = 25,781; 6424/5248/4 → 0.00302848; day total 32,209 | | ✓ |
| Vercel `kimi-k3` | billed 0.037207; at least 0.00054 | 0.036567 + 0.00054 = 0.037107, + 0.0001 fee = 0.037207 | | ✓ |
| QA spend, Z.AI | ~$1.07 | 1.04597484 + proxy 0.03216888 = **1.0781** | | ✗ say ~$1.08 |
| QA spend, total / subscription tokens | ~$1.13; ~71k | 1.1345; 70,868 | | ✓ |

## Corrections the executor must make to README.md

1. **AC7:** change the verdict to PARTIAL as in H1. State that no post-response or validation failure was injected and that `c7f` produced no store failure. Disclose the change from the frozen "turn finishes" expectation. Cite `c7b-t3`, not `c7a`, for macOS insert refusal.
2. **AC6:** disclose the "N conversations could not be read in full" counts (1/3/22/34/44) and investigate them or caveat the verdict. State that your own pre-`5bae` ledger has no price records, so it does not exercise Major 3. Record that no real pre-09-21 ledger exists.
3. **Top line and "For Travis":** remove "fixed on real runs on both platforms". Add a per-finding status (the table above) and say the Blocker was verified only on a placeholder attempt.
4. **AC8:** note that only the persisted trunk reviewer was exercised, not an ephemeral fork. Add the Linux Technical-details screen or cite the ledger dump.
5. **AC3:** archive the invalid-value stderr (macOS developer and default builds, and Linux) or downgrade the claim. The `c3-bad*.jsonl` files are empty.
6. **AC11:** mark it "Pending review", with the isolation deviation stated against the AC wording. Fix the `REVIEW.md` line once the review exists.
7. **AC12 and AC2:** add Kimi Code and OpenAI API-key real requests to the NOT VERIFIABLE list. Say that AC2's openai evidence is a custom route plus a unit test, because the built-in `openai` recorded nothing.
8. **Evidence:** archive the SSE trace files (or redacted usage extracts), the Z.AI pricing extract, every per-home `config.toml`, and the step that copied the real ledgers.
9. **Safety:** scan or delete the scratch trace and TUI logs on both hosts and record the result. Disclose that the key value passes briefly through `env`'s argv.
10. **Remove "Every verdict rests on the `/cost` screens"** and name the evidence each verdict actually rests on (see L1).
11. **List the five #352 nits.** Add or file the anomalies in L2.
12. **Smaller fixes:**
    - Z.AI spend is $1.08, not $1.07.
    - Replace "three weeks" with 09-22..10-03.
    - AC6 shows "$0.003028 (rounded)", not "exact".
    - Reconcile the frozen-design time (12:15:40Z vs ~12:25Z).
    - Add a per-AC platform coverage note.
    - Add the DeepSeek pricing column header.
    - State whether the Z.AI account holds a Coding Plan, and whether the balance was checked for the `zai-anthropic` and coding requests (M6).
