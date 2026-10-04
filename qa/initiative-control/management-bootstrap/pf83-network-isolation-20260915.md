# PF-83 executor: the preflight verdict, and the one control still missing

The harness now drives the provisioned macOS guest for real and the authoritative
preflight has run. Result: **2 proven positive observations, 173 of 175 controls
UNPROVEN**. No case was executed; all 380 verdicts remain `not_reached`.

That is the harness working correctly. It refuses to claim isolation it cannot
demonstrate, rather than running cases and calling the evidence good.

## What is already right

- Genuine macOS 26.2 arm64 Apple Virtualization guest.
- Clean `agent` account: uid 503, not in `admin`, no `sudo`, empty home, no
  credential files, no Corbanu source, no `Codex Auth` keychain item.
- Key-authenticated SSH; the manager private key never left this host.
- `BoundaryWire` carries its original framing, deadlines and checks unchanged
  over SSH stdin/stdout — no case logic was rewritten to fit the transport.
- Pinned package built by the proven lane: `cargo build --release -p codex-cli
  --bin corbanu`, rustc/cargo 1.95.0 aarch64-apple-darwin, guest reports
  `corbanu 0.1.42`, binary SHA-256 `3e99a6cb…`, manifest SHA-256 `85889028…`.
  A mismatched manifest hash is refused.
- `tmux 3.7c` passed the guest PTY probe; a Python PTY fallback exists unused.
- 17 offline checks pass; the 64-file evidence bundle verifies.

## The blocking control

**Network egress is unrestricted.** Both the bridge process and its child
reached `https://github.com` with HTTP 200 and connected to the host at
`192.168.64.1:22`. The contract requires network denied except exact approved
test services and mediated inference, so while egress is open, essentially every
dependent control stays `UNPROVEN` — which is why 173 of 175 are.

This cannot be fixed from inside the account: `pf` needs root, and the executor
account deliberately has no `sudo`. It is a host- or VM-level change.

## Options

1. **Isolated VM network.** Switch the guest to an isolated/internal network so
   it has no internet route and no host route, then open one pinhole to the
   mediator the executor is allowed to use. Cleanest fit to the contract.
2. **Host-side packet filter.** Keep the current network and enforce the denial
   on the host with `pf`, scoped to the guest address.
3. **Guest-side firewall.** Requires root on the guest, i.e. a privileged helper
   the executor cannot reach.

The mediated-inference pinhole matters in every option: the executor needs model
access, and that access must be mediated rather than arbitrary, or it becomes a
general network path again.

## Not blocking, but open

CLI-only package — standalone helper/resource qualification is still open. Actor
and target probes, frozen fixtures and cases, a preflight rerun, independent
review, and the separate manager execution decision all remain before any case
runs.
