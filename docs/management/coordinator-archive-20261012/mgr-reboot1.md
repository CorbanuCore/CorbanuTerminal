You are the manager-loop worker, round 2. Read mgr-daemon1.result.md, mgr-recover1.result.md and sec-common.md in
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/ (binding: never delete worktrees referenced by the live owner
settings; keychain rule). Travis decided: login dependency stays as is (option 2: manager runs in his GUI session and
stops on logout); YES to a reboot-safe install. Manager stays on standby (no allocations, no mode change).
Do: 1) requalify the new package that includes merged PR #268 (manager pre-cycle timeout counts as an error) per the
existing owner-manager qualification records; 2) make both jobs reboot-safe: install them as proper launchd plists
(owner job in its current domain, manager job as a LaunchAgent in ~/Library/LaunchAgents loading in the gui domain at
login) via the supported installer path, repinning to the qualified package; 3) verify both tick idle with no holds for
~15 min, and verify the plists load correctly (launchctl print, plist lint); do not actually reboot the Mac; 4) record
under qa/initiative-control/ in a docs-only PR, merge when green (`gh pr merge N --merge --match-head-commit <sha>`).
Report concisely, including how to confirm after Travis's next reboot.
