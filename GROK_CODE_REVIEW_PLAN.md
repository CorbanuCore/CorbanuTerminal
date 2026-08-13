# Grok Adversarial Code Review Plan

## Mission

Perform a rigorous, evidence-based review of the complete PFTerminal integration branch before it is promoted to production.

- Repository: `/home/pfrpc/repos/PfTerminal-triage-clean`
- Review branch: `integrate/pfterminal-20260707`
- Baseline: `origin/main`
- Primary range: `origin/main...HEAD`
- Expected scale at dispatch time: about 29 commits, 146 files, 10,000 added lines
- Review only. Do not modify production code, rewrite commits, push, merge, or delete files.
- Write the final review to `GROK_CODE_REVIEW_FINDINGS.md` in the repository root.

Treat the current implementation and its tests as claims to verify, not as proof of correctness. Look for general failure modes at ownership boundaries: provider selection, authentication, request serialization, conversation replay, model capability gating, persistence, installer isolation, TUI state, and orchestration lifecycle.

## Required Review Standard

A valid finding must include:

1. Severity: `P0`, `P1`, `P2`, or `P3`.
2. A short title describing the broken invariant.
3. Exact file and line reference.
4. A concrete trigger or reproduction sequence.
5. Actual behavior and expected behavior.
6. Why existing tests do not catch it.
7. The smallest defensible repair direction.

Do not report style preferences, speculative concerns without a reachable path, pre-existing issues outside the review range, or generic requests for more tests. Prefer a few proven findings over a long list of guesses.

## Phase 0: Establish Ground Truth

Run and record:

```bash
cd /home/pfrpc/repos/PfTerminal-triage-clean
git status --short --branch
git log --oneline --decorate origin/main..HEAD
git diff --stat origin/main...HEAD
git diff --name-status origin/main...HEAD
git diff --check origin/main...HEAD
```

Read `AGENTS.md` before reviewing Rust code. Identify unrelated or generated churn, but do not let it hide behavior changes. Build a commit-to-subsystem map before diving into individual files.

## Phase 1: Threat Model and Invariants

Write down the invariants you will test before reading implementation details:

### Installation and state isolation

- PFTerminal must never write to, migrate, rename, or corrupt upstream `~/.codex` state.
- Renaming the binary to `codex` must not change its home directory.
- `PFTERMINAL_HOME`, `CODEX_HOME`, and default-home precedence must match the intended contract.
- Packages and installers must not ship or leave behind a `codex` executable or symlink.
- Upgrade, rollback, Windows, Unix, npm package, and stale-install paths must remain coherent.

### Provider and authentication isolation

- Provider-scoped API keys must never fall through to ChatGPT access-token refresh.
- Saving or replacing a vault key must affect the running process without leaking or caching stale absence.
- OpenRouter, Meta, Anthropic, Ambient, Z.AI, Baseten, Vercel, OpenAI, and command-backed auth must not borrow one another's headers or recovery behavior.
- A 401 must report the failing provider and auth source accurately.
- Provider switching mid-thread must rebuild the correct client/auth/request state.

### Conversation and tool continuation integrity

- Multi-tool turns must preserve required item IDs, call IDs, ordering, roles, and payload types.
- Existing rollouts created by older builds must resume safely.
- Provider-specific replay repair must not mutate canonical history unpredictably or poison prompt caches.
- Function, custom, shell, web-search, image, and structured-edit results must survive serialization on every supported wire API.

### Vision

- Models advertised as vision-capable must receive image bytes, not only a path or textual placeholder.
- Text-only models must reject or omit image input consistently.
- Direct attachments and `view_image` tool results must work through Responses, Chat Completions, and Anthropic Messages transports.
- Image ordering, MIME types, detail values, cache-control blocks, tool-result pairing, history replay, and size preparation must remain valid.

### TUI and orchestration

- Model/provider grouping must be intelligible and selection must update both model and provider atomically.
- Hidden/unsupported models must not become selectable through search, resume, presets, or stale config.
- Orchestration must not leak agents, nicknames, tasks, panes, timers, or database records across success, failure, cancellation, and restart.
- Parent/child authority and role restrictions must be enforced on every dispatch path.
- Scroll, selection, keyboard, clipboard, and mouse behavior must not regress outside the changed screen.

## Phase 2: Review by Risk Cluster

Review each cluster independently. For every cluster, inspect the full diff, follow call sites in unchanged code, and identify the owning tests.

