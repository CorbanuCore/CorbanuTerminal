---
title: "Arbitrary-model Autoreview"
status: draft
change_class: product-initiative
priority: P1
owner: "Alex Good — Head of Product"
activation_authority: "Travis Good — final product authority"
activation_basis: "2026-08-24 request to propose explicit Autoreview with arbitrary configured models; implementation remains pending activation"
target_release: "TBD"
deadline: "TBD"
created: 2026-08-24
updated: 2026-08-24
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "Shipping MVP — LIVE"
  requirement_excerpt: "Agent orchestration: Sauron → Nazgul → Troll → Orc orchestration, model-aware delegation, durable mailboxes, supervision, resume, and recovery."
implementation_worktrees: []
---

# Arbitrary-model Autoreview

Policy: repository-root `AGENTS.md`

Plan lifecycle: `docs/plans/index.md`

This is a **proposed plan**. It does not consume an active-plan slot, authorize
implementation, change the active P0 security sequence, or describe a finished
feature.

## Activation record

| Field | Value |
| --- | --- |
| Status | **Draft / proposed** |
| Active-plan slot | None |
| Product authority | Travis Good |
| Product owner | Alex Good |
| Delivery owner | Jim Ricketts |
| Authoritative decision | Produce an executable proposal for explicit Autoreview on an arbitrary configured Corbanu provider/model |
| Activation gate | Explicit product authorization, an available plan slot, exact worktree coordinates, and a dependency decision for the secret scanner |
| Target release | TBD |
| Deadline | TBD |

## User pain

Corbanu can already delegate to several model providers, but a user cannot start
one hardened code-review workflow and know that the exact requested reviewer
received the complete intended change, no secret crossed the provider boundary,
the reviewer had no write authority, and the result came back in one consistent
format. The inherited `/review` flow changes the model inside the parent
provider, while model-authored cross-provider spawning has transport,
full-history, and authentication gaps. Users must currently assemble those
pieces by hand and cannot trust a repeatable result.

## Product intent and ideal flow

The user explicitly invokes `$autoreview` from a repository and selects a
target (`local`, `branch`, or `commit`), provider, model, reasoning effort,
and finding threshold. If provider or model is omitted, a host-owned picker asks
for it; the agent never guesses.

Corbanu validates the target and provider readiness before creating a reviewer.
It then builds the exact review packet, including untracked content and deleted
diff lines, writes it to owner-only temporary storage, and scans the exact bytes
that would leave the machine. Missing or failing scanners, detected credentials,
an unauthorized provider, invalid model, missing authentication, or an
oversized unpartitionable packet stops before any reviewer call.

A reviewer runs on the exact selected runtime with no inherited conversation,
repository write access, shell, web, MCP, skill, or subagent tools. Corbanu shows
the selected provider/model, pass progress, elapsed time, and cancellation
control. Normal inputs run once; large inputs may be divided into at most eight
complete bounded packets without truncation. Corbanu validates and merges the
structured result, records route and usage evidence, and presents advisory
findings. It never edits code or starts another review unless the user explicitly
asks.

## Product linkage

| Field | Value |
| --- | --- |
| Exact product-spec heading | `Shipping MVP — LIVE` |
| Requirement excerpt | “Agent orchestration: Sauron → Nazgul → Troll → Orc orchestration, model-aware delegation, durable mailboxes, supervision, resume, and recovery.” |
| Product outcome advanced | Model-aware delegation becomes an exact, safe, user-visible review workflow across every configured and authorized model provider |
| North-star criterion advanced | A user can obtain a second-model code review without manually moving diffs, credentials, or provider configuration |

## Source analysis and disposition

