# PF-60-S03 independent acceptance: third targeted re-run (2026-10-08)

This is the final confirming re-run before Travis decides on PF-60-S03. It checks the fixes merged since the
[second re-run](../rerun2-20261008/README.md):

- PR #318, which fixes the #289 residuals R1, R1b, R2 and R3, plus criterion 13b
- PR #322, which fixes #308 (delete with the clock behind the ledger checkpoint)

It also repeats one case each of the criteria that passed before: 8a, 9 and 15.

| Item | Verdict | One-line evidence |
| --- | --- | --- |
| 1. #289 R1, R1b and 13b (default build) | **PASS** | `/usage requests`, `… 2026-10-07` and `… 2026-10-09` each print one line, "Per-request cost history is not part of this build.", with no page and no next step (`mac-289-r1-01`…`03`). This home was signed out, so `/usage` shows "Sign in with ChatGPT to view OpenAI account usage." (`mac-289-r1-04`). Two sub-items are **NOT VERIFIABLE** in a signed-out home: the signed-in `/usage` menu and #318's "usage error" text |
| 1. Criterion 13a (default build): `/cost` | **PASS** | "Unrecognized command '/cost'" (`mac-289-13a`) |
| 2. #289 R2: week and month buckets that reach today, and the week after today | **PASS** | The bucket reaching today is titled "Cost so far — this conversation" and says "In progress — totals so far, recorded through 22:45:11.964Z". Its $0.015922 counts in the range total, and it has no next step. The week after reads "Not started yet — it starts after this view was read." (`mac-289-r2-01`…`06`) |
| 2. #289 R3: retention floor wording | **PASS** | Both the range header and the day page say "daily totals kept since 2025-10-09" (`mac-289-r2-01`, `-03`, `-05` vs `mac-c9-02`) |
| 3. #308: `delete --force` with the clock about 57 s and 6 days behind the checkpoint, then the "deleted conversations" disclosure | **PASS** (succeeded, state consistent) | Both deletes exit 0 with "Deleted session …" and no error, and the rollout is gone (`rtx-308-del58s`, `rtx-308-del6d`). `resume` says "No saved session found" (`rtx-308-03`). Today's page leaves both out and says "Deleted conversations or subagents sent 4 request attempts on this day" (2 + 2) (`rtx-308-01`). 2026-10-02 shows nothing (`rtx-308-02`) |
| 4. Regression: 8a (a no-usage conversation names its own provider) | **PASS** | The `zainousage` conversation's `/cost` and Request 1 page both say "check the bill from **zainousage**" (`mac-c8-01`, `mac-c8-02`) |
| 4. Regression: 9 (deleted history) | **PASS** | A clean single delete at the real clock on macOS, root plus subagent. The viewer's day page says "Deleted conversations or subagents sent **6** request attempts on this day". `d1` made 6 requests (`mac-c9-01`, `mac-c9-02`) |
| 4. Regression: 15 (turn with the clock behind the checkpoint) | **PASS** | `sk58s` ran 58 s behind real time, which was 49 s behind the checkpoint (`behind_ms=48950`). It reached `turn.completed`. All 7 requests were recorded: 50,427 tokens, exact USD 0.01776348, admission time 22:39:09.255Z on the faked clock (`rtx-c15-01`…`03`). Still reproduces open #287 residual 2 (see §4) |

Every displayed total that I checked matches my own recomputation from provider-reported usage at Z.AI's published
prices, to the micro-dollar ([data/recompute-mac.txt](data/recompute-mac.txt),
[data/recompute-rtx.txt](data/recompute-rtx.txt), and the `-exact.txt` files next to them). **No new defect was found**; one known residual of closed #287 (residual 2) still reproduces (§4).

Executor: an independent Codex worker, code-blind. One separate code-blind reviewer audited this record: installed
`corbanu exec`, `claude-plan`, `claude-opus-5-5-plan`, `high` effort, `-s read-only`. Its report is
[REVIEW.md](REVIEW.md).

## What I read (code-blind boundary)

I read only these sources:

- the earlier independent records: [../README.md](../README.md), [../rerun-20261008/](../rerun-20261008/README.md) and
  [../rerun2-20261008/](../rerun2-20261008/README.md), with their REVIEW files, captures, data and tools
