# RETURN — acct-coverage-122

Status: BLOCKED by literal writable scope; no implementation candidate produced.

## Dispatch verification

- Action: `acct-coverage-122`; frozen assignment/brief allocation: `acct-coverage-121` (retained as supplied).
- Allocation digest: `0b23b35fd38533d88a36408d5c52fc943fd71ea207cfe4f07714a2ad2a10b96d`.
- Claim: `9d38c8dc-1234-413c-9f3a-bfc7f81dbf78`.
- Runtime: OpenAI `gpt-6-astra`, high.
- Brief SHA-256 verified with `shasum -a 256`: `970f55850ce6a464401ab2f7ee5c32f03948062e5eef70382797ecee410cd81a`.
- Checkout HEAD equals frozen base: `3d6ea8f7f822aa6e401ad13ddf87392db6b60de5`.
- Initial working tree clean. No live profile, ledger, credentials, coordinator or dashboard state accessed. No push.
- Classified as an authorized product-initiative extension of active PF-60 / in-progress PF-60-S03. Existing plan citation: product-spec heading **Product measurement**, excerpt “No commercial performance numbers have been supplied.” The frozen brief supplies the explicit coverage instruction; this report does not amend plan/sprint authority.

## Blocking scope correction

`codex-rs/core/src/session/turn.rs` is outside the supplied writable paths. It independently gates actual collector creation on provider ID:

- Lines 1638–1643: Anthropic mode AND `model_provider_id == "anthropic"` AND Anthropic wire.
- Lines 1669–1674: Responses mode AND `model_provider_id == "openai"` AND Responses wire.
- Lines 1686–1690: Chat mode AND `model_provider_id == "openai"` AND Chat wire.

Therefore widening `developer_accounting_mode` and `eligible` in allocated files cannot make `claude-plan`, `openrouter`, `corbanu`, or custom-provider sessions collect. Changing provider IDs to bypass these conditions would corrupt provider selection/attribution and is not an acceptable workaround. The manager must add `codex-rs/core/src/session/turn.rs` to the literal implementation scope for a complete revision.

The dispatch binding also needs actual selected-provider identity and billing/auth provenance. `AccountingMode` currently holds only a UUID scope and endpoint; `Sampling::start_request` hard-codes `anthropic` or `openai`. Endpoint equality alone cannot distinguish two providers sharing a gateway. Do not introduce a process-global identity registry or encode identity inside an endpoint merely to avoid extending the proper call site.

Inspect `codex-rs/core/src/client.rs` for a bounded additional allocation if resolved auth/request-shape checks need caller plumbing: Anthropic evidence is currently constructed at 2515 before its typed request is built at 2557; Chat/Responses resolve at 3056/3216/3470. This report establishes `session/turn.rs` as necessary; it does not claim every possible implementation must edit `client.rs`.

## Three gates: rationale and preserved properties

1. **Provider-ID selector.** `core/src/accounting.rs:60–76` enables only the original direct provider/dialect pairs. `core/src/accounting_tests.rs:91` explicitly excludes other IDs. `core/src/accounting.rs:202–249` assigns fixed provider names and wire-specific dialects; `accounting_prices.rs` likewise projects only the original provider authority. The original Responses allocation (`docs/research/agent-cost-accounting/responses-dispatch-allocation.md:97`) explicitly separates modes and unsupported routes. The ID allowlist is now superseded by the frozen coverage instruction, but exact provider attribution and correct wire interpretation must survive.
2. **API-key/direct-route eligibility.** `accounting_responses.rs::eligible` requires Responses plus typed API-key auth and rejects auth overrides. `accounting_chat.rs::eligible` additionally requires OpenAI identity/auth and excludes plan/gateway routing shapes. `accounting_responses_tests.rs:100` and `accounting_chat_tests.rs:110` enforce those original limits. The Responses allocation at lines 99–105 explains that overrides could replace API-key economics and that the full final URL must match after auth. Preserve exact endpoint binding, rejection of redirects, AWS/unknown-route exclusions, request routing-override exclusions, response-local retry identity, and failure after an initialized logical request changes binding. Authentication method alone is not a reason to lose known-dialect token usage.
3. **Price authority.** `accounting_prices.rs::project` requires Metered billing; `openai_project` also accepts the API-key side of AuthDependent but returns no snapshots for Plan, PlanSchedule and Local. `accounting_prices_tests.rs:263` and its Chat/Responses tests exercise non-metered/unknown authority. The Responses allocation at lines 136–140 requires exact provider/model authority and no price for unknown economics. Preserve no invented rates or plan-to-cash conversion. AuthDependent API-key rates must not be reused for ChatGPT sessions.

No restrictions were edited or deleted in this dispatch.

## Activation matrix and exclusions

No new activation matrix was implemented or executed. Current actual collector creation, before downstream checks:

| Selected provider ID | Anthropic wire | Responses wire | Chat wire |
| --- | --- | --- | --- |
| anthropic | Selector/call-site admitted | Disabled | Disabled |
| openai | Disabled | Selector/call-site admitted | Selector/call-site admitted |
| every other ID | Disabled | Disabled | Disabled |

