# PF Terminal Post-Convergence Release Readiness Specification

> Superseded for release execution by
> `PFTERMINAL-0.1.27-COMPREHENSIVE-RELEASE-SPEC-20260731.md`. This file remains
> incident evidence and design history.

Date: 2026-07-31  
Status: Draft release blocker specification  
Target: the first release after the installed `0.1.26` rollback baseline  
Candidate branch: `integrate/upstream-20260730`  
Candidate merge head: `45a60f03d`  
Related plans:

- `docs/current-sprint/upstream-codex-convergence-plan-20260730.md`
- `docs/current-sprint/upstream-convergence-release-baseline-20260730.md`
- `FORK_POLICY.md`

## 1. Executive decision

**NO-GO for release.**

The converged debug runtime can complete real provider-backed coding work, but the
source state and qualification evidence do not yet meet a release standard.

The 2026-07-31 isometric-game qualification proved that:

- Claude Opus 5 could inspect the repository, execute browser tooling, apply source
  patches, recover from malformed patch attempts, and run verification commands.
- A provider-wrapped `apply_patch` call could previously terminate the debug turn with
  an incompatible-payload fatal error. The working tree contains a repair, but that
  repair is not yet part of a clean, reproducible release commit.
- PF Terminal falsely reported an OpenAI cyber-security downgrade to GPT-5.2 when the
  expected Claude Plan alias `claude-opus-5-plan` was reported by Anthropic as its
  upstream model `claude-opus-5`. Opus 5 continued running; the warning was fabricated
  locally by a provider-agnostic mismatch handler.
- The candidate worktree contains 537 modified tracked files and one untracked file.
  A binary built from this state cannot be reproduced from the candidate commit.
- Targeted tests and live smokes passed, but the complete workspace, provider,
  orchestration, resume, installer, and platform release matrices have not run against
  one frozen candidate.

PF Terminal may be promoted only after every P0 in this specification is repaired and
the exact packaged binaries pass all mandatory release gates.

## 2. Objective

Produce a clean, auditable PF Terminal release candidate that:

1. reports the actual provider and model without false aliases, invented downgrade
   explanations, or silent substitution;
2. preserves upstream Codex tool-loop behavior while accepting valid provider wire
   representations of advertised tools;
3. switches, compacts, resumes, and spawns on the exact selected provider/model pair;
4. preserves released PF homes, databases, vaults, wallets, and session history;
5. passes complete automated and live qualification from one immutable source commit;
6. installs, upgrades, rolls back, and identifies itself reproducibly on every shipped
   platform;
7. preserves the GPU-rental workflow and makes the qualified
   `deepseek-ai/DeepSeek-V4-Flash-0731` deployment the recommended DeepSeek rental path.

## 3. Non-goals

- Do not add prompt text that tells models to ignore false warnings.
- Do not special-case the isometric-game prompt, its repository path, or literal tool
  calls from the incident.
- Do not infer provider identity from display names or model-name regexes.
- Do not weaken exact-route requirements by silently substituting a supposedly similar
  model.
- Do not treat generated screenshots from the qualification game as PF Terminal product
  changes.
- Do not publish benchmark claims from a dirty or locally patched binary.

## 4. Frozen incident evidence

### 4.1 Candidate and build state

Captured on 2026-07-31:

```text
branch: integrate/upstream-20260730
head: 45a60f03d
tracked changes: 537
untracked files: 1
diff: 7,404 insertions, 64,618 deletions
debug version: pfterminal 0.1.26
debug SHA-256: c3a37ce5c40d974324445383330091c5787be1664b52a0953c41a1b7bf69e26a
```

This SHA identifies the qualification binary only. It is not a release artifact because
its source tree is not clean.

### 4.2 Initial failing live session

Thread:

```text
019fb82d-04e1-7933-a29e-54841f9e3679
```

Rollout:

```text
/home/pfrpc/.pfterminal-debug/sessions/2026/07/31/
rollout-2026-07-31T12-36-25-019fb82d-04e1-7933-a29e-54841f9e3679.jsonl
```

The model emitted a normal function call named `apply_patch` with arguments shaped as:

