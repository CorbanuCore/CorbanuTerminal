# PF Terminal 0.1.27 Comprehensive Release Specification

> **Emergency override (2026-08-01):**
> `PFTERMINAL-0.1.27-EMERGENCY-PRODUCT-PRESERVATION-SPEC-20260801.md` governs
> all feature-preservation and product-branding decisions. No released PF Terminal feature is
> authorized for deprecation, and no user-facing Codex product identity is permitted. If this
> document can be read to allow either outcome, the emergency specification controls.

Date: 2026-07-31  
Status: Draft, **NO-GO**  
Target: PF Terminal 0.1.27, the first release after 0.1.26 and the upstream Codex convergence  
Release branch: to be cut from `integrate/upstream-20260730` after source reconciliation  
Rollback baseline: `pfterminal-v0.1.26-pre-convergence` / installed PF Terminal 0.1.26

This specification supersedes the release portions of:

- `docs/current-sprint/upstream-codex-convergence-plan-20260730.md`;
- `docs/current-sprint/pfterminal-post-convergence-release-readiness-spec-20260731.md`.

Those documents remain incident evidence and design history. This document is the single
release checklist.

## 1. Release decision in plain language

PF Terminal 0.1.27 is not ready to publish.

The current branch has a coherent upstream Codex merge at `45a60f03d`, but the product
changes after that merge remain in a large dirty worktree. The frozen path inventory, rather
than a count that changes as release evidence is added, defines the reconciliation scope.
Some deletions intentionally
remove PF-only runtime policies that should never have diverged from Codex. Other deletions
remove released PF product surfaces and must be restored or deliberately replaced before
release.

A local debug binary and passing targeted tests prove that parts of the work function. They
do not prove that a clean commit, packaged binary, installer, upgrade, or resumed user
session works.

The release becomes a GO only when:

1. every intended change is committed in a clean, reviewable source history;
2. every retained PF feature below exists in the converged product;
3. every P0 and P1 behavior passes its automated and live acceptance tests;
4. the exact packaged artifacts, rather than an earlier debug build, pass installation,
   upgrade, resume, rollback, and performance qualification;
5. the release evidence bundle identifies the exact source and binary hashes; and
6. the operator explicitly approves promotion.

### 1.1 Current state at the time of this specification

This table distinguishes code that exists in the working tree from code that has passed a
release gate. “Implemented” does not mean “ready to ship.” Until a behavior is committed,
built into the frozen RC artifact, and tested through that artifact, it remains unqualified.

| Workstream | Current evidence | Release status |
| --- | --- | --- |
| Source recovery | Recovery ref `refs/pfterminal/recovery/pre-0.1.27-convergence-20260731`, `SOURCE.json`, and the frozen path inventory exist. | Preserved; path dispositions remain incomplete. |
| Upstream Codex convergence | Pinned upstream is merged at `45a60f03d`; duplicate PF turn/completion/pane runtimes are being removed. | Implemented in source; complete parity and workspace tests remain. |
| PF product surfaces | Provider setup, plan auth, wallet, vault, Telegram, GPU rental, `/spawn`, `/orchestrate`, and native `/agent` paths exist in the dirty tree. | Present but not release-qualified. |
| Model catalogue | Typed capability, billing, and Chat Completions reasoning-wire records exist; GPT-5.5 is hidden/ineligible; Sol, Terra, Luna, Claude, Kimi, GLM, Grok, plan, API, and local route classes are represented. | The name-check removal passes a focused core test, 57/57 model-manager tests, and 272/272 protocol tests; full core and live route gates remain. |
| Model switching and identity | The TUI waits for a settings acknowledgement before changing displayed/persisted route. Live thread records can expose exact provider, model, effort, and service tier. | Targeted tests pass; resume, compaction, failure, and live exact-route matrices remain. |
| Agent visibility | `/agent` can display exact child route plus catalogue capability, billing, vision, and typed selection provenance. Explicit selection provenance is written into the child rollout and restored on resume. | Focused spawn, persistence, resume, schema, app-server, and TUI tests pass; nested/cancel/end-to-end artifact tests remain. |
| DeepSeek rental | The selectable recipe has been replaced by `DeepSeek-V4-Flash-0731` on 2×H200 with an immutable revision. | Unit coverage exists; mocked end-to-end and approved live rental qualification remain. |
| Installers and packaging | PF-owned Linux/macOS and Windows installer paths are restored; shell installer tests pass 7/7 and package-builder tests pass 18/18. | No clean RC artifacts or platform install matrix yet; PowerShell tests require Windows CI. |
| TUI | Focused route/identity tests pass. The earlier full pre-fix run recorded 3,383 passes, 17 failures, and one timeout. | Full post-fix TUI suite and snapshot review remain mandatory. |
| App-server protocol | Runtime route was added to loaded thread data and generated JSON/TypeScript exports. | Focused read test and compile checks pass; full protocol/app-server suites and no-diff regeneration remain. |
| Live/provider testing | Earlier debug and benchmark runs provide incident evidence. | No mandatory live matrix has run against a frozen 0.1.27 RC artifact. |
| Release artifacts | None. | **NO-GO.** No RC tag, signed package, public prerelease, or stable release exists. |

