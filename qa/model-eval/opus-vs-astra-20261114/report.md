# Opus 5.5 High vs Astra High: model evaluation, 2026-11-14

Owner request (Travis, 2026-11-14): compare Claude Opus 5.5 High with Astra High (`gpt-6-astra`, reasoning high) on our real work, to decide when Astra may be used to raise throughput. Run by bench-1114. A readable version with charts is in `docs/management/astraVsOpus.html`.

**Short answer:** Astra High is good enough to do first-pass reviews and everyday implementation, as long as an independent Opus 5.5 High review still happens before merge. The two models are close overall, each with its own strengths, and neither was reliable alone. The samples are small, and the Track A cases favour each model on the bugs it first found (see Caveats).

## Method

- **Isolation.** Each arm ran in its own clone that shares objects with the main repository but contains only the target commit's history: no later refs and no remote. Briefs began with "LOCAL ONLY" and banned the network and GitHub. One Opus arm ran at a time, with at most one implementation arm per model running concurrently.
- **Frozen keys.** The Track A answer key `trackA/key.md` (sha256 `ccc920d4…0664`) and the blind X/Y key `grade/blind-key.md` were created at 11:09:19Z. The first arm started at 11:11:24Z (`trackA/key-timestamps.md`). Their headers say "~11:40Z"; that was a wrong estimate, left unchanged so the hashes stay valid.
- **Track A.** Each arm got the same neutral review brief (`trackA/briefs/`).
- **Track B.** Each arm got the same task brief, which included the program's working rules and the usual gate. The hidden checks were run afterwards by us (`trackB/*-hidden-check.md`).
- **Blinding.** Graders saw solutions as X and Y. The X/Y assignment was randomised before any arm ran, and model names, token counts and arm paths were removed from packets.
- **Commands.** Opus: `corbanu exec -m claude-opus-5-5-plan -c model_provider="claude-plan" -c model_reasoning_effort="high"`. Astra: `-m gpt-6-astra -c model_provider="openai" -c model_reasoning_effort="high"`. Reviews and graders ran with `-s read-only`; implementation arms with `-s danger-full-access`.

## Track A: review benchmark (8 known defects + 2 clean controls)

| Case | PR | Class | First found by | Opus 5.5 High | Astra High |
|---|---|---|---|---|---|
| A01 | #1837 | security classifier | Astra | missed · REQ ✓ | **found** · REQ ✓ |
| A02 | #1962 | test infra (dependency bump) | maintainer | missed · APPROVE ✗ | missed · APPROVE ✗ |
| A03 | #1055 | classifier instrumentation | Astra | missed · APPROVE ✗ | **found** · REQ ✓ |
| A04 | #2043 | sync conflict (security) | Opus | **found** · REQ ✓ | **found** · REQ ✓ |
| A05 | #2133 | security classifier | Astra | partly (1 of 2) · REQ ✓ | **found** (both) · REQ ✓ |
| A06 | #1171 | logic / message | Opus | partly · REQ ✓ | missed · REQ ✓ |
| A07 | #1175 | agent control (TUI) | Opus | **found** (+ more secondaries) · REQ ✓ | **found** · REQ ✓ |
| A08 | #1889 | Windows test quality | Opus | **found** · REQ ✓ | **found** · REQ ✓ |
| A09 | #1878 | control | — | APPROVE ✓ | APPROVE ✓ |
| A10 | #1944 | control | — | APPROVE ✓ | REQUEST_CHANGES ✗ (see note) |

| Metric | Opus 5.5 High | Astra High |
|---|---|---|
| Known defects (full = 1, partial = ½) | **4.0 / 8** | **6.0 / 8** |
| … on the 3 Astra-found cases | 0.5 | 3.0 |
| … on the 4 Opus-found cases | 3.5 | 3.0 |
| Correct verdict against the frozen key | 8 / 10 | 8 / 10 |
| Real extra findings | 23 | 14 |
| Extra findings judged false | 0 | 0 |
| Wall time, 10 reviews | 27 min | 29 min |
| Tokens, 10 reviews | 969k | 877k |

- **Selection effect.** The overall gap comes almost entirely from the three cases Astra itself first found. On the cases Opus first found, Opus is slightly ahead.
- **A10.** Astra's reason for blocking the control #1944 was this: `GIT_ALLOW_PROTOCOL=file` re-allows local-file submodule clones that git blocks by default. Both blinded graders judged it real. The verdict still counts as wrong against the frozen key.
- **Extra findings.** We could not settle 13 of them ourselves; they went, unattributed, to both graders, who rated all 13 real. Since neither grader rejected anything, treat "0 false" as weak evidence.
- **Both missed A02.** Spotting it needed knowledge of the assert_cmd 2.2 change, which was not available offline.

