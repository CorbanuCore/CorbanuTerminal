---
sprint_id: "PF-60-S05"
title: "Collection correctness and declared billing basis"
status: draft
plan_file: "docs/plans/active/portfolio-agent-cost-accounting.md"
plan_feature: "PF-60"
execution_order: 4
owner: "Codex accounting lane (proposed); Travis Good accepts"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-60-S02"
created: 2026-10-08
updated: 2026-10-08
---

# PF-60-S05 — Collection correctness and declared billing basis

**Status: draft, blocked on Travis's decision on "both" behaviour** ([options memo](../../../../qa/portfolio/agent-cost-accounting/pf-60-s05/both-behaviour-options.md)). Approved by Travis on 2026-10-08 to run before PF-60-S04. It fixes the Blocker and Majors 2-5 of the [independent review of the 09-20..10-02 work](../../../../qa/portfolio/agent-cost-accounting/pf-60-s03/body-review-20261008/review.md). Those are collection and pricing defects (S02 territory), so this sprint depends on S02, not on S03's acceptance.

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

## Open decision: "both" behaviour

Sometimes one provider can be paid both ways: a membership and an API balance with the same vendor, quota followed by paid overflow (Claude usage credits, Kimi Extra Usage, ChatGPT credits), or a route that bills either way. That behaviour is **not decided**. The [options memo](../../../../qa/portfolio/agent-cost-accounting/pf-60-s05/both-behaviour-options.md) sets out four options with worked examples and a recommendation. Implementation starts only after Travis chooses one. The chosen option becomes acceptance criterion 12.

## Plan linkage

- Plan: [Unified agent cost and usage accounting](../../../plans/active/portfolio-agent-cost-accounting.md)
- Feature: `PF-60`. Plan acceptance advanced: a missing price, or a denied provider, produces no invented zero and no double charge; unknown and estimated values stay visibly distinct.

## Code boundaries

- Existing: `model-provider-info/src/lib.rs::built_in_model_providers`; `core/src/accounting.rs::turn_mode` and `attach_turn`; `core/src/accounting_prices.rs::plan_original`; `state/src/runtime/accounting_pricing.rs` (`is_plan`); `state/src/runtime/accounting_estimates.rs`; `core/src/client.rs` (`resolve_path`); `core/src/accounting_transport.rs`; `core/src/accounting_extensions.rs`; `ext/web-search/src/tool.rs`; `ext/image-generation/src/backend.rs`; `core/src/guardian/review_session.rs`; `tui/src/chatwidget/tokens.rs` and `tokens/scope.rs`.
- Planned: the basis declaration and the config key (with schema), the basis carried on each attempt, a ledger format version and format gate, the best-effort paths, guardian attribution and the `accounting.excluded` warning. The exact file list and literal write scope must be fixed before `ready`.
- Evidence: `qa/portfolio/agent-cost-accounting/pf-60-s05/`.

## Preconditions

- [ ] Travis has chosen a "both" option, and acceptance criterion 12 is written from that choice.
- [ ] Worktree, branch, 40-character base and literal write scope are allocated and recorded in the plan; `parallel_sprint_limit` respected.
- [ ] Real credentials are available in the vault for a Kimi Code membership, a Claude subscription, a ChatGPT login and Z.AI; the labels are recorded and the values never printed.

## Done

- [x] 2026-10-08 Sprint specified from review findings 1-5 and 10, Travis's billing-basis requirements, the cited defaults table and the "both" options memo. No code changed.

## Remaining

- [ ] **Declaration (AC1-3):** add the per-provider basis and the per-credential entries to the built-in table, the config key and its schema, and the "not declared" state. Remove the inference from auth mode and from `provider.auth`.
- [ ] **Basis on the attempt (AC4-5):** bind the basis to every attempt so a subscription attempt without a price is still subscription work (the review's fix: a plan record with no rates). Cover the ChatGPT models with no catalogue price, non-default tiers, ChatGPT image and realtime calls, and Claude subscription work reported through the pane bridge.
- [ ] **Ledger format (AC6):** reproduce first. Validate a ledger written by a developer-accounting build at `5bae03414e^`, plus any real pre-09-21 state DB that can be found, against the new build, and record the result even if it passes. Then add an explicit format version: older payloads are read under their own rules, and a newer or unknown format turns collection off with one warning instead of failing turns. The new per-attempt basis is written under this version.
- [ ] **Best effort (AC7):** on `execute` and `resolve_path` for compaction, web search and image generation, an admission or observation failure logs the reason class (never the endpoint), marks the attempt unrecorded (a gap record when the store is writable, plus the existing gap warning) and returns the provider response.
- [ ] **Uncollected sessions (AC8-9):** record guardian forks under the parent conversation's thread, labelled as a review. Emit `accounting.excluded` once per ephemeral session. Re-run the 09-21 coverage audit method and write a dated audit listing every remaining uncollected session kind.
- [ ] **Both (AC12):** implement the chosen option.
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
- [ ] **AC12:** the "both" behaviour chosen by Travis. To be written once decided.
- [ ] Gate (`sec-common.md`): `just test` for core, state and tui, with and without `developer-accounting`; Linux clippy; a tmux run on GLM 5.2 (`-c model_provider="zai"`); one Opus 5.5 High code review; demo videos per `qa/demos/README.md`. Every run uses `CORBANU_TEST_NO_NATIVE_KEYRING=1` and a disposable home.
- [ ] `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.

## Exit evidence

- [ ] Merge commits, binary digests and test counts linked from `qa/portfolio/agent-cost-accounting/pf-60-s05/`.
- [ ] Code-blind acceptance record and its review linked; every AC has a recorded result.
- [ ] S03's Remaining list cross-referenced for the findings closed here; Travis accepts; record archived and plan backlinks updated.