### 1.2 Blocking priority

P0 means the release cannot be cut. P1 means an internal RC may be built for testing, but
the public prerelease or stable release cannot be promoted.

P0 blockers:

- finish the source disposition and produce a clean, reviewable RC commit;
- eliminate model-name-driven routing/reasoning behavior in favor of typed catalogue
  policy;
- prove exact child route, billing basis, capability evidence, and persisted selection
  provenance through nested spawning, cancellation, and artifact-level resume;
- prove transactional model switching across compaction, failure, and resume;
- prove Anthropic long-session request projection, signed-thinking replay, and output
  behavior do not recreate the reported 413/thinking/output-limit failure loops;
- prove released PF homes resume safely, foreign homes are refused without mutation, and
  debug/stable homes never collide;
- pass all targeted changed-crate suites, generated-file checks, snapshots, formatting,
  and release-mode compilation.

P1 blockers:

- run the live exact-route, multi-model orchestration, and long visual endurance tests on
  the frozen artifact;
- pass the OpenAI parity benchmark with equal run counts and the declared 5% gate;
- pass fresh-install, upgrade, resume, rollback, and checksum enforcement on every
  supported platform;
- complete mocked GPU qualification and, only with explicit approval, the capped real
  rental;
- complete release notes, changelog, README, evidence bundle, secret scan, and public
  prerelease install-back test.

### 1.3 Terms used in this document

- **Route** means the complete inference destination: provider, provider model ID,
  endpoint/wire protocol, authentication/billing mode, reasoning effort, and service tier.
- **Exact route** means PF Terminal either uses that complete route or refuses before any
  inference spend. It never silently substitutes another model or provider.
- **Plan route** means inference covered by an authenticated subscription/plan. An API
  route is metered separately, even when it reaches the same model family.
- **Frozen RC** means one clean Git commit and the artifacts built once from that commit.
  Tests against a debug binary or a later rebuild do not qualify the frozen RC.
- **Upstream parity** means the PF Terminal OpenAI path behaves and performs within the
  declared gate relative to the pinned OpenAI Codex source on the same task and model.

## 2. Release objective

Ship a PF Terminal release that keeps current upstream Codex as the coding-agent runtime
while preserving PF Terminal's multi-provider product:

- Codex owns turn execution, context handling, compaction, tool loops, permissions,
  terminal mechanics, native agent control, and mailbox lifecycle.
- PF Terminal owns provider adapters, the canonical model catalogue, model-aware
  allocation, billing metadata, provider credentials, plans, wallet, GPU rental,
  Telegram, branding, packaging, and the separate PF home.
- OpenAI behavior remains equivalent to the pinned upstream Codex revision unless an
  explicitly enabled PF feature is active.
- Third-party differences exist only at typed provider, model, billing, or product
  boundaries. No model-name regex, prose classifier, or scattered hard-coded constant may
  silently control routing, spend, continuation, or completion.

## 3. Frozen source baselines

| Purpose | Reference |
| --- | --- |
| Released rollback source | tag `pfterminal-v0.1.26-pre-convergence` |
| Released main at convergence start | `d9e2a383a` |
| Released 0.1.26 preparation commit | `5e7952720` |
| Pinned upstream Codex | `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa` |
| Convergence merge commit | `45a60f03d` |
| Active worktree | `integrate/upstream-20260730` plus uncommitted changes |

