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

That first fix was later refined by the Opus/Max review: availability/API states
remain skippable, but architecture and artifact-integrity failures are terminal,
and auto mode scans later engines for integrity failures before preferring a
ready result over a remediable action.

## Corrected-candidate reviews

Codex GPT-5.5 Autoreview of `cd7839ee98fbf8f7efb853cbf55566c079bafb79`
reported two accepted `P2` findings:

1. A stopped preferred engine that retained a PF-31 worker could be skipped and
   lead to creation of a second worker on another engine.
2. A symlinked fixture child could escape the repository-scoped fixture
   directory.

Both were fixed in `8162a8cfb2ccc31021c3fa3492c5e7db33674415` with a
new unavailable-engine ownership fixture and per-fixture scoped resolution. The
same model/effort rerun returned clean: no accepted/actionable findings, patch
correct, confidence 0.83.

After Opus found further precedence, first-run lock, multi-engine inventory and
ancestor-symlink cases, final code commit
`9ab30bc79ccdaf65a65f03fcf85c0464016a6d67` added the corresponding fixtures
and containment rules. Codex GPT-5.5 Autoreview with high reasoning returned
clean: no accepted/actionable findings, patch correct, confidence 0.84.

The first final-follow-up candidate `ef11085bd723918526ce42651e066715ab7d9bea`
still allowed auto mode to recommend creating a preferred-engine worker when a
later engine owned a stopped worker. The accepted `P2` finding was fixed by
prioritizing remediation of the existing owned worker after the complete
integrity scan; fixture 27 covers it. The next review found that the new real
symlink checks wrote under the checkout, breaking read-only source mounts. That
accepted `P2` was fixed by extracting a shared canonical resolver and creating
the temporary leaf/ancestor symlinks only under the OS temp directory.

Codex GPT-5.5 Autoreview of final code commit
`cf05164e7d5c21a8e716a36c70455c5cff26f5ca` returned clean: no
accepted/actionable findings, patch correct, confidence 0.84. Final macOS,
read-only-checkout, Linux and Windows results: one manifest valid, 27/27
fixtures, 27/27 deterministic replays, five manifest-policy mutation checks and
ten path checks passed. These repeats prove deterministic pure evaluation, not
live engine idempotence or TOCTOU safety.
