---
title: "GLM-5.3-Flash Vast GPU preset"
status: active
change_class: product-initiative
priority: P1
owner: "Jim Ricketts"
activation_authority: "Product owner"
activation_basis: "User request on 2026-08-26 to add a preconfigured /gpu setting and provision an inference endpoint."
target_release: "TBD"
deadline: "continuous"
created: 2026-08-26
updated: 2026-08-26
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "Shipping MVP — LIVE"
  requirement_excerpt: "Vast.ai and RunPod rental workflows with price, spend, duration, readiness, stop, and termination controls."
implementation_worktrees:
  - path: "/home/pfrpc/repos/CorbanuTerminal-glm53-flash"
    branch: "feat/glm-5-3-flash-vast-preset"
    base_commit: "1bdc515bff48a4d9048dae7d06c6214e884265bc"
---

# GLM-5.3-Flash Vast GPU preset

Policy: repository-root `AGENTS.md`

Plan lifecycle: `docs/plans/index.md`

## Activation record

| Field | Value |
| --- | --- |
| Status | **Active** |
| Active-plan slot | **2 of 2** |
| Product authority | Product owner |
| Authoritative decision | User request on 2026-08-26 to configure GLM-5.3-Flash through `/gpu` and provision a Vast endpoint |
| Target release | TBD |
| Deadline | continuous |

## User pain

A user can rent GPU capacity through `/gpu`, but GLM-5.3-Flash is not a curated
choice. Reconstructing its runtime flags, immutable artifacts, H200 topology,
authentication, context bound, and readiness contract by hand is error-prone
and can waste billable provisioning time.

## Product intent and ideal flow

The user opens `/gpu`, configures the Vast API key through masked Vault entry,
and selects a hardware-specific GLM-5.3-Flash preset. Corbanu offers the
qualified four-H200 profile and an experimental two-B300 FP8 profile, collects
hourly and total spend limits plus duration, shows compatible offers, and
performs the existing final billable confirmation. Each immutable vLLM recipe
starts an authenticated OpenAI-compatible server, validates the hardware and
runtime, downloads the pinned checkpoint, and exposes the endpoint only after
readiness succeeds. The B300 profile supports a controlled 4–256 stream
benchmark with mixed short and long contexts. Failure remains visible and
recoverable; termination continues until Vast confirms billing has stopped. A
ready endpoint appears in `/model` and can be called with its distinct
per-rental bearer credential.

## Product linkage

| Field | Value |
| --- | --- |
| Exact product-spec heading | **Shipping MVP — LIVE** |
| Requirement excerpt | “Vast.ai and RunPod rental workflows with price, spend, duration, readiness, stop, and termination controls.” |
| Product outcome advanced | One-click, bounded, authenticated GLM-5.3-Flash hosting through the existing rental lifecycle |
| North-star criterion advanced | A user can move from model selection to a qualified inference endpoint without assembling provider-specific launch commands |

## Scope

### In

- Retain the immutable four-H200 GLM-5.3-Flash vLLM recipe in the curated `/gpu` catalog.
- Add an experimental two-B300 native-FP8 recipe using the same pinned model and dedicated runtime image.
- Require allocation-local high-bandwidth interconnect for both hardware profiles.
- Pin the Hugging Face checkpoint and dedicated vLLM image by digest.
- Bound the H200 profile for Hopper BF16 KV-cache headroom and the B300 profile for a 131,072-token, 256-stream Blackwell FP8-KV evaluation.
- Measure the B300 profile at 4, 8, 16, 32, 64, 128, and 256 closed-loop streams with a deterministic mixed-context workload averaging 6,000 requested output tokens.
- Preserve Vault-backed Vast rental authentication and per-rental endpoint authentication.
- Add catalog, manifest, hardware, launch, and TUI regression evidence.
- Use the normal price, total-spend, duration, offer, final-confirmation, readiness, and termination flow for a live rental.

### Out

- Reusing the Vast API key as the inference endpoint credential.
- Bypassing `/gpu` spending confirmation or provider-confirmed termination.
- Claiming the checkpoint's one-million-token maximum on four H200s.
- Adding an unauthenticated public port, a second rental controller, or direct ad hoc Vast scripts.
- Adding SGLang support for this preset while vLLM has the current model-specific deployment recipe.
- Changing existing GLM-5.2, DeepSeek, RunPod, or model-selection behavior.

## Invariants