Before implementation continues, record a new machine-readable inventory containing the
current branch, `HEAD`, status, diff statistics, migration hashes, toolchain, lockfiles,
debug binary hash, and all untracked release documents. Do not reset or discard the dirty
tree.

## 4. What must ship

### 4.1 Upstream Codex runtime

The release must retain the pinned upstream implementations for:

- context projection and compaction;
- Responses continuation and `previous_response_id` lifecycle;
- tool registration, execution, failure recovery, and approvals;
- streaming and retry semantics;
- native agent control, registry, mailbox, wait, cancel, close, and resume;
- TUI input, transcript, scrolling, resize/reflow, and interruption;
- sandbox and permission profiles;
- app-server protocols and generated schemas.

Any PF modification in these areas must appear in `FORK_POLICY.md` with an owner,
reproduced evidence, configuration surface where applicable, tests, and removal condition.

### 4.2 PF product surfaces

The following are release requirements, not optional legacy code:

- `pfterminal` binaries, branding, update channel, and `~/.pfterminal` home isolation;
- `pfterminal-debug` resolving the local debug binary and defaulting to
  `~/.pfterminal-debug` without reading the stable database;
- `/model`, `/providers`, `/vault`, `/wallet`, `/agent`, `/spawn`, and `/orchestrate`;
- OpenAI account auth, Claude Plan auth, PF prepaid plans, API-key providers, and custom
  providers;
- encrypted credential lookup without exposing secrets to model context or logs;
- the local Solana wallet, bounded unlock, balances, backup, restore, and plan purchase;
- Telegram control with the selected sandbox and exact resumed route;
- model-aware native agent spawning and durable agent identity;
- GPU rental selection, authorization, persistence, readiness, and termination;
- Linux, macOS, and Windows release packages and installers.

`/orchestrate` may retain named PF workflows as an opt-in profile. It must use the same
native agent controller, catalogue resolver, mailbox, accounting, and resume path as
ordinary spawning. It must not reintroduce a parallel pane runtime or leak manager
instructions into normal agents.

### 4.3 Supported model/provider behavior

The canonical catalogue must be the only product source for:

- provider ID and exact model ID;
- wire protocol and provider-reported aliases;
- authentication mode and billing class;
- input, output, cache-read, and cache-creation price;
- plan eligibility and plan preference;
- context and output limits;
- reasoning controls and supported effort values;
- vision, tools, image generation, and web-search capability;
- service tier and automatic-spawn eligibility.

The release must support and test these route classes:

- OpenAI Sol, Terra, and Luna as distinct routes;
- direct Anthropic and Claude Plan, including Opus 5 and Fable 5;
- Kimi K3 through its supported direct and OpenRouter paths;
- GLM 5.2 through Z.AI, OpenRouter, and Vercel where configured;
- Grok through its configured route;
- PF plans and local/custom OpenAI-compatible providers.

GPT-5.5 remains ineligible for automatic orchestration. An explicit route is either
honored exactly or refused before inference. No fallback may spend money on another model
without a new explicit user decision.

### 4.4 GPU rental and DeepSeek

The only selectable DeepSeek rental recipe is:

```text
recipe_id: deepseek-flash-0731-2xh200
model: deepseek-ai/DeepSeek-V4-Flash-0731
revision: 9e165c30e2704aec5d9d593cce3eebd58bbef1cb
runtime: vLLM 0.26.0
optimization: DSpark, seven speculative tokens
topology: 2 × NVIDIA H200 with the declared high-bandwidth link
```

The obsolete DeepSeek V4 preview SGLang recipes, the old 4×H200 recipe, and the Huihui
DeepSeek preview derivative must not appear in source as launchable recipes or in the
selectable catalogue. Historical rental records may be displayed as retired so a user can
inspect and terminate them. They must not be relaunched from an obsolete embedded recipe.

The GPU rental TUI and event path deleted by convergence must be ported to the upstream TUI
or replaced with an equivalent first-class flow. The UI must resolve the recommendation
through `RecipeCatalog::recommended_for_family("deepseek")`, never list order or display
text.