Per-case reasoning is in `trackA/scoring-notes.md` and raw reviews in `trackA/reviews/`. The blinded extra-finding grading is in `grade/A-extra-brief.txt` and `grade/A-extra-grader-*.txt`, with the map in `grade/extra-map.json`.

## Track B: paired implementation (4 tasks, plus a B1 rerun)

| Task | Hidden check: Opus | Hidden check: Astra | Preferred by Opus grader | Preferred by Astra grader |
|---|---|---|---|---|
| B1 #2046 guardian stop gap (core) | PASS (new reference test 1/1, adapted existing 1/1; own fail→pass) | PASS (same) | Astra | Astra |
| B2 #2102 picker refresh (TUI) | PARTIAL (new reference snapshot test 0/1: open picker jumps tab; own fail→pass) | PARTIAL (same miss) | Opus | Astra |
| B3 sync conflict codex-agent-roles | PASS (src identical to merged resolution; +97 lines tests) | PASS (identical; no tests) | Opus | Opus |
| B4 #2018 classifier brace groups (fresh) | **FAIL (task rule)**: suites pass, but 2 command lines (bash and zsh) are quieter than the base, found after grading | PASS (old verdict kept as floor) | Astra | Astra |
| B1 rerun (variance) | PASS (same as B1) | PASS (same as B1) | Astra | Astra |

Rubric means over B1–B4 (1–5; correctness / security & rules / test quality / scope / clarity / honesty):

| Work | Grader | Correct. | Sec./rules | Tests | Scope | Clarity | Honesty | Total /30 (mean) |
|---|---|---|---|---|---|---|---|---|
| Opus | Opus | 4.00 | 3.75 | 4.25 | 4.75 | 4.25 | 4.00 | 25.00 |
| Opus | Astra | 3.75 | 3.50 | 3.75 | 5.00 | 4.25 | 3.75 | 24.00 |
| Astra | Opus | 4.25 | 4.25 | 3.50 | 3.25 | 3.50 | 4.75 | 23.50 |
| Astra | Astra | 4.25 | 4.50 | 3.25 | 5.00 | 4.50 | 4.75 | 26.25 |

**Grader agreement:** 3 of 4 preferences agree; 32 of 48 paired dimension scores are exactly equal (mean absolute difference 0.40).

**Self-preference:**
- The Astra grader scored Astra's work higher than the Opus grader did on every task: +3, +2, +3 and +3 points (of 30) for B1–B4.
- The two graders gave Opus's work the same totals on B1–B3; on B4 the Opus grader scored it 4 points higher.
- So the lean comes mostly from the Astra grader, and its vote produced the B2 split.
- Totals per grader are in `grade/rubric.json`; `grade/agreement.md` shows the margin gaps.

Notes:
- **B1 clippy:** Opus's result file said strict clippy fails "with the pinned 1.99.1". Our rerun with 1.99.1 passes for both solutions. The 10 errors appear only with the 1.95.0 default on the worker PATH, in files neither solution touched.
- **B1 re-grade:** the first Astra grading of B1 (`grade/b1-grader-astra.v0.txt`) ran before we added our clippy result to the packet. It took Opus's "clippy fails" claim at face value and scored Opus's work security/rules 3, honesty 5. With the clippy evidence added it gave security/rules 4, honesty 3. The Opus grader saw only the corrected packet.
- **B4:** see `trackB/b4-postgrade-probe.md`. On the base, `! { echo ok; } | bash` and `time { echo ok; } | bash` are judged UnseenCode (bash and zsh, plain and named). Opus's change judges them as nothing, which breaks the task's never-quieter rule; Astra's keeps them.
- **Scope:** Astra added extra QA/evidence documents in B1, B2 and B4, which the graders marked down on scope. Opus left smaller diffs.

### B1 variance rerun

| | Opus 5.5 High | Astra High |
|---|---|---|
| Hidden check | PASS (own test fails before, passes after) | PASS (own test fails before, passes after) |
| Wall time, run 1 → run 2 | 28 → 38 min | 31 → 27 min |
| Tokens, run 1 → run 2 | 124k → 116k | 292k → 205k |
| Rubric total, Opus grader / Astra grader | 27 / 27 | 25 / 26 |
| Preferred by | — | both graders (on correctness) |

The outcome is stable. Wall time swapped between runs, so the timing differences are within noise. In its second run, Opus correctly identified the toolchain mismatch that had misled it in run 1.

## Cost

| | Opus 5.5 High | Astra High |
|---|---|---|
| Track A (10 reviews) | 27 min, 969k tokens | 29 min, 877k tokens |
| Track B (4 tasks) | 88 min, 533k tokens | 91 min, 755k tokens (+42%) |
| B1 rerun | 38 min, 116k tokens | 27 min, 205k tokens |
| Grading (6 runs each) | 10 min, 437k tokens | 10 min, 376k tokens (+1 superseded B1 re-grade) |

