# Management bootstrap — core engineering checkpoint

Accepted internal engineering checkpoint, September 13, PF-80-S01. Parent owns this
unit under the [bootstrap allocation](../../../docs/research/tasknode-integration/coordinator-bootstrap-20260913.md).
Base `1c0df3cb8517b4b578c421ca59ba215f48c7d8c3`; receiving worktree
`worktrees/management-workstreams-20260911`, integration branch of that name.

## Frozen first-review scope

Exactly five new implementation/test files in `scripts/initiative_control/`:
`coordinator.py`, `coordinator_cli.py`, `integration.py`, `test_coordinator.py`,
`test_integration.py`. 902 total lines / 591 non-test at first review. This file
records evidence, not a sixth implementation surface. Allocation target1600/900,
hard2000/1200 is unchanged.

Owner-operated SQLite event/revision/claim/ACK/result state, bounded fresh-manager
packets with three chronological recent actions and durable older history, and
an explicit JSON owner bridge. Native calls must be performed by the parent and
their actual receipts supplied; this code does not invent a native dispatch API.
Exclusive common-Git-directory lock and persistent failed/crashed merge gate,
exact branch/base/source/scope preflight, actual argv-based receiving checks and
private hashed logs. No push, reset, conflict resolution or automatic uncertain
retry is performed. Integration approval remains a trusted owner input.

## Evidence so far

- `python3 -m unittest discover -s scripts/initiative_control -p test_coordinator.py -v`:
  14 passed, including stale decisions, exclusive manager, duplicate/conflicting
  events, pause, exact ACK binding, uncertain dispatch after reopen, retained
  history and chronological briefings, and dependency predicate tests.
- `python3 -m unittest discover -s scripts/initiative_control -p test_integration.py -v`:
  9 passed using real disposable Git repositories, actual merge/receiving check,
  actual conflict, competing lock, dirty/stale/scope denial, receiving failure,
  crash-marker reopen, timeout cleanup and test-induced tree mutation denial.
- CLI help, `git diff --check`, plan and sprint checkers pass.
- Initial no-code remote route check: publisher online, Serve unconfigured,
  port8443 occupied. No service/route or access policy changed.

## Review and limits

Bootstrap core review01 is an additional scoped Fable 5.1 High review authorized
by the bootstrap allocation and integrator delegation. Preserve prior PF80 review
history; no budget reset. Private output root:
`.codex-work/bootstrap-core-review.ZmRFnr`. Original results retained below.

Review01 returned six actionable findings. All are verified in-scope blockers:
rename-detection could hide an out-of-scope deletion; reconciled dispatches lost
future watchdog coverage; long action IDs could break derived events; known
interrupted claims could not be reconciled immediately; oversized/invalid CLI
output could leave an undisclosed manager claim; successful tests could leave
background children alive. First correction addresses all six in the same five
files. Regression evidence: 19 coordinator tests and 11 real-Git integration
tests pass. Candidate982 total/605 non-test remains within allocation. Review02
is the scoped changed-candidate follow-up, not another opinion on unchanged code.

The first broad run used the existing dashboard venv and failed three module
imports because it does not contain `slack_sdk` (188 discovered test entries).
That failure is retained, not a product pass. A disposable venv at the review
root now contains exactly the three versions in the repository requirements;
the complete suite is being rerun there.

Actual private-route attempt received “Serve is not enabled on your tailnet.”
Mac tailnet is different and its peer inventory lacks productionrpc; the browser
requires administrator authentication. Local SSH and remaining remote Serve
process were both stopped; fresh remote status showed no route. A scoped setup
request was posted successfully to approved Slack channel C0C0X2ELFKR,
timestamp1789278370.283259, with a PF-80-S01 hyperlink and exact context. Receipt
is in `.codex-work/bootstrap-link-alert.Znrbsk/receipt.json`. This administrative
alert does not qualify automated decision reply/agent ACK. Only remote-link
qualification requires that human setup; engineering continues independently.

Review02 found four additional in-scope blockers: manager-authored operational
fields could bypass the watchdog; large worker evidence could wedge briefings;
watchdog predicates needed transactional rechecking; receiving-test inputs needed
validation before any merge. Scope classification: all four affect the exact
existing owner boundary and requested failure/restart contract, with no new
product authority, external service, release or other worktree needed. Second
correction rejects unknown action fields, retains full evidence in a digest-keyed
SQLite table while briefing bounded previews, batches pending events, rechecks
watchdog state under its transaction, and preflights every test. 22 coordinator
and12 integration tests pass; candidate1063 total/637 non-test. Review03 checks
this correction; any further findings receive a fresh convergence audit before
patching. Original reviews01/02 and every failure remain preserved.

Convergence audit after review03: three findings remain, all introduced by this
unit and in-scope: bound manager rationales (same oversized-briefing class), permit
owner-proven terminal failure of dispatched/running workers (same restart/resource
contract), and reconcile the common Git marker only from its exact receiving
branch. No expanded feature, policy, service or owner is needed. Third correction
is therefore authorized within the same files and original line ceilings; review04
checks that changed candidate, preserving all prior findings. The second-corrected
full pinned suite passed298 tests in190.438s; that is not third-correction proof.

Review04 left one verified in-scope blocker: a fixed reconciliation event ID
prevented recording that a previously recovered worker later died. The owner
failure transition now uses the mutation revision in its event ID (not merely
the dispatch epoch, which is unchanged on terminal failure), and a found/ACK/dead
regression proves the resource is released with both original events retained.
Integrator explicitly authorizes core review05 for this changed line/regression,
and launcher review02 for the separately returned startup correction. Bootstrap
usage so far is core01–04 plus launcher01; these two additional checks extend
that allowance to seven, without resetting any historical PF80 ledger. Further
substantive findings still require scope/convergence classification.

The full pinned-environment suite passed294 tests in189.916s on the first-corrected
candidate; that receipt does not qualify the second correction. Canonical private
decision `bootstrap-private-route-admin` now exists at feed revision24, preserving
all earlier records. First save rejected an unsupported research evidence path;
it was corrected to the existing sprint path before successful CAS publication
to local state. Remote dashboard publication remains pending.

This is internal control engineering, with reasoned internal-only N/A for GUI
acceptance of these modules. It does NOT qualify the later dashboard/Slack
workflow, independent isolated executor, actual native ACK, real fresh Fable
cycle, live policy initialization, automatic completion/archive/successor
transition, manager-to-integration wiring, recurring operation or full restart
rehearsal. Those remain required under the full five-part goal. Product workers
and old schedules remain paused. The launcher candidate is separate from this
core checkpoint and still requires its own changed-candidate/live qualification.

## Final core checkpoint

Review05 completed successfully with no actionable findings on the final
reconciliation correction. No further opinion review on unchanged core code is
required. The final focused runs pass25 coordinator and13 integration tests;
the preceding full pinned suite passed301 tests in190.008s before the last
one-line reconciliation correction and its new regression. Do not label that
301-test receipt as a full-suite run on the later tree. Final five-file size is
1114 total/644 non-test lines, within the original allocation. The integrator
accepts this bounded internal increment and its reasoned functional N/A only;
all full-loop, live and independent user-facing gates above remain open.