No billable provider request occurs until the authorization view shows the exact model,
immutable recipe revision, GPU topology, hourly price, total cap, termination time, and
whether spend enforcement is provider-guaranteed or controller-dependent.

## 5. Product behavior that must be removed

The release must not restore these former PF-wide policies:

- five continuations per turn;
- completion decisions based on short regex-selected transcript tails;
- shell/tool budgets inferred from natural-language regexes;
- automatic pause after three dispatch cycles;
- durable child-report truncation to a fixed count or character limit;
- a global fourth-identical-tool-call stop independent of upstream behavior;
- a fixed eight-use Anthropic web-search cap;
- a global 32,000-token Anthropic output limit;
- a PF-wide one-retry rule for streams lasting more than 60 seconds;
- a `You are Claude Code` product-identity prompt;
- model/provider selection inferred from names;
- UI success before a model switch is committed;
- normal-agent inheritance of the opt-in orchestration personality.

A legitimate provider limit must live in typed provider/model policy, be operator-visible
when it affects a run, and have a contract test based on provider evidence.

## 6. Required repairs and convergence work

### 6.1 Source reconciliation

- [ ] Preserve the dirty tree as a recoverable patch and path manifest.
- [ ] Classify every modified/deleted file as upstream adoption, retained PF product,
  provider adapter, generated artifact, test/evidence, or accidental loss.
- [ ] Audit every deleted released PF module before accepting the deletion.
- [ ] Restore or replace the GPU rental, provider/plan, Telegram, Task Node, orchestration,
  and other released TUI paths that lack an upstream equivalent.
- [ ] Keep intentional removal of duplicate turn, pane, completion, and orchestration
  runtimes.
- [ ] Split semantic changes into reviewable commits; keep generated schemas/snapshots
  with the change that requires them.
- [ ] Produce a clean release-candidate commit and tag it `pfterminal-v0.1.27-rc.1`.

### 6.2 Model identity and switching

- [ ] Represent selected, upstream-requested, acceptable reported aliases, and actual
  reported model IDs as structured route identity.
- [ ] Make model switching one acknowledged provider/model state transition.
- [ ] Keep the old route active if acknowledgement, auth, or compaction fails.
- [ ] Use the newly committed route for pre-turn compaction, remote/local compaction, and
  the first sample after switching.
- [ ] Accept declared Claude Plan aliases without fabricating an OpenAI cyber downgrade.
- [ ] Emit a safety downgrade only from explicit provider metadata, never string mismatch.
- [ ] Persist enough identity evidence to prove the route used after resume.

### 6.3 Native model-aware agents

- [ ] Resolve task requirements against the canonical catalogue, then operator policy,
  then availability.
- [ ] Prefer an authorized capable plan route before metered API spend when no exact model
  was requested.
- [ ] Refuse automatic candidates with unknown billing or missing required capabilities.
- [x] Expose the chosen provider, model, effort, billing class, vision capability, cost
  basis, and selection rationale to the parent and `/agent` UI.
- [ ] Preserve exact child identity through compaction, close/resume, cancellation, and
  nested spawning.
- [ ] Deliver one terminal child result to the direct parent without exposing internal
  mailbox envelopes or triggering an unsolicited paid parent turn.
- [ ] Keep full-history forks on the parent runtime; allow exact eligible overrides only
  for fork modes whose history contract permits it.

### 6.4 Provider reliability

- [ ] Anthropic: preserve stable cache markers, signed thinking blocks, stop reasons,
  tool-only turns, model-owned output budgets, and bounded request-size projection.
- [ ] Anthropic: retain images durably while omitting oldest inline images only from an
  oversized request; show that omission to the model; bound any 413 retry.
- [ ] Kimi: replay `reasoning_content` with tool calls, honor reasoning effort, interpret
  the actual completion boundary, and avoid synthetic continuation on unsupported wires.
- [ ] OpenAI: preserve upstream `previous_response_id`, cache behavior, compaction,
  WebSocket/HTTPS fallback, and retry semantics.
- [ ] OpenRouter: use sticky session identity and avoid transformations that destroy
  prefix caching unless explicitly requested.
- [ ] GLM/Z.AI/Ambient/Vercel: send the catalogue-selected reasoning representation and
  preserve exact upstream pinning where the gateway requires it.
