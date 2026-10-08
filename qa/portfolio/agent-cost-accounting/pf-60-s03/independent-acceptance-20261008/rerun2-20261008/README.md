# PF-60-S03 independent acceptance: second targeted re-run (2026-10-08)

This re-run covers criterion 8 / #288 and the #289 residuals listed in the [first re-run](../rerun-20261008/README.md),
including its criterion-13 question. The first re-run built `b294864916`, which did not include PR #305, the fix for
#288. This one builds current `origin/main`, which does.

| Item | Verdict | One-line evidence |
| --- | --- | --- |
| 8a. The no-usage conversation's own pages give a next step that names its own provider | **PASS** | Its `/cost`, provider, request and technical pages all say "check the bill from **zainousage**" (`mac-c8-01`…`05`). Other conversations name it too (`mac-c8-10`, `mac-c8-13`) |
| 8b. Missing usage told apart from a missing price | **NOT VERIFIABLE** (no contradiction on this setup) | The "usage incomplete" label is for a priced request, and the built-in priced provider cannot be pointed at the proxy (`data/repoint-probe-error.txt`). On the custom provider, the rows show both facts: "tokens not reported" (`zainousage`) vs "20,304 tokens" (`zaiproxy`), plus "no price available", which is true here. The cost reason itself does not mention the missing usage |
| 8c. Request priced only in part | **NOT VERIFIABLE** | Same reason. The closest case is partial usage on an unpriced custom provider (`zaipartial`, `completion_tokens` removed): next step "check the bill from zaipartial" and "Output: not reported (2 attempts)" (`mac-c8-10`, `mac-c8-11`) |
| #289 R1. Default build, `/usage requests <past day>`: the next step reads "if this build records costs, send a turn…" | **FAIL** (unchanged) | `mac-289-01` was taken in a home where the default build had already completed a real 2-request turn (`exec-json/mac-def-zai.jsonl`), and it still says "accounting ledger not installed". So the next step leads nowhere |
| #289 R1b. Default build, `/usage requests <future day>`: "Run /usage requests for today" | **FAIL** (unchanged) | `mac-289-02`; today's page is the same "not installed" page (`mac-289-08`) |
| #289 R2. A week or month bucket reaching today is titled "Bucket unavailable", and its next step cannot complete it | **FAIL** (unchanged) | `mac-289-05`, `mac-289-07`: title "Bucket unavailable", next step "send a turn … then select Refresh", for buckets that end on 10-12 and 11-01 |
| #289 R3. The range header says "oldest daily total kept none"; day pages in the same home say "daily totals kept since 2025-10-09" | **FAIL** (unchanged) | `mac-289-04`, `mac-289-06` vs `mac-c8-01` |
| Criterion 13a. `/cost` is absent from a default build | **PASS** (macOS; Linux passed in the first run) | `mac-289-03`: "Unrecognized command '/cost'" |
| Criterion 13b. Should a default build open the "Cost — this conversation" page for `/usage requests`? | **NOT VERIFIABLE**: needs a product decision; behaviour unchanged | `mac-289-01`, `mac-289-08` |

Every number I report below matches my own recomputation from provider-reported usage. That usage comes from the
product's own `codex_api=trace` log for `nu-zai` and from the outside-the-product proxy logs for the three proxy routes.

Executor: an independent Codex worker, code-blind. One separate reviewer audited this record. It ran as installed
`corbanu exec`, `claude-plan`, `claude-opus-5-5-plan`, `high` effort, `-s read-only`, code-blind, on a copy of this
directory plus context. Its report is [REVIEW.md](REVIEW.md); see "Independent evidence review" at the end.

## What I read (code-blind boundary)

I read only these sources:

- this directory's parents: [../README.md](../README.md), [../rerun-20261008/README.md](../rerun-20261008/README.md),
  their REVIEW files, captures, data and tools
- `../../acct-acceptance-75/acceptance.md` and the sprint record's acceptance lines
- `gh issue view` for #286–#289
- `/cost help` (`mac-c8-06`)
- `docs/integrations/zai-glm-52.md`
- `sec-common.md`