- `../../acct-acceptance-75/acceptance.md`
- `gh issue view` for #286–#289 and #308, including the #289 comment that describes #318's intended behaviour
- `docs/integrations/zai-glm-52.md`
- `sec-common.md`

I did not read `codex-rs/**`, PR diffs, lane briefs, review files or the fix lanes' evidence directories. From #308 I
read the issue text and its "Fixed by #322" comment, which describes the intended behaviour; I did not read #322's diff.

## Build provenance ([data/build-receipts.txt](data/build-receipts.txt))

I polled `gh pr view 322 --json state` every 5 minutes from 21:49Z. It reported `MERGED` at 22:34:10Z, and I then built
the new `origin/main`. **#322 is included.**

| | macOS | Linux (RTX box) |
| --- | --- | --- |
| Commit | `origin/main` `ccc38bfc030122cc33ac84241e9be4c2987ef806` (merge of PR #322). Both #318 (`290bb28532`) and #322 are ancestors (`git merge-base --is-ancestor`, exit 0) | the same commit, in a fresh clone |
| Host | macOS 26.6.2 arm64, rustc 1.95.0 | Linux 6.8.0 x86_64, rustc 1.95.0 |
| Developer-accounting build | [tools/build-mac.sh](tools/build-mac.sh): `cargo build --locked -j8 -p codex-cli --bin corbanu --features codex-core/developer-accounting,codex-tui/developer-accounting`. sha256 `d0d0f32d…2194944e7c` | [tools/build-rtx.sh](tools/build-rtx.sh) `acct`. sha256 `95f07597…62da26` |
| Default build | `tools/build-mac.sh default`. sha256 `43fa46c3…836b98c29af5c` | built (`d7c97490…d8c337`) but not used |

Each host used a new target dir. While #322 was in CI, each dir got one warm-up build at `adbe1b0f81`. The binaries
above were then rebuilt from `ccc38bfc03`, and each macOS binary was copied to `<scratch>/bin` before the next build ran.
libfaketime is `wolfcw/libfaketime` `e8d8c85`, built from source on the RTX box (`libfaketime-time64.so.1`).

## Safety

- Every built binary ran with `CORBANU_TEST_NO_NATIVE_KEYRING=1` and new disposable `CODEX_HOME`, `CORBANU_HOME` and
  `PFTERMINAL_HOME` homes:
  - macOS: `mh-def` and `mh-dev`
  - RTX: `h-del`
  The real keychain was never touched.
- The real key came only from the installed `corbanu`:
  - **macOS** ([tools/execrun.sh](tools/execrun.sh)): the substitution is the first prefix on the consuming
    `corbanu exec`: `ZAI_API_KEY="$(~/.local/bin/corbanu vault auth-helper provider/zai_api_key)" … corbanu exec …`
  - **RTX** ([tools/rtxrun.sh](tools/rtxrun.sh)): the substitution is now attached to the consuming command, a
    `bash -c` that pipes it over ssh stdin. The remote side reads it into the process environment and never writes it.
    Earlier runs substituted it into `printf` instead.
- Every TUI session, every `delete` and every `resume` used a placeholder key.
- **Key scan** ([data/key-scan.txt](data/key-scan.txt)): `grep -rlF`, with the helper substituted directly on the grep
  command, found 0 files in this directory, 0 in the macOS scratch logs, homes and raw captures, and 0 in the RTX ones.
- Paths are redacted to `<scratch>`, `<rtx-dir>` and `<corbanu-root>`, and the host to `<rtx-host>`.
- Deviations, disclosed: on RTX the helper's substitution feeds a `bash -c` relay (`printf` builtin into ssh stdin), not
  the `corbanu` process itself, because the consumer is on another host. The key scan substitutes the key into `grep`'s
  arguments, so for the scan's duration it was visible in the local process table. The macOS `d1` delete had no
  wrapper; its exact command (placeholder key, keyring off, disposable home) is in
  [data/supplementary-evidence.txt](data/supplementary-evidence.txt) §3.

## Method

The method is the same as in the earlier re-runs:

- **Real work:** real GLM 5.2 `exec` turns on the built-in `zai` provider, on `sindresorhus/is-plain-obj` `666df7c` and
  `chalk/ansi-regex` `f923667`. The prompt was "Read readme.md and summarize it in one sentence."
  - `d1` instead asked the model to spawn one subagent to read `index.js`.
  - `nu-nousage` used `-c model_provider="zainousage"`, through the usage-stripping proxy
    ([tools/usage_proxy.py](tools/usage_proxy.py) `--strip-usage`, an unchanged copy).
- **Usage sources:** the product's `codex_api=trace` SSE lines for `zai`, and the outside-the-product proxy log for
  `zainousage`.
- **Prices:** Z.AI's published rates, in USD per 1M tokens: input 1.40, cached input 0.26, output 4.40.
  `docs/integrations/zai-glm-52.md` has no prices; the rates are the ones the first run took from Z.AI's pricing page,
  and the reviewer checked them there independently.
  [tools/recompute.py](tools/recompute.py) and [tools/product_seen_usage.py](tools/product_seen_usage.py) are unchanged
  copies (byte-identical to rerun2). [tools/exact_usd.py](tools/exact_usd.py) is new: it prints the unrounded sums
  from `recompute.py`'s per-request lines.
- **Screens:** the TUI ran in tmux (socket `pf60rerun3`) and was captured by scrolling with the same scripts. As before,
  each capture is the union of the popup's lines, de-duplicated in first-seen order.
  - [tools/scrollcap.sh](tools/scrollcap.sh) gained one header word, `Cost so far`, so it captures the new bucket title.
  - Three captures are raw screens instead (`mac-289-r1-04*`, `mac-289-r2-02-top`, `mac-289-r2-04-raw`), so that a page
    title is visible.
- **Clock shifts:** RTX only, libfaketime with `FAKETIME_DONT_FAKE_MONOTONIC=1`.
  - The delete wrapper is [tools/rtx-del.sh](tools/rtx-del.sh). It lists the session's rollout files before and after.
  - A control run of `date` under the same preload and `FAKETIME` printed the faked time
    (`2026-10-02T22:41:05Z`, real `2026-10-08T22:41:06Z`; tail of `rtx-308-del6d`).
- **Run records:** thread ids are in [data/threads.txt](data/threads.txt), runner exit lines in
  [data/runner-exit-lines.txt](data/runner-exit-lines.txt), and `exec --json` output in `captures/exec-json/`.

All runs happened on UTC day 2026-10-08, between 22:38Z and 22:50Z.

## 1. #289 R1, R1b and criterion 13b (macOS default build): PASS

Home `mh-def`. The default build first completed a real 2-request turn (`def-zai`, `turn.completed`). I then resumed
that conversation view-only and confirmed with `ps` that `corbanu-default` was the running binary.

| Command | What it says | Capture |
| --- | --- | --- |
| `/usage requests` | "• Per-request cost history is not part of this build." | `mac-289-r1-01` |
| `/usage requests 2026-10-07` (past) | the same single line | `mac-289-r1-02` |
| `/usage requests 2026-10-09` (future) | the same single line | `mac-289-r1-03` |
| `/usage`, typed (command popup) | "/usage  view account usage or use a rate-limit reset" | `mac-289-r1-04a` |
| `/usage`, sent | "■ Sign in with ChatGPT to view OpenAI account usage." | `mac-289-r1-04` |
| `/usage bogus` | the same sign-in line | `mac-289-r1-05` |
| `/cost` (13a) | "Unrecognized command '/cost'. Type "/" for a list of supported commands." | `mac-289-13a` |

- No form opens a "Cost — this conversation" page.
- No line says "accounting ledger not installed", "if this build records costs" or "Run /usage requests for today".
- Neither the `/usage` description nor the signed-out hint mentions per-request history.
- **Limit:** the disposable home has no ChatGPT sign-in, so the signed-in `/usage` menu (the menu item #318 changed)
  could not be shown. That one sub-item is NOT VERIFIABLE. I did not sign in, because doing so needs a real account in a
  test home.
- **Limit:** #318 also changed "the usage error". `/usage bogus` returned the signed-out hint instead
  (`mac-289-r1-05`), so the usage error was not reached either. It is NOT VERIFIABLE for the same reason.
- The `ps` line and the 0-count of 401s in the default home's `stderr.log` are in
  [data/supplementary-evidence.txt](data/supplementary-evidence.txt) §1–2 (added after the review).
- I cleared the composer (which still held "/cost") before closing the session. No prompt was sent: the session's
  stderr log has no 401.

## 2. #289 R2 and R3 (macOS developer build): PASS

Home `mh-dev`. The viewer conversation was `dev-zai`, 2 real requests. The ledger was current to 22:45:11.964Z.

| Command | Result | Capture |
| --- | --- | --- |
| `/usage requests 2026-10-05 2026-10-12 week` | Range line: "In progress — totals so far, recorded through 2026-10-08T22:45:11.964Z", estimated $0.015922. Bucket coverage `[2026-10-05, 2026-10-08T22:45:11.965Z)`. Listed as "Week [2026-10-05…, 2026-10-12…) (in progress)" | `mac-289-r2-01` |
| that bucket, opened | Title "**Cost so far — this conversation**". "In progress — totals so far…". "This conversation, so far in [2026-10-05, 2026-10-12) (UTC): 2 requests, 25,819 tokens, $0.015922". "Known subtotal exact USD: 0.0159218". No next step | `mac-289-r2-02`, `mac-289-r2-02-top` |
| `/usage requests 2026-10-05 2026-10-19 week` | The current week is in progress, as above. The next week, `[2026-10-12, 2026-10-19)`, has coverage "not started" and is listed "(not started)". The range total is $0.015922, which is only the in-progress bucket | `mac-289-r2-03` |
| the 2026-10-12 bucket, opened | Title "**Not started yet**". "Not started yet — it starts after this view was read." No next step | `mac-289-r2-04`, `mac-289-r2-04-raw` |
| `/usage requests 2026-10-01 2026-11-01 month` | "Month […) (in progress)". Opened, it is titled "Cost so far — this conversation", shows the same totals and has no next step | `mac-289-r2-05`, `mac-289-r2-06` |

**R3.** Every range header above says "Retention: request detail kept since 2026-07-10T…; daily totals kept since
2025-10-09". The day page in the same home says the same (`mac-c9-02`). The old "oldest daily total kept none" no longer
appears.

**Recomputed:** `dev-zai` has 2 requests, 12,566 + 13,253 = 25,819 tokens and exact USD 0.0159218
([data/recompute-mac-exact.txt](data/recompute-mac-exact.txt)).

**Observation (not a defect):** a range that starts after today is refused.
`/usage requests 2026-10-12 2026-10-19 week` prints "Range refused: future start is unavailable. Usage: /usage requests
START END hour|day|week|month" (`mac-289-r2-07`). Days after today are still reachable inside a range that starts today
or earlier, as shown above.

## 3. #308: delete with the clock behind the ledger checkpoint (RTX developer build): PASS

Home `h-del`, in this order:

1. `a1`, `a2` and `vk` (the viewer) ran at the real clock, 2 requests each.
2. `sk58s` ran (§4).
3. `cp1` ran at the real clock, 2 requests, to write a fresh checkpoint. The ledger was then current to
   22:40:51.235Z.
4. The two deletes ran, each through [tools/rtx-del.sh](tools/rtx-del.sh) with a placeholder key.

| Case | Clock | Output | Rollout | Capture |
| --- | --- | --- | --- | --- |
| delete `a1` ("58 s" in the tool tags) | `FAKETIME="@2026-10-08 22:39:54"`, about 57 s behind the checkpoint, taken right after `cp1` ended | "Deleted session 01a11daa-d8fc-…", **exit 0**, empty stderr | 1 file before, none after | `rtx-308-del58s` |
| delete `a2` | `FAKETIME="@2026-10-02 22:41:05"`, real time − 6 days (5 d 23 h 59 m 46 s behind the checkpoint) | "Deleted session 01a11daa-d8a3-…", **exit 0**, empty stderr | 1 file before, none after | `rtx-308-del6d` |

The state afterwards is consistent:

- **Resume.** `corbanu resume <a1>` and `resume <a2>` exit 1 with "No saved session found with ID …" (`rtx-308-03`).
  This now agrees with the delete's success message. Before #322, delete reported failure while the session was already
  gone.
- **The other sessions are untouched.** The rollouts of `vk`, `sk58s` and `cp1` are still present
  ([data/supplementary-evidence.txt](data/supplementary-evidence.txt) §4: the listing from the run, and again after it).
- **Today's page in `vk`** (`/usage requests`, `rtx-308-01`):
  - Own: 2 requests, 13,429 tokens, "Known subtotal exact USD: 0.00411176". Recomputed: 0.00411176.
  - Other conversations: "9 requests in 2 conversations … 64,338 tokens … $0.022646". Recomputed (`sk58s` + `cp1`):
    9 requests, 64,338 tokens, 0.02264604.
  - **"Deleted conversations or subagents sent 4 request attempts on this day. Their recorded cost was deleted with
    them, so it is not included here; any cost they incurred is on your provider's bill."** 4 = `a1` 2 + `a2` 2.
- **The faked day carries nothing.** `/usage requests 2026-10-02` says "no recorded requests" and "No other
  conversation recorded requests on this day", with no deletion line (`rtx-308-02`). So the 6-day-behind delete did not
  move anything onto the faked day.
- **`sk58s`'s page** shows the same picture: other conversations 4 requests, 27,340 tokens, $0.008994 (recomputed `vk` +
  `cp1`: 0.00899432), plus the same "4 request attempts" deletion line (`rtx-c15-01`).

Neither delete logged any warning or error ([data/accounting-log-lines-rtx.txt](data/accounting-log-lines-rtx.txt)).
Code-blind, I cannot show from the product that the behind-clock path ran. I can show:

- the libfaketime control (the faked `date`)
- the arithmetic: for the short case, the same preload and offset produced "negative or backward checkpoint" and a
  half-delete in the first re-run (`../rerun-20261008/captures/rtx/rtx-c15-08`, `-12`). There is no earlier 6-day delete
  to compare with
- the ledger's "current to" time, which stayed at 22:40:51.235Z after both deletes. The deletes did not advance it.

The "fail and leave the session intact" branch could not be exercised, because both deletes succeeded. Nor could the
remaining window that the #308 fix comment names (a failure between the preflight and the real delete).

## 4. Regression pass (one case each)

### 8a: PASS

The `nu-nousage` conversation (`zainousage`) is in home `mh-dev`. The provider reported 2 responses and 20,366 tokens
([data/proxy-stripped.jsonl](data/proxy-stripped.jsonl)). The product saw 0 usage chunks in 63 traced SSE events
([data/product-seen-usage.txt](data/product-seen-usage.txt)).

- **Its own `/cost`** (`mac-c8-01`): "zainousage · Z.AI GLM 5.2 — Pay per use. 2 requests, tokens not reported.
  Estimated cost: no price available." Next step: "check the bill from **zainousage**. No published price covers them…"
- **Its Request 1 page** (`mac-c8-02`): "Provider: zainousage", "Tokens: tokens not reported", and the same next step.

### 9: PASS (and, for the first time on macOS, a clean single delete)

1. `d1` ran at the real clock with one subagent (child `01a11db0-b608-…`): 6 requests, 72,368 tokens, exact USD
   0.05430952.
2. `corbanu delete --force <d1>` printed "Deleted session …" and exited 0. Both the root's and the child's rollouts were
   removed. The other two rollouts stayed (`mac-c9-01`, listings before and after).
3. The viewer `dev-zai`'s `/usage requests` (`mac-c9-02`) shows:
   - other conversations "2 requests in 1 conversation", which is `zainousage`, "tokens not reported"
   - "**Deleted conversations or subagents sent 6 request attempts on this day.** Their recorded cost was deleted with
     them…"

   6 equals `d1`'s provider-reported request count, subagent included.

### 15: PASS (Linux)

`sk58s` ran with `FAKETIME="@2026-10-08 22:39:09"`, started at real 22:40:07, so 58 s behind real time.

- **Distance to the checkpoint.** `vk` had written its checkpoint at about 22:39:58, so the process was 49 s behind it.
  The log's WARN line says "the system clock is behind the accounting ledger … `behind_ms=48950`". The Z.AI response
  ids (UTC+8, provider time) confirm the 58 s offset: for example, request 2's id is stamped 06:40:21 (+08), while the
  product's faked log time for it is 22:39:23 ([data/recompute-rtx.txt](data/recompute-rtx.txt)). (Request 1's pair,
  06:40:09 vs 22:39:20, gives 49 s, because the id marks request start and the log line marks usage end.)
