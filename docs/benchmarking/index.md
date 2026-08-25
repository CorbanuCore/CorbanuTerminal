# Benchmarking

## The pain

A terminal can pass unit tests and still become worse at real work. It can route
the wrong model, stall inside a provider, produce a technically valid but poor
website, fail a long coding task, or look successful in a non-interactive smoke
test while the true TUI is broken.

Corbanu therefore uses several benchmark families. They answer different
questions and are not interchangeable.

## Benchmark map

| Benchmark | Question answered | Harnesses compared or exercised | Construction | What it can prove |
| --- | --- | --- | --- | --- |
| Competitive live-repository benchmark | Has Corbanu retained real-world agent performance against peers? | Corbanu Terminal, Hermes, and Kilo Code | One frozen task in TensorCash or Isometric Game; identical base commit, prompt, rubric, permissions, tools, and cutoff; one disposable worktree per lane | The live-repository portion of the three-release gate |
| [Website builder benchmark](website-builder.md) | Which coding harness produces the stronger finished website with the same model? | Corbanu Terminal and Claude Code; OpenAI visual judge; deterministic Playwright verifier | Byte-identical baseline and prompt, isolated homes and keys, concurrent contestant lanes, browser interaction checks, six matched captures, blind balanced A/B judging | Website capability, visual quality, validity, wall time, and attributable cost |
| [Coding performance benchmark](coding-tests.md) | Has correctness, runtime, or spend regressed across relevant production models? | Required gate: Corbanu Terminal across the frozen relevant model set; optional diagnostic campaigns may also use Hermes, Kilo Code, Codex, and Claude Code | Every third release uses the full checked-in task catalog, including QueueCraft, with fresh candidate trees, external verifiers, route evidence, wall time, and actual or calculated spend | The coding-performance portion of the three-release gate |
| Claude pane workflow suite | Do headless Claude-pane providers complete representative workflows and preserve evidence? | One or more configured Claude-pane provider profiles | Product-owned fixtures for a tiny website, NumPy/Pandas timing, code review, and auditability | Pane/provider integration regression evidence |
| Native-provider TUI benchmark | Does a native provider complete two turns through a real PTY-driven TUI? | One Corbanu native-provider route per run | Launches the TUI in a PTY, sends prompt and Enter as separate key events, finds the persisted rollout, and records completions, tool calls, timing, and tokens | True-TUI provider continuity and persistence evidence |
| Hammer-reduction benchmark | Does local backoff stop repeated provider requests after a rate limit? | Corbanu against a local mock provider | Returns a controlled HTTP 429, runs two requests, and verifies that the second is blocked locally | A narrow rate-limit regression, not agent quality |

## Three-release benchmark cycle

Every third release requires both the competitive live-repository component and
the full coding performance matrix. In the competitive component, Corbanu,
Hermes, and Kilo Code receive one frozen development task against separate
disposable worktrees from the same TensorCash or Isometric Game base. In the
performance component, Corbanu runs every checked-in coding task across the
frozen relevant production model set. The evidence records correctness,
end-to-end runtime, and spend for every required pair.

`benchmarks/README.md` is the sole authority for cadence, the exact freeze
requirements, verdict definitions, the current ledger, and the required release
evidence. Do not decide release readiness from this overview.

## Evidence and interpretation

Keep these dimensions separate:

| Dimension | Evidence |
| --- | --- |
| Correctness | Frozen rubric, visible tests, external verifier, browser assertions, or live-repository acceptance checks |
| Quality | Blind visual judgment or a task-specific rubric |
| Reliability | Pass rate across independent waves and valid-lane count |
| Routing | Provider, model, endpoint, and request or rollout evidence |
| Performance | End-to-end wall time and declared regression thresholds |
| Cost | Provider billing deltas or native usage priced from a recorded source |
| Safety | Spend cutoff, permissions, exact-key scan, and P0 security observations |

A visually preferred site can still be invalid. A fast coding run can still be
wrong. A passing synthetic task and a product-owned diagnostic answer narrower
questions than a live TensorCash or Isometric Game comparison.

## Product-owned code references

| Surface | Code reference |
| --- | --- |
| Benchmark policy, performance matrix, harness ownership, and public ledger | `benchmarks/README.md` |
| Release evidence contract | `AGENTS.md` and `qa/release/<version>/benchmarks/` |
| Coding harness and task packets | `benchmarks/coding/` |
| Website builder harness | `benchmarks/website-builder/` |
| Exact-key scanner | `benchmarks/scan_exact_keys.py` |
| Claude pane workflow command | `codex-rs/cli/src/main.rs::ClaudePaneWorkflowSuiteCommand` |
| Claude pane suite orchestration and reports | `codex-rs/tui/src/claude_panes/smoke.rs::run_claude_pane_workflow_suite` |
| Claude pane workflow implementations | `codex-rs/tui/src/claude_panes/smoke_workflows.rs` |
| Real-PTY provider benchmark | `scripts/native-provider-tui-benchmark` |
| Rate-limit hammer diagnostic | `scripts/hammer-reduction-benchmark` |

The reusable website and coding harnesses, task packets, fixtures, verifiers,
and scanners live under `benchmarks/` in the Corbanu repository. Credentials,
virtual environments, generated workspaces, and historical result trees do not.
`benchmarks/README.md` governs how any harness may contribute to release
evidence.
