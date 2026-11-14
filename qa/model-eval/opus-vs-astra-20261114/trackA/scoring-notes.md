# Track A scoring notes (scorer: bench-1114, judged against code at the defective head)

## A01 #1837 (P: inherited stdin lost behind chroot)
- opus: MISSED. Verdict REQ (correct). Extras: (1) `eval --`/`eval eval`/`noglob`/`trap`/`coproc chroot` not unwrapped — plausible, unverified → uncertain; (2) `--skip-chdir` checks child in outer cwd, losing unknown folder / wrapper `-C` — plausible (also found by astra), unverified → uncertain; (3) doc blank-line nit → real (trivial). Severity n/a.
- astra: FULL (#1 High: passing only `run` discards pipe_from/stdin_from; `chroot / bash -s <payload` loses script). Verdict REQ. Extras: (2) unseen-output markers erased by reparse → uncertain; (3) `--skip-chdir` + `env -C` → uncertain (same as opus #2); (4) no dispatch-level integration tests → policy opinion, count as real-low (AGENTS asks integration coverage) — judged uncertain. Severity High = calibrated.

## A02 #1962 (P: assert_cmd 2.2 cargo_bin panics; utils/cargo-bin relies on Err)
- opus: MISSED (named lib.rs:46 deprecated call but concluded safe). Verdict APPROVE (wrong). Extras: checksum nit (unverifiable, not a defect) → none counted.
- astra: MISSED. Verdict APPROVE (wrong). No extras.
- Note: 2.2.2 source not in local registry and network barred; both had to rely on prior knowledge.

## A03 #1055 (P: feed-registration loop lacks work::step())
- opus: MISSED. Verdict APPROVE (wrong). Extras: (1) low test-quality (no known-answer assert) → real-low/opinion → counted real (minor); (2) perf nit O(commands×depth) still → plausible, real (minor, not regression); (3) doc nit → real (trivial).
- astra: FULL (shell.rs:837, plus existing source-scanning test `every_classifier_loop_steps_the_counter` would fail). Verdict REQ. Severity Medium = calibrated. No extras.

## A04 #2043 (P: PF-33/PF-27-S02 network settings dropped on profile switch; S1 required provider replaced by stale gpu-*; S2 config_tests don't compile)
- opus: FULL (#1 High, exact fn + fix identical to the real repair). Also S2 found (contains() + Tui literals). Verdict REQ. Severity High calibrated. Extras: #3 required provider + inherited model now fails closed instead of falling back (mod.rs:5002/5091) — related area to S1 but a different mechanism; plausible, unverified → uncertain.
- astra: FULL (#1 High). Also S2 found (#3, #4). Verdict REQ. Extras: #2 network_proxy_credential.rs:235/:324 call `build_state_with_audit_metadata(audit_metadata)` with 1 arg vs 3-param signature → VERIFIED real compile error (pre-existing on sync branch from upstream signature, exposed not introduced) → real.

## A05 #2133 (P1 possible literal assignment value lost → protected read unclassified; P2 quadratic nested background marking)
- opus: PARTIAL (P2 found exactly at shell.rs:1320-1328, rated Low vs expected Medium → within ±1, calibrated). P1 not found. Verdict REQ. Extras: (1) conditional brace group / newline after &&,|| loses condition → real (maintainer later fixed "conditional and compound assignment boundaries", a01d7b6a21); (2) pipeline assignments → real (maintainer repair "handles pipeline ... provenance"); (3) function body assignment → real (same repair: function provenance); (4) stuck keyword group from heredoc text → safe-side usability, uncertain.
- astra: FULL (P1 #1 High at tainted_action.rs:2526 exact: `(p=$HOME; cat "$p/.docker/config.json")` loses protected path; P2 #6 Medium exact). Verdict REQ. Severity calibrated. Extras: #2 conditional group/newline → real (fixed a01d7b6a21); #3 pipeline → real (maintainer repair); #4 backgrounded AND-list marks only last command → likely real (maintainer pass6 "retain the outer AND-list") → real; #5 redirection substitution clears just_closed_group → plausible, unverified → uncertain.

## A06 #1171 (P: relative .git path attributed to own repo even when command changed folder)
- opus: PARTIAL (#3 notes `git -C sub config` reports `.git/config` relative to sub but it is resolved against cwd — right mechanism, consequence framed as "missed" not "misattributed"; not the centrepiece). Verdict REQ (correct). Extras: #1 Permissive advice wrong under approval Never → plausible design point, uncertain; #2 read-denial lines mentioning .git counted as blocked write → real (both models); #3 whitespace splitting breaks paths with spaces → real; #4 outermost .git nit → uncertain; #5 TUI test duplicates wording nit → real (trivial).
- astra: MISSED. Verdict REQ (correct, for other reasons). Extras: #1 false write attribution on read denial → real; #2 paths with spaces → real; #3 integration test uses POSIX printf on Windows → uncertain (not checked whether gated).

## A07 #1175 (P: person's task turn/start carries visible pane's cached settings; S1 parent-owned panes; S2 kill switch re-check, Running before turn)
- opus: FULL (#1 High: workspace roots + sandbox fallback from visible pane via submit_thread_op — same mechanism as the real fix "no cwd/model/approval/sandbox/permissions"; S2 both parts found: #2 Running when no turn started, #3 kill switch not re-checked after materialization). Verdict REQ. Severity calibrated. Extras: #4 missing refusal tests → real (minor); #2 second part, rejected steer moves the visible pane's pending steer / acts on the wrong pane (event_dispatch via thread_routing:857/918) → real (Astra found the same, its #3). Counted: 2 real.
- (A07) astra: FULL (#1 High: submit_thread_op supplies displayed pane's workspace roots as explicit override — same leak; sandbox part not named). S2 partly (#2 Running when capacity rejected; kill switch re-check not found). Verdict REQ. Severity calibrated. Extras: #3 rejected steer uses displayed pane's recovery queue → real (Opus found the same, #2).

## A08 #1889 (P: check reads full SDDL incl. inherited Low label → fails; should read DACL only)
- opus: FULL (exact mechanism, line, fix). Verdict REQ. Severity Medium calibrated. No extras.
- (A08) astra: FULL (exact). Verdict REQ. Severity Medium calibrated.

## A09 #1878 (control)
- opus: APPROVE (correct). Extras: brittle exact-list test (low, real-minor opinion), scan only sees one helper name (low, real limitation), misleading error text (nit, plausible). No false alarm.
- (A09) astra: APPROVE (correct). No extras.

## A10 #1944 (control)
- opus: APPROVE (correct). Extras: locale-dependent assertion (low) → real-minor; test uses developer's global git config (nit) → real-minor.
- astra: REQUEST_CHANGES (incorrect vs key). Extras: #1 High: GIT_ALLOW_PROTOCOL=file sets file transport to "always", overriding git's protocol.file.allow=user default, so a hostile .gitmodules can clone a local repo → technically accurate per git docs, but low impact under the sandbox (the command could read those files directly; Astra itself notes filesystem limits still apply); severity overstated → UNCERTAIN, sent to blinded extra grading; #2 locale-dependent assertion → real-minor (same as opus #1).

## Blinded grading of the 13 findings we could not settle (2026-11-14 ~13:20Z)
Both graders (Opus and Astra, unattributed packet `grade/A-extra-brief.txt`, map `grade/extra-map.json`) rated all 13 REAL. They differed on severity (the Opus grader rated lower). All 13 were moved from uncertain to real (`grade/extra-adjust.json`). Since neither grader rejected anything, this is weak evidence that the findings are real; it is not proof. Under the frozen key, Astra's A10 verdict (REQUEST_CHANGES on a control) is still counted as wrong, even though its reason was judged real.
