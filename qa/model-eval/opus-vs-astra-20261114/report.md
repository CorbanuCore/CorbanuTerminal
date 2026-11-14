# Opus 5.5 High vs Astra High: model evaluation, 2026-11-14

Owner request (Travis, 2026-11-14): compare Claude Opus 5.5 High with Astra High (`gpt-6-astra`, reasoning high) on our real work, to decide when Astra may be used to raise throughput. Run by bench-1114. A readable version with charts is in `docs/management/astraVsOpus.html`.

**Short answer:** Astra High was at least as good as Opus 5.5 High on almost everything we measured. It is suitable for first-pass review and everyday implementation, as long as an independent Opus review still happens before merge. Neither model was reliable alone. The samples are small (see Caveats).

## Method

- **Isolation.** Each arm ran in its own clone that shares objects with the main repository but contains only the target commit's history: no later refs and no remote. Briefs began with "LOCAL ONLY" and banned the network and GitHub. One Opus arm ran at a time, with at most one implementation arm per model running concurrently.
- **Track A.** Each arm got the same neutral review brief (`trackA/briefs/`). The answer key `trackA/key.md` was frozen before any arm ran (`trackA/key.sha256` = `ccc920d4…0664`).
- **Track B.** Each arm got the same task brief, which included the program's working rules and the usual gate. The hidden checks were run afterwards by us (`trackB/*-hidden-check.md`).
- **Blinding.** Graders saw solutions as X and Y. The X/Y assignment (`grade/blind-key.md`) was randomised and written before any arm ran. Model names, token counts and arm paths were removed from packets.
- **Commands.** Opus: `corbanu exec -m claude-opus-5-5-plan -c model_provider="claude-plan" -c model_reasoning_effort="high"`. Astra: `-m gpt-6-astra -c model_provider="openai" -c model_reasoning_effort="high"`. Reviews and graders ran with `-s read-only`; implementation arms with `-s danger-full-access`.

## Track A: review benchmark (8 known defects + 2 clean controls)

| Case | PR | Class | Opus 5.5 High | Astra High |
|---|---|---|---|---|
| A01 | #1837 | security classifier | missed · REQ ✓ | **found** · REQ ✓ |
| A02 | #1962 | test infra (dependency bump) | missed · APPROVE ✗ | missed · APPROVE ✗ |
| A03 | #1055 | classifier instrumentation | missed · APPROVE ✗ | **found** · REQ ✓ |
| A04 | #2043 | sync conflict (security) | **found** · REQ ✓ | **found** · REQ ✓ |
| A05 | #2133 | security classifier | partly (1 of 2) · REQ ✓ | **found** (both) · REQ ✓ |
| A06 | #1171 | logic / message | partly · REQ ✓ | missed · REQ ✓ |
| A07 | #1175 | agent control (TUI) | **found** (+ more secondaries) · REQ ✓ | **found** · REQ ✓ |
| A08 | #1889 | Windows test quality | **found** · REQ ✓ | **found** · REQ ✓ |
| A09 | #1878 | control | APPROVE ✓ | APPROVE ✓ |
| A10 | #1944 | control | APPROVE ✓ | REQUEST_CHANGES ? (finding judged real by both graders) |

| Metric | Opus 5.5 High | Astra High |
|---|---|---|
| Known defects (full = 1, partial = ½) | **4.0 / 8** | **6.0 / 8** |
| Correct verdict | 8 / 10 | 8 / 9 (A10 debatable) |
| Real extra findings (after blinded grading of the 13 we could not settle) | 23 | 14 |
| False findings | 0 | 0 |
| Wall time, 10 reviews | 27 min | 29 min |
| Tokens, 10 reviews | 969k | 877k |

- **Astra was stronger on security-classifier code:** it found the known bug fully in 3 of 3 classifier cases, against 0 for Opus (one partly).
- **Opus was stronger on agent-control/TUI code** (more of the related parts in A07, and partly found A06) and raised more additional real issues.
- **Both missed A02.** Spotting it needed knowledge of the assert_cmd 2.2 behaviour change, and the source of that version wasn't available offline.

Per-case reasoning is in `trackA/scoring-notes.md`, raw reviews in `trackA/reviews/`, and the blinded extra-finding grading in `grade/A-extra/`.

## Track B: paired implementation (4 tasks)

| Task | Hidden check: Opus | Hidden check: Astra | Preferred by Opus grader | Preferred by Astra grader |
|---|---|---|---|---|
| B1 #2046 guardian stop gap (core) | PASS (merged tests 2/2; own fail→pass) | PASS (2/2; own fail→pass) | Astra | Astra |
| B2 #2102 picker refresh (TUI) | PARTIAL (1/2: open picker jumps tab) | PARTIAL (1/2: same jump) | Opus | Astra |
| B3 sync conflict codex-agent-roles | PASS (src identical to merged resolution; +97 lines tests) | PASS (identical; no tests) | Opus | Opus |
| B4 #2018 classifier brace groups (fresh) | PASS suites; **quieter on 4 lines** (confirmed after grading) | PASS suites; old verdict kept as floor | Astra | Astra |

Rubric means over the 4 tasks (1–5; correctness / security & rules / test quality / scope / clarity / honesty):

