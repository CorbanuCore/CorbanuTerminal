# Disposable full lifecycle rehearsal — September 13

PF-80-S01, **Internal delivery control — TO BUILD**, “Use sequential sprints per
initiative”. This is internal operational qualification, not completion of a real
product sprint or independent dashboard/Slack functional acceptance.

## Boundary and review

Parent uses integrated source db8afbba3 and two private operator scripts under
`/private/tmp/life.TuWkuY/`. Three synthetic lanes preserve the three-reservation
model; only synthetic delivery is enabled. The disposable Git repository has no
remote. Production coordinator remains revision253, disabled with all lanes paused.
Private Fable state/authentication and raw logs are not exported.
Operator SHA256: owner.py f14b47e25f5fb38674676ab6b531869f003fd7dba6dec9bd26953b866ed05874;
receive.py 8f86ea78b273fac7ee17e2dd7c85e7c4a52c759ddceb68157c4de66983201ffe.

Material Fable High review01 found one P2: the script could exit successfully on
an empty/held manager result or unexpected proposed actions. Three added lines
retain the raw outcome but reject everything except accepted/exact requested ID.
Four bad-status and three wrong-action checks reject; both positive controls pass.
Corrective review02 is clean. Private operator commits77e4484 and886cfd3 preserve
the initial candidate and correction. Earlier reviews are not reset.

## Actual completed rehearsal

- Two initial fresh-process denials: premature archival and duplicate initialization,
  both with unchanged coordinator snapshot.
- Fresh Fable manager cdae90ac-a4e5-4ea4-8a7a-293be63ab818 prepared the exact
  integration action. Real Astra High Cicero01a09cc3-8217-76f1-ad73-dc0f5f91913f
  acknowledged the actual claim before START-WORK.
- The executor invoked the fixed Integrator once (handlecc5e64, exit0), merging
  candidate8d346452f77df3d398f42fbe0c03a296987b5e4d into
  receiving421f3a1a5140e43144510d01c71f00deec9f7ec2. Exact parents, clean branch,
  one successful marker check, log hashes and absent writer marker verified.
  Parent recorded the actual return, closed the worker and observed not_found,
  then independently accepted the receipt.
- Fresh Fable manager cbe36350-9758-4199-b02f-38aa3e9b7c3a prepared completion.
  Actual owner operation then completed synthetic REHEARSAL-S01 at revision15.
  Stale revision, missing gate, premature archive and duplicate completion rejected.
- Preserved failed attempt: asking for a successor manager before the completion
  event returned empty, with no manager launch. The corrected script exited1.
  Completion subsequently supplied the real event for a fresh successor proposal;
  the original empty result/context remain untouched.

- Fresh Fable manager b64094a9-e475-4458-8114-d442bbbcf830 prepared the successor
  after the completion event. Activation without separate authority and activation
  before archival both rejected. Parent actually moved the fixture Markdown from
  current to archive, changed its status to completed, added receiving evidence
  and committed it asdeb7635. Only then did the fresh-process archive and separately
  authorized activation succeed. Duplicate archive/activation rejected.
- Fresh Fable manager96a78791-c8d6-4247-84af-b4d0872774d3 prepared the first successor
  assignment. Fresh Astra High Russell01a09cc7-8a11-76c2-b163-54fcb952d8db acknowledged
  its exact binding before START-WORK. Its actual check (handlea06afb, exit0) ran
  from a separate successor worktree at verified receiving421f3a1a, marker ready,
  clean tree. Parent independently compared the result/base/marker and actual
  shutdown/not_found before accepting. Initial incomplete binding responses were
  not accepted; only the complete claim-bound acknowledgment was used.
- Final synthetic state revision29: S01 completed and archived; S02 selected with
  its first work accepted; all three lanes paused, global disabled, manager null.
  Every CLI operation reopened SQLite in a fresh process. Original failed/denied
  attempts and raw manager/native receipts are retained, not relabelled as passes.
  The retained CLI ledger contains79 successful operations (including snapshots)
  and11 expected denials; these are operation counts, not79 distinct tests.

All four actual manager receipts show claude-fable-5-1-plan / claude-plan / high,
distinct fresh sessions and clean, non-forced shutdown with session gone and no
cleanup errors. Session suffixes (full IDs in retained validated receipts):
01a09cc3-22af-7dc3-94ec-ec2c528e437c,
01a09cc5-0fe0-7682-b07b-dd8f0201593a,
01a09cc6-0c79-7232-ae65-ae02235c24f2,
01a09cc6-e19d-7b20-98d3-ab37e818db3b.

This closes the operator-driven full completion/archive/successor rehearsal gap.
It does not qualify an unattended controller. The earlier actual native
crash-window/watchdog/restart proof remains in the [manager-cycle record](manager-cycle.md),
not a new crash injected in this run. Actual human Slack reply/native acknowledgment,
private HTTPS, isolated functional execution and recurring enablement remain open.

## External prerequisites observed in parallel

Direct RPC SSH works. A bounded Serve attempt exited124 after reporting Serve
not enabled; subsequent Serve/Funnel configurations remain empty. The existing
browser authorization tab still displays administrator sign-in. No public route,
ACL change, relay activation or security bypass occurred.

Slack's actual listener remains connected. Same-session authentication renewed at
21:47:27UTC; the last drain found no ACK-only test reply. The supported journal
projection was refreshed at21:55:26UTC to last-verified, sent test alert, zero replies
and zero agent acknowledgments. This is projection input, not a new publication
or complete two-way qualification. At that observation the published feed was stale.

At22:04UTC parent reassessed both open questions from the actual SSH/browser and
Slack observations, saved feed29 through compare-and-swap, and refreshed the Slack
projection. Every original question revision and history was preserved. The first
attempt used the report clock's offset-format timestamp; validation rejected it
before intent/state mutation. The corrected documented UTC-Z format succeeded.
New publication is separately verified; freshness never implies human approval.
