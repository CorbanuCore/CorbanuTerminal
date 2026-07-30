# PF Terminal Upstream Codex Convergence Plan

Date: 2026-07-30

## Objective

Rebase PF Terminal's product extensions onto current upstream Codex while removing
PF-only runtime policies that were introduced without a provider requirement,
documented product requirement, or operator-visible configuration.

The end state is:

- Codex owns the coding-agent runtime, turn lifecycle, compaction, tool execution,
  permissions, native agent control, mailbox behavior, and terminal fundamentals.
- PF Terminal owns multi-provider model metadata, provider wire adapters, billing and
  capability metadata, vault and wallet features, PF-specific commands, branding, and
  packaging.
- A PF-specific runtime divergence must have an owner, evidence, tests, configuration
  policy, and an upstream incompatibility that explains why the divergence exists.
- On the OpenAI route, PF Terminal must remain behaviorally and economically equivalent
  to the same upstream Codex revision unless a documented PF feature is active.

## Verified starting point

- Released PF branch: `release/0.1.26-anthropic-endurance`
- PF commit: `5e79527205440cd0447200c1fcb191ad8140d0ea`
- Current upstream Codex commit checked on 2026-07-30:
  `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`
- Common ancestor: `d66708232299bdbf373ec55b0d6b938c246cfa60`
- Divergence from the common ancestor:
  - 493 commits on the PF side
  - 1,117 commits on the upstream side
- A merge-tree simulation produced 213 conflicted files:
  - 62 in `codex-rs/core`
  - 56 in `codex-rs/tui`
  - 24 in app-server code
  - 13 in model/protocol code
  - 21 in packaging and CI
  - 37 elsewhere

This is too much semantic overlap for a safe conflict-by-conflict merge. The integration
must be upstream-first and capability-oriented rather than commit-oriented.

## Governing rule

Do not remove a difference merely because it differs from Codex. Classify it first:

| Class | Action |
| --- | --- |
| Required PF product surface | Retain and port onto the upstream extension boundary. |
| Required provider compatibility | Retain near wire serialization or provider metadata, with live evidence and tests. |
| Upstream now has an equivalent | Delete the PF implementation and use upstream. |
| Unexplained runtime policy | Remove or disable by default. |
| Legitimate limit or spend policy | Put it in typed model/provider/runtime policy, expose it to the operator, and test it. |
| Historical compatibility | Preserve only when a released session, database, config, or CLI contract depends on it. |

The model catalogue is intentional PF scope. Runtime behavior must consume catalogue
metadata rather than rediscovering models through names, provider labels, or scattered
constants.

## Behavior disposition ledger

The first implementation PR must turn this table into a checked, code-referenced ledger.

| Current behavior | Disposition | Required replacement |
| --- | --- | --- |
| Anthropic `max_tokens = 32_000` for every model | Replace | Model-owned output budget from catalogue/provider metadata, with an operator override and a continuation test above 32K. |
| Five server-side continuations per turn | Remove | Upstream turn lifecycle plus a configurable total turn/spend budget. Never fail solely because a sixth productive continuation is needed. |
| Completion classifier sees only 4K objective characters and 6K response characters | Replace | Default to upstream completion semantics. Permit a provider-specific ambiguity adapter only for a route with reproduced evidence; retain structured evidence without silently judging a tail fragment as the whole turn. |
| Two incomplete responses without tool progress stop continuation | Remove as global policy | Provider/runtime stall detection based on canonical progress events and an operator-visible budget. |
| Anthropic web search fixed at eight uses | Parameterize | Model/provider tool policy with a visible default and per-session override. |
| Shell-command budget inferred from five regexes | Delete | Only structured CLI/config/tool-budget inputs may impose a runtime command limit. Natural-language instructions remain model-visible instructions. |
| Fourth identical tool call blocked | Reconcile with upstream | Use upstream tool-loop behavior. Add provider-specific malformed-call recovery only where live failures justify it. |
| Equivalent malformed tool call becomes fatal after one correction attempt | Reconcile with upstream | Canonical upstream error/retry lifecycle with bounded, visible provider policy where necessary. |
| Parent auto-processing pauses after three dispatch cycles | Replace | Upstream native agent mailbox and wait lifecycle. Any safety budget must be configurable, based on total work/spend, and announce why it paused. |
| Parent retains 12 child reports and truncates each to 12K characters | Replace | Durable upstream mailbox/history semantics. UI previews may be bounded, but full evidence must remain addressable. |
| Claude Plan injects `You are Claude Code` | Remove unless proven necessary | PF Terminal identity. If plan authentication requires a compatibility header or prompt, document the contract and scope it to the transport adapter rather than presenting it as product identity. |
| Anthropic request body reduced to 30 MB and 15 MB after 413 | Retain provider constraint, parameterize policy | Keep protection for Anthropic's real payload limit. Move budgets and image-retention order into typed provider policy; show omissions and preserve durable image references. |
| Third-party provider cooldown and cross-process lease | Retain with configuration | This has a written hammer-reduction design. Make TTL, cooldown, applicability, and override visible and test crash recovery. |
| Provider allowlist | Retain | It is explicit operator spend policy. Unset means every configured provider is eligible; the active policy must be visible in spawn diagnostics. |
| Model cost, billing class, vision, reasoning, and plan preference | Retain | One canonical catalogue consumed by picker, spawn tools, runtime resolution, and accounting. Unknown billing makes a model ineligible for automatic spawn rather than inventing a price. |
| Nazgul/Troll/Orc orchestration prompts | Make opt-in | Preserve the named `/orchestrate` workflow as a profile. Normal native agent spawning must use upstream Codex behavior plus model-aware runtime selection, without inheriting the hierarchy's managerial personality. |

