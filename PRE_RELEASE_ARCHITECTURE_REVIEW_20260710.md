# PFTerminal Pre-Release Architecture and Quality Review

**Review date:** 2026-07-10
**Review branch:** `integrate/pfterminal-20260707`
**Reviewed head:** `7d75c15fe` (`Keep provider status loading off TUI thread`)
**Production baseline:** `origin/main` / `f07d792f4` (`rust-v0.1.9`)
**Change size:** 33 commits, 155 files, 11,698 additions, 1,115 deletions

## Executive Decision

**Recommendation: conditional NO-GO for production promotion.**

No unresolved P0 data-loss or credential-leak defect is known at this head. Several previously
confirmed release blockers have been repaired, including PFTerminal/Codex home isolation, npm
command isolation, provider-auth separation, image replay ordering, and the provider-screen UI
freeze.

The branch is nevertheless not ready for an unconditional production release. The primary reasons
are:

1. The branch changes installation, state migration, provider authentication, request serialization,
   model selection, agent routing, and orchestration in one promotion unit.
2. The new orchestrate subsystem is a 2,416-line TUI module whose state and cleanup ownership span
   `App`, pane persistence, spawn routing, periodic timers, and two execution backends.
3. The complete workspace and platform release matrix has not been run against this exact head.
4. Fresh-machine coexistence with stock Codex and Windows installer behavior have not been executed
   end to end.
5. A real provider-screen freeze escaped unit tests and was initially misreported as tested. The
   corrected implementation has since been exercised in a real PTY, but this incident demonstrates
   that renderer responsiveness needs an explicit release gate.

Promotion should occur only after the release gates in this document are completed and attached to
the release record.

## What Changed

### 1. Agent routing, dispatch recovery, and internal testing

- Recovered dispatches that could be lost between agent/task state transitions.
- Preserved Nazgul delegation through Troll routing.
- Hardened orchestrator tool behavior and internal testing release blockers.
- Added sandbox startup fallback after approval.
- Added task and session state changes supporting dispatch persistence and recovery.

Primary ownership:

- `codex-rs/core/src/session/`
- `codex-rs/core/src/tasks/mod.rs`
- `codex-rs/core/src/tools/orchestrator.rs`
- `codex-rs/tui/src/spawn_orchestration.rs`
- `codex-rs/tui/src/app/thread_routing.rs`

### 2. Native orchestrate whips

The branch adds `/orchestrate`, a native watchdog/follow-up system for Codex threads and Claude
panes. A whip binds standing instructions to a target and can fire when that target becomes idle.

Supported concepts include:

- Review and automatic modes.
- Holder and target relationships.
- Expiry, cooldown, maximum fire count, pause/resume, detach, extend, manual fire, and test.
- Stop markers and protections against empty-output loops.
- Agent-authored fenced orchestrate commands with narrower authority than user commands.
- Guided TUI creation, saved global/project instruction documents, pane naming, and status views.
- Persistence through pane layout state and restoration on resume.
- A 45-second periodic sweep event.

Primary ownership:

- State model and transitions: `codex-rs/tui/src/orchestrate.rs`
- Runtime fields and periodic tick: `codex-rs/tui/src/app.rs`
- Event dispatch: `codex-rs/tui/src/app/event_dispatch.rs`
- Target and parent routing: `codex-rs/tui/src/spawn_orchestration.rs`
- Claude pane lifecycle: `codex-rs/tui/src/claude_panes/`
- Persistence: pane layout state and `persist_pane_state()` call sites

### 3. Product and state isolation from stock Codex

- PFTerminal home selection no longer depends on executable name.
- Precedence is `PFTERMINAL_HOME`, then `CODEX_HOME`, then the PFTerminal default.
- State database filenames are namespaced as `pfterminal_*`.
- Foreign SQLX migrations are rejected before migration with a repair pointer.
- Legacy state rename is refused in a directory named `.codex`.
- Unix and Windows installers remove stale package-local `codex` binaries.
- The npm package no longer publishes a `codex` command alias.

Primary ownership:

- `codex-rs/utils/home-dir/`
- `codex-rs/state/src/runtime.rs`
- `scripts/install/install.sh`
- `scripts/install/install.ps1`
- `codex-cli/package.json`
- `scripts/codex_package/test_npm_metadata.py`

### 4. Models, providers, and model-selection UX

- Added GPT-5.6 Sol, Terra, and Luna catalog entries and plan-oriented visibility behavior.
- Reorganized the model picker by provider.
- Added OpenRouter Grok 4.5, DeepSeek V4 Pro, and Tencent HY3 Free entries.
- Added the Meta Model API provider.
- Added provider-specific model correction and selection behavior.
- Added provider credential rows, inline Claude subscription login, and visible credential status.

