# Corbanu Terminal 0.1.38 integration candidate

Date: 2026-09-03 UTC

Branch: `integration/reconcile-release-0.1.37`

Release target: untagged `0.1.38` development candidate

Release authorization: none. This record covers line reconciliation and
qualification only; it does not authorize a tag or publication.

## Included scope

- Full history from published `rust-v0.1.37`, including Task Node profile
  isolation, Corbanu API wallet balance, wallet-funded models, and Fable 5.1.
- Current `main` through `6dd9ad646beb4a7407521439411f436f21ea4af1`,
  including PF-48–PF-57 unified provider authentication, immediate credential
  activation, atomic vault storage, and subsequent security integration.
- Semantic conflict resolution that retains the unified provider/vault
  lifecycle, the arbitrary-balance wallet surface, both Fable 5/5.1 routes,
  and the Corbanu API Kimi Messages route.

## Qualification evidence

- Merge commit `aeaabfeaae1c2595a80c7118926e855de4ad6537` preserves both
  exact histories. Its main-line parent is
  `64b12a2e42a73ed17fa9186cf590380d77e27ad6` (including main through
  `6dd9ad646beb4a7407521439411f436f21ea4af1`), and
  `rust-v0.1.37` (`43ff201aabb633a8b6a1a10ea2b1544d92ad3902`) are both
  ancestors of the candidate.
- Semantic reconciliation routes unified onboarding's deferred Corbanu choice
  into the release line's wallet-funded Corbanu API balance/key flow. New Plan
  sales and legacy entitlement UI remain retired; persisted compatibility IDs
  remain stable.
- `just fix`, `just fmt`, `git diff --check`, plan/sprint governance checks,
  and the portable-skills mirror check pass.
- Focused final-tree suites: provider-auth 64/64; Corbanu API UI 10/10;
  provider credentials 12/12; wallet unlock 7/7; onboarding 72/72; wallet menu
  25/25; provider manager 8/8.
- Focused provider/stream core: 31/31, nextest run
  `75e64c46-ee5b-4ff2-920f-c0b16d9cfd5e`.
- True-TMUX: multi-provider onboarding 11/11 on the formatted final source,
  including deferred-passcode cancellation/re-entry and the one-time API-key
  create/store/reveal/activate path without restart; plus the exact PF-52
  Claude recovery journey 1/1.
- Independent TMUX + Corbanu Terminal + Claude Fable 5.1 Max review converged
  to `FINAL_REVIEW: CLEAN`; findings and fixes are recorded in
  `autoreview-fable.md`.
- Native macOS release build: pass at `3f64f36c6383005a3c71fad9fc032de8698a7b85`;
  `corbanu --version` reports `0.1.38`. The successful resumed build took
  318.93 seconds after the first build exposed and stopped on the corrected
  macOS-only compile defect.
- Fresh detached-worktree Linux release build on the remote builder: pass at
  the same exact commit in 377.26 seconds; `corbanu --version` reports
  `0.1.38`. The clean detached source path required new Cargo fingerprints, so
  this is release evidence rather than a warm incremental benchmark.
- macOS SHA-256: `corbanu`
  `279e671e96f75751bb91df3b86f563c8b55a28fe6bde160a635258b1caf21251`;
  `corbanu-acp`
  `1aa66623c5c465b25c86f8a2402752854f407d3dec347e720d3197d1c4d6d73d`;
  `corbanu-walletd`
  `5874fbc9cea2b03a12615a403873b82769204f9cf3cc09fc7617226e7152923d`;
  `pfterminal-walletd`
  `ef09ab6bd46c42b7d79723c978845151191b90a59021749849ce7f4abcce63ce`;
  `codex-code-mode-host`
  `86da26851055ec156a51d74ddc2560326a661034a4bdd568a44f2f90c820ab68`.
- Linux SHA-256: `corbanu`
  `241d9fa818d32f40b347f67c2885ac5d8f6dfbedd87f19a4ae10cdea3aa31df7`;
  `corbanu-acp`
  `2ee62c018ac244b123a3c3d8cbff7c6486a5e0b1a13cda3784d275d34b7b4a91`;
  `corbanu-walletd`
  `17225e0aa90fbd8f25d824aef01f9d4c0f9c9021b7c5079d92f98d130e9a15a2`;
  `pfterminal-walletd`
  `3596b6b67cbbb90935dadce0a42c72979c4304c10c4c4b66219f5f0e261f53e4`;
  `codex-code-mode-host`
  `a3642e3dbf416d1229c34e1330c3f3804c1ea4ccbdc8ed956d4f8bb0593e5365`.
- Installed-app smoke: pass. `/Applications/Corbanu Terminal Launcher.app`
  resolved the stable `bin/corbanu` and `bin/codex-code-mode-host` links to
  the freshly built release artifacts, started the `0.1.38` binary, and placed
  its window through the launcher. The window was then brought to the
  foreground for human acceptance.
- Strict MkDocs is still blocked by 19 pre-existing excluded/missing-link
  warnings outside this candidate's changed pages; the non-strict site build
  completes and changed pages introduce no new warning.
- Human acceptance remains pending.

## Disclosed incomplete gates

- No named-human acceptance has yet been recorded for the combined candidate.
- The benchmark bootstrap cycle is due and has not been run.
- Cross-platform and live-repository release qualification remain pending.

These incomplete gates block publication absent explicit release authority;
they do not block development integration or local human acceptance testing.
