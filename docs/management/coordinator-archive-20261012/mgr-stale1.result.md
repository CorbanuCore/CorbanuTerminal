Both cleanups are done, but the "security paused" claim isn't fully gone. The manager will still conclude security is paused until some old text stored in the coordinator is fixed, and the live dashboard can't be refreshed yet.

**What changed** (coordinator revision 2981 → 2989; the manager stays on standby with no allocations, and launchd jobs, packages and pins weren't touched):
- **Security:** I added a Travis-authority event saying security has been active and run by hand since 2026-10-06, and that the manager gets no allocation or dispatch. The workstream itself was already `mode: enabled`, so there was no mode to change.
- **`slack-receiver-02`:** I first confirmed the receiver is really gone (no session, socket or process, and no Slack listener running). Then I closed it as failed using the coordinator's normal reconcile command, with that reason recorded. Nothing was relaunched and no Slack message went out.
- **Its allocation:** closing the claim left the allocation looking unused, which made the owner wake the manager once. I marked the allocation used up with the same call routine cleanup makes. Without that, the manager would have been woken every 12 hours for a week.

**What the manager did:** the changes triggered two automatic manager runs. Neither did anything (no actions, no prepared work).
- The first one's reasoning correctly said security is resumed and run by hand.
- The second one still said "security and product continuation remain paused".

Afterwards both jobs were idle with no holds or errors, and no further wake-ups came.

**Dashboard:** I removed the "3. Security — paused" card from the dashboard settings file, keeping a backup (`control.json.bak-20261008`). A local test render confirms the card is gone.

**Mistakes I made, and their state:**
- My attempt to publish the dashboard failed, and its error handler marked the live dashboard as "sync failed" (02:18Z). The marker is still there, and the live site still shows the Oct 6 version with the old card.
- The local test render overwrote the live `state/source.json`. I restored it byte-for-byte from the Oct 6 copy, which matches the server's.

**Still open:**
1. **The stored "paused" text.** The security workstream still carries its 2026-09-13 "paused" text and an "Owner paused" history line. That's why the manager's second run said paused again. No coordinator command can change that text, and I didn't edit the database directly. It needs either a small new coordinator command or Travis approving a one-off, recorded edit. It doesn't affect dispatch.
2. **Dashboard publication.** Publishing refuses because the dashboard's source checkout (`worktrees/management-workstreams-20260911`) is detached rather than on its branch. Another session owns it, so I left it alone. Once it's back on `integrate/management-workstreams-20260911`, re-running `sync-source.sh` will publish the change and clear the failure marker.

**Record:** `qa/initiative-control/coordinator-stale-state-20261008.md`, docs-only. PR #265 merged as `003298a509` and the follow-up PR #271 as `f6fc0c83e7`, both with all checks green. Temporary worktrees and copies are removed.