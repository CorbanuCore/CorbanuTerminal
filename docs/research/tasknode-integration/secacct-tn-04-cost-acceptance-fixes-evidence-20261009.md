# Task Node evidence record: SECACCT-TN-04

Compile the SECACCT-TN-04 Eight-Fix Cost Accounting Evidence Record. Task `task_7e2280a41804bcd4f1e98af473c78937`, request `req_2437417115760b6676ad84e648299ad76a75e2731e97c5407a8a4e8fc27bcb9f`. Every PR below is merged to `main` in CorbanuCore/CorbanuTerminal; each merge commit was checked with `git merge-base --is-ancestor <sha> origin/main` at `3254a302fd` (2026-10-09). Repository quotes are copied verbatim from the files named above them (relative links re-pointed to this file's location); PR-description quotes are verbatim from `gh pr view <n> --json body` on 2026-10-09. Task map: [tasknode-task-map.md](../../../qa/initiative-control/p1-security-and-accounting/tasknode-task-map.md).

**Status: all eight fixes merged; #286, #287, #288, #289 and #308 are closed.** `/cost` is developer-only (`developer-accounting` feature, debug builds). The acceptance runs that found and confirmed these fixes are a separate task (SECACCT-TN-05). #339 landed after PF-60-S03 was archived; it is presentation-only and was checked by its own review and TUI captures, not by an acceptance re-run. **Open, not claimed:** #325 and the #287 WARN wording residual.

Records: [docs/sprints/archive/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md](../../sprints/archive/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md), acceptance runs under [qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/](../../../qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/).

| PR | Merge commit on main | Merged (UTC) | Title |
| --- | --- | --- | --- |
| [#291](https://github.com/CorbanuCore/CorbanuTerminal/pull/291) | `c76e6d0845` | 2026-10-08T12:45:17Z | tui: /cost tells missing usage from a missing price; next steps for every unavailable state (PF-60-S03) |
| [#299](https://github.com/CorbanuCore/CorbanuTerminal/pull/299) | `1f7d8f2eec` | 2026-10-08T15:14:31Z | accounting: never block a model request; tolerate a clock behind the ledger (#287) |
| [#303](https://github.com/CorbanuCore/CorbanuTerminal/pull/303) | `a73d40e0af` | 2026-10-08T15:00:56Z | accounting: /cost discloses deleted conversations' spend instead of an empty day (#286) |
| [#305](https://github.com/CorbanuCore/CorbanuTerminal/pull/305) | `f0d71e62b6` | 2026-10-08T16:30:51Z | tui: /cost next step for every request without a complete estimate (#288) |
| [#306](https://github.com/CorbanuCore/CorbanuTerminal/pull/306) | `b294864916` | 2026-10-08T15:56:15Z | accounting: /cost wording fixes from the S03 acceptance run (#289) |
| [#318](https://github.com/CorbanuCore/CorbanuTerminal/pull/318) | `290bb28532` | 2026-10-08T21:18:30Z | PF-60-S03: fix #289 residuals (default-build /usage requests, in-progress buckets, retention floor) |
| [#322](https://github.com/CorbanuCore/CorbanuTerminal/pull/322) | `ccc38bfc03` | 2026-10-08T22:34:10Z | accounting: delete tolerates a clock behind the ledger and never half-deletes (#308) |
| [#339](https://github.com/CorbanuCore/CorbanuTerminal/pull/339) | `c261d7b2e5` | 2026-10-09T10:05:02Z | tui(cost): date-format hint for /cost and plain billed-cost wording |

Verification output (`gh pr view <n> --json state,baseRefName,mergeCommit,statusCheckRollup`, then `git merge-base --is-ancestor`; "checks" counts the PR's final check conclusions):

```text
#291 MERGED base=main merge=c76e6d0845 checks: SKIPPED=10 SUCCESS=25; git merge-base --is-ancestor c76e6d0845 3254a302fd -> exit 0
#299 MERGED base=main merge=1f7d8f2eec checks: SKIPPED=10 SUCCESS=30; git merge-base --is-ancestor 1f7d8f2eec 3254a302fd -> exit 0
#303 MERGED base=main merge=a73d40e0af checks: SKIPPED=10 SUCCESS=24; git merge-base --is-ancestor a73d40e0af 3254a302fd -> exit 0
#305 MERGED base=main merge=f0d71e62b6 checks: SKIPPED=10 SUCCESS=27; git merge-base --is-ancestor f0d71e62b6 3254a302fd -> exit 0
#306 MERGED base=main merge=b294864916 checks: SKIPPED=10 SUCCESS=24; git merge-base --is-ancestor b294864916 3254a302fd -> exit 0
#318 MERGED base=main merge=290bb28532 checks: SKIPPED=10 SUCCESS=25; git merge-base --is-ancestor 290bb28532 3254a302fd -> exit 0
#322 MERGED base=main merge=ccc38bfc03 checks: SKIPPED=10 SUCCESS=31; git merge-base --is-ancestor ccc38bfc03 3254a302fd -> exit 0
#339 MERGED base=main merge=c261d7b2e5 checks: SKIPPED=10 SUCCESS=24; git merge-base --is-ancestor c261d7b2e5 3254a302fd -> exit 0
```

Issues fixed:

| Issue | State | Closed (UTC) | Title |
| --- | --- | --- | --- |
| [#286](https://github.com/CorbanuCore/CorbanuTerminal/issues/286) | CLOSED | 2026-10-08T17:48:05Z | PF-60-S03: deleting a session silently erases its spend; /cost then claims no other requests that day |
| [#287](https://github.com/CorbanuCore/CorbanuTerminal/issues/287) | CLOSED | 2026-10-08T17:48:08Z | Developer accounting: clock behind ledger checkpoint fails every model request ("Native Anthropic accounting failed" on Z.AI) |
| [#288](https://github.com/CorbanuCore/CorbanuTerminal/issues/288) | CLOSED | 2026-10-08T16:33:44Z | PF-60-S03: requests with no reported usage are labelled 'no price available' and never get a next step |
| [#289](https://github.com/CorbanuCore/CorbanuTerminal/issues/289) | CLOSED | 2026-10-08T21:18:32Z | PF-60-S03: /cost wording inconsistencies (fresh-home 'collection remains off', unknown components beside exact costs, future buckets 'unavailable') |
| [#308](https://github.com/CorbanuCore/CorbanuTerminal/issues/308) | CLOSED | 2026-10-08T22:34:11Z | Developer accounting: delete with clock behind ledger checkpoint fails after removing the session (non-atomic) |

| PR | Fixes | Gate quoted below | Confirmed by |
| --- | --- | --- | --- |
| #299 | #287 (clock behind the ledger fails every request) | PR description | re-run 1 (Linux PASS for turns), re-run 3 regression 15 PASS |
| #303 | #286 (deleted conversations' spend) | PR description | re-run 1 criterion 9 PASS both platforms, re-run 3 regression 9 PASS |
| #291 | #288 (missing usage vs missing price; next steps) | PR description; archived record Done | re-run 2: 8a PASS (with #305); 8b, the label distinction, NOT VERIFIABLE on that setup (no contradiction) |
| #305 | #288 (next step for every request without a complete estimate) | PR description | re-run 2 8a PASS (re-run 1 predates it: FAIL), re-run 3 regression 8a PASS |
| #306 | #289 (wording) | PR description | re-run 1 "#289 observations: mostly fixed" |
| #318 | #289 residuals R1, R1b, R2, R3, criterion 13b | PR description; archived record Done | re-run 3 items 1 and 2 PASS |
| #322 | #308 (delete with the clock behind; half-deletes) | PR description | re-run 3 item 3 PASS |
| #339 | Travis's 2026-10-09 request (date hint, plain billed wording) | PR description | its own review and TUI captures in `pf-60-s03/cost-hint-wording-20261009/` |

## Re-run verdict tables that confirm the fixes

Verbatim, `qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/rerun-20261008/README.md` lines 1-17:

> # PF-60-S03 independent acceptance: targeted re-run (2026-10-08)
>
> This re-run checks the three criteria that failed in the [first independent run](../../../qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/README.md), plus the #289
> observations. The fixes merged since then are PR #299 (#287), PR #303 (#286) and PR #306 (#289).
>
> | Re-run item | macOS | Linux | Verdict |
> | --- | --- | --- | --- |
> | Criterion 15 / #287: turns with the clock behind the ledger checkpoint | **NOT VERIFIABLE**: blocked at `thread/start`; cause not isolated, most likely libfaketime on macOS | **PASS** for turns | **PASS (Linux only)**. Residual: >90-day unrecorded spend is not disclosed in `/cost` (commented on #287). New defect in `corbanu delete`: #308 |
> | Criterion 9 / #286: deleted history | **PASS** (clock faked *ahead*; delete via a retry after a #308 partial delete) | **PASS** (clean delete) | **PASS** |
> | Criterion 8 / #288: no-usage requests | **FAIL** | not run (same as the first run) | **FAIL**, unchanged |
> | #289 observations (not a criterion) | mostly fixed | mostly fixed | residual wording points, plus a criterion-13 question for the default build, listed below |
>
> Every displayed number that I checked matched an independent recomputation from provider-reported usage, to the
> micro-dollar.
>
> Executor: an independent Codex worker, code-blind. One separate reviewer audited this record: installed `corbanu exec`, `claude-plan`,
> `claude-opus-5-5-plan`, `high` effort, `-s read-only`, code-blind, on a copy of this directory. Its report is [REVIEW.md](../../../qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/rerun-20261008/REVIEW.md); the corrections it led to are listed at the end.

Verbatim, `qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/rerun2-20261008/README.md` lines 1-21:

> # PF-60-S03 independent acceptance: second targeted re-run (2026-10-08)
>
> This re-run covers criterion 8 / #288 and the #289 residuals listed in the [first re-run](../../../qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/rerun-20261008/README.md),
> including its criterion-13 question. The first re-run built `b294864916`, which did not include PR #305, the fix for
> #288. This one builds current `origin/main`, which does.
>
> | Item | Verdict | One-line evidence |
> | --- | --- | --- |
> | 8a. The no-usage conversation's own pages give a next step that names its own provider | **PASS** | Its `/cost`, provider, request and technical pages all say "check the bill from **zainousage**" (`mac-c8-01`…`05`). Other conversations name it too (`mac-c8-10`, `mac-c8-13`) |
> | 8b. Missing usage told apart from a missing price | **NOT VERIFIABLE** (no contradiction on this setup) | The "usage incomplete" label is for a priced request, and the built-in priced provider cannot be pointed at the proxy (`data/repoint-probe-error.txt`). On the custom provider, the rows show both facts: "tokens not reported" (`zainousage`) vs "20,304 tokens" (`zaiproxy`), plus "no price available", which is true here. The cost reason itself does not mention the missing usage |
> | 8c. Request priced only in part | **NOT VERIFIABLE** | Same reason. The closest case is partial usage on an unpriced custom provider (`zaipartial`, `completion_tokens` removed): next step "check the bill from zaipartial" and "Output: not reported (2 attempts)" (`mac-c8-10`, `mac-c8-11`) |
> | #289 R1. Default build, `/usage requests <past day>`: the next step reads "if this build records costs, send a turn…" | **FAIL** (unchanged) | `mac-289-01` was taken in a home where the default build had already completed a real 2-request turn (`exec-json/mac-def-zai.jsonl`), and it still says "accounting ledger not installed". So the next step leads nowhere |
> | #289 R1b. Default build, `/usage requests <future day>`: "Run /usage requests for today" | **FAIL** (unchanged) | `mac-289-02`; today's page is the same "not installed" page (`mac-289-08`) |
> | #289 R2. A week or month bucket reaching today is titled "Bucket unavailable", and its next step cannot complete it | **FAIL** (unchanged) | `mac-289-05`, `mac-289-07`: title "Bucket unavailable", next step "send a turn … then select Refresh", for buckets that end on 10-12 and 11-01 |
> | #289 R3. The range header says "oldest daily total kept none"; day pages in the same home say "daily totals kept since 2025-10-09" | **FAIL** (unchanged) | `mac-289-04`, `mac-289-06` vs `mac-c8-01` |
> | Criterion 13a. `/cost` is absent from a default build | **PASS** (macOS; Linux passed in the first run) | `mac-289-03`: "Unrecognized command '/cost'" |
> | Criterion 13b. Should a default build open the "Cost — this conversation" page for `/usage requests`? | **NOT VERIFIABLE**: needs a product decision; behaviour unchanged | `mac-289-01`, `mac-289-08` |
>
> Every number I report below matches my own recomputation from provider-reported usage. That usage comes from the
> product's own `codex_api=trace` log for `nu-zai` and from the outside-the-product proxy logs for the three proxy routes.
>

Verbatim, `qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/rerun3-20261008/README.md` lines 1-27:

> # PF-60-S03 independent acceptance: third targeted re-run (2026-10-08)
>
> This is the final confirming re-run before Travis decides on PF-60-S03. It checks the fixes merged since the
> [second re-run](../../../qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/rerun2-20261008/README.md):
>
> - PR #318, which fixes the #289 residuals R1, R1b, R2 and R3, plus criterion 13b
> - PR #322, which fixes #308 (delete with the clock behind the ledger checkpoint)
>
> It also repeats one case each of the criteria that passed before: 8a, 9 and 15.
>
> | Item | Verdict | One-line evidence |
> | --- | --- | --- |
> | 1. #289 R1, R1b and 13b (default build) | **PASS** | `/usage requests`, `… 2026-10-07` and `… 2026-10-09` each print one line, "Per-request cost history is not part of this build.", with no page and no next step (`mac-289-r1-01`…`03`). This home was signed out, so `/usage` shows "Sign in with ChatGPT to view OpenAI account usage." (`mac-289-r1-04`). Two sub-items are **NOT VERIFIABLE** in a signed-out home: the signed-in `/usage` menu and #318's "usage error" text |
> | 1. Criterion 13a (default build): `/cost` | **PASS** | "Unrecognized command '/cost'" (`mac-289-13a`) |
> | 2. #289 R2: week and month buckets that reach today, and the week after today | **PASS** | The bucket reaching today is titled "Cost so far — this conversation" and says "In progress — totals so far, recorded through 22:45:11.964Z". Its $0.015922 counts in the range total, and it has no next step. The week after reads "Not started yet — it starts after this view was read." (`mac-289-r2-01`…`06`) |
> | 2. #289 R3: retention floor wording | **PASS** | Both the range header and the day page say "daily totals kept since 2025-10-09" (`mac-289-r2-01`, `-03`, `-05` vs `mac-c9-02`) |
> | 3. #308: `delete --force` with the clock about 57 s and 6 days behind the checkpoint, then the "deleted conversations" disclosure | **PASS** (succeeded, state consistent) | Both deletes exit 0 with "Deleted session …" and no error, and the rollout is gone (`rtx-308-del58s`, `rtx-308-del6d`). `resume` says "No saved session found" (`rtx-308-03`). Today's page leaves both out and says "Deleted conversations or subagents sent 4 request attempts on this day" (2 + 2) (`rtx-308-01`). 2026-10-02 shows nothing (`rtx-308-02`) |
> | 4. Regression: 8a (a no-usage conversation names its own provider) | **PASS** | The `zainousage` conversation's `/cost` and Request 1 page both say "check the bill from **zainousage**" (`mac-c8-01`, `mac-c8-02`) |
> | 4. Regression: 9 (deleted history) | **PASS** | A clean single delete at the real clock on macOS, root plus subagent. The viewer's day page says "Deleted conversations or subagents sent **6** request attempts on this day". `d1` made 6 requests (`mac-c9-01`, `mac-c9-02`) |
> | 4. Regression: 15 (turn with the clock behind the checkpoint) | **PASS** | `sk58s` ran 58 s behind real time, which was 49 s behind the checkpoint (`behind_ms=48950`). It reached `turn.completed`. All 7 requests were recorded: 50,427 tokens, exact USD 0.01776348, admission time 22:39:09.255Z on the faked clock (`rtx-c15-01`…`03`). Still reproduces open #287 residual 2 (see §4) |
>
> Every displayed total that I checked matches my own recomputation from provider-reported usage at Z.AI's published
> prices, to the micro-dollar ([data/recompute-mac.txt](../../../qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/rerun3-20261008/data/recompute-mac.txt),
> [data/recompute-rtx.txt](../../../qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/rerun3-20261008/data/recompute-rtx.txt), and the `-exact.txt` files next to them). **No new defect was found**; one known residual of closed #287 (residual 2) still reproduces (§4).
>
> Executor: an independent Codex worker, code-blind. One separate code-blind reviewer audited this record: installed
> `corbanu exec`, `claude-plan`, `claude-opus-5-5-plan`, `high` effort, `-s read-only`. Its report is

## Fixes as received (archived sprint record)

Verbatim, `docs/sprints/archive/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md` lines 36-36:

> - Fixes received: #299 (#287), #303 (#286), #291 and #305 (#288), #306 and #318 (#289), #322 (#308).

Verbatim, `docs/sprints/archive/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md` lines 103-103:

> - [x] **2026-10-08 remaining S03 code gaps closed, PR #291** (`7405aefde5`, `2fb21ad640`, `5ba0c02f67`; TUI only). "usage incomplete" (a price, a missing token count) is told apart from "no price available" on every label, mixed causes counted; every unavailable state (ledger absent, thread gone, lag, refresh, too large, expired detail, remote, no thread, switched, DB closed, unreadable, timeout) names a next step in the command this build resolves (`/cost` only with `developer-accounting`, shipped text unchanged); `/cost` errors name `/cost`; a subscription-only day no longer points at a bill. Closes the missing-usage label left open by acct-scope-62; PR #305 then gives every pay-per-use request without a complete estimate a next step naming its own provider (no price, priced only in part, incomplete usage; #288), except requests whose every reported count is zero, which cost nothing. Left open from its review: a price sheet that sets cache writes to zero but omits another rate can make a partly priced request read as no-price, because the TUI reads raw usage rather than the priced counts. Regression tests for no-usage, stale estimate, current command, next steps and the footer: 207/207 default, 208/208 with the feature. Linux clippy `-D warnings` on the RTX, default and feature, exit 0. Two independent Opus 5.5 High rounds: REQUEST CHANGES (9 findings, all fixed), then APPROVE WITH NITS (4 of 5 fixed). **Demo videos** (GLM 5.2 on zai, disposable home, developer-accounting debug build of `5ba0c02f67`, [index](../../../qa/demos/index/PF-60-S03.md)): `/cost` drill-down with Back/Esc and an empty conversation with the missing-price step; a past day and ranges by day and week on seeded history; three concurrent `corbanu exec` runs exiting 0 with no accounting failure, then `/cost` on one. Seeding is off screen via `codex-rs/state/examples/accounting_demo_seed.rs` and the demo script's new `run`/`capture` steps.

Verbatim, `docs/sprints/archive/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md` lines 104-104:

> - [x] **2026-10-08 #289 residuals from the second re-run (PR #315) fixed, PR #318.** R1/R1b and criterion 13b, decided here and open to Travis: `/cost` is developer-only and the 2026-09-16 answer keeps `/usage requests` unshipped, so a default build now answers every `/usage requests…` form with one line, "Per-request cost history is not part of this build.", and its `/usage` menu, `/usage` description, usage error and signed-out hint no longer mention it; no cost page, no next step. The alternative (keep the page, reword its next step) still shows a page that can never fill. R2: a bucket that reaches the read time and is short only at its end is "In progress — totals so far, recorded through …", titled "Cost so far", counted in the range total and given no next step; later buckets are "Not started yet"; buckets cut short by retention, the requested range or unverified days stay partial. Open follow-up: a bucket, past or current, that contains or ends after the last recorded request and is not "in progress" as defined here (e.g. yesterday, or the current hour, before any turn since) is still "Partial — excluded". R3: ranges state "daily totals kept since" from the same floor as the day pages (one `aggregate_day_floor` and one 365-day `REPLAY_MS` in `state/src/runtime/accounting.rs`; outside the frozen scope, as are `accounting_retention_atomic.rs`, `accounting_scope.rs`, `tui/src/slash_command.rs`, three new `*_default_build` menu snapshots and two edited snapshots). Regression tests in both feature sets.

## PR descriptions (gates: tests, clippy, review)

### #291

Verbatim, PR #291 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/291), in full:

> PF-60-S03 remaining gaps, developer-only `/cost` inspector (codex-rs/tui only).
>
> - **Missing usage vs missing price.** A pay-per-use attempt that has a price but an unreported token count reads "usage incomplete"; one with no rate reads "no price available"; mixed groups count each ("1 attempt had no price, 1 had incomplete usage"). The exact-USD detail line follows suit.
> - **Unavailable-backend next steps.** Ledger absent, thread gone, checkpoint lag, refresh needed, too large, expired detail, remote server, no thread, conversation switched, DB not open, unreadable (now logged), timeout: each states a `Next step:` line.
> - **Current command.** `/cost` usage errors name `/cost`; hints name the command the build resolves (`/cost` only with `developer-accounting`, otherwise `/usage requests`, so shipped text is unchanged).
> - A day of only subscription work no longer points at a provider bill (unless other conversations' pay-per-use figures are on screen).
> - **Regression tests:** no-usage, stale estimate (superseded reply never replaces a refresh), current command, every unavailable state names a next step, subscription-only footer.
>
> Gates: `just test -p codex-tui accounting cost usage scope subscription signed_out` 207/207 default, 208/208 with `--features developer-accounting`; `just fmt`, `just fix -p codex-tui`; Linux clippy `-D warnings` on the RTX (results in PR comments). Independent Opus 5.5 High review: round 1 REQUEST CHANGES (9 findings, all addressed in 2fb21ad640), round 2 pending.

### #299 (#287)

Verbatim, PR #299 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/299), in full:

> Refs #287 (closed after the post-merge code-blind acceptance re-run).
>
> **Contract** (the product owner, accounting lane, 2026-10-08): "Accounting must never block a model request; handle clock skew safely and fail open for the request while recording an honest accounting gap." Issue #287 has the PF-60-S03 acceptance evidence (#292, criterion 15).
>
> ## Problem
> In a developer-accounting build, if the wall clock was behind the ledger's retention checkpoint, every `AsOf::Now` store write failed with "negative or backward checkpoint". The acceptance run hit this at 58 s and at 6 days behind. Accounting was fail-closed, so every model request in the turn stopped before it was sent, with "Native Anthropic accounting failed" even on Z.AI.
>
> ## Fix
> 1. **Clock skew (state).**
>    - `AsOf::Now` is never sampled behind the checkpoint that the write's transaction sees. The ledger time holds at the checkpoint until the clock passes it.
>    - Each attempt keeps the dispatch time the clock actually read. Explicit `AsOf::At` times stay strict.
>    - When the clock is more than the 90-day detail window behind, recording is impossible, so the request goes out unrecorded and the gap is shown.
> 2. **Fail open (core).** Accounting watches requests and never stops one.
>    - Any failure closes the sampling, and the request goes out unrecorded: the store, attributing the route, admission, an observation, a poisoned slot, or a wait for the write gate or the state DB longer than 15 s.
>    - A closed sampling records nothing more. Later requests use the ordinary client.
>    - The turn warns once: "Developer accounting could not record one or more model requests in this turn. They were sent anyway; /cost does not include them…" This covers ordinary turns and `/compact` (local, legacy and remote v2).
>    - The cause is logged under `codex_core::accounting`.
> 3. **Redirects.**
>    - A recorded request goes out on a client that never follows redirects, so a response from elsewhere is never attributed to the approved route.
>    - If the provider redirects, accounting gives up on that request. The request is resent once, unrecorded, on the caller's ordinary redirect-following transport (`with_redirect_fallback`). This happens on every path: streaming, compaction, realtime call creation and memory summaries.
>    - A WebSocket handshake redirect fails exactly as it does with accounting off.
>    - The ledger keeps the refused redirected send as an attempt with usage unknown. The resend that was served is a gap, which the warning reports. This is accepted for now.
> 4. **Streams (codex-api).** Rejected usage evidence stops observation for that response only. It never ends the Chat, Responses, WebSocket or Messages stream.
> 5. **Only remaining stop.** A stage-one memory denial seen mid-stream. It is a privacy guard, not accounting, and now has its own message. A role that changes the provider's `wire_api` is still rejected when the agent is spawned, with a message of its own.
> 6. `record_sent_request` reports `false` when the usage was not recorded.
>
> ## Tests
> - **New:**
>   - state: `now_writes_hold_at_the_checkpoint_when_the_clock_is_behind_it` (58 s and 6 days)
>   - core: `accounting_clock_behind_the_checkpoint_still_records_and_sends` and `accounting_clock_far_behind_the_checkpoint_sends_unrecorded` (100 days)
>   - end-to-end Chat and Responses: `*_clock_behind_checkpoint_still_runs_and_records`
>   - redirect resend on Anthropic, Chat, Responses, WS fallback and legacy compaction
>   - an observation failure in `record_sent_request`
> - **Rewritten:** tests that asserted fail-closed now assert fail-open. The request is sent, nothing more is recorded, there is no repair send, and exactly one gap warning appears (`core_test_support::assert_accounting_gap`).
> - **Results:**
>   - core lib `accounting`: 79 passed.
>   - core suite `accounting` + `compact`: 208 passed and 2 failed. The two failures are `remote_compact_trim_estimate_uses_session_base_instructions` and `token_budget_auto_compact_fallback_uses_buffer_until_new_context`, which fail identically on main.
>   - `codex-api`: all passed. `codex-state accounting`: all passed.
>   - `config_schema_matches_fixture` also fails on main.
> - **Clippy:** clean on macOS. On Linux (RTX box, `-D warnings`) it is clean with and without `developer-accounting`; the latest round is in the comments.
>
> ## Follow-ups (not model requests)
> - Thread deletion and the `/cost` inspector still refuse a clock that is behind the checkpoint.
> - A durable gap marker in the ledger, so `/cost` can show its totals are incomplete. This is a product call.
> - Run the stream-time stage-one memory check even when no sampling exists. This is a pre-existing gap.
>
> ## Product linkage
> - Exact heading: **Product measurement** in `docs/corbanu-product-spec.md` (via the PF-60 plan, `docs/plans/active/portfolio-agent-cost-accounting.md`, "Product linkage"). Requirement excerpt: "No commercial performance numbers have been supplied."
> - PF-60 plan, "User pain": "missing prices must not look like free work". "Acceptance flows" (Failure/cancel): "A duplicate event, interrupted request, missing price or denied provider produces no invented zero and no double charge." "Invariants": "Unknowns remain unknown".

### #303 (#286)

Verbatim, PR #303 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/303), in full:

> Refs #286 (closed after the post-merge code-blind acceptance re-run).
>
> ## Problem
> `corbanu delete --force <session>` removes the session's accounting attempts and their cost, including its subagents'. Afterwards, the `/cost` day page in every other conversation said **"No other conversation recorded requests on this day."** That was false: the acceptance runs `rtx-30`, `rtx-32` and `rtx-33` had 7 to 18 real requests on those days. `mac-07` shows the other-conversations figure dropping with no disclosure.
>
> ## Fix
> Deletion keeps one replay fence (tombstone) per attempt. It expires exactly `REPLAY_MS` after dispatch, so the dispatch day can be recovered from it. A tombstone only counts as a deletion if its dispatch is inside the 90-day detail window at both the checkpoint and the reader's clock. The reader's clock matters because batched expiry can run ahead of the checkpoint.
> - **state:** `OtherConversations::deleted_attempts` (`DeletedAttempts::Counted(n)` / `PastDetailWindow` / `Unread`) counts the tombstones for the inspected day.
>   - Inside the 90-day raw detail window, every tombstone comes from a deletion. Retention expiry and compact-only late imports leave tombstones only for attempts already past the window.
>   - For a day that reaches past the window, the count is `PastDetailWindow`. If the budget runs out, there is no checkpoint yet, or the read fails, it is `Unread`. Neither is reported as 0.
>   - The read uses the same snapshot and budget as the other-conversations read and never fails the view.
> - **tui:** the day page now discloses deleted spend, after the `/resume` step:
>   - `Deleted conversations or subagents sent N request attempts on this day. Their recorded cost was deleted with them, so it is not included here; any cost they incurred is on your provider's bill.`
>   - With no other saved conversation, the page says "No other saved conversation has recorded requests on this day." and then that line.
>   - "No other conversation recorded requests on this day." appears only when nothing was deleted that day.
>   - An uncountable day says which of those two reasons applies.
>
> ## What this does not do
> It shows the count of deleted request attempts, not their dollar amount. Travis needs to accept "count, not amount" explicitly. Keeping the amount would need a new ledger table, and existing ledgers have no upgrade path (`install_on_connection` only migrates an absent ledger). If Travis wants the amount kept as an anonymous "deleted conversations" population, that is a follow-up with a migration.
>
> ## Tests
> - `accounting_inspect_counts_deleted_conversations_attempts` (state): the day reads `conversations: 0, deleted_attempts: Counted(1)` after deleting the only other conversation.
> - `accounting_deleted_attempts_count_only_what_retention_cannot_explain` (state): the window edge at the checkpoint and at the reader's clock, an exhausted budget, and a missing checkpoint.
> - `every_day_view_says_whether_other_conversations_were_read` (tui) is extended to the counted, past-detail-window, unread and alongside-saved-conversations cases.
> - `cargo test -p codex-state accounting`: all pass. `cargo test -p codex-tui --features developer-accounting --lib tokens`: 90 pass. Clippy is clean on macOS; the Linux result is in the comments.
>
> Contract (accounting lane, 2026-10-08): "deleting a session must not make /cost claim 'no other requests' that day; show the deleted spend honestly."
>
> ## Product linkage
> - Exact heading: **Product measurement** in `docs/corbanu-product-spec.md` (via the PF-60 plan, `docs/plans/active/portfolio-agent-cost-accounting.md`, "Product linkage"). Requirement excerpt: "No commercial performance numbers have been supplied."
> - PF-60 plan, "User pain": "missing prices must not look like free work". "Acceptance flows" (Failure/cancel): "A duplicate event, interrupted request, missing price or denied provider produces no invented zero and no double charge." "Invariants": "Unknowns remain unknown".

### #305 (#288)

Verbatim, PR #305 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/305), in full:

> Fixes the second half of #288 (the first half, the missing-usage label, landed in #291).
>
> - A pay-per-use request on a provider with no published price that reported no tokens now gets the missing-price next step naming **its own** provider (before, the only step on that page could name another conversation's provider).
> - A priced request whose provider did not report every token count gets its own step: "Next step for requests with incomplete usage: check the bill from …".
> - Partly priced attempts (stated as "at least") and plan work get no step, as before.
>
> Tests: `just test -p codex-tui accounting cost usage scope subscription signed_out unpriced` 208/208 default; 209/209 with `--features developer-accounting` (the existing `cost_scope` tmux test failed once on a lost `/cost` keystroke, then passed on retry). New: `unpriced_request_without_usage_names_its_own_provider`; `missing_usage_is_labelled_apart_from_missing_price` extended. Linux clippy and independent review results follow in comments.

### #306 (#289)

Verbatim, PR #306 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/306), in full:

> Refs #289 (closed after the post-merge code-blind acceptance re-run; item 4b is re-checked there). These are the wording and consistency findings from the PF-60-S03 independent acceptance run (#292). Each numbered item below corresponds to the same item in the issue.
>
> 1. **Fresh home** (`mac-40`, `mac-41`):
>    - In a developer-accounting build, `/cost` before the first request now says: "Unavailable — no requests recorded in this home yet; the accounting ledger is created when the first one is."
>    - Other builds say "Unavailable — accounting ledger not installed."
>    - "Collection remains off" is gone. The next step added in #291 still follows.
> 2. **Component lines that contradicted their costs** (`mac-08`, `mac-01`):
>    - Some attempts did not report cache writes and are bound to a price that charges nothing for them (inclusive dialect). Those are costed as input − cache read with a write charge of zero, so the cost is exact.
>    - The technical details now show the count that cost used: `Noncached input (derived for inclusive input): 14188 (derived as input − cache read: cache writes were not reported, and this price charges nothing for cache writes)` and `Cache write: not reported — this price charges nothing for cache writes`. Previously they said "unknown" next to an exact cost and "$0.000000".
>    - Day pages append how many attempts were costed that way.
>    - New `ObservationQuote::priced_counts()` in state exposes the counts the pricer used.
> 3. **Week or month buckets that run past the ledger** (`rtx-23`, `rtx-24`, `rtx-20`):
>    - A bucket now reports the interval it covers, `[start, ledger-current-to)`, the same way a day bucket containing today already does (`mac-21`). Previously it showed "effective coverage: unavailable".
>    - A bucket the ledger has not reached at all is still unavailable. So is a lagging day inside the ledger (detail awaiting compaction).
> 4. **`/usage requests <date>`** (`rtx-52`):
>    - A correctly formatted date after today now gets the reason: "YYYY-MM-DD is after today (…, UTC)…". Previously it got the syntax help, as if the input were malformed.
>    - I could not reproduce a valid past date being refused in a default build. A composer-level test (Enter on `/usage requests <yesterday>`) opens the inspector. I will check this again code-blind on the RTX box with a default build.
> 5. **Request numbering:** request rows are numbered in the order they were sent, not by request UUID. The group pages use the same order and numbers, and attempts within a request are listed in send order.
>
> Tests:
> - `accounting_inspect_free_cache_writes_show_the_counts_their_costs_used`, `accounting_inspect_requests_are_numbered_in_dispatch_order` and `accounting_inspect_future_day_says_why` (tui).
> - `accounting_inspect_range_bucket_reaching_past_the_ledger_states_its_coverage` (state).
> - Updated the fresh-home snapshots.
> - `cargo test -p codex-tui --lib tokens`: 93 passed, with and without `developer-accounting`. `cargo test -p codex-state accounting`: all passed.
>
> #288 and the label work from #291 are not touched.
>
> ## Product linkage
> - Exact heading: **Product measurement** in `docs/corbanu-product-spec.md` (via the PF-60 plan, `docs/plans/active/portfolio-agent-cost-accounting.md`, "Product linkage"). Requirement excerpt: "No commercial performance numbers have been supplied."
> - PF-60 plan, "User pain": "missing prices must not look like free work". "Acceptance flows" (Failure/cancel): "A duplicate event, interrupted request, missing price or denied provider produces no invented zero and no double charge." "Invariants": "Unknowns remain unknown".

### #318 (#289 residuals)

Verbatim, PR #318 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/318), in full:

> Fixes the #289 residuals the second re-run (#315) found on 886f19c.
>
> - **R1, R1b, criterion 13b:** `/cost` is developer-only, so a default build now answers every `/usage requests…` form with one line: "Per-request cost history is not part of this build." The `/usage` menu no longer offers "Recorded requests", and the signed-out `/usage` hint no longer points there. The decision and the reasoning are in the sprint file; Travis may revisit it.
> - **R2:** a week or month bucket that reaches the read time and is short only at its end is now shown as in progress: "Cost so far", "In progress — totals so far, recorded through …". It counts toward the range total and gets no next step. Buckets cut short by retention or by the requested range are still partial.
> - **R3:** the range header now says "daily totals kept since …", using the same floor as the day pages (one shared `aggregate_day_floor` in state).
>
> Tests: regression tests pass on both the default build and `developer-accounting`.

### #322 (#308)

Verbatim, PR #322 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/322), in full:

> Fixes #308.
>
> **Problem.** In developer-accounting builds, `corbanu delete --force` with the clock behind the ledger checkpoint removed the rollout files first, then failed in the state database's accounting step with "negative or backward checkpoint". The session was half-deleted.
>
> **Fix.**
> - **Clock:** deletion reads its time with the #299 rule (`AsOf::Now.sample_on`). While the clock is behind, the time holds at the checkpoint. Explicit test clocks (`delete_threads_at`) still reject backward times.
> - **Order:** before it removes any rollout, `thread/delete` runs `StateRuntime::preflight_delete_threads`. That is the same state-database delete transaction, always rolled back. If the ledger can't be updated, nothing is deleted and the error says so: "could not delete conversation X: its local records (including its cost records) cannot be updated right now, so nothing was deleted. Try again; …".
> - **Why no lock is held:** keeping the write lock open while rollouts are removed was rejected, because the thread store scans every rollout for fork references and can be remote.
> - **Remaining window:** a failure that appears between the preflight and the real delete is still possible. Its message now says the conversation was deleted and to delete it again to finish; a retry completes the delete.
> - **Deleted spend:** still recorded as #303 tombstones and counted under "deleted conversations".
>
> **Tests.**
> - state: `delete_holds_at_the_checkpoint_when_the_clock_is_behind_it` (58 s and 6 days behind). The delete succeeds, the checkpoint never goes backward, a tombstone is left, and `deleted_attempts` returns `Counted(1)`. Also `failed_delete_preflight_changes_nothing`.
> - app-server: `thread_delete_tolerates_a_clock_behind_the_accounting_checkpoint` (58 s and 6 days behind). `thread_delete_leaves_the_rollout_when_accounting_fails` uses an injected ledger trigger: the rollout and ledger stay, the error is the plain-language one, and a retry after the fault clears succeeds.
> - With the fix reverted, both app-server tests and the state clock test fail. The clock test fails with the original `negative or backward checkpoint` error.
> - `sqlx` is added as an app-server dev-dependency, for the fault-injection trigger.

### #339 (date hint, billed wording)

Verbatim, PR #339 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/339), in full:

> Two presentation changes to the developer-only `/cost` screens that Travis asked for on 2026-10-09. They apply only to `developer-accounting` builds; default builds look the same to users.
>
> **1. Date-format hint.** Slash commands can now declare a dimmed argument hint (`SlashCommand::argument_hint`). It appears in the composer after `/name ` and disappears once an argument is typed. `/cost` uses this to show `[YYYY-MM-DD]  or  START END hour|day|week|month  (UTC)`. The hint is hidden in these cases:
> - the hint wouldn't fit whole
> - multi-line input, tabs, bash mode or masked input
> - disabled input or parent-owned threads, where `/cost` can't be submitted
> - any build where `/cost` isn't visible
>
> The `/cost` view also shows the same forms as a dimmed footer note: `Other dates: /cost …`.
>
> **2. Plain billed-cost wording.** This removes `Billed cost: unavailable — no settlement evidence` and `Estimate versus billed difference: unknown — no settlement evidence`. Each page now has at most one line about billing:
> - The provider stated its charge: unchanged, `Billed cost: $X — as stated by the provider with each response`.
> - The provider never reports charges (for example Z.AI): `Billed cost: Z.AI doesn't report its charges, so any cost here is an estimate — check Z.AI's bill.`
> - The provider reports charges but stated none on these responses (OpenRouter, Vercel, the Corbanu API): `… OpenRouter stated no charge for this work, so any cost here is an estimate — check OpenRouter's bill.` A refused attempt that reported nothing says it is normally not charged, and still points at the bill in case it stopped mid-response.
> - The first screen says it once, in its overview line: `Costs are estimates from published prices; Z.AI doesn't report its charges — check Z.AI's bill.` The details don't repeat it.
> - Range headers no longer repeat it on every bucket page. A complete range states it once.
> - Pages with only subscription work, or with nothing to cost, have no billing line.
>
> Only presentation code changed: `tokens.rs`, the composer and the slash-command table. Accounting data and collection are unchanged.
>
> **Gate**
> - **Focused tests:** codex-tui (`chatwidget::tokens`, `chat_composer`, `slash`), 555 passing with and without `developer-accounting`. Snapshots were updated deliberately.
> - **Full codex-tui lib:** passes on both feature sets except two tests that fail regardless of this change. `command_popup_default_items` is macOS snapshot drift, and `default_daemon_auto_connect_probes_socket_only` fails because the socket path is too long (`SUN_LEN`).
> - **Linux clippy, `-D warnings`:** clean with and without the feature, run on the RTX box.
> - **Review:** an independent Opus 5.5 High review through `corbanu exec`, read-only. Round 1 requested changes; the Major is fixed. Round 2 approved with nits, which are also fixed.
> - **TUI check:** a fresh debug build, a real GLM 5.2 turn on zai in a disposable home, and captures in `qa/portfolio/agent-cost-accounting/pf-60-s03/cost-hint-wording-20261009/`.

Where a PR description gives no review verdict or no Linux clippy line, this record does not claim one; the archived record's Done entries quoted above give the review rounds for #291 and #318.

## Open, not done

| Issue | State | Closed (UTC) | Title |
| --- | --- | --- | --- |
| [#325](https://github.com/CorbanuCore/CorbanuTerminal/issues/325) | OPEN | — | PF-60-S03: /cost leftovers after #289 (rejected attempt counted as billed; bucket past the last request reads Partial) |

- #287 residual: the clock-behind WARN says "recording at the ledger's time" while the request page shows the recorded admission time (archived record, Closure).
