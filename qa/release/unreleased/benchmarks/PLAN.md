# Benchmark cycle plan: post-sync bootstrap (#2717)

Status: **draft, preparation only.** No paid model call was made. This plan
freezes the inputs that `benchmarks/README.md` requires before any harness
starts, so the run on the post-sync candidate is mechanical. The machine-readable
freeze is [`frozen-config.json`](frozen-config.json); the commands are in
[`RUNBOOK.md`](RUNBOOK.md). When the candidate's version is known, move this
directory to `qa/release/<version>/benchmarks/` and fill in the evidence there.

## What still needs a decision or access

| Item | State | Who |
| --- | --- | --- |
| Live repository access (TensorCash or Isometric Game) | **Blocked.** Both canonical URLs return "repository not found" to the maintainers' GitHub login, and no `CORBANU_TENSORCASH_REPO` / `CORBANU_ISOMETRIC_REPO` path exists on the Mac or glitch (same as the 0.1.49 preflight). | Access from @goodalexander or a recorded local path |
| Competitive task, prompt and rubric | Not frozen: they need the repository. The rubric shape is fixed below. | Release owner, with access |
| Thresholds and common cutoff | Proposed below; need product-authority approval before any lane runs. | Travis |
| Relevant model set | Derived by rule (33 routes); no exclusions proposed. Any exclusion needs a written rationale and approval. | Travis |
| Real-run spend | Estimated at about **$349** (below). | Travis |

## Frozen inputs

**Candidate.** The post-sync signed build. Record its version, commit, SHA-256,
signing identity and `INSTALL.json` `launcher_target` when it exists. Run the
binary itself, not the `~/.local/bin/corbanu` launcher, so the launcher's
`activate.sh` cannot replace the per-run home. The dry run used 0.1.49
(`ff2b81a7b4`, SHA-256 `896684ea…d082f`, Developer ID K5N2T25C23).

**Competitors** (installed in isolated homes on the Mac under
`/Volumes/CorbanuDrive/Corbanu/.codex-work/bench-prep-2717`, outside the repo):

| Harness | Pinned version | Install |
| --- | --- | --- |
| Hermes Agent | v0.23.3, commit `caafe538…` (latest stable, 2026-11-20); Python 3.14.9, OpenAI SDK 2.54.0 | the release's own `scripts/install.sh` with `--channel stable --branch v0.23.3 --hermes-home <home> --non-interactive --skip-browser --skip-computer-use`, run under `env -i` with a scratch `HOME` |
| Kilo Code | `@kilocode/cli` 7.8.32 (latest, 2026-11-23); node v25.2.1 | `npm install --prefix <dir> @kilocode/cli@7.8.32` under `env -i` with a scratch `HOME` and npm cache |

Exact commands, paths and the Kilo platform-binary digest are in
`frozen-config.json` under `harnesses`. Neither install wrote to the real home
(`~/.hermes`, `~/.config/kilo` and `~/.local/bin/hermes` are absent).

**Competitive component** (three-way, live repository).

- Repository: proposed **Isometric Game** (`goodalexander/isometricgame`), base
  `59821b7a85524f186f946c4670480c7ee96483cb`, the base used by the 0.1.38 and
  0.1.41 release QA. Alternative: TensorCash, last recorded base `9325ed67…`.
  Confirm the base still exists once access is granted.
- Task: one development task, chosen with the repository open. Freeze the
  prompt file and rubric (with SHA-256) before any lane starts.
- Rubric shape: the repository's test command passes; task-specific acceptance
  checks, each pass/fail, written before the run; no edits outside scope and no
  deleted or weakened tests; diff, transcript, timing and usage preserved.
- Three repeats per harness, one disposable worktree each, all from the base.

**Route and model.** The shared route is **OpenRouter** with
**`openai/gpt-6-sol`** at reasoning **high**: GPT-6 Sol is Corbanu's default model,
and OpenRouter is an OpenAI-style connection that all three harnesses support,
so they see the same model, endpoint and price list. This follows the
2026-10-13 owner rule that the OpenAI-style connection is the default for any
multi-route choice. Kilo's model id is `openrouter/openai/gpt-6-sol`.

**Permissions and tools.** Full auto-approval in a disposable workspace for
every harness (Corbanu `--dangerously-bypass-approvals-and-sandbox`, Hermes
`--yolo --accept-hooks`, Kilo `--auto`); each harness's default built-in tools in
a fresh home; no added MCP servers, skills, plugins or memories; matched
explicit reasoning (Corbanu `model_reasoning_effort`, Hermes `--reasoning`, Kilo
`--variant`). No harness-specific turn, tool or workflow cap.

**Coding performance component.** Every task in
`benchmarks/coding/configs/all-tasks.example.json` (11, including QueueCraft;
per-task tree digests in the freeze) for every route in the relevant model set,
one wave. The set is derived by rule from the candidate's
`codex-rs/models-manager/models.json`: every route with visibility `list` and
orchestration status `eligible`, at its default reasoning level. On current
`main` that is 33 routes and 363 runs. `make_run_configs.py refresh` re-derives
it on the candidate; review the diff before freezing.