| Work | Grader | Correct. | Sec./rules | Tests | Scope | Clarity | Honesty | Total /30 (mean) |
|---|---|---|---|---|---|---|---|---|
| Opus | Opus | 4.00 | 3.75 | 4.25 | 4.75 | 4.25 | 4.00 | 25.0 |
| Opus | Astra | 3.75 | 3.50 | 3.75 | 5.00 | 4.25 | 3.75 | 24.0 |
| Astra | Opus | 4.25 | 4.25 | 3.50 | 3.25 | 3.50 | 4.75 | 23.5 |
| Astra | Astra | 4.25 | 4.50 | 3.25 | 5.00 | 4.50 | 4.75 | 26.2 |

**Grader agreement:** 3 of 4 preferences agree; 26 of 36 paired dimension scores are exactly equal (mean absolute difference 0.33).

**Self-preference:** each grader gave its own model's work a higher total than the other grader did. The gap was +3, +2, +3 and +7 for B1–B4. That is mild but consistent, so pair the graders whenever models are being compared.

Notes:
- **B1:** Opus's result file said strict clippy fails "with the pinned 1.99.1". Our rerun with 1.99.1 passes for both solutions. The 10 errors appear only with the 1.95.0 default on the worker PATH, in files neither solution touched.
- **B4:** see `trackB/b4-postgrade-probe.md`. On the base, `! { echo ok; } | bash` and `time { echo ok; } | bash` are judged UnseenCode (bash and zsh, plain and named). Opus's change judges them as nothing, which breaks the task's never-quieter rule. Astra's change keeps them.
- **Scope:** Astra added extra QA/evidence documents in B1, B2 and B4, which the graders marked down on scope. Opus left smaller diffs.

B1 variance rerun: see the section below.

## Cost

| | Opus 5.5 High | Astra High |
|---|---|---|
| Track A (10 reviews) | 27 min, 969k tokens | 29 min, 877k tokens |
| Track B (4 tasks) | 88 min, 533k tokens | 91 min, 756k tokens |
| Grading (5 runs each) | 9 min, 377k tokens | 9 min, 318k tokens (+1 superseded B1 re-grade) |

Opus uses limited Claude-plan capacity (it was in a multi-day cooldown until today). Astra uses the OpenAI provider. Token counts are each harness's own report.

## Compliance

- No arm ran a GitHub-write or push command, and none used `git stash`, `rm -f`, `pkill`/`killall` or tracing (`compliance.txt`).
- GitHub activity for the account during the run window matches other workers only (#2141, #2142, #2143, comments on #1532/#1628).
- Astra's implementation logs list exactly 78 commands each, so they may be capped. For Astra's Track B, compliance also rests on the clones having no remote and on the GitHub activity check.

## Recommendation by task class

| Task class | Recommendation | Evidence |
|---|---|---|
| First-pass code review | **Astra-OK** | Track A 6/8 vs 4/8 known defects; all extra findings real |
| Final security review (D318 sign-off) | **Opus-only for now** | Policy step; neither model complete alone; add Astra as a second lens |
| Security code changes | **Astra + Opus final review** | B4 Astra kept never-quieter and Opus broke it; Track A classifier 3/3 vs 0; n=1 task |
| Sync conflict resolution | **Astra + Opus final review** | B3 both identical to merged; A04 both caught; security paths |
| Ordinary bug fixes | **Astra-OK** | B1 both pass, both graders prefer Astra; B2 same miss for both |
| CI / flake | **Astra-OK** | A08/A09 equal; little implementation evidence |
| Docs | **Astra-OK (not tested)** | no direct evidence; low risk under normal review |
| Grading / triage | **Astra-OK; pair with Opus for model comparisons** | agreement 3/4, mean score gap 0.33, mild self-preference |

"Astra-OK" changes which model does the work, not who approves it. The independent Opus 5.5 High review before merge and the corbanu-security approval on owned paths stay as they are.

## Caveats

- Small n: 8 + 2 cases in Track A and 4 tasks in Track B, mostly one run per model.
- Defect selection: A01, A03 and A05 were first found by Astra on 11-11; A04, A06, A07 and A08 by Opus.
- Track A reviews were static and offline.
- In B1/B2 the hidden tests were added as an extra module next to each solution's own tests. B4 has no reference fix. Only lib test suites were run, not `--test all`.
- Blinding was imperfect: writing style, the experiment folder name, and B2's subject matter (model names) may hint at the model.
- The Opus log header doesn't echo reasoning effort for claude-plan; both arms got the same flag.

## Decisions for Travis

1. **Allow Astra High as implementer and first-pass reviewer** for ordinary bug fixes, CI/flake work, docs, triage and sync conflicts, keeping the independent Opus review before merge. Options: (a) yes; (b) review only; (c) wait for more data. **Recommendation: (a).**
2. **Security:** keep Opus as the only sign-off (D318), and add Astra as a required second reviewer on security-classifier PRs. **Recommendation: yes.**
3. **New issue to file** (this experiment could not write to GitHub): `GIT_ALLOW_PROTOCOL=file` from #1944 lifts git's `protocol.file.allow=user` guard for submodules in the Windows no-network sandbox. Both graders judged it real (medium/high); impact is limited.
4. **Worker toolchain:** the default `rustc` on PATH is 1.95.0, but the repo pins 1.99.1. This produced false clippy-failure reports. **Recommendation:** put the pinned toolchain first in the worker PATH.
5. **#2018** is still open. The local experimental fixes are in `.codex-work/bench-astra-20261114/wt/b4-*`; Astra's is safe under the never-quieter rule. The issue was offered to contributor @ryssroad, so ask them first.