```json
{"input":"*** Begin Patch\n..."}
```

`ToolRouter::build_tool_call()` represented this as `ToolPayload::Function`, while
`ApplyPatchHandler::matches_kind()` accepted only `ToolPayload::Custom`. The registry
returned `FunctionCallError::Fatal`; the debug drain path then panicked through
`error_or_panic()` and left the turn unusable.

Relevant boundaries:

- `codex-rs/core/src/tools/router.rs`
- `codex-rs/core/src/tools/registry.rs`
- `codex-rs/core/src/tools/handlers/apply_patch.rs`
- `codex-rs/core/src/session/turn.rs`
- `codex-rs/core/src/util.rs`

### 4.3 Post-repair live session

Thread:

```text
019fb83f-0e16-7440-8ea0-e61662b7040c
```

Rollout:

```text
/home/pfrpc/.pfterminal-debug/sessions/2026/07/31/
rollout-2026-07-31T12-56-07-019fb83f-0e16-7440-8ea0-e61662b7040c.jsonl
```

Isolated game worktree:

```text
/tmp/pfterminal-isometric-postfix-20260731
```

Observed behavior:

- malformed patch attempts returned model-visible parse errors;
- the same turn corrected them and continued;
- multiple patches were applied successfully;
- `runtime/engine.js` and `test/math.test.mjs` were changed;
- `npm run test:math` exited 0;
- `npm run gate:p3` exited 0 and completed both browser scenarios;
- the session remained responsive until the operator-owned 20-minute process timeout.

The game edit exposed a viewport-dependent false-green in that repository's P9
occlusion assertion. That is evidence that PF Terminal performed meaningful work, but
it is not a PF Terminal release gate or a reason to merge the disposable game diff.

### 4.4 False model downgrade warning

The post-repair rollout recorded:

```text
model rerouted: claude-opus-5-plan -> claude-opus-5
Your account was flagged ... routed to gpt-5.2 as a fallback
```

The second statement was false.

The session metadata recorded `model_provider = "claude-plan"`. The Anthropic request
builder intentionally maps the plan-facing alias to the upstream Anthropic model:

```text
claude-opus-5-plan -> claude-opus-5
```

This mapping is declared by:

- `codex-rs/model-provider-info/src/lib.rs`
  (`CLAUDE_PLAN_MODEL`, `CLAUDE_PLAN_UPSTREAM_MODEL`);
- `codex-rs/core/src/client.rs` (`anthropic_upstream_model()`).

Anthropic then reported `claude-opus-5`, which was the expected route. However,
`Session::maybe_warn_on_server_model_mismatch()` in
`codex-rs/core/src/session/mod.rs` compares the local alias directly with the upstream
model. Every inequality is currently classified as
`ModelRerouteReason::HighRiskCyberActivity`, and the displayed warning hard-codes
GPT-5.2 and GPT-5.3 Codex. The method does not first verify the provider, its expected
upstream alias, or an explicit safety reroute signal.

This is a model-identity integrity failure even though no actual substitution occurred.

### 4.5 DeepSeek GPU-rental convergence state

The pre-convergence GPU catalogue contains qualified 2×H200 and 4×H200 recipes for the
superseded `deepseek-ai/DeepSeek-V4-Flash` preview weights. DeepSeek released the
official replacement on 2026-07-31. The new recommended manifest must pin:

```text
recipe: deepseek-flash-0731-2xh200
model: deepseek-ai/DeepSeek-V4-Flash-0731
model revision: 9e165c30e2704aec5d9d593cce3eebd58bbef1cb
runtime: vLLM 0.26.0
wire API: OpenAI-compatible chat completions
```

DeepSeek's official model card states that `DeepSeek-V4-Flash-0731` supersedes the
preview, substantially improves agentic capability, and includes the DSpark speculative
decoding module. Its documented vLLM command uses the DSpark method with seven draft
tokens. The official vLLM recipe makes 0731 the default variant and requires vLLM
0.26.0 for the complete DSpark path. A rental upgrade must therefore change the pinned
model revision, runtime image, entrypoint, serving flags, download accounting, and
memory envelope together rather than replacing one model string.

