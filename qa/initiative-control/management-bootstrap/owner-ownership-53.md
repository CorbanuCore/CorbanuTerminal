# RETURN — owner-ownership-53
Base: `0ef70486f8a11d6fc7f6593fc38f83d0fd282b0d`; frozen brief SHA-256 verified:
`1ad5943baa19c5bbf8290f65fc7b74c4076ec582199f3a808b48628c36ebeb73`.
Bounded ownership fix under **Internal delivery control — TO BUILD**,
“durable event dispatch, acknowledgments and watchdog”; active
initiative-delivery-control / PF-80-S01 (in_progress). Worktree and branch:
`/Volumes/CorbanuDrive/Corbanu/worktrees/owner-recurrence-20260917`,
`bootstrap/owner-recurrence-20260917`. Shared plan/sprint records remain manager-owned.
Internal installer evidence only: no TUI path changed; no live-repository,
independent functional acceptance or human-test readiness claimed. Manager retains
PF-80 recurring-owner functional acceptance and live activation. No live profile,
credential access, native prompt, live activation, push, Rust test or formatter.

Fresh install now queries both current-UID gui and user domains before creating a
plist/receipt/tick state or bootstrapping. Only an explicit service absence
(launchctl print exit113 with the expected label-specific diagnostic) counts.
A loaded label fails with `service_conflict: <domain>/<label>`; an unobservable
domain fails with `service_observation_unavailable: <domain>/<label>`.
Reinstall also refuses an alternate-domain conflict; receipt-domain changes stay
prohibited. Both owned domains were readable without elevated privileges here.
These checks are observations under the existing per-root installation lock;
they do not serialize external launchctl commands or concurrent different roots.

Real disposable trials on macOS26.6.2/25G83, UID501; inert /usr/bin/true jobs,
RunAtLoad=false, KeepAlive=false, no interval. Both actual CLI attempts exited1:
```text
fable_launcher.LaunchError: service_conflict: gui/501/com.corbanu.initiative-owner.test-o53-gui-sv8gqpht
fable_launcher.LaunchError: service_conflict: user/501/com.corbanu.initiative-owner.test-o53-user-sv8gqpht
```
First: GUI loaded/print0, user absent/print113, user install refused; second:
user loaded/print0, GUI absent/print113, GUI install refused. Both refused roots
contained only installation.lock. Destination remained absent after each refusal.

Correction to the brief: simultaneous duplicate jobs did NOT reproduce here.
Direct alternate-domain bootstrap returned `Bootstrap failed: 5: Input/output error`
in both directions; the IDENTICAL alternate plist loaded successfully (exit0)
after booting out the original. Same-path/two-session plist behaved identically.
This is a host observation, not proof that every macOS version prevents duplicates.
The initial harness wrongly asserted duplicate bootstrap would succeed; its
failed run and cleanup are preserved separately, not relabeled as a pass.

Cross-domain uninstall IS reachable after a manual move leaves a stale receipt.
A GUI-receipted, byte-identical shared plist was booted out of GUI and loaded in
user; new uninstall removed it, marked uninstalled and both prints returned113.
A moved job at a different plist path was refused in both directions with
`unowned_service: <actual-domain>/<label>`, left loaded, and removed explicitly
by the fixture owner. Uninstall validates every observed instance's receipt-bound
path/hash before any bootout, then requires absence in BOTH domains before deleting
the plist or marking uninstalled. Two simultaneous owned instances, an orphan,
foreign ownership, probe failure and failed removal are covered by unit tests;
the two-simultaneous-instance execution is simulated, not a real-host claim.
All test labels were absent in both domains at cleanup.

Safe manager migration: use the receipted GUI uninstall, establish both-domain
absence, preserve its receipt/history, then install user/Background into a fresh
private schedule root with reviewed pins/config. Reusing the old root still
refuses a domain change. If a domain cannot be observed, resolve its session/access
first; do not skip it or infer absence. Foreign-path conflicts require their
own verified removal. No live migration was performed by this allocation.

Raw replay: `/private/tmp/owner-ownership-53-sv8gqpht/trials.json`,
SHA-256 `1bbae77f233c6809e3888009f0d79c035039659da6c12be3b2077b7eb8f1bf3a`.
Original failed trial: `/private/tmp/owner-ownership-53-86lg6bio/trials.json`,
SHA-256 `bf34c79dda8da299cf722866af381030d82300e555f6b035539508de7945af56`.
Harness: test_owner_ownership_53.py. Full CLI stderr/commands and selected real
launchctl identity/state lines retained; inherited environment is not exported.

Disposable environment: `/private/tmp/owner-ownership-53-env.xw0Smn` (E).
Created with `env -i HOME=E PATH=/opt/homebrew/bin:/usr/bin:/bin python3 -m venv E/venv`;
then `E/venv/bin/python -m pip install --isolated --disable-pip-version-check --no-input -r scripts/initiative_control/requirements.txt` under the same env.
Installed only markdown-it-py3.0.0, mdurl0.1.2, slack-sdk3.44.1 plus bundled pip26.0.1.
Tests: env-i, HOME/all three profile aliases=E, TMPDIR=/private/tmp, checkout-only
PYTHONPATH, venv/Homebrew/system PATH, PYTHONDONTWRITEBYTECODE=1 and native-keyring denial.
Focused command: venv python -m unittest discover -v -s scripts/initiative_control -p 'test_*.py'.
Owner tests: **62/62 passed**, 8.466s. Full suite: **744 tests, 736 passed, 8 failed**,
467.228s, exit1, no skips/errors. Six failures in test_fable_launcher.RealTmux
(startup/error assertions) and two blank-pane fixture timeouts in test_owner_tmux.
All eight replayed unchanged: **7/8 passed**, 42.824s, exit1; late-abort still returned
launcher_failure instead of final_evidence_invalid. Those modules/tests match the base;
no separate whole-base run or root-cause proof claimed. Full-suite gate remains failed.
Logs in E: pip.log, owner-tests.log, suite.log, failure-replay.log, trial.log, trial-replay.log.
Suite log SHA-256: `60d46cc4284fbd1e7a3006cacf21f4461bf9fa5c9031070146f525dca944e5b9`.
Both governance checkers and git diff --check pass; git status lists only assigned paths.
Changed lines (additions+deletions): 34 production + 85 evidence = **119 outside tests**;
131 unit-test + 144 real-trial harness = **275 inside tests**; **394 total**.
