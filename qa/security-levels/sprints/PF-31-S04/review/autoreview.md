# Codex autoreview disposition

Review target: branch diff from dispatch base
`ea23dfa38bc4f2cbfe0aceadd6777c3e436a53d4` through implementation commit
`738307c35df0539d98b999c971057e137818e6a8`.

Engine/model: Codex / GPT-5.5. Parallel focused checks passed.

## Accepted finding

`P2` — automatic engine selection stopped at an installed Podman engine before
checking eligibility, so a stopped or otherwise unusable Podman could block a
fully eligible Docker engine. This contradicted the contract's “prefer Podman
when equally eligible” rule.

Disposition: accepted as an in-scope state-machine defect. Automatic selection
now probes candidates in preference order, immediately chooses the first ready
engine, otherwise chooses the first action-required engine, and reports a
blocked result only when no engine is usable. Explicit selection still returns
that engine's visible failure without fallback. Fixture
`14-auto-skips-ineligible-podman.json` covers the regression; the original
automatic-Podman fixture continues to prove the tie preference.

Post-fix local and Linux results: one manifest valid, 14/14 fixtures passed, and
14/14 repeated idempotency checks passed. A clean autoreview rerun is required
after the fix commit.
