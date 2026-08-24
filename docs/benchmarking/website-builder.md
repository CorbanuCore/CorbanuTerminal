# Website builder benchmark

## The pain

A generated website can satisfy a file checklist while still being visually
weak, non-responsive, inaccessible, or functionally inert. A comparison is also
meaningless if contestants receive different models, prompts, baselines,
permissions, or judging conditions.

The repository-owned website builder benchmark controls those variables and
reports capability, visual quality, validity, time, and cost separately.

## What is compared

| Lane | Coding harness | Controlled model route |
| --- | --- | --- |
| Corbanu | Corbanu Terminal | Direct Anthropic `claude-opus-5` by default |
| Reference | Claude Code | The same direct Anthropic model |

Each lane uses a separate Anthropic key, isolated tool home, and fresh
workspace. Contestant lanes run concurrently. Attempts within a lane run
serially so they do not compete for one key's rate limit. The default campaign
uses three fresh attempts per lane.

OpenAI image-generation spend, visual-judge spend, verifier compute, and
preflight probes are experiment overhead. They are not silently charged to a
contestant.

## How the benchmark is constructed

### 1. Freeze the experiment

Before a paid wave, the harness records:

- exact contestant binary paths and hashes;
- exact prompt and baseline hashes;
- model and provider routes;
- isolated credential and tool-home assignments;
- permissions, timeout, wave count, and Claude Code budget;
- the deterministic verifier and visual judge implementation from the same
  repository commit.

The task asks each harness to build a polished static Corbanu Terminal marketing
site with an installation action, interactive orchestration explanation,
responsive desktop and mobile behavior, and exactly three local `gpt-image-2`
images recorded in a manifest.

### 2. Run independent lanes

Each attempt starts in a new candidate workspace. The harness may work within
the shared timeout and spend controls, but it cannot reuse another attempt's
files or tool state.

A lane is valid only when the agent exits successfully, the recorded model route
matches, the repository-owned benchmark source remains byte-identical,
deterministic verification passes, and the complete matched capture set exists.
A source mutation skips verifier execution and fails the lane. The harness does
not repair contestant output after cutoff.

### 3. Run deterministic verification

The verifier first checks the artifact tree:

- exactly three final generated images and a complete image manifest;
- the required image model and minimum dimensions;
- local image references with no remote runtime assets;
- no embedded secrets;
- required product concepts and benchmark interaction hooks;
- a usable static site or declared build output.

It then launches the site and uses Playwright at 1440 × 1000 and 390 × 844. It
rejects console, page, request, and horizontal-overflow failures. It proves that
interactions change real state: the install panel opens, copy feedback appears,
orchestration controls update their displayed state, and the mobile menu opens.
Before/after screenshots are compared so an unchanged page cannot masquerade as
a successful interaction.

The verifier captures six matched views for each site:

1. desktop hero;
2. desktop full page;
3. desktop install state;
4. desktop orchestration state;
5. mobile hero;
6. mobile menu state.

### 4. Run blind visual judging

The OpenAI judge receives only the six full-resolution captures for opaque Site
A and Site B. It does not receive source code, transcripts, cost, contestant
identity, lane-bearing filenames, or image prompts.

Every pair is judged twice with the A/B order reversed. If the two passes
disagree, the result is order-sensitive and has no visual winner. Functional
validity, visual judgment, wall time, and provider cost remain separate fields.

## Full benchmark versus the mock-website workflow

These are intentionally different tests.

| Surface | Purpose | Verification depth |
| --- | --- | --- |
| Full visual bakeoff | Compare finished website capability between coding harnesses | Frozen multi-lane experiment, route controls, generated-image contract, static checks, real browser interactions, six captures, blind balanced visual judge, cost and timing |
| Product `mock-website` workflow | Check that a Claude pane can create files and return a completion marker | Looks for `index.html`, the exact fixture text, either CSS or JavaScript, a successful pane turn, and `PFT_MOCK_SITE_DONE` |

The product workflow is useful integration evidence. It is not a website-quality
benchmark and must not be reported as the visual bakeoff.

## Code references

| Responsibility | Repository code reference |
| --- | --- |
| Harness operating guide | `benchmarks/website-builder/README.md` |
| Frozen contestant prompt | `benchmarks/website-builder/task_prompt.md` |
| Byte-identical baseline | `benchmarks/website-builder/baseline/` |
| Concurrent contestant lanes and wave records | `benchmarks/website-builder/run_pair.py` |
| Static and Playwright verifier | `benchmarks/website-builder/verify_site.py` |
| Balanced blind A/B judge and rubric | `benchmarks/website-builder/judge_pair.py` |
| Dependency lock range | `benchmarks/website-builder/requirements.txt` |
| Harness regression tests | `benchmarks/website-builder/tests/test_harness.py` |
| Shared exact-key scanner | `benchmarks/scan_exact_keys.py` |
| Product mock workflow | `codex-rs/tui/src/claude_panes/smoke_workflows.rs::run_mock_website_workflow` |
| Product suite runner | `codex-rs/tui/src/claude_panes/smoke.rs::run_claude_pane_workflow_suite` |
| Product CLI definition | `codex-rs/cli/src/main.rs::ClaudePaneWorkflowSuiteCommand` |

The earlier `benchmarks/coding/tasks/toothpaste_site/` task established the
three-image, desktop/mobile, verifier-plus-vision pattern. It remains a smaller
coding task, not the canonical full visual bakeoff described here.