- [ ] All routes: distinguish failure before output from failure after partial/billable
  output so retries cannot duplicate work silently.

### 6.5 Tool-call compatibility

- [ ] Accept native freeform `apply_patch` and supported provider wrappers at the tool
  adapter boundary.
- [ ] Return malformed model payloads as recoverable tool errors; never panic or terminate
  the session runtime for a provider shape mismatch.
- [ ] Allow a corrected call in the same turn and retain upstream tool-loop semantics.
- [ ] Keep hook input canonical and stable across provider representations.

### 6.6 State and operational safety

- [ ] Preserve PF migrations 0040-0045 byte-for-byte and keep later migrations uniquely
  numbered.
- [ ] Refuse `.codex`/foreign-distribution databases without modifying them.
- [ ] Keep debug and stable homes isolated.
- [ ] Resume released root sessions, child sessions, compacted sessions, and open-child
  states from copies of 0.1.24, 0.1.25, and 0.1.26 homes.
- [ ] Make `pfterminal doctor` diagnose the exact database collision and provide a safe,
  non-destructive repair path.
- [ ] Prove rollback to 0.1.26 preserves the pre-upgrade fixture and does not require a
  database downgrade.

### 6.7 Terminal usability

- [ ] Verify keyboard input remains responsive during provider retry and agent activity.
- [ ] `Ctrl+C` interrupts the active turn once without requiring pane destruction.
- [ ] Transcript scrollback works with mouse, keyboard, resized terminals, focused agent
  views, and long wrapped output.
- [ ] Model-switch and provider warnings remain visible without taking control of input.
- [ ] A focused child view always identifies the full agent path, thread ID, model route,
  and separate billing scope.

## 7. Automated test plan

All test commands run from `codex-rs`. Tests must run against the frozen RC commit. Record
the command, start/end time, exit status, and log artifact. Do not conceal flakes with
unbounded reruns.

### 7.1 Required targeted crate suites

Run at minimum:

```sh
just test -p codex-model-provider-info
just test -p codex-model-provider
just test -p codex-models-manager
just test -p codex-api
just test -p codex-core
just test -p codex-app-server-protocol
just test -p codex-app-server
just test -p codex-state
just test -p codex-tui
just test -p codex-gpu-market
just test -p codex-vault
just test -p codex-wallet
just test -p codex-wallet-daemon
just test -p codex-telegram
just test -p codex-cli
```

Also run targeted suites for every other changed crate identified by the final path
manifest. A crate omitted from this list is not exempt if its source changed.

Run the installer and package-builder suites from the repository root:

```sh
python3 -m unittest scripts.install.test_install_sh
python3 -m unittest discover -s scripts/codex_package -p 'test_*.py' -v
```

Run the Windows installer contract suite on both Windows PowerShell 5.1 and PowerShell 7:

```powershell
./scripts/install/install_architecture.tests.ps1
```

The current working-tree evidence is 7/7 passing Linux/macOS installer tests and 18/18
passing package-builder tests. These results must be repeated from the frozen RC; they do
not waive the platform installation matrix.

### 7.2 Required integration coverage

Automated tests must prove:

| Area | Required scenarios |
| --- | --- |
| Model catalogue | Unique provider/model pairs; typed billing; plan/API distinction; capability and vision metadata; GPT-5.5 ineligible; Sol/Terra/Luna distinct; stale/contradictory records rejected. |
| Switching | Same-provider and cross-provider; plan→API and API→plan; below/above compaction threshold; failed auth; failed compaction; resume after switch; old endpoint receives no post-switch request. |
| Identity | Expected Plan aliases accepted; unexpected substitution refused; structured safety downgrade fixture; displayed/configured/requested/reported identities agree. |
| Agents | Exact and automatic spawn; allowlist refusal; full/partial/no-history forks; nested spawn; cancel; close/resume; parent/child mail; no duplicate terminal result; no unsolicited parent inference. |
| Anthropic | Cache marker stability; signed thinking replay; tool-only completion; `max_tokens`; output above the former 32K cap; 30 MB projection; bounded 413 retry; image source preserved. |
| Kimi | Reasoning replay; effort values; tool continuation; final stop; partial stream; no-progress handling without model-name hardcoding. |
| OpenAI | Responses server-state continuation; prompt cache; WebSocket→HTTPS fallback; remote/local compaction; parity with upstream request lifecycle. |
| Chat/gateways | GLM, OpenRouter, Z.AI, Ambient, Vercel reasoning fields; route pinning; provider error finish reason; retry safety. |
| Tools | Native and wrapped patch; JSON string wrapper; malformed then corrected call; hook representation; no host panic. |
| State | Migration checksum fixtures; foreign DB refusal; corruption diagnosis; released-home resume; descendant recovery; concurrent runtime access. |
| GPU | Exactly one selectable DeepSeek recipe; immutable 0731 metadata; DSpark flags; entrypoint propagation; RunPod/Vast request shape; pricing revalidation; authorization; readiness; cleanup. |
| TUI | Provider/model picker, wallet/vault, GPU authorization, `/agent`, focused-child identity, scrolling, resize/reflow, interruption, and warning snapshots. |