The design source is OpenClaw's MIT-licensed
[Autoreview skill at commit `128a4ea6`](https://github.com/openclaw/agent-skills/tree/128a4ea60a93badf0d6d8f3381057f93f7085b29/skills/autoreview).
The analysis covered its
[`SKILL.md`](https://github.com/openclaw/agent-skills/blob/128a4ea60a93badf0d6d8f3381057f93f7085b29/skills/autoreview/SKILL.md),
[`scripts/autoreview`](https://github.com/openclaw/agent-skills/blob/128a4ea60a93badf0d6d8f3381057f93f7085b29/skills/autoreview/scripts/autoreview),
and hardening tests.

| Upstream contract | Corbanu decision |
| --- | --- |
| Explicit invocation only | Require a host-recognized `$autoreview` selection; never run ambient or Guardian approval review |
| Exact engine/model and one bounded run | Require an exact configured provider/model/effort and forbid silent fallback or automatic reruns |
| Complete validated bundle, including untracked and deleted content | Build one canonical packet in Core and partition only at complete section/file boundaries |
| Exact outgoing packet scanned with TruffleHog; missing/error/finding fails closed | Preserve the fail-closed scan and add Corbanu exact-key canary coverage |
| Isolated reviewer workspace and sanitized environment | Give the native reviewer only the packet and structured-output channel; expose no tools or inherited conversation |
| Structured, advisory findings | Validate the schema, merge bounded packets deterministically, and never auto-apply |
| Fixed external CLI engines | Replace with Corbanu's configured provider/model catalog and native runtime; do not retain an engine allowlist |

Corbanu will adopt the contract, not copy the fixed external-CLI implementation.
The repository skill will be a thin, original adapter to a host-owned Core
service and will record the upstream source, commit, and license. No modified
partial copy of the upstream skill directory is permitted. If upstream code is
later reused, it must be synced as a complete canonical directory and kept
separate from Corbanu-specific integration code.

## Existing boundaries and required repairs

| Boundary | Current evidence | Required repair |
| --- | --- | --- |
| Existing code review | `codex-rs/core/src/session/review.rs::spawn_review_thread` selects `review_model` but retains the parent provider; `codex-rs/core/src/tasks/review.rs` owns the inherited review task | Keep `/review` behavior separate; Autoreview owns exact provider and model selection |
| OpenAI cross-provider dispatch | `codex-rs/core/src/tools/spec_plan.rs::spec_for_model_request` applies the reserved OpenAI collaboration schema, which omits Corbanu runtime fields; the plaintext adapter is described as a retry after native failure | Add one host-side exact-runtime dispatch path that selects message encoding after resolving the target; never make the model perform a failed probe/retry |
| Full-history runtime override | `codex-rs/core/src/tools/handlers/multi_agents_v2/spawn.rs::handle_spawn_agent` rejects only role override for full history and can continue through model/effort override logic | Reject provider, model, reasoning, service-tier, and role overrides when `fork_turns=all`; Autoreview always uses no parent history |
| Provider authentication | `codex-rs/tui/src/spawn_orchestration.rs::ensure_native_spawn_provider_ready` checks auth in the TUI, while Core spawn validates authorization and eligibility without the same readiness gate | Move readiness into shared Core policy and call it from TUI, model-authored spawn, and Autoreview before child creation |
| Review naming | Guardian uses `auto_review` for approval routing and `/review` is an inherited review mode | Use `Autoreview` only for this explicit skill workflow; do not change Guardian or `/review` semantics |

The three routing, full-history, and authentication repairs are mandatory feature
work, not optional cleanup.

## Feature register

| ID | Feature | User problem solved | Product contract |
| --- | --- | --- | --- |
| PF-14 | Arbitrary-model Autoreview | A user needs a hardened second-model review on any configured and authorized provider without manually transferring code or trusting model-selected routing | One explicit request produces one secret-scanned, isolated, exact-runtime, structured advisory review with visible status and no automatic edits |

## Scope

### In

- A repository `autoreview` skill with explicit invocation and a structured
  Core request.
- `local`, `branch`, and `commit` targets; optional repository-relative
  prompt and dataset inputs.
- Any provider/model/effort combination present in Corbanu's eligible catalog
  and authorized by operator policy.
- Complete bounded packet construction, owner-only temporary files,
  TruffleHog and exact-key scanning, and at-most-eight-pass partitioning.
- Shared Core provider readiness, deterministic cross-provider dispatch, and
  fail-closed full-history override validation.
- Tool-free reviewer isolation, environment sanitization, structured result
  validation, deterministic merge, progress, cancellation, and durable result
  inspection.
- Focused regressions, true-TUI tests, live TensorCash and Isometric Game
  qualification, finished-feature documentation, and release evidence.

### Out

- Automatic or background review without an explicit user invocation.
- Automatic application of findings, code changes, approvals, signing,
  brokerage, wallet, or financial actions.
- Review panels, multi-model voting, or fallback chains inside one request.
- Adding providers, credentials, entitlements, or bypassing
  `agents.provider_allowlist`.
- Changing the product meaning of inherited `/review` or Guardian
  `auto_review`.
- Copying or maintaining the upstream external-CLI helper as Corbanu's runtime.

## Invariants

- The selected provider, model, effort, target, and threshold are recorded
  before dispatch and never silently changed.
- A third-party route is identified before use; provider privacy and billing
  remain visible.
- No provider call occurs unless the exact outgoing bytes pass all required
  scanners.
- The reviewer receives no credentials, inherited conversation, repository
  filesystem, shell, web, MCP, skill, plugin, or subagent access.
- Autoreview uses `fork_turns=none`; generic full-history forks reject all
  runtime and role overrides.
- Bundle partitioning never truncates content and never exceeds eight calls.
- Every response is schema-validated; malformed output fails visibly and is not
  converted into a successful empty review.
- Findings are advisory and cannot mutate the worktree.
- Cancellation stops remaining provider calls; interrupted work does not
  silently resume or rerun.
- Existing `/review` and Guardian `auto_review` remain behaviorally distinct.

## Ownership and implementation worktrees

| Owner | Worktree | Branch | Base commit | Scope |
| --- | --- | --- | --- | --- |
| Jim Ricketts | UNALLOCATED | UNALLOCATED | UNALLOCATED | PF-14 implementation after activation |

## Useful code references

| Path or symbol | Why it matters |
| --- | --- |
| `.codex/skills/` | Repository skill conventions; planned `.codex/skills/autoreview/SKILL.md` |
| `codex-rs/core/src/session/review.rs::spawn_review_thread` | Existing same-provider review boundary that Autoreview must not silently reuse |
| `codex-rs/core/src/tasks/review.rs::start_review_conversation` | Existing review isolation and structured-output behavior |
| `codex-rs/core/src/tools/handlers/multi_agents_v2/spawn.rs::handle_spawn_agent` | Current provider/model spawn and full-history bug boundary |
| `codex-rs/core/src/tools/handlers/multi_agents_common.rs` | Shared runtime resolution, allowlist, and planned readiness policy |
| `codex-rs/core/src/tools/spec_plan.rs::spec_for_model_request` | OpenAI reserved-schema boundary that currently strips runtime selection fields |
| `codex-rs/core/src/exec_env.rs::remove_provider_auth_env_vars` | Existing child-process credential sanitization |
| `codex-rs/tui/src/spawn_orchestration.rs::ensure_native_spawn_provider_ready` | TUI-owned auth checks to move into shared Core policy |
| `codex-rs/tui/src/app/thread_routing.rs` | Existing review request routing and planned Autoreview TUI event routing |
| `benchmarks/scan_exact_keys.py` | Existing exact-key scanner used for canary regression evidence |
| `docs/features/model-providers.md` | Finished provider and authentication contract |
| `docs/features/spawn-orchestration.md` | Finished model-aware delegation contract |

Planned native boundaries:

- `codex-rs/core/src/autoreview/{mod,request,bundle,runner,report}.rs`
- `codex-rs/core/src/tools/handlers/autoreview.rs`
- `codex-rs/core/tests/suite/arbitrary_model_autoreview.rs`
- `codex-rs/tui/src/chatwidget/autoreview.rs`

## Sprint execution map

All records map to the single PF-14 product feature. Each sprint owns one
mechanical code outcome and remains `draft` until this plan is activated and
worktree coordinates are assigned.

| Feature ID | Current sprint records | Completion evidence |
| --- | --- | --- |
| PF-14 | [PF-14-S01](../../sprints/current/arbitrary-model-autoreview/pf-14-s01-request-and-skill-contract.md)<br>[PF-14-S02](../../sprints/current/arbitrary-model-autoreview/pf-14-s02-review-packet-and-secret-gate.md)<br>[PF-14-S03](../../sprints/current/arbitrary-model-autoreview/pf-14-s03-core-provider-readiness.md)<br>[PF-14-S04](../../sprints/current/arbitrary-model-autoreview/pf-14-s04-exact-runtime-dispatch.md)<br>[PF-14-S05](../../sprints/current/arbitrary-model-autoreview/pf-14-s05-full-history-invariant.md)<br>[PF-14-S06](../../sprints/current/arbitrary-model-autoreview/pf-14-s06-isolated-review-runner.md)<br>[PF-14-S07](../../sprints/current/arbitrary-model-autoreview/pf-14-s07-tui-qualification-and-docs.md) | pending |

## Acceptance flows

| Flow | Starting state | User action | Expected visible result | Pass criterion |
| --- | --- | --- | --- | --- |
| Exact-runtime success | Authenticated, allowlisted non-parent provider; local changes | Invoke `$autoreview`, choose exact provider/model/effort and `local` target | Preflight summary, progress heartbeat, exact route, structured findings, usage evidence | Observed route matches request; packet is complete; worktree is unchanged |
| Missing selection | No provider/model supplied | Invoke `$autoreview` | Host picker requests both; no child starts | No model guesses or hidden default |
| Unauthorized or unauthenticated | Provider is absent from allowlist or lacks auth | Select that provider/model | Actionable failure before packet dispatch | No child, provider call, or secret-bearing artifact is created |
| Secret gate | Diff includes canary in added, deleted, prompt, dataset, or untracked content | Start Autoreview | Named refusal before send | Provider mock records zero calls |
| Invalid/full-history route | Runtime override is paired with `fork_turns=all` | Attempt exact-runtime spawn | Core rejects the request | No child is created on native or plaintext paths |
| Malformed result | Reviewer returns invalid or incomplete schema | Wait for completion | Visible failed result with preserved diagnostics | No false “no findings” success |
| Cancel | Review is running or has remaining bounded packets | Press Esc/cancel | Current run stops and remaining calls are skipped | Status is interrupted; no automatic rerun |
| Return and inspect | A prior run completed or failed | Reopen its pane/result | Route, target, timestamps, pass count, findings/error, and usage remain visible | No secret or raw auth value appears in persistence or logs |
| Existing-review regression | `/review` and Guardian approval review are configured | Use each existing path | Existing semantics remain unchanged | Existing suites and TUI snapshots pass |

## Implementation sequence

1. Define the explicit skill and typed request contract.
2. Build and harden the exact outgoing review packet.
3. Centralize provider authorization and authentication readiness in Core.
4. Add deterministic exact-runtime dispatch across provider transports.
5. Enforce the full-history runtime inheritance invariant.
6. Run and collect isolated, bounded, structured reviewer sessions.
7. Qualify the complete TUI flow and publish documentation only after it passes.

## Automated evidence

Run formatting before the final affected tests.

| Check | Final-tree command | Result | Artifact |
| --- | --- | --- | --- |
| Spawn routing regressions | `cargo test -p codex-core multi_agents_tests` | pending | linked from sprint evidence |
| Autoreview Core contract | `cargo test -p codex-core arbitrary_model_autoreview` | pending | linked from sprint evidence |
| Secret/adversarial packet tests | `cargo test -p codex-core autoreview_secret_gate` | pending | linked from sprint evidence |
| TUI behavior | `cargo test -p codex-tui autoreview` | pending | linked from sprint evidence |
| Plan and sprint records | `python3 docs/plans/check.py && python3 docs/sprints/check.py` | pending | CI log |
| Documentation | `mkdocs build --strict` | pending | CI log |

## True-TUI evidence

Launch with `RUST_LOG=trace just codex -c log_dir=<private-temp-log-dir>`.
For every prompt, send text and Enter as separate key actions. Corbanu
`exec` is not evidence.

| Flow | Candidate binary | Test repo/worktree | Keys/actions | Visible checkpoints | Result | Artifact |
| --- | --- | --- | --- | --- | --- | --- |
| Exact non-parent provider | candidate `corbanu` | TensorCash isolated worktree | Type explicit `$autoreview` request; Enter; select target/provider/model; Enter | Exact route, scan pass, progress, structured findings, unchanged diff | pending | trace + recording |
| Secret refusal | candidate `corbanu` | TensorCash canary worktree | Type explicit request; Enter | Refusal before provider activity; implicated file shown | pending | trace + provider-call ledger |
| Cancel | candidate `corbanu` | Isometric Game isolated worktree | Start review; Enter; press Esc during run | Interrupted state; no remaining packet starts | pending | trace + recording |
| Return/inspect | candidate `corbanu` | Isometric Game isolated worktree | Reopen review pane/result | Exact route and result remain visible; no secrets | pending | trace + recording |
| Existing-review regression | candidate `corbanu` | TensorCash isolated worktree | Run `/review`; exercise Guardian approval | Existing review labels and behavior remain distinct | pending | trace + recording |

## Live-repository applicability

| Repository | Applicable to this initiative? | Resolved checkout/test worktree | Base commit | Reason or result |
| --- | --- | --- | --- | --- |
| TensorCash | yes | pending | pending | Review a live Layer 1 change and a canary-bearing adversarial change |
| Isometric Game | yes | pending | pending | Review a mixed visual/code change; cancel and inspect persisted result |

## Human acceptance

| Tester | Date | Candidate version/commit | Flow | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| Product authority or delegate | pending | pending | Exact-runtime success, refusal, cancel, and result inspection | pending | linked recording and logs |

## Documentation

Finished-feature documentation is created only after the candidate passes.

| Finished-feature doc | Product-spec citation present | Verified candidate |
| --- | --- | --- |
| `docs/features/autoreview.md` | pending | pending |
| `docs/skills.md` Autoreview entry | pending | pending |
| `docs/features/spawn-orchestration.md` exact-runtime note | pending | pending |

## Dependencies, decisions, and blockers

| Item | Type | Owner | Needed by | State / decision |
| --- | --- | --- | --- | --- |
| Plan activation and exact worktree | authorization | Travis Good / Jim Ricketts | PF-14-S01 | pending |
| TruffleHog version and distribution on supported platforms | implementation dependency | Jim Ricketts | PF-14-S02 | pin and fail closed if absent |
| Operator provider allowlist and authentication | runtime precondition | user/operator | PF-14-S03 onward | never bypassed |
| Upstream Autoreview source provenance | design dependency | Jim Ricketts | PF-14-S01 | pinned to `128a4ea6`; native implementation, no partial copy |
| Benchmark cadence | release gate | release owner | candidate release | run full campaign if this is a due third release |

## Release linkage

- Release record: `qa/release/<version>/`
- Benchmark tracker row: required when the candidate is the third release since
  the last qualifying performance campaign
- Remaining blocker: draft status, activation, worktree allocation, and
  scanner-distribution decision

## Completion

- [ ] Product linkage, scope, invariants, and worktrees are current.
- [ ] Every implementation unit is represented by a valid single-feature sprint.
- [ ] All three identified spawn-boundary repairs pass regressions.
- [ ] Required final-tree automated evidence passes.
- [ ] Required true-TUI and live-repository evidence passes.
- [ ] Human acceptance passes.
- [ ] Finished documentation matches the candidate.
- [ ] Release and benchmark records are linked.
- [ ] No hard release gate remains pending.
