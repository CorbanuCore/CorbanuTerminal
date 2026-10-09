---
sprint_id: "PF-60-S05"
title: "Collection correctness and declared billing basis"
status: in_progress
plan_file: "docs/plans/active/portfolio-agent-cost-accounting.md"
plan_feature: "PF-60"
execution_order: 4
owner: "Codex accounting lane (acct-s05); Travis Good accepts"
parallel_lane: "accounting-pf60-s05-collection"
write_scope: "codex-rs/model-provider-info/src/, codex-rs/config/src/config_toml.rs, codex-rs/config/src/thread_config.rs, codex-rs/config/src/thread_config/remote.rs, codex-rs/login/src/auth_env_telemetry.rs, codex-rs/model-provider/src/provider.rs, codex-rs/app-server/src/request_processors/thread_processor_tests.rs, codex-rs/core/src/config/config_tests.rs, codex-rs/core/src/accounting_basis_tests.rs, codex-rs/core/src/accounting_websocket_tests.rs, codex-rs/core/src/client_tests.rs, codex-rs/core/src/compact_tests.rs, codex-rs/core/src/memory_stage_one_tests.rs, codex-rs/core/src/realtime_conversation_tests.rs, codex-rs/core/tests/responses_headers.rs, codex-rs/core/tests/suite/client.rs, codex-rs/core/tests/suite/client_websockets.rs, codex-rs/core/tests/suite/stream_error_allows_next_turn.rs, codex-rs/core/tests/suite/stream_no_completed.rs, codex-rs/state/src/runtime/accounting_late_import.rs, codex-rs/state/src/runtime/accounting_late_import_tests.rs, codex-rs/state/src/runtime/accounting_compact_values_tests.rs, codex-rs/state/src/runtime/accounting_format_tests.rs, codex-rs/state/tests/accounting_contract_golden.rs, codex-rs/core/config.schema.json, codex-rs/core/src/config/mod.rs, codex-rs/core/src/accounting.rs, codex-rs/core/src/accounting_prices.rs, codex-rs/core/src/accounting_prices_tests.rs, codex-rs/core/src/accounting_policy_tests.rs, codex-rs/core/src/accounting_tests.rs, codex-rs/core/src/accounting_basis.rs, codex-rs/core/src/accounting_transport.rs, codex-rs/core/src/accounting_extensions.rs, codex-rs/core/src/accounting_extensions_tests.rs, codex-rs/core/src/client.rs, codex-rs/core/src/compact.rs, codex-rs/core/src/compact_remote.rs, codex-rs/core/src/realtime_conversation.rs, codex-rs/core/src/guardian/review_session.rs, codex-rs/core/src/session/, codex-rs/core/tests/suite/accounting_anthropic.rs, codex-rs/core/tests/suite/accounting_chat.rs, codex-rs/core/tests/suite/accounting_responses.rs, codex-rs/ext/web-search/src/tool.rs, codex-rs/ext/image-generation/src/backend.rs, codex-rs/state/accounting_migrations/, codex-rs/state/src/migrations.rs, codex-rs/state/src/runtime/accounting_store.rs, codex-rs/state/src/runtime/accounting_store_tests.rs, codex-rs/state/src/runtime/accounting_pricing.rs, codex-rs/state/src/runtime/accounting_pricing_tests.rs, codex-rs/state/src/runtime/accounting_estimates.rs, codex-rs/state/src/runtime/accounting_estimates_tests.rs, codex-rs/state/src/runtime/accounting_lifecycle.rs, codex-rs/state/src/runtime/accounting_compact_values.rs, codex-rs/state/src/runtime/accounting_format.rs, codex-rs/state/tests/fixtures/accounting/, codex-rs/state/tests/accounting_store.rs, codex-rs/state/BUILD.bazel, codex-rs/app-server/src/request_processors/turn_processor.rs, codex-rs/tui/src/chatwidget/tokens.rs, codex-rs/tui/src/chatwidget/tokens_tests.rs, codex-rs/tui/src/chatwidget/tokens/, codex-rs/tui/src/app/background_requests.rs, qa/portfolio/agent-cost-accounting/pf-60-s05/, qa/demos/specs/pf60s05-subscription-not-spent.toml, qa/demos/specs/pf60s05-basis-not-declared.toml, qa/demos/specs/pf60s05-overflow-note.toml, qa/demos/index/PF-60-S05.md, docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s05-collection-correctness-and-billing-basis.md"
integration_gate: "Small PRs to main, merged by the lane when green. Each slice: focused tests with and without developer-accounting, Linux clippy -D warnings on the RTX box, one Opus 5.5 High review. Codex management receives; code-blind acceptance then Travis."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf60-s05-20261009"
branch: "work/pf60-s05-20261009"
base_commit: "7ac8fcb6cb5eb568a62161ec190d5f04ba1fee2a"
depends_on: "PF-60-S02"
created: 2026-10-08
updated: 2026-10-09
---

