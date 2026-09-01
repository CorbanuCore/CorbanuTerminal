# Corbanu Terminal 0.1.37 release candidate

Date: 2026-09-01 UTC

Branch: `release/0.1.37`

Release target: `rust-v0.1.37`

Release authorization: explicitly authorized by a human with release authority
on 2026-09-01. Under Corbanu development policy 1.5, the micro release must be
pushed and incomplete qualification evidence must be disclosed rather than used
as an agent-created veto.

## Included scope

- Kimi K3 early image/stream crash recovery already present in the 0.1.36 base
  through `0c3129f266` (`fix(core): recover early response text deltas`).
- Correct model-header display synchronization already present in the 0.1.36
  base through `f737c43e26` (`fix(tui): keep session model header in sync`).
- Persistent Claude subscription credential onboarding through the scoped
  cherry-pick recorded as `f9bc22fedc`.
- Task Node per-Corbanu-profile session isolation and its evidence through
  `c17d59a6c3`, `3431399e97`, and `c428cb1021`.
- Claude Fable 5.1 Plan availability, exact upstream request mapping, picker
  visibility, and generalized regressions for both supported Fable versions.

Fable 5.1 is exposed only through the persisted Claude Plan credential as
`claude-fable-5-1-plan`, mapped to upstream `claude-fable-5-1`. This release does
not add a metered Anthropic catalog row or invent direct-API pricing.

## Explicit exclusions

- GLM 5.3 work from the unrelated dirty session-persistence worktree.
- Unrelated post-0.1.36 security implementation or provider changes.
- Default-crew changes, removal of Fable 5, or direct Anthropic pricing changes.

## Recorded qualification evidence

- `just fmt`: passed before version finalization and on the final candidate tree.
- Model-provider-info suite: 61 passed; nextest run
  `0cdc7607-7ba1-4f5d-ab23-78364f1e6755`.
- Models-manager suite: 62 passed; nextest run
  `503b58e7-52da-40d9-b9d8-76b86c480f71`.
- Model-provider suite: 73 passed; nextest run
  `96587ff7-0a0a-424c-a822-1ac13d0265a9`.
- Exact Fable request mapping: 1 passed; nextest run
  `5051c924-4912-43f2-a4de-43eccf34b90f`.
- Focused TUI picker and reviewed snapshot: 1 passed; nextest run
  `fc643a91-cea3-4fad-b2cc-f1ff4a56ea85`.
- Claude vault suite: 55 passed; nextest run
  `5b3f0a41-fbbf-4b6f-af47-34c06cbd059d` (two tests passed on their configured
  retry after first-attempt timeouts).
- Task Node session suite: 13 passed; nextest run
  `5c9124d5-b248-4be5-abd3-340e6f295cb3`.
- JSON parse and `git diff --check`: passed on the final candidate tree.
- The first focused TUI picker attempt compiled the changed tree but the LLVM
  linker terminated with a bus error while the filesystem had only 1.2 GB free.
  This was an environmental failure, not a test assertion failure. The release
  worktree build cache was cleaned, restoring more than 30 GB; the retry reached
  the expected new-snapshot review, and the post-review rerun passed.

## Disclosed incomplete or failing gates

- No new production true-TUI or named-human acceptance artifact has been
  recorded for Fable 5.1.
- `docs/plans/check.py` reports three active plans against the configured limit
  of two because the still-active Corbanu API plan is retained honestly.
- `docs/sprints/check.py` reports pre-existing duplicate sprint identifiers
  plus allocation and dependency debt in the still-open Corbanu API sprint.
  PF-55 is completed and archived with exact worktree, branch, base, dependency,
  and execution metadata.

These disclosures do not override the explicit human instruction to push this
release. The multi-platform release workflow remains responsible for building,
smoke-testing, and attaching the actual release assets before publication.