Per Track B task, Astra used −10% to +135% tokens relative to Opus. Opus uses limited Claude-plan capacity (it was in a multi-day cooldown until today); Astra uses the OpenAI provider. Token counts are each harness's own report (`trackA/reviews/*.meta.txt`, `trackB/*.meta.txt`, `grade/*-grader-*.meta.txt`).

## Compliance

- **Commands.** No arm ran a GitHub-write or push command, and none used `git stash`, `rm -f`, `pkill`/`killall` or tracing (`compliance.txt`, which scans executed commands only). The `gh_write_cmds_in_log` field in the .meta files counts any matching text, including the brief's own "do not git push" line, so it is not a violation count.
- **GitHub activity.** The account's activity during the run window matches other workers only (#2141, #2142, #2143, comments on #1532/#1628).
- **Possible log cap.** Astra's B1–B4 logs each list exactly 78 commands (its B1 rerun lists 59), so its log may be capped. For Astra's Track B, compliance also rests on the clones having no remote and on the GitHub activity check.

## Recommendation by task class

| Task class | Recommendation | Evidence |
|---|---|---|
| First-pass code review | **Astra-OK** | Track A 6/8 vs 4/8 known defects (the gap is mostly from Astra-found cases; on Opus-found cases, 3.0 vs 3.5). Opus raised more extra issues, so the two complement each other. |
| Final security review (D318 sign-off) | **Opus-only for now** | Policy step. Neither model was complete alone. |
| Security code changes | **Astra + Opus final review** | B4: Astra kept the never-quieter rule and Opus broke it (1 task, 1 run). |
| Sync conflict resolution | **Astra + Opus final review** | B3 both identical to the merged resolution (Opus added tests); A04 both caught; these PRs touch security paths. |
| Ordinary bug fixes | **Astra-OK** | B1 (twice) both pass, both graders prefer Astra; B2 same miss for both. |
| CI / flake | **Astra-OK** | A08/A09 equal; little implementation evidence. |
| Docs | **Astra-OK (untested)** | No direct evidence; low risk under normal review. |
| Grading / triage | **Astra-OK; pair with Opus for model comparisons** | Agreement 3/4, mean score gap 0.40; the Astra grader leans to Astra's work by 2–3 of 30. |

"Astra-OK" changes which model does the work, not who approves it. The independent Opus 5.5 High review before merge and the corbanu-security approval on owned paths stay as they are.

## Caveats

- **Small n:** 8 + 2 cases in Track A and 4 tasks in Track B, one run per model except B1 (run twice).
- **Defect selection:** A01, A03 and A05 were first found by Astra on 11-11; A04, A06, A07 and A08 by Opus. This explains most of the Track A gap.
- **Offline review:** Track A reviews were static (no builds) and offline.
- **Hidden-check strength:** B1 and B2 each have one new reference test. The other reference test in each already existed (in B1 the fix adapted it). We did not run the reference tests on the base. B4 has no reference fix. Only lib test suites were run, not `--test all`.
- **Blinding was imperfect:** writing style (for example, Astra's absolute-path links), the experiment folder name, and B2's subject matter (model names) may hint at the model.
- **Reasoning effort:** the Opus log header doesn't echo it for claude-plan; both arms got the same flag.
- **Rebuilding `data.json`:** the build scripts are included (`*.py.txt`) for reference. They read the working-folder layout, so they won't run from this folder without adjusting paths. Their inputs are included as `prose.json`, `trackB/hidden.json` and `grade/extra-adjust.json`.

## Decisions for Travis

1. **Allow Astra High as implementer and first-pass reviewer** for ordinary bug fixes, CI/flake work, triage and sync conflicts (docs too, though untested), keeping the independent Opus review before merge.
   - Options: (a) yes; (b) review only; (c) wait for more data.
   - **Recommendation: (a).**
2. **Security:** keep Opus as the only sign-off (D318). Consider adding Astra as a second reviewer on security-classifier PRs.
   - **Recommendation:** yes, as a low-cost extra check. The two models found different things.
   - The Track A classifier result is biased toward Astra-found bugs, so it does not show Astra is the better security reviewer.
3. **New issue to file** (this experiment could not write to GitHub): `GIT_ALLOW_PROTOCOL=file` from #1944 lifts git's `protocol.file.allow=user` guard for submodules in the Windows no-network sandbox.
   - Both graders judged it real (medium by the Opus grader, high by the Astra grader).
   - Impact is limited, because the sandboxed command could read those files directly.
4. **Worker toolchain:** the default `rustc` on PATH is 1.95.0, but the repo pins 1.99.1. This produced false clippy-failure reports.
   - **Recommendation:** put the pinned toolchain first in the worker PATH.
5. **#2018** is still open.
   - The local experimental fixes are in `.codex-work/bench-astra-20261114/wt/b4-*`; Astra's is safe under the never-quieter rule.
   - The issue was offered to contributor @ryssroad, so ask them first.
