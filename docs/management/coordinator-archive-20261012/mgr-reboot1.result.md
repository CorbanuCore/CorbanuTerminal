Both jobs are now reboot-safe and live on the requalified package (owner armed at generation 13, no holds), and the record is merged as PR #276. I didn't reboot the Mac, so "comes back after a reboot" is only simulated, not proven.

**Installer change first.** The existing installer couldn't put jobs in `~/Library/LaunchAgents`. Also, macOS only reloads that folder into the GUI session at login, not into the owner job's background domain (`user/501`). On this Mac, Homebrew's postgresql plist there lists the background session too, but after boot and login it is loaded only in the GUI session. So I added an installer option, PR #275 (merged `8d107f4c1d`):
- **Manager job:** its plist now lives in `~/Library/LaunchAgents` and loads in the GUI domain at login.
- **Owner job:** it stays in `user/501` with its existing plist. A small login agent, `~/Library/LaunchAgents/com.corbanu.initiative-owner-login.plist`, loads it back at each login if it's missing.
- **Gate:** three Opus 5.5 High reviews (two asked for changes, the third approved). The full test suite has the same 2 failures as untouched `main`.

**Requalification:** the package with #268 and #275 (`e4d91473…`) passed a fresh run on the PF-83 VM, repeated step for step from round 7. The independent review verdict was **QUALIFIED**.
- The VM's network fence and credentials were restored byte-identical.
- No direct connections to model providers.
- The broker-down control failed closed.
- The worker completed ACK, START and RETURN, and the owner closed it.

**Live repin:** both jobs were uninstalled after completed ticks, the owner was disarmed, and the preflight passed all 5 items. The repin script then stopped early, so I finished it with a follow-up script (`live/restage-continue-r9.py`):
- **Why it stopped:** its own off-state test tick collided with launchd's first tick and didn't run. The next tick latched the expected `owner_off` hold.
- **Spacing fix:** both jobs had loaded within a second of each other, so their 30-second ticks lined up and blocked each other. I reinstalled the manager about 15 s later.
- **Then:** armed generation 13 and cleared the `owner_off` hold with evidence.
- **Unchanged:** the manager stays on standby with no new allocations, and its config is the same except the package digest.

**Verification:**
- **Plists:** `plutil -lint` passes for all three. `launchctl print` shows each loaded from the expected path in the expected domain, and nothing is disabled.
- **Simulated post-login:** I removed the owner job from `user/501` and triggered the login agent. The job came back from the right plist and kept ticking normally.
- **15-minute watch:** the owner ran 28 ticks, all active with no holds or errors. The manager ran 30 ticks, idle apart from 2 normal waits for the owner's lock. No manager cycle started and no worker launched.
- **Login Items:** couldn't check whether macOS has approved the jobs as Login Items. `sfltool dumpbtm` needs admin rights; my one attempt hung and I stopped it after 25 s. If Travis saw a password prompt around then, it was from that and can be cancelled.

**To confirm after Travis's next reboot:**
1. Log in. Within about a minute:
   - `launchctl print user/501/com.corbanu.initiative-owner | grep -E 'path|state|runs'` shows `…/corbanu-owner-live/schedule/owner.plist`.
   - `launchctl print gui/501/com.corbanu.initiative-owner.manager | grep -E 'path|runs'` shows `~/Library/LaunchAgents/com.corbanu.initiative-owner.manager.plist`.
   - `launchctl print gui/501/com.corbanu.initiative-owner-login | grep 'last exit'` shows 0.
2. Both `~/Library/Application Support/corbanu-owner-live/{schedule,manager-schedule}/tick.json` show `"hold": null` and a recent `completed_at`.
3. If either shows `interrupted_tick`, shutdown killed a tick mid-run. Clear it by running `owner_daemon.py --schedule <root> --recover "<evidence>"` from the installed runtime with `-B`.
4. If macOS says "Background Items Added" for python3.14 or `sh`, leave them allowed. Turning them off in Login Items stops the reload.

I removed my two worktrees and the VM-staging binary copies. Nothing the live settings point to (`worktrees/owner-loop-r7-20261003`) was touched. The record is `qa/initiative-control/management-bootstrap/owner-manager-reboot-safe-20261008.md`, and the evidence is in `.codex-work/owner-manager-enable-20261002/round9/`.