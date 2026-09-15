# PF-83 harness increment 3 — preparatory client; native execution blocked

## Outcome and authority

**Partial infrastructure increment. No candidate package was produced and no
functional case was executed.** All 380 expanded cases remain `blocked` /
`not_reached`. The release build lane failed during analysis. The native service,
enforced environment, qualified external transport bindings and independent
executor are still absent. This is not a functional or human-test handoff.

Action `pf83-harness-impl-04`; Astra High worker under Fable. Allocation digest
`ce412eb1a2615b51f191449b221ddf425020b1676bcefd07a40b8bb74899fe72`;
claim `5ad574f5-1c40-43d3-897f-59493de98afe`. Read the frozen brief first;
`shasum -a 256` matched
`f460e3bfe2d65da46fc5b5cd10e63d2732381e84b41fff6b72432ca01b64ce69`.

Class: **routine QA infrastructure**, supporting active plan
[p0-security-levels](../../../docs/plans/active/p0-security-levels.md) and
[PF-83-S01](../../../docs/sprints/current/p0-security-levels/pf-83-s01-permission-confirmation.md).
Product heading **Permission selection confirmation — TO BUILD**:
“A submitted selection is not a confirmed change.” Also:
“Existing active-turn approval/sandbox snapshots and pending approvals are not
retroactively changed.” No product behavior, plan or sprint status changed.
Independent functional-design N/A for this internal increment is the worker's
assessment; integrator acceptance is not supplied. The later independent PF-83
execution/evidence gate remains required. No reviews or agents were dispatched.

Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-pf83-harness-20260915`;
branch `bootstrap/pf83-harness-20260915`; base
`c027eb4281f467d5fb7cb49d538bc6358456c919`. This receipt is the only repository
file changed. Private harness root, abbreviated **H** below:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-pf83.20260915`.

Read root policy, the Corbanu development skill, test-isolation guidance, the
[frozen allocation](functional-gate-allocation-20260915.md), increment-1 receipt
including revision 02, and the isolated-execution contract. The requested
`functional-harness-increment2-20260915.md` does not exist in this checkout.
Read its retained predecessor artifacts under `H/increment-03/` and the manager's
[provisioning blocker record](../../initiative-control/management-bootstrap/pf83-isolation-provisioning-20260915.md).
That absence is not an inferred successful increment or approval.

## Pinned source and actual build result

Created a separate shared clone under `H/increment-04/source`, detached at
`e3bd579bf4e0c7c58ad863af2a9c6098e2297f98`, tree
`dd4ea6fc1584a10064aa15b38f50bd7eae35c259`. Commit/tree and clean status were
checked before and after the build. No source file was edited. Source version
is `0.1.42`; no candidate `--version` command ran.

Both attempts ran `/opt/homebrew/bin/just build-for-release` from that clone.
The recipe changes cwd to `codex-rs` and invokes exactly:

```sh
bazel build //codex-rs/cli:release_binaries
```

Exact environment for the second attempt, using **B = H/increment-04** as an
abbreviation here only; the attestation retains all literal absolute values:

```sh
env -i PATH="$B/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin" \
  HOME="$B/build-home" TMPDIR="$B/tmp" TEST_TMPDIR="$B/tmp" \
  BAZELISK_HOME="$B/bazelisk" CARGO_HOME="$B/cargo-home" \
  CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/pf83-gate-20260915 \
  XDG_CACHE_HOME="$B/cache" /opt/homebrew/bin/just build-for-release
```

The explicitly requested Cargo target directory was created; Bazel uses its own
private output/cache directories. No shared Cargo target directory was used.
The first attempt had the same environment without `$B/bin` in PATH. It exited
**2**, before compilation: Bazel expanded workspace `~/.cache` to
`/Users/Neo/.cache/bazel-disk-cache` despite private HOME, then failed with
`File exists` / `Error initializing RemoteModule`. Its logs remain intact.

The second attempt used a retained two-line Bazel wrapper, adding `--nohome_rc`
and `--bazelrc=$B/build.bazelrc`. That additional rc overrides disk, repository
and repository-content caches with absolute private paths. It exited **1**:

```text
configurable attribute "target_triple" in
@@rules_rs++toolchains+default_rust_toolchains//:macos_aarch64_1_95_0_rust_toolchain
doesn't match this configuration
Analysis of target '//codex-rs/cli:release_binaries' failed; build aborted
```

Bazel reported 26.193 seconds and one internal process. Rust compilation did
not begin. No fallback target, source patch, Cargo build/test or older package
was substituted. The owned private Bazel server was shut down successfully.

Tool identities:

