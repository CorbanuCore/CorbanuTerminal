# Runbook: #2717 benchmark cycle on the post-sync candidate

Inputs are frozen in [`frozen-config.json`](frozen-config.json); the reasons are
in [`PLAN.md`](PLAN.md). Do not start a paid step until Travis has approved the
spend, thresholds and model set, and the live task is frozen.

## 0. Variables (Mac)

```bash
unset CORBANU_HOME CODEX_HOME PFTERMINAL_HOME
REPO=/path/to/CorbanuTerminal          # main at the candidate commit
V=<candidate version>                  # e.g. 0.1.50
CAND=<absolute path of the signed candidate corbanu binary>   # INSTALL.json launcher_target
BENCH=/Volumes/CorbanuDrive/Corbanu/.codex-work/bench-prep-2717
HERMES_ROOT=$BENCH/homes/hermes2
HERMES=$HERMES_ROOT/hermes-agent/.hermes/bin/hermes
KILO=$BENCH/tools/kilo-7.8.32/node_modules/.bin/kilo
OUT=$BENCH/run-$V
cd "$REPO"
git mv qa/release/unreleased/benchmarks "qa/release/$V/benchmarks"   # in the evidence PR
EVID=$REPO/qa/release/$V/benchmarks
```

Check the disk first (`df -h /Volumes/CorbanuDrive`; keep at least 20 GB free).

## 1. Check the pins

```bash
codesign --verify --strict "$CAND" && shasum -a 256 "$CAND"; "$CAND" --version
HERMES_HOME=$HERMES_ROOT "$HERMES" --version   # v0.23.3 · local caafe5389ae5 · OpenAI SDK 2.54.0
"$KILO" --version                              # 7.8.32
shasum -a 256 $BENCH/tools/kilo-7.8.32/node_modules/@kilocode/cli-darwin-arm64/bin/kilo   # af2bfa05…
```

Record the candidate's version, commit, SHA-256 and signing identity in
`frozen-config.json` `candidate`. A different Hermes or Kilo version means the
freeze changed: stop and ask.

## 2. Re-derive the model set on the candidate, then generate configs

```bash
python3 "$EVID/make_run_configs.py" refresh        # review `git diff`; route changes need approval
python3 "$EVID/make_run_configs.py" refresh --check
G="python3 $EVID/make_run_configs.py generate --corbanu-bin $CAND --hermes-bin $HERMES \
   --hermes-root $HERMES_ROOT --kilo-bin $KILO"
$G --dry-run --out "$OUT/configs" --run-root "$OUT/runs"
$G --out "$OUT/configs" --run-root "$OUT/runs"
```

## 3. Dry run (no model calls) and canary (about $1.50)

```bash
for c in hermes-efficiency coding-matrix; do
  python3 benchmarks/coding/runner.py --config "$OUT/configs/$c-dry-run.json" run --confirm-paid-run > "$OUT/$c-dry-run.log"
done   # expect exit 1: stubs solve nothing; check every summary.json has returncode 0 and source_integrity ok
python3 - "$OUT" <<'EOF'
import json, sys
out = sys.argv[1]
cfg = json.load(open(f"{out}/configs/hermes-efficiency.json"))
cfg.update(run_dir=f"{out}/runs/canary", waves=1, tasks=cfg["tasks"][:1])
cfg["caps"]["max_total_runs"] = len(cfg["agents"])
json.dump(cfg, open(f"{out}/configs/canary.json", "w"), indent=2)
EOF
OPENROUTER_API_KEY="$(corbanu vault auth-helper <openrouter-bench-label>)" \
  python3 benchmarks/coding/runner.py --config "$OUT/configs/canary.json" run --confirm-paid-run
```

The canary must show, for all three harnesses: the route observed, usage
recorded (Hermes `hermes.usage.json`), reasoning "high" in the request (Corbanu's
`corbanu.*.request.json`), no user-level skills or instructions in the request,
and the verifier ran. The dry run does not exercise the run flags or fresh-home
sign-in, so this is their first test. Fix wiring before continuing; a canary
failure is not a benchmark result.

## 4. Paid runs

The shared-route campaigns use a dedicated OpenRouter key whose hard limit is
1.5 × their estimate (about $98); the matrix's OpenRouter routes use a separate
key. Every credential is fetched inside the consuming command; never export or
print one.

