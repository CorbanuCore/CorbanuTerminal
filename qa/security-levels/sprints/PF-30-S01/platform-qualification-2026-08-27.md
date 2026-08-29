# PF-30-S01 Mac/Linux qualification findings

Historical failed attempt. The subsequent [repairs and retest](platform-fixes.md)
fix these three findings and pass the Mac/Linux backend smoke and negative
confinement checks. Broader platform certification and Windows remain open.

Date: 2026-08-27 Arizona time (2026-08-28 UTC). **Mac and Linux failed the
committed backend smoke; Windows remains network-blocked.** Infrastructure
prerequisites are now working. This supersedes the initial blocked
[preflight](platform-preflight-2026-08-27.md), not the prior test/review history.

Existing product initiative: active `p0-security-levels` plan, PF-30-S01
`in_progress`. Product authority: **Moderate/Aggressive isolation and content
provenance** — “Support Windows, Linux, and macOS with containerized Scrapling”;
“Missing isolation denies the affected acquisition path rather than falling
back to the host browser.” No production code was changed during qualification.

## Exact candidate and execution

- Worktree: `/Users/travisgood/Documents/ChatGPT/corbanu-pf30-s01`;
  branch `codex/pf-30-isolated-runtime`.
- Checkout: `ddfb27b4daed6d022403ae0c4e2d869f1910f02a`;
  code: `322c81b88cbe8e90ba02b16937a4c7b4a792bb1c`.
- All 28 [candidate hashes](candidate-files.sha256) verified on Mac and in the
  Linux source copy exported with `git archive` from the exact checkout commit.
- Linux disposable source/build directory:
  `/home/travis/corbanu-pf30-qualify.0XirOU/source`.
- Rust/Cargo **1.95.0**, as pinned by `codex-rs/rust-toolchain.toml`, on both.
  Linux bootstrap: official static.rust-lang.org rustup **1.28.2**, SHA-256
  `20a06e644b0d9bd2fbdbfd52d42540bdde820ea7df86e92e533c073da0cdd43c`,
  checked against its official `.sha256` before execution. Its Cargo/Rustup
  homes are inside the disposable directory; no shell-profile changes.