The convergence worktree currently deletes:

```text
codex-rs/tui/src/chatwidget/gpu_menu.rs
codex-rs/tui/src/chatwidget/gpu_menu_tests.rs
```

Consequently, retaining the backend catalogue alone does not preserve the released GPU
rental product. The candidate must restore or deliberately port the rental UI and prove
that its recommended DeepSeek action resolves to the qualified V4 Flash manifest before
release.

## 5. Severity and release blockers

| ID | Severity | Defect | Release impact |
| --- | --- | --- | --- |
| RR-P0-01 | P0 | Expected provider aliases can be reported as unrelated OpenAI safety downgrades. | Users cannot trust which model ran. |
| RR-P0-02 | P0 | Valid provider-wrapped tool calls can be classified as fatal payload mismatches. | A productive coding turn can freeze on its first edit. |
| RR-P0-03 | P0 | The candidate binary is built from 537 uncommitted tracked changes. | Source, tests, binary, and release evidence are not reproducible. |
| RR-P0-04 | P0 | Exact cross-provider switching, compaction, and first post-switch sampling lack final live proof. | The UI may claim a route that did not receive the turn. |
| RR-P0-05 | P0 | Exact child provider/model identity lacks final live multi-provider spawn proof. | Orchestration can spend on or evaluate the wrong model. |
| RR-P0-06 | P0 | Released-home migration and resume have not been qualified against the frozen RC artifact. | Upgrade could strand or mutate user sessions. |
| RR-P0-07 | P0 | The convergence worktree deletes the GPU-rental TUI path even though the V4 Flash backend recipe remains. | Users cannot reliably launch the advertised DeepSeek rental from the product. |
| RR-P1-01 | P1 | Complete workspace and platform test matrices are absent for the frozen RC. | Integration regressions can escape targeted tests. |
| RR-P1-02 | P1 | Large-image compaction and long-context endurance lack final live qualification. | Long visual sessions can regress to 413 or thinking-block failures. |
| RR-P1-03 | P1 | Install, upgrade, rollback, and update-channel behavior are unverified for the new artifact. | A sound binary may still fail operationally. |
| RR-P1-04 | P1 | OpenAI parity benchmarks have not been rerun on the exact RC. | The release could regress cost or latency relative to Codex. |

## 6. Required architecture

### 6.1 Canonical route identity

Model identity must be resolved from one structured route record, not from string
comparison in the session loop.

The effective route must contain at least:

```text
selected_provider_id
selected_model_id
upstream_request_model_id
acceptable_server_model_ids
billing_route
wire_api
```

The canonical model/provider catalogue must own this data. Provider request builders,
model switching, spawn selection, telemetry, and server-model verification must consume
the same record.

For a server-reported model, the verifier must produce one of these outcomes:

| Outcome | Meaning | Required behavior |
| --- | --- | --- |
| `ExactMatch` | Server ID equals selected or upstream ID. | Continue without warning. |
| `ExpectedAlias` | Server ID is declared equivalent by the selected route. | Continue; record structured debug telemetry only. |
| `ExplicitProviderReroute` | Provider supplied an authenticated reroute reason. | Emit the actual from/to IDs and provider reason. Apply exact-route policy. |
| `UnexpectedSubstitution` | Server ID is outside the route's declared identity set. | Stop before accepting further model output and report the discrepancy. Never invent a cause. |
| `Unreported` | The wire does not return a server model. | Continue only if that route explicitly permits unreported identity; mark verification unavailable. |

Requirements:

- [ ] RR-IDENT-01: Move expected upstream/server IDs into typed catalogue/provider
  metadata accessible before the request is sent.
- [ ] RR-IDENT-02: Make `maybe_warn_on_server_model_mismatch()` provider-aware or
  replace it with a route-verification component outside generic session presentation.
- [ ] RR-IDENT-03: Treat Claude Opus Plan, Claude Fable Plan, and legacy plan aliases as
  expected aliases for their declared upstream Anthropic models.
