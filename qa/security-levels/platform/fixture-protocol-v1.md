# PF-27 platform fixture protocol v1

Version: `corbanu.platform-probe/v1`

The controller creates random synthetic canaries in a fresh private temporary
directory. A separately spawned worker receives only an operation code and a
synthetic path, descriptor number, process identifier, or mapped canary address.
It never receives a
credential, vault label, provider token, workspace path, username, hostname, or
network destination outside loopback.

The worker probes these boundaries:

1. non-mutating query access to the controller process (POSIX signal-zero or a
   Windows query-only process handle);
2. same-user read access to a controller file;
3. same-user mutation of a controller config fixture;
4. accidental descriptor inheritance with `close_fds` enabled;
5. local IPC peer-credential observation;
6. worker loopback connection without a worker-specific network policy;
7. process-memory/debug access to the controller;
8. executable signing/entitlement observation without treating a signature as
   containment;
9. non-interactive elevation;
10. read/delete/rename/symlink/rollback attacks against protected-store
    fixtures.

The inherited-handle check first proves discrimination with an intentionally
inherited canary descriptor, then requires the normal `close_fds` launch to hide
that exact device/inode. The IPC check binds the accepted peer to the actual
spawned worker PID, current UID, and a controller-generated random canary; peer
self-report alone is never verification.

Every worker response is bounded JSON. Exceptions are converted to stable detail
codes; filesystem paths and exception strings are not evidence. Fixture cleanup
is mandatory. Preparation creates no socket that survives the run, no runtime
route, no host-wide policy, no administrator change, and no secret resolver.

`supported` means the named mechanism was positively verified for this exact
probe. `unsupported` includes an observed bypass or a platform with no selected
mechanism. `untested` includes unavailable APIs and probe errors. Consumers reject
unknown versions, duplicate/missing capabilities, stale/future results, target
identity mismatch, probe-binary identity mismatch, and any eligibility claim
containing a non-supported result. Status/observation pairs are also bound:
supported requires a denial or verification, unsupported requires an allowed or
not-applicable observation, and untested requires not-tested or probe-error.
`--validate` binds a result to the current
machine/boot and exact probe binary. `--validate-evidence` verifies an archival
result's shape, timestamp lifetime, capability semantics, and exact probe binary
against the real current clock without requiring archival freshness or
authorizing it for the current target.

The generic preparation fixture intentionally records signing/entitlement
containment as unsupported: a signature alone is not isolation, and this sprint
has no actual broker/worker runtime to inspect. Signing/entitlements remains a
required activation capability. The all-supported semantic self-test exercises
the eligible validator path; PF-27-S04/S02 must add and verify a real enforced
runtime predicate rather than remove or bypass this gate.
