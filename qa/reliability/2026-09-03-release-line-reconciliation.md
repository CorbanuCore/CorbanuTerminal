# 0.1.37 release-line reconciliation

- Date: 2026-09-03
- Change class: bounded fix
- Owner: Codex primary integration agent

## Product authorization

- Product-spec heading: **Shipping MVP — LIVE**.
- Runtime excerpt: “Rust, Apache-2.0, Linux/macOS/Windows, the `corbanu`
  command, and legacy `pfterminal` command and state compatibility.”
- Provider excerpt: “OpenAI, Anthropic/Claude Plan, Kimi, Z.AI, DeepSeek,
  OpenRouter, Ambient, Meta, Baseten, Vercel, Bedrock, Ollama, LM Studio,
  Corbanu Plan, and custom providers.”
- Additional preserved surfaces: the already-shipped wallet/payments and Task
  Node/identity rows under the same product-spec heading.

This reconciliation restores one coherent main line for behavior that is
already product-authorized and, in part, already released. It adds no new user
goal, authorization boundary, credential format, financial action, or data
disclosure contract.

## Divergence assessment

- Current main: `6dd9ad646beb4a7407521439411f436f21ea4af1`.
- Published release tag: `rust-v0.1.37` at
  `43ff201aabb633a8b6a1a10ea2b1544d92ad3902`.
- Merge base: `fb54216dc19f688490eeafa0f3873fe945d83ca1`.
- The release side has 35 unique commits; current main has 143 unique commits.
- `rust-v0.1.37` is not an ancestor of main. A version-only bump would hide
  missing release-line code, while installing 0.1.37 would discard newer
  provider-auth and security work.

The 0.1.37 side contains released Claude onboarding and Fable 5.1 mapping,
Task Node profile isolation, Corbanu API/wallet work, model-picker work, release
evidence, and the package version bump. Current main contains the later
PF-48–PF-57 unified-provider work, credential-store liveness repairs, security
work, follow-up TUI fixes, and human-test documentation.

## Reconciliation contract

- Merge the complete `rust-v0.1.37` history; do not cherry-pick only its version
  or reconstruct released commits.
- Resolve overlaps semantically: retain current-main provider catalog,
  authentication, vault/keyring, model-selection, and governance contracts while
  preserving release-only Task Node, wallet/API, Fable, and compatibility work.
- Preserve both parents in the final history and verify their ancestry.
- Use `0.1.38` for the combined development candidate so it never advertises
  itself as the older 0.1.35 build or invites a downgrade to 0.1.37.
- Do not create a tag, publish a release, or claim human/release acceptance in
  this change.

## Conflict dispositions

| Surface | Resolution |
| --- | --- |
| Version identity | The combined candidate is `0.1.38`; it must not advertise itself as the installed `0.1.35` build or offer the older `0.1.37` release as an upgrade. |
| Provider lifecycle | Retain PF-48–PF-57's shared catalog, status, onboarding, management, liveness, and vault contracts from main. |
| Corbanu inference product | Retain the release line's later Corbanu API decision: arbitrary dollar balance and wallet-owned API keys; do not restore new Plan sales, recovery, tier, receipt, or entitlement UI. |
| Deferred onboarding | Keep unified onboarding's queue/cancel/fallback behavior, but hand the queued Corbanu choice to wallet creation/unlock and the Corbanu API balance/key view. |
| Provider selection | Ordinary API-key creation selects the recommended Corbanu model. Deferred setup preserves an existing usable provider; without a fallback it selects Corbanu only after the generated key is successfully stored. |
| Compatibility identifiers | Keep persisted/internal `corbanu-plan` identifiers and legacy event names where required for state compatibility; user-visible copy says **Corbanu API**. |
| Fable and streaming | Retain the release's Fable 5.1 model mapping and main's Anthropic/Vercel streaming recovery and cache behavior. |
| Task Node and platform work | Preserve the release-only Task Node profile-isolation history and main's later security/platform history without reconstructing either side. |

## Evidence ledger

- [x] Merge commit `aeaabfeaae1c2595a80c7118926e855de4ad6537`
  retains both exact histories. Its exact main-line parent is
  `64b12a2e42a73ed17fa9186cf590380d77e27ad6` (including main through
  `6dd9ad646beb4a7407521439411f436f21ea4af1`), and ancestry checks pass
  for that parent and release
  `43ff201aabb633a8b6a1a10ea2b1544d92ad3902`.
- [x] Conflict-by-conflict dispositions are recorded above.
- [x] `just fix`, `just fmt`, `git diff --check`, plan/sprint checkers, and the
  portable-skill mirror checker pass. Strict MkDocs reaches all pages but
  remains blocked by 19 pre-existing broken/excluded-link warnings outside this
  change; no changed documentation file introduces one of those warnings.
- [x] Focused final-tree suites pass: provider-auth 64/64, Corbanu API UI 10/10,
  provider credentials 12/12, wallet unlock 7/7, onboarding 72/72, wallet menu
  25/25, and provider manager 8/8.
- [x] The focused provider/streaming core run passed 31/31 (nextest run
  `75e64c46-ee5b-4ff2-920f-c0b16d9cfd5e`); the later edits are confined to the
  TUI reconciliation and documentation.
- [x] The complete affected multi-provider true-TMUX module passes 11/11 after
  formatting, including fresh/locked wallet, fallback/no-fallback, cancel and
  ordinary re-entry, one-time key create/store/reveal/activate without restart,
  restart, and request paths. The exact PF-52 Claude recovery journey separately
  passes 1/1.
- [ ] Release build produces Corbanu 0.1.38 plus a matching code-mode host.
- [x] Independent Fable 5.1 Max review through TMUX + Corbanu Terminal
  converged to `FINAL_REVIEW: CLEAN`; its findings and dispositions are in
  `qa/release/0.1.38/autoreview-fable.md`.
- [ ] Exact candidate commit and binary digests are recorded and pushed.
- [ ] Named-human acceptance of the installed candidate is recorded.

## Release status

This record authorizes integration and qualification only. Publication remains
blocked on the repository's normal named-human, live-repository, applicable
platform, benchmark, release-ledger, and release-owner decisions.
