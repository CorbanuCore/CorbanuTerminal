# RETURN — owner-ownership-56
Base: `16b4fd24aa2b888dcf30bbaed123513940bb8cbf`; frozen brief SHA-256 verified:
`d77ff0109009c6392f4ef93f5133100c6e25f841d2d843cf26c6f16d6c97f57e`.
Bounded fix under **Internal delivery control — TO BUILD**, “durable event dispatch, acknowledgments and watchdog”.
Existing initiative-delivery-control / PF-80-S01 (`in_progress`); assigned worktree `owner-recurrence-20260917`, branch `bootstrap/owner-recurrence-20260917`. Shared plan/sprint coordinates remain manager-owned.
Internal installer revision only; no TUI change, release, live-repository proof or independent functional acceptance claimed. Manager retains the later recurring-owner acceptance gate.

## Behavior
Service print exit0 means present; exit113 plus the label-specific missing-service diagnostic means the domain is readable and clear.
Exit112 plus an exact diagnostic line identifying the requested domain/UID means `domain_absent`: `Could not find domain for user gui: <uid>` or `Could not find domain for uid: <uid>`.
Only the sibling may be missing. Install, idempotent reinstall and uninstall proceed and retain `sibling_observation={domain,state:"domain_absent",reason:<diagnostic>}` in installation.json.
The absent domain's service could not be checked; the receipt records why rather than claiming a successful service probe. Readable clear siblings record state `absent`, reason null.
Loaded conflicts still refuse before tick/plist/receipt writes. Permission failures, timeouts, malformed/mismatched diagnostics and missing selected domains still refuse.
Uninstall rechecks both domains before deleting the plist. Schedule-health observations retain their existing unknown/observation-unavailable representation.
The installer catches only `OSError` and `subprocess.SubprocessError`; validation errors propagate. Real malformed-label CLI result: `fable_launcher.LaunchError: invalid_label`, exit1.

## Native trials and limits
Disposable inert runtimes/labels, synthetic configuration, UID501; no operator credentials, live activation, inference or dispatch.
1. Selected user, GUI sibling present/conflicting: `service_conflict: gui/501/com.corbanu.initiative-owner.test-o56-user-4bhqsaux`. Only installation.lock remained; selected print113.
2. Selected user, GUI sibling present/clear: phase installed; user print0; receipt `{"domain":"gui/501","state":"absent","reason":null}`; uninstall phase uninstalled, both prints113.
3. Missing GUI probe at unused UID2147483646: exit112, `Bad request.\nCould not find domain for user gui: 2147483646\n`.
4. Reverse: selected GUI refused loaded user sibling with `service_conflict: user/501/com.corbanu.initiative-owner.test-o56-gui-4bhqsaux`; after removal installed successfully, receipt sibling user/501 absent, then uninstalled; both prints113.
Reverse missing-user probe at UID2147483646 returned exit1, `Could not print domain: 1: Operation not permitted\n`. It is NOT evidence of absence.
The GUI absence probe is real but at another UID, not an end-to-end missing-sibling install. Both current-UID missing-sibling directions, install/reinstall/uninstall and receipt diagnostics pass deterministic tests only.
Thus the requested four-case native gate is incomplete. This host has both current-UID domains; no login domain was removed and no denial bypassed.
All disposable loaded jobs were removed, verified print113 in both domains. Harness: `test_owner_ownership_56.py`.
Raw trial output: `/private/tmp/owner-ownership-56-trials-4bhqsaux/trials.json`; SHA-256 `d766102ec751cf996d5c227144344c50971d74c53507e655d725890b6bdf123d`.
Preserved harness failures: first used the wrong fixture class (before any launchd action); second omitted the bad-label fixture directory after successful service trials. Original logs and cleanup retained separately; final replay exit0.

## Activation judgment
I would not yet run the live-label user-domain activation: I want a disposable no-GUI login/account install/uninstall trial, a reachable reverse missing-user trial or the manager's explicit prerequisite disposition, and resolution/disposition of the final suite failures.
The focused correction tests and current-host disposable user-domain lifecycle pass; qualification is incomplete because the native gate is unmet and the final full suite failed.
Keep the already-reviewed migration order: receipted GUI uninstall, verify conflicts removed, preserve history, fresh private user-domain root and reviewed runtime/config pins. No live-profile inspection was performed.
No factual defect in the brief was established. Its four-case native gate could not be fully exercised in this login session; an inaccessible user-domain probe cannot establish absence.

## Verification
Environment E: `/private/tmp/owner-ownership-56-env.pucdLM`.
Created with `env -i HOME=E PATH=/opt/homebrew/bin:/usr/bin:/bin python3 -m venv E/venv`; installed with E/venv/bin/python `-m pip install --isolated --disable-pip-version-check --no-input -r scripts/initiative_control/requirements.txt` under the same environment.
Only pinned markdown-it-py3.0.0, mdurl0.1.2, slack-sdk3.44.1 plus bundled pip; no inherited site-packages.
Tests use env-i, HOME/all three profile aliases=E, native-keyring denial, TMPDIR=/private/tmp, checkout-only PYTHONPATH, venv/Homebrew/system PATH, PYTHONDONTWRITEBYTECODE=1.
Suite command: E/venv/bin/python `-B -m unittest discover -v -s scripts/initiative_control -p 'test_*.py'`.
Initial owner run: 66 passed, 9.934s. First full run: 748 passed, 450.505s, exit0; predates the final schedule-health compatibility assertion and is retained separately.
Final tree: 749 tests, 746 passed, 3 failed, no skips/errors; 446.330s, exit1. Owner subset: 67/67 passed. Suite SHA-256: d748a31c7ccaf1d53676cab78d949f863a79b368db966bfddcc095b69201451a.
Remaining failure names (all test_decision_manager.ManagerTests): test_tmux_handoff_end_to_end_mixed_transport_refusals_and_one_unlock; test_tmux_ingress_during_ack_preserves_native_final_fence; test_tmux_slow_model_turn_keeps_fresh_witness_and_unlocks.
Late-abort failure did not recur: test_fable_launcher.RealTmux.test_late_abort_invalidates_candidate_and_unrelated_process_survives passed in both full runs; no attribution investigation performed.
Logs in E: pip.log, owner-tests.log, suite.log, suite-final.log, trials.log, trials-replay.log, trials-final.log, trials-final-tree.log. Final native replay exit0; prior replay artifacts retained.
Plan/sprint checkers pass. No Rust tests, workspace formatter, native credential prompt or push.
Changed lines (additions+deletions): 31 production + 48 evidence = 79 outside tests; 82 unit tests + 113 trial harness = 195 inside tests; 274 total. Only assigned paths changed.
