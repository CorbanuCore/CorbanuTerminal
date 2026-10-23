The PF-23-S03 code and the PF-23-S02 follow-ups are merged with all checks green. One gate item is missing: Z.AI rejects every request with "Insufficient balance or no resource package" (code 1113), so the GLM 5.2 tmux run and videos weren't recorded. The PF-23-S03 sprint record therefore stays `in_progress`. PF-24-S02 can build on the merged code now.

## What shipped
**PF-23-S03**, as four PRs merged in order (#246, #247, #248, #249), plus #251 for the sprint record:
- **Restart recovery (#246):** a stored security level and the revocations come back at every start, and the stricter of the stored and configured level applies. An unreadable state file enforces Aggressive plus the kill switch, with a warning. A repository's `.codex/config.toml` can no longer lower the level, which it could before.
- **Confirmed transitions (#247):** a confirmed change is prepared, then committed or cancelled.
  - A commit merges into the saved state under a file lock, so another session's revocations aren't lost. It saves before anything changes in memory.
  - Grants, pending approvals and child sessions' cached policy are invalidated.
  - Stricter changes apply at once, even if saving fails; downgrades apply at the next start.
- **Fan-out (#248):** restrictive changes, the kill switch and session end now revoke the broker's credentials. This is the trigger PF-27-S04 was missing. They also reset "for session" approvals and reach other sessions in the same process.
- **Memory (#249):** each turn records the level it ran under. Memory treats a past session's level as a floor and never summarizes a session that ran under Aggressive.
- **Size:** #247 is about 1,430 lines, more than half of them tests. I couldn't split it further cleanly; the other three are within the 800-line guidance.

**PF-23-S02 follow-ups (#250):**
- **Linux bug fixed:** with full access and the protection rules on, every command failed on Linux with `bwrap: Can't mkdir /.git`. That covered Aggressive from the start and Moderate after untrusted content.
- **Linux sandbox test:** it confirms protected files stay read-only, including through a symlinked dotfile and after their folder is renamed. It also records the known limits: the rename, the rebuild and removing the link all go through on Linux.
- **New coverage:** submodule git folders and the folder `core.hooksPath` points to (such as `.husky`) are now protected by the sandbox. Unreadable or oversized folders make the whole area read-only. A new nested `.git` is still covered only by the command-text check.

## Gate evidence
- **Tests:**
  - Focused PF-23-S03 tests pass 26 of 26.
  - Full core and touched crates: 5,524 of 5,528. The four failures are three that also fail on main and one that fails only under load.
  - The PF-23-S02 follow-ups pass 33 of 33 on macOS and all tests on the Linux box. The Linux core security tests (241) also pass with main merged in.
- **Linux clippy** on the RTX box is clean for every PR's final tree.
- **Opus 5.5 High review:**
  - PF-23-S03 was approved in round 3. Earlier rounds found real bypasses, all fixed: the saved state was last-writer-wins, the kill switch could be lost, and approvals had races.
  - The follow-ups were approved in round 5. Earlier rounds found real bypasses, all fixed: the submodule check followed links, a fake folder could hide submodules, and making a folder unreadable dropped protection.
- **Video:** one, with no model turn: [unreadable security state warns and fails closed](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s03-pf23s03-unreadable-state-warns-c60857fc5d47-2026-10-07.mp4). The three GLM specs are written but not recorded.

## Decisions for you
- **Recharge Z.AI** so the GLM run and videos can be recorded and the sprint archived.
- **Memory summaries and merging under Aggressive:** I kept both off, the strictest option. Turning summaries on is probably safe because the input is labelled, but it loosens Aggressive. I'd keep them off until PF-25.

## Handed to PF-24-S02
- Call the transition commit off the async runtime; it can wait up to 2 seconds for the lock.
- Reconcile the picker's level file with the new state file.
- Give the user a way out when a change is refused because the saved state changed in another session, and from the session-long kill switch an unreadable state file triggers.
- Warn when a project or managed setting holds the level above a requested downgrade.

All the worktrees and build folders I created are removed, local and on the RTX box. The report is saved as `.codex-work/workers-20261002/sec-taint8.result.md`.