### A. Home directory, migration, package, and installers

Primary paths:

- `codex-rs/utils/home-dir/`
- `codex-rs/state/`
- `scripts/install/`
- `scripts/codex_package/`
- release and CI workflows

Adversarial cases:

- argv0 renamed to `codex`, symlinked execution, nested launcher, Windows `.exe` naming.
- Both home variables set, empty variables, relative paths, nonexistent paths, unwritable paths.
- Existing upstream database with unknown migrations or checksums.
- Existing PFTerminal legacy state, interrupted rename, destination collision, cross-filesystem rename.
- Upgrade from a package that previously installed `codex`; uninstall and rollback afterward.
- Search the complete package staging logic for case-insensitive `codex*` artifacts.

### B. Model catalog, picker, plan gating, and provider switching

Primary paths:

- `codex-rs/models-manager/`
- `codex-rs/model-provider-info/`
- `codex-rs/tui/src/chatwidget/model_popups.rs`
- `codex-rs/tui/src/chatwidget/settings.rs`
- related snapshots and config tests

Adversarial cases:

- Duplicate slugs across providers and suffix-based model matching.
- Remote catalogs overriding bundled visibility, modalities, limits, or reasoning levels.
- Switching OpenRouter Grok -> MiniMax -> Claude -> Meta within one resumed thread.
- Preset provider/model mismatch and stale provider labels.
- Search/filter/keyboard selection after groups expand, collapse, or contain one item.
- Context-window and reasoning-effort values that a provider rejects only at turn time.

### C. Auth, vault, provider client construction, and 401 recovery

Primary paths:

- `codex-rs/login/`
- `codex-rs/secrets/`
- `codex-rs/model-provider/`
- `codex-rs/core/src/client.rs`
- `codex-rs/tui/src/chatwidget/provider_credentials.rs`
- `codex-rs/tui/src/app/event_dispatch.rs`
- app-server account login processing

Adversarial cases:

- Key absent at startup, added in-process, rotated, replaced with invalid key, then corrected.
- Valid ChatGPT login plus invalid OpenRouter key: ensure no ChatGPT refresh occurs.
- Invalid API key through Chat, Responses, and Anthropic transports.
- Command-backed OAuth provider still refreshes after 401.
- Concurrent requests while a provider key is updated.
- Vault read failure, keyring fallback, legacy plaintext migration, cache invalidation, and logout.
- Confirm secrets never enter logs, errors, rollout JSONL, telemetry, command lines, snapshots, or Git diff.

### D. Responses, Chat, Anthropic, Meta, and vision serialization

Primary paths:

- `codex-rs/codex-api/src/common.rs`
- `codex-rs/core/src/client.rs`
- `codex-rs/protocol/src/models.rs`
- `codex-rs/core/tests/suite/client.rs`
- image preparation and history normalization

Adversarial cases:

- Text before, between, and after multiple images.
- Base64 PNG/JPEG/GIF/WebP, remote URL, malformed data URL, unsupported MIME type, oversized image.
- Image-only user turn and image-only tool result.
- `view_image` result after a function call on OpenRouter and Anthropic.
- Tool result containing text plus multiple images plus encrypted content.
- Meta continuations with missing IDs from legacy history, server IDs, locally generated IDs, compaction, resume, and three or more tool calls.
- ID stability across retries and repeated request construction. Check whether request-time repair generates different IDs for the same canonical history and whether any provider depends on stability.
- Chat tool-result image injection must not violate assistant/tool/user ordering or confuse subsequent tool calls.
- Cache-control markers must land only on provider-supported block types.

### E. Session, server state, compaction, resume, and request reuse

Primary paths:

- `codex-rs/core/src/session/`
- `codex-rs/core/src/context_manager/`
- `codex-rs/core/tests/suite/vercel_server_state.rs`
- app-server thread processing

Adversarial cases:

- Retry after partial stream, 401, transport timeout, and malformed tool output.
- Provider/model switch with `previous_response_id` or cached request state present.
- Compaction before and after image/tool items.
- Resume old rollout with missing IDs or legacy image shapes.
- Cancellation while tool output is being recorded.
- Confirm canonical history is incremental and bounded; flag any new fragment over 1,000 tokens as P0 per `AGENTS.md`.

### F. Orchestration and role enforcement

Primary paths:

- `codex-rs/tui/src/orchestrate.rs`
- `codex-rs/tui/src/spawn_orchestration.rs`
- `codex-rs/core/src/tools/orchestrator.rs`
- agent role/control code
- task and persistence code

Adversarial cases:

- Spawn failure after nickname reservation.
- Cancellation at every await boundary.
- Parent closes while children run; child completes after UI pane disappears.
- Worker/explorer creation and `send_task` escalation across forbidden hierarchy edges.
- Duplicate task delivery, whitespace variants, replay after restart, and delayed completion.
- Pane creation failure, tmux unavailable, malformed output, and stale registry entries.
- Maximum agents, long chains, reused nicknames, and persistence failure.
- Audit the 2,400-line orchestration module for state machines split across callbacks with no single cleanup owner.

## Phase 3: Test the Tests

Do not merely run the existing suite. Review whether assertions prove the behavior users depend on.

Check for:

- Tests that assert only a field instead of the complete request shape.
- Mocks that accept payloads stricter providers reject.
- Tests that enable feature flags production leaves disabled.
- Tests that use a fresh process and therefore miss hot key rotation or stale caches.
- Tests that skip under ordinary CI environment variables.
- Snapshot updates that accidentally bless broken layout or labels.
- Missing resume/upgrade coverage for compatibility changes.
- Random IDs or timing that make tests pass nondeterministically.

Run targeted commands using repository policy (`just test`, not direct `cargo test`):

```bash
cd /home/pfrpc/repos/PfTerminal-triage-clean/codex-rs
just test -p codex-protocol
just test -p codex-model-provider
just test -p codex-models-manager
just test -p codex-core client::tests::
just test -p codex-tui
```

Do not run the complete workspace test suite unless the dispatcher explicitly authorizes it. Record skipped, timed-out, flaky, and environment-blocked tests separately from passes.

## Phase 4: Bounded Live Probes

Only perform live API probes if credentials are already present and the dispatcher has allowed network/API spending. Never print, log, persist, or place a credential in a command argument. Fetch a whitelisted provider key only through the PFTerminal vault helper at execution time.

If authorized, cap live work to one short turn per probe:

1. OpenRouter Grok plain turn.
2. OpenRouter Grok attachment plus description.
3. OpenRouter MiniMax attachment plus description.
4. Meta two sequential function calls followed by a final answer.
5. Anthropic direct attachment followed by `view_image` in a later continuation.
6. Invalid provider key against a local mock: verify the error remains provider-specific and never invokes ChatGPT refresh.

Do not use arbitrary destructive commands, external uploads, private files, or unbounded web crawling.

## Phase 5: Cross-Cutting Searches

At minimum, search for:

```bash
rg -n 'CODEX_HOME|PFTERMINAL_HOME|\.codex|\.pfterminal|codex\.exe|bin/codex'
rg -n 'unauthorized_recovery|refresh_token|env_key|provider_api_key|Authorization|x-api-key' codex-rs
rg -n 'InputImage|input_image|image_url|input_modalities|view_image' codex-rs
rg -n 'previous_response_id|item_ids_enabled|assign_id_if_missing|set_id' codex-rs
rg -n 'reserve|nickname|send_task|spawn|close_agent|cancel|Drop' codex-rs/core codex-rs/tui
```

Trace every suspicious match to its caller and tests. Do not treat a search hit itself as a finding.

## Required Output

Write `GROK_CODE_REVIEW_FINDINGS.md` with this exact top-level structure:

```markdown
# Grok Code Review Findings

## Verdict

Ship / Do not ship, with a concise reason.

## Coverage

Commits, subsystems, commands, and live probes actually reviewed.

## Findings

Ordered P0 -> P3. Each finding follows the required seven-part standard.

## Test Gaps

Only concrete gaps tied to a plausible regression.

## Clean Areas

High-risk areas reviewed where no defect was found, including evidence.

## Residual Risk

What could not be verified and why.
```

If no defects are found, say so explicitly, but still provide coverage evidence and residual risk. Do not claim the branch is safe merely because tests pass.

## Completion Gate

The review is complete only when:

- Every risk cluster has evidence of inspection.
- The latest commits for vision, Meta item IDs, and provider-key 401 recovery received line-level review.
- At least one multi-step continuation payload was inspected structurally.
- Existing-session and resume behavior were considered, not only fresh sessions.
- Findings are deduplicated and ranked by user impact.
- The final Markdown file contains no secrets or credential material.