```bash
# Hermes efficiency: 99 runs, three concurrent lanes (est. $51.08)
OPENROUTER_API_KEY="$(corbanu vault auth-helper <openrouter-bench-label>)" \
  python3 benchmarks/coding/runner.py --config "$OUT/configs/hermes-efficiency.json" run --confirm-paid-run

# Coding matrix: 440 runs, one lane per provider (est. $289.90, $77.29 of it plan capacity)
OPENAI_API_KEY="$(corbanu vault auth-helper <openai-label>)" \
ANTHROPIC_API_KEY="$(corbanu vault auth-helper <anthropic-label>)" \
CLAUDE_CODE_OAUTH_TOKEN="$(corbanu vault auth-helper <claude-plan-label>)" \
OPENROUTER_API_KEY="$(corbanu vault auth-helper <openrouter-matrix-label>)" \
ZAI_API_KEY="$(corbanu vault auth-helper zai_key_3)" \
AI_GATEWAY_API_KEY="$(corbanu vault auth-helper <vercel-label>)" \
AMBIENT_API_KEY="$(corbanu vault auth-helper <ambient-label>)" \
BASETEN_API_KEY="$(corbanu vault auth-helper <baseten-label>)" \
DEEPSEEK_API_KEY="$(corbanu vault auth-helper <deepseek-label>)" \
KIMI_API_KEY="$(corbanu vault auth-helper <kimi-code-label>)" \
CORBANU_PLAN_API_KEY="$(corbanu vault auth-helper <corbanu-plan-label>)" \
MODEL_API_KEY="$(corbanu vault auth-helper <meta-label>)" \
  python3 benchmarks/coding/runner.py --config "$OUT/configs/coding-matrix.json" run --confirm-paid-run
```

The `claude-plan` routes sign in with `CLAUDE_CODE_OAUTH_TOKEN` (a fresh home
has no saved login, and the keychain must not prompt mid-run); confirm on the
canary pattern (one task) that every provider authenticates from a fresh home
before the matrix. If the cumulative calculated spend reaches 1.5 × the matrix
estimate, stop the runner by its PID and record the cutoff.

**Competitive live task** (after access and freeze): export the base commit to
a clean directory, add the frozen task to a copy of `hermes-efficiency.json`
with `"tasks": [{"name": "<task>", "baseline": "<export dir>", "prompt":
"<frozen prompt>", "verifier": "<frozen rubric verify.py>", "timeout_seconds":
3600, "visible_command": [<repo test command>]}]`, `"waves": 3`, a new
`run_dir`, and `max_total_runs: 9`, then run it like the efficiency campaign
(est. $13.93). The runner caps visible and hidden checks at 300 s each; if the
repository's tests take longer, put them in the rubric verifier's own budget.

**Website builder** (optional, est. $41): run `run_pair.py` as in
`benchmarks/website-builder/README.md`, passing keys as
`--corbanu-anthropic-key-file <(corbanu vault auth-helper <label-a>)` (the
helper runs inside the consuming command and nothing is written to disk; likewise
for the Claude Code and OpenAI keys), and `--claude-bin` pointing at a wrapper
that runs the real Claude Code under
`sandbox-exec -p '(version 1)(allow default)(deny mach-lookup (global-name "com.apple.SecurityServer") (global-name "com.apple.securityd.xpc") (global-name "com.apple.securityd.systemkeychain"))'`.

## 5. Diffs, scan, tally, decide

```bash
python3 - "$OUT/configs" "$OUT/runs" <<'PY'
import json, pathlib, subprocess, sys
configs, runs = map(pathlib.Path, sys.argv[1:])
baselines = {t["name"]: t["baseline"] for c in configs.glob("*.json") for t in json.loads(c.read_text())["tasks"]}
for summary in runs.glob("*/results/*/*/*/summary.json"):
    run = json.loads(summary.read_text())
    out = subprocess.run(["diff", "-ruN", "-x", "__pycache__", baselines[run["task"]], run["workspace"]],
                         capture_output=True, text=True).stdout
    (summary.parent / "final.diff").write_text(out)
PY
```

