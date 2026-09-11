# Frozen review scope: planning-ledger repair

User request: "Please fix and then merge again." Target: main, based on
`0d1c3240d752335cc89faeb2149c5030063e5d69`.

This is a routine documentation/ledger repair, not product implementation.
Resolve seven inherited checker errors and the same P0 feature-identity
collision on two sibling records. P0 identities move to PF-76–78; Claude-auth
archives and dependencies keep PF-42–45. Restore missing feature contracts and
backlinks; preserve every status, worktree, owner, dependency and evidence claim.
Two historical release links change destination only. Update current landing
pages and index counts. The unchanged global checker must pass with zero errors.

No runtime/auth/config/network/API changes, Task Node writes, dispatch, deployment,
policy cap change or new product authorization is in scope. PF-76-S01 remains
unallocated draft; reconcile shipped provider work before implementing it.
PF-77-S01 stays draft with its natural production observation unchecked.
The sibling relink repair is already completed; do not invent a retrospective
dependency on unfinished production observation.

Review the staged diff and added regression/integrity tests. Existing QA
snapshot scripts deliberately assert seven historical errors and are retained
unchanged; they are not current-tree gates. The new fixed-baseline check proves
this repair only. Do not request runtime/TUI testing or count this model review
as product acceptance. Historical release/QA gaps outside this diff are follow-ups,
not permission to expand the repair.

Use Fable 5.1 high through the selected Corbanu engine. No nested reviewers,
model fallback, web research, live Task Node access or raw transcript inspection.
Return the helper's structured findings; concrete introduced bugs only.
Scope manifest and file hashes are in `review-manifest.json` alongside this file.
