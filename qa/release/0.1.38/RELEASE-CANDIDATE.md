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

- Pending final merge commit, formatting, focused suites, release build,
  true-TUI smoke, and independent review.

## Disclosed incomplete gates

- No named-human acceptance has yet been recorded for the combined candidate.
- The benchmark bootstrap cycle is due and has not been run.
- Cross-platform and live-repository release qualification remain pending.

These incomplete gates block publication absent explicit release authority;
they do not block development integration or local human acceptance testing.
