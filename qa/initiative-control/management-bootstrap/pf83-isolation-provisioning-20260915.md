# PF-83 code-blind executor: isolation provisioning is blocked

The last open item on [PF-83-S01](../../../docs/sprints/current/p0-security-levels/pf-83-s01-permission-confirmation.md)
is an independent code-blind functional execution. Its
[frozen gate allocation](../../reliability/live-permission-transition-20260913/functional-gate-allocation-20260915.md)
and the [isolated execution contract](../../code-blind-functional/isolated-execution.md)
both require a genuinely enforced boundary. The contract is explicit that a fresh
agent with host tools, a new working directory, a container sharing host
credentials, or a "do not read source" instruction in a prompt do **not** qualify.

Harness increment 1 is built and received. Increment 2 stopped, correctly, on two
blockers.

## Blocker A — build lane (manager-solvable, no human needed)

The worker attempted the pinned candidate package through the guarded test
wrapper (`just test --cargo-profile dev-small --no-run`), which exits 2 before
Cargo launches. The repository already provides the correct lane:
`just build-for-release` → `bazel build //codex-rs/cli:release_binaries`.
The next increment is redirected there. No decision required.

## Blocker B — enforced macOS isolation (needs Travis)

The gate requires a **disposable macOS VM, or a dedicated synthetic test account
with independently enforced OS filesystem / process / IPC / network / Keychain
isolation**. A fresh account or a Seatbelt policy alone is explicitly insufficient.

The manager cannot provision either: this host has no passwordless `sudo`, so it
cannot create an account, configure a VM, or install a virtualization service.
`UTM.app`, `VMware Fusion.app` and `VBoxManage` are present but unconfigured.
An existing second account (`travis2`) is a personal account, not a synthetic
test identity, and using it would be a human decision regardless.

Running the executor on this host without that boundary would produce evidence
that the contract already rejects, so nothing is executed and all 380 expanded
cases remain `blocked` / `not_reached`.

## What is unaffected

Accounting and Task Node continue. Within PF-83 itself the harness, packet
generation, budget accounting, classifier, artifact sealing and the design
manifest are all built and reviewed; only the execution environment is missing.
