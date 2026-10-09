# Task Node evidence record: SECACCT-TN-05

Compile the SECACCT-TN-05 PF-60-S03 Acceptance Evidence Record. Task `task_196d0de5eb848ffc237328b28aadfd02`, request `req_73ae166752cda1771fec9ca88fe4473da3ac8b68b1e3753ad6c0bb5d9c657273`. Every PR below is merged to `main` in CorbanuCore/CorbanuTerminal; each merge commit was checked with `git merge-base --is-ancestor <sha> origin/main` at `3254a302fd` (2026-10-09). Repository quotes are copied verbatim from the files named above them (relative links re-pointed to this file's location); PR-description quotes are verbatim from `gh pr view <n> --json body` on 2026-10-09. Task map: [tasknode-task-map.md](../../../qa/initiative-control/p1-security-and-accounting/tasknode-task-map.md).

**Status: Travis accepted PF-60-S03 on 2026-10-09; the record is archived (#335).** The first independent code-blind run did not accept (#286, #287, #288, #289); three targeted re-runs confirmed the fixes, and re-run 3 on main `ccc38bfc03` found everything it covered passing with no new defect. Every run recomputed each displayed total from provider-reported usage, and each has its own code-blind Opus 5.5 High evidence review. These were scratch-home runs, not the sealed-VM isolated execution; Travis accepted them as the sprint's qualification. **Open, not claimed:** the follow-ups and carried limits in the archived record (quoted below).

Records: [docs/sprints/archive/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md](../../sprints/archive/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md); runs under [qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/](../../../qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/).

| PR | Merge commit on main | Merged (UTC) | Title |
| --- | --- | --- | --- |
| [#292](https://github.com/CorbanuCore/CorbanuTerminal/pull/292) | `a682edc61e` | 2026-10-08T12:38:47Z | qa(pf-60-s03): independent functional acceptance (real GLM 5.2, macOS + Linux) |
| [#311](https://github.com/CorbanuCore/CorbanuTerminal/pull/311) | `886f19c873` | 2026-10-08T17:46:36Z | qa: PF-60-S03 independent acceptance targeted re-run (2026-10-08) |
| [#315](https://github.com/CorbanuCore/CorbanuTerminal/pull/315) | `ef512a9d7c` | 2026-10-08T19:34:04Z | qa: PF-60-S03 independent second targeted re-run (#288, #289) |
| [#328](https://github.com/CorbanuCore/CorbanuTerminal/pull/328) | `df44211c88` | 2026-10-09T05:37:18Z | QA: PF-60-S03 independent third re-run (#289 residuals, #308, regression) |
| [#335](https://github.com/CorbanuCore/CorbanuTerminal/pull/335) | `0c3ad37a10` | 2026-10-09T07:42:37Z | docs(accounting): archive PF-60-S03 (accepted by Travis 2026-10-09) |

Verification output (`gh pr view <n> --json state,baseRefName,mergeCommit,statusCheckRollup`, then `git merge-base --is-ancestor`; "checks" counts the PR's final check conclusions):

```text
#292 MERGED base=main merge=a682edc61e checks: SKIPPED=10 SUCCESS=24; git merge-base --is-ancestor a682edc61e 3254a302fd -> exit 0
#311 MERGED base=main merge=886f19c873 checks: SKIPPED=10 SUCCESS=24; git merge-base --is-ancestor 886f19c873 3254a302fd -> exit 0
#315 MERGED base=main merge=ef512a9d7c checks: SKIPPED=10 SUCCESS=24; git merge-base --is-ancestor ef512a9d7c 3254a302fd -> exit 0
#328 MERGED base=main merge=df44211c88 checks: SKIPPED=10 SUCCESS=24; git merge-base --is-ancestor df44211c88 3254a302fd -> exit 0
#335 MERGED base=main merge=0c3ad37a10 checks: SKIPPED=13 SUCCESS=21; git merge-base --is-ancestor 0c3ad37a10 3254a302fd -> exit 0
```

## First run (#292): not accepted

Verbatim, `qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/README.md` lines 1-21:

> # PF-60-S03 independent functional acceptance (2026-10-08)
>
> **Result:** S03 is **not accepted**. The inspector's arithmetic and attribution are exact. Every displayed total on macOS
> and Linux matched an independent recomputation from provider-reported usage at Z.AI's published prices, to the
> micro-dollar. That included subagents, concurrent `corbanu exec` runs, restart and day/hour/ISO-week/month boundaries.
> Three criteria fail on real runs:
>
> - **Deleted history:** after a session is deleted, the day screen states "No other conversation recorded requests on this
>   day" for days on which that session and its subagents really spent money. Excluding deleted spend may be a product
>   choice; asserting its absence is the defect. Issue [#286].
> - **No-usage:** a no-usage request's own page gives no next step. Its only next-step line names a different provider.
>   Issue [#288].
> - **Store robustness:** with developer accounting on, while the clock is behind the ledger checkpoint (58 s and 6 days
>   tested), the turn fails before any request is sent. The error names Anthropic for a Z.AI request. It recovered once the
>   clock passed the checkpoint. This is probably inherited S02 fail-closed behaviour. Issue [#287].
>
> Lower-severity wording issues are in [#289]. The `exec --json` usage gaps in [#290] are recorded as an out-of-scope
> observation, not an S03 FAIL. An independent reviewer (Opus 5.5 high, code-blind, read-only) audited this record; its
> report is [REVIEW.md](../../../qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/REVIEW.md), and the corrections it prompted are listed at the end.
>
> Executor: independent Codex worker, code-blind. Decision `acct-s03-acceptance-20260917`, option 2.

Verbatim, `qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/README.md` lines 174-195:

> ## Verdicts
>
> | # | Criterion (source) | Verdict | Evidence |
> | --- | --- | --- | --- |
> | 1 | User can explain each displayed total from constituent requests without inspecting storage (PF-60 acceptance) | **PASS** | Conversation, provider, request and attempt pages show tokens, components, rates, price source and exact USD; all reconcile (table above). Day-wide statements after deletion fail under 9. |
> | 2 | Inspect one run and its descendants; root = own + resolved descendants (plan outcome; S03-B) | **PASS** | mac TUI, mac c3, RTX k1/k3/b-day |
> | 3 | Distinguish measured tokens, estimated cost, billed cost, Corbanu API balance (plan outcome) | tokens/estimate **PASS**; billed and balance **NOT VERIFIABLE** | Every page says "Billed cost: unavailable — no settlement evidence"; there is no settlement seam. Corbanu Plan/API balance does not appear in `/cost` and was not exercised (Z.AI only). |
> | 4 | Reopen the same totals after restart (plan outcome) | **PASS** | `mac-01` vs first view; every RTX view is a fresh process |
> | 5 | User-selected ranges: hour/day/ISO-week/month on UTC; partial buckets labelled and never totalled; hour refused outside raw-detail window; future day refused (S03-C) | **PASS** (P3 wording in #289) | `mac-20..27`, `rtx-12..14`, `rtx-20..25`. Displayed TZ was Phoenix on RTX, all output UTC. A week/month bucket reaching the future shows coverage "unavailable" rather than the partial interval. |
> | 6 | Empty day vs unavailable distinguished | **PASS** (P3 wording in #289) | Empty: `mac-10`. Before retention: `mac-11`, `mac-26`. Deleted conversation: `rtx-34/35`. Fresh store: `mac-40`, but "Collection remains off" is wrong (`mac-41`). |
> | 7 | Missing price never shown as free; next step to the provider's bill (`acct-scope-62`) | **PASS** for a priced-tokens custom provider | `mac-03`, `mac-05`: "no price available", "at least $…", "check the bill from zaiproxy" |
> | 8 | No-usage requests | **FAIL** | `mac-04`: the conversation's own page gives no next step for its unpriced requests; the only next-step line names another provider (`zaiproxy`). This contradicts the sprint claim "every page showing an unpriced request now carries the next step". The "no price available" label cannot be isolated here, because `zainousage` is a custom provider with no price either way; a no-usage request on a priced built-in provider was not achievable, since built-ins cannot be re-pointed. #288 |
> | 9 | Deleted history | **FAIL** | `rtx-30`, `rtx-32`, `rtx-33`: after deletion, the day screen states "No other conversation recorded requests on this day" for days with 7–18 real requests (incl. subagents). `mac-07`: the drop happens with no disclosure. The defect is the false statement; whether to keep deleted spend is a product decision. #286 |
> | 10 | Scope: an empty or partial conversation is not presented as a day-wide zero (`acct-scope-62` fix) | **PASS** (except after deletion, see 9) | `rtx-31`: a conversation with no requests that day says so for itself and lists the other conversation's 7 requests and $0.012682. `mac-10`: the empty-day statement "No other conversation recorded requests on this day" is true there. |
> | 11 | Concurrent runs and subagents: no loss, no double count | **PASS** | mac 3 concurrent + TUI; RTX 4 concurrent; per-request provider ids reconcile 1:1 |
> | 12 | Narrow screen, mixed provider | **PASS** | `mac-30` (60 columns, wraps, nothing truncated); `mac-03` (zai, zaiproxy, zainousage listed separately) |
> | 13 | Developer-only activation: `/cost` absent from a default build | **PASS** (Linux only) | `rtx-50/51`: "Unrecognized command '/cost'". `rtx-52`: `/usage requests` prints the developer syntax (#289). |
> | 14 | Historical sessions remain inspectable | **PASS** within retention | Past days 09-30 … 10-07 via resume; 90-day detail window enforced (`mac-27`) |
> | 15 | Accounting store failure handling (sprint: "unavailable-backend" states; executor's inference that collection must not stop work) | **FAIL** | `data/r-skew60s*`, `data/r-month-backward-errors.txt`: two turns with the clock behind the checkpoint (58 s, 6 days) failed before sending. `data/r-recover-turn-completed.jsonl`: the same home worked once the clock was ahead again. Likely inherited S02 fail-closed design. #287 |
> | 16 | CLI/JSON outputs | **OBSERVATION (out of S03 scope; not counted as a FAIL)** | `captures/exec-json/`: `turn.completed.usage` is zeros when usage is unknown (`mac-nousage`) and excludes subagents (`mac-c3`: 37,933 input vs 58,128). #290 |
> | 17 | Platforms macOS + Linux; live repositories; real keys | **PASS for macOS and Linux**; Windows **not run** | both hosts; 3 public repos; real Z.AI key |
> | 18 | Estimate vs provider's actual bill | **NOT VERIFIABLE** | No access to the Z.AI billing console; recomputed from provider-reported usage and the published price instead. |

Verbatim, `qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/REVIEW.md` lines 104-106:

> ## Overall
>
> **Supported, with corrections.** "S03 not accepted" is well supported, and the sprint record's own open items would block acceptance anyway. "Arithmetic exact" is supported: I found no discrepancy anywhere. The deletion FAIL is real on the narrower false-statement ground. The no-usage FAIL is real but already known, and its evidence is confounded. The clock-skew FAIL is real, but its breadth is overstated and it is probably inherited from S02. The exec-JSON row should not count as an S03 FAIL. The record should also list the unexercised failure, cancel and retry paths as open gaps.

## Re-run 1 (#311)

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

Verbatim, `qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/rerun-20261008/REVIEW.md` lines 1-3:

> Conclusion: **Supported with corrections**
>
> The four verdicts hold, and every headline number I could recompute matches the data and the captures exactly. No finding is blocking. Several claims go further than the evidence: two macOS points rest on evidence that was never captured, one PASS comes from a retry rather than a clean delete, one #289 "Fixed" was tested too narrowly, and the record cites a review that doesn't exist.

## Re-run 2 (#315)

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

Verbatim, `qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/rerun2-20261008/REVIEW.md` lines 7-9:

> ## Pass 2 (complete record)
>
> **Conclusion: Supported with corrections.** Every verdict follows from its captures, and every number in the README matches my own recomputation. Three fixes are needed before #288 is commented on and before the record is relied on: the "#288 fixed" summary goes too far, the key-scan file it cites is missing, and one product observation cites a capture that doesn't show it.

## Re-run 3 (#328)

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

Verbatim, `qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/rerun3-20261008/REVIEW.md` lines 7-9:

> # PF-60-S03 third re-run (`ccc38bfc03`): independent audit
>
> **Conclusion: Supported with corrections.** Every verdict holds and every displayed number recomputes exactly. The corrections are about scope and disclosure: the "For Travis" summary leaves out acceptance gates and one #287 residual that this record's own evidence reproduces, and a few smaller claims are unevidenced or slightly overstated.

## Archived record (#335)

Verbatim, `docs/sprints/archive/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md` lines 22-48:

> ## Closure — 2026-10-09
>
> Completed. Travis **accepted** PF-60-S03 on 2026-10-09 (in chat with the coordinator), after the independent
> code-blind acceptance and its three targeted re-runs. Archived by the accounting integration owner.
>
> Evidence (all under `qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/`):
>
> - [First run](../../../qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/README.md), PR #292: macOS and Linux, real GLM 5.2; every total exact to the micro-dollar; not
>   accepted on #286, #287 and #288 (#289 wording).
> - [Re-run 1](../../../qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/rerun-20261008/README.md), PR #311: #286 and #287 (Linux) pass; #288 still failing; #308 found.
> - [Re-run 2](../../../qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/rerun2-20261008/README.md), PR #315: #288 passes; #289 residuals R1–R3 and 13b still failing.
> - [Re-run 3](../../../qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/rerun3-20261008/README.md), PR #328: everything it covered passes on main `ccc38bfc03`
>   (#318, #322 included); no new defect.
> - Each run has its own code-blind Opus 5.5 High evidence review (`REVIEW.md` beside each README).
> - Fixes received: #299 (#287), #303 (#286), #291 and #305 (#288), #306 and #318 (#289), #322 (#308).
> - Demo videos: [index](../../../qa/demos/index/PF-60-S03.md); PR #332 re-recorded them at `7a11f9068bdf` and added
>   four covering the acceptance fixes.
>
> Follow-ups (tracked, not blockers):
>
> - #287 residual: the clock-behind WARN says "recording at the ledger's time" while the request page shows the
>   recorded admission time.
> - #325: a rejected attempt is counted as billed; a bucket past the last recorded request reads "Partial".
> - The [body review](../../../qa/portfolio/agent-cost-accounting/pf-60-s03/body-review-20261008/README.md)
>   findings, now owned by [PF-60-S05](../../sprints/current/portfolio-agent-cost-accounting/pf-60-s05-collection-correctness-and-billing-basis.md) (PR #330). Its mandate covers the Blocker, Majors 2–5 and Minor 10. Its
>   record excludes Minors 6, 7 and 9 and Nits 11–14; they stay open with no sprint yet.
> - Open limits carried from this record, not discharged: listed under Remaining.

Verbatim, `docs/sprints/archive/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md` lines 106-130:

> ## Remaining
>
> All items are resolved, moved or carried as listed. Nothing here is closed by being moved.
>
> - [x] **Independent functional acceptance:** not accepted on 2026-10-08, then accepted by Travis on 2026-10-09 after three
>   re-runs. #286, #287, #288, #289 and #308 are closed; #287's WARN wording and #325 are follow-ups (see Closure).
> - [x] **Body review findings:** transferred to [PF-60-S05](../../sprints/current/portfolio-agent-cost-accounting/pf-60-s05-collection-correctness-and-billing-basis.md) as recorded in Closure. Minors 6, 7, 9 and Nits 11–14 are
>   open follow-ups with no sprint.
> - [x] **Carried, not discharged, from acct-sweep-64:** thread deletion and explicit maintenance still run the full sweep
>   under the lock (~9 s per deleted conversation on the live ledger); no core test for an interrupt during a lease
>   write; no test for two processes expiring at once; non-accounting writers still give up after 5 s.
> - [x] **PF-83 overlap:** PF-83-S01 is archived and its release of the three TUI paths is recorded there.
> - [x] **Carried, not discharged, from PF-60-S02:** Corbanu plan gateway economics, legacy evidence acquisition,
>   permanent anonymous 365-day fencing, the six leaky test markers and the provider-publication P3. Startup prewarm
>   and auxiliary paths are now collected (09-20..09-21 capture coverage, in Done); complete coverage is S05's audit
>   (AC9).
> - [x] **Carried disclosed limits:** the work budget's denominator is still whole-store rows (an owner/dispatch-day index
>   needs a schema and product decision); the S03-B candidate bound counts unrelated threads on the day; the lineage
>   surface still decides ownership by untyped string equality. Each refuses rather than inventing an amount.
> - [x] **Developer-only activation:** delivered (`developer-accounting` feature, debug builds only); shipping `/cost` stays
>   unauthorised.
> - [x] **Mixed raw/compact ranges:** qualified at their boundaries (Done, `5ece0338d`); compact attribution and
>   full-range totals remain unavailable by design.
> - [x] **Outputs, counterexamples, limits and handoff:** recorded in the acceptance records and in Closure; next are
>   PF-60-S05, then PF-60-S04.

Verbatim, `docs/sprints/archive/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md` lines 146-153:

> ## Exit evidence
>
> - [x] Output: main `ccc38bfc030122cc33ac84241e9be4c2987ef806` (re-run 3), with build commands and binary digests in
>   [its provenance table](../../../qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/rerun3-20261008/README.md).
> - [x] Travis accepted the bounded output on 2026-10-09; go: S05 next, then S04.
> - [x] Handoff: S05 owns the review's Blocker, Majors 2–5 and Minor 10; follow-ups and carried limits are in Closure and
>   Remaining.
> - [x] Archived under `docs/sprints/archive/portfolio-agent-cost-accounting/`; plan backlinks updated.

## Open, not done

| Issue | State | Closed (UTC) | Title |
| --- | --- | --- | --- |
| [#325](https://github.com/CorbanuCore/CorbanuTerminal/issues/325) | OPEN | — | PF-60-S03: /cost leftovers after #289 (rejected attempt counted as billed; bucket past the last request reads Partial) |

- #287 WARN wording residual; body-review Minors 6, 7, 9 and Nits 11-14 (no sprint yet).
- Carried limits listed under Remaining above (full sweep under the lock on delete, work-budget denominator, untyped ownership comparison, PF-60-S02 carry-overs); `/cost` stays developer-only.
- Next sprints: PF-60-S05 (awaiting Travis, SECACCT-TN-07), then PF-60-S04.
