# PF-60-S03 independent acceptance: targeted re-run (2026-10-08)

This re-run checks the three criteria that failed in the [first independent run](../README.md), plus the #289
observations. The fixes merged since then are PR #299 (#287), PR #303 (#286) and PR #306 (#289).

| Re-run item | macOS | Linux | Verdict |
| --- | --- | --- | --- |
| Criterion 15 / #287: turns with the clock behind the ledger checkpoint | **NOT VERIFIABLE**: blocked at `thread/start`; cause not isolated, most likely libfaketime on macOS | **PASS** for turns | **PASS (Linux only)**. Residual: >90-day unrecorded spend is not disclosed in `/cost` (commented on #287). New defect in `corbanu delete`: #308 |
| Criterion 9 / #286: deleted history | **PASS** (clock faked *ahead*; delete via a retry after a #308 partial delete) | **PASS** (clean delete) | **PASS** |
| Criterion 8 / #288: no-usage requests | **FAIL** | not run (same as the first run) | **FAIL**, unchanged |
| #289 observations (not a criterion) | mostly fixed | mostly fixed | residual wording points, plus a criterion-13 question for the default build, listed below |

Every displayed number that I checked matched an independent recomputation from provider-reported usage, to the
micro-dollar.

Executor: an independent Codex worker, code-blind. One separate reviewer audited this record: installed `corbanu exec`, `claude-plan`,
`claude-opus-5-5-plan`, `high` effort, `-s read-only`, code-blind, on a copy of this directory. Its report is [REVIEW.md](REVIEW.md); the corrections it led to are listed at the end.

## What I read (code-blind boundary)

I read only these sources:

- the first run's directory (README, REVIEW, captures, data, tools)
- `../../acct-acceptance-75/acceptance.md`
- the GitHub issue texts #286–#289
- `/cost help`, which prints the syntax
- `docs/integrations/zai-glm-52.md`
- `sec-common.md`

I did not read `codex-rs/**`, the PR diffs, lane briefs or review files.

## Build provenance

PR #306 was still in CI when I started. I polled `gh pr view 306 --json state` until it reported `MERGED` at about
15:57Z, then built. **#306 is included.**

