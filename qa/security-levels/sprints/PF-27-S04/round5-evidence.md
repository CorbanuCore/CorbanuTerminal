# PF-27-S04 round-five integration stage

This is an unfinished product initiative, not protected-mode availability.
Product citation: **Non-negotiable controls** — “Permit agents to reference
credentials only by label; resolve them solely inside the trusted execution
boundary.” The current sprint and active plan govern the allocation.

## Provenance and frozen scope

- Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/security-round5-broker`.
- Branch: `feat/security-round5-broker`; source base `07791288b`.
- Allocation: `4f263ca73`; recovered leaf checkpoint: `90ae3a0cf`.
- Recovery source: `recovery/p0-security-isolated-broker-2026-09-02`,
  `cdb821289`. Only assigned code/evidence leaves were recovered; allocation,
  shared registrations and other sprint records were not overwritten.
- New stage: `103d5a106`, `9b90047ef`, `6981d8367`; final formatted source
  `d5b01a144`.
- Intended behavior: digest-bound durable dispatch audit and bounded native
  Linux socket/session primitives. No configuration, activation, UI, generic
  secret resolver, platform installer, or shared security-event schema change.

## Implemented stage

- A PF-41 `ReferenceJournal` adapter checks the exact controller/session/task/
  run/generation/reference/operation before reserving a durable dispatch permit.
  It binds the complete original authorization and exact operation/path into a
  digest. Shared events retain PF-41's minimized schema; raw path, destination
  and purpose are not copied into that journal. This supersedes the historical
  handoff's suggestion to persist a raw path in the shared event schema.
- Recovery, protected root and live authorization are external service
  obligations; the adapter cannot create authority or silently recover an
  ambiguous dispatch. A failed/ambiguous commit cannot produce a permit.
- Linux socket peers come from actual `SO_PEERCRED` observations, not frame
  fields. Frames/receipts have allocation bounds and I/O deadlines. EOF,
  malformed input, denial and timeout close the registered generation.
- Client close interrupts in-flight reads/writes without waiting for their
  mutex, and subsequent calls cannot replay on the closed connection.
- A registered runtime connection delegates every frame through the existing
  generation and revocation checks. A fresh same-run generation cannot grant
  its credentials to an old retained connection.

## Executed scoped checks

Execution was remote-only on the authorized RTX Linux host, under
`/home/travis/worktrees/security-round5-broker`. Shared target/cache builds were
serialized with `/home/travis/security-round5/locks/build.lock` and eight Cargo
jobs. No local compilation occurred.

| Check | Observed result |
| --- | --- |
| `just fix -p codex-secret-broker` | Passed |
| `just fmt` | Passed after installing repository-CI-pinned `uv==0.11.3` into a lane-owned remote venv |
| `just test -p codex-secret-broker` | Final post-format run: 42 passed, 0 skipped |
| Native child-service death | Actual fixture subprocess killed after connection; client returned unknown and refused replay |
| Native replacement/revoke | Existing runtime over real Unix sockets: old connection denied after generation replacement, new connection works, revoke denies next dispatch |
| PF-41 persistence | Durable-before-permit, minimized record, wrong binding, ambiguous root, pending restart and terminal outcomes passed |

Development nextest run: `85e821df-db1d-4a75-9dc5-40f12a845c36`.
Raw log: `/home/travis/security-round5/evidence/broker/development-tests.log`.
Final nextest run: `528f049c-1caf-47f4-bc61-602e622968ba`.
Final log: `/home/travis/security-round5/evidence/broker/final-check.log`;
SHA-256 `37ea5ae60587d5c25b65b6bd5cdc0c2f9557fb6e6e02898548a4643ec60ff1cf`.
Remote formatter changes were imported with `apply_patch` and committed as
`d5b01a144`; the source tree is the tree that passed the final tests.

The exact-source broker binary was built remotely and copied while holding the
build lock to
`/home/travis/security-round5/evidence/broker/codex-secret-broker-d5b01a144`.
SHA-256 `f6da5f93a83ddd81a5f5b531700d312ea82699f947540ea6b2323b802400dfc0`.
Direct launch exited **78**, displaying “qualified OS service launch required”,
as required for this unqualified host/service path.

All credentials in these fixtures are synthetic. Unit fixture authorization
reports are explicitly synthetic and cannot count as platform qualification.

## Independent review and remediation

Four of the maximum five review invocations have been consumed:

1. Astra High via Codex CLI 0.145.0 failed before inference because the backend
   required a newer CLI. No verdict exists for that attempt.
2. Astra High via the app-bundled CLI 0.153.1 completed against
   `4f263ca73..14b1aa73a`, finding one P1 and two P2s. All were verified against
   real paths and fixed in `165ae1534`: observe socket disconnect concurrently
   with dispatch, reject unresolvable mandate bindings, and require exact grant
   correlation. The post-format suite passed **44/44**.
3. Fable 5.1 High through Corbanu Terminal 0.1.38 `exec` inside private TMUX
   reviewed `4f263ca73..bebc2abb3`, including the shared registrations. It
   independently confirmed all three Astra fixes and found the pending shared
   Cargo/Bazel lock update plus an overbroad five-second socket read timeout.
   The timeout defect was fixed in `b6941d82e`: healthy idle/first-receipt waits
   remain open, but a partial prefix/body has one absolute bounded deadline.
   Final post-format tests pass **47/47**, including >5s idle/backend waits and
   partial-prefix timeout. The coordinator supplied exact Cargo lock parity in
   `e93f3e37f`; the lane Bazel lock check subsequently passed without a delta.
4. Fable 5.1 High reviewed the frozen `4f263ca73..e93f3e37f` diff through the
   same private-TMUX route. It confirmed all prior repairs and lock parity,
   reporting “patch is correct” with one explicitly non-blocking P3: interrupted
   Linux `poll`/raw `read` calls currently close the session rather than retry
   `EINTR`. This is a fail-closed availability limitation, not credential
   disclosure. The helper exited **1**, not a finding-free success. The
   coordinator approved deferral to the next real-service substage, with signal-handling tests;
   no fifth review is spent merely to obtain a cleaner summary.

Fable's suggestion that same-run generation increments alone consume additional
tracked-run map entries was rejected: the map replaces the generation under the
same run key. The concrete idle/response timeout defect was accepted and fixed;
the bounded run-history policy did not change.

The native disconnect regression pauses a synthetic backend before its final
cooperative fence check, signals full and half client closure, waits for actual
session cancellation, and only then releases the backend. It proves a live
cancellation fence, not interruption of arbitrary blocking syscalls or already
sent external effects. No production HTTP upload implementation is claimed.

Review outputs are preserved in `round5-astra-2.{txt,json}`,
`round5-fable-3.{txt,json}` and `round5-fable-4.{txt,json}`. Actual external route: structured helper through
the Corbanu Claude Plan wrapper, model `claude-fable-5-1-plan`, high effort,
private TMUX socket `corbanu-broker-review`, session `fable-high`. This is a
review execution route, not an interactive product TUI test.

After shared registrations, affected fix/format passed; full
broker/Vault/network-proxy suites passed **335/335**, and focused Core
broker-client/config tests passed **6/6** (2367 intentionally filtered out).
Nextest IDs: `26d4f1d2-f2fe-4bca-82ca-03fc57a78df1` and
`fabb33cc-cffb-43bf-b3d5-80f42fd46f96`. Log:
`/home/travis/security-round5/evidence/broker/integrated-check.log`, SHA-256
`560fc406c85bf1c7379868ade09fb14eb76673a8f922c25c548da2d0593d80b6`.
Only subsequent production change is the scoped timeout correction, whose
47-test run is `9a971fd5-12ba-4fc9-b098-bba1ed5505c5`; log
`/home/travis/security-round5/evidence/broker/fable-remediation-tests.log`,
SHA-256 `7d93595a6c5372bdd3d67599a14ffcc4ed1344a121601117211f31c07d083290`.

## Final post-timeout and lock-parity proof

At `e93f3e37f`, affected `just fix` and `just fmt` passed, followed by **338/338**
broker/Vault/network-proxy tests, **6/6** focused Core adapter/config tests
(2367 intentionally filtered out), and `just bazel-lock-check`. Bazel exited
successfully with no `MODULE.bazel.lock` delta; it emitted existing root-versus-
resolved `platforms` and `rules_cc` version warnings. Cargo lock remained clean.
The only formatter output was the alphabetical Core module registration order,
imported exactly with coordinator authorization; it matches the tested tree.

- Full suite nextest: `98a7153c-9a3c-47a5-9125-3d49eca7bff4`.
- Core nextest: `bd172c95-e6e7-43f3-99ab-423d20f4d2c1`.
- Full suite log: `/home/travis/security-round5/evidence/broker/post-timeout-final.log`,
  SHA-256 `294bb7e0fec17c631d7da6201a305fd81325086e2ff8644bf804182b2af1cc8c`.
- Core/Bazel log: `/home/travis/security-round5/evidence/broker/post-timeout-core-bazel.log`,
  SHA-256 `32bbf6fca6f23afdbdfbf8ccb3a23677159f0b8c0be6a0293b37b2584fc5742e`.
- Fresh per-run temporary directories were `final-tmp.LtHRNT` and
  `core-final-tmp.zF8cn9` beneath the broker evidence root, not the previously
  contaminated shared temporary directory. Existing evidence was preserved.

The first Core compile reused a stale cross-worktree network-proxy artifact and
reported missing exports that were present in the source. Updating only the
lane's `network-proxy/src/lib.rs` mtime under the build lock forced recompilation
and all six tests passed. No source fix or cache deletion was needed. Future
parallel scheduling should separate workspace-package caches by lane rather
than relying solely on serialization. The failed first compile remains in the
full log; it is not presented as a passing invocation.

The persistent Bazel server inherited the build-lock file descriptor after its
successful dependency check. Normal lane-local `bazel shutdown` released it;
no process was force-killed. Future serialized Bazel scripts must shut down
their persistent server before releasing the surrounding lock.

## Remaining gates

- Deferred non-blocking signal-interruption follow-up from review 4; no
  finding-free helper exit is claimed.
- Supporting candidate TMUX workflow and combined-tree affected suites.
- Real dedicated-UID Linux service provisioning, macOS authenticated XPC/helper
  isolation, Windows service SID/AppContainer/named-pipe token isolation.
- Real broker-side provider request/response streaming data plane and native
  cached TLS-handler/upload/concurrent-revoke proof against the actual service.
- All-OS qualification, both applicable live repositories, PF-26 final
  candidate evidence, human acceptance and due benchmark evidence.

The service binary still refuses unqualified launch. This stage does not
enable Moderate/Aggressive, finish secretless agent launches, or complete PF-27.
