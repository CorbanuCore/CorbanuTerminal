# RETURN — owner-probe-51
Frozen base: `8b968c94b1384e14379b610249d3b88f41b684c2`; brief SHA-256 verified:
`ed88811a0b9f7c3beaa66ca9ec23bbc387f003125cecaa3b91d1d92346b8ffc5`.
Product-initiative revision within active initiative-delivery-control / PF-80-S01
(in_progress). Product heading **Internal delivery control — TO BUILD**:
“durable event dispatch, acknowledgments and watchdog”; “Show blockers, rendered
sprints, human test plans, machines, run logs and freshness”.
Frozen allocation owns this worktree/base; plan lists this branch at its earlier
allocation base. Shared plan/sprint reconciliation and acceptance remain manager-owned.
This is internal implementation/diagnostic evidence, not an independent functional
acceptance or human-test handoff. Later PF-80 recurring-owner functional gates remain.
No live job enabled, domain-wide setting changed, credential access, push or release.

Installation receipts now record `domain` as `gui/<uid>` or `user/<uid>`.
Both attribution and health observation read it; uninstall also follows it.
Only the current user's two domains are accepted. Missing legacy fields mean GUI,
because the old installer could only use GUI. Installer `--domain user` explicitly
selects user/Background; default GUI is unchanged. Reinstall rejects domain changes;
this does not migrate or activate the live owner. Tests cover both install/observe/
attribute/uninstall routes, legacy receipts, cross-domain conflict and invalid domains.

Host: macOS 26.6.2 (25G83). Six disposable labels, identical fixture/runtime,
StartInterval=30, ThrottleInterval=0, ProcessType=Background; only domain and
LimitLoadToSessionType differ. Successful bootstraps received one initial kickstart.

| Requested domain | Session limit | Bootstrap | Automatic firings in 107.506s |
| --- | --- | --- | --- |
| GUI | Aqua | 0 | 0; interval pending |
| GUI | Background | EIO (5) | Not installed |
| GUI | omitted | 0 | 0; interval pending |
| user | Aqua | EIO (5) | Not installed |
| user | Background | 0 | 3, plus two explicit kickstarts |
| user | omitted | EIO (5) | Not installed |

Every failed combination was absent in BOTH domains (print=113): no silent rerouting.
The two cross-combinations could not be installed on this host, so these trials
cannot isolate domain from session type. EIO alone does not establish a universal
launchd restriction. The supported claim remains **GUI/Aqua on-demand-only does
not fire here; user/Background does**. A domain-only causal claim is unsupported.
All five samples captured service print, domain headers, launchd log show and
pmset batt/therm/custom: GUI on-demand count=1/session=Aqua, user session=Background,
AC power/lowpowermode=0, no recorded thermal/performance/CPU-power warning.
UTC samples: 06:05:13.956, 06:05:46.997, 06:06:21.954, 06:06:24.909, 06:06:58.004
on September 17. GUI/Aqua and GUI/omitted each stayed at one completed run;
user/Background counts were 1,2,3,4,5.

Actual post-interval attribution (launchctl print PID and blame captured inside each run):
- 06:05:42.633 PID73268: immediate reason=interval, firing=interval.
- 06:06:13.184 PID73627: interval again; health=recurring-at-observation (30.552s gap).
- Kickstart issued 06:06:22.887, exit0; 06:06:23.107 PID73706:
  immediate reason/blame=non-ipc demand, firing=other, previous_success=null,
  health=stalled/recurrence-unproven. Prior intervals did not turn it into an interval.
- 06:06:53.646 PID73918: interval resumes; previous_success remains null and health
  stays unproven until a second consecutive interval. No pending-interval field was
  present on this manual firing; the synthetic pending-field regression remains separate.

Raw evidence: `/private/tmp/owner-probe-51-52zuiggd/diagnostic.json`,
SHA-256 `c6f22f99cadfafdfb76e7b6c468841ca721fdfe77e3a884b1be24e26d3051431`.
Per-run prints/blame/tick/health, private plists and logs remain beside it.
All three installed jobs booted out successfully; all six labels then absent in
both domains. Uninstalled combinations returned bootout=3 (no such process).
Live owner label independently observed absent in both domains (113).
Final owner_daemon.py matches the copied trial runtime:
`1c07f5300ce651f005f3538d56ac6fd0fbbb17dfbb32f2f237f3b3d897a9a305`.

Environment: `/private/tmp/owner-probe-51-env.lQDPDT/venv`, freshly created by
`env -i HOME=<disposable-root> PATH=/opt/homebrew/bin:/usr/bin:/bin python3 -m venv <venv>`.
Then `<venv>/bin/python -m pip install --isolated --disable-pip-version-check --no-input
-r scripts/initiative_control/requirements.txt` under env -i. Only markdown-it-py3.0.0,
mdurl0.1.2, slack-sdk3.44.1 plus bundled pip26.0.1 installed.
Tests use env -i, disposable HOME/all three profile aliases, venv/Homebrew/system PATH,
PYTHONDONTWRITEBYTECODE=1 and CORBANU_TEST_NO_NATIVE_KEYRING=1. No native prompt.
Owner tests: 56/56 passed in 7.991s. First full run: 738 tests/446.601s,
727 passed, 3 failures and 8 errors (exit1): seven noncanonical temporary-path errors,
two child module-import failures and two blank-pane TMUX startup timeouts.
Preserved in suite.log. Replay adds TMPDIR=/private/tmp and checkout-only PYTHONPATH;
no code changes between runs. Command: venv python -m unittest discover -v
-s scripts/initiative_control -p 'test_*.py'. Final replay: **738/738 passed**,
451.178s, exit0, no skips. All earlier failures remain in the first log.
Logs: disposable root's owner-tests.log, suite.log, suite-replay.log and trial.log.
No Rust tests or formatter.
Both governance checkers and git diff --check pass. No contradiction found in the
brief; its caution about the domain/session confound is upheld, with the host constraint above.
Final suite log SHA-256: `d67ca22400a929ee9679d14190e07f0bc0541d112812c63ea0bf37b0858487dc`.
Changed lines (additions + deletions): 35 production + 86 evidence = 121 outside
 tests; 65 unit-test + 130 diagnostic-test = 195 inside tests; 316 total.
