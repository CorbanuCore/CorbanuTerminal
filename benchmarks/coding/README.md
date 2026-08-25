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