- No billable request occurs before exact hourly, total-spend, duration, offer, and final confirmation.
- The marketplace key remains Vault-backed and never enters recipe arguments, logs, chat, or endpoint authentication.
- Every rental gets a distinct generated endpoint token and readiness probes authenticate with it.
- The model revision and serving image are immutable; mutable tags cannot pass recipe validation.
- Three-way tensor parallelism is never selected for this 64-head model; the presets allocate TP4 on H200 and TP2 on B300.
- H200 uses BF16 KV cache; the launch must not request FP8 KV cache on Hopper.
- B300 uses the official recipe's Blackwell FP8 KV-cache path and requires CUDA 13 plus driver 580.65.06 or newer.
- The B300 preset remains experimental until live evidence establishes successful request counts, aggregate output throughput, per-stream throughput, and latency across the declared sweep.
- READY is the only state that makes the endpoint selectable through `/model`.
- Stop-serving never claims billing stopped; only provider-confirmed termination does.

## Ownership and implementation worktrees

| Owner | Worktree | Branch | Base commit | Scope |
| --- | --- | --- | --- | --- |
| Jim Ricketts | `/home/pfrpc/repos/CorbanuTerminal-glm53-flash` | `feat/glm-5-3-flash-vast-preset` | `1bdc515bff48a4d9048dae7d06c6214e884265bc` | Recipe, catalog/TUI regressions, true-TUI qualification, live Vast provisioning evidence |

## Useful code references

| Path or symbol | Why it matters |
| --- | --- |
| `codex-rs/gpu-market/src/recipe.rs::RecipeCatalog` | Owns curated immutable model presets and verified-manifest invariants |
| `codex-rs/gpu-market/src/vast.rs::VastProvider` | Searches compatible Vast offers and creates the authenticated instance |
| `codex-rs/gpu-market/src/controller.rs::GpuController` | Owns readiness, endpoint tokens, registration, health, spend, and termination |
| `codex-rs/tui/src/chatwidget/gpu_menu.rs` | Presents recipes and the existing three-limit plus final-confirmation flow |
| `codex-rs/tui/src/chatwidget/gpu_menu_tests.rs` | Owns user-visible GPU catalog and billing-flow regressions |
| `docs/features/gpu-rentals.md` | Finished behavior documentation after candidate qualification |

## Sprint execution map

| Feature ID | Plan feature | Current sprint records | State |
| --- | --- | --- | --- |
| `PF-27` | Curated GLM-5.3-Flash H200/B300 recipes, qualified endpoint, and B300 concurrency evidence | [PF-27-S01](../../sprints/current/glm-5-3-flash-vast-preset/pf-27-s01-curated-recipe-and-qualified-endpoint.md) | in progress |

## Acceptance flows

| Flow | Starting state | User action | Expected visible result | Pass criterion |
| --- | --- | --- | --- | --- |
| Primary success | Vast key exists; no GLM rental | Select the B300 GLM preset, enter limits/duration, choose offer, confirm | TP2 B300 rental progresses to READY and appears in `/model` | Authenticated completion succeeds and the 4–256 stream sweep records secret-free results |
| Failure/cancel | Preset selected before billable confirmation | Cancel or reject an invalid/over-limit offer | No provider create request and no rental charge | TUI returns safely and state contains no new billable rental |
| Recovery/resume | Provisioning or readiness is pending/degraded | Restart, open `/gpu status`, then terminate if needed | Durable stage/spend state resumes; termination remains pending until Vast confirms | No orphan endpoint token or false billing-stopped claim |

## Implementation sequence

1. Retain the immutable vLLM TP4/H200 recipe and add the experimental TP2/B300 FP8 profile with generalized launch/hardware regressions; live-canary argument-parser failures require a launch-revision bump and a generalized correction before benchmarking.
2. Add a reproducible mixed-context concurrency workload and update the `/gpu` catalog snapshot.
3. Build the final candidate and drive selection/cancellation through the true tmux TUI harness.
4. With user-approved spend limits and duration, rent on Vast, monitor READY, exercise the authenticated OpenAI-compatible API, sweep 4–256 streams, and record cleanup state.

## Automated evidence

Run fix and formatting tools before the final affected tests.