## Integration strategy

### Phase 0: freeze and protect recovery

- [ ] Tag and archive the exact 0.1.26 source, binaries, checksums, config schema,
  database schema, and release evidence.
- [ ] Create `integrate/upstream-20260730` from upstream commit
  `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`.
- [ ] Do not develop the convergence work in the old dirty `PfTerminal` checkout.
- [ ] Copy every released PF database migration unchanged and verify its checksum before
  adding any new migration.
- [ ] Make fixtures from real 0.1.24, 0.1.25, and 0.1.26 homes, including the
  `pfterminal_state_5.sqlite` collision incident.
- [ ] Record all user-visible PF commands, config keys, provider IDs, model IDs, session
  metadata, and binary/package names as compatibility contracts.
- [ ] Keep 0.1.26 install artifacts available as the rollback release.

Exit gate: a convergence build can open copies of released PF homes read-only without
changing migration checksums or corrupting session discovery.

### Phase 1: establish a clean upstream runtime

- [ ] Build and run the exact upstream commit before any PF port.
- [ ] Freeze upstream's core, protocol, app-server, TUI, tool, compaction, permission,
  and native-agent test results as the convergence baseline.
- [ ] Import upstream changes as a coherent source state, not 1,117 individual
  cherry-picks.
- [ ] Adopt upstream's current context, compaction, streaming, permissions, tool runtime,
  and agent-control modules wholesale.
- [ ] Adopt current upstream fixes that are directly relevant to PF incidents, including:
  - delegated-task preservation across compaction (`4f6d06d48`);
  - model-owned token budget defaults (`fbe65995b`);
  - live-parent history for agent forks (`c5779ed6b`);
  - final messages in completion summaries (`80f3c3141`);
  - multi-agent settings preservation (`2f19a5770`);
  - configurable v2 subagent developer instructions (`49025589b`);
  - environment inheritance for spawned agents (`fe01054a2`);
  - resilient thread-history projection (`6256a7ccc`);
  - centralized tool registration (`89a0eed93`);
  - current permission-profile execution (`0dcad0c97`).

Exit gate: the branch behaves like unmodified upstream Codex on OpenAI before any
multi-provider code is introduced.

### Phase 2: port PF foundations without changing runtime semantics

Port these as separate, reviewable changes:

- [ ] Product names, `pfterminal` binary aliases, `~/.pfterminal` home separation,
  installers, release packaging, and update channels.
- [ ] Released database migrations and state compatibility.
- [ ] Vault credential storage and provider-key lookup.
- [ ] Wallet and PF-specific account surfaces.
- [ ] PF slash commands that do not alter the turn lifecycle.
- [ ] Branding and model-picker presentation.

Rules:

- [ ] No provider request shaping in this phase.
- [ ] No new core turn-loop behavior.
- [ ] No prompt-based provider detection.
- [ ] No hard-coded product model selection outside the canonical catalogue.

Exit gate: all PF product shells work while OpenAI runtime requests remain byte-for-byte
or semantically equivalent to upstream, excluding explicit branding metadata.

### Phase 3: port the canonical model and provider catalogue