- [ ] RR-IDENT-04: Never emit `HighRiskCyberActivity` from a generic string mismatch.
- [ ] RR-IDENT-05: For OpenAI routes, emit cyber downgrade messaging only when the
  route's documented server contract or explicit metadata supports that conclusion.
- [ ] RR-IDENT-06: Derive warning text from actual selected and reported IDs. Remove
  hard-coded GPT-5.2/GPT-5.3 model claims from the generic path.
- [ ] RR-IDENT-07: Abort on an unexpected substitution when the operator or spawn call
  requested an exact route.
- [ ] RR-IDENT-08: Persist requested, upstream, and reported model identity once per
  turn in machine-readable telemetry and rollout evidence.
- [ ] RR-IDENT-09: Update protocol/app-server schemas if a new mismatch reason or route
  verification event is introduced.

### 6.2 Cross-provider tool-payload normalization

Advertised tool semantics must survive provider wire differences.

Requirements:

- [ ] RR-TOOL-01: `apply_patch` must accept its native freeform payload and provider
  function wrappers containing either a JSON string or `{"input": "..."}`.
- [ ] RR-TOOL-02: Normalize only at the tool adapter/handler boundary. Do not prompt the
  model to reproduce a provider-specific encoding.
- [ ] RR-TOOL-03: A syntactically valid patch received in any supported representation
  must execute through the same parser, safety, approval, hook, and diff-tracking path.
- [ ] RR-TOOL-04: Unsupported or malformed payloads must return
  `RespondToModel`/failed-tool output and permit correction in the same turn.
- [ ] RR-TOOL-05: A model-emitted payload mismatch must never call
  `FunctionCallError::Fatal` merely because the provider encoded the call differently.
- [ ] RR-TOOL-06: Preserve fatal errors for true host invariants such as task-join or
  registry corruption; do not globally hide internal failures.
- [ ] RR-TOOL-07: Hooks must receive one stable command-shaped representation and must
  be able to rewrite both native and wrapped calls without changing response pairing.

### 6.3 Exact model switching and compaction

- [ ] RR-SWITCH-01: A switch is committed only after app-server acknowledges the exact
  provider/model pair.
- [ ] RR-SWITCH-02: The TUI must not update its displayed route before acknowledgement.
- [ ] RR-SWITCH-03: Pre-turn, remote, and local compaction after a switch must use the
  newly committed route unless an explicit, tested policy says otherwise.
- [ ] RR-SWITCH-04: A failed switch leaves the previous route visibly active and emits
  one actionable error.
- [ ] RR-SWITCH-05: Resume restores the committed provider/model pair and does not replay
  an obsolete compaction route.
- [ ] RR-SWITCH-06: Rollout telemetry must prove which provider received compaction and
  which provider received the first post-switch sample.

### 6.4 Exact model-aware spawning

- [ ] RR-SPAWN-01: Native V2 spawning must resolve one eligible catalogue record before
  creating a child.
- [ ] RR-SPAWN-02: An explicit provider/model request is honored exactly or refused;
  there is no substitution.
- [ ] RR-SPAWN-03: Child configuration, request telemetry, list-agents output, and final
  result must agree on provider, model, effort, billing, vision, and capability.
- [ ] RR-SPAWN-04: Automatic selection prefers authorized plan capacity when capability
  matches, then compares metered candidates using catalogue economics.
- [ ] RR-SPAWN-05: Models with unknown billing or missing required capability metadata
  are ineligible for automatic selection.
- [ ] RR-SPAWN-06: GPT-5.5 remains ineligible; OpenAI Sol, Terra, and Luna are distinct
  catalogue routes and must never be collapsed into one generic frontier label.
- [ ] RR-SPAWN-07: Normal native spawning must not inherit the opt-in
  Nazgul/Troll/Orc managerial personality.
- [ ] RR-SPAWN-08: Parent and child survive compaction, resume, cancellation, and nested
  completion without duplicate mailbox output or unrequested paid turns.

### 6.5 Clean and reproducible source state

- [ ] RR-SOURCE-01: Preserve the current dirty tree before reconciliation using a named
  recovery branch or tag, a binary diff bundle, an untracked-file manifest, and hashes.