Every agent-logic change requires a `codex-core` integration test using the normal test
harness. Every user-visible TUI change requires reviewed `insta` snapshots. Every protocol
change requires regenerated stable and experimental schemas plus compatibility tests.

### 7.3 Static and generated checks

- [x] `just write-config-schema` is idempotent by SHA-256 comparison after a
  second generation.
- [x] App-server stable and experimental exports are regenerated and are
  idempotent by a full schema-directory SHA-256 comparison.
- [x] `just bazel-lock-update` and `just bazel-lock-check` pass after dependency
  convergence.
- [x] `cargo insta pending-snapshots --manifest-path tui/Cargo.toml` reports no
  unreviewed snapshot. (`cargo-insta` does not accept `-p` for this subcommand.)
- [x] `just argument-comment-lint` passes across all 852 Bazel targets. The gate
  also compiled previously skipped test/sample targets and exposed stale
  `runtime_selection`/`runtime_route` constructors, which are now corrected.
- [x] `git diff --check` passes; `.snap` files explicitly preserve
  semantically meaningful terminal-cell padding through `.gitattributes`.
- [x] Preflight secret scan reports zero unreviewed findings across 6,249 text
  files; eight inert test fixtures are allowed only by exact path, kind, and
  SHA-256 fingerprint. Repeat against the frozen RC and artifacts.
- [ ] Version, changelog, release workflow defaults, installer asset names, package
  manifests, and `Cargo.lock` all identify 0.1.27 consistently.
- [ ] `just fix -p <crate>` is completed for each changed crate and `just fmt` leaves the
  tree clean.
- [ ] Release-mode builds complete without hidden failure or killed jobs.

### 7.4 Complete workspace gate

After targeted suites pass and the RC commit is frozen, obtain the explicit approval
required by repository policy and run:

```sh
just test
```

The full suite must exit zero. A known flake requires a reproduced baseline on the pinned
upstream commit, a linked issue, and a written release decision. “It usually flakes” is not
an acceptable waiver.

## 8. Live qualification plan

Live tests use isolated homes, dedicated route credentials where practical, explicit spend
caps, and the exact RC binary. Each run records requested provider/model, upstream model,
reported model, effort, service tier, billing class, elapsed time, token telemetry, settled
cost, exit condition, and transcript ID.

### 8.1 Route smoke matrix

Run one edit-and-test task on every mandatory route:

- Claude Opus 5 Plan;
- Claude Fable 5 Plan;
- direct Anthropic Opus 5;
- OpenAI Sol, Terra, and Luna;
- Kimi K3 direct and through OpenRouter;
- GLM 5.2 through Z.AI, OpenRouter, and Vercel;
- Grok through its configured route.

Pass conditions: exact route or explicit refusal, one successful edit, one successful tool
execution, no false reroute warning, no duplicate paid turn, and clean terminal completion.

### 8.2 Switching and compaction incident test

Using a copied long 0.1.26 session:

1. start on Fable Plan;
2. select Sol;
3. force compaction above the threshold;
4. complete an edit-and-test turn;
5. switch back to Opus Plan;
6. resume the session in a new process.

Pass conditions: the UI changes only after acknowledgement; only the selected endpoint
receives compaction and sampling; the previous provider receives no post-switch request;
resume restores the last committed route; no unsupported-Fable-on-ChatGPT error appears.