```bash
python3 benchmarks/scan_exact_keys.py --path benchmarks --path "$EVID" --path "$OUT" \
  --key-file <(corbanu vault auth-helper <openrouter-bench-label>) \
  --key-file <(corbanu vault auth-helper <openai-label>) [... one per key used] \
  --output "$OUT/scan.json"            # hit_count must be 0
```

Per run, take tokens (input, cached input, output) and cost from: Corbanu
`turn.completed` usage in its stdout JSONL; Hermes `hermes.usage.json`; Kilo's
JSON step events. Price missing costs with the frozen prices; never record
unknown spend as zero. Then apply the decision rule in `frozen-config.json`
`hermes_efficiency.decision_rule`:

```python
import math, random
# pairs: {task: [(corbanu_cost, hermes_cost), ...one per wave...]}, both runs finished with known spend
def gm(rs): return math.exp(sum(math.log(r) for r in rs) / len(rs))
def ratios(tasks): return [c / h for t in tasks for c, h in pairs[t]]
est = gm(ratios(list(pairs)))
rng, names, boots = random.Random(2717), list(pairs), []
for _ in range(10_000):
    boots.append(gm(ratios([rng.choice(names) for _ in names])))
boots.sort(); lo, hi = boots[249], boots[9_750]   # 250th smallest / 250th largest
# claim only if the correctness gate holds and hi <= 0.95; regression if lo > 1.05; otherwise no claim
```

Write the per-pair table, component verdicts and final verdict under `$EVID`,
copy the OpenRouter price snapshot and `scan.json` there, and update the ledger
row in `benchmarks/README.md`.

## Spend estimate (frozen prices)

| Harness / part | Runs | Estimate |
| --- | ---: | ---: |
| Corbanu, coding matrix (40 routes; `muse-spark-1.1` unpriced) | 440 | $289.90 |
| Corbanu, shared route (efficiency + live) | 33 + 3 | $21.67 |
| Hermes, shared route (efficiency + live) | 33 + 3 | $21.67 |
| Kilo, shared route (efficiency + live) | 33 + 3 | $21.67 |
| Website builder, Corbanu + Claude Code + judge | 6 | $41.00 |
| **Total** | | **$395.91** |

The Claude plan routes ($77.29 of the matrix) draw on plan capacity rather
than cash.

## Dry-run record

Prepared 2026-11-23 on the Mac against the installed signed build
(`~/.local/bin/corbanu` → `INSTALL.json` `launcher_target`, corbanu 0.1.49,
SHA-256 `896684ea…d082f`), with no model or provider call:

- `runner.py plan` on `all-tasks.example.json`: 11 tasks including QueueCraft;
  Corbanu, Hermes and Kilo all resolve when the pinned installs are on `PATH`.
- `make_run_configs.py refresh --check`: current; 40 routes, 440 matrix runs.
- Hermes-efficiency dry run: 99 runs (11 tasks × 3 waves × 3 harnesses), and
  coding-matrix dry run: 440 runs (11 × 40). Every run: stub exit 0; fresh
  per-run `HOME` plus the harness's own home (Corbanu `CODEX_HOME` =
  `CORBANU_HOME`, `PFTERMINAL_HOME` unset; Hermes under `profiles/`, with OpenAI
  SDK 2.54.0 available; Kilo XDG directories and `KILO_CONFIG`), all created;
  source integrity held; visible and hidden verifiers executed and failed the
  unsolved baselines; `summary.json` and `report.md` written; the step 5 diff
  script wrote `final.diff` for all 539 runs (diffs include the tests a
  debugging packet's verifier restores). Versions seen: corbanu 0.1.49, Hermes
  v0.23.3, Kilo 7.8.32.
- Website builder dry run (stubs, dummy keys, Playwright 1.65.2 Chromium):
  route evidence verified for both lanes, the browser verifier wrote
  `verification.json` and three captures per lane and failed the unsolved
  baseline; the series stopped after wave 1 as designed. Real Claude Code was
  not run.
- `scan_exact_keys.py` over `benchmarks/`, this directory and every dry-run
  artifact (14,009 files, 69 MB) with the dummy keys and the one provider key
  present in the worker environment: 0 hits.
- `benchmarks/coding`, `benchmarks/website-builder` and scanner unit tests pass.
