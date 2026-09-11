# Main reconciliation — September 10, 2026

Routine integration/QA continuation of PF-58. Product linkage: **Shipping MVP —
LIVE**, operational credential use without placing raw values in chat. No new
product scope or release authorization is inferred.

## Verified baseline

- GitHub `origin/main`: `0d1c3240d752335cc89faeb2149c5030063e5d69`.
- Latest published release: [0.1.41](https://github.com/CorbanuCore/CorbanuTerminal/releases/tag/rust-v0.1.41), published September 8, source
  `739897fd64527898fdddc2dec1ab15be287f620a`.
- Successful release workflow: [34180024130](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/34180024130).
  Later release attempts 34542400957 and 34548784536 were cancelled/failed;
  they are not evidence of a newer published version.
- The release's Task Node profile relink/account-isolation repair was already
  present in our old integration baseline `472b8fed5`.
- Main's two subsequent commits, `6ca801add` and `0d1c3240d`, add planning,
  draft sprint and QA documentation/navigation (92 files), not application code.

## Preservation and reconciliation

All pending source, process and QA evidence was checkpointed locally as
`646d63980` on `backup/provider-reauth-pre-main-20260910`. Raw terminal captures
were preserved, including their trailing blank lines. This is a recovery
checkpoint, not an additional clean-review assertion.

The branch contains substantial unmerged security history. Rather than replay
that entire previously integrated history, a separate worktree combined the
existing integration anchor `472b8fed5` with current main, producing `acfc9f205`.
The three PF-58 follow-up/checkpoint commits were then rebased onto that updated
combined baseline. The trial tree and allocated worktree tree match exactly.
Final rebased source checkpoint: `da77f7c03827d56284d62a3dadb477aed36bb6ce`.
`origin/main` is an ancestor. No conflicts required manual resolution; the
sprint-index and MkDocs additions were inspected and retained.

The allocated branch remains `feat/provider-reauth-health`, in
`/Volumes/CorbanuDrive/Corbanu/worktrees/provider-reauth-health`.
No main merge or remote push was performed. Original sprint allocation bases
remain historical coordinates; this record supplies the updated integration base.

## Verification and handoff state

- `codex-rs` Git tree before/after: `a0716c307d48c308acc71592a45ba063980beae1`.
- `scripts/codex_package` tree before/after: `ef5bf92ec52ccc17950721da05b9ede9366210d3`.
- Code-blind checker: 13 tests pass.
- Portable skills: 25 files match. Plan checker: passes, two active plans.
- Sprint checker: the same eight inherited PF-43/PF-44/PF-45/backlink/order
  errors remain; not an all-green governance result.
- No compilation was necessary for this documentation-only baseline delta.
  Existing Mac/Linux binaries retain their original build provenance; no fresh
  build or new functional execution is claimed by source-tree equivalence.
- The requested fresh-context test-design and independent evidence-check passes
  have **not run**. PF-58 already consumed five reviews; an explicit two-pass
  budget amendment was requested. No further review is dispatched until resolved.
- Human testing is **not ready for a new unqualified handoff**. After the budget
  decision, freeze the independent proposals, execute every case against the
  exact candidate, retain failures/prerequisites, and obtain the evidence check.
  Existing yellow checks and human acceptance remain unchanged.

Recheck main/release freshness immediately before the eventual merge after
human testing; this record does not promise that main will remain stationary.