I did not read `codex-rs/**`, PR diffs, lane briefs or review files. To decide what to test I used only the PR titles from
`git log` and the #288 issue comments.

## Build provenance ([data/build-receipts.txt](data/build-receipts.txt))

| | |
| --- | --- |
| Commit | `origin/main` `886f19c873dd12dc62ddd5598732ec784a9334a7` |
| Contains #305 and #291 | `git merge-base --is-ancestor <merge> HEAD` returns exit 0 for both: `f0d71e62b6` is the merge commit of PR #305 and `c76e6d0845` that of #291 (`gh pr view N`) |
| Host | macOS 26.6.2 arm64, rustc 1.95.0. RTX was not needed |
| Developer-accounting build | [tools/build-mac.sh](tools/build-mac.sh): `cargo build --locked -j8 -p codex-cli --bin corbanu --features codex-core/developer-accounting,codex-tui/developer-accounting`. sha256 `033518e6…39152ab`, `corbanu 0.1.48` |
| Default build | `tools/build-mac.sh default`, the same build with no feature. sha256 `01913860…d0178d` |

Both builds used one new target dir (`.codex-work/targets/pf60-s03-indep-rerun2-20261008`). The developer-accounting
binary was copied to `<scratch>/bin/corbanu-acct` before the default build overwrote it. The sha256 values are those of
the copies that ran; the receipt has the raw output.

## Safety

- Every built binary ran with `CORBANU_TEST_NO_NATIVE_KEYRING=1` and disposable `CODEX_HOME`, `CORBANU_HOME` and
  `PFTERMINAL_HOME` homes under a new scratch dir. The real keychain was never touched.
- The real key appeared only as the first prefix on the consuming `exec` command:
  `ZAI_API_KEY="$(~/.local/bin/corbanu vault auth-helper provider/zai_api_key)"` ([tools/execrun.sh](tools/execrun.sh)).
- Every TUI session was view-only and used a placeholder key ([tools/tuiview.sh](tools/tuiview.sh)), as did the re-point
  probe.
- Key scan ([data/key-scan.txt](data/key-scan.txt)): `grep -rlF` with the helper substituted directly on the command
  found 0 files, both in this directory and in every scratch log and home.

## Method

- **Proxy setup:** the same as before ([tools/proxy-providers.toml](tools/proxy-providers.toml)). Each proxy is stdlib
  Python, runs outside the product and forwards to `api.z.ai`. It logs the provider's original `usage` for every response.
  - `zaiproxy`: pass-through, using [tools/usage_proxy.py](tools/usage_proxy.py), an unchanged copy (sha256 equal).
  - `zainousage`: strips `usage` (`--strip-usage`).
  - New, `zaipartial`: drops only `completion_tokens` from `usage`. It uses
    [tools/usage_proxy_partial.py](tools/usage_proxy_partial.py), which is `usage_proxy.py` plus `--drop-keys`.
    - Only one shape was tested: a missing output count.
    - The proxy's tail-buffer path does not drop keys. That made no difference here, because the product saw no
      `completion_tokens` in either chunk.
- **Runs:** in a new home `mh-nousage2` on the real clock, each was one real GLM 5.2 `exec` turn on
  `sindresorhus/is-plain-obj` `666df7c`. The prompt was "Read readme.md and summarize it in one sentence."
  - `nu-zai`: built-in `zai`
  - `nu-proxy`: `-c model_provider="zaiproxy"`
  - `nu-nousage`: `-c model_provider="zainousage"`
  - `nu-partial`: `-c model_provider="zaipartial"`
  - Thread ids: [data/threads-mac.txt](data/threads-mac.txt). `exec --json` output: `captures/exec-json/`.
- **What the product saw:** the `codex_api=trace` SSE lines ([data/product-seen-usage.txt](data/product-seen-usage.txt),
  from [tools/product_seen_usage.py](tools/product_seen_usage.py)).
  - `nu-nousage`: 0 usage chunks in 77 events.
  - `nu-partial`: 2 usage chunks with no `completion_tokens`.
  - The provider's original usage is in [data/proxy-stripped.jsonl](data/proxy-stripped.jsonl) and
    [data/proxy-partial.jsonl](data/proxy-partial.jsonl).