**Hermes efficiency comparison.** Corbanu, Hermes and Kilo on the shared route,
all 11 tasks, three waves each (99 runs). This is the evidence for or against
the efficiency claim; the decision rule is below.

**Price source.** Per route, the candidate's catalogue billing rows. For the
shared OpenRouter route and for catalogue routes without billing rows
(auth-dependent GPT-5.6 and the Claude plan routes, priced at API-equivalent
rates), an OpenRouter `/api/v1/models` snapshot fetched 2026-11-23T20:24Z,
SHA-256 `945e342c…be931c`; the prices used are copied into the freeze.
Provider-reported actual cost wins when a harness records it. Unknown spend is
never zero.

## Proposed thresholds (need approval)

Bootstrap absolute caps, since no earlier qualifying cycle exists:

- Coding pair: verifier pass; wall time ≤ 1,200 s (a timeout fails);
  calculated spend ≤ $5.00.
- Competitive, Corbanu: rubric met in at least 2 of 3 repeats; ≤ 3,600 s and
  ≤ $15.00 per run.

Regression thresholds for later cycles against this baseline: per-route pass
count must not fall; per-route median wall time ≤ 1.25 × baseline; per-route
spend ≤ 1.25 × baseline.

## Common safety and spend cutoff

- Same per-run wall timeout for every harness: 1,200 s coding, 3,600 s live.
- A dedicated OpenRouter key with a hard credit limit of 1.5 × the shared-route
  estimate (about $98); a run that hits it is recorded as cut off.
- Direct-provider matrix: the operator stops the runner (by PID) when the
  calculated spend reaches 1.5 × the matrix estimate, and records the cutoff.

## Hermes efficiency decision rule

1. Correctness first: Corbanu's verifier passes must be at least Hermes's minus
   one over the 33 task × wave runs, and Corbanu must meet the live rubric in at
   least 2 of 3 repeats. Otherwise, no claim.
2. For each task × wave where both runs finished with known spend, take the cost
   ratio Corbanu / Hermes. Estimate: geometric mean. Interval: 95% percentile
   bootstrap, resampling tasks (all waves of a task together), 10,000 resamples,
   seed 2717.
3. Claim only if the interval's upper bound is ≤ 0.95, stated as "Corbanu used
   X% less spend than Hermes (95% CI a–b%)" with the point estimate and
   interval. If the interval overlaps 0.95–1.05, publish nothing. If its lower
   bound is above 1.05, file a regression instead.
4. Wall time and tokens get the same treatment, reported separately; they never
   change the spend claim.

## Spend estimate

Per-run token profile (80k uncached input, 320k cached input, 12k output)
calibrated on the 0.1.46 matched-reasoning diagnostics (GPT-6 Sol over
OpenRouter: $1.20 for four Corbanu runs), times the frozen price, times a 1.5
planning margin:

| Part | Runs | Estimate |
| --- | ---: | ---: |
| Coding matrix, Corbanu, 33 routes | 363 | $242.98 (of which $77.29 is Claude plan routes, used from plan capacity) |
| Hermes efficiency, three harnesses | 99 | $51.08 (about $17 per harness) |
| Competitive live task, 3 repeats × 3 harnesses | 9 | $13.93 (about $4.64 per harness) |
| Website builder (optional diagnostic) | 6 + judge | $41.00 |
| **Total** | | **$348.99** (cash about $272 without plan routes) |

Most of the matrix spend is the eight Claude routes (Opus and Fable, API and
plan): about $133 together.

## Evidence the dry run produced

See [`RUNBOOK.md`](RUNBOOK.md#dry-run-record). In short: both runner configs
generate and plan, every lane ran against stubs that record the exact argv and
call only `--version` on the real binaries, every verifier executed and failed
the unsolved baselines as designed, source integrity held, and the exact-key
scan was clean.

## Harness findings to carry into the run

- **Hermes per-run homes.** A brand-new `HERMES_HOME` has no dependency
  environment in v0.23.3: the first command copies about 420 MB and 15 s of
  packages into it, and `--version` reports no OpenAI SDK. The runner's built-in
  Hermes adapter uses such a home. The frozen template instead puts each run's
  home under `<installed home>/profiles/`, which shares the installed environment
  (1 s, 8 KB per run). Install Hermes with its default checkout location (inside
  its home) so it can find that environment.
- **Kilo flag.** Kilo 7.8.32 does not list `--dangerously-skip-permissions`,
  which the runner's built-in Kilo adapter passes; the frozen template uses
  `--auto`. The built-in adapter also does not isolate Kilo's home; the template
  gives every run fresh XDG directories and pins `autoupdate: false` through
  `KILO_CONFIG`.
- **Usage capture.** The runner reads native cost only from stdout and request
  dumps. Hermes writes tokens and cost to `--usage-file`, which the template
  sets; tally it separately.
- **Route check.** For Vercel's provider-qualified slugs (`vercel/...`) the
  request carries the upstream model id, so the runner's `route_verified` will be
  false; reconcile it from the request dump.
- **Claude Code.** The website builder runs the real Claude Code binary. Per the
  keychain rule it may run only inside a `sandbox-exec` profile that denies the
  keychain services.
