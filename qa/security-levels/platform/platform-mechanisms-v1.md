# PF-27 platform mechanism choices v1

These are bounded design candidates for PF-27-S04/S02, not implemented or
accepted containment. Each row remains blocked until the actual launch path
passes every v1 probe on the weakest supported target.

## Linux

- Broker: a system service under a dedicated non-login UID, never the agent UID.
- Worker: dedicated unprivileged UID plus mount/user/pid/network namespaces,
  `no_new_privs`, seccomp and a bounded cgroup. Landlock may narrow file access
  but is not the sole boundary because kernel/ABI support varies.
- IPC: Unix-domain socket in a broker-owned directory, `SO_PEERCRED` identity,
  then the run-scoped authenticated transcript from the contract.
- Store: broker-only ownership/ACL plus an integrity-chained generation journal;
  rollback resistance needs a privileged monotonic anchor selected by PF-20.
- Network: worker namespace with an explicit egress policy owned outside the
  worker; a proxy environment variable is not enforcement.
- Blockers observed: same-user process/file/config/network/store access succeeds;
  signing has no selected containment role. Current host is ineligible.

## macOS

- Broker: a separately identified launchd service/helper with a dedicated
  principal and narrowly scoped Keychain access group.
- Worker: a signed hardened runtime with an enforceable App Sandbox profile and
  explicit inherited-handle allowlist. If arbitrary shell/tool execution cannot
  be hosted in a supported sandbox mechanism, protected activation is
  unsupported rather than falling back to the host user.
- IPC: authenticated XPC/Mach connection whose audit token, code requirement and
  controller instance are verified before the run-scoped transcript.
- Store: Keychain/helper-owned state plus an integrity-chained generation head;
  ordinary same-user files are rejected as authoritative storage.
- Network: entitlement/profile or privileged broker enforcement bound to the
  worker identity; loopback and alternate transports must be covered.
- Blockers observed: same-user process/file/config/network/store access succeeds;
  code signing alone is not containment and the selected Python runtime exposed
  no peer-credential API. Current host is ineligible.

## Windows

- Broker: a Windows service under a dedicated service SID with no interactive
  logon and a minimal token.
- Worker: AppContainer or a purpose-built restricted token, explicit handle
  inheritance list, Job Object limits, and ACLs denying the worker SID access to
  broker process/state. Merely running unelevated as the same account is not
  containment.
- IPC: broker-owned named pipe; verify server identity at the client and verify
  the client PID/token/AppContainer or restricted SID at the server before the
  run-scoped transcript. Reject impersonation/downgrade and replay.
- Store: service-SID ACL plus DPAPI/service-owned material and an integrity-
  chained generation head; open handles must reject reparse-point substitution,
  rename/delete sharing, stale replacement and rollback.
- Network: AppContainer/firewall/WFP policy bound to the worker identity and all
  transports. Environment proxy settings do not qualify.
- Elevation: explicit UAC/human setup only, with no stored password and a
  post-restart audit.
- Blocker: the authorized target is not currently reachable from this Mac's
  Tailscale tailnet, so all Windows mechanism observations remain untested.

## Selection rule

PF-27-S04 may refine a mechanism but may not weaken the ten required capability
names or the fail-closed result rules. A platform selection is accepted only
after independent review, actual-launch probe success, source/artifact identity,
restart/rollback proof, and a documented unsupported path. An unavailable or
partially supported platform remains visibly ineligible.