Primary ownership:

- `codex-rs/models-manager/`
- `codex-rs/model-provider-info/`
- `codex-rs/model-provider/`
- `codex-rs/tui/src/chatwidget/model_popups.rs`
- `codex-rs/tui/src/chatwidget/provider_credentials.rs`
- `codex-rs/tui/src/chatwidget/claude_code_login.rs`

### 5. Authentication and secrets

- Provider API keys no longer fall through to ChatGPT refresh behavior on unauthorized responses.
- Claude plan authentication is delegated to the installed Claude Code CLI; PFTerminal displays the
  browser URL and accepts the one-time code in a masked field.
- Claude Code continues to own its OAuth token storage.
- Provider status loads outside the TUI event thread and reads vault metadata once rather than
  revealing each secret.
- Local encrypted-secret loading gained scrypt compatibility handling.

### 6. Vision and provider wire compatibility

- Added multimodal Chat Completions and Anthropic message content.
- Added direct image attachment and `view_image` result replay across provider transports.
- Fixed image-only tool outputs that duplicated full base64 content into tool text.
- Coalesced parallel Chat tool calls and kept tool results contiguous before synthetic image input.
- Preserved Meta response item IDs across continuations.
- Switched Meta edit tools away from unsupported custom tools to function tools.
- Repaired Vercel server-state continuations when a user message is required.

Primary ownership:

- `codex-rs/core/src/client.rs`
- `codex-rs/codex-api/src/common.rs`
- `codex-rs/protocol/src/models.rs`
- `codex-rs/core/tests/suite/client.rs`
- `codex-rs/core/tests/suite/vercel_server_state.rs`

## Architecture Assessment

### Strong decisions

1. **Isolation is structural rather than cosmetic.** Namespaced homes, databases, installers, and
   npm command surfaces reduce the probability of another stock-Codex collision.
2. **Provider authentication is explicitly scoped.** API-key providers no longer reuse ChatGPT
   unauthorized recovery, and Claude OAuth remains owned by Claude Code.
3. **Wire adapters repair provider differences before exposure.** Meta IDs, Meta edit-tool shape,
   Chat image ordering, and Vercel continuation behavior are handled at serialization/session
   boundaries rather than prompt special cases.
4. **Orchestrate has an explicit persisted state model.** `WhipMode`, `WhipState`, counters,
   expiry, cooldown, idle generation, pending review fire, and stop behavior are represented as
   data rather than inferred from transcript text.
5. **Authority checks exist for agent-originated orchestration.** Tests cover replacement refusal,
   unlimited agent whips, self-owned pause, ignored review behavior, empty loops, and stop markers.
6. **User-visible TUI changes generally have snapshot or focused interaction coverage.** Model
   grouping, provider status rows, Claude login, and many orchestrate flows have dedicated tests.

### Architectural concerns

#### A. Orchestrate has fragmented lifecycle ownership (P1 release risk)

`orchestrate.rs` owns parsing, file discovery, UI construction, authorization, transition logic,
fire planning, dispatch, status rendering, and tests. Runtime state is stored on `App`; restoration
comes from pane layout; firing crosses native thread and Claude pane paths; timer creation lives in
`app.rs`; persistence is invoked manually at several transition sites.

This makes it difficult to prove that every transition persists exactly once and that every failure
or cancellation releases pending state. The code has useful focused tests, but no single test drives:

`attach -> target runs -> idle -> review/auto fire -> pane disappears -> restart -> restore -> expire`

across both native and Claude targets.

Recommended follow-up architecture:

- Extract a transport-independent `WhipRegistry` state machine.
- Make transitions return explicit effects such as `Persist`, `Dispatch`, `Notify`, and `Detach`.
- Keep TUI view construction separate from state mutation.
- Give persistence one owner instead of calling `persist_pane_state()` at distributed sites.
- Add deterministic clock injection rather than relying on wall-clock behavior in orchestration logic.

#### B. The TUI event loop remains a sensitive shared boundary (P1 process risk)

The first provider status implementation synchronously read every encrypted credential while
constructing selection rows. On this machine, one encrypted vault access takes about 5.2 seconds,
which froze `/providers`. Unit and snapshot tests did not detect it.

The corrected design now:

- Opens immediately with `Checking...` rows.
- Loads Claude and vault status in background work.
- Uses one metadata-only vault listing.
- Applies a 10-second bound.
- Preserves selection when rows refresh.

This is fixed at the current head and was verified in a real tmux PTY. The incident should still be
treated as a release-process finding: any TUI callback that can perform filesystem encryption,
keyring, subprocess, database, or network work needs a responsiveness test.