| Tool | Observed version / SHA-256 |
| --- | --- |
| Bazel | `9.0.0`; `2c3cce548a4b6a97a2a5267712187b784b52714c4a2b0613e7386b15669d783c` |
| Bazelisk | installed path `Cellar/bazelisk/1.29.0/bin/bazelisk`; `58a54e95d0a4d5b1859441531656b06312667ece5c96934f128ca86d0aec8fe2` |
| just | `1.58.0`; `6be21425c935f30a3a63def350f7e5ef344559c4728a62216374a5b9ff856394` |
| Python | `3.14.4`; `5c3ea934d18a7979253ca08ff16151db78a896be29dbc37fd9bf7e7c549593b8` |
| Rust | source-declared `1.95.0`; compiler was not invoked, so no measured compiler hash |

Host: macOS 26.6.2, build 25G83, arm64. Bazel 9.0.0 and build dependencies were
downloaded into the private build area; this was not an offline build.
The downloaded Bazel path is
`B/bazelisk/downloads/sha256/2c3cce548a4b6a97a2a5267712187b784b52714c4a2b0613e7386b15669d783c/bin/bazel`.
The attestation retains full tool paths, wrapper/rc bytes and build stdout/stderr.
Source-file SHA-256 manifest digest:
`a5b8e09e7211c8c1d443cc64322828555e6b6409fa2f0bd333d573e3875c7c34`.

**Package/binary paths and package/archive hashes: unavailable.** The six-platform
CLI release filegroup also does not invoke the canonical standalone package
builder or assemble the design's required helper/resource inventory. Even a
successful filegroup build would need that packaging step. Signing, archive
stream verification, helper identities and target before/after attestation are
not fulfilled by this increment.

## Implemented mechanics and their limits

- `verify_package` requires an independently frozen manifest digest, matching
  source commit/tree, exact file bytes/modes/inventory and internal symlink
  targets. Missing pins, extra files, changed modes/bytes and escaping links
  refuse. The native client verifies before use and after cleanup. Since the
  real build failed, a real package pin remains unset; the positive measurement
  test used explicitly synthetic non-executable content and never launched it.
- `BoundaryWire` supplies bounded JSON framing over a coordinator-owned Unix
  endpoint. `NativeDriver` supplies a PTY **protocol client**, request/frame
  validation, separate text/Enter, frozen fixture/restart IDs, existing Budget
  accounting, checkpoints, eight recovery actions, raw-byte capture, allowlisted
  artifact hash checks and owned-close requests. `native_dispatch` remains false.
  This is not the environment-side native PTY implementation: controlling-terminal
  launch, real screenshots, marker/process/provider/MCP collection, guest package
  measurement and actual cleanup still require the service. No code-blind actor,
  120-second model transport or continuity-window renewal is wired.
- External fault helpers preserve/pass through or hold/release original replies;
  known rejection and unsupported replies require interception before forwarding.
  Uncertainty withholds the reply. Duplicate and wrong-method requests refuse.
  An MCP `elicitation/create` request helper journals its synthetic request ID.
  These are message helpers, **not running external fault/MCP servers**. Real TUI
  transport routing, disconnect/reorder/barrier scheduling, MCP initialization,
  tool triggers, reviewer receipts and refresh controls remain unimplemented.
- All retained observations are sealed with SHA-256. No screenshot, effect marker,
  kernel denial, native prompt result or functional pass was synthesized.

## One preflight entrypoint

```sh
python3 -B /Volumes/CorbanuDrive/Corbanu/.codex-work/functional-pf83.20260915/run_pf83.py preflight /absolute/environment.json
```

The coordinator-supplied JSON contains `package`, `manifest`,
`manifest_sha256` and `boundary_socket`. The manifest digest must be independently
frozen. An unset example is retained as `B/environment.template.json`.
The command checks the package before opening the endpoint, requests fresh
nonce-bound evidence, retains raw response/report and seals its receipt.

Required inventory is **175 controls**: 27 denials for actor, actor-child, target,
target-child, bridge and bridge-child, plus 13 positives. It covers repository,
history/prior findings, synthetic credential files and Keychain-service denial;
filesystem/symlink/package/host-mount access; process inspection/memory/signals;
PTY, sockets, pipes, Mach/AppleEvents, clipboard and host agent/Docker sockets;
loopback/LAN/DNS/Internet; child escape and actor tools. Positives cover package,
PTY, separate text/Enter, child commands, fixture IPC, actor/target mediated
inference, R/F behavior, transport policy queries and private profile/state.

The parser requires actual-PID/policy/nonce association, syscall/target identity,
paired successful target controls surrounding the attempted denial, and expected
denial errno. Missing, malformed or simulated evidence refuses. ENOENT, dead-port
connection refusal and timeout cannot stand in for denial.

