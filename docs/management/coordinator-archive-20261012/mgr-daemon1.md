You are the manager-loop worker. Read mgr-status1.result.md, mgr-recover1.result.md and sec-common.md in
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/ (binding rules). Travis decided: manager stays on STANDBY
(option A: no allocations, no mode change), and (B) fix the login dependency so the manager job runs without his GUI
login session, like the owner job (user/501 LaunchAgent vs gui domain: pick the supported mechanism, e.g. a background
LaunchAgent/LaunchDaemon in the user domain, keeping least privilege; no root unless unavoidable and justified).
Also fold in the small fix: a single pre-cycle TimeoutExpired counts as an error (like the owner lane) instead of a
latched HOLD. Do this as a normal sprint on a branch: code + tests, Opus 5.5 High review, re-qualify the new package
per the existing owner-manager qualification records (the pinned package changes), then reinstall/repin the live jobs
only after qualification passes, verify both tick idle with no holds for ~15 min, and confirm the manager keeps
ticking after a logout-equivalent check (e.g. `launchctl print` domain + survives `gui` session absence, as far as can
be tested without logging Travis out). Never delete worktrees referenced by the live owner settings. PR to main, merge
when green (`gh pr merge N --merge --match-head-commit <sha>`). Report concisely, incl. anything needing Travis.
