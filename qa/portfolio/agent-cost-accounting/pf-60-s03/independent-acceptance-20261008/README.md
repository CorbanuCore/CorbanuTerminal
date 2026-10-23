# PF-60-S03 independent functional acceptance (2026-10-08)

**Result:** S03 is **not accepted**. The inspector's arithmetic and attribution are exact. Every displayed total on macOS
and Linux matched an independent recomputation from provider-reported usage at Z.AI's published prices, to the
micro-dollar. That included subagents, concurrent `corbanu exec` runs, restart and day/hour/ISO-week/month boundaries.
Three criteria fail on real runs:

- **Deleted history:** after a session is deleted, the day screen states "No other conversation recorded requests on this
  day" for days on which that session and its subagents really spent money. Excluding deleted spend may be a product
  choice; asserting its absence is the defect. Issue [#286].
- **No-usage:** a no-usage request's own page gives no next step. Its only next-step line names a different provider.
  Issue [#288].
- **Store robustness:** with developer accounting on, while the clock is behind the ledger checkpoint (58 s and 6 days
  tested), the turn fails before any request is sent. The error names Anthropic for a Z.AI request. It recovered once the
  clock passed the checkpoint. This is probably inherited S02 fail-closed behaviour. Issue [#287].

Lower-severity wording issues are in [#289]. The `exec --json` usage gaps in [#290] are recorded as an out-of-scope
observation, not an S03 FAIL. An independent reviewer (Opus 5.5 high, code-blind, read-only) audited this record; its
report is [REVIEW.md](REVIEW.md), and the corrections it prompted are listed at the end.

Executor: independent Codex worker, code-blind. Decision `acct-s03-acceptance-20260917`, option 2.

## What I read (code-blind boundary)

I read only these sources:

- `../acct-acceptance-75/acceptance.md`
- the sprint record `docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md`
- the plan's product-intent and activation sections
- `docs/integrations/zai-glm-52.md`
- the two `developer-accounting` feature comments in `codex-rs/core/Cargo.toml` and `codex-rs/tui/Cargo.toml`
- the documented build script (`integration-build-20260922a/dev-build.sh`)
- `sec-common.md`
- the in-product help: `/cost help` prints the `/usage requests` syntax

I did not read the S03 implementation source, lane briefs, worker returns or prior review files.

## Build provenance

| | macOS | Linux (RTX box) |
| --- | --- | --- |
| Commit | `origin/main` `63ea3d0cbd0ccb54d352b25a12c3e07f9f934bff` (merge of PR #278) | same, fresh clone |
| Host | macOS 26.6.2 arm64 | Linux 7.0.0-31-generic x86_64 |
| Toolchain | rustc 1.95.0 | rustc 1.95.0 |
| Build | `cargo build --locked -j8 -p codex-cli --bin corbanu --features codex-core/developer-accounting,codex-tui/developer-accounting` ([tools/build-mac.sh](tools/build-mac.sh)) | same ([tools/build-rtx.sh](tools/build-rtx.sh)) |
| Binary sha256 | `a303aa4871ba2a983fc95cc454ee290afe7c1069abe53e261efeea5d8e7b1c74` (`corbanu 0.1.48`) | `e3f94501a2ee4a4df4be97467b151cf9bc4c1db89f751e9a3e4be76d86ebb41a` |
| Default build (no feature) | not built | `fb26f8cb7bc296f68f765f4a1737985bc4ffff9206e939695b863195ee58cef9` |

## Safety

- Every run of a built binary had `CORBANU_TEST_NO_NATIVE_KEYRING=1` and a disposable `CODEX_HOME`, `CORBANU_HOME` and
  `PFTERMINAL_HOME` under scratch directories.
- The real keychain was never used by a built binary. The installed signed `corbanu vault auth-helper provider/zai_api_key`
  ran only inside a quoted command substitution, with the disposable-home variables unset.
- On the RTX box the key was piped over ssh stdin to each run and never written to disk. View-only TUI sessions there used
  a placeholder key.
- The evidence was scanned for the key with `grep -F` against the helper's value: 0 hits. Paths and host are redacted
  (`<scratch>`, `~`, `<rtx-host>`). No sudo was used.

## Method

### Real work

Three public repositories, cloned shallow:

- `pallets/itsdangerous` `672971d6`
- `sindresorhus/is-plain-obj` `666df7c1`
- `chalk/ansi-regex` `f923667b`

All model calls were GLM 5.2 on the built-in `zai` provider unless noted. That covered:

- an interactive TUI session that spawned two subagents
- concurrent `corbanu exec --json` runs: three on macOS, one with a subagent; four on Linux, two with subagents
- custom providers for the missing-price and no-usage cases

Sessions and thread ids are in [data/threads-mac.txt](data/threads-mac.txt) and [data/threads-rtx.txt](data/threads-rtx.txt).

### Provider-reported usage, independent of the inspector

- **Built-in `zai`:** the raw Z.AI SSE chunk carrying `usage` (with the provider response id) was logged before parsing
  via `RUST_LOG=codex_api=trace`. Source: exec stderr, or the TUI log DB.
- **Custom providers:** a stdlib pass-through proxy ([tools/usage_proxy.py](tools/usage_proxy.py)) captured the wire.
  For the `zaiproxy` run, the trace channel's parsed `usage` objects matched the proxy's field-for-field on both responses (same provider ids), which validates the trace
  channel.
- **No-usage case:** the proxy stripped `usage` before the product saw it. The provider did report it
  ([data/proxy-stripped.jsonl](data/proxy-stripped.jsonl)).
- **How independent these channels are:** they are semi-independent.
  - The built-in channel is the product's own trace of raw provider bytes, validated against the external proxy on 2
    responses.
  - Request start times use the product's own `ms_to_first_sse_byte` metric.
  - Per-thread splits for k1 and k3 use the session rollouts (`token_count`), a different subsystem from the accounting
    ledger. Their per-thread sums equal the provider-reported totals.
  - For the mac TUI conversation, the 4-request child total of 0.04031996 is inferred: it is the only 4-request subset of
    the provider-reported requests with that sum, and its timing fits. No thread id is retained in that log source.

Extracts: [data/usage-mac.jsonl](data/usage-mac.jsonl), [data/usage-rtx.jsonl](data/usage-rtx.jsonl) and
[tools/extract_usage.py](tools/extract_usage.py).

### Prices

I looked these up on docs.z.ai/guides/overview/pricing on 2026-10-08. GLM-5.2, USD per 1M tokens:

| Input | Cached input | Output |
| --- | --- | --- |
| 1.40 | 0.26 | 4.40 |

Cost = (prompt − cached) × input + cached × cached + completion × output. Reasoning tokens are included in completion.
These rates match the rates the product shows in "Technical details" (1.4 / 0.26 / 4.4).

### Recomputation

- [tools/recompute.py](tools/recompute.py) dedups by provider response id and computes exact decimal totals.
- [tools/buckets.py](tools/buckets.py) buckets by request start. Request start is first-chunk time minus
  `ms_to_first_sse_byte` from the per-call metrics line.
- It flags requests whose start and usage chunk fall in different buckets.

### Day boundaries with real calls

`libfaketime` (built from source, no sudo) shifted the process wall clock to just before the following UTC boundaries:

- 2026-09-30/10-01 (month)
- 2026-10-04/05 (ISO week W40/W41)
- 2026-10-06/07 (day, with a subagent)

These were real Z.AI calls. `FAKETIME_DONT_FAKE_MONOTONIC=1` was required; without it two runs hung and were killed
(see Incidents). The views were then read at the real time.

### Screens

The TUI ran in tmux, scrolled line by line and captured as text. All captures are in `captures/mac/` and `captures/rtx/`.
Captures are reconstructed from scroll frames with identical lines de-duplicated (`awk '!seen[$0]++'`). They are not
verbatim: a repeated line, such as an identical request row, would appear once. Some long lists are truncated where
the step count ran out (`mac-30` stops at Request 4 of 8). Numeric checks used the first screen or the `Known subtotal`
lines, which are unique.
[tools/](tools/) holds the helpers and exact launch scripts.

## Reconciliation (product vs independent)

| View | Product (`/cost` / `/usage requests`) | Independent | Match |
| --- | --- | --- | --- |
| mac TUI conversation, 2 subagents, 2026-10-08 | 8 req, 107,759 tok, exact 0.08881468; root 4 / descendants 4 = 0.04031996 | 8 req, 107,759 tok, 0.08881468; children 0.04031996 | exact |
| …same, each of 8 request rows | 0.006395 … 0.020484 | same eight values | exact |
| …same, request 8 components | in 14,188 / cached 0 / out 141; 0.0198632 + 0.0006204 = 0.0204836 | `20261008184519b0…`: 14,188 / 0 / 141 → 0.0204836 | exact |
| …same, other conversations | 10 req, 121,450 tok, $0.098936 | probe + c1 + c2 + c3 = 10 req, 121,450, 0.098936 | exact |
| mac exec c3 (subagent, run concurrently with c1, c2) | 5 req, 58,410 tok, 0.04161648; root 3 / desc 2 | 5 req, 58,410, 0.041616 | exact |
| mac after restart (resume) | unchanged totals | — | same |
| mac hour bucket [10:00Z, 11:00Z) | 8 req, 0.08881468 | 8 req, 0.08881468 | exact |
| mac custom `zaiproxy` | 2 req, 20,324 tok, "no price available"; others "at least $…" | 2 req, 20,324 (proxy) | tokens exact; no invented price |
| RTX `/usage requests 2026-10-06` (day boundary, subagent) | 7 req, 34,359 tok, 0.01268184 | 7 req, 34,359, 0.01268184 (by start) | exact |
| RTX 2026-10-07 for same conversation | 8 req, 43,805 tok, 0.01665244 | 8 req, 43,805, 0.01665244 | exact |
| RTX straddling request | admission 2026-10-06T23:59:59.686Z, bucketed 10-06 | start ≈ 23:59:59.681Z, usage chunk 00:00:02.644Z | consistent |
| RTX range 10-06..10-08 by day / hour across midnight | $0.029334; hour buckets 7 / 8 | 0.02933428; 7 / 8 | exact |
| RTX month Sept 2026 (boundary request start ≈ 23:59:57.97Z, independent estimate) | 4 req, 27,258 tok, 0.01496616 | 4 req, 27,258, 0.01496616 | exact |
| RTX ISO week W40 (boundary request start ≈ Sunday 23:59:56.9Z, independent estimate) | 3 req, 20,617 tok, 0.00968756 | 3 req, 20,617, 0.00968756 | exact |
| RTX 2026-10-01 day view (month boundary, other side) | 34 req, 220,862 tok, $0.082922 (`rtx-26`) | 34 req, 220,862, 0.08292244 | exact; boundary request not double counted |
| RTX 2026-10-05 (ISO W41 side of the week boundary) | no recorded requests (`rtx-27`, a truly empty day with collection on) | 0 (boundary request started 10-04) | consistent |
| RTX killed run `r-mid1` (SIGTERM during `spawn_agent`) | 1 req, 6,647 tok, 0.00356308; no phantom attempts (`rtx-36`) | 1 req, 0.00356308 ([data/r-mid1-killed-run-usage.jsonl](data/r-mid1-killed-run-usage.jsonl)) | exact |
| RTX failed turn (clock skew, nothing sent) | "no recorded requests" for that conversation (`rtx-37`) | no request reached the provider | consistent |
| RTX 4 concurrent exec runs, today, viewed from k4 | own 2 req, 0.01034596; others 39 req, 398,333 tok, $0.203782 | k4 0.010346; k1 + k2 + k3 39 req, 398,333, 0.203782 | exact |
| RTX k1 (2 subagents) root / descendants | 4 / 4: 0.0166426 / 0.0159556 | per-thread `token_count` from the session rollouts, not the accounting ledger ([data/rtx-thread-usage-k1-k3.jsonl](data/rtx-thread-usage-k1-k3.jsonl)), sums equal provider-reported totals: 0.01664260 / 0.01595560 | exact |
| RTX k3 (1 subagent) root / descendants | 5 / 24: 0.01930652 / 0.14179384 | 0.01930652 / 0.14179384 | exact |

Full listings: [data/recompute-mac.txt](data/recompute-mac.txt), [data/recompute-rtx.txt](data/recompute-rtx.txt) and
[data/recompute-rtx-buckets.txt](data/recompute-rtx-buckets.txt).

Total list-price spend of this QA was about US$0.65:

| Source | Requests | US$ |
| --- | --- | --- |
| macOS | 22 | 0.236 |
| no-usage proxy | 2 | 0.017 |
| Linux | 114 | 0.399 |

## Verdicts

| # | Criterion (source) | Verdict | Evidence |
| --- | --- | --- | --- |
| 1 | User can explain each displayed total from constituent requests without inspecting storage (PF-60 acceptance) | **PASS** | Conversation, provider, request and attempt pages show tokens, components, rates, price source and exact USD; all reconcile (table above). Day-wide statements after deletion fail under 9. |
| 2 | Inspect one run and its descendants; root = own + resolved descendants (plan outcome; S03-B) | **PASS** | mac TUI, mac c3, RTX k1/k3/b-day |
| 3 | Distinguish measured tokens, estimated cost, billed cost, Corbanu API balance (plan outcome) | tokens/estimate **PASS**; billed and balance **NOT VERIFIABLE** | Every page says "Billed cost: unavailable — no settlement evidence"; there is no settlement seam. Corbanu Plan/API balance does not appear in `/cost` and was not exercised (Z.AI only). |
| 4 | Reopen the same totals after restart (plan outcome) | **PASS** | `mac-01` vs first view; every RTX view is a fresh process |
| 5 | User-selected ranges: hour/day/ISO-week/month on UTC; partial buckets labelled and never totalled; hour refused outside raw-detail window; future day refused (S03-C) | **PASS** (P3 wording in #289) | `mac-20..27`, `rtx-12..14`, `rtx-20..25`. Displayed TZ was Phoenix on RTX, all output UTC. A week/month bucket reaching the future shows coverage "unavailable" rather than the partial interval. |
| 6 | Empty day vs unavailable distinguished | **PASS** (P3 wording in #289) | Empty: `mac-10`. Before retention: `mac-11`, `mac-26`. Deleted conversation: `rtx-34/35`. Fresh store: `mac-40`, but "Collection remains off" is wrong (`mac-41`). |
| 7 | Missing price never shown as free; next step to the provider's bill (`acct-scope-62`) | **PASS** for a priced-tokens custom provider | `mac-03`, `mac-05`: "no price available", "at least $…", "check the bill from zaiproxy" |
| 8 | No-usage requests | **FAIL** | `mac-04`: the conversation's own page gives no next step for its unpriced requests; the only next-step line names another provider (`zaiproxy`). This contradicts the sprint claim "every page showing an unpriced request now carries the next step". The "no price available" label cannot be isolated here, because `zainousage` is a custom provider with no price either way; a no-usage request on a priced built-in provider was not achievable, since built-ins cannot be re-pointed. #288 |
| 9 | Deleted history | **FAIL** | `rtx-30`, `rtx-32`, `rtx-33`: after deletion, the day screen states "No other conversation recorded requests on this day" for days with 7–18 real requests (incl. subagents). `mac-07`: the drop happens with no disclosure. The defect is the false statement; whether to keep deleted spend is a product decision. #286 |
| 10 | Scope: an empty or partial conversation is not presented as a day-wide zero (`acct-scope-62` fix) | **PASS** (except after deletion, see 9) | `rtx-31`: a conversation with no requests that day says so for itself and lists the other conversation's 7 requests and $0.012682. `mac-10`: the empty-day statement "No other conversation recorded requests on this day" is true there. |
| 11 | Concurrent runs and subagents: no loss, no double count | **PASS** | mac 3 concurrent + TUI; RTX 4 concurrent; per-request provider ids reconcile 1:1 |
| 12 | Narrow screen, mixed provider | **PASS** | `mac-30` (60 columns, wraps, nothing truncated); `mac-03` (zai, zaiproxy, zainousage listed separately) |
| 13 | Developer-only activation: `/cost` absent from a default build | **PASS** (Linux only) | `rtx-50/51`: "Unrecognized command '/cost'". `rtx-52`: `/usage requests` prints the developer syntax (#289). |
| 14 | Historical sessions remain inspectable | **PASS** within retention | Past days 09-30 … 10-07 via resume; 90-day detail window enforced (`mac-27`) |
| 15 | Accounting store failure handling (sprint: "unavailable-backend" states; executor's inference that collection must not stop work) | **FAIL** | `data/r-skew60s*`, `data/r-month-backward-errors.txt`: two turns with the clock behind the checkpoint (58 s, 6 days) failed before sending. `data/r-recover-turn-completed.jsonl`: the same home worked once the clock was ahead again. Likely inherited S02 fail-closed design. #287 |
| 16 | CLI/JSON outputs | **OBSERVATION (out of S03 scope; not counted as a FAIL)** | `captures/exec-json/`: `turn.completed.usage` is zeros when usage is unknown (`mac-nousage`) and excludes subagents (`mac-c3`: 37,933 input vs 58,128). #290 |
| 17 | Platforms macOS + Linux; live repositories; real keys | **PASS for macOS and Linux**; Windows **not run** | both hosts; 3 public repos; real Z.AI key |
| 18 | Estimate vs provider's actual bill | **NOT VERIFIABLE** | No access to the Z.AI billing console; recomputed from provider-reported usage and the published price instead. |

## Not exercised (open gaps, not verdicts)

- Retry predecessors: always "none" in these runs.
- Non-empty "unknown parent" and "unknown provider/model attribution" populations: always "none".
- The busy-host / work-budget refusal (TooLarge).
- Stale-estimate and current-command states.
- Cancel by user (Esc / Ctrl-C mid-turn) in the TUI. Only a SIGTERM-killed exec (`rtx-36`) and a failed turn (`rtx-37`)
  were inspected.
- The no-usage label on a priced provider.
- Week/month boundaries on the *later* side in a bucket view. W41 and October buckets reach the future and are withheld;
  the later-side day views `rtx-26` and `rtx-27` stand in.
- Old history: all "historical" rows are hours old on a faked clock. The 90-day hour refusal (`mac-25`, `mac-27`) was
  checked only on an empty period.
- macOS default build; Windows.

## Discrepancies and observations (not counted above)

- No arithmetic discrepancy was found anywhere.
- "Snapshot is not current; newer activity is unverified" appears on an idle ledger. "Ledger current to" is the last
  write, so a past bucket only becomes whole after a later write. This is conservative, but on a quiet day an hour stays
  "partial" until the next request.
- The scope disclosure in range views ("Other conversations are not included in this view") is clear. The range view does
  not show request or token counts at the range level, only the rounded estimate; exact values are one level down.
- I ran the acceptance request's retained-evidence verifier (`acct-inventory-79/verify_acceptance.py`) on this
  checkout, with and without this directory present. Both runs printed `BASELINE MATCH` and
  `RESULT agreement=20 disagreement=0 unavailable=3 exit=2`, as documented. It checks saved evidence, not function.

## Incidents in this run (harness, not product)

1. **Accidental fork (mine).** On macOS my capture helper pressed Esc twice on an empty composer, which opened the
   backtrack editor. The next typed command forked conversation `mac-custom` with no model call. I fixed the helper so it
   never double-presses Esc. The `/cost 2026-10-07` capture (`mac-13`) was retaken and is therefore from the fork.
2. **Hung runs.** Two libfaketime runs without `FAKETIME_DONT_FAKE_MONOTONIC=1` hung at start or at `spawn_agent` and were
   killed. One of them (`r-mid1`) had made 1 request; it is excluded from the tables.
3. **Backward clock on the first home.** The out-of-order fake-clock runs on that home exposed #287. Boundary data then
   came from a fresh home (`h-rtx2`) with runs in chronological order.

## Commands (exact, redacted)

```sh
# build (both hosts)
cargo build --locked -j8 -p codex-cli --bin corbanu --features codex-core/developer-accounting,codex-tui/developer-accounting
# every built-binary run
CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$H CORBANU_HOME=$H PFTERMINAL_HOME=$H \
  ZAI_API_KEY="$(corbanu vault auth-helper provider/zai_api_key)" RUST_LOG=codex_api=trace,warn \
  corbanu exec --json --skip-git-repo-check -s read-only [-m glm-5.2 -c 'model_provider="zaiproxy"'] "<prompt>"
# boundary runs (Linux)
TZ=UTC FAKETIME_DONT_FAKE_MONOTONIC=1 FAKETIME="@2026-10-06 23:59:45" LD_PRELOAD=libfaketime.so.1 corbanu exec ...
# TUI views (tmux)
/cost    /cost 2026-10-07    /usage requests 2025-01-01    /usage requests 2026-10-09
/usage requests 2026-10-07 2026-10-09 day
/usage requests 2026-10-06T23:00:00Z 2026-10-07T01:00:00Z hour
/usage requests 2026-09-28 2026-10-12 week
/usage requests 2026-09-01 2026-10-01 month
/usage requests 2026-06-01T00:00:00Z 2026-06-01T02:00:00Z hour
# deletion
corbanu delete --force <session-uuid>
# independent recomputation
python3 tools/recompute.py --err <exec.err>... --logdb <tui logs.sqlite> --proxy <proxy.jsonl>
python3 tools/buckets.py day|hour|week|month <exec.err>...
```

The per-run prompts and arguments are in [tools/execrun.sh](tools/execrun.sh) and
[tools/rtx-execrun.sh](tools/rtx-execrun.sh). The literal per-session prompts are the first `item` events in
`captures/exec-json/`.

## For Travis

- The arithmetic and attribution are sound and reproducible on real work. The blockers are behavioural:
  - **#286 (deletion):** a product decision is needed. Either keep deleted sessions' attempts as an anonymous
    "deleted conversations" population, or keep deleting them and say so on every day/range page. Recommendation: keep an
    anonymous population, because spend did happen.
  - **#287 (accounting blocks inference on clock skew):** must be fixed before any wider developer activation.
  - **#288 (no-usage label and next step):** already partly known.
- Still not verifiable: billed cost and Corbanu balance (no settlement seam), comparison with a real provider bill,
  Windows, and a macOS default build.

## Independent evidence review

One reviewer audited this record: installed `corbanu exec`, `claude-plan`, `claude-opus-5-5-plan`, `high`, `-s read-only`.
It saw only a copy of this directory plus the acceptance request, sprint record and Z.AI doc, and was told not to read
source. Its report is [REVIEW.md](REVIEW.md), verbatim apart from path redaction.

**Conclusion:** "Supported, with corrections". It independently recomputed every headline number with no discrepancy.

**Corrections applied here:**

- Row 16 is now an out-of-scope observation; the FAIL count is three.
- #287's scope is narrowed: recovery was tested and passes, and the behaviour is likely inherited from S02.
- The deletion FAIL is reframed around the false statement.
- The no-usage confound is disclosed.
- Semi-independent channels, inferred attribution and de-duplicated captures are disclosed.
- "Admitted" is replaced by "start ≈" for my own estimates.
- Added: later-side boundary day views, a truly empty day with collection on, and the killed-run and failed-turn views.
- Added the "Not exercised" list.
- Listed the TUI root thread and `mac-c5`.
- Kept the verifier output ([data/verify_acceptance-output.txt](data/verify_acceptance-output.txt)).

**Not changed:**

- Row 13 stays PASS. The typed command (`/usage requests 2026-10-07`) is visible in `rtx-52` above the usage line.
- The reviewer noted `tools/execrun.sh` holds the key in a shell variable `K` within the wrapper before passing it to the
  consuming process, rather than substituting it directly on the consuming command. The value was never printed or
  written (key scan: 0 hits). I record this as a deviation from the vault-helper usage rule.

[#286]: https://github.com/CorbanuCore/CorbanuTerminal/issues/286
[#287]: https://github.com/CorbanuCore/CorbanuTerminal/issues/287
[#288]: https://github.com/CorbanuCore/CorbanuTerminal/issues/288
[#289]: https://github.com/CorbanuCore/CorbanuTerminal/issues/289
[#290]: https://github.com/CorbanuCore/CorbanuTerminal/issues/290
