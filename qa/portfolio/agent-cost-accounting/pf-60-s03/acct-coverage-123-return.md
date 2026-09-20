# RETURN — acct-coverage-123

Status: implementation in progress; verification pending.

Frozen brief SHA-256 verified: `5f090bc4a3a2a81b3ad32c1c344ae23b0bb794a2335bdf993828ac3e476654db`.
Base: `6322a6e7cc4e317b1d57ee8618ec860a29078c40`.
Allocation digest: `298d6f2dc588948230c0e411bf65fb6a081f97b5aa527f9c687be6eeb568b27f`.
Worker: OpenAI gpt-6-astra, high. No delegation, push, live profile or live ledger access.

## Authority and qualification

Product initiative, PF-60, active plan `docs/plans/active/portfolio-agent-cost-accounting.md`, sprint PF-60-S03 (in_progress). Product-spec heading **Product measurement**, excerpt “No commercial performance numbers have been supplied.” The frozen assignment conveys Travis's additional all-provider collection requirement and supplies this worktree, exact base and expanded literal scope. Historical plan/sprint coordinates predate this dispatch; no governance records outside the allocated scope were changed.

This is a developer-only implementation return to the manager. Independent code-blind design/execution/evidence review, final packaged true-TUI proof, live-repository qualification and human acceptance are not claimed. The functional gate remains with PF-60-S03; this return does not qualify a human-test candidate or authorize release.

## Gates and identity

- Developer selector: every provider ID for Responses, Chat or Anthropic, excluding AWS signing and a configured chat_completions_provider override.
- Route eligibility: the same dialect/configuration policy admits API keys, ChatGPT subscription authentication, no CodexAuth (provider-managed/local authentication), configured bearer and command authentication. Chat request routing fields are additionally refused.
- Turn collector: the provider-aware mode is rebound from the actual turn provider ID and its resolved API base endpoint. Exactly one dialect's slot is attached: Anthropic immediate sampling, Responses deferred sampling (HTTP/WebSocket), or Chat deferred sampling. Existing explicit native embedding modes retain their prior restrictions.

Sampling copies the provider ID from the mode, never from its arithmetic dialect. Turn rebinding handles provider switches. Physical HTTP send URLs must exactly match the pinned endpoint; WebSocket provenance must match the resolved endpoint and any reused connection. Request routing fields are checked again before HTTP admission. No request body or credentials are persisted by the new check.

## Admission matrix

The executable matrix covers six IDs (anthropic, openai, claude-plan, corbanu, openrouter, custom), three dialects, five authentication setups (none, API key, ChatGPT subscription, configured bearer, configured command) and three configuration shapes (ordinary, AWS, chat_completions_provider): 270 cells.

| Shape | All IDs / supported dialects / five auth setups | Reason |
| --- | --- | --- |
| Ordinary | Collect | Supported wire usage; selected endpoint is pinned |
| AWS signing | Refuse | Signing does not establish the supported endpoint/dialect |
| chat_completions_provider | Refuse | Override changes serving-provider attribution |
| Unknown wire dialect | Refuse during configuration parsing | No usage dialect adapter |
| Request provider | Refuse | Request can select a different serving provider |
| Request provider_options | Refuse | Request can change serving-provider attribution |
| Request plugins | Refuse | Plugin routing cannot be attributed to the selected provider |
| Endpoint mismatch / unsafe endpoint | Fail closed | Request destination is not the pinned destination |
| Uninspectable HTTP request body | Fail closed | Routing keys cannot be checked |

The client already excludes agent-identity telemetry from Chat/Responses accounting; that existing exclusion is outside the five matrix authentication setups and is not claimed qualified here. Startup prewarm and auxiliary inference remain outside sampling collection.

## Money

Provider-aware samples default to no price snapshot. Only the existing native provider ID, native endpoint and API-key authority can enable bundled API estimates. Plan/custom routes and ChatGPT authentication do not use API-key prices. Deferred collectors re-evaluate the already-resolved authentication at request time and reject a pricing-authority change after initialization.

No snapshot now means MissingRate even for zero-token buckets, so all_buckets_priced cannot become a complete zero estimate from absent pricing. The inspector explicitly labels a wholly unpriced attempt's token cost unavailable. A recorded attempt with unavailable money remains distinct from an absent ledger.

No API-equivalent counterfactual was added.

Compatibility limitation: immutable stored-quote validation recomputes the quote and compares its exact serialized payload. Pre-existing unpriced quotes containing zero buckets calculated under the earlier rule can therefore fail validation under the corrected rule. This developer-only change does not rewrite old ledger evidence or provide a historical migration; no live ledger was accessed.

## Never-ships guards (unchanged quotations)

1. Non-default core Cargo feature: `developer-accounting = []`, under `[features]`; there is no default feature activation.
2. `#[cfg(all(feature = "developer-accounting", not(debug_assertions)))]` and `compile_error!("developer-accounting is debug-only and must not be enabled in distribution builds");`
3. `std::hint::black_box(b"CORBANU_DEVELOPER_ACCOUNTING_NOT_FOR_DISTRIBUTION");`
4. Package byte-marker check raises `f"Refusing distribution package: developer-accounting enabled in {path}"`.

## Changed source files

Source diff SHA-256: `720f8b4201f744b7cc21a2dbffac15c15c4fede6d4b819701c2509dd331b0c9a`.

20 Rust files, 412 production lines changed and 448 test lines changed (860 total additions plus deletions). The total exceeds the 800-line guidance because the frozen allocation requires all selection/transport/turn gates, identity and money semantics to move together, with the requested cross-crate matrix and integration proof. Production changes remain below 500 lines.

| File | Added | Removed |
| --- | ---: | ---: |
| `codex-rs/core/src/accounting.rs` | 130 | 17 |
| `codex-rs/core/src/accounting_chat.rs` | 46 | 4 |
| `codex-rs/core/src/accounting_chat_tests.rs` | 9 | 5 |
| `codex-rs/core/src/accounting_policy_tests.rs` | 152 | 0 |
| `codex-rs/core/src/accounting_responses.rs` | 46 | 3 |
| `codex-rs/core/src/accounting_responses_tests.rs` | 4 | 4 |
| `codex-rs/core/src/accounting_tests.rs` | 22 | 24 |
| `codex-rs/core/src/accounting_transport.rs` | 31 | 9 |
| `codex-rs/core/src/accounting_websocket.rs` | 11 | 3 |
| `codex-rs/core/src/accounting_websocket_tests.rs` | 40 | 0 |
| `codex-rs/core/src/agent/role.rs` | 1 | 0 |
| `codex-rs/core/src/config/mod.rs` | 12 | 8 |
| `codex-rs/core/src/session/turn.rs` | 39 | 31 |
| `codex-rs/core/tests/suite/accounting_anthropic.rs` | 93 | 0 |
| `codex-rs/core/tests/suite/accounting_chat.rs` | 32 | 0 |
| `codex-rs/core/tests/suite/accounting_responses.rs` | 35 | 0 |
| `codex-rs/state/src/runtime/accounting_pricing.rs` | 1 | 1 |
| `codex-rs/state/src/runtime/accounting_pricing_tests.rs` | 1 | 5 |
| `codex-rs/tui/src/chatwidget/tokens.rs` | 14 | 5 |
| `codex-rs/tui/src/chatwidget/tokens_tests.rs` | 22 | 0 |

## Verification

Pending. Ordinary Rust verification uses only this checkout's guarded just test. Raw attempts are preserved under the same evidence directory. No tests are represented as passed before execution.
