# Task Node evidence record: SECACCT-TN-07

Compile the SECACCT-TN-07 PF-60-S05 Awaiting-Acceptance Evidence Record. Task `task_05f44e6ca71a4a02867e640f71ad72e5`, request `req_5631b641a410caadc6a1b6e56a4dd31f3e58cb4c5aafc6418eaa805de979c97c`. Every PR below is merged to `main` in CorbanuCore/CorbanuTerminal; each merge commit was checked with `git merge-base --is-ancestor <sha> origin/main` at `3254a302fd` (2026-10-09). Repository quotes are copied verbatim from the files named above them (relative links re-pointed to this file's location); PR-description quotes are verbatim from `gh pr view <n> --json body` on 2026-10-09. Task map: [tasknode-task-map.md](../../../qa/initiative-control/p1-security-and-accounting/tasknode-task-map.md).

**Status: shipped and independently accepted; awaiting Travis's acceptance. Not accepted, not archived.** All slices are merged, and the independent code-blind acceptance (#355) recommends accepting "for what can be checked today". The sprint record still reads `status: ready` and its Remaining list is unchanged since #346 (it predates #355). Verdicts: AC1, AC3, AC6, AC8, AC9 PASS; AC7 PARTIAL (two modes fail at turn level in the provider-request throttle check, outside accounting, #351); AC4 and AC5 NOT VERIFIABLE; AC2, AC10 and AC12 PASS for the observable parts, with the rest NOT VERIFIABLE. Credentials not in the vault: Kimi Code membership, ChatGPT login, a Claude plan login a built binary may use, OpenAI API key, Kimi Open Platform and BigModel. AC11 ran on the host under self-discipline, not inside an OS-enforced sandbox.

Records: [docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s05-collection-correctness-and-billing-basis.md](../../sprints/current/portfolio-agent-cost-accounting/pf-60-s05-collection-correctness-and-billing-basis.md), [qa/portfolio/agent-cost-accounting/pf-60-s05/independent-acceptance-20261009/README.md](../../../qa/portfolio/agent-cost-accounting/pf-60-s05/independent-acceptance-20261009/README.md), [qa/portfolio/agent-cost-accounting/pf-60-s05/independent-acceptance-20261009/REVIEW.md](../../../qa/portfolio/agent-cost-accounting/pf-60-s05/independent-acceptance-20261009/REVIEW.md), [qa/demos/index/PF-60-S05.md](../../../qa/demos/index/PF-60-S05.md).

| PR | Merge commit on main | Merged (UTC) | Title |
| --- | --- | --- | --- |
| [#330](https://github.com/CorbanuCore/CorbanuTerminal/pull/330) | `7ac8fcb6cb` | 2026-10-09T05:51:16Z | docs(PF-60-S05): collection-correctness sprint before S04 + 'both' options memo |
| [#334](https://github.com/CorbanuCore/CorbanuTerminal/pull/334) | `8ec1aee7ab` | 2026-10-09T06:52:44Z | docs(PF-60-S05): option B decided, AC12, sprint ready |
| [#337](https://github.com/CorbanuCore/CorbanuTerminal/pull/337) | `6b4b24829b` | 2026-10-09T08:42:54Z | accounting: versioned ledger format; pre-09-21 ledgers read again (PF-60-S05 slice 1) |
| [#338](https://github.com/CorbanuCore/CorbanuTerminal/pull/338) | `2ecf6fdbe8` | 2026-10-09T10:13:12Z | accounting: billing basis declared per route and credential (PF-60-S05 slice 2) |
| [#342](https://github.com/CorbanuCore/CorbanuTerminal/pull/342) | `08034c2946` | 2026-10-09T11:10:12Z | PF-60-S05 slice 3: /cost states each declared billing basis; overflow note (option B) |
| [#344](https://github.com/CorbanuCore/CorbanuTerminal/pull/344) | `38362eaf57` | 2026-10-09T11:58:40Z | PF-60-S05 slice 4: guardian reviews under the reviewed conversation; ephemeral sessions named; newer ledger off once; best-effort search and image |
| [#346](https://github.com/CorbanuCore/CorbanuTerminal/pull/346) | `d7846e29d5` | 2026-10-09T12:45:39Z | PF-60-S05: real-provider evidence, demo videos; sprint ready for code-blind acceptance |
| [#355](https://github.com/CorbanuCore/CorbanuTerminal/pull/355) | `3254a302fd` | 2026-10-09T15:02:09Z | PF-60-S05: independent code-blind functional acceptance (2026-10-09) |

Verification output (`gh pr view <n> --json state,baseRefName,mergeCommit,statusCheckRollup`, then `git merge-base --is-ancestor`; "checks" counts the PR's final check conclusions):

```text
#330 MERGED base=main merge=7ac8fcb6cb checks: SKIPPED=13 SUCCESS=21; git merge-base --is-ancestor 7ac8fcb6cb 3254a302fd -> exit 0
#334 MERGED base=main merge=8ec1aee7ab checks: SKIPPED=13 SUCCESS=21; git merge-base --is-ancestor 8ec1aee7ab 3254a302fd -> exit 0
#337 MERGED base=main merge=6b4b24829b checks: SKIPPED=10 SUCCESS=27; git merge-base --is-ancestor 6b4b24829b 3254a302fd -> exit 0
#338 MERGED base=main merge=2ecf6fdbe8 checks: SKIPPED=10 SUCCESS=30; git merge-base --is-ancestor 2ecf6fdbe8 3254a302fd -> exit 0
#342 MERGED base=main merge=08034c2946 checks: SKIPPED=10 SUCCESS=30; git merge-base --is-ancestor 08034c2946 3254a302fd -> exit 0
#344 MERGED base=main merge=38362eaf57 checks: SKIPPED=10 SUCCESS=30; git merge-base --is-ancestor 38362eaf57 3254a302fd -> exit 0
#346 MERGED base=main merge=d7846e29d5 checks: SKIPPED=13 SUCCESS=24; git merge-base --is-ancestor d7846e29d5 3254a302fd -> exit 0
#355 MERGED base=main merge=3254a302fd checks: SKIPPED=13 SUCCESS=21; git merge-base --is-ancestor 3254a302fd 3254a302fd -> exit 0
```

## Independent acceptance (#355)

Verbatim, `qa/portfolio/agent-cost-accounting/pf-60-s05/independent-acceptance-20261009/README.md` lines 1-17:

> # PF-60-S05 independent functional acceptance (2026-10-09)
>
> **Result.**
>
> - On real runs, the accounting changes behave as the sprint specifies on macOS and Linux. Every pay-per-use total I
>   report equals my recomputation from provider-reported usage to the micro-dollar.
> - Each finding of the body review is fixed where it could be observed, but several are only partly verifiable with the
>   credentials in the vault. See [Body-review findings](../../../qa/portfolio/agent-cost-accounting/pf-60-s05/independent-acceptance-20261009/README.md#body-review-findings).
> - One AC7 mode is a real gap: with a read-only state DB, or another writer holding it for about 2 minutes, compaction
>   finishes but the turn then fails. The failure is in the provider-request throttle check, not in accounting
>   ([#351]).
> - Credentials missing from the vault make some parts **NOT VERIFIABLE**: Kimi Code, ChatGPT login, a Claude plan login
>   a built binary may use, OpenAI API key, Kimi Open Platform and BigModel.
> - [#352] collects low-severity `/cost` display nits.
>
> Executor: an independent Codex worker, code-blind. One independent reviewer audited the first draft:
> [REVIEW.md](../../../qa/portfolio/agent-cost-accounting/pf-60-s05/independent-acceptance-20261009/REVIEW.md) (conclusion "Supported, with corrections"). The corrections are listed at the end.

Verbatim, `qa/portfolio/agent-cost-accounting/pf-60-s05/independent-acceptance-20261009/README.md` lines 19-52:

> ## Verdicts
>
> Platform: **M** = macOS, **L** = Linux.
>
> | AC | Verdict | Platforms | Evidence |
> | --- | --- | --- | --- |
> | AC1 declarations | **PASS** | M, L | **Observed with placeholder credentials:** 20 of 21 built-ins recorded an attempt on L and 19 on M (`claude-plan` was not run on M). Every basis matches the defaults table: Kimi Code, Z.AI Anthropic and Claude Plan are subscription; Ollama and LM Studio are local; the rest are pay per use (`captures/{mac,rtx}/*-c1-day.txt`, `data/ledger-dumps-*.txt`). Ollama and LM Studio are told apart only in the ledger dump, because `/cost` labels both "gpt-oss" ([#352]). **Built-in `openai`:** it never sends a request with a bad key (WebSocket 401 at the handshake), so its row rests on the unit test `every_built_in_provider_declares_its_default_basis` (74/74 pass, `data/unit-tests-rtx.log`). |
> | AC2 not from auth type | **PASS** (observable parts); **NOT VERIFIABLE**: ChatGPT login, mutation tests | M (Bedrock, routes); M+L (Kimi) | **Bedrock:** `amazon-bedrock` with a command `auth` is pay per use (`mac-c2-bedrock-cmdauth-day`; config in `data/configs/mac-h-c2.toml`). **Kimi:** `kimi-code` with `KIMI_API_KEY` is subscription. **OpenAI:** the built-in recorded nothing (see AC1). A custom provider at `api.openai.com/v1` with an API key is pay per use (`mac-c12-routes-day`), and `basis_does_not_follow_the_auth_type` passes. **Mutation tests:** they need source edits, which a code-blind executor can't make. |
> | AC3 override, invalid, undeclared | **PASS** | M, L | **Override:** with `model_providers.zai.billing = "subscription"`, T2 shows "(set in your config)" and is not spent. **History:** T1 keeps its estimate, $0.017597 on M (exact 0.01759704) and $0.003078 on L (exact 0.00307788). **Invalid value:** `billing = "free"` fails both M builds and L with `unknown variant … in model_providers.zai.billing` (`data/c3-invalid-billing-*.txt`). **Undeclared custom provider:** "Billing basis not declared … Next step: set model_providers.zaiproxy.billing = …"; tokens are recorded, and the work is counted as neither spent nor subscription. |
> | AC4 ChatGPT, priority, image, realtime, pane bridge | **NOT VERIFIABLE** | — | These paths need a ChatGPT login or Claude panes. **Substitute evidence (M, L):** subscription routes with no catalogue price (Z.AI coding plan, `zai-anthropic`, Kimi Code, a `billing = "subscription"` override) show "Covered by your subscription". They never show "Pay per use" or "no price", and are not in the "N attempts had no price" count: 11 = exactly the unpriced pay-per-use attempts in `mac-c1-day`. |
> | AC5 Kimi Code real | **NOT VERIFIABLE** | — | No membership key in the vault. A placeholder-key attempt records as "Covered by your subscription" (M, L). |
> | AC6 ledger format | **PASS**, with caveats below | M, L | **Tests:** `ledger_written_before_plan_basis_validates_and_reads` and `newer_ledger_format_is_refused_by_name` pass. **Pre-`5bae` ledger (L):** my own ledger, written by a binary built at `3b6c338f8f`, reads in the new build: 4 requests and 25,781 tokens, equal to the provider-reported usage. A new turn is recorded: "$0.003028 (rounded)", recomputed 0.00302848. The ledger stays in format 1. **Real ledgers:** 09-22..10-09 copies read on 09-23, 10-02 and 10-03 (M, L). **Future format:** one warning per session, turns succeed and nothing new is recorded (M, L). |
> | AC7 best effort | **PARTIAL**: PASS as written for the modes run; turn-level **FAIL** in two modes, caused outside accounting ([#351]); **NOT VERIFIABLE**: validation failure, post-response failure, web search, image generation | M, L (refusal); M (others) | **Passing modes** (auto-compaction finished, the turn finished, one gap warning): new attempts refused by the store, M `c7b-t3` and L `l-c7-t2`; the store locked for 40 s, past accounting's ~16 s budget, M `c7e-t2`. **Failing modes** (compaction finished and one gap warning appeared, then the turn failed in the provider-request throttle check, `M c7c-t2`, `c7d-t2`): a read-only DB, and a lock held for about 2 minutes. My frozen expectation was "the turn finishes", so I record these as turn-level failures. **Not injected:** every failure I injected hit before the request was sent (`step="admit attempt"` or `"open sampling"`). A failure after the paid response (the review's Major 4 case) was not injected. My corrupted-estimate run (`c7f`) caused no write failure: validation is not on the write path within the hour, and `/cost` flagged that conversation as unreadable. Web search and image generation need OpenAI credentials. |
> | AC8 guardian | **PASS** | M, L | **Single review:** with `approvals_reviewer = "auto_review"` and an escalated `curl`, the reviewer's request is under the parent conversation and Technical details shows turn `review:…` (M; L from the ledger dump). The conversation equals the provider-reported total: 0.0448102 on M and 0.012911 on L. **Parallel reviews (M `c8b`):** 3 reviews, 2 of them concurrent, left only 1 reviewer rollout file, so at least one ran in an unpersisted fork. All 3 are under the parent thread as `review:` turns, and the conversation is 6 requests, exact 0.03890804 = provider-reported. Nothing is counted twice: the reviewer's own thread records nothing. |
> | AC9 ephemeral | **PASS** | M, L | `exec --ephemeral` (2 requests) logs exactly 1 `accounting.excluded="ephemeral_session"` and records nothing. The control logs 0 and records 2. The 2026-10-09 coverage audit lists `exec --ephemeral`. |
> | AC10 real providers | **PASS** (pay per use); **NOT VERIFIABLE**: `claude-plan`, ChatGPT, `kimi-code` | M, L | **Exact to the micro-dollar:** `zai` GLM 5.2 (M: 0.01170668, 0.00697936; L: 0.00307788, 0.00498788, 0.00501388) and `deepseek` off-peak (M: 0.001753362, 0.000170376; L: 0.000315918, 0.000233664). **OpenRouter:** "Billed by OpenRouter" equals the provider's `usage.cost` (0.0017016, 0.0083586; 0.00180282). **Others:** `ambient` and `vercel` are pay per use (M). |
> | AC11 independent acceptance | **Done, with an isolation deviation** | — | The design was frozen at 12:15:40Z, before any S05 result was read (`FROZEN-DESIGN.md`, `data/frozen-design-sha256.txt`). One code-blind Opus 5.5 High review: [REVIEW.md](../../../qa/portfolio/agent-cost-accounting/pf-60-s05/independent-acceptance-20261009/REVIEW.md). AC11 asks for the [isolated execution gate](../../../qa/code-blind-functional/isolated-execution.md); I ran on the host under self-discipline, as S03's executor did, not inside an OS-enforced sandbox. |
> | AC12 option B routes | **PASS** (Z.AI routes, unknown route, overflow note); **NOT VERIFIABLE**: real requests on Kimi Code, Kimi Open Platform, BigModel coding and general, OpenAI API key, ChatGPT | M, L (Z.AI); M (placeholder routes) | **Real requests:** the Z.AI general URL is pay per use; the coding URL (custom provider) and `zai-anthropic` are subscription. A local proxy route is "not declared". **Overflow note:** across all 65 captured S05-build views, it appears exactly on the views that contain subscription work; no pay-per-use-only view shows it. The 3 S03-build comparison views predate the note. **Placeholder keys:** the declared basis is recorded for `api.moonshot.ai` and BigModel general (pay per use), BigModel coding and a Kimi Code custom route (subscription), and `api.openai.com/v1` with an API key (pay per use). **Z.AI plan state:** I can't see whether the vault account holds a GLM Coding Plan or whether its balance was debited (no console access). The coding URL and the Anthropic route served every request with this key. |
>
> ### AC6 caveats
>
> - **Partial reads on real ledgers.** Some real-ledger day views say "N conversations could not be read in full; their
>   cost is unknown and not included". N is 1, 3 and 34 on 09-23, 10-02 and 10-03 for the `acct63` copy (and 1, 22 and
>   44 for the runtime-home copy).
>   - The S03-accepted build `63ea3d0cbd` gives **identical** output on the same copy: the same counts, totals and lines
>     (`captures/rtx/rtx-c6-a63-{s03,acct}-build-*.txt`). So this is not an S05 regression.
>   - On 09-23, the conversation not read in full is the one with 1,010 attempts that day. I infer a read-size limit;
>     the cause is not visible code-blind.
> - **What my pre-`5bae` ledger covers.** It holds 4 unpriced attempts and 8 pre-`5bae` estimate payloads, but no price
>   snapshot (the old build wrote none for these providers). The pre-`5bae` *price-record* path of Major 3 therefore rests
>   on the lane's committed fixture test.
> - **No real ledger from before 2026-09-21 exists** on either host (`data/real-ledger-search.txt`); the absence is
>   recorded here.
> - **Not tested:** an older build opening a newer ledger.

Verbatim, `qa/portfolio/agent-cost-accounting/pf-60-s05/independent-acceptance-20261009/README.md` lines 54-63:

> ## Body-review findings
>
> | Finding | Status | Basis |
> | --- | --- | --- |
> | Blocker 1 (Kimi Code counted as spend) | **Partly verified** | Placeholder attempts record as subscription on both platforms. No real Kimi tokens (AC5 NOT VERIFIABLE). |
> | Major 2 (priceless plan work shown as "Pay per use") | **Partly verified** | Priceless subscription routes (Z.AI coding, `zai-anthropic`, a `billing` override, Kimi Code) show subscription and are not in the no-price count. The named ChatGPT, priority-tier, image, realtime and pane-bridge paths: NOT VERIFIABLE. |
> | Major 3 (older ledgers stop reading) | **Verified for estimates and real 09-22+ ledgers; price-record path by the fixture test** | See the AC6 caveats. |
> | Major 4 (best effort) | **Partly verified** | Admission and open failures: compaction returns. Post-response and validation failures: not injected. Web search and image: NOT VERIFIABLE. Turn-level failure in two modes outside accounting ([#351]). |
> | Major 5 (uncollected sessions) | **Verified** | `exec --ephemeral` gives one warning. Guardian reviews, including parallel ones without a persisted thread, are recorded under the parent conversation. |
> | Minor 10 (command `auth` treated as subscription) | **Verified** (macOS) | Bedrock with a command `auth` is pay per use. |

Verbatim, `qa/portfolio/agent-cost-accounting/pf-60-s05/independent-acceptance-20261009/README.md` lines 183-198:

> ## For Travis
>
> - **Recommendation:** accept PF-60-S05 for what can be checked today.
>   - Subscription work is never counted as spending.
>   - Undeclared routes say so instead of guessing.
>   - Old and real ledgers still read.
>   - Compaction, guardian reviews and ephemeral sessions behave as specified.
>   - No total disagrees with an independent recomputation.
> - **Open items:**
>   - [#351] is a small follow-up outside accounting, so that a busy or read-only state DB doesn't end the turn.
>   - [#352] is cosmetic.
> - **To close the NOT VERIFIABLE parts, provide:**
>   - a Kimi Code membership key (AC5, and Kimi in AC10 and AC12);
>   - a ChatGPT login (AC4, AC10, AC12);
>   - an OpenAI API key (web search and image in AC7, and the API-key route in AC12);
>   - a Claude plan token that a disposable-home build may use (real `claude-plan` in AC10).

Verbatim, `qa/portfolio/agent-cost-accounting/pf-60-s05/independent-acceptance-20261009/REVIEW.md` lines 1-7:

> # Independent evidence review of the PF-60-S05 acceptance record
>
> Reviewer: separate code-blind session, installed `corbanu exec -m claude-opus-5-5-plan -c model_provider=claude-plan -c model_reasoning_effort=high -s read-only`, run 2026-10-09 in a packet holding only a copy of this directory (first draft) plus the sprint record, defaults table, options memo, coverage audit, ledger-format check and the S03 body review. It was told not to read source. The prompt is in `data/review-prompt.md`. Report below is verbatim apart from path redaction; the executor's responses are in README.md, "Review corrections applied".
>
> # PF-60-S05 code-blind evidence review
>
> **Conclusion: Supported, with corrections.** I recomputed every headline number and none disagrees with the record. Three verdicts overstate what the evidence shows: AC6, AC7 and AC11. So do the README's top-line and "For Travis" summaries.

## Sprint record

Verbatim, `docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s05-collection-correctness-and-billing-basis.md` lines 63-70:

> ## Done
>
> - [x] 2026-10-08 Sprint specified from review findings 1-5 and 10, Travis's requirements, the cited defaults table and the "both" options memo; 2026-10-09 option B recorded, AC12 written, worktree allocated (PR #334).
> - [x] 2026-10-09 **Ledger format (AC6), PR #337:** Major 3 reproduced with a ledger written at `5bae03414e^` (committed fixture); both stored forms read again; explicit format with upgrade-on-write and `NewerLedgerFormat` refusal; two real 09-23..10-03 ledgers validate ([check](../../../qa/portfolio/agent-cost-accounting/pf-60-s05/ledger-format-check.md)).
> - [x] 2026-10-09 **Declaration (AC1-3, AC12 table), PR #338:** `model-provider-info/src/billing.rs` declares a basis per built-in provider and credential plus route rows; `billing` config key (alias ids canonicalised, the real id wins); inference from auth mode and `provider.auth` removed; unknown route or credential is "not declared".
> - [x] 2026-10-09 **Basis on every attempt and `/cost` (AC3-4, AC12), PR #342:** every admitted attempt binds its basis and source (a record with no rates when there is no price); subscription, local and undeclared work are never pay per use, "no price" or spend; "Billing basis not declared" with its next step; the overflow note wherever subscription work appears.
> - [x] 2026-10-09 **Sessions and best effort (AC6-9), PR #344:** guardian reviewers (trunk and forks) record in the reviewed conversation's ledger as `review:` turns; a reviewer of an unpersisted conversation and any other ephemeral session log one `accounting.excluded`; a newer-format ledger turns collection off with one warning; compaction, web search and image generation return the provider's answer when the store refuses; [coverage audit](../../../qa/portfolio/agent-cost-accounting/pf-60-s05/acct-coverage-audit-20261009.md).
> - [x] 2026-10-09 **Lane gate:** tests with/without `developer-accounting` (core accounting/billing/guardian 321/316, state 398, tui tokens/cost 124/123, model-provider-info 74; Linux nextest core 326/321, tui 124); Linux clippy `-D warnings` 0/0 per slice; Opus 5.5 High review rounds per slice, all APPROVE; tmux run on GLM 5.2 with injected store failures and [real-provider checks](../../../qa/portfolio/agent-cost-accounting/pf-60-s05/real-provider-checks-20261009.md) (AC8, AC9, zai/deepseek recomputes exact); [demo videos](../../../qa/demos/index/PF-60-S05.md).

Verbatim, `docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s05-collection-correctness-and-billing-basis.md` lines 72-76:

> ## Remaining
>
> - [ ] Code-blind functional design and acceptance (AC11), then Travis.
> - [ ] **NOT VERIFIABLE (credential not in the vault, listed for Travis):** Kimi Code membership (AC5, AC10, AC12), Claude plan login (AC10, AC4 pane bridge), ChatGPT login (AC4, AC10, AC12, realtime row), OpenAI and Anthropic API keys, BigModel keys.
> - [ ] Follow-ups: `/side` conversations and memory consolidation stay uncollected (named in the log and the audit; proposed for S04); a user-declared `pay_per_use` route upgrades the ledger to format 2, which older developer builds won't write; "Price source" lines on basis-only records.

## Demo videos

6 assets, all HTTP 200:

| Demo | Commit | Length | Asset | HTTP |
| --- | --- | --- | --- | --- |
| `pf60s05-subscription-not-spent` | `00f02f78ed57` | 17s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s05-pf60s05-subscription-not-spent-00f02f78ed57-2026-10-09.mp4 | 200 |
| `pf60s05-subscription-not-spent` | `00f02f78ed57` | 17s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s05-pf60s05-subscription-not-spent-00f02f78ed57-2026-10-09.cast | 200 |
| `pf60s05-basis-not-declared` | `00f02f78ed57` | 17s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s05-pf60s05-basis-not-declared-00f02f78ed57-2026-10-09.mp4 | 200 |
| `pf60s05-basis-not-declared` | `00f02f78ed57` | 17s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s05-pf60s05-basis-not-declared-00f02f78ed57-2026-10-09.cast | 200 |
| `pf60s05-overflow-note` | `00f02f78ed57` | 21s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s05-pf60s05-overflow-note-00f02f78ed57-2026-10-09.mp4 | 200 |
| `pf60s05-overflow-note` | `00f02f78ed57` | 21s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s05-pf60s05-overflow-note-00f02f78ed57-2026-10-09.cast | 200 |

## Open, not done

| Issue | State | Closed (UTC) | Title |
| --- | --- | --- | --- |
| [#351](https://github.com/CorbanuCore/CorbanuTerminal/issues/351) | OPEN | — | Turn fails on 'provider request throttle state' when the state DB is busy (>~1 min) or read-only, after accounting has already stepped aside |
| [#352](https://github.com/CorbanuCore/CorbanuTerminal/issues/352) | OPEN | — | PF-60-S05 acceptance: /cost display nits (local providers shown as gpt-oss, OpenRouter GLM shown as Ambient, Anthropic-wire '1+ tokens', wrong 'no API price' reason, basis source) |

- Travis's acceptance; archiving the record; the sprint record's Verification and Exit evidence items.
- NOT VERIFIABLE parts (credentials above); mutation tests for AC2 (need source edits).
- Follow-ups in Remaining: `/side` conversations and memory consolidation uncollected; a user-declared `pay_per_use` route upgrades the ledger to format 2; "Price source" lines on basis-only records.
