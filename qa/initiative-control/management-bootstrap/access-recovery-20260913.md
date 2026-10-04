# Access recovery and local boundary probe

PF-80-S01 remains in progress; product dispatch and recurring operation remain
paused. Product basis: **Internal delivery control — TO BUILD**, “Use sequential
sprints per initiative”. This is bootstrap engineering, not functional acceptance.

## File access recovered

Travis reported allowing the pending access on September 13. Subsequent reads of
the Corbanu development skill, canonical policy, source and Git state succeeded.
Before that intervention, an owned `cat` process remained in macOS `__open` for
about fourteen minutes. This proves an observed access stall and recovery, not
a diagnosed disk failure. No drive unmount, service restart or permissions rewrite
was performed by the coordinator.

The interrupted status patch did not land; canonical HEAD was clean at
`635b30a453891fa71d961be6b5b8b1e29f330471`. Remote integration still pointed to
`2fc80fa360a45bdc0ad51dfd101ac46c9a92cc83`. Coordinator revision60 remained disabled,
with no manager and security/accounting/delivery all paused.

## Actual cached Docker boundary result

The allocated three-file probe passed on a fresh attempt at 11:09 UTC:
`.codex-work/docker-boundary.WcQbTX/attempt-rcnwtc_i/receipt.json`.
Receipt SHA256: `dd15f8017e1b972f56b7e14b5d893a027066a5ef1330fae04e0dbe166ab362f3`.

Both actual parent and forked child reported UID65534, zero effective capabilities,
NoNewPrivs1 and Seccomp2. Existing host source/auth/prior-result paths and Docker
socket were absent from the namespace (ENOENT); symlink escape also returned
ENOENT. Root write returned EROFS, interpreter write EACCES. Mount/unshare/ptrace
returned EPERM; external connection returned ENETUNREACH. Scratch write/read,
interpreter read and private loopback worked. Effective cgroup values were128MiB,
32PIDs and25000/100000 CPU quota; this is limit inspection, not exhaustion proof.
Guest exit0 and container terminal state were verified; the owned container was
removed. No user container was started, stopped or modified.

Input SHA256 values:

- probe.py: `24c7a68499ae917714b084efa419f09ca0574c42c96e05a139b175351a73fbeb`
- guest.py: `205ed79a214f42bafcd51feca9273eb6083a8f7f77cb9c5568aaf0e71dab1a16`
- seccomp.json: `9e97bb6b924cc6ca0486bf29c2cb30b7b2096715cd23e56a6b3e186928324ad6`

The explicit seccomp policy is default-allow with enumerated denied operations,
not a syscall allowlist. This establishes only cached-Python container feasibility;
cross-run IPC, exact product/browser/PTY controls, mediated reasoning and independent
frozen-case execution remain unqualified. The allocated Fable engineering review
is still required before this boundary is used by a functional executor.

### Delayed first attempt reconciled

The original `attempt-hth1l_8a` failed its15-second Docker create bound during the
access stall; immediate inspection had returned no object. After access recovery,
the exact labelled container appeared in `created` state, never started, PID0:
`5f6981f92e3ec34c812bffae8552f1159086ed82b96a65d1ceb598decc8c9311`.
The coordinator checked its exact name/label and removed that stopped container
by full ID. A subsequent bootstrap-label inventory was empty. Original failed
receipts remain unchanged: client timeout/process cleanup did not prove absence
of a delayed server-side create. Future runner admission must reconcile that case.

## Review and communication checkpoint

Hilbert's unchanged five-file transport candidate retained879 total/535 non-test
lines. Parent reran18 synthetic tests successfully in1.999s; artifacts:
`/private/var/folders/3p/j2_d1_f57vz5d0tj3ycvjg9w0000gn/T/isolated-transport-tests-7rcqhbbk`.
The original Fable material review exited1 after provider-auth helper timeouts,
with no accepted result. Fresh same-model High review02 started in actual session
`01a09a74-c491-7ea2-886c-7ad23ddf8a6d`; final verdict remains pending at this checkpoint.
No synthetic test is relabeled as exact-candidate native OS/live qualification.

The bounded manual Slack collector succeeded after recovery at revision60→60:
`.codex-work/bootstrap-poll-review.JBT5DY/actual-gfx4rxn1/receipt.json`.
One tracked thread, one previously recorded human reply, one duplicate, no new
event. This is not periodic monitoring or complete Slack decision/ACK qualification.
Dashboard health still served generation `build-uv2183uj`, published09:33:13UTC,
with stale decision feed and unknown Slack status explicitly visible.

RPC direct SSH is separate from RTX's tailnet and from private off-Mac browser
qualification; existing coordinator instructions already record that distinction.
No final completion notification has been sent.