### 8.3 Multi-model orchestration test

From one root, explicitly spawn distinct children on Opus Plan, Kimi K3, GLM 5.2, and
OpenAI Luna to review the same repository at the same commit. Then compare their reports,
send one follow-up to each, close the process, and resume it.

Pass conditions:

- all four children report the exact requested provider/model and rationale;
- no child silently inherits the root route when an eligible override was requested;
- no GPT-5.5 route is selected;
- each terminal report arrives once at its direct parent;
- internal mailbox envelopes are not rendered as ordinary assistant prose;
- resume restores open descendants and keeps explicitly closed descendants closed;
- cost is attributed to the correct route and child.

### 8.4 Long visual Anthropic endurance test

Run the isometric-game visual repair prompt through Opus using repeated screenshots, video
or image inspection, edits, tests, and compaction long enough to cross the old failure
boundaries.

Pass conditions:

- no repeating “engine rewrite” output loop;
- no 413 poison loop;
- no modified signed-thinking-block rejection;
- no artificial 32K output stop;
- no hard five-continuation stop;
- oversized images are bounded in request projection while their source files remain
  reopenable;
- `Ctrl+C`, user steering, transcript scrollback, and later resume remain functional.

### 8.5 GPU rental qualification

First run the full flow against mocked RunPod and Vast providers. Then, with explicit
operator approval and a capped budget, launch one real 2×H200 0731 rental.

Pass conditions:

- selection resolves only `deepseek-flash-0731-2xh200`;
- immutable image/model revisions and DSpark flags match the recipe;
- topology, driver, auth, model identity, readiness, and inference probes pass;
- one tool-using inference request succeeds;
- termination is confirmed through provider inventory;
- final settled cost does not exceed the authorized cap;
- no obsolete DeepSeek recipe is offered or relaunched.

## 9. OpenAI upstream parity gate

PF Terminal is a Codex fork, so the release must prove the PF layer did not degrade the
OpenAI path.

Run five paired PF/upstream waves for each of QueueCraft, TextWright, and QueryForge using:

- the same OpenAI model, reasoning effort, and service tier;
- the same prompt, repository state, host, and verifier;
- fresh isolated homes;
- alternating lane order;
- equal run counts, including failures and timeouts in denominators.

Required gate:

- equal solve count for each task;
- PF median uncached input, cached input, output, model calls, tool calls, wall time, and
  settled cost within 5% of the pinned upstream Codex result unless a declared PF feature
  was exercised;
- every individual PF run more than 15% slower or more expensive receives a trace-level
  diagnosis;
- no intermittent cache-collapse pattern;
- the measured PF binary hash equals the packaged release artifact hash.

The Hermes/Claude Code launch benchmarks may be rerun for publication, but favorable
marketing comparisons are not a substitute for this upstream parity gate.

## 10. Package and installation matrix

Build once from the frozen RC commit using the release workflow. Record SHA-256 hashes
before installation tests.

| Platform | Required artifact/test |
| --- | --- |
| Linux x86_64 | Archive, standalone installer, fresh install, upgrade from 0.1.26, resume, rollback, uninstall. |
| Linux aarch64 | Archive and installer smoke on native or declared CI hardware. |
| macOS Apple Silicon | Archive, DMG, mount/install/launch, upgrade, quarantine/signing behavior. |
| macOS Intel | Archive and DMG build with bounded retry; install/launch proof. |
| Windows x64 | ZIP/installer under PowerShell 5.1 and 7; locked-executable upgrade. |
| Windows ARM64 | Architecture selection, extraction, launch, and upgrade. |

Every platform must prove:

- `pfterminal --version` reports 0.1.27;
- the binary uses `~/.pfterminal`, never `~/.codex`;
- stable and debug commands resolve the intended binary and home;
- `/providers`, `/vault`, `/wallet`, `/model`, and `/agent` open;
- a fresh OpenAI turn and one third-party route complete;
- upgrade preserves sessions and credentials;
- rollback instructions work on a copied fixture;
- installer checksums/signatures reject altered artifacts.

## 11. Release evidence bundle

Store one immutable directory under `qa/release/0.1.27/<rc-commit>/` containing:

- `SOURCE.json`: PF commit, upstream commit, clean status, toolchain, lockfile hashes;
- `DISPOSITION.csv`: every changed/deleted path and its release disposition;
- `ARTIFACTS.json`: names, platforms, sizes, SHA-256 hashes, signatures;
- `TESTS.md`: commands, times, exit codes, failures, reruns, and approved waivers;
- `ROUTES.json`: selected/requested/upstream/reported model identity for live runs;
- `ORCHESTRATION.json`: root/child IDs, exact routes, rationale, billing, outcomes;
- `STATE.md`: fixture hashes, migration checksums, resume, upgrade, rollback results;
- `GPU.md`: mocked and live recipe/topology/readiness/termination evidence;
- `PERFORMANCE.md`: raw-run links and parity calculations;
- `INSTALL.md`: per-platform installation evidence;
- `KNOWN_ISSUES.md`: only non-blocking issues, each with owner and impact;
- `SECRET_SCAN.json`: zero-hit result.

Never include API keys, OAuth tokens, vault contents, wallet recovery material, raw
environment dumps, or unsanitized user databases.

## 12. Release sequence

1. [x] Preserve and inventory the dirty convergence tree.
2. [ ] Complete the path-by-path deletion and feature-retention audit.
3. [ ] Land source reconciliation in reviewable commits.
4. [ ] Repair all remaining P0/P1 behavior and add regression tests.
5. [ ] Restore all mandatory PF product surfaces on the upstream TUI/runtime.
6. [ ] Regenerate schemas, snapshots, and dependency locks.
7. [ ] Run targeted tests, lint/fix, formatting, and static checks.
8. [ ] Freeze and tag RC1 from a clean tree; build artifacts once and hash them.
9. [ ] Run live route, switching, orchestration, endurance, state, GPU, and PTY tests on
   the exact artifacts.
10. [ ] With required approval, run the complete workspace test suite.
11. [ ] Run OpenAI parity and platform install/upgrade/rollback matrices.
12. [ ] Review the evidence bundle and secret scan.
13. [ ] Update README, changelog, version metadata, installer URLs, and release notes.
14. [ ] Obtain explicit operator approval.
15. [ ] Publish 0.1.27 as a prerelease, install it once from the public release, then
    promote the stable/latest pointer.

## 13. Automatic no-go conditions

Do not release if any of these is true:

- the source tree is dirty or the artifact cannot be reproduced from its recorded commit;
- a released PF feature disappeared without a written retirement and migration plan;
- the TUI offers any obsolete DeepSeek recipe;
- a selected or spawned model silently changes provider/model;
- UI, telemetry, and provider evidence disagree about the running route;
- a child route or billing class is unknown;
- a child completion triggers duplicate or unsolicited inference;
- model switching reports success before the new route is committed;
- a valid provider tool call can panic or strand the runtime;
- a long image session enters a repeatable 413/thinking/output-limit failure loop;
- a released database fixture is modified in place, rejected incorrectly, or becomes
  unreadable;
- PF fails any mandatory test or performs outside the OpenAI parity gate;
- tests ran against a binary other than the packaged artifact;
- any evidence or artifact contains a secret;
- any P0 is waived without a written replacement decision from the operator.

## 14. Definition of done

- [ ] One clean 0.1.27 release commit is the tip of the reviewed convergence series and
  contains the retained PF product.
- [ ] Every path in the frozen preflight inventory has an explicit disposition.
- [ ] All required PF commands and provider surfaces exist in the packaged TUI.
- [ ] The canonical catalogue drives picker, switching, spawning, provider requests,
  capability matching, and accounting.
- [ ] Exact routing, compaction, orchestration, mailbox, and resume behavior pass.
- [ ] Anthropic, Kimi, OpenAI, GLM, gateway, and tool compatibility gates pass.
- [ ] DeepSeek-V4-Flash-0731 is the sole selectable DeepSeek rental recipe and passes live
  capped qualification.
- [ ] Released-state migration, debug isolation, install, upgrade, and rollback pass.
- [ ] Targeted tests and the approved complete workspace test pass.
- [ ] OpenAI upstream parity passes on equal run counts.
- [ ] All platform artifacts install and match recorded hashes.
- [ ] The evidence bundle is complete and secret-free.
- [ ] The operator explicitly approves promotion to stable.