#### C. Core request serialization is increasingly provider-conditional (P2 maintainability risk)

`core/src/client.rs` gained roughly 450 changed lines in this branch and now contains substantial
OpenAI, OpenRouter, Anthropic, Meta, Vercel, Ambient, Z.AI, and Baseten branching. The recent fixes
are correct at their identified boundaries, but future provider additions can regress another
transport unless request-shape tests remain systematic.

Recommended follow-up architecture:

- Move provider wire normalization into transport/provider adapters with a shared canonical turn
  representation.
- Maintain a transport conformance matrix for text, image, parallel tools, IDs, retry, and resume.
- Require exact serialized request fixtures for every newly advertised capability.

#### D. Model truth is split across bundled, remote, and UI mappings (P2 product risk)

Model capability and visibility depend on bundled JSON, remote catalog overlays, provider inference,
plan gating, and TUI grouping. A remote overlay can change modalities or availability after the
bundled tests pass. Provider correction helpers reduce mismatch, but the number of truth sources is
still high.

Recommended follow-up architecture:

- Define one resolved model record containing provider, plan eligibility, modalities, limits,
  reasoning efforts, visibility, and source provenance.
- Make the picker consume only resolved records.
- Record why a model is hidden or disabled for diagnostics.

## Prioritized Quality Findings

| Priority | Finding                                                                                                      | Current state     | Required action                                                                                            |
| -------- | ------------------------------------------------------------------------------------------------------------ | ----------------- | ---------------------------------------------------------------------------------------------------------- |
| P1       | No end-to-end orchestrate lifecycle test across restart, cancellation, pane loss, and both dispatch backends | Open              | Add deterministic lifecycle integration coverage and persistence-failure injection                         |
| P1       | Exact-head full workspace/platform release matrix is not attached                                            | Open              | Run the complete approved matrix and archive results by commit SHA                                         |
| P1       | Fresh-machine stock Codex coexistence gate has not been executed                                             | Open              | Run renamed PFTerminal, stock Codex install/doctor, upgrade, uninstall, and filesystem audit               |
| P1       | Windows installer and state isolation are code-reviewed but not executed                                     | Open              | Run Windows VM install/upgrade/uninstall and verify no `codex` launcher or `.codex` mutation               |
| P2       | Malformed/non-base64 Anthropic `data:` image URLs fall back to URL-shaped source data                        | Open              | Reject malformed non-HTTP image sources before sending the request                                         |
| P2       | Vercel continuation regression coverage is network-gated                                                     | Open              | Add a deterministic local protocol mock for server-state continuation and self-heal                        |
| P2       | Orchestrate state mutation and persistence are distributed across a 2.4k-line module and `App`               | Open debt         | Extract a registry/state machine after release gating, or before release if lifecycle tests expose defects |
| P2       | Provider and model behavior depend on multiple truth sources                                                 | Open debt         | Introduce a resolved model/provider capability record                                                      |
| P2       | TUI responsiveness was not part of the original provider-screen acceptance test                              | Corrected locally | Add a durable PTY responsiveness regression to CI                                                          |
| P3       | Several existing Clippy warnings remain outside this branch's changes                                        | Pre-existing      | Track separately; do not mix cleanup into release fixes                                                    |

## Confirmed Repairs from Adversarial Review

The following previously reported high-priority findings are repaired at this head:

1. **Chat image-only duplication:** image bytes now appear once; tool text uses a bounded
   acknowledgement.
2. **Parallel Chat tool ordering:** adjacent tool calls are batched, tool outputs remain contiguous,
   and image input is emitted after the tool result block.
3. **npm `codex` alias:** the PFTerminal npm metadata exposes only `pfterminal`.
4. **Provider key 401 crossover:** provider keys do not invoke ChatGPT refresh behavior.
5. **Meta custom-tool rejection:** Meta edit tools use supported function-tool shapes.
6. **Meta continuation IDs:** missing IDs are assigned and preserved across tool continuations.
7. **Provider screen freeze:** encrypted status discovery is off the TUI thread and selection is
   preserved during refresh.

## Test and Verification Evidence

### Automated evidence

- GitHub `pfterminal-ci`, Codespell, blob-size-policy, and cargo-deny completed successfully for
  `7d75c15fe` on 2026-07-10.
