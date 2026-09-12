# Next bounded broker service substage

Proposal for coordinator allocation, not implementation authority or a completed
feature. PF-27-S04 remains in progress; PF-35 qualification is independent.

## Compose an actual service without a dependency cycle

`codex-vault` depends on `codex-secret-broker` for the typed backend trait.
Putting a Vault-backed service main inside the broker package would add the
reverse edge and create a cycle. Allocate a narrow
`codex-rs/secret-broker-service/` executable composition crate with dependencies
on broker, Vault and security-audit; keep shared Cargo/Bazel registration and
locks under the integration coordinator.

Its first Linux implementation should consume trusted bootstrap socket/key/
journal handles, not worker-controlled JSON claiming a UID or a healthy platform.
The service must run under a dedicated non-login principal, obtain native peer
identity, bind each registered controller/run, and use the externally protected
PF-20 integrity-root adapter. Service death/restart must discard channel keys
and handles, while pending audit intents remain unreplayable.

## Explicit installation/qualification boundary

Creating a service principal, installing a system service, altering ownership
or ACLs, and handing existing Vault material to another principal are distinct
privileged setup/migration actions. Obtain explicit setup authority and record
the exact reversible changes before performing them. Current code tests use
synthetic credentials and same-user test fixtures only; they cannot establish
production isolation or satisfy all ten platform capabilities.

Linux proof needs the actual service and untrusted worker launch path, not just
SO_PEERCRED. macOS still needs authenticated XPC/audit-token/helper containment;
Windows still needs service SID/AppContainer/named-pipe token containment.
Unavailable mechanisms keep protected activation unavailable.

Before adding real service signal handlers, handle `EINTR` explicitly in the
Linux disconnect poll and partial-frame read loops while preserving the one
absolute frame deadline. Fable review 4 identified the current fail-closed
session teardown on interruption as a non-blocking availability limitation.
Add native signal-delivery regression proof with the actual service lifecycle;
do not confuse a benign interrupted syscall with verified peer death.

## Keep provider streaming a separate reviewable step

Current operations carry an exact path and return a secret-free status/counter
receipt. They do not transport a request body or model response stream.
The following bounded step must specify request-body framing/backpressure,
per-request byte/time limits, cancellation across connect/upload/download, and
fresh/cached proxy handlers. Model-visible response bytes require the PF-28
output protection gate before reflecting credential-bearing service output.
Do not solve missing stream protection by returning raw backend responses.

This proposal preserves the current unavailable service default and does not
authorize an interim user-visible protection claim or PF-27 archival.