- [ ] RR-SOURCE-02: Do not reset, discard, or overwrite the current tree while producing
  the release candidate.
- [ ] RR-SOURCE-03: Classify all 537 tracked changes as upstream import, retained PF
  product behavior, provider adapter, generated artifact, test/evidence, or accidental
  loss.
- [ ] RR-SOURCE-04: Audit the 64,618 deleted lines against the behavior disposition
  ledger and `FORK_POLICY.md`.
- [ ] RR-SOURCE-05: Commit semantic changes in reviewable units. Generated schemas and
  mechanical upstream imports must not hide runtime policy changes.
- [ ] RR-SOURCE-06: The frozen RC tree must be clean and its commit must recreate the
  tested binary hash.
- [ ] RR-SOURCE-07: Record the PF commit, upstream Codex commit, Rust toolchain, lockfile,
  generated schemas, build command, and artifact checksums.

### 6.6 Released-state compatibility

- [ ] RR-STATE-01: Preserve released PF migrations 0040-0045 byte-for-byte.
- [ ] RR-STATE-02: Run migration and resume tests only on copies of 0.1.24, 0.1.25, and
  0.1.26 homes.
- [ ] RR-STATE-03: Include the foreign-version-45 Codex/PF collision fixture.
- [ ] RR-STATE-04: Debug qualification must remain isolated under
  `~/.pfterminal-debug`; it must never open or migrate the stable database.
- [ ] RR-STATE-05: Resume old root and child sessions, including compacted and open-child
  states.
- [ ] RR-STATE-06: Prove rollback to the installed 0.1.26 package does not require a
  database downgrade or lose pre-RC data.

### 6.7 GPU-rental recipe selection and preservation

- [ ] RR-GPU-01: Restore or port the released GPU-rental TUI, event dispatch, snapshots,
  and tests onto the converged upstream TUI rather than silently deleting the feature.
- [ ] RR-GPU-02: Define the recommended DeepSeek recipe through one typed catalogue
  selection API. Do not rely on vector order, display-name matching, or a TUI literal.
- [ ] RR-GPU-03: The recommended DeepSeek selection must resolve to the qualified
  `deepseek-flash-0731-2xh200` manifest for
  `deepseek-ai/DeepSeek-V4-Flash-0731`.
- [ ] RR-GPU-04: Keep model identity, immutable weight revision, runtime image,
  serving optimization, hardware requirements, and stability status as separate
  catalogue fields.
- [ ] RR-GPU-05: The 0731 recipe must use its DSpark-aware pinned vLLM runtime/image,
  command, memory envelope, readiness proof, and live qualification. The superseded
  SGLang/EAGLE preview recipes and Huihui preview derivative must not appear in the
  selectable catalogue.
- [ ] RR-GPU-06: Existing rentals retain their recorded recipe ID and revision across
  upgrade and resume; changing the recommendation affects only new rental selection.
- [ ] RR-GPU-07: The authorization screen must show the exact model, GPU topology,
  hourly and total caps, immutable recipe revision, and whether enforcement is provider
  guaranteed or local-controller dependent before any billable request is made.

## 7. Implementation sequence

### Phase A — preserve and inventory

- [ ] Capture current branch, commit, status, diff statistics, binary hash, database
  migration hashes, and debug-home session manifest.
- [ ] Create non-destructive recovery artifacts for the dirty convergence state.
- [ ] Produce a path-level disposition report for all modified and deleted files.
- [ ] Identify every PF feature removed by convergence and link it to an intentional
  disposition or restoration task.

Exit gate: no current work can be lost, and every change has an owner/category.

### Phase B — repair route identity

- [ ] Add catalogue/provider metadata for selected, upstream, and acceptable server IDs.
- [ ] Replace provider-agnostic mismatch inference with structured route verification.
- [ ] Remove the hard-coded generic OpenAI cyber warning.
- [ ] Add route verification telemetry and protocol/schema changes as required.
- [ ] Add unit and integration tests before another paid live test.

Exit gate: Claude Plan aliases produce no reroute warning; a simulated genuine OpenAI
downgrade reports its actual route and reason; an unexpected third-party substitution
fails closed.

