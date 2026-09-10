# Fable 5.1 — scoped main publication review

## Frozen task and receiving boundary

User: “Please merge your planning work to main so it is available for the
upcoming scrum with Alex.” The existing user-selected review route is Fable 5.1
via the Corbanu tmux harness, high reasoning, no substitute model.

Receiving branch: `docs/alex-scrum-planning-20260910`.
Base: `3cec54d9917b776bedaffb586247d7cd7c633df6` on origin/main.
Original draft provenance: recovery base
`6f8f4ce46446e951437e1ce7e7c4a526e5b9e546`.

Intended delta: 16 proposed plans PF-60–PF-75, 50 unallocated draft sprints,
portfolio overview, scrum landing page, proposed/sprint navigation and bounded
QA evidence/check scripts. Runtime delta is zero. Existing product spec, root
policy, active plans/sprints, lifecycle checkers, CI and releases are unchanged.
Do not infer authorization to activate proposals, alter allocations, merge the
separate dashboard implementation, deploy, enable Task Node writes or release.

The old recovery portfolio has already had full Fable review and a clean
closeout. This pass reviews its integration into newer main, including all
changed material and the new publication checker. Old review-scope/manifest/
closeout files are historical evidence, not instructions for this review.

## Main reconciliation and checks already observed

- Main retains its two active plans and two reserved security sprints. The user
  approved up to three independent initiatives with sequential sprints; the
  policy/checker/allocation transition is explicitly separate and not landed by
  this docs-only merge. PF-75's two-builder cap bounds a calibration experiment.
- Main already fails its sprint checker with seven errors involving PF-43,
  PF-44 and duplicate PF-45-S01. The unchanged checker continues to report them.
  The new fixed-publication checker compares the exact inherited set and adds no
  errors; it is not a dispatch or release waiver. See baseline.json.
- Portfolio validation: 16 plans, 50 sprints, max 73 sprint lines, draft/
  unallocated, exact product-spec citations, navigation and links pass.
- Plan checker passes (2/2 active). Existing unit tests pass (4 plans, 19 sprints).
  YAML syntax passes. Full MkDocs rendered build is unavailable, not claimed.
- Source reconciliation updates PF-60/PF-61 to the receiving Corbanu API
  balance/key decision, not legacy Plan entitlements, and PF-72 to reuse main's
  existing Campaign Tracker pilot. Historical title/source anchors remain.

## Review questions

1. Does the diff preserve newer main policy/product/release/active records and
   import only the intended planning/QA boundary?
2. Are current-versus-historical claims, draft authority, three-initiative
   transition, dependencies and human capacity internally consistent?
3. Do citations, links, IDs, nav and testable handoffs hold on receiving main,
   without resurrecting obsolete behavior or duplicating existing work?
4. Is the fixed-baseline check honest about inherited errors and lack of
   implementation/human acceptance? Check for concrete checker regressions.
5. Does public planning exclude private recording identifiers, credentials and
   raw private logs, while preserving useful source attribution?

Use read-only inspection. Do not read credential files, raw model traces, private
recording/transcript or unrelated personal files. The curated planning is the
authorized review input. No web search, nested review, mutation or live testing.
Report concrete introduced defects, with exact path/line, impact and smallest
correction. Inherited ledger defects belong to follow-up, not scope expansion.
Missing future worktree/owner/budget is intentional in a draft, not a runtime bug.
After a clean result, stop. Review is advisory, not true-TUI qualification or
acceptance of any proposed sprint.

## Focused closeout after review 1

The full receiving-main review judged the patch correct and found one P3
navigation portability issue in the portfolio's Campaign Tracker spec link.
Accepted as a small in-scope publication defect. The fragment is removed and
the exact section title is now in the link label, avoiding different GitHub/
MkDocs slug rules. The target file and exact heading exist on receiving main.
Only that row and this scope receipt changed since the completed full review.
Recheck that correction, the unchanged planning boundary and focused checks;
do not restart broad product discovery or demand unrelated inherited repairs.
No product behavior, authority, allocation or contract was changed.