- Focused tests cover:
  - Home resolution and renamed binary behavior.
  - Foreign migration refusal and namespaced state files.
  - Model catalog visibility, provider grouping, and model-picker snapshots.
  - Provider-key unauthorized recovery selection.
  - Meta request IDs and edit-tool shape.
  - Chat and Anthropic image request shapes.
  - Image-only and parallel tool-result replay.
  - npm command metadata.
  - Claude login subprocess ownership, cancellation, masked entry, and visual snapshot.
  - Provider status row rendering.
  - Orchestrate parsing, stop markers, empty-output protection, review suppression, ownership,
    agent restrictions, detach/expiry behavior, and instruction precedence.

### Live evidence completed

- OpenRouter Claude successfully called `view_image` on a 2.5 MB image and produced an accurate
  description. The continuation request contained one image payload and valid tool ordering.
- Claude plan login completed through the inline flow and a plan model completed a turn.
- A real tmux PTY opened `/providers` immediately, accepted navigation while encrypted vault status
  loaded, preserved the selected row after refresh, displayed the expected stored-key and Claude
  account statuses, and responded to Up and Esc.
- Current vault metadata resolved Anthropic, Ambient, Z.AI, OpenRouter, and Meta as stored; Baseten
  as not configured; Claude as `alex@agti.net` with a Max subscription.

### Evidence not completed

- Full workspace test suite against this exact head.
- Windows build/install/runtime exercise.
- Fresh Linux VM install, upgrade, uninstall, and stock Codex coexistence exercise.
- Live Meta vision/tool continuation after all current serialization changes.
- Live Grok tool-driven `view_image` continuation; Grok reached OpenRouter but did not issue the
  requested tool call in the probe.
- End-to-end orchestrate restart/cancellation/pane-loss matrix.
- Long-duration orchestrate soak with multiple simultaneous whips and both native/Claude targets.

## Required Release Gates

### RR0 - Reproducible branch state

- [ ] Record `git status`, exact SHA, toolchain versions, and package version.
- [ ] Confirm no untracked review/WIP files enter the release artifact.
- [ ] Archive `git diff --check origin/main...HEAD` and dependency audit output.

### RR1 - Complete automated matrix

- [ ] Run the full approved Rust workspace suite at the release SHA.
- [ ] Run TUI snapshots with no pending snapshots.
- [ ] Run package-builder, npm staging, installer, cargo-deny, codespell, and Bazel checks.
- [ ] Attach failures, retries, and final logs; do not report only the final green status.

### RR2 - Installation and coexistence

- [ ] Fresh Linux VM: install PFTerminal, run renamed binary, verify only `~/.pfterminal` changes.
- [ ] Install and run stock Codex afterward; verify `codex doctor` and `~/.codex` remain healthy.
- [ ] Upgrade from the prior PFTerminal package and confirm stale `codex` launchers are removed.
- [ ] Repeat install/upgrade/uninstall checks on Windows.
- [ ] Inspect npm tarball contents and executable names before publication.

### RR3 - Provider and model acceptance

- [ ] For every exposed provider/model: plain turn, tool turn, continuation, invalid auth, corrected
      auth, provider switch, resume, and advertised image capability.
- [ ] Verify no provider API key triggers ChatGPT refresh.
- [ ] Verify every vision model receives actual image content through its configured transport.
- [ ] Verify remote catalog overlays cannot expose unsupported capabilities without a passing probe.

### RR4 - Orchestrate acceptance

- [ ] Native target: attach, review fire, auto fire, pause/resume, detach, expiry, maximum fires.
- [ ] Claude target: same matrix.
- [ ] Restart with armed, paused, expired, and pending-review whips.
- [ ] Cancel at each dispatch boundary and verify no duplicate task or stale pending review.
- [ ] Remove/close holder and target panes while a fire is pending.
- [ ] Inject pane persistence failure and verify state is either rolled back or visibly degraded.
- [ ] Run multiple whips for a sustained soak and verify cooldown, idle generation, and task counts.

## Management Questions

1. Is orchestrate required in the first release containing the provider/model work, or can it be
   feature-gated to reduce the promotion unit?
2. Who owns the orchestrate state machine and its production incident response?
3. Which exact provider/model combinations are release-supported versus experimental?
4. Is Windows a blocking platform for this release or a separately scheduled preview?
5. What evidence must be attached before anyone may describe a feature as tested or release-ready?

## Final Assessment

The branch contains important repairs and several sound boundary-level decisions. The home/state
isolation work materially reduces data-loss risk, and the provider/vision fixes address concrete
production failures rather than papering over examples.

The release risk comes from integration breadth and lifecycle proof, especially orchestrate. A
green focused suite is not sufficient evidence for a branch that modifies state migration,
installers, credentials, model routing, wire formats, agent dispatch, and persistent automation at
the same time. Complete RR0-RR4, or feature-gate orchestrate and any unverified provider/model paths,
before production promotion.
