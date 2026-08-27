# PF-30-S01 runtime selection and preflight

Date: 2026-08-27. Owner: Jim Ricketts. Product authority: Travis Good.
Status: implementation inputs selected; **no platform isolation certification**.
Plan: `docs/plans/active/p0-security-levels.md`, **Browser runtime lifecycle decision**.
Product: **Moderate/Aggressive isolation and content provenance** — “Support
Windows, Linux, and macOS with containerized Scrapling.”

## Candidate and boundaries

- Worktree: `/Users/travisgood/Documents/ChatGPT/corbanu-pf30-s01`.
- Branch: `codex/pf-30-isolated-runtime`.
- Base: `9fc9c9106c8afd38aff48d0e5ad4a5f2552b723c`; PF-26-S01 pushed and clean.
- Upstream: `openai/codex`, `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`, inherited
  through the plan's verified fork merge; no upstream upgrade in this stage.
- Product code: existing registered `core/src/security/browser_isolation/` and
  `network-proxy/src/browser_policy.rs`. No new Cargo/Bazel dependencies or
  shared registrations in the first stage. Expand scope before needing them.
- Core is only the native host-runtime adapter seam; content normalization and
  deterministic protected-action authorization stay with PF-29/PF-23. Runtime
  lifecycle must not be inserted into generic tools or a second policy engine.
- Exact planned checks: `just test -p codex-core --lib security::browser_isolation`,
  `just test -p codex-network-proxy browser_policy`, plus real backend probes.
  PF-27 `independent-health` and `cancel-resume-incarnation` contracts must pass
  when lifecycle integration lands. Current policy scaffolding is not that proof.
- Upgrade disposition: retain isolated modules; revalidate the thin native
  health/cancellation/facade adapters at integration. No provider, persistence,
  history or model-wire change in the first internal stage.

## Pinned acquisition input

Queried GitHub release/tag and GHCR manifests/config via HTTPS on 2026-08-27:

| Input | Immutable identity |
| --- | --- |
| Scrapling source v0.4.15 | `333fa22b7a5821194ce66b59b11f4b16a6484f02` |
| OCI image | `ghcr.io/d4vinci/scrapling@sha256:1bacbc8ec90b3090d462e12f6555e241daf0dfeb684ab326ffa09d52d8226e69` |
| linux/amd64 manifest | `sha256:65fc31598af6cc5053c63b5f5205524a94964d82fd33b24b6c96c7cb0be35ada` |
| linux/arm64 manifest | `sha256:1a5e7c7a08b6a892b793cd5c596a3bde5184f9a7c1bd74efb846cf5550377f13` |
| arm64 configuration | `sha256:1ca734b07ddef61f6bc75175baaf112f822857e2ebd5c3715677b0b7dc3dbdb1` |

The registry tag is `0.4.15` (not `v0.4.15`); tags were used only to resolve the
digest. ARM64 config labels match version and source revision. ARM64 compressed
layers total 675,366,512 bytes; expanded storage, VM and build space are extra.
This is a selected upstream input, not an approved Corbanu runtime by itself.
The image defaults to root with `uv run scrapling --help`; a fixed, bounded
worker entrypoint and containment are required. Do not perform runtime package
resolution, expose a generic MCP browser, or infer health from an open port.