| | macOS | Linux (RTX box) |
| --- | --- | --- |
| Commit | `origin/main` `b294864916c5e824e0bf5a2c0ee89654ef8ca417` (merge of PR #306; contains #299 and #303) | same, fresh clone |
| Host | macOS 26.6.2 arm64 | Linux 7.0.0-31-generic x86_64 |
| Toolchain | rustc 1.95.0 | rustc 1.95.0 |
| Developer-accounting build | `cargo build --locked -p codex-cli --bin corbanu --features codex-core/developer-accounting,codex-tui/developer-accounting` ([tools/build-mac.sh](tools/build-mac.sh)) | same ([tools/build-rtx.sh](tools/build-rtx.sh) `acct`) |
| sha256 | `9d0c47b4f6af9e945b971de878c31f9ac2a403224177f81cab7f5911fbd3908d` (`corbanu 0.1.48`) | `ef3b0bc40cd644b708d03d6d73562ba72b191560c8391584cc6ae8be52a44ff9` |
| Default build (no feature) | not built | `23d8290c2d5599863a0d3cdbb8dbdc29f490edc1821894ff9ef7fffde1e92e46` ([tools/build-rtx.sh](tools/build-rtx.sh) `default`) |

I could not reuse the first run's RTX directory: it had already been removed. I recreated it at the same path, with a
fresh clone and libfaketime built from source (`wolfcw/libfaketime` `e8d8c85`). Its default target now produces
`libfaketime-time64.so.1`. On macOS I built the same source with `make -f Makefile.OSX`.

## Safety

- Every built binary ran with `CORBANU_TEST_NO_NATIVE_KEYRING=1` and a new disposable `CODEX_HOME`, `CORBANU_HOME` and
  `PFTERMINAL_HOME` under a scratch directory. The real keychain was never used.
- On macOS, the key came only from `ZAI_API_KEY="$(~/.local/bin/corbanu vault auth-helper provider/zai_api_key)"`, run
  with the installed corbanu. That assignment is the first prefix on the consuming command, so the helper runs with the
  caller's real environment ([tools/execrun.sh](tools/execrun.sh), [tools/tui.sh](tools/tui.sh)). This fixes the
  shell-variable deviation the first review noted.
- On Linux, the substitution is attached to `printf`, which pipes the key over ssh stdin
  ([tools/rtxrun.sh](tools/rtxrun.sh)). The remote side reads it into the process environment and never writes it
  ([tools/rtx-execrun.sh](tools/rtx-execrun.sh)).
- View-only TUI sessions and every `delete` used a placeholder key.
- I scanned the committed evidence for the key value with `grep -F` (helper substituted directly on the command): 0
  hits. The same scan over the raw scratch logs and homes (trace stderr, TUI log DBs) on both hosts also found 0 files.
  Those were then deleted.
- **Security observation, #310.** A key supplied as `ZAI_API_KEY` is visible to the model's shell commands
  ([captures/exec-json/mac-envcheck.jsonl](captures/exec-json/mac-envcheck.jsonl), which prints only `PRESENT`). This
  is not accounting-specific. The leaked libfaketime `LD_PRELOAD` (below) shows the same inheritance. Paths and the host are redacted (`<scratch>`, `<rtx-dir>`, `<corbanu-root>`, `<rtx-host>`). No sudo was used.

## Method

The method is the same as in the first run:

- **Real work:** real GLM 5.2 calls on the built-in `zai` provider, against the same shallow clones (`pallets/itsdangerous`
  `672971d`, `sindresorhus/is-plain-obj` `666df7c`, `chalk/ansi-regex` `f923667`).
- **Usage:** provider-reported usage came from the `RUST_LOG=codex_api=trace` SSE `usage` chunk, the TUI log DB, or the
  stdlib pass-through proxy.
- **Prices:** Z.AI's published prices: input 1.40, cached input 0.26, output 4.40 USD per 1M tokens.
- **Tools:** [tools/recompute.py](tools/recompute.py), [tools/buckets.py](tools/buckets.py),
  [tools/extract_usage.py](tools/extract_usage.py) and [tools/usage_proxy.py](tools/usage_proxy.py) are unchanged
  copies: their sha256 equals the first run's `../tools/` files.
- **How independent the channels are.** They are semi-independent, as in the first run.
  - Built-in `zai` usage is the product's own `codex_api=trace` log of the raw provider SSE bytes. Only the 4 proxy
    responses come from outside the product; the 2 pass-through ones match `nu-proxy`'s trace field for field.
  - Day bucketing uses the product process's log timestamps, which libfaketime fakes.
  - **Cross-check:** Z.AI response ids encode the provider's real time (UTC+8). They place every run at the offset I
    claim (reviewer recomputation: `sk58s` −55 to −57 s, `sk6d` −6 d, `sk95d` −95 d, `m1` +0.73 d, `mk` +1.81 d).
- **Screens:** the TUI ran in tmux and was captured by scrolling. Captures are de-duplicated reconstructions, as before.
- **Clock shifts:** libfaketime with `FAKETIME_DONT_FAKE_MONOTONIC=1`.

Every home was configured with `model = "glm-5.2"`, `model_provider = "zai"` ([tools/home-config.toml](tools/home-config.toml)).
Thread ids: [data/threads-rtx.txt](data/threads-rtx.txt), [data/threads-mac.txt](data/threads-mac.txt). Per-run
`exec --json` output: `captures/exec-json/`.

## 1. Criterion 15 / #287: clock behind the ledger checkpoint

### Linux: PASS

The scratch home was `h-skew2`. Runs were in this order on ansi-regex; the prompt was "Read readme.md and summarize…".

| Run | Process clock | Result | Accounting |
| --- | --- | --- | --- |
| `sk0` | real | completed, 2 req | recorded |
| `sk58s` | real − 58 s (behind the checkpoint `sk0` had just written) | **completed** (`turn.completed`; runner exit 0), 4 req | **recorded**, 4 req, 27,748 tok, $0.024372 (`rtx-c15-05`). Admission time is the faked clock, 16:06:51.191Z (`rtx-c15-07`) |
| `sk6d` | real − 6 days | **completed** (`turn.completed`; runner exit 0), 8 req | **recorded on 2026-10-02** (the faked day): 8 req, 57,308 tok, exact USD 0.01963924 (`rtx-c15-02`). Today's page says "no recorded requests" for it (`rtx-c15-01`) |
| `sk95d` | real − 95 days | **completed** (`turn.completed`; runner exit 0), 11 req | **not recorded; reported as an accounting gap** in the turn: "Developer accounting could not record one or more model requests in this turn. They were sent anyway; /cost does not include them…" ([exec-json](captures/exec-json/rtx-sk95d.jsonl)) |
| `sk-after` | real | completed, 2 req | recorded |

- **Anthropic is never named.** `grep -il anthropic` over every Linux `.err` and `.jsonl` of this re-run gives 0 files
  ([data/anthropic-hits.txt](data/anthropic-hits.txt)). The new log lines say "the system clock is behind the
  accounting ledger" ([data/accounting-log-lines-rtx.txt](data/accounting-log-lines-rtx.txt)).
- **Independent totals.**
  - Today, other conversations as seen from `sk6d`: 8 req, 55,484 tok, $0.041550 (`sk0` + `sk58s` + `sk-after`).
    Recomputed: 8 req, 55,484 tok, 0.041550.
  - 2026-10-02: 0.01963924. Recomputed: 0.01963924.
  - [data/recompute-rtx.txt](data/recompute-rtx.txt), [data/buckets-rtx.txt](data/buckets-rtx.txt).
- **Request technical details check out** for `sk58s` request 1 (`rtx-c15-07`): 6,541 × 1.4 + 44 × 4.4 = 0.009351.

Exit codes come from the runner's output and are transcribed in
[data/runner-exit-lines.txt](data/runner-exit-lines.txt); every `.jsonl` ends in `turn.completed`. The log's `behind_ms`
values (57,999, 518,378,007 and 8,207,957,009) all point back to the same checkpoint, 16:07:49.155Z, so the test
condition was really reached.

The "Anthropic" check is weak evidence of absence: no run reached the old failure path, so the old message could not
appear anyway. What it does show is that the new paths do not name Anthropic.

Observations (residuals, commented on #287):

- **The >90-day gap is visible only in the turn.** `/cost` in the `sk95d` conversation says "Today (UTC) in this
  conversation: no recorded requests" (`rtx-c15-03`). The day it ran on, 2026-07-05, is outside detail retention
  (`rtx-c15-04`). Its 11 requests (79,478 tok, $0.028317 at list price) do not appear anywhere in `/cost`, and nothing
  there says a gap exists.
- **The log message does not match the recorded time.** The WARN line says "recording at the ledger's time until it
  catches up", but the attempts are recorded at the faked process time (58 s case) and on the faked day (6-day case).
- **The >90-day cause line reads like test text.** It says `admit attempt: unsupported compact-only late import in this
  fixture`. That is test-fixture wording in a production log.
- **libfaketime leaked into child commands.** `LD_PRELOAD` reached the model's shell commands, which printed libfaketime
  semaphore warnings. The model retried; accounting was not affected.

### macOS: NOT VERIFIABLE (blocked at `thread/start`; cause not isolated, most likely libfaketime on macOS)

With libfaketime on macOS (DYLD interposition), any run whose clock is behind real time timed out at `thread/start`
("in-process app-server request timed out after 30 seconds") before any model call. Probes:

- `ms58s`, with a ledger
- `mftprobe-behind-freshhome`, a fresh home with no ledger, clock −6 days
- a third fresh home with a placeholder key

All three failed this way ([data/mac-libfaketime-behind-probes.txt](data/mac-libfaketime-behind-probes.txt)). A fresh
home with the clock +60 s completed normally (`mftprobe-ahead`). The hang happens without any ledger, which rules out
accounting, but not other date-sensitive startup code. The only control was Homebrew `python3` under the same
interposition with the clock behind; it ran normally. No Rust/tokio control was run. Criterion 15 is therefore verified
on Linux only.

### New defect found on the same path: `corbanu delete` with the clock behind the checkpoint (#308)

On Linux, I ran a real turn (`sk-predel`) and then `corbanu delete --force <sk0>` with the clock 58 s behind
(`rtx-c15-08`). The command printed
`thread/delete failed: failed to delete app-server state …: negative or backward checkpoint (code -32603)` and exited 1.
The session's rollout file was nevertheless removed.

`/cost` still counted `sk0`'s 2 requests among other conversations (`rtx-c15-09`): 5 req, 34,276 tok, $0.024734;
recomputed 0.024734. Retrying the delete at the real clock succeeded (`rtx-c15-10`). The page then showed 3 req,
20,363 tok, $0.012419 (recomputed 0.012419), and "Deleted conversations or subagents sent 2 request attempts on this
day" (`rtx-c15-11`).

A second Linux reproduction (`rtx-c15-12`) used a fresh real turn followed by a delete 58 s behind. It again exited 1
with the rollout gone, and `corbanu resume <id>` then failed with "No saved session found with ID …". So the
conversation is already gone from `resume` while the command reports failure.

macOS showed the same error and exit 1 (`mac-c9-03`). A listing taken before the retry
([data/mac-rollouts-after-failed-delete.txt](data/mac-rollouts-after-failed-delete.txt), transcribed) shows the
deleted sessions' rollouts gone. No pre-delete listing was captured on macOS. So the delete is not atomic, and it fails closed
on the clock-skew condition that #299 fixed for model requests. This is outside the criterion-15 wording, which covers
turns, but it is a store-robustness defect.

## 2. Criterion 9 / #286: deleted history. PASS on both platforms

The steps are the first run's: sessions with a subagent on two days, delete one, then view the day from another
conversation.

**Linux** (`h-del`):

1. `d1` ran with one subagent at 2026-10-06 23:59:40 (faked). All 7 requests started on 10-06.
2. `d2` ran with one subagent at 2026-10-07 12:00 (faked).
3. `dk`, the viewer, ran at the real clock.

| View from `dk` | Before deleting `d1` | After `corbanu delete --force d1` (fresh process) |
| --- | --- | --- |
| `/usage requests 2026-10-06` | other conversations 7 req, 36,646 tok, $0.021341 (`rtx-c9-01`) | "This conversation on 2026-10-06 (UTC): no recorded requests. No other saved conversation has recorded requests on this day. **Deleted conversations or subagents sent 7 request attempts on this day.** Their recorded cost was deleted with them, so it is not included here; any cost they incurred is on your provider's bill." (`rtx-c9-04`) |
| `/usage requests 2026-10-07` | other 7 req, 36,605 tok, $0.013091 (`rtx-c9-02`) | unchanged (`rtx-c9-05`) |

Recomputed: `d1` is 7 req (root + subagent), 36,646 tok, 0.02134052. `d2` is 7 req, 36,605 tok, 0.01309060. The
range view and its 10-06 bucket are scoped to the open conversation and make no day-wide claim (`rtx-c9-06`, `rtx-c9-07`).

**macOS** (`mh-del`). Past days need a clock behind real time, which hangs on macOS, so I used a faked clock **ahead**
of real time for every run and view:

1. `m1` ran with a subagent on 2026-10-09.
2. `m2` ran with a subagent and `m3` without one, both on 2026-10-10.
3. `mk`, the viewer, ran on 2026-10-10 12:00.
4. The TUI views ran at 2026-10-10 12:30–12:50.

| View from `mk` | Before delete | After deleting `m1` and `m2` |
| --- | --- | --- |
| 2026-10-09 (only deleted spend) | other 5 req, 58,919 tok, $0.053777 (`mac-c9-01`) | "No other saved conversation has recorded requests on this day. Deleted conversations or subagents sent **5** request attempts on this day…" (`mac-c9-07`) |
| 2026-10-10 (mixed) | other 8 req in 2 conversations, 98,306 tok, $0.042556 (`mac-c9-02`) | other 2 req, 26,234 tok, $0.008352 (`m3`), plus "Deleted conversations or subagents sent **6** request attempts on this day…" (`mac-c9-08`) |

Recomputed:

| Run | Requests | Tokens | Exact USD |
| --- | --- | --- | --- |
| `m1` (root + subagent) | 5 | 58,919 | 0.05377672 |
| `m2` (root + subagent) | 6 | 72,072 | 0.03420384 |
| `m3` | 2 | 26,234 | 0.00835228 |
| `m2` + `m3` | 8 | 98,306 | 0.04255612 |

The disclosed attempt counts (7, 5, 6) equal the deleted sessions' provider-reported request counts, subagents included.

**Caveat (reviewer):** macOS never shows a clean single delete. The first attempt ran at the real clock, which was
behind the ahead-faked ledger. It failed with "negative or backward checkpoint" (`mac-c9-03`). The rollouts were gone
afterwards ([data/mac-rollouts-after-failed-delete.txt](data/mac-rollouts-after-failed-delete.txt)), and the accounting
rows were still present (`mac-c9-04`, `mac-c9-05`). Rerunning `delete` on the faked clock succeeded (`mac-c9-06`), and
the "after" captures above come from that retry. The Linux run is a clean single delete.

Days with no deleted spend still use the old line "No other conversation recorded requests on this day"
(`rtx-c15-02`, `mac-289-02`). That is correct there.

## 3. Criterion 8 / #288: no-usage requests. FAIL, unchanged

The setup is the same as the first run (`mh-nousage`, [tools/proxy-providers.toml](tools/proxy-providers.toml),
[tools/usage_proxy.py](tools/usage_proxy.py)):

- `zaiproxy` passes usage through.
- `zainousage` strips `usage` from every SSE chunk. The provider did report usage: 2 responses, 20,368 tok
  ([data/proxy-stripped.jsonl](data/proxy-stripped.jsonl)).
- Runs: `nu-zai` (built-in), `nu-proxy` and `nu-nousage`.

**No next step for its own requests.** The `zainousage` conversation's own `/cost` (`mac-c8-01`) says:

- "zainousage · Z.AI GLM 5.2 — Pay per use. 2 requests, tokens not reported. Estimated cost: no price available."
- "Known subtotal exact USD: none — no price for these attempts".

Its only next-step line is "Next step for requests with no price: check the bill from **zaiproxy**", which names
another conversation's provider. Its own provider page (`mac-c8-02`, recaptured in full after the review), request page
(`mac-c8-03`) and technical page (`mac-c8-04`) have no next step. Other conversations' pages also name only `zaiproxy` (`mac-c8-06`).

**Missing usage is not told apart from a missing price** in the cost wording. The request page shows "Tokens: tokens not
reported", but the cost reason is the same as for `zaiproxy`, whose tokens are known: "no price available" and "Token
cost: unavailable — no applicable price" (`mac-c8-04`). The confound from the first run remains: a custom provider has
no price either way, so the label cannot be isolated on a priced provider.

Totals for the other rows match:

| Row | Product | Recomputed |
| --- | --- | --- |
| Z.AI | 2 req, 25,889 tok, $0.024594 | 0.024594 |
| zaiproxy | 2 req, 20,344 tok | 20,344 |

## 4. #289 observations (not a criterion)

| Item | First run | Now | Evidence |
| --- | --- | --- | --- |
| Fresh-home `/cost` before the first request | "ledger not installed. Collection remains off", no next step | **Fixed.** "Unavailable — no requests recorded in this home yet; the accounting ledger is created when the first one is. Next step: send a turn in this conversation, then run /cost again…" After the first TUI turn: 4 req, 59,017 tok, exact 0.04706756, matching the TUI log DB exactly | `mac-289-01`, `mac-289-02` |
| Component lines vs costs | "Noncached input: unknown" next to an exact cost | **Fixed.** "Noncached input …: 13497 (derived as input − cache read…)" with cost 0.0188958 (= 13,497 × 1.4); cache read 1,280 → 0.0003328; output 23 → 0.0001012; subtotal 0.0193298, equal to the provider's. "Cache write: not reported — this price charges nothing for cache writes" next to $0. Day pages now explain "not reported … costed as input − cache read" | `mac-289-03`, `rtx-c15-07`, `mac-289-02` |
| Week and month buckets reaching today | coverage "unavailable" | **Fixed.** Effective coverage is the partial interval `[2026-10-05, ledger-current-to)` / `[2026-10-01, ledger-current-to)` with "Partial bucket — excluded from totals". **Residual:** the page title is "Bucket unavailable", and its next step ("send a turn … then select Refresh") cannot make a bucket that ends in the future whole | `rtx-289-04`…`07` |
| Request numbering | Request 8 was the conversation's first | **Fixed.** Root-only: requests 1–4 are in provider order (0.0189336, 0.00406064, 0.0193298, 0.00474352). With a subagent (`d2`, added after the review): requests 1–7 are $0.002305, 0.002188, 0.001406, 0.001639, 0.001572, 0.001569 and 0.002412, matching the root and subagent requests in first-chunk order | `mac-289-02`, [data/recompute-mac.txt](data/recompute-mac.txt), `rtx-289-08`, [data/request-order-d2.txt](data/request-order-d2.txt) |
| Default Linux build: `/usage requests 2026-10-07` (yesterday) | printed the developer usage line | Opens a "Cost — this conversation" popup: "Unavailable — accounting ledger not installed. Next step: **if this build records costs**, send a turn and run /usage requests again; otherwise check your provider's bill." **Residual:** a default build never records, so this hedge sends the user to send a turn for nothing | `rtx-289-01` |
| Default Linux build: `/usage requests 2026-10-09` (tomorrow) | — | "2026-10-09 is after today (2026-10-08, UTC); there is nothing recorded for it yet. Run /usage requests for today." The same pointless next step: today has nothing recorded either | `rtx-289-02` |
| Default Linux build: `/cost` | unrecognized | still "Unrecognized command '/cost'" | `rtx-289-03` |

**Criterion 13 question (reviewer).** In the default build, `/usage requests <past day>` now opens a full "Cost — this
conversation" page with coverage and retention lines (`rtx-289-01`). In the first run it printed a usage line. The
acceptance record says shipping `/usage requests` "remains held". Whether this page may appear in a default build
needs a criterion-13 (developer-only activation) decision. `/cost` itself is still unrecognized.

There is one more residual wording point. The range header says "oldest daily total kept none", while the day pages
in the same home say "daily totals kept since 2025-10-09" (`rtx-289-04` vs `rtx-c15-02`).

## Spend

| Source | Requests | US$ at list price |
| --- | --- | --- |
| macOS exec (incl. `nu-proxy`, `mftprobe-ahead`) | 22 | 0.181 |
| macOS TUI (fresh home) | 4 | 0.047 |
| no-usage proxy (`nu-nousage`, usage stripped) | 2 | 0.006 |
| Linux exec (incl. `sk-predel2`, added after the review) | 45 | 0.146 |
| macOS `envcheck` (one turn; product-reported usage only, stderr not kept) | 1 turn | 0.022 |
| **Total** | **73 requests + 1 turn** | **≈0.40** |

"ALL" in [data/recompute-mac.txt](data/recompute-mac.txt) (24 requests, 0.186618) is the macOS exec rows plus the
stripped proxy log; the TUI turn is the second block in that file.

## Incidents (harness, not product)

1. **First Linux runs without a home config.** `sk0` and `sk58s` in `h-skew` defaulted to the OpenAI provider and failed
   with 401 before any model call. No key was sent there; `ZAI_API_KEY` is not used by that provider. I moved them to
   `logs/aborted/`, added [tools/home-config.toml](tools/home-config.toml) and restarted in a new home `h-skew2`.
2. **The mac build was killed twice.** The background job died with its parent shell. It was rerun in tmux; no source
   changed.
3. **macOS libfaketime cannot set a clock behind real time** (see criterion 15).
4. **The `/usage requests` capture in the default build was retaken.** The first attempt typed three commands into an
   open popup and was discarded.
5. **A first attempt at the second #308 reproduction made no turn.** It used the redacted `tools/rtxrun.sh`, so no
   checkpoint was written and the clock was not behind. The delete of `sk-after` therefore succeeded cleanly. That
   capture was discarded, and `rtx-c15-12` was redone with a real turn first.

## Commands (exact, redacted)

```sh
# wait for #306, then build (both hosts)
gh pr view 306 --json state        # polled until MERGED
cargo build --locked -p codex-cli --bin corbanu --features codex-core/developer-accounting,codex-tui/developer-accounting
cargo build --locked -p codex-cli --bin corbanu        # Linux default build

# exec runs: tools/execrun.sh (macOS) and tools/rtxrun.sh -> tools/rtx-execrun.sh (Linux)
#   <home> <cwd> <tag> <faketime|-> <prompt> [-c 'model_provider="zaiproxy"']
# Linux clock-behind:
TZ=UTC FAKETIME_DONT_FAKE_MONOTONIC=1 FAKETIME="@<now-58s | now-6d | now-95d>" LD_PRELOAD=libfaketime-time64.so.1 corbanu exec ...
# macOS clock-ahead (deletion runs, TUI views): prefix assignments on the binary, no /usr/bin/env
TZ=UTC FAKETIME_DONT_FAKE_MONOTONIC=1 FAKETIME="@2026-10-10 12:00:00" DYLD_FORCE_FLAT_NAMESPACE=1 DYLD_INSERT_LIBRARIES=libfaketime.1.dylib corbanu ...

# deletion
corbanu delete --force <session-uuid>
# TUI (tmux socket pf60rerun): tools/rtxview.sh, tools/macview.sh + tools/tui.sh | tools/tui-ft.sh; tools/cmdcap.sh, tools/select.sh, tools/pagecap.sh
/cost   /cost help   /usage requests 2026-10-02   /usage requests 2026-07-05   /usage requests 2026-10-06
/usage requests 2026-10-07   /usage requests 2026-10-09   /usage requests 2026-10-06 2026-10-09 day
/usage requests 2026-10-05 2026-10-12 week   /usage requests 2026-10-01 2026-11-01 month

# recomputation
python3 tools/recompute.py --err <exec.err>... [--proxy proxy.jsonl] [--logdb pfterminal_logs_2.sqlite] --group name=tag,...
python3 tools/buckets.py day <exec.err>
```

The scripts in `tools/` are redacted copies. Replace `<scratch>`, `<rtx-dir>`, `<corbanu-root>` and `<rtx-host>` before
running them.

## For Travis

- **#286 and #287 (model-request path) are fixed on real runs.** #289 is mostly fixed.
- **#288 is not fixed.** A no-usage conversation still gets no next step for its own requests, and still reads as a
  missing price.
- **New #308.** `corbanu delete --force` with the clock behind the ledger checkpoint (58 s is enough) exits 1 but has
  already removed the session's rollout: `resume` says "No saved session found", while its spend stays attributed to it.
  A retry at the correct clock succeeds. Recommendation: either apply the same clock tolerance as #299, or make
  deletion all-or-nothing.
- **A >90-day-behind turn is reported only in the turn output.** `/cost` gives no hint that requests went unrecorded.
  The log's "in this fixture" wording should be cleaned up. Both are noted on #287.
- **Decision needed (criterion 13).** In a default build, `/usage requests <past day>` now opens an accounting page
  instead of printing a usage line.
- **Security, #310.** A provider key supplied via `ZAI_API_KEY` is readable by the model's shell commands.
- **Criterion 15 on macOS remains unverified.** libfaketime could not run a process behind real time there. A VM with a
  settable clock would be needed.

## Independent evidence review

One reviewer audited this record: installed `corbanu exec`, `claude-plan`, `claude-opus-5-5-plan`, `high` effort,
`-s read-only`. It was code-blind and saw only a copy of this directory, the first run's README and REVIEW, the
acceptance request, the Z.AI doc and the issue texts. Its report is [REVIEW.md](REVIEW.md), verbatim apart from path
redaction.

**Conclusion:** "Supported with corrections". It found nothing blocking and recomputed every headline number with no
discrepancy.

**Corrections applied** (numbers are the review's findings):

1. Corrected the reviewer model line. The `../README.md` link is valid in the repository layout; the reviewer saw a
   flattened copy.
2. **#308 evidence.** Added a second Linux reproduction with before/after listings and the `resume` failure
   (`rtx-c15-12`). Added the macOS post-failure listing ([data/mac-rollouts-after-failed-delete.txt](data/mac-rollouts-after-failed-delete.txt)) and disclosed that no macOS pre-delete listing exists.
3. The macOS criterion 9 PASS is now qualified as obtained via a retry after a partial delete.
4. macOS criterion 15 is reworded to "blocked at `thread/start`; cause not isolated", with the python control disclosed.
5. Criterion 15's residuals are now in the verdict row and noted on #287. "Exit 0" is backed by
   [data/runner-exit-lines.txt](data/runner-exit-lines.txt).
6. Retested request numbering in a subagent conversation (`rtx-289-08`, [data/request-order-d2.txt](data/request-order-d2.txt)): in order.
7. The default-build page is flagged as a criterion-13 question, and the `rtx-289-02` next step is noted.
8. Redacted `~/corbanu-rtx/…` in the three default-build captures.
9. Recaptured the provider page `mac-c8-02` in full; it has no next step.
10. Disclosed the semi-independent channels and the response-id time cross-check.
11. Added the scanned-file list to [data/anthropic-hits.txt](data/anthropic-hits.txt), build receipts
    ([data/build-receipts.txt](data/build-receipts.txt)) and the tools' hash equality. The Anthropic check is labelled
    weak evidence.
12. Scanned the raw scratch logs on both hosts for the key: 0 files. Tested environment inheritance: the key is visible
    to tool commands (#310).
13. Added the command line to `rtx-c9-03`, and a note on the old wording for days without deletions.

The `rtx-c15-12` reproduction, the `d2` numbering capture, the `mac-c8-02` recapture and the `envcheck` run were made
after the review, so the reviewer did not see them.