**This validates reports supplied by a future service; it does not independently
establish their truth or implement the probes.** The service and authenticated
connection provenance do not exist. A deployable preflight that actually tests
the provisioned environment is therefore still incomplete. Its server-side
adapter must perform the probes from the actual runtimes/children and provide raw
evidence for independent review. Changing connection details alone is not yet
sufficient to deliver the requested native execution system. No environment was
connected, provisioned, weakened or represented as enforced here.

## Final-tree offline evidence

| Check | Exit / actual result |
| --- | --- |
| `python3 -B checks.py` | 0; 12 tests passed, including altered manifest-hash and missing-pin refusals |
| Offline package/fault/preflight/socket seams | 0; byte/mode/extra-file/escaping-link refusals, message transforms, four preflight refusals and socketpair framing |
| Native client method seam | 0; 56 ordinary actions plus seven recovery observations and blocked finish; 64 calls, 132 sealed files |
| `run_pf83.py preflight increment-04/environment.template.json` | 1; `package pin absent`; no endpoint connection; all 175 controls `UNPROVEN` |

The client method test bypasses constructor initialization solely to test pure
methods against an in-memory response seam. It creates **no admission record**,
native process, actual PTY or functional case. The constructor's real disabled
admission refusal is tested separately. Text/Enter and every observation are
journaled; no synthetic elapsed time is claimed as native execution evidence.
The earlier failed build and earlier check report remain preserved.

Final 12-test report:
`H/checks/attempt-snnjsyxu/report.json`, SHA-256
`f47923ff6efcd814331e16e859c66dbfe9ad3d82429676c32d6c899ee141aede`.
Final attempt seal digests:

- Message/package seams, `B/seams-xy9rbydw`:
  `a9556f542113ad0150ab7027be005dffd3ced0e188c352f392de163542b13c51`.
- Client method seam, `B/driver-seam-ssu12qba`:
  `f69f8896f5db03d49886749d69c54b2817603994964daf624c23cb009860c7e4`.
- Preflight refusal, `H/preflight/attempt-vhkh26s0`:
  `42beda85d909f4eff9dba0e2b1e9ebcb81fe277fc6754ffa31b689a465452570`.

`B/evidence/` contains 34 sealed files: build attestation/logs, source manifest,
exact offline commands, final harness copies/hashes/deltas, report references
and all 380 blocked outcomes. Seal digest:
`36f7b2f57788439ef1e7ad71bce25f5cf4abed85435f492aaca5f58a27bc48ff`.
Worker rehashing proves retained byte integrity, not independent evidence review.

## Changed harness inventory and remaining allocation

| File | Added | Deleted | Final lines | Nonblank |
| --- | ---: | ---: | ---: | ---: |
| `run_pf83.py` | 26 | 4 | 450 | 432 |
| `native_guest.py` | 142 | 2 | 346 | 324 |
| `fixtures.py` | 63 | 2 | 346 | 319 |
| `checks.py` | 9 | 0 | 199 | 181 |
| `cases.json` | 18 | 1 | 33 | 33 |
| `boundary-policy.txt` | 16 | 1 | 35 | 35 |
| `README.md` | 17 | 1 | 100 | 97 |

Seven authored harness files, 1,509 lines / 1,421 nonblank. Build wrapper/rc,
source clone, pinned dependencies and generated evidence are separate inventory.
Frozen original/normalized packets and historical attempts remain unchanged.
The coordinator and README are at their per-file ceilings. Full service/probe,
PTY/collector and external-server implementation needs an integrator scope
amendment/separate allocation; controls were not removed to fit this increment.

Remaining work, including after an environment exists:

1. Resolve the pinned Bazel release-target/toolchain analysis failure in an
   authorized build allocation. Any source change requires an explicit new
   candidate pin. Build the standalone package with required helpers/resources;
   freeze archive/file/link/mode identities and verify archive contents.
2. Provision the manager-owned enforced native environment; this worker does not
   solve or work around that open Travis decision. Implement and qualify its
   actual service adapter, native PTY/collectors, child probes and mediated
   inference against the documented protocol. Run preflight with real paired
   controls under one unchanged policy and independently review raw evidence.
3. Supply real external fault/MCP server bindings and public native navigation,
   startup/held-input/confirmation/stale/restart routes. Freeze literal actions,
   resource/budget values, original design provenance/screenshots and applicability.
   Resolve both live-repository bases/task snapshots and two actual model routes.
4. Allocate the separate fresh-context code-blind executor; execute every frozen
   case/control and accepted profile/platform/repository variant on the exact
   package. Preserve failures and retries; obtain the independent evidence review,
   actual schema-2 validation and sprint evidence reconciliation.

No live profile was used, no native credential prompt was observed, no Rust test
was run, no product case was dispatched and no push was performed. This increment
does not establish package, native isolation, functional, human-test or release
readiness. Required deliverables remain partial as explicitly listed above.