# PF-60-S05 — Collection correctness and declared billing basis

**Status: in progress** since PF-60-S03 was archived (2026-10-09). Travis chose option B for "both" behaviour on 2026-10-09 ([options memo](../../../../qa/portfolio/agent-cost-accounting/pf-60-s05/both-behaviour-options.md)). It fixes the Blocker and Majors 2-5 of the [independent review of the 09-20..10-02 work](../../../../qa/portfolio/agent-cost-accounting/pf-60-s03/body-review-20261008/review.md): collection and pricing defects (S02 territory), so it depends on S02. Work lands in slice PRs from branches cut in the recorded worktree (and a second checkout, `worktrees/pf60-s05-s2`, for parallel slices).

## Execution mandate

- **Goal 1 (Blocker):** Kimi Code membership work is never recorded or shown as money spent.
- **Goal 2 (Major 2):** subscription work keeps its basis when no catalogue price exists. Today plan-ness lives only on the price record, so priceless plan turns show as "Pay per use".
- **Goal 3 (Major 3):** the 2026-09-21 change to stored bytes (`5bae03414e`) is checked against real old ledgers and fixtures, and the ledger format is versioned whatever that check finds.
- **Goal 4 (Major 4):** compaction, web search and image generation are best effort. If accounting fails, the attempt is marked unrecorded and the paid response is still returned.
- **Goal 5 (Major 5):** guardian review forks are attributed to the parent conversation. Any other uncollected session (for example `exec --ephemeral`) logs a named warning and is listed in the coverage audit.
- **Also closed, same mechanism:** Minor 10 (any provider with a command-based login is treated as a subscription, Bedrock included).
- **Excludes:** shipping `/cost` to users; changing prices; extending API-equivalent figures to more providers; reading provider quota APIs; review Minors 6, 7 and 9 and Nits 11-14; S03 acceptance items #286, #287 and #289. Those stay in S03's Remaining list.

## Billing basis design (Travis's requirements a and b)

- **Declared, not guessed.** Every provider has a declared billing basis: `subscription`, `pay_per_use` or `local` (runs on your own machine, no charge). The built-in provider table holds the defaults. Users can override per provider with `model_providers.<id>.billing = "..."` in `config.toml`, for built-in and custom providers alike.
- **Several credentials, several declarations.** A provider that takes more than one kind of credential declares a basis for each kind explicitly in the table: `openai` is `subscription` when signed in with ChatGPT and `pay_per_use` with an API key. Code never derives a basis from `AuthMode` or from `provider.auth` being set. That predicate in `turn_mode` (`core/src/accounting.rs:420-441`) is removed.
- **Unknown stays unknown.** A custom provider with no declaration still has its tokens recorded. It is shown as "billing basis not declared" with a next step naming the config key, and it counts as neither spent nor subscription.
- **Recorded on every attempt.** The basis and where it came from (built-in default or user config) are bound to every attempt at admission, whether or not a price record exists. A subscription attempt is never counted as spent. Its API-price equivalent, where today's rules give one, is shown as "not spent". Pane-bridge and extension paths carry the basis of the provider they used.
- **Defaults: subscription** for `claude-plan`, `kimi-code`, `zai-anthropic` and `openai` signed in with ChatGPT. **Pay per use** for `openai` with an API key, `anthropic`, `ambient`, `pfterminal-plan` / `pfterminal-plan-anthropic` (Corbanu API balance), `zai`, `openrouter`, `openrouter-anthropic`, `deepseek`, `meta`, `baseten`, `baseten-anthropic`, `vercel`, `vercel-anthropic`, `vercel-anthropic-fast` and `amazon-bedrock`. **Local** for `ollama` and `lmstudio`. Each default, with its official source, is in the [defaults table](../../../../qa/portfolio/agent-cost-accounting/pf-60-s05/billing-basis-defaults.md).