- [ ] Define one typed record for provider, wire API, model ID, upstream model ID,
  authentication mode, billing class, input/output/cache price, plan eligibility,
  context limit, output limit, reasoning controls, vision, tool support, web search,
  service tiers, cache policy, and spawn eligibility.
- [ ] Make the TUI picker, `/model`, native spawn tools, `/spawn`, `/orchestrate`,
  accounting, and provider request construction resolve from that record.
- [ ] Reject automatic spawning when billing or required capability metadata is absent.
- [ ] Treat plan routes and metered API routes as distinct billing records even when they
  reach the same named model.
- [ ] Validate provider/model pairs structurally; do not infer a provider from a model
  name.
- [ ] Add a catalogue provenance date and a test that detects stale or internally
  contradictory records.

Exit gate: a running child reports its resolved provider, model, reasoning, billing
class, vision capability, service tier, and selection rationale from the same catalogue
record that built its request.

### Phase 4: port provider adapters one wire at a time

Order:

1. Direct Anthropic and Claude Plan.
2. Generic OpenAI-compatible chat.
3. OpenRouter.
4. Kimi/Moonshot.
5. Z.AI and Ambient.
6. Vercel AI Gateway.
7. Grok and remaining configured providers.

For each adapter:

- [ ] Keep request compatibility logic next to serialization.
- [ ] Normalize provider finish reasons into upstream lifecycle events.
- [ ] Declare cache markers, cache telemetry, reasoning, vision, tool, and output-budget
  capabilities in metadata.
- [ ] Use provider-documented payload limits; keep policy headroom configurable.
- [ ] Add mock wire tests for exact payloads and event translation.
- [ ] Add one small live smoke test and one long-running endurance test.
- [ ] Demonstrate that disabling the adapter returns to unchanged upstream behavior.

Exit gate: each route passes independently before the next route is added.

### Phase 5: rebuild orchestration as an extension of upstream native agents

- [ ] Use upstream agent control, registry, mailbox, waits, history forks, compaction,
  environment inheritance, cancellation, and completion events.
- [ ] Add only the PF model-aware allocation layer:
  task requirements -> catalogue candidates -> operator policy -> resolved runtime.
- [ ] Make selection rationale model-visible and user-visible.
- [ ] Preserve explicit user model/provider requests exactly or report a refusal; never
  silently substitute.
- [ ] Keep plan capacity preferred when capabilities match and operator policy allows it.
- [ ] Make normal `spawn_agent` independent of the opt-in Nazgul/Troll/Orc workflow.
- [ ] Implement `/spawn` and `/orchestrate` as clients of the same native control plane,
  rather than parallel dispatch engines.
- [ ] Remove the three-cycle auto-dispatch stop and 12-report durable-data limit.
- [ ] Preserve bounded UI previews without truncating durable reports.

Exit gate: Kimi, GLM 5.2, an OpenAI 5.6 route, and an Anthropic plan route can be
explicitly spawned, accurately identified, resumed, compacted, cancelled, and compared
within one parent session.

### Phase 6: remove arbitrary policies

- [ ] Delete natural-language shell-budget regex parsing.
- [ ] Replace the global Anthropic 32K output default with catalogue policy.
- [ ] Remove the five-continuation terminal error.
- [ ] Disable the completion classifier globally and re-enable only a qualified
  provider capability if a reproduced route still requires it.
- [ ] Replace fixed web-search usage with configured provider/tool policy.
- [ ] Remove or justify the Claude Code identity injection.
- [ ] Reconcile tool-repeat and malformed-tool handling with upstream.
- [ ] Move payload, cooldown, lease, retry, search, continuation, and spend limits into
  typed policy.
- [ ] Emit a structured event whenever a policy omits context, blocks a tool, pauses a
  turn, changes route, or prevents another model request.
- [ ] Add `FORK_POLICY.md` containing every remaining semantic divergence, its owner,
  evidence, configuration, tests, and removal condition.
- [ ] Add CI that fails when a new PF-only semantic policy is introduced without a
  corresponding ledger entry.

Exit gate: an audit can account for every remaining PF-specific runtime behavior from a
single ledger.

## Verification matrix

### Upstream parity

- [ ] Run the same released OpenAI model, effort, service tier, prompt, repository,
  environment, and fresh home through upstream Codex and converged PF Terminal.
- [ ] Use five paired runs for QueueCraft, TextWright, and QueryForge.
- [ ] Require the same solve count for each task.
- [ ] Require PF median uncached input, cached input, output, model calls, tool calls,
  wall time, and settled cost to remain within 5% of upstream unless a declared PF
  feature was exercised.
