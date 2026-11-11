# acc-tart1: PF-60 follow-up isolated live run on Tart (2026-10-20)

The follow-up live run that the [accounting follow-ups plan](../../../../docs/plans/completed/main-2026-10-20-portfolio-agent-cost-accounting-followups.md)
waits on. Owner decisions (Travis):
- D25 (2026-10-16): re-check PF-60-S10's gaps in [#898](https://github.com/CorbanuCore/CorbanuTerminal/issues/898)'s
  acceptance run: ACC-06 `/review` cost, the Request IDs of ACC-11/14/18, and ACC-15's `side:` screen.
- D3 (2026-10-16), raised by D54 (2026-10-20, option a: about $0.05-0.06, at most 4 Moonshot requests): a real Moonshot
  cache read.
- D16 (2026-10-16): one real OpenAI realtime session, under $0.05.
- D34 (2026-10-19): run on Tart.

The method is PF-60-S10's
([followups-acceptance-20261016](../followups-acceptance-20261016/README.md)), inside the
[isolated execution gate](../../../code-blind-functional/isolated-execution.md). **Not accepted: this run needs Travis's
acceptance.**

## Result

25 cases: **18 passed, 2 failed, 5 blocked** ([dispositions.md](dispositions.md), [results.json](results.json)).

- **D25, all three S10 gaps closed:**
  - ACC-06 / #898 (FUP-03, passed): after `/review`, the 4 review requests are listed under the reviewing
    conversation with `Turn: code-review:`, and the exact sum of that conversation's 5 requests equals the product's
    subtotal.
  - ACC-15 (FUP-05, passed): the `/side` turn's technical page, with `Turn: side:…`, is on a captured screen.
  - ACC-11/14/18 (FUP-10, passed): Request IDs read before later reopens (R1–R3 before four more reopens, R10) were
    re-read unchanged in the final audit, which lists all 14 requests once.
- **D16, realtime (FUP-21–23, passed):** one real `gpt-realtime-1.5` session is listed as
  `Realtime: 1 session; token counts and cost unknown, not zero`, never as $0. It has its own request page, start and
  end times and a `realtime:ws:` technical page, and it is unchanged after a restart. The session itself cost about
  $0.005. The harness's realtime path also paid for app-server text warm-ups, and these may push the D16 total past
  $0.05 (see Spend).