- **Result.** The turn reached `turn.completed`, runner exit 0, with 7 requests.
- **Its `/cost`** (`rtx-c15-01`): 7 requests, 50,427 tokens, "Known subtotal exact USD: 0.01776348". Recomputed: 7
  requests, 50,427 tokens, 0.01776348.
- **Request 1** (`rtx-c15-02`, technical page `rtx-c15-03`): $0.002083, 6,588 tokens, "Admission time:
  2026-10-08T22:39:09.255Z", which is the faked clock. Recomputed: 0.002083.
- **#287 residual 2 still reproduces.** The WARN says it is "recording at the ledger's time until it catches up", but
  the attempt is admitted at the faked process time, about 49 s before the checkpoint (≈22:39:58.134Z), and its price
  is observed at that time too (`rtx-c15-03`). The criterion-15 outcome (the turn completes and is recorded, not lost)
  is unaffected, so the verdict stays PASS; the wording/timestamp mismatch was reported on #287.
- **Other channels.** `grep -il anthropic` is no longer meaningful here (no failure path was reached), so I did not
  repeat it. The libfaketime `LD_PRELOAD` reached the model's shell commands again, as in the first re-run (semaphore
  warnings in `rtx-sk58s.jsonl`). Accounting was not affected.

## Spend

| Source | Requests | US$ at list price |
| --- | --- | --- |
| macOS: `def-zai`, `dev-zai`, `d1`, plus `zainousage` (provider-reported, via the proxy log) | 12 | 0.112201 |
| RTX: `a1`, `a2`, `vk`, `sk58s`, `cp1` | 15 | 0.050583 |
| **Total** | **27** | **≈0.163** |