## Decision: "both" behaviour is option B (Travis, 2026-10-09)

- **Route and credential decide.** The built-in table declares a basis per credential and per route (Z.AI coding URL subscription, general URL pay per use; Kimi Code URL subscription, Kimi Open Platform pay per use; ChatGPT login subscription, OpenAI API key pay per use). A route or credential the table doesn't know is "billing basis not declared", with a next step, never a guess.
- **Overflow is never spent.** Paid overflow beyond a plan (usage credits, Extra Usage, ChatGPT credits) is not counted. Subscription work carries a plain note that overflow isn't visible here and where to check it.
- **Override stays.** `model_providers.<id>.billing` remains for unusual setups. Option C (an overflow upper bound) is not built; revisit only if a provider signals overflow per request.

## Plan linkage

- Plan: [Unified agent cost and usage accounting](../../../plans/active/portfolio-agent-cost-accounting.md)
- Feature: `PF-60`. Plan acceptance advanced: a missing price, or a denied provider, produces no invented zero and no double charge; unknown and estimated values stay visibly distinct.

## Code boundaries

- Existing: `model-provider-info/src/lib.rs::built_in_model_providers`; `core/src/accounting.rs::turn_mode` and `attach_turn`; `core/src/accounting_prices.rs::plan_original`; `state/src/runtime/accounting_pricing.rs` (`is_plan`); `state/src/runtime/accounting_estimates.rs`; `core/src/client.rs` (`resolve_path`); `core/src/accounting_transport.rs`; `core/src/accounting_extensions.rs`; `ext/web-search/src/tool.rs`; `ext/image-generation/src/backend.rs`; `core/src/guardian/review_session.rs`; `tui/src/chatwidget/tokens.rs` and `tokens/scope.rs`.
- Planned: the basis declaration and route table (`model-provider-info/src/billing.rs`), the config key and schema, the basis on each attempt (`core/src/accounting_basis.rs`), a ledger format version (`state/accounting_migrations/0002_*`, `state/src/runtime/accounting_format.rs`) with a committed pre-09-21 fixture, the best-effort paths, guardian attribution and `accounting.excluded`. The literal file list is `write_scope`.
- Evidence: `qa/portfolio/agent-cost-accounting/pf-60-s05/`.

## Preconditions

- [x] 2026-10-09 Travis chose option B and AC12 is written from it; worktree, branch, base and literal write scope are allocated and in the plan.
- [ ] Real credentials are available in the vault for a Kimi Code membership, a Claude subscription, a ChatGPT login and Z.AI; the labels are recorded and the values never printed.

## Done

- [x] 2026-10-08 Sprint specified from review findings 1-5 and 10, Travis's billing-basis requirements, the cited defaults table and the "both" options memo. No code changed.
- [x] 2026-10-09 Option B recorded here and in the memo; AC12 written; sprint `ready` with its worktree allocated.
- [x] 2026-10-09 **Ledger format (AC6), PR #337:** Major 3 reproduced with a ledger written at `5bae03414e^` (committed fixture); both stored forms read again; explicit format with upgrade-on-write and `NewerLedgerFormat` refusal; two real 09-23..10-03 ledgers validate ([check](../../../../qa/portfolio/agent-cost-accounting/pf-60-s05/ledger-format-check.md)). Core's one-warning gate is in the declaration slice.

## Remaining