| Check | Final-tree command | Result | Artifact |
| --- | --- | --- | --- |
| Focused | `cd codex-rs && just test -p codex-gpu-market recipe` | 12 passed | [PF-27-S01 evidence](../../../qa/gpu-rentals/sprints/PF-27-S01/evidence.md) |
| Integration | `cd codex-rs && just test -p codex-gpu-market` | 77 passed | [PF-27-S01 evidence](../../../qa/gpu-rentals/sprints/PF-27-S01/evidence.md) |
| Workload | `python3 qa/gpu-rentals/benchmarks/glm53-b300/run_mixed_sweep.py --validate-only` | seven levels; weighted output exactly 6,000 tokens | [benchmark README](../../../qa/gpu-rentals/benchmarks/glm53-b300/README.md) |
| Snapshot | `cd codex-rs && just test -p codex-tui gpu_menu` | 9 passed | reviewed `insta` snapshot and [evidence](../../../qa/gpu-rentals/sprints/PF-27-S01/evidence.md) |
| Governance | `python3 docs/plans/check.py && python3 docs/sprints/check.py` | passed | [PF-27-S01 evidence](../../../qa/gpu-rentals/sprints/PF-27-S01/evidence.md) |

## True-TUI evidence

| Flow | Candidate binary | Test repo/worktree | Keys/actions | Visible checkpoints | Result | Artifact |
| --- | --- | --- | --- | --- | --- | --- |
| Catalog selection | `64034f2e8a` | this implementation worktree | `/gpu`, Enter | qualified 4×H200 and experimental 2×B300 presets visible | passed | [PF-27-S01 evidence](../../../qa/gpu-rentals/sprints/PF-27-S01/evidence.md) |
| Failure/cancel | `64034f2e8a` | this implementation worktree | `/gpu`, Enter, Esc | menu dismissed without provider search/create | passed | [PF-27-S01 evidence](../../../qa/gpu-rentals/sprints/PF-27-S01/evidence.md) |
| Primary live rental | final candidate | this implementation worktree | limits, offer, final confirmation | READY and authenticated completion | pending | live rental evidence |
| Recovery/resume | final candidate | this implementation worktree | restart; `/gpu status`, Enter | durable rental/readiness/spend state | pending | live rental evidence |

## Live-repository applicability

| Repository | Applicable to this initiative? | Resolved checkout/test worktree | Base commit | Reason or result |
| --- | --- | --- | --- | --- |
| TensorCash | no | not applicable | not applicable | GPU catalog and provider lifecycle do not depend on repository content |
| Isometric Game | no | not applicable | not applicable | GPU catalog and provider lifecycle do not depend on repository content |

## Human acceptance

| Tester | Date | Candidate version/commit | Flow | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| Product owner | pending | pending | Select preset, approve bounded Vast rental, call endpoint, terminate | pending | live rental evidence |

## Documentation

| Finished-feature doc | Product-spec citation present | Verified candidate |
| --- | --- | --- |
| `docs/features/gpu-rentals.md` | yes | pending update after qualification |

## Dependencies, decisions, and blockers

| Item | Type | Owner | Needed by | State / decision |
| --- | --- | --- | --- | --- |
| Runtime choice | decision | implementation owner | recipe | vLLM selected because its official recipe is current and model-specific; the live B300 canary is authoritative for accepted CLI flags |
| GPU topology | decision | implementation owner | recipe | retain four-H200 TP4 and add two-B300 TP2; 64 attention heads and 288 routed experts divide cleanly by both |
| Context limit | decision | implementation owner | recipe | H200 remains 65,536; B300 starts at 131,072 with FP8 KV cache and is validated empirically |
| Benchmark traffic | decision | implementation owner | live qualification | closed-loop concurrency sweep at 4–256 streams; 50% 1K/2K, 25% 8K/6K, 12.5% 32K/8K, and 12.5% 96K/20K input/output-token buckets, averaging 6K requested output tokens |
| Spend and duration | financial authorization | product owner | live rental | pending exact hourly cap, total cap, and duration through `/gpu` |
| Vast availability | external dependency | Vast.ai | live rental | verified 2×B300/NVLink inventory observed on 2026-08-26; revalidate immediately before confirmation because offers are volatile |
| Candidate release | release dependency | product owner | plan close | TBD |

## Release linkage

- Release record: pending under `qa/release/`
- Benchmark tracker row: not due until a release candidate is named
- Remaining blocker: live bounded rental, endpoint qualification, termination evidence, and human acceptance

## Completion

- [ ] Product linkage, scope, invariants, and worktrees are current.
- [ ] Every implementation unit is represented by a valid single-feature sprint.
- [ ] Required final-tree automated evidence passes.
- [ ] Required true-TUI and live-repository evidence passes.
- [ ] Human acceptance passes.
- [ ] Finished documentation matches the candidate.
- [ ] Release and benchmark records are linked.
- [ ] No hard release gate remains pending.