### Phase C — finish tool compatibility

- [ ] Retain the working-tree `apply_patch` normalization repair.
- [ ] Add end-to-end integration coverage where the provider emits the observed function
  wrapper and the patch is actually applied.
- [ ] Add adjacent cases: JSON string wrapper, malformed wrapper followed by correction,
  native freeform call, and a different incompatible tool payload.
- [ ] Confirm no debug panic and no orphaned in-flight tool future.

Exit gate: one turn can fail a malformed patch, correct it, edit a file, run a test, and
finish normally on Anthropic and OpenAI-style tool wires.

### Phase C2 — restore GPU rental and bind its recommendation to the catalogue

- [ ] Port the GPU-rental menu and event path onto the converged TUI.
- [ ] Add a typed recommended-recipe lookup for model family `deepseek`.
- [ ] Bind the UI's recommended DeepSeek action to that lookup.
- [ ] Add catalogue, TUI, persistence, authorization, and controller regression tests.
- [ ] Run a non-billable mocked-provider flow before any live rental qualification.

Exit gate: the product exposes one recommended DeepSeek rental, and the resulting
authorization request records `deepseek-flash-0731-2xh200` with its immutable V4 Flash
revision without mutating existing rentals.

### Phase D — produce a frozen RC source commit

- [ ] Reconcile and commit the upstream convergence in logical stages.
- [ ] Regenerate and review required config/app-server schemas.
- [ ] Run `git diff --check`, repository lints, `just fix` in changed crates, then
  `just fmt`.
- [ ] Require a clean `git status` before building the RC.
- [ ] Build release artifacts once and record hashes before live qualification.

Exit gate: every subsequent test names one immutable commit and artifact hash.

### Phase E — automated qualification

- [ ] Run targeted tests for every changed crate.
- [ ] Run catalogue, route verification, provider wire, model switching, compaction,
  MultiAgentV2, state, CLI, app-server, TUI, and installer test slices.
- [ ] Review all changed TUI snapshots and generated schemas.
- [ ] Obtain operator approval required by repository policy, then run complete
  workspace `just test` against the frozen commit.
- [ ] Run release-mode builds with warnings and failures captured in the evidence bundle.

Exit gate: zero failing mandatory tests, zero unreviewed snapshots, zero schema drift.

### Phase F — live provider and workflow qualification

- [ ] Run the matrix in section 8 using isolated homes and exact RC binaries.
- [ ] Capture requested, upstream, and reported model IDs for every run.
- [ ] Keep provider billing/usage evidence separated by route and API key where needed.
- [ ] Treat any substitution, unidentified child, duplicate paid turn, false warning,
  unrecoverable tool error, or database collision as a release-blocking failure.

Exit gate: every mandatory row passes without source edits or manual workspace repair.

### Phase G — package, install, and promote

- [ ] Build Linux, macOS, and Windows artifacts from the frozen commit.
- [ ] Verify signatures/checksums and installer digest enforcement.
- [ ] Test fresh install, upgrade from 0.1.26, resume, rollback, and uninstall.
- [ ] Install the candidate in a disposable prefix and verify `--version`, debug/stable
  home isolation, provider login, `/wallet`, `/providers`, `/model`, and `/agent`.
- [ ] Publish release notes containing the upstream base, fixed incidents, known limits,
  migration boundary, and rollback instructions.
- [ ] Promote the stable pointer only after evidence review and explicit operator go.

## 8. Mandatory test matrix