- **D3, Moonshot cache read (FUP-17–19 blocked, FUP-20 passed):** real cache reads happened on both Kimi models. The
  conversation pages show `Cache read: 11,776` (K3) and `11,264` (K2.7 Code), and each conversation's exact subtotal
  reconciles at Moonshot's published rates. The three cases stay blocked because the executor never reached the
  per-request technical pages, so the per-request rates are on no screen. This is a navigation miss: the raw terminal output (`pty.raw`, the
  integrator's read) shows the `→ Technical details (attempt 1)` link was rendered on K3 Request 1's page (the only
  request page where the executor scrolled). The relay's
  log of Moonshot's own reported usage reproduces every request page's cost exactly; that is the integrator's
  supplementary read, not the basis of any disposition.
- **Failed (2):**
  - FUP-14 (blocker): editing the *first* message continues in a new conversation whose warm-up is an ordinary
    new-conversation warm-up (ledger `prewarm:`). It is not labelled as a reopen warm-up and no notice is shown.
    Filed as [#1038](https://github.com/CorbanuCore/CorbanuTerminal/issues/1038). The issue also says that
    `skip_reopen_warmup` would not skip this warm-up and gives the cause; both come from the integrator's reading of
    the code and were not tested. Editing a later message passed in S10 (ACC-16). The request is recorded and counted
    once.
  - FUP-06 (advisory): `/review` inside `/side` correctly sends nothing. The product writes its refusal, then clears
    the screen at once; for 6 minutes only the `■` glyph stayed visible. Filed as
    [#1039](https://github.com/CorbanuCore/CorbanuTerminal/issues/1039).
- **Blocked (5):**
  - FUP-17, FUP-18 and FUP-19 (above).
  - FUP-15 (advisory, `/resume` onto the conversation already open): not exercised, because the executor opened an
    abandoned fork instead of the open conversation.
  - FUP-02 (advisory): for 2026-01-01 the product says the day's request detail is unavailable (outside its
    request-detail history, though the same screen says daily totals are kept since 2025-10-21) and shows no total, so the "no requests recorded" branch was not reached.
- **Also passed:** basic `/cost` pages (FUP-01); review and `/side` attribution kept apart (FUP-04);
  history after a restart (FUP-07); no warm-up on Z.AI (FUP-08); OpenAI reopen warm-ups labelled and announced on
  restart, `resume <id>`, `/resume` and `/side` (FUP-09, 11–13); `skip_reopen_warmup` by flag and by `config.toml`
  (FUP-16, 25); Kimi conversations kept apart (FUP-20); realtime seen from another conversation (FUP-22); no
  credential printed (FUP-24).
- Raw executor verdicts differ in three cases, FUP-02 (passed -> blocked), FUP-10 (failed -> passed) and FUP-15
  (failed -> blocked); FUP-06 keeps the executor's failed verdict for another reason. Details are in [dispositions.md](dispositions.md).
- `qa/code-blind-functional/check.py` stops at the first unresolved case, FUP-02 ([checker-output.txt](checker-output.txt)).

## Observations (not case failures)

- The realtime harness started its app-server thread without naming a model, so that thread's startup warm-up used the
  default `gpt-6-sol`: 14,645 tokens, $0.029290. That is more than the realtime session itself, which costs about
  $0.005. The warm-up is recorded, labelled `Turn: prewarm:` and kept separate (FUP-21). Filed for a decision as
  [#1045](https://github.com/CorbanuCore/CorbanuTerminal/issues/1045).
- The executor's mistaken first edit in FUP-14 opened a fork that sent a paid reopen warm-up ($0.001465) and was never
  used. The fork is listed under "other conversations", and it can be opened from `/resume` because it carries T1's
  copied history (FUP-15 opened it).
- `/cost`'s first read right after a reply can show `1 attempt had incomplete usage` until it is reopened (FUP-11;
  ledger lag, as the design's amendment 1 expected).
- A Kimi K3 or K2.7 Code start warns `Model metadata for 'kimi-k3' not found. Defaulting to fallback metadata` (as in S10).

## Design and freeze

- A fresh read-only Opus 5.5 High designer wrote 24 cases from an intent packet: the S06–S09 intent, the owner decisions
  and 13 text screens ([design/](design/), [designer-session.md](design/designer-session.md)).
- A separate fresh reviewer returned CHANGES with 25 amendments, including the new case FUP-25, all applied as written
  ([amendments-final.md](design/amendments-final.md), [review-session.md](design/review-session.md)). The integrator's
  answers to the designer's questions state facts about the harness only ([answers-v1.md](design/answers-v1.md),
  [answers-final.md](design/answers-final.md)).
- The design was frozen and pushed at `b53816429f` (2026-10-20T02:01:19Z) ([FROZEN.sha256](design/FROZEN.sha256)).
  Executor packet v1, the harness and the candidate manifest were pushed at `854b02f7e8` (02:02:20Z). The lock was taken
  and the clone made at 02:02:23Z ([common/vm-start.txt](common/vm-start.txt)). Every group ran on packet v1, and no
  replay was run.

## Evidence checks

5 of 5 review rounds were used: design, design review, and three fresh read-only Opus 5.5 High evidence checks of a
`git archive` snapshot. r1 and r2 failed on statements only, and every correction was applied: the FUP-02
disposition (passed -> blocked), several sums and citations, and the shakedown's OpenAI warm-up. r3 **passed** with
no required correction. Its four optional clarifications were applied afterwards and have not been re-checked
([review-ledger.md](review-ledger.md), [review/](review/)).

## Candidate

`origin/main` `78dd811d6e`, the coordinator's signed developer build (0.1.48, `bin/corbanu` SHA-256 `e7200306…25bdaa3`),
copied into the guest over SSH ([candidate/](candidate/candidate-manifest.md)).

## Execution

- **Where:** one fresh Tart clone (`acc-tart1-run`) of `corbanu-qa-base` (macOS 26.6.2 25G83, #930); one Tart VM at a
  time; UTM "macOS 2" and "macOS 3" stayed stopped and untouched. The lock was held 02:02:23Z–05:30:47Z.
- **Isolation:** the S10 harness ported to Tart, with a relay for real routes and the key injected outside the executor
  ([common/effective-policy.txt](common/effective-policy.txt), [harness/](harness/)). Real routes:
  - Z.AI;
  - OpenAI: WebSocket Responses, plus `/v1/realtime` capped at 2 sessions; 1 used;
  - Moonshot: capped at 4 requests; 4 used, none refused.
- **Gate:** 109 of 109 probes, the PTY control and the smoke turn, in one attempt ([common/gate/](common/gate/)). Every
  group re-ran the probes and every executor ran them itself; every isolation receipt is true. Guard: 0 SecurityAgent
  prompts in 12,043 checks.
- **Groups**, one fresh GLM 5.3 Flash executor each, one at a time ([common/seq.log.txt](common/seq.log.txt)):

  | Group | Lane | Time (UTC) |
  | --- | --- | --- |
  | Z | agmx1 | 02:06–02:46Z |
  | O | agmx2 | 03:07–04:20Z |
  | R | agmx3 | 04:22–04:53Z |
  | M | agmx1 | 04:54–05:27Z |

  All four exited 0.
- **Shakedown:** a separate clone ran before the freeze (01:49–01:54Z) to test the port. It sent one Z.AI smoke turn
  and no Moonshot request. The relay refused its realtime start, but about 75 ms later the realtime driver's app-server
  thread still opened one real OpenAI Responses WebSocket (01:54:12Z, about 1 s, the same pattern as group R's GPT-6 Sol
  warm-up). That was probably one billed warm-up of about $0.03; the relay doesn't log WebSocket usage
  ([addendum](common/effective-policy-addendum.txt)). Its three harness fixes are listed in effective-policy.txt; its
  logs are in [shakedown/common/](shakedown/common/).
- **Wrap-up:** the final logs were captured, the three real key files deleted and checked absent, then the clone was
  stopped and deleted and the lock removed ([common/vm-state-end.txt](common/vm-state-end.txt),
  [common/erase-keys.out](common/erase-keys.out)).
- **Key scan:** the three real keys (Z.AI, OpenAI, Moonshot) were found in 0 files of this record
  ([keyscan-record.txt](keyscan-record.txt)) and of the run folder ([keyscan-run-folder.txt](keyscan-run-folder.txt)).
- **Spend (approximate):**
  - **Moonshot: $0.0538** for four requests (K3 $0.040466, K2.7 Code $0.013321), inside D54's $0.05–0.06.
  - **OpenAI: about $0.059 in the run**, plus probably about $0.029 in the shakedown (about $0.088 in all).
    - Group O, $0.0252 by `/cost`: T1 $0.016309, T2 $0.003695, the edit branch $0.003695, the abandoned fork $0.001465.
    - The D16 realtime check: about $0.034 in the run, or about $0.063 counting the shakedown's probable warm-up. That
      is about $0.013 over D16's $0.05.
      - The session itself is about $0.0048: the relay logged 1,179 text input and 3 output tokens, priced at
        gpt-realtime-1.5's $4/$16 per million; the product shows it as "no price".
      - The rest is app-server text warm-ups: the run thread's GPT-6 Sol warm-up ($0.029290) and the shakedown's
        probable one.
  - **Z.AI: about $1.13** at the catalogue's GLM 5.3 Flash rates (446 requests; 33.2 M input tokens, 32.6 M of them
    cached; 133 k output), almost all of it the executors' own context ([common/mediator-final.jsonl.gz](common/mediator-final.jsonl.gz)).

## Tart compared with UTM (D34)

- A clone reached SSH 12 s after `tart run`. No host share and no FASTDRIVE2 were needed, because the build and packet
  were copied in over SSH.
- Erasing is deleting the clone, so there is no erase script, restore or verification boot as on UTM, and no state
  carries over between runs.
- The port needed three harness changes, all found by the shakedown before the freeze:
  - `tart run` must live in its own tmux session (started with nohup, it died with the tool shell).
  - An unprivileged account can't bind 127.0.0.1:443, so the relay's direct realtime listener was dropped. It wasn't
    needed: the product's realtime client goes through the proxy.
  - `seal-guest.sh` pins only Z.AI's addresses, so `provision.sh` adds OpenAI's and Moonshot's.
- The guest runs macOS 26.6.2; UTM "macOS 3" runs 26.2.

## Record format

- `runs/<group>/` (Z, O, R, M): the executor's report is `collected/evidence/result.md`.
  - Every screen read is in `screens.jsonl`; also the raw terminal output (`pty.raw`), `actions.jsonl` and the
    transcript `executor-events.jsonl`. Any file over 400 KB is stored gzipped (`.gz`).
  - Where the executor also saved screens as files (groups Z and O), that folder is packed as one `.tar.gz`
    (`group-Z.tar.gz`, `screens.tar.gz`); groups R and M kept their captures only as labelled entries in `screens.jsonl`.
  - Each run also has `isolation-<case>.json`, `probes.json`, `mediator-window.jsonl`, `ledger-attempts.txt` (the
    integrator's read of the accounting ledger after the run, with Request IDs) and `product-home.manifest.txt`.
- Not committed (git-ignored): each run's `collected/fixture/workspace/.env` (the fixture's synthetic canary) and
  `collected/product-logs/codex-tui.log`. Fixture `.git` folders are kept as `dot-git`, without objects.
- `harness/agms-probe` is stored as `harness/agms-probe.txt`; `tools/` holds the integrator's scripts and `final.json`.

## Changed after the run

- #1045 (PR #1059): `harness/agms-product` and `harness/realtime_driver.py` now start the realtime thread on the
  command's model (`--thread-model`; `agms-tui realtime` passes gpt-6-luna), so a rerun's text warm-up is not on
  GPT-6 Sol. The run used the earlier versions: their sha256 are in `common/provision.out` (`bin/agms-product`,
  `bin/agms-realtime-driver`), and they are at commit `854b02f7e8`.

## Named limits

- FUP-17/18/19: the per-request rate and component screens are missing (see Result). FUP-15: the "already viewing"
  path is untested.
- OpenAI's WebSocket usage is not logged by the relay, so OpenAI spend comes from `/cost`'s estimates; the realtime
  amount comes from the relay's `response.done` usage at list prices.
- Executor deviations, disclosed per case:
  - FUP-07: a first turn attempt never reached the composer and sent nothing.
  - FUP-14: a mistaken first edit, then a recovery restart that added warm-up R11 and left the abandoned fork.
  - FUP-15: the wrong conversation was picked.
- The relay logs no tool names, so model-visible tool sets are not inventoried ([common/tool-inventory.json](common/tool-inventory.json)).
- The same model family (GLM 5.3 Flash) drives the executor and the Z.AI product turns.
- The ledger reads (`runs/*/ledger-attempts.txt`) and the relay-usage checks are the integrator's, made after the
  run. Dispositions rest on the executors' screens, except where a summary says otherwise.
- The relay log has 7 Z.AI `session_error` lines: idle connections timing out after 600 s and one broken pipe. No turn
  was lost.
- The first designer launch died with its tool shell before its first event and is not counted (designer-session.md).
- The designer and the design reviewer were isolated by instruction and a read-only sandbox in a packet-only working
  directory; their reads were not otherwise restricted (their access records list what they read).
- The lock release and UTM's end state are shown by a host check after the run
  ([common/host-end-check.txt](common/host-end-check.txt)); `tart-run.sh down` printed "lock released" to its terminal,
  outside its captured log.
- FUP-10's intermediate Request-ID audits (amendment 12) were partial after FUP-09 (R4's ID was not read until the final
  audit) and not done after FUP-12 and FUP-14; only the final audit and earlier single reads exist.
- The frozen designer-session.md says the packet has eight S06–S08 screens; there are nine (13 screens in all, as
  stated above).
- Two explanations are the integrator's, made after the run, and are not part of the code-blind evidence: #1038's
  cause comes from reading the code, and #1039's refusal text comes from the raw terminal output.
