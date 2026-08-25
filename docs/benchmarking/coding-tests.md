# Coding test benchmarks

## The pain

A coding agent can appear productive while leaving the contract broken,
editing the tests, using the wrong provider route, or succeeding only once by
chance. Coding benchmarks need reproducible tasks, clean workspaces, independent
verification, repeated attempts, and evidence that the requested harness and
model actually ran.

Corbanu owns the coding runner and task packets under `benchmarks/coding/`.
These synthetic tasks complement the live-repository competitive benchmark;
they do not replace it.

## What is compared

The portable runner has built-in command adapters for:

| Harness | Runner kind | Isolated state |
| --- | --- | --- |
| Corbanu Terminal | `corbanu` | Per-run `CODEX_HOME`, request evidence, stdout, and stderr |
| Hermes | `hermes` | Per-run `HERMES_HOME`, stdout, and stderr |
| Kilo Code | `kilo` | Per-run candidate workspace and JSON output |
| Codex | `codex` | Per-run `CODEX_HOME`, JSON output, stdout, and stderr |
| Claude Code | `claude-code` | Per-run `CLAUDE_CONFIG_DIR`, stream JSON, stdout, and stderr |

A config may supply an explicit argv template when a campaign needs a different
provider route or newly released CLI syntax. The runner invokes argv directly
without a shell.

For the release-gating comparison, the required competitors remain Corbanu,
Hermes, and Kilo Code as defined by `benchmarks/README.md`.

## How a coding task is constructed

A task packet contains:

- an unsolved baseline or deliberately bugged implementation;
- a frozen task prompt and public API or behavior contract;
- visible tests available to the contestant;
- a separate verifier with additional cases;
- pristine tests for debugging packets where the verifier restores the complete
  test set;
- a defined implementation path used for freshness evidence.

The harness copies the candidate into a fresh workspace and writes the prompt as
`BENCHMARK_TASK.md`. It records baseline hashes before starting. After the agent
exits or reaches the cutoff, visible tests and the external verifier run
independently. Test files are compared with the frozen baseline so an agent
cannot earn a pass by weakening, deleting, or adding tests. The runner also
hashes its source tree before execution. If an agent changes a prompt, verifier,
pristine test, task packet, or harness file, external verification is skipped
and the lane fails closed.

Solved candidates and reference implementations are deliberately excluded from
the repository-owned benchmark source.

## Required three-release performance campaign

At least every three releases, the release-gating campaign runs the full task
catalog—including QueueCraft—through Corbanu Terminal for every route in the
frozen relevant production model set. The release record fixes the task-catalog
digest, model/provider routes, reasoning settings, price source, runtime and
spend thresholds, and any product-authority-approved model exclusions before
execution begins.

Every required task/model pair must report correctness, end-to-end wall time,
timeout state, token use, and actual or reproducibly calculated spend. A missing
pair, unknown runtime, unknown spend, verifier failure, or crossed threshold
makes the performance campaign fail or remain incomplete and blocks the due
release. Missing spend is never recorded as zero.

The exact cadence, model-set rule, verdicts, evidence contract, and public
ledger are governed by `benchmarks/README.md`.

## Waves, lanes, and cost

Each task/agent pairing can be repeated across waves. Independent workspaces
make pass rate and variance visible instead of reporting a single lucky attempt.

The scheduler groups agents by billing lane. Agents sharing a billing endpoint
are serialized so attribution windows do not overlap; independent provider
lanes may run concurrently. Within a shared lane, agents are interleaved by
wave.

The harness records:

| Dimension | Measurement |
| --- | --- |
| Correctness | Visible result, external verifier result, and full pass |
| Reliability | Passes per independent wave |
| Route fidelity | Expected provider/model and observed model evidence when the CLI exposes it |
| Time | End-to-end wall seconds with timeout state |
| Cost | Native client cost when emitted; provider-billed reconciliation remains a separate artifact |
| Integrity | Baseline hashes, candidate hashes, test-tree comparison, explicit run cap, and exact-key scan |

Correctness, time, route, and cost remain separate. A cheap or fast failure does
not outrank a correct run, and missing route evidence is reported as unknown
rather than silently assumed to pass.

## Task families

Ad hoc diagnostic campaigns may use a subset. The required three-release
performance campaign must use every task in the checked-in full catalog.

| Task family | Contract shape | Repository references |
| --- | --- | --- |
| EventForge | Event-sourced financial ledger and monthly reporting | `benchmarks/coding/tasks/eventforge/task_prompt.md`; `benchmarks/coding/tasks/eventforge/verifier/verify.py` |
| LogTriage | Structured log parsing, deduplication, summaries, and time windows | `benchmarks/coding/tasks/logtriage/task_prompt.md`; `benchmarks/coding/tasks/logtriage/verifier/verify.py` |
| RateGate | Token-bucket and sliding-window rate limiting | `benchmarks/coding/tasks/rategate/task_prompt.md`; `benchmarks/coding/tasks/rategate/verifier/verify.py` |
| ChronoLedger | Append-only double-entry ledger, temporal balances, and WAL replay | `benchmarks/coding/tasks/chronoledger/task_prompt.md`; `benchmarks/coding/tasks/chronoledger/verifier/verify.py` |
| QueryForge | CSV query engine with expressions, joins, aggregation, ordering, and null semantics | `benchmarks/coding/tasks/queryforge/task_prompt.md`; `benchmarks/coding/tasks/queryforge/verifier/verify.py` |
| PipeFlow, ApiGate, ConfClerk, QueueCraft, and TextWright | Deliberately bugged implementations with visible failures, pristine tests, and verifier bug probes | `benchmarks/coding/tasks/<task>/bugged/`; `benchmarks/coding/tasks/<task>/tests_pristine/`; `benchmarks/coding/tasks/<task>/verifier/verify.py` |
| Toothpaste Site | Static ecommerce site with exactly three generated images and a mock checkout | `benchmarks/coding/tasks/toothpaste_site/task_prompt.md`; `benchmarks/coding/tasks/toothpaste_site/verifier/verify.py` |

The Toothpaste Site task is a compact coding packet. The dedicated website
builder benchmark adds controlled same-model lanes, stronger browser-state
proof, six matched captures, and a balanced blind visual judge.

## Code references

| Responsibility | Repository code reference |
| --- | --- |
| Harness operating guide | `benchmarks/coding/README.md` |
| Portable runner, lane scheduler, command adapters, verification, and reports | `benchmarks/coding/runner.py` |
| Configuration schema | `benchmarks/coding/config.schema.json` |
| Small Corbanu/Hermes/Kilo example campaign | `benchmarks/coding/configs/example.json` |
| Full task-catalog campaign | `benchmarks/coding/configs/all-tasks.example.json` |
| Unsolved and bugged task packets | `benchmarks/coding/tasks/` |
| Runner regression tests | `benchmarks/coding/tests/test_runner.py` |
| Shared exact-key scanner | `benchmarks/scan_exact_keys.py` |

## Relationship to the release benchmark

Synthetic coding tests are controlled and repeatable, which makes them useful
for the required runtime-and-spend regression matrix. The competitive
live-repository component asks a different question: can Corbanu, Hermes, and
Kilo perform one frozen development task against the same real TensorCash or
Isometric Game commit?

A qualifying three-release cycle requires both components. A synthetic pass
does not answer the live-repository question, and a live-repository pass does
not replace the full coding performance matrix.
`benchmarks/README.md` remains the sole authority for the competitive run
method, verdict, ledger, and evidence package.
