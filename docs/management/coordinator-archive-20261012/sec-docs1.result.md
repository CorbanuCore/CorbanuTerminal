I opened PR #214 for the decisions bookkeeping and merged it (merge commit `00376a1fb0`). All PR checks passed, and both checkers pass on main after the merge. While it was open, main moved four times (PF-30-S03, PF-33-S02, PF-28-S02 and Claude-pane PRs landed). I merged origin/main each time and kept the other lanes' versions of their own records.

**Archived** (all three had their per-sprint gate evidence complete):
- **PF-27-S02:** its two open decisions are recorded as you made them, and Windows moved out (below).
- **PF-28-S01 (PR #208):** its only blocker was PF-27-S02. Its open items moved to PF-28-S02, which has since closed most of them in PR #216. Its two remaining PF-28-S01 items (scrub text fields per type; per-session stream state and fewer rescans) are still open there. The financial derived-view item went to PF-39-S01 in the P1 plan.
- **PF-33-S01 (PR #210):** the credential host/port/method/path binding went to PF-28-S02 (the coordinator had already put it there). Proxy pinning and the test seam were done in PF-33-S02. Its known costs are recorded as documented behaviour, not scheduled work.

Their open milestone items (code-blind VM run, PF-26 requalification) are marked as moved to the milestones and the P1 plan, following the earlier PF-27-S04 archive.

**Moved:** PF-27-S06 (Windows broker and launch) is now in the P1 hardening plan, still depending on PF-27-S02. It became order 45 because 47 is already taken there. PF-27-S02's Windows verification item points to it.

**Recorded:**
- **PF-41-S01 Remaining:** a new item to show MCP servers, hooks, the `!` shell and app-server `command/exec` as "not contained" in protected levels.
- **Claude panes:** I put this in PF-24-S02 (it's about what a protected level applies; PF-25 covers grants and revocation). The TUI worker's PR #220 landed while my PR was open: panes are refused while Aggressive is active or saved. So it's recorded as done, and bringing them under the launch contract is tracked in issue #218.
- **Nested `corbanu exec`:** recorded in PF-24-S02 as an open item: configurable between refuse and pass Aggressive down, default refuse. The archived PF-24-S03 had sent its follow-ups there.

The indexes, plan sprint maps, proposed-plans index, mkdocs nav and the source-reconciliation links are updated. I also fixed old broken links there to PF-27-S04 and PF-30-S01. I couldn't run `mkdocs build` because mkdocs isn't installed here; a link check over the changed files found nothing broken.

**Left open:**
- The nested `corbanu exec` setting (TUI worker implementing).
- PF-41-S01's new "not contained" display item.
- PF-27-S06 Windows work, until the P1 plan is activated.
- Issue #218 (Claude panes under the contract).
- PF-28-S02's two open PF-28-S01 items.
- PF-33-S02's open items, including carrying the pinned peers through the broker; its record says "coordinator to place".

Nothing was held back for incomplete gate evidence.