There were no stray or rejected requests. The TUI sessions used placeholder keys and sent no prompt.

## Incidents (harness, not product)

1. **The first `resume` check gave no message.** Run outside a TTY, `corbanu resume <id>` printed only a backtrace hint.
   I reran it in tmux, where the message, "No saved session found with ID …", went to the session's stderr log
   (`rtx-308-03`).
2. **The first bucket capture missed the title.** `scrollcap.sh` did not know the new title "Cost so far". I added it to
   the script's header pattern and added raw-screen captures.

## Commands (exact, redacted)

```sh
gh pr view 322 --json state                                    # polled every 5 min until MERGED
git merge-base --is-ancestor 290bb285324e… HEAD; git merge-base --is-ancestor ccc38bfc0301… HEAD   # exit 0, 0
tools/build-mac.sh; tools/build-mac.sh default; tools/build-rtx.sh acct; tools/build-rtx.sh default
python3 tools/usage_proxy.py --port 18472 --strip-usage --log proxy-stripped.jsonl
# macOS turns: tools/execrun.sh <home> <cwd> <tag> - "<prompt>" [-c 'model_provider="zainousage"']
CORBANU_BIN=corbanu-default tools/execrun.sh <scratch>/homes/mh-def <cwd> def-zai - "<prompt>"
# RTX turns: tools/rtxrun.sh "<rtx-dir>/h-del <rtx-dir>/repos/<repo> <tag> <-|'YYYY-MM-DD HH:MM:SS'> '<prompt>'"
# deletes
corbanu delete --force 01a11db0-9d96-7ec1-a06c-b8aaa7d185c7                        # macOS, real clock
rtx-del.sh <rtx-dir>/h-del '2026-10-08 22:39:54' del58s 01a11daa-d8fc-7c30-bfd7-1769227e63f8
rtx-del.sh <rtx-dir>/h-del '2026-10-02 22:41:05' del6d  01a11daa-d8a3-7901-8252-240fb51e5b1d
# TUI, view-only: tools/macview.sh <sess> "env CORBANU_BIN=<bin> tools/tuiview.sh <home> <cwd> resume <id>"; rtxview.sh on RTX
/usage requests   /usage requests 2026-10-07   /usage requests 2026-10-09   /usage   /usage bogus   /cost
/usage requests 2026-10-05 2026-10-12 week   /usage requests 2026-10-05 2026-10-19 week
/usage requests 2026-10-01 2026-11-01 month  /usage requests 2026-10-12 2026-10-19 week   /usage requests 2026-10-02
# recomputation
python3 tools/recompute.py --err … --proxy proxy-stripped.jsonl --group …;  python3 tools/exact_usd.py data/recompute-*.txt
```