- [ ] **Declaration (AC1-3):** add the per-provider basis and the per-credential entries to the built-in table, the config key and its schema, and the "not declared" state. Remove the inference from auth mode and from `provider.auth`.
- [ ] **Basis on the attempt (AC4-5):** bind the basis to every attempt so a subscription attempt without a price is still subscription work (the review's fix: a plan record with no rates). Cover the ChatGPT models with no catalogue price, non-default tiers, ChatGPT image and realtime calls, and Claude subscription work reported through the pane bridge.
- [ ] **Best effort (AC7):** on `execute` and `resolve_path` for compaction, web search and image generation, an admission or observation failure logs the reason class (never the endpoint), marks the attempt unrecorded (a gap record when the store is writable, plus the existing gap warning) and returns the provider response.
- [ ] **Uncollected sessions (AC8-9):** record guardian forks under the parent conversation's thread, labelled as a review. Emit `accounting.excluded` once per ephemeral session. Re-run the 09-21 coverage audit method and write a dated audit listing every remaining uncollected session kind.
- [ ] **Both (AC12, option B):** route rows in the table (Z.AI coding and general URLs, Kimi Open Platform), "not declared" with its next step for unknown routes, and the overflow note on subscription work only.
- [ ] Code-blind functional design frozen before any results are disclosed (AC11).

## Verification

- [ ] **AC1** (every AC is checked on one recorded binary, on macOS and Linux)**:** every one of the 21 built-in providers has a declaration matching the defaults table. A test enumerates `built_in_model_providers` and fails on any undeclared provider.
- [ ] **AC2:** the basis does not follow the auth type. With a command `auth`, `amazon-bedrock` stays pay per use. `kimi-code` using an environment key is a subscription. `openai` follows its per-credential entries. Mutation tests show each of these fails if inference comes back.
- [ ] **AC3:** a `billing` override changes new attempts only, and history is not re-priced. An invalid value is a config error that names the key. A custom provider with no declaration shows "not declared" and its next step.
- [ ] **AC4:** a ChatGPT turn on a model with no catalogue price, a priority-tier turn, ChatGPT image and realtime calls, and pane-bridge Claude work all show "Subscription". None shows "Pay per use", "no price available" or "check the bill", and none adds to the pay-per-use "had no price" count.
- [ ] **AC5 (real):** after a `kimi-code` turn with a real membership key, `/cost` shows Subscription with $0 counted as spent, and the day's spent total is unchanged.
- [ ] **AC6:** a pre-`5bae03414e` fixture ledger (committed) validates and reads. A real old ledger does too, or its absence is recorded. A future-format ledger turns collection off with one warning while turns still succeed.
- [ ] **AC7 (real provider, injected store failure):** with the store busy past its budget, failing validation, or read-only, auto-compaction finishes, web search returns results and image generation returns the image. One gap warning appears per turn.
- [ ] **AC8 (real):** a guardian approval review's requests appear under the parent conversation in `/cost`, and the totals reconcile.
- [ ] **AC9:** an `exec --ephemeral` run logs exactly one `accounting.excluded` warning, and the new coverage audit lists it.
- [ ] **AC10 (real providers):** `claude-plan`, ChatGPT login and `kimi-code` are subscription. `zai` (GLM 5.2) and `deepseek` or `openrouter` are pay per use. Pay-per-use estimates match an independent recomputation at the provider's published prices to the micro-dollar.
- [ ] **AC11 (independent, code-blind, like S03's):** an executor who reads only this record, the plan intent, the defaults table and in-product help runs AC1-AC10 and AC12 with real keys under the [isolated execution gate](../../../../qa/code-blind-functional/isolated-execution.md) and records pass or fail for each. A code-blind Opus 5.5 High reviewer then audits that record.
- [ ] **AC12 (option B, real):** for each built-in provider with more than one route or credential in the table, a real request on each route is recorded with the declared basis; an unknown route shows "not declared"; and the overflow note appears only on subscription work. A route whose credential isn't in the vault is recorded NOT VERIFIABLE and listed for Travis.
- [ ] Gate (`sec-common.md`): `just test` for core, state and tui, with and without `developer-accounting`; Linux clippy; a tmux run on GLM 5.2 (`-c model_provider="zai"`); one Opus 5.5 High code review; demo videos per `qa/demos/README.md`. Every run uses `CORBANU_TEST_NO_NATIVE_KEYRING=1` and a disposable home.
- [ ] `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.

## Exit evidence

- [ ] Merge commits, binary digests and test counts linked from `qa/portfolio/agent-cost-accounting/pf-60-s05/`.
- [ ] Code-blind acceptance record and its review linked; every AC has a recorded result.
- [ ] S03's Remaining list cross-referenced for the findings closed here; Travis accepts; record archived and plan backlinks updated.
