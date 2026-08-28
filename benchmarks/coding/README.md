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

## Isolation and pinning invariants

The 2026-08-28 investigation (`/home/pfrpc/MODEL_EVAL_HANDOFF_2026-08-28.md`)
found several contamination paths. The runner now enforces:

- **No wrapper binaries.** `corbanu`/`codex` agents must give an absolute
  binary path, and script wrappers (files starting with `#!`) are rejected. A
  `~/.local/bin` wrapper previously re-exported `CODEX_HOME` to the global
  Corbanu home and silently defeated per-run isolation.
- **Isolated environment.** Candidates get a private `HOME`, `XDG_*` tree, and
  `PYTHONNOUSERSITE=1`; `PYTHONPATH` is dropped; operator-home `PATH` entries
  are stripped; only `required_env`/`env_passthrough` variables cross the
  boundary. Verification subprocesses get the same treatment.
- **Explicit reasoning.** Corbanu agents must set `reasoning_effort` in the
  config. GLM 5.3 routes default to `max` preserved reasoning when the effort
  is left implicit. The outbound payload's `reasoning_effort`/`enable_thinking`
  are recorded and verified in each summary (`route_and_usage`).
- **Sandboxed candidates.** Corbanu/codex agents run under
  `--sandbox workspace-write` (plus `--ignore-user-config`) by default instead
  of `--dangerously-bypass-approvals-and-sandbox`. Set `"sandbox":
  "danger-bypass"` only when the environment is externally sandboxed.
- **Run roots outside git repositories.** A run root nested in a repo lets
  candidates inherit that repo's `AGENTS.md` and `.codex/skills`, which burned
  millions of cached input tokens in earlier campaigns. The runner refuses such
  roots unless `"allow_run_root_in_repo": true`.
- **Loop caps.** `caps.max_agent_commands` (default 120) and
  `caps.max_identical_commands` (default 12) kill runs stuck in
  inspect/edit/retest loops instead of letting them burn the full timeout.
- **Provenance.** `manifest.json` records the benchmark git commit/dirty state,
  runner and config SHA-256, and each agent binary's SHA-256; every
  `summary.json` repeats the binary hash and per-run isolation evidence,
  including whether the per-run `CODEX_HOME` actually received a session.

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