The raw `.err` traces are not committed: they are large and hold full model traffic. The extracted usage is in
`data/recompute-*.txt` and `data/product-seen-usage.txt`.

## For Travis

- **Every item this re-run covered passes on `ccc38bfc03`.** The targeted defects are fixed as reported: #289 R1, R1b,
  R2 and R3, criterion 13b, and #308.
- **#308.** A delete with the clock about 57 s or 6 days behind the ledger succeeds, removes the session cleanly and
  discloses the deleted attempts on the right day. Not exercised: the fail-and-leave-intact branch (no failure occurred)
  and the window between preflight and real delete that the fix comment itself leaves open.
- **#289 R1/R1b/13b.** A default build now says only "Per-request cost history is not part of this build." Two
  sub-items need a signed-in home and were not seen: the signed-in `/usage` menu and the usage error text.
- **This re-run does not clear the S03 acceptance gates** recorded in `acct-acceptance-75/acceptance.md`, which were out
  of its scope: the open P2 scope-zero finding (no fix or accepted waiver; shipping `/usage requests` is held) and the
  round-66 intermittent timeout. Accepting S03 still needs a decision on those.
- **Residuals of closed issues, still present:**
  - #287 residual 2 (reproduced here, §4): the clock-behind WARN says "recording at the ledger's time", but attempts are
    recorded at the behind process time. Residual 1 (a turn more than 90 days behind is disclosed only in the turn
    output) and residual 3 (test wording in the >90-day log line) were not re-tested.
  - #288 item 3: "Pay per use" on custom providers (`mac-c8-01`, `mac-c8-02`). The sprint record tracks it as a
    body-review Major.
  - #289: a 401-rejected attempt is counted as "billed", and "Partial — excluded" for past buckets that end after the
    last recorded request (#318's follow-up in the sprint file). Not re-tested.
  - Out of S03 scope: #290 (`exec --json` reports unknown usage as 0) is open. #310 (key visible to tool commands) is
    closed; not re-tested here.
  - Criterion 15 on macOS remains unverified (libfaketime cannot run behind real time there).

## Independent evidence review

The reviewer's report is in [REVIEW.md](REVIEW.md), verbatim. The reviewer was installed `corbanu exec`,
`-m claude-opus-5-5-plan -c model_provider=claude-plan -c model_reasoning_effort=high -s read-only`, code-blind, on a
copy of this directory plus `context/` (the earlier runs' README/REVIEW files, two first-re-run captures,
`acceptance.md`, `zai-glm-52.md`, issue texts #286–#289 and #308). Conclusion: "Supported with corrections"; every
verdict holds and every number recomputes exactly.

**Applied** (numbers are its findings):

1. "For Travis" now states that the S03 acceptance gates (scope-zero P2, round-66 timeout) are outside this re-run.
2. #287 residual 2 is reported (§4, "For Travis") and commented on #287.
3. The listing of the untouched RTX rollouts is now in [data/supplementary-evidence.txt](data/supplementary-evidence.txt).
4. The short delete is described as about 57 s behind the checkpoint ("58 s" survives only in tool tags).
5. The usage error is stated as not reached; the `ps` check and the 401 count are recorded.
6. The clock-offset example now uses a pair that shows 58 s, and explains the 49 s pair.
7. The 6-day delete is stated to have no earlier precedent; the #308/#322 reading statement is corrected.
8. The price source is stated.
9. The key-handling deviations are disclosed and the macOS delete command is recorded.
10. Capture notes: `mac-289-r2-02-top` is the top of the page as scrolled (Details under the pinned title), not the
    first screen; `mac-289-r2-02` is an incomplete union (incident 2), and `mac-289-r2-06` is the complete equivalent.
11. #310's status is corrected (closed) and #308's remaining window is mentioned.
