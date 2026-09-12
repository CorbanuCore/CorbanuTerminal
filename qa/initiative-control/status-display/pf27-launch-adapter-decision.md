# Approved: isolated Linux launch adapter

[PF-27-S04](../../../docs/sprints/current/p0-security-levels/pf-27-s04-isolated-credential-broker.md)
is no longer waiting for this approval. Decision ID: `pf27-linux-launch-adapter`.
Travis Good explicitly answered “I approve the isolated adapter” in the manager
task; recorded September12 17:23:24UTC, resolved question revision1. Original
question/evidence remain below as history; approval is not runtime qualification.

The existing security owner has been instructed to record the exact allocation,
resume bounded isolated-adapter implementation and run non-root synthetic Linux
tests with independent security review. Travis confirms multiple Linux machines
are available; use already configured test hosts and record the actual OS/kernel/
libc/architecture matrix. The main broker's unsafe-code prohibition stays intact.
No privileged installation, live credentials/protected activation, main push or
release is authorized by this decision. The next steps are owner engineering,
not another question for Travis. Owner acknowledgment/actual execution is tracked
separately in [current handoffs](current-handoffs.md).

## Question and recommendation

May the security owner implement a small, separately audited Linux launch adapter
that confines necessary unsafe Rust/FFI to one isolated crate, while retaining the
main broker service's unsafe-code prohibition and all existing authorization gates?

Recommendation: approve only that bounded implementation and non-root synthetic
OS testing, with independent security reviews before integration. This does not
authorize privileged installation, live protected execution, credentials, release,
or main pushes. A separate exact source/graph/test allocation remains required.

## Why explicit approval was required

The available safe Rust APIs did not provide the required complete combination
of immutable executable binding and stable child-process ownership. The security
owner located a maintained GNU libc operation that appears to provide it, but a
small first-party interoperability wrapper is needed. Rust cannot automatically
verify memory safety inside that wrapper. This is a new explicitly reviewed
unsafe/FFI trust boundary, not just replenishing a review budget; the manager is
not silently treating the general review delegation as approval of this boundary.

The proposed service-facing API accepts only an owned validated image and bounded
fixed recipe. It has no shell/path fallback, arbitrary callback, worker-supplied
command, or numeric-PID signaling substitute. It uses the retained sealed file
through qualified procfs, not a promise to call execveat directly. Main service
`forbid(unsafe_code)` stays intact. Child ownership, errors and cleanup must remain
explicit, including late completion and cooperative cancellation.

## Options and tradeoffs

- Approve the isolated boundary and its bounded non-root implementation/testing.
  This unblocks the next concrete security increment, with added audit obligations.
- Keep the current construction constraint. This launch increment remains held
  until an existing safe supported backend is qualified; no credible date is known.

No option changes credential policy, privilege grants or production activation.
Source qualification is not proof of runtime correctness or leak-free cleanup.

## Evidence and notification state

Owner checkpoint021f82e6c10b2f58e3b67ad5bfdcc1562a4bde0f is a463-line proposal,
not implementation. Parent read the complete118-line new amendment. Matching
Ubuntu libc package-version source, symbols and checksum consistency were found;
the signing chain and reproducible binary correspondence were not independently
established. Runtime failure, cleanup and actual non-root execution proof remain.
The owner proposed adapter+OS-proof followed by owner integration as two coherent
stages, not a mock-only substitute. At that historical checkpoint no new source
or invocation was authorized; the bounded approval above now supersedes that hold.

The answer was received directly in the manager task, recorded in the append-only
decision history and sent to the existing security owner. Updated dashboard
publication follows; Slack is not connected and no Slack delivery is claimed.
