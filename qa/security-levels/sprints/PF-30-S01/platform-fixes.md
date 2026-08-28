# PF-30-S01 real-platform repairs and retest

Date: 2026-08-27 Arizona time (2026-08-28 UTC). Travis requested fixes and
retesting after [Mac/Linux qualification failures](platform-qualification-2026-08-27.md).
The three reported defects are repaired; real backend smoke and negative
confinement checks now pass on **Mac Docker and rootless Linux Podman**.
This is not complete platform, PF-26, sprint or release certification.

## Authority and scope

Existing **product initiative**, active `p0-security-levels` plan, PF-30-S01
`in_progress`. Product: **Moderate/Aggressive isolation and content provenance** —
“Support Windows, Linux, and macOS with containerized Scrapling”; “Missing
isolation denies the affected acquisition path rather than falling back to the
host browser.” Worktree `/Users/travisgood/Documents/ChatGPT/corbanu-pf30-s01`,
branch `codex/pf-30-isolated-runtime`, base checkout
`ddfb27b4daed6d022403ae0c4e2d869f1910f02a` plus the four-file repair patch.

Only `browser-isolation/src/container.rs`, its sibling tests, `worker/worker.py`
and `worker/test_worker.py` changed: 170 additions / 15 deletions. No Core,
network-proxy, provider, persistence, public tool, installer, global engine or
dependency-manifest change. The upstream baseline remains the previously recorded
`413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`; the isolated crate stays behind the
unchanged thin Core adapter. No upstream upgrade is claimed.

## Repairs

- **Docker launch:** omit its invalid `--pid=private`/`--uts=private` options,
  selecting private defaults; retain Podman's explicit options. Inspection
  rejects host/unknown PID, UTS and engine-specific cgroup namespace modes.
- **Podman capabilities:** validate its computed `EffectiveCaps` and
  `BoundingCaps`, accepting null/empty sets but rejecting absent, malformed or
  nonempty sets. Do not infer safety from the expanded `CapDrop` names. A real
  subset-drop diagnostic showed null effective caps but a nonempty bounding
  set, confirming why both are necessary.
