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

## Evidence ledger

- [ ] Merge completed with both exact parents retained.
- [ ] Conflict-by-conflict disposition recorded.
- [ ] Formatting and governance checks pass.
- [ ] Affected provider, model, Task Node, wallet/API, and update/version tests
  pass on the combined tree.
- [ ] Release build produces Corbanu 0.1.38 plus a matching code-mode host.
- [ ] True-TUI smoke covers startup, provider status, Fable selection, and one
  request/cancel/recovery path on the combined candidate.
- [ ] Independent review has no unresolved actionable finding.
- [ ] Exact candidate, binary digests, remaining gates, and main push recorded.

## Release status

This record authorizes integration and qualification only. Publication remains
blocked on the repository's normal named-human, live-repository, applicable
platform, benchmark, release-ledger, and release-owner decisions.