License: [BSD-3-Clause at the pinned source](https://github.com/D4Vinci/Scrapling/blob/333fa22b7a5821194ce66b59b11f4b16a6484f02/LICENSE).
Retain required notices in any redistributed worker/image. Upstream Dockerfile
uses mutable build inputs; a digest fixes the selected artifact, not source-build
reproducibility. Any rebuilt Corbanu image needs its own lock and digest.

## Engine installation inputs

Reuse existing engines without upgrading/replacing them. Qualification records
the actual engine version and capabilities; the application image remains pinned.
Candidate new macOS/Windows Podman installer release: **6.1.0**, from the
[official release](https://github.com/podman-container-tools/podman/releases/tag/v6.1.0).
Release-provided SHA-256 values observed (installer bytes/signatures not yet tested):

| Asset | SHA-256 |
| --- | --- |
| `podman-installer-macos-arm64.pkg` | `ba618cb9a648b57a28708c1832d95ab69d339d9697fdf3265e46e6e9a0cb287d` |
| `podman-installer-windows-amd64.msi` | `1958aac22abb3a9cf7b52626c71ba1a26015c323f0b5fa74671e303b22b043d3` |
| `podman-installer-windows-arm64.msi` | `1d5f33844e168c0892e10f8e0e5c3085f250e5641f916ac1b7bf3665f97efbab` |

Linux uses signed distribution packages with the resolved transaction/version
recorded before consent. Ubuntu 26.04 test host currently offers Podman
`5.7.0+ds2-3build1`; `uidmap` is also absent. Do not force the desktop 6.1.0
installer onto Linux or treat a remote-client-only tarball as a full engine.
VM image pins, Intel Mac installation support and remaining distro transactions
must be resolved before their S03 install paths become ready.

[Podman installation](https://podman.io/docs/installation) and
[machine requirements](https://docs.podman.io/en/stable/markdown/podman-machine.1.html)
distinguish rootless containers from host installation. Windows user-scope
installation can avoid elevation, but WSL/Hyper-V prerequisites may require it.
[Docker Windows permissions](https://docs.docker.com/desktop/setup/install/windows-permission-requirements/)
and [Mac permissions](https://docs.docker.com/desktop/setup/install/mac-permission-requirements/)
likewise distinguish user-mode operation from privileged setup. Use OS-controlled
authentication; never send an installation password through captured application IO.

## Platform preflight, not acceptance

| Host | Observed | Required next evidence | Result |
| --- | --- | --- | --- |
| Local macOS, arm64 | Docker CLI 28.0.1 at `/usr/local/bin/docker`; `desktop-linux` daemon unreachable; no Podman; about 26 GiB free | Reuse Docker; consented daemon startup if needed; hardened worker and real containment/health/cancel/recovery probes | pending |
| Authorized Linux host `100.99.88.49`, account `travis` | Ubuntu 26.04 LTS x86_64; no Podman/Docker on PATH; no uidmap helpers; about 193 GiB free; Python/tmux available | User-approved Podman prerequisite installation, rootless capability checks, then same backend probes | pending |
| Windows | No test host allocated yet | Request instructions from Travis after Mac/Linux completion; test WSL/VM, existing engines, first install, elevation/cancel/recovery | pending |

Read-only commands: `docker version`, `command -v podman`, `uname -m`,
`cat /etc/os-release`, `df -h .`, `apt-cache policy podman uidmap slirp4netns passt`.
SSH host-key verification succeeded; password authentication was used only at
SSH's non-echoing prompt. No password is included here. No runtime installation,
image pull, daemon start, privilege escalation or remote configuration changes
were performed during preflight. No TUI/live-repository proof collected yet.

## Remaining security proof

- Non-root, read-only rootfs, dropped capabilities, resource limits and fresh
  per-acquisition profiles; no host IPC, runtime socket, vault, profiles or mounts.
- Enforced egress at actual connections, DNS and every redirect. Proxy environment
  variables alone are not containment; direct network bypass must be denied.
- Bounded output and quarantined files; explicit promotion only in S02.
- Bounded owned-service recovery; fail closed for collision, unsupported engine,
  failed installer, stale config, timeout, cancellation or failed acquisition probe.
- Independent browser/content health, current authority after resume, and no
  Permissive setup or behavior changes.
- Mac/Linux/Windows real backend evidence, native actual-key S03 workflows,
  user-selected independent model review, and Travis Good's human acceptance.

## Decision/preflight checkpoint validation

This checkpoint changes plans, sprint allocation and evidence only. No production
runtime code has been implemented or certified here.

| Check | Result |
| --- | --- |
| `python3 docs/plans/check.py` | pass; one active plan |
| `python3 docs/sprints/check.py` | pass; 24 current / 86 archived |
| `python3 -m unittest discover -s docs/plans/tests -p 'test_*.py'` | 4 passed |
| `python3 -m unittest discover -s docs/sprints/tests -p 'test_*.py'` | 19 passed |
| `python3 scripts/security-level-standards-check --check-plan --manifest qa/security-levels/sprints/PF-26-S01/crosswalk-pending.json` | 65 result slots remain pending; no qualification pass |
| Existing `.venv-docs/bin/mkdocs build --strict` | pass after correcting stale archived-sprint navigation and out-of-site evidence links; output `/tmp/corbanu-pf30-docs.rqaK2W` |
| `git diff --check` | pass |

No Rust tests, true-TUI runs, live-repository acceptance, independent model review,
human sign-off or release/benchmark qualification is claimed by these checks.