- [ ] Investigate any individual PF run that exceeds upstream cost or time by 15%.
- [ ] Preserve raw provider usage and settled billing evidence.

### Provider correctness

- [ ] Direct Anthropic Opus plan/API output-limit endurance.
- [ ] OpenRouter Kimi K3 continuation and tool-boundary endurance.
- [ ] GLM 5.2 through OpenRouter and Vercel.
- [ ] Z.AI and Ambient reasoning controls.
- [ ] OpenAI Sol/Terra/Luna model resolution.
- [ ] Grok route resolution.
- [ ] Cache creation/read behavior for every cache-capable wire.
- [ ] Exact provider/model/billing identity in parent and child telemetry.

### Incident regressions

- [ ] Resume a copied session with the released PF database migrations.
- [ ] Resume after automatic compaction.
- [ ] Run the isometric-game visual task through Opus with repeated screenshots until
  it crosses the former 32K output and Anthropic payload boundaries.
- [ ] Confirm old images are omitted only from a request projection, remain durable,
  and can be reopened.
- [ ] Confirm a sixth productive continuation does not terminate the turn.
- [ ] Confirm a long child report remains retrievable after UI preview truncation.
- [ ] Confirm parent and child survive compaction with task ownership intact.
- [ ] Confirm `Ctrl+T` transcript scrolling and `--no-alt-screen` scrollback.
- [ ] Confirm provider overload, 413, 429, malformed tool calls, and interrupted streams
  recover without duplicate billing loops.

### Build and release

- [ ] Run targeted `just test -p ...` for every changed crate.
- [ ] Run app-server schema and config schema generation when their types change.
- [ ] Run `just fix -p ...` and `just fmt` in the required order.
- [ ] Ask before the complete workspace `just test`, as required by repository
  instructions, then run it before release.
- [ ] Build Linux, macOS, and Windows release artifacts.
- [ ] Exercise clean install, upgrade from 0.1.26, resume, rollback, and uninstall.
- [ ] Run the full benchmark package against the exact release candidate binaries.

## Pull-request sequence

Keep each semantic PR below roughly 500 changed lines where possible. Mechanical source
imports and generated schemas are isolated from behavioral review.

1. **PR A — convergence fixtures and compatibility contracts**
2. **PR B — upstream source baseline**
3. **PR C — PF packaging, home, migrations, and vault**
4. **PR D — canonical catalogue and runtime resolution**
5. **PR E — Anthropic and Claude Plan adapters**
6. **PR F — OpenAI-compatible, OpenRouter, and Kimi adapters**
7. **PR G — Z.AI, Ambient, Vercel, Grok, and remaining adapters**
8. **PR H — native model-aware orchestration**
9. **PR I — arbitrary-policy deletion and fork-policy CI**
10. **PR J — release candidate, parity evidence, endurance evidence, and installers**

No PR may combine an upstream import, a provider behavior change, and a benchmark claim.

## Rollout and rollback

- [ ] Publish a debug binary under an isolated executable name and isolated test home.
- [ ] Run existing 0.1.26 and converged debug sessions side by side.
- [ ] Promote to an RC only after migration, provider, orchestration, and OpenAI parity
  gates pass.
- [ ] Do not overwrite the stable standalone package pointer during RC qualification.
- [ ] Store the exact upstream commit, PF commit, binaries, checksums, configs, and
  benchmark inputs in release evidence.
- [ ] Make rollback restore the 0.1.26 binary without downgrading or rewriting the state
  database.
- [ ] Release only after the RC resumes both old and new sessions and the old binary can
  still open pre-RC data where schema compatibility promises it should.

## Definition of done

- [ ] PF Terminal is based on an identified current upstream Codex commit.
- [ ] OpenAI behavior and economics meet the parity gate.
- [ ] Multi-provider routes pass their wire, cache, reasoning, cost, and endurance gates.
- [ ] Normal native spawning is model-aware without using a parallel orchestration
  runtime.
- [ ] No fixed 32K Anthropic output policy remains.
- [ ] No five-continuation terminal error remains.
- [ ] No natural-language regex silently changes a runtime tool budget.
- [ ] No durable child report is lost to a UI prompt-size limit.
- [ ] Every remaining semantic divergence appears in `FORK_POLICY.md`.
- [ ] Released PF homes and databases resume without collision or migration damage.
- [ ] The release can be rolled back without data loss.
