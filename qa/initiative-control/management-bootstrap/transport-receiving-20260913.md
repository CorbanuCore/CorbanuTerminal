# Fixed transport receiving and next executor gate

PF-80-S01 internal bootstrap increment; product basis **Internal delivery control
— TO BUILD**, “Use sequential sprints per initiative”. Whole-sprint acceptance,
independent functional execution and recurring enablement remain open.

## Reviewed transport integrated

Source `d19d49a0e19aa2f1fbe845444f15517cbcb1b600` was merged by the existing
exclusive-writer Integrator into `da4ebbaa6a779a56c308bdaaa11fa87ea44ec1c1` from
receiving base `1791e7004f2a1575b4b3f659c36f41413a68635e`. Exact five-file scope,
approval, merge and six receiving commands are retained in private receipt
`.codex-work/bootstrap-core-review.ZmRFnr/receiving/fixed-transport-d19d49a0e.json`.
Status verified; no conflict, timeout or unclean receiving state. No main push.

Receiving tests:19 normal in2.147s,19 optimized in2.105s,43 coordinator in1.265s,
13 integration in3.509s:94 actual test executions, all passing. Both governance
checkers pass,3active plans/115current and126archived sprints. The receipt hashes
each log; elapsed command time is separate from test-runner timing.

Fable review04 completed clean, no findings, after the sole Unicode parser defect
and stale historical QA wording were corrected. Original failed review, reproduced
three regression failures and prior tests remain retained. Exact final candidate,
scope,19-test proofs and successful native Astra High connection are in
[the extraction record](isolated-model-transport.md). No fresh negative OS proof,
independent functional acceptance, full Slack ACK chain or release is inferred.

## Local container correction

The initial cached-Python probe and delayed-create recovery are retained in
[the access recovery record](access-recovery-20260913.md). Fable review01 found
two actionable gaps: O_CREAT root-write should require EROFS, not accept ordinary
DAC denial; denying unshare/setns did not prevent clone-created namespaces.

Parent tightened both within the same three-file allocation. Strengthened guest
probes before the profile correction genuinely created a user namespace from both
parent and child; unexpected children exited immediately and were reaped. clone3
returned EINVAL. Keep failing `attempt-rlpa_84a`, with verified container removal.

Corrected profile denies each of seven namespace clone flags with separate
MASKED_EQ(flag,flag) rules, leaving ordinary child creation usable. clone3 returns
ENOSYS to retain libc fallback. The previous review's suggested masked-zero DENY
would reverse the intended condition and was not applied. First-party references:
[Moby seccomp profile](https://github.com/moby/profiles/blob/main/seccomp/default.json)
and [Linux generic syscall definitions](https://github.com/torvalds/linux/blob/master/include/uapi/asm-generic/unistd.h).
This probe explicitly requires the pinned AArch64 image, not portable syscall IDs.

Actual corrected `attempt-9vex119u` passed: all seven clone flags EPERM, clone3
ENOSYS, root EROFS, existing denial/positive controls and actual child exit0.
Container cleanup verified; no labelled bootstrap container remained. Raw receipt
SHA256 `55d32fd4354d8688555b056ad6387e9503c2767f10e39c04c1fadd66e0f68d6f`.
Private three-file root `.codex-work/docker-boundary.WcQbTX/`,225 authored lines:

- probe.py: `56b1ed0841be5939267ee8716362dcd71fbfe43dd815fa4c6b8dd4aa40a20efe`
- guest.py: `596d3acd0ab2386183ec8bc4563e7698bc0d783b40eccb3c29888ebe2d2d4879`
- seccomp.json: `64679f517b26ef7788ca9759aa773e4c20e1fed74bf3ccb89e7e26dde5e7f61f`

Fable corrective review02 completed clean in `.codex-work/docker-boundary-review.ZOMfwf/`,
session01a09a7f-fd7e-7841-a1aa-c616235bf0b6. Parent accepts this bounded diagnostic, not an executor.
The delayed-server-create P3 remains an explicit owner reconciliation requirement,
not a successful cleanup claim. Another finite poll cannot prove an arbitrarily
delayed daemon action absent. An unattended runner must durably gate on unresolved
creation and reconcile exact identities before retry; the diagnostic is not that runner.

## Publication and next actions

Actual Luna Extra High Tesla01a09a78-28aa-78b1-826f-78cf489106d7 ran one source sync
and closed. Published source1791e7004, generationbuild-1si9uh92,11:15:50UTC,
tree digest69aa7ce061fdc2bae812e86c3298fd0205da5f7b27fed6f633cc1f75c887a84c,
18references. Parent independently verified health and FacilitiesHTTP200/header.
Worker verified timerdisabled/inactive and webenabled/active; tunnel preserved.
That publication preceded the transport merge and is not its publication receipt.

Next manager-owned gate is exact-candidate isolation requalification and a bounded
mediated action executor with real PTY/browser positives, cross-run denials and
independent frozen-case execution. Then complete actual Slack decision/native ACK
and restart, private off-Mac access, supervised integration/dependency handoff and
the full three-stream rehearsal. No product successor or recurring operation is
enabled by these engineering passes. No final Slack completion notification sent.