- Build/run from each source's `codex-rs`: `cargo run --locked -p
  codex-browser-isolation --example qualify --profile ci-test`; Linux used
  `-j 8`. Mac reused the PF-27 worktree's `codex-rs/target` build cache.
  Linux initially failed for missing OpenSSL development files; after installing
  build prerequisites it compiled successfully. No lockfile change or full
  Core-suite run was made.

| Binary / image | Mac arm64 | Linux amd64 |
| --- | --- | --- |
| `qualify` SHA-256 | `c2288dafd9a7b5b605618620fa3f16802befac71178c7dbdf7bc45fffc527b9b` | `13aef060ac7f2eb7e99500973e41f576829eb90873b7a1e56e4c28169da34e8d` |
| Derivative image ID | `sha256:3b452032df4f536b4845a09cfd94c601af412fd918a90efc4f595d293e0a2ddc` | `e34f1eb2917364d44f5de00921bfd8c6e6686b25a51c98906df950d536079f78` |
| Engine | Docker 28.0.1 / Desktop 4.39.0, LinuxKit 6.10.14 | Rootless Podman 5.7.0, crun 1.21, cgroups v2 |
| Committed runtime smoke | exit **1**, `HealthCheckFailed` | exit **1**, `HealthCheckFailed` |

Both derivatives use the unchanged pinned Scrapling OCI input in
[runtime selection](runtime-selection.md). Recipe hash:
`ce008e92a30f2a390b770edd88a9387cfd434c5e73b682ed1c9ad18ea1cdaeb8`.
The real examples failed before public acquisition/private-URL assertions.
They were rerun directly after the builds with the same failures. Exact command
outputs, inspections and diagnostic code are in
[platform captures](platform-captures-2026-08-27.json).

## Confirmed failures and required repairs

1. **Docker launch arguments are incompatible.** The committed
   `browser-isolation/src/container.rs` sends `--pid=private` and
   `--uts=private`. Docker returns exit 125, first “invalid PID mode,” then
   “invalid UTS mode” when only the PID flag is omitted. Omitting both in a
   diagnostic launch succeeds; inspect reports the private defaults as empty
   `PidMode` and `UTSMode`. The production source was not modified. Repair must
   use engine-correct private namespace arguments and retain verification.
2. **Podman capability inspection is incompatible.** With the exact committed
   launch flags, Podman expands `--cap-drop=ALL` to eleven named capabilities.
   `verify_confinement` requires an `ALL`/`CAP_ALL` element and rejects this real
   inspection even though `/proc/self/status` reports `CapEff=0`. Repair must
   validate effective confinement for each engine, not simply remove the check
   or accept arbitrary capability subsets. Retain regressions from real captures.
3. **Mac syscall filtering is not verified.** Docker reports its global seccomp
   profile as unconfined. In the diagnostic container, `SecurityOpt` contains
   only `no-new-privileges`, while the actual process reports **`Seccomp: 0`**.
   The existing verifier only rejects explicit `unconfined` strings and the
   worker readiness probe reports healthy. This is a gap in the intended
   confinement check, not an observed host escape. Linux reports `Seccomp: 2`.
   Resolve fail-closed capability checks or a pinned per-container filter without
   changing the user's global engine configuration; do not certify the Mac on
   the strength of the current “healthy” worker response.

The unchanged native allowlist/DNS ordering issue and the earlier broker/worker/
endpoint coverage gaps in [review fixes](review-fixes.md) also remain open.
Working infrastructure and clean earlier model review do not supersede these
real-platform findings. Code repairs need final affected tests, Fable High
re-review and another exact-candidate platform run before acceptance.

## Diagnostic support evidence — not candidate certification

Created one explicitly labeled disposable diagnostic container per host with
the derivative image and production confinement flags. **Only the Mac
diagnostic omitted the two rejected namespace flags.** These are direct
engine/worker diagnostics, not successful `BrowserRuntime` acquisitions.

Both packaged Chromium probes returned `{"type":"healthy","version":1}`.
Both showed UID/GID 65532, zero effective capabilities, `NoNewPrivs=1`, a
read-only root (`EROFS` on a synthetic write), no Docker/Podman socket at the
two probed paths, and `ENETUNREACH` for direct TCP to `1.1.1.1`, `10.0.0.1`
and `169.254.169.254` on port 443. Cgroup observations on both: 1 GiB memory,
zero swap, 256 PIDs and `cpu.max=100000 100000`. This is bounded supporting
evidence, not an exhaustive mount/escape/egress or resource-stress test.

Measured engine JSON payloads were below the current 128 KiB limit: Mac info
11,723 bytes, context 789, container 9,146, image 4,742; Linux info 4,894,
container 15,707, image 19,095. Sizes are observations from this run, not bounds
on arbitrary installations. No output-limit change was made.

No end-to-end acquisition, redirects, quarantine promotion, cancellation/crash
recovery or complete PF-26 matrix pass is claimed. S01 has no public TUI path;
S02/S03 retain actual-key setup/integration proof. Live-repository, human
acceptance by Travis Good, benchmark and release gates remain separate and open.

## Authorized host changes and cleanup

After the initial preflight, Travis explicitly authorized sudo authentication
for Linux setup and recovery of the shared Mac engine. Authentication was
entered only at SSH/sudo prompts; no credentials were written to scripts or
evidence. This test-host authorization does not change the product's S03
password-handling requirements.

- Mac: the GUI restart and normal `docker desktop restart --timeout 45` failed
  to stop existing processes. `docker desktop stop --force --timeout 30`, then
  `docker desktop start --timeout 45`, recovered the engine. All three existing
  Ambient containers remained present and restarted. No factory reset, purge,
  upgrade or context/global security configuration change was performed.
- Linux: installed Podman, uidmap, passt, slirp4netns and their dependencies
  (23 new packages, 250 MB installed estimate; no upgrades/removals). Installed
  `libssl-dev 3.5.5-1ubuntu3.4`, `cmake 4.2.3-2ubuntu2` and dependencies
  (five new packages, 75.4 MB estimate; no upgrades/removals). Distribution
  package hooks created the standard netavark service/socket links. Rootless
  Podman uses existing subuid/subgid ranges; no sysctl/security relaxation.
- Pulled/built pinned images on both hosts. Retained those caches and the Linux
  disposable source/toolchain/build directory (about 2.3 GiB) for repair reruns.
  Final observed free space: Mac about **8.5 GiB**, Linux about **164 GiB**.
- Verified exact labels, image identities and IDs before removing only the two
  diagnostic containers. No Corbanu-owned test containers remained; the Mac's
  three unrelated Ambient containers remained running. Recreating the diagnostic
  containers is possible from the retained images; no user data was deleted.
  Invalidated the test sudo authentication timestamp and closed the SSH master.

## Windows hold

Use the host/fingerprint and existing productionrpc route recorded in the
preflight when access is available. DNS/port checks failed from both current
hosts; no private key was copied, tailnet changed or Windows login attempted.
The supplied installed Windows binary is not evidence for this source candidate.
Windows qualification remains required after Mac/Linux repairs and qualification.

## Record validation

Plan and sprint checkers passed (one active plan; 24 current / 86 archived
sprints), as did `git diff --check`, capture JSON consistency checks and the
unchanged candidate's 28 source fingerprints. Strict MkDocs built successfully
to `/tmp/corbanu-pf30-platform-docs.1uVNHm`; existing archive-link informational
notices remain. These documentation checks do not convert failed runtime tests
into passes. Evidence/plan updates are uncommitted; no push was requested here.
