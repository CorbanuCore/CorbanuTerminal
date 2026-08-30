# Corbanu tmux / Claude Opus 5 Max review — 2026-08-30

## Runtime attestation

- Harness: tmux session `pf-browser-opus5-main-20260830`, Corbanu Terminal
  `0.1.35`, built from rebased commit
  `1a5562738cb3d53bd4d0b6668761cfe76bd4b93e`.
- Review conversation: `01a051a8-4a83-7383-b383-b9724878e674`.
- Requested provider/model/effort: `Claude Plan` /
  `claude-opus-5-plan` / `max`.
- Provider-reported model: `claude-opus-5`.
- Terminal/safety trace: `tmux/3.7c`, `approval_policy=never`,
  `sandbox_policy=read-only`.
- Prompt SHA-256:
  `126df8815ecbe0c0fa8839fb23a987b581f191f993b5732fbca9deee447922e8`.
- Local trace directory:
  `.codex-work/p0-security-browser-retrieval/tmux-opus5-main-20260830/logs-final/`.
- Local rollout:
  `~/.corbanu/sessions/2026/08/30/rollout-2026-08-30T00-52-58-01a051a8-4a83-7383-b383-b9724878e674.jsonl`.

The rendered response cited the immediately preceding `01a051a7...` rollout
while checking its model metadata. The evaluation itself ran in the conversation
above; Corbanu's trace independently records every turn as
`model=claude-opus-5-plan`, `codex.turn.reasoning_effort=max`, with Anthropic
`message_start.model=claude-opus-5`. This correction prevents the response's
rollout-selection mistake from becoming a false provenance claim.

## Initial verdict and disposition

The combined PF-31-S04/PF-33-S03 review ended `CHANGES REQUIRED` after 13
minutes. PF-31 finding `P2-1` was accepted: the retriever manifest and
`browser-isolation` runtime source contained the same immutable Scrapling digest
without a drift assertion, and PF-31 evidence did not disclose the runtime pin or
automatic pull path.

Remediation:

- `scripts/security-retriever-artifact-check` now extracts the canonical
  `BASE_IMAGE` and worker Dockerfile `FROM` literals and requires both to equal
  the manifest reference.
- Ten policy mutations cover manifest drift plus missing/duplicate source pins.
- This README now records all three pins and the runtime pull/build behavior.

The other actionable findings belong to reopened PF-33-S03 and are not claimed
closed by this PF-31 remediation record. The initial verdict remains immutable;
the corrected-candidate follow-up is recorded below after it runs.

## First corrected-candidate follow-up

Prompt SHA-256:
`2091a2babaf6dc3bf743498520338d74f0097a5533c6c0cf6a0deb59c691d6b8`.
The same tmux/Corbanu Terminal/Opus 5 Max conversation returned
`CHANGES REQUIRED`. It found that `worker/Dockerfile` holds the decisive third
pin, that the new runtime-source read bypassed the validator's canonical-path
policy, that only the manifest-side mismatch had a negative test, and that the
reopened sprint's `updated` date was stale. All four findings were accepted.

The second correction binds the manifest, `BASE_IMAGE`, and Dockerfile `FROM`
references; routes both source reads through canonical containment checks; adds
missing/duplicate negative cases for both source literals; and corrects the
sprint date. This follow-up verdict remains immutable.

## Second corrected-candidate follow-up

Prompt SHA-256:
`80f7d736d10f166c6f0e1b748fd883886d559a01a4b6d46737a26209eb6be270`.
The same trace-backed runtime returned `CHANGES REQUIRED`. It independently
confirmed that all prior PF-31 findings were closed, then found two P3 completion
defects: no shipped mutation reached the decisive worker-`FROM` equality arm,
and the sprint Remaining ledger still described the superseded one-pin,
six-mutation design. Both were accepted.

The final correction adds a recipe-only digest-drift mutation, bringing the
repaired policy count to eleven, and updates the sprint ledger to describe the
two runtime literals and canonical source resolution. A final follow-up verdict
is recorded below after it runs.

## Final corrected-candidate follow-up

Prompt SHA-256:
`f0527081e632d0ed3fc7f52cee28eefb0e7826e072d5895fd1f445ff4d820c80`.
The same tmux/Corbanu Terminal/Claude Opus 5 Max conversation independently
reproduced the 11 mutation, 27 fixture, 27 replay, governance and no-activation
properties. It reported no findings, marked the initial P2-1 and every follow-up
finding fully closed, and ended with the standalone verdict `CLEAN` after 2
minutes 5 seconds.

The integration owner separately ran the complete validator on the writable
Mac worktree, including all ten temporary-path/symlink checks, plus Ruff,
byte-compilation, JSON parsing, both governance checkers and `git diff --check`.
The bounded remediation commit is
`c2168575695dfb2ad015bf45ef24d9e4b173b571`.