- **Actual syscall filtering:** Docker explicitly selects its
  [built-in per-container seccomp profile](https://docs.docker.com/reference/cli/docker/container/run/#security-configuration)
  even when the daemon's default is unconfined. Podman uses
  [default rather than image-selected seccomp policy](https://docs.podman.io/en/latest/markdown/podman-run.1.html#seccomp-policy-policy).
  Before every idle/probe/acquire mode, the fixed worker reads kernel status and
  requires `Seccomp=2`, `NoNewPrivs=1`, all four UID/GID entries 65532 and zero
  inheritable/permitted/effective/bounding/ambient capabilities. Missing or
  malformed status fails closed before Chromium or untrusted input.

These are the selected engine's default syscall policies, **not a newly pinned
or universally audited syscall allowlist**. Actual filter presence is checked;
the existing engine version is preserved and recorded. No daemon security
setting, host policy, privileged workload or unsandboxed fallback was introduced.

## Final source and automated proof

[28-file manifest](platform-fix-candidate.sha256) identifies the exact final
source, verified on Mac, in the isolated review snapshot and in the Linux
disposable source/build directory. Prior `candidate-files.sha256`, JUnit and
failed-platform records remain historical; none was overwritten or relabeled.

From `codex-rs`, using
`CARGO_TARGET_DIR=/Users/travisgood/Documents/ChatGPT/corbanu-pf27-s01/codex-rs/target`:

```text
just fix -p codex-browser-isolation --profile ci-test
just fmt
just test -p codex-browser-isolation -p codex-network-proxy -p codex-core --lib --cargo-profile ci-test -E 'package(codex-browser-isolation) | package(codex-network-proxy) | (package(codex-core) & test(security::))'
```

Fix/lint and formatting passed before final tests. **272 passed**: 20 browser,
223 network-proxy, 29 Core security. **2,306 other Core tests excluded**, not a
complete Core run. Existing Core dead-code warnings remain. [JUnit](platform-fix-focused.junit.xml):
`02cd3c54-a045-48d7-83c2-7c0eb33377cb`, 2026-08-27T20:34:27.474-07:00.

`python3 -B -m unittest discover -s codex-rs/browser-isolation/worker -p
'test_*.py'` passed **six tests on each host**. New tests cover engine-specific
launch arguments, namespace and capability-inspection failures, actual-kernel
state parsing and rejection before every worker mode. No direct `cargo test`
was used. No source changed after final lint/format/test qualification.

## Real-engine retest

Both hosts used repository-pinned Rust/Cargo 1.95.0 and `cargo run --locked -p
codex-browser-isolation --example qualify --profile ci-test` (Linux `-j 8`).
Linux reused `/home/travis/corbanu-pf30-qualify.0XirOU/source` and its isolated
toolchain; only the four changed source files differed from the recorded export.

| Host | Binary SHA-256 | Result |
| --- | --- | --- |
| macOS 15.6.1 arm64, Docker 28.0.1 / Desktop 4.39.0 | `aa23358c43c3eb6df6088b2e37b7578439fe03bfd30cb7d7737a04adef192b5a` | Public example.com fetch (559 bytes), enforcing health and four private/file denials pass; repeated direct binary run also passes |
| Ubuntu 26.04 x86_64, rootless Podman 5.7.0 / crun 1.21 | `911ac3e38898f87b5aaf1040f77d326f119279b17c5ddee7676282215f79954b` | Same smoke assertions pass; repeated direct binary run also passes |

New worker recipe hash:
`bd62a75de80c6ea2ed0fb06c7f1ea9249bad0e1dd2dc170fa7c6ce27b352b564`.
The base Scrapling OCI digest remains unchanged. Built image IDs:

- Mac: `sha256:75e63e06e2a8767c441b06aed760b95aa8d93a51927d5dc4f5f55c6cf1ae1468`.
- Linux: `49f2876f60e5e751d5a6ce7baa2907faa149120b0a91c261f788dad0b6d7ebd0`.

[Raw commands and captures](platform-fix-captures.json) also record real-engine
negative tests. Deliberately disabling seccomp, or adding only
`NET_BIND_SERVICE` back to the otherwise empty capability set, makes each host's
worker return `{"type":"failed"}` with exit 1 **before Chromium starts**.
Those nonzero exits are the expected rejection results, not failed positive
smokes. Diagnostic containers used no network, host mounts or credentials and
were automatically removed. No Corbanu test containers remained afterward.

## Existing three Docker containers

Travis noticed three Scrapling-named containers. Image/ownership inspection
confirmed they are the pre-existing Ambient stack, not three PF-30 workers:

| Container | Role / image |
| --- | --- |
| `ambient-scrapling` | One Scrapling browser service, pinned existing `985d67067bd7…` image |
| `ambient-scrapling-egress` | Egress helper, `ghcr.io/stacklok/toolhive/egress-proxy:latest` |
| `ambient-scrapling-dns` | DNS helper, `dockurr/dnsmasq:latest` |

None has a Corbanu owner label. All three remained running and were not changed,
stopped or removed by these repairs. Corbanu uses its own per-acquisition,
networkless containers and a host-side broker; it does not reuse this stack.
No new runtime, sudo operation or shared-engine restart was needed this turn.
Images/build caches are retained for reruns; Mac free space was about 7.6 GiB.

## Review and remaining gates

Fable 5 High returned **no in-scope findings** after reviewing all four changed
files against the committed baseline, the affected call paths and all 28 hashes.
The [final accessibility capture](fable-platform-fix-review.txt) retains its
structured findings/verdict and follow-ups. Review used the logged-in Claude
app via Computer Use, Manual permissions and read-only isolated source/diff/hash
access. No CLI authentication fallback, nested reviewer or model change. No
review-triggered patch cycle or source change followed the clean verdict.

Platform-matrix follow-up: hosts reporting a non-private cgroup namespace or
missing Podman computed capability fields are intentionally rejected. Check
legacy/cgroups-v1 engine behavior explicitly rather than relaxing these checks
for availability. Default seccomp profile contents remain engine-version
dependent; filter presence is not a pinned-profile certification. The existing
128 KiB engine-JSON output cap still needs measurement, not speculative changes.

The Windows tailnet/SSH blocker, native not-allowlisted DNS ordering, remaining
broker/worker/endpoint coverage and full adversarial/cancel/crash/recovery matrix
remain open. S01 is internal and unactivated, so no new public TUI workflow is
applicable here; S02/S03 own actual-key integration/setup proof. Live-repository,
Travis Good human acceptance, benchmark and release gates are not passed by this
repair. S01 remains `in_progress`; no dependent sprint is activated.

## Record validation and handoff

- `python3 docs/plans/check.py`: pass, one active plan.
- `python3 docs/sprints/check.py`: pass, 24 current / 86 archived.
- `git diff --check`: pass.
- `shasum -a 256 -c qa/security-levels/sprints/PF-30-S01/platform-fix-candidate.sha256`:
  all 28 pass after review and evidence updates; no source drift.
- Existing docs environment `mkdocs build --strict --site-dir
  /tmp/corbanu-pf30-fix-docs.5j1K4g`: pass. Existing archived-link INFO messages
  remain; no new documentation dependency or environment was installed.

The development skill kept these repairs inside S01, required final-tree proof
and preserved the distinction between scoped smoke success and certification.
Autoreview retained Travis's selected Fable High reviewer without substitution.
This handoff is an **uncommitted** repair/evidence patch on the recorded branch;
no commit or push was requested for this turn.