| Area | Route/scenario | Required proof | Pass condition |
| --- | --- | --- | --- |
| Identity | Claude Opus 5 Plan | `claude-opus-5-plan` request; server reports `claude-opus-5` | Opus continues; no warning or reroute event. |
| Identity | Claude Fable 5 Plan | Plan alias and upstream Fable ID | Expected alias accepted; actual IDs persisted. |
| Identity | OpenAI safety fixture | Explicit/contractual downgrade metadata | Actual from/to and reason shown; no hard-coded unrelated model. |
| Identity | Unexpected third-party model | Mock provider reports an undeclared model | Turn fails closed before accepting output. |
| Tools | Anthropic function-wrapped `apply_patch` | Observed `{"input": patch}` payload | Patch applies; turn continues. |
| Tools | Native freeform `apply_patch` | Upstream custom-tool payload | Patch applies through the same runtime. |
| Tools | Malformed then corrected patch | Two calls in one turn | First returns tool failure; second applies; no panic. |
| Switch | Fable Plan to Sol | History below and above compaction threshold | Only Sol route receives compaction/sample after acknowledgement. |
| Switch | Sol to Opus Plan | Cross-provider reverse switch | Display, config, telemetry, and requests agree. |
| Switch | API to plan and plan to API | Resume after each switch | Resumed route equals last committed route. |
| Spawn | Opus Plan child | Explicit provider/model/effort | Child reports and uses exact plan route. |
| Spawn | Kimi K3 child | `kimi-code/k3` or declared OpenRouter route | Exact Kimi route; billing and vision metadata correct. |
| Spawn | GLM 5.2 child | OpenRouter and Vercel variants | Provider-specific route retained; no name inference. |
| Spawn | OpenAI Luna child | Explicit Luna request | Luna remains distinct from Sol/Terra; no GPT-5.5. |
| Spawn | Disallowed provider | Allowlist refusal | Refused without fallback or spend. |
| Orchestration | Parent plus four distinct children | Code-review task with captured identities | All child routes are visible, distinct, resumable, and billed once. |
| Long context | Opus visual game session | Repeated screenshots and edits | No 413 loop, thinking-block corruption, or output-cap dead end. |
| Compaction | Large-image projection | Oversized screenshot plus continued turn | Request projection bounded; source image remains reopenable. |
| Resume | Released 0.1.24/0.1.25/0.1.26 homes | Copied fixtures | Sessions and children resume without migration collision. |
| GPU rental | Recommended DeepSeek path | Open rental UI and select recommended DeepSeek recipe against mocked providers | Authorization and durable request use `deepseek-flash-0731-2xh200` / `deepseek-ai/DeepSeek-V4-Flash-0731`; no billable provider call occurs before confirmation. |
| GPU rental | Existing pre-upgrade rental | Resume fixture pinned to an older recipe revision | Existing rental keeps its recorded recipe/revision; only new selection uses the recommendation. |
| OpenAI parity | QueueCraft/TextWright/QueryForge | Five paired RC/upstream runs per task | Equal solves; median cost/time/token metrics within defined gate. |
| PTY | `pfterminal-debug --yolo` | Real terminal interaction and scrolling | Input, Ctrl+C, scrollback, model switch, and transcript remain usable. |
| Packaging | Fresh/upgrade/rollback | Exact packaged artifacts | Correct binary/home/version; no state damage. |

## 9. OpenAI parity and performance gate

Use the frozen methodology in the upstream convergence plan:

- five paired runs each for QueueCraft, TextWright, and QueryForge;
- identical model, effort, service tier, repository, prompt, environment, and fresh home;
- equal solve count per task;
- PF medians for uncached input, cached input, output, model calls, tool calls, wall time,
  and settled cost within 5% of the matched upstream Codex revision unless a declared PF
  feature was exercised;
- investigate every individual PF run more than 15% slower or more expensive;
- retain raw usage, settled billing, route identity, and exact binary hashes.

Benchmark publication is prohibited until the measured binary equals the release
artifact byte-for-byte.

## 10. Required evidence bundle

Create one immutable release-evidence directory containing:

- `SOURCE.json`: PF commit, upstream commit, clean-tree proof, toolchain, lockfile hash;
- `ARTIFACTS.json`: artifact names, platforms, sizes, SHA-256 hashes, signatures;
- `ROUTES.json`: requested/upstream/reported identities for every live matrix row;
- `TESTS.md`: commands, start/end times, exit codes, failures, reruns, exclusions;
- `PROVIDERS.md`: configured routes and redacted credential source types;
- `STATE.md`: fixture hashes, migration checksums, resume/rollback results;
- `ORCHESTRATION.json`: parent/child IDs, exact routes, billing class, terminal outcomes;
- `PERFORMANCE.md`: parity summaries with raw-run links;
- `INSTALL.md`: clean install, upgrade, rollback, uninstall evidence;
- `KNOWN_ISSUES.md`: remaining non-blocking defects with owners and impact;
- `SECRET_SCAN.json`: zero-hit scan over published source and evidence.

