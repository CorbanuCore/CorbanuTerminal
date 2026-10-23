# Adaptive event batches — September 13, 2026

PF-80-S01 internal coordination implementation under **Internal delivery control —
TO BUILD**, “Use sequential sprints per initiative”. Base cfffcd3f0. Parent owns
the four source/test files and the linked allocation; Nash's separate reviewed
owner-transition candidate remains frozen for receiving integration.

Before inference, the driver finds the largest oldest-event prefix that fits the
unchanged 64 KiB and original-evidence limits. A revision-bound owner transaction
restricts the claim once; it neither consumes events nor extends its deadline.
The original claim and selected claim are retained separately. Only acceptance
consumes selected events. Deferred events remain pending in order across restart.
All three streams, recent actions and the selected packet's original evidence
remain present. An oversized first event holds; later small events cannot jump it.

## Verification and review

- Combined command: `env PYTHONPATH=scripts/initiative_control python3 -B -m unittest test_coordinator test_manager_cycle test_native_owner test_integration`.
- 112 tests passed in 8.035 seconds; governance and diff checks passed.
- The initial 32-test run retained one expected-contract mismatch: it expected a
  small earlier event to be held behind a later oversized event. The revised case
  proves the earlier batch progresses, then the oversized first event holds and
  cannot be skipped in favor of a subsequent small event.
- Offline replay of retained failed claim a75bf3e7 selects 18 of its 21 events
  at 65,436 bytes with 47 complete original evidence bodies and no omissions.
  Three events in that retained claim are deferred; this is not a count of the
  entire live queue or an actual manager acceptance.
- Fable review01 in private event-batch-review.QO8hPa found no material code
  issue and one P3 requesting this missing QA record. This record supplies the
  requested test and sizing evidence. Code is unchanged after review; no extra
  unchanged-code review is commissioned. Prior reviews/failures remain retained.

The owner accepts internal-only functional N/A for this SQLite/packet unit.
Independent isolated dashboard/Slack acceptance remains a later required gate.
Live fresh-manager selection, actual receiving integration and complete restart
rehearsal remain unqualified at this source checkpoint. No product sprint,
recurrence, Task Node posting, human acceptance or release is enabled here.

## Actual fresh-manager and receiving handoff

At committed 5b2ac8ea6, manager 145eadfa-ab7f-4bb5-852e-1bd0604ae682 selected
18 events from 25 pending, leaving seven deferred at that claim snapshot. The
65,461-byte briefing retained all selected original evidence; digest
5f3814bea30d5bd72f573ab6387a107a500a2c9991953ea11dc6db28bd209605.
Actual Fable session 01a09ca9-9203-79f1-b556-c3f7bca74135 used
claude-fable-5-1-plan / claude-plan / high and exited cleanly without forced
termination. Receipt digest
c1da444e172a16bf1fdd4ec8a60a19cadb3d83bfbe86fcceb128d6b68415a5c2.
The owner checked that exactly 18 events were consumed by this manager. Later
owner-state events joined the still-pending queue; deferred does not mean deleted.

The accepted proposal actually reached fresh Astra High integration executor
01a09caa-75f6-7580-b4d1-733e6b7ec56a after exact native binding acknowledgment.
Single-writer receiving merged ca0474fe4 into ee93206c0, passed 119 combined
tests in 8.997 seconds and all four assigned checks. The owner rehashed all logs,
checked both merge parents and literal four-file scope, verified the clean tree,
closed the actual executor and accepted its receipt at coordinator revision 253.
Private artifacts remain in transition-receive.qL5nIN; original failed attempts
remain in that directory and tr.dGurO6. This proves a batched live manager-to-
integration handoff, not the remaining full sprint/successor, Slack/private-HTTPS,
independent functional or unattended recurring acceptance gates.