- **Screens:** the TUI ran in tmux (socket `pf60rerun2`) and was captured by scrolling with
  [tools/cmdcap.sh](tools/cmdcap.sh), [tools/select.sh](tools/select.sh), [tools/pagecap.sh](tools/pagecap.sh) and
  [tools/scrollcap.sh](tools/scrollcap.sh). As before, each capture is the union of the popup's lines in first-seen order, so
  lines can appear out of screen order. Paths are redacted to `<scratch>` and `<corbanu-root>`. Capture ids are new to this run: for example, `mac-c8-06` is
  now `/cost help`, not the page it was in the first re-run.
- **Prices:** Z.AI's published rates, in USD per 1M tokens: input 1.40, cached input 0.26, output 4.40.
  [tools/recompute.py](tools/recompute.py) is unchanged.

## 1. Criterion 8 / #288

### 8a. Next step for its own requests: PASS

`mac-c8-01` is the `zainousage` conversation's own `/cost`. It was taken before incident 2 below:

- "zainousage · Z.AI GLM 5.2 — Pay per use. 2 requests, tokens not reported. Estimated cost: no price available."
- "Next step for requests with no price: check the bill from **zainousage**, zaiproxy and zaipartial. No published price
  covers them, so no cost is shown for them here."

The other pages of this conversation give the same next step, "check the bill from zainousage":

| Page | Capture |
| --- | --- |
| Provider page | `mac-c8-02` |
| Request 1 | `mac-c8-03` |
| Request 2 | `mac-c8-05` |
| Request 1, technical | `mac-c8-04` |

Other conversations now name `zainousage` too: the built-in `zai` conversation (`mac-c8-13`) and `zaipartial` (`mac-c8-10`).
In the first re-run, only `zaiproxy` was ever named.

### 8b. Missing usage vs missing price: NOT VERIFIABLE (no contradiction on this setup)

The row shows both facts. Its tokens are "tokens not reported", where `zaiproxy` shows "20,304 tokens". Its cost is "no
price available", which is true, because a custom provider has no price. The technical page (`mac-c8-04`) shows:

- every component as "unknown — no retained numeric evidence"
- "Completion/billing status: not recorded"
- "Token cost: unavailable — no applicable price"

The cost reason itself does not mention the missing usage. That would be wrong only on a priced provider, where #291's
"usage incomplete" label is meant to apply. That case cannot be set up here:
`-c 'model_providers.zai.base_url=…'` fails with "model_providers contains reserved built-in provider IDs: `zai`. Built-in
providers cannot be overridden." The proxy received 0 requests ([data/repoint-probe-error.txt](data/repoint-probe-error.txt)).
The first run hit the same limit.

### 8c. Priced only in part: NOT VERIFIABLE (same limit). Partial usage on a custom provider was checked

`zaipartial` (`completion_tokens` removed) behaves as follows:

- **Conversation page** (`mac-c8-10`): "2 requests, 20,308 tokens. Estimated cost: no price available". Next step "check the
  bill from **zaipartial**, zainousage and zaiproxy". Components: "Input: 20,237", "Cache read: 19,904", "Output: not
  reported (2 attempts)", "Total: 20,308".
- **Request page** (`mac-c8-11`): "check the bill from zaipartial", "Tokens: 9,996 tokens".
- **Technical page** (`mac-c8-12`): "Input: 9975", "Cache read: 9920", "Output: unknown".

What the product received still carried `total_tokens` and `reasoning_tokens` (`data/product-seen-usage.txt`). Output was
therefore derivable as total − prompt (21 and 50). Showing it as unknown is the conservative choice, not an error.

### Recomputed totals ([data/recompute-mac.txt](data/recompute-mac.txt))