Secrets, raw API keys, OAuth tokens, vault contents, and unsanitized environment dumps
must never enter this bundle.

## 11. Release approval checklist

### P0 closure

- [ ] RR-P0-01 route identity fixed and live-verified.
- [ ] RR-P0-02 tool-payload crash fixed and live-verified.
- [ ] RR-P0-03 clean reproducible source commit produced.
- [ ] RR-P0-04 switching/compaction exact-route matrix passed.
- [ ] RR-P0-05 multi-provider spawn identity matrix passed.
- [ ] RR-P0-06 released-state migration/resume matrix passed.
- [ ] RR-P0-07 GPU-rental UI restored and V4 Flash recommendation verified.

### Automated quality

- [ ] Every changed crate's targeted tests pass.
- [ ] Complete workspace `just test` passes on the frozen commit.
- [ ] Required schema generation produces no uncommitted diff.
- [ ] `just fix` and `just fmt` produce no uncommitted diff.
- [ ] `git diff --check` passes.
- [ ] Final source tree is clean.

### Live quality

- [ ] Opus Plan, Fable Plan, OpenAI Sol, OpenAI Luna, Kimi K3, and GLM 5.2 mandatory
  rows pass.
- [ ] No false model warnings occur.
- [ ] No exact request is substituted.
- [ ] No valid tool call terminates a turn because of provider payload shape.
- [ ] No child runs on an unidentified or unintended route.
- [ ] Recommended DeepSeek rental resolves to the qualified V4 Flash manifest, and
  existing rentals retain their recorded recipe revisions.
- [ ] Model switching, compaction, resume, and long-image sessions pass.
- [ ] OpenAI parity and cost gates pass.

### Operations

- [ ] Linux, macOS, and Windows artifacts install successfully.
- [ ] Upgrade from 0.1.26 preserves state.
- [ ] Rollback instructions are executed successfully on a fixture.
- [ ] Update manifest, checksums, and release notes reference the exact artifacts.
- [ ] Stable package pointer remains unchanged until explicit approval.

## 12. Automatic no-go conditions

Do not release if any of the following is true:

- the source tree is dirty when the candidate artifact is built;
- requested, upstream, and reported model identity cannot be reconciled;
- a warning states a provider, model, or cause unsupported by structured evidence;
- an explicit spawn or switch silently substitutes another route;
- the GPU-rental workflow is absent, or its recommended DeepSeek action is inferred
  from list order/display text instead of the typed catalogue selection;
- any mandatory provider row lacks billing or capability metadata;
- a malformed model tool call panics or freezes the runtime instead of returning a
  recoverable tool error;
- a released database fixture is migrated in place, rejected incorrectly, or becomes
  unreadable after rollback;
- a complete mandatory test command fails, is killed, or has its exit status hidden by
  a shell pipeline;
- tests were run against a different binary than the packaged artifact;
- benchmark or release evidence contains a secret;
- a P0 is waived without a written replacement release decision from the operator.

## 13. Definition of done

The release is ready only when:

- [ ] one clean commit contains the reviewed post-convergence product state;
- [ ] the exact commit recreates every published artifact hash;
- [ ] Opus Plan aliases are recognized correctly and never produce the fabricated
  GPT-5.2 warning;
- [ ] genuine substitutions fail closed or display their actual structured reason;
- [ ] native and provider-wrapped patch calls share one recoverable execution path;
- [ ] switching, compaction, spawning, and resume retain exact provider/model identity;
- [ ] the restored GPU-rental workflow recommends the qualified DeepSeek V4 Flash
  recipe for new rentals without rewriting existing rental state;
- [ ] all mandatory automated, live, state, parity, and packaging gates pass;
- [ ] the evidence bundle is complete and secret-free;
- [ ] rollback to the installed 0.1.26 baseline is proven;
- [ ] the operator explicitly approves promotion.
