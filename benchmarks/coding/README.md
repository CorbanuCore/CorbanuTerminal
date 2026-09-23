# Coding benchmark harness

This directory owns Corbanu Terminal's reproducible synthetic coding benchmark
runner and unsolved task packets. Historical run outputs remain outside the
source tree; new run artifacts go under the ignored `runs/` directory or a
release evidence directory chosen in the config.

This harness does not define release policy. The three-release cadence,
required full-catalog performance matrix, relevant-model-set rule, verdicts,
and ledger remain in `benchmarks/README.md`.

## What it runs

The runner has built-in argv adapters for:

- Corbanu Terminal;
- Hermes;
- Kilo Code;
- Codex;
- Claude Code.

A config can also provide an explicit argv template. Commands are executed
directly without a shell. Agent processes inherit credentials from the
environment; configs record only required environment-variable names, never
secret values.

Agents in one billing `lane` are serialized. Independent lanes run
concurrently. Every task/agent/wave combination gets a fresh candidate tree,
isolated agent state, stdout/stderr, independent visible and verifier results,
test-integrity evidence, route observations when exposed, source-tree
integrity, and a summary. Any harness or task-source mutation skips external
verifier execution and fails the lane.

## Dry plan

The small checked-in example resolves the EventForge and LogTriage packets:

```bash
python3 benchmarks/coding/runner.py \
  --config benchmarks/coding/configs/example.json \
  plan
```

Use `benchmarks/coding/configs/all-tasks.example.json` to inspect the full task
catalog. A plan is read-only and makes missing binaries visible. Live execution requires
both credentials in the environment and an explicit paid-run acknowledgement:

```bash
python3 benchmarks/coding/runner.py \
  --config benchmarks/coding/configs/example.json \
  run --confirm-paid-run
```

The runner refuses to reuse a nonempty run root or existing workspace. Adjust
`run_dir` for every fresh campaign.

For the required every-third-release campaign, derive a release-owned config
from the full catalog, add one Corbanu agent entry for every route in the frozen
relevant model set, raise `max_total_runs` to the exact bounded matrix size, and
write results under `qa/release/<version>/benchmarks/`. The release record must
reconcile missing native cost with recorded tokens and its frozen pricing
source; missing spend is not zero and does not satisfy the gate.

## Leak-isolated OpenRouter campaigns

Without isolation, contestants run as the operator with full disk and network access.
They can read the verifier and `tests_pristine/` next to their workspace, earlier
waves' solved workspaces, the operator's agent memories, rules and skills, and the
public GitHub copy of every task packet. An `isolation` block closes those routes:

- each run starts a fresh container from a pinned contestant image
  (`isolation/Dockerfile`, built by `isolation/build_image.sh`) that contains only
  the three CLIs;
- the container mounts exactly two host paths: the fresh candidate at
  `/workspace` and an empty agent home at `/home/bench`. Task packets, verifiers,
  other runs, and operator homes are not present;
- the container's environment is exactly the per-run env file. Operator
  variables are never inherited;
- the container uses an internal Docker network with no internet or host route.
  `openrouter.ai` resolves to the campaign relay (`isolation/relay.py`), which
  presents a campaign-only CA certificate. Every harness therefore uses its
  unmodified native OpenRouter provider, while the real key stays in the relay;
- the relay accepts only per-run tokens and forwards only inference plus
  read-only catalogue calls. It rejects any model other than the frozen one and
  any OpenRouter server-side web plugin or tool. Every request body, response,
  generation id and `usage.cost` is recorded;
- web tools are disabled identically in all harnesses. The network boundary
  would block them anyway;
- `leak_audit.py` runs before any spend. It fails if a candidate contains the
  verifier and reports which hidden-test strings the prompt quotes. After each
  run it scans every forwarded request for hidden-test-only strings and
  published-source references.

Prompts are the frozen task files, byte for byte, for every harness. The
harness adds nothing. Each harness keeps its own defaults, including system
prompt, tools and reasoning settings; the relay records them for disclosure.

```bash
# 1. Pin and build the contestants (outside this repository).
benchmarks/coding/isolation/build_image.sh 0.1.44 v2026.9.21 7.7.9 /data/scratch/bench-image
# 2. No-spend checks.
export CORBANU_BENCH_RUNS=/data/scratch/bench-runs   # outside any git checkout
python3 benchmarks/coding/runner.py --config benchmarks/coding/configs/openrouter-three-harness.json audit
python3 benchmarks/coding/runner.py --config benchmarks/coding/configs/openrouter-three-harness.json plan
# 3. Paid run. OPENROUTER_API_KEY is read only by the relay.
OPENROUTER_API_KEY=... python3 benchmarks/coding/runner.py \
  --config benchmarks/coding/configs/openrouter-smoke.json run --confirm-paid-run
```

Lanes are models: the three model lanes run concurrently, and within a lane
the harnesses are interleaved wave by wave so they face the same provider
conditions. Cost comes from OpenRouter's per-generation `usage.cost` recorded
by the relay; generation ids are kept for reconciliation. Toothpaste Site is
excluded because it requires the OpenAI Image API rather than OpenRouter.

Residual risk: the task packets are public in this repository, so a model
trained on it could have memorized hidden tests. Isolation cannot remove that.
The relay records make it measurable: contestant output that reproduces
hidden-test-only strings appears under `hidden_literals_in_contestant_output`.
For headline claims, prefer freshly authored private tasks.

## Task packet

Each entry under `tasks/` contains:

- an unsolved `baseline/` or deliberately `bugged/` candidate;
- `task_prompt.md` or `BENCHMARK_TASK.md`;
- visible tests inside the candidate;
- an external `verifier/verify.py`;
- `tests_pristine/` for debugging packets where the verifier restores the
  complete test set.

Solved candidates and reference implementations are intentionally excluded from
the canonical harness.

## Safety and interpretation

The runner requires an explicit live-run flag, isolates agent homes, enforces a
maximum number of processes, records timeouts, and never writes credential
values to config or manifests. Run the repository-level exact-key scanner after
every paid campaign:

```bash
python3 benchmarks/scan_exact_keys.py \
  --path benchmarks/coding \
  --path benchmarks/coding/runs/example \
  --key-file /path/to/provider-key
```

Synthetic correctness, time, route, and native-cost observations remain
separate. A synthetic pass does not replace true-TUI QA or the live TensorCash
or Isometric Game competitive benchmark.