| Row (product) | Product | Recomputed from provider usage |
| --- | --- | --- |
| Z.AI (`nu-zai`) | 2 req, 25,764 tok, $0.015854, "Known subtotal exact USD: 0.0158538"; input 25,693, cache read 17,920, output 71 (`mac-c8-13`) | 2 req; 12,725 + 13,039 = 25,764; (7,456 × 1.4 + 5,248 × 0.26 + 21 × 4.4 + 317 × 1.4 + 12,672 × 0.26 + 50 × 4.4) / 1e6 = **0.0158538**; input 25,693, cached 17,920, output 71 |
| zaiproxy | 2 req, 20,304 tok | 9,996 + 10,308 = 20,304 |
| zaipartial | 2 req, 20,308 tok; input 20,237; cache read 19,904; reasoning 10 | product-seen `total_tokens` 9,996 + 10,312 = 20,308; prompt 9,975 + 10,262 = 20,237; cached 9,920 + 9,984 = 19,904; reasoning 9 + 1 = 10. Request 1: 9,975 / 9,920 / 9,996 / 9 (`mac-c8-12`) |
| zainousage | 2 req, tokens not reported (3 after incident 2) | provider reported 2 responses, 20,322 tok, list price 0.006018 ([data/proxy-stripped.jsonl](data/proxy-stripped.jsonl)); the third attempt was a 401 with no response |
| "Other conversations" seen from `zainousage` (`mac-c8-01`, `-07`) | 6 req in 3 conversations | 2 + 2 + 2 |
| "Other conversations" seen from `zaipartial` / `zai` / `zaiproxy` after incident 2 (`mac-c8-10`, `-13`, `-14`) | 7 req in 3 conversations | 3 + 2 + 2 (including the unbilled 401) |

## 2. #289 residuals and criterion 13

| Residual | Now | Capture |
| --- | --- | --- |
| R1: default build, past day | Unchanged. "Unavailable — accounting ledger not installed. Next step: if this build records costs, send a turn and run /usage requests again; otherwise check your provider's bill." It appears after the default home had completed a real turn (`def-zai`: 2 requests, `turn.completed`). Today's page is the same (`mac-289-08`). That capture also shows a stray 401 turn (incident 2), so the verdict rests on `mac-289-01` | `mac-289-01`, `mac-289-08` |
| R1b: default build, future day | Unchanged. "2026-10-09 is after today (2026-10-08, UTC); there is nothing recorded for it yet. Run /usage requests for today." | `mac-289-02` |
| R2: bucket reaching today | Unchanged. Title "Bucket unavailable" and "Partial bucket — excluded from totals". Coverage `[2026-10-05, 2026-10-08T18:04:40.946Z)` (week) and `[2026-10-01, …18:08:58.231Z)` (month). Next step "send a turn in this conversation to bring the ledger up to date, then select Refresh". The coverage interval itself is correct | `mac-289-05`, `mac-289-07` |
| R3: retention wording | Unchanged. The range header says "oldest daily total kept none"; the day page in the same home says "daily totals kept since 2025-10-09" | `mac-289-04`, `mac-289-06` vs `mac-c8-01`. Both pages were opened in the `zainousage` TUI session (run log); the captures themselves don't show the conversation. They are linked by the range's coverage end, 18:04:40.946Z, which is 1 ms after `mac-c8-01`'s ledger time |
| Criterion 13a | `/cost` → "Unrecognized command '/cost'" in the default build | `mac-289-03` |
| Criterion 13b | `/usage requests` and `/usage requests <past day>` still open the "Cost — this conversation" page in a default build. Product decision still owed | `mac-289-01`, `mac-289-08` |

No PR since #306 claims to address these residuals, so "unchanged" was the expected result.

### New observations (low severity, wording)

1. **Noncached input is shown as unknown on unpriced providers. On reflection this is consistent, not a defect.**
   - In `zaipartial`, the technical page shows "Input: 9975" and "Cache read: 9920", but "Noncached input (derived for
     inclusive input): unknown" (`mac-c8-12`).
   - On review I checked `zaiproxy`, which has complete usage and also no price. It shows the same: "Input: 9975", "Cache
     read: 0 (reported)", "Noncached input: unknown", "Cache write: unknown" (`mac-c8-15`; day page `mac-c8-14`). So the
     cause is the missing price, not the missing output.
   - The priced path derives the value because "this price charges nothing for cache writes" (`mac-c8-13`; first
     re-run `mac-289-03`). With cache writes unreported and no price, inclusive arithmetic cannot derive it.
   - Not reported as a defect.