Responses/Chat then require API-key auth and their additional `eligible` conditions. Anthropic has no equivalent `eligible` function in these modules. This table is static code evidence, not an executed test matrix.

Required revised matrix: every provider ID across Anthropic, Responses and Chat, crossed with API-key, ChatGPT/subscription, provider-managed plan/header/command authentication, and no-auth where the transport supports it. Eligibility must be based on the attributable route, not the provider's display name. Concrete exclusions to name and prove during implementation:

- **AWS-signed route:** signing may change destination/protocol outside the collected direct wire contract.
- **chat_completions_provider override:** downstream provider selection is not bound by the selected outer endpoint.
- **request provider/provider_options/plugins:** request-side routing or transformation prevents claiming the selected direct provider/dialect.
- **unapproved or unpinnable endpoint:** final request destination cannot be proven equal to the approved binding.
- **unsupported wire dialect:** no matching usage decoder exists (all three current WireApi variants have collectors).
- **changed in-flight binding / incompatible reused WebSocket:** fail the logical request visibly rather than attach old evidence to a new provider or continue a partial retry chain.

The last item is a failure condition, not permission to silently stop collecting. Existing agent-identity exclusions in `client.rs` and WebSocket provenance also need explicit assessment in the revised matrix; they were not silently removed or declared justified here.

## Plan tokens and unavailable money

No new plan token rows were produced. The existing state model can persist an admitted attempt and numeric observations with an empty original-price snapshot set (`state/src/runtime/accounting_native.rs:10–40`). Positive observed buckets then yield `BucketQuote::MissingRate`, distinct from `MissingUsage`; `all_buckets_priced` is absent and inspection renders an unavailable full estimate (`accounting_pricing.rs:198–211,260–278`).

Two hazards must be covered by the eventual revision:

- `model-provider-info/src/lib.rs:1003–1022` resolves ChatGPT-family default auth endpoints to `CHATGPT_CODEX_BASE_URL`, while the current developer selector defaults Responses/Chat to `https://api.openai.com/v1`. Merely removing auth eligibility would trigger a fatal endpoint mismatch.
- `accounting_pricing.rs:261` treats a reported zero bucket as priced zero without a rate. If every bucket is reported/derived zero, the full estimate may be zero with no snapshot. The requested rule that plan money remain explicitly unavailable must be tested for zero usage too; positive-token examples alone are insufficient.

Provider switching also cannot rely only on endpoint equality: `config/mod.rs:2065–2070` deliberately preserves the original accounting mode across configuration rebuilds. Existing test `accounting_developer_loader_override_survives_refresh` pins that behavior. The revision must preserve explicit internal OFF/bindings while ensuring developer routes cannot reuse another provider's attribution or API-key price branch.

## Inspector language inspected

Static inspection of `tui/src/chatwidget/tokens.rs`:

- 347: `No recorded attempts in this day; collection coverage unknown.` — empty population.
- 351–365: known estimated subtotal plus `Full recorded estimate: unavailable (...)` when estimates are incomplete.
- 403–407: attempt page calls the same estimate helper using `all_buckets_priced`.
- 448–449: `unknown — no retained numeric evidence` versus `unknown — rate unavailable`.
- 489: `Price: unavailable — no dispatch-time price snapshot`.
- 515, 738, 778: billed cost explicitly unavailable without settlement evidence.
- 546: root/day overview uses the estimate helper.
- 650: root-own, descendants and provider/model groups use the estimate helper.
- 697: unknown-ancestry breakdown uses the estimate helper.
- 841–845: complete range totals use the estimate helper; bucket pages route through inspection pages.

These lines already distinguish a nonempty, incompletely priced population from empty. No isolated wording change is justified by that distinction. The all-zero/no-snapshot case above is a pricing-state issue that can cause line 351 to display a complete zero estimate for a plan; changing that label alone would not repair the state. No inspector line was changed and no new TUI qualification is claimed.

## Never-ships guard

All guard files and lines remain untouched:

- `core/Cargo.toml:16–17`: `# Developer qualification only. Never enable in distributed builds.` / `developer-accounting = []` (non-default).
- `core/src/accounting.rs:5–6`: `#[cfg(all(feature = "developer-accounting", not(debug_assertions)))]` / `compile_error!("developer-accounting is debug-only and must not be enabled in distribution builds");`.
- `core/src/accounting.rs:50`: `std::hint::black_box(b"CORBANU_DEVELOPER_ACCOUNTING_NOT_FOR_DISTRIBUTION");`.
- `scripts/build_codex_package.py:29–32`: scans for that marker and raises `SystemExit` with `Refusing distribution package: developer-accounting enabled in {path}`.

## Validation and changes

Read `docs/development/test-isolation.md`. Rust tests were not launched because implementation is blocked and no revised candidate exists. Exact counts: 0 executed, 0 passed, 0 failed; failure names: none (not a pass). Requested accounting, usage and developer-accounting gates remain unrun. No formatter, raw cargo test/nextest, live profile or native credential access was used.

Only this evidence file was added. Source changes: 0 files, +0/-0 lines. No functional handoff, independent acceptance, release readiness, or approval is asserted. Resume after the manager corrects the literal scope; preserve this blocked attempt.
