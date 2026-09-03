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
- Strict MkDocs is still blocked by 19 pre-existing excluded/missing-link
  warnings outside this candidate's changed pages; the non-strict site build
  completes and changed pages introduce no new warning.
- Final release build, binary digests, installed-app smoke, and human acceptance
  remain pending.

## Disclosed incomplete gates

- No named-human acceptance has yet been recorded for the combined candidate.
- The benchmark bootstrap cycle is due and has not been run.
- Cross-platform and live-repository release qualification remain pending.

These incomplete gates block publication absent explicit release authority;
they do not block development integration or local human acceptance testing.