2. **A 401-rejected, unbilled attempt is counted as a billed, pay-per-use request** (incident 2; `mac-c8-07`…`09`).
   - It appears as "no price available" with "check the bill from zainousage".
   - The summary counts it as "3 of 3 billed attempts incomplete" (`mac-c8-07`).
   - Its technical page says "Completion/billing status: not recorded".
   - A provider does not bill a rejected request, so "billed" and the next step are misleading. Reported on #289 as a
     low-severity wording point.
   - Every capture taken after incident 2 includes this attempt: `mac-c8-07`, `mac-c8-10`, `mac-c8-13`, `mac-289-06` and
     `mac-289-07`. Their counts are therefore provider responses + 1 for `zainousage`.
3. **#288 item 3 is unchanged.** Custom providers with no known billing are still labelled "Billing: Pay per use"
   (`mac-c8-02`, `mac-c8-11`). The sprint record already tracks this as an open body-review Major: "plan work with no
   catalogue price shown as 'Pay per use'".
4. **`exec --json` reports unknown usage as 0** (`mac-nu-nousage.jsonl`, and `output_tokens` in `mac-nu-partial.jsonl`).
   This is #290, out of S03 scope.

## Spend

| Source | Requests | US$ at list price |
| --- | --- | --- |
| `nu-zai`, `nu-proxy`, `zainousage` (provider-reported), `zaipartial` | 8 | 0.056452 |
| `def-zai` in both default homes, plus the aborted first `nu-zai` ([data/recompute-mac-spend-other.txt](data/recompute-mac-spend-other.txt)) | 6 | 0.071806 |
| 401s from the placeholder key (incident 2) and incident 1's refused connections (no provider call) | 2 + none | 0 |
| **Total** | **14** | **≈0.128** |

## Incidents (harness, not product)

1. **The first proxies died with their launching shell.** The `nu-proxy`, `nu-nousage` and `nu-partial` turns in the
   first home, `mh-nousage`, failed with "error sending request", with no provider call. That home was set aside. The
   proxies were restarted in tmux, and every run was redone in a new home, `mh-nousage2`.
2. **Two stray prompts were sent with the placeholder key, both rejected with 401, at no cost.**
   - Extra Escapes in the `zainousage` TUI turned the half-typed `/usage requests 2026-10-01 2026-11-01 month` into a
     prompt. It got a 401 through the stripping proxy and added a third attempt to that conversation (`mac-c8-07`…`09`).
     `mac-c8-01`…`05` were captured before this.
   - In the default build, the composer kept "/cost" after "Unrecognized command". The next capture sent
     "/cost/usage requests" to Z.AI and got a 401. That capture was discarded and retaken (`mac-289-08`; its last screen
     still shows the 401).
3. **The first default-build home was opened by the wrong binary.** tmux did not pass `CORBANU_BIN` to the session, so
   the developer-accounting binary opened `mh-default`. That home was set aside. Its turn was redone with the default
   build in a new home, `mh-default2`, and the binary was confirmed with `ps` before capturing.

## Commands (exact, redacted)

```sh
git merge-base --is-ancestor f0d71e62b6419593a8888f03c3ad74753842ad48 HEAD   # exit 0
tools/build-mac.sh            # developer accounting
tools/build-mac.sh default    # default build
# proxies (tmux)
python3 tools/usage_proxy.py --port 18471 --log proxy-pass.jsonl
python3 tools/usage_proxy.py --port 18472 --strip-usage --log proxy-stripped.jsonl
python3 tools/usage_proxy_partial.py --port 18473 --drop-keys completion_tokens --log proxy-partial.jsonl
# runs: tools/execrun.sh <home> <cwd> <tag> - "<prompt>" [-c 'model_provider="zaiproxy|zainousage|zaipartial"']
CORBANU_BIN=corbanu-default tools/execrun.sh <home> <cwd> def-zai - "<prompt>"
# TUI (view-only, placeholder key)
tools/macview.sh <sess> "env CORBANU_BIN=<bin> tools/tuiview.sh <home> <cwd> resume <thread>"
/cost   /cost help   /usage requests   /usage requests 2026-10-07   /usage requests 2026-10-09
/usage requests 2026-10-05 2026-10-12 week   /usage requests 2026-10-01 2026-11-01 month
# recomputation
python3 tools/recompute.py --err nu-zai.err nu-proxy.err nu-nousage.err --proxy proxy-pass.jsonl proxy-stripped.jsonl proxy-partial.jsonl
```

The raw `.err` traces are not committed: they are large and hold full model traffic. Only the extracted usage lines are
kept, in `data/product-seen-usage.txt`. `nu-partial.err` is left out of `recompute.py` on purpose: the usage the product saw there has no `completion_tokens`,
which the script requires. The proxy log carries the original usage for those responses.

## For Travis

- **#288: item 2 (no next step) is fixed.** A no-usage conversation now names its own provider in the next step on every
  page.
  - Item 1 is unchanged on the custom-provider setup: "no price available" is still the label, which is true there. It
    is not verifiable on a priced provider. The same goes for "priced only in part": built-in providers cannot be
    re-pointed at a proxy, so verifying it would need a test hook or a priced custom-provider route.
  - Item 3 ("Pay per use" on custom providers) is unchanged; the sprint record tracks it as a body-review Major.
  - #288 stays closed, with a comment that supersedes the first re-run's "still FAIL".
- **#289 stays open.** All three residuals are unchanged. One new point was added there: a 401-rejected attempt is
  counted as "billed".
- **Criterion 13 decision still needed.** Should a default build open the "Cost — this conversation" page for
  `/usage requests`? If yes, its next step should not say "if this build records costs".

## Independent evidence review

The reviewer's report is in [REVIEW.md](REVIEW.md), verbatim. The reviewer was installed `corbanu exec`, `claude-plan`,
`claude-opus-5-5-plan`, `high` effort, `-s read-only`, code-blind.

- **Pass 1** ran on a copy that was missing this README, because of an executor copy error. It still recomputed every
  number in `data/` exactly.
- **Pass 2** audited the complete record. Its conclusion was "Supported with corrections", with no discrepancy in any
  number.

**Applied after pass 2** (numbers are its findings):

1. **#288 wording.** "For Travis" no longer says the defect "as reported is fixed". Item 2 is fixed. Item 1 is unchanged
   on this setup and not verifiable on a priced provider. Item 3 is unchanged and tracked in the sprint record. The #288
   comment supersedes the earlier "still FAIL".
2. **Key scan.** Added [data/key-scan.txt](data/key-scan.txt).
3. **Removed the "credential was rejected" observation.** The capture it cited does not show it on the `zainousage` route.
4. **Rewrote noncached-input observation 1.**
   - I checked `zaiproxy`, which has complete usage and no price (`mac-c8-14`, `mac-c8-15`, captured after the review).
     It also shows noncached input as unknown, so the cause is the missing price, not the missing output.
   - The observation is now recorded as consistent and is not reported on #289.
5. **This section added.**
6. **Provenance.** Added `gh pr view` merge commits and the `git status` of the tracked tree. Added
   [tools/product_seen_usage.py](tools/product_seen_usage.py) and noted that raw `.err` traces are not committed.
7. **Labels kept.** The task asks for PASS / FAIL / NOT VERIFIABLE. R1–R3 are "FAIL (unchanged)", meaning the residual
   reproduces. 13b is "NOT VERIFIABLE: needs a product decision".
8. **R3 link stated.** Both pages were opened in the `zainousage` session; the coverage end matches its ledger time.
9. **Split the "other conversations" row** by the viewing conversation.
10. **Usage sources stated.** The recompute line now names them: the product trace for `nu-zai`, the proxy logs for the
    rest.
11. **Narrow test stated.** The partial proxy tested one shape, and its tail-buffer path does not drop keys.
