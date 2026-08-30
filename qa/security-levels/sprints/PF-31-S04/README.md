# PF-31-S04 candidate evidence

Product citation: **Reconciled security scope — TO BUILD** — “Unknown or
unsupported protected paths fail visibly rather than falling back to raw secrets
or unscreened execution.”

## Candidate and scope

- Dispatch base: `ea23dfa38bc4f2cbfe0aceadd6777c3e436a53d4`
- Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-browser-retrieval`
- Branch: `feat/p0-security-browser-retrieval`
- Candidate commit: recorded after this evidence lands.
- Contract: `pf31-retriever-artifact/v1`
- Artifact: `ghcr.io/d4vinci/scrapling@sha256:1bacbc8ec90b3090d462e12f6555e241daf0dfeb684ab326ffa09d52d8226e69`
- Source: Scrapling `v0.4.15` at `333fa22b7a5821194ce66b59b11f4b16a6484f02`

The changed paths are limited to the PF-31-S04 sprint record, the standalone
validator, retriever manifest/fixtures/SBOM/license evidence, and this evidence
directory. No Cargo/Bazel file, module registry, Core/protocol path, TUI path,
runtime broker, sandbox hook, or network-proxy route changed. The validator is a
pure JSON evaluator and never invokes Docker/Podman, opens a socket, pulls an
image, starts a worker, or modifies a user's engine choice.

## Artifact and supply-chain findings

The upstream OCI index is multi-architecture:

| Platform | Manifest digest | Compressed | Unpacked measured |
| --- | --- | ---: | ---: |
| `linux/amd64` | `sha256:65fc31598af6cc5053c63b5f5205524a94964d82fd33b24b6c96c7cb0be35ada` | 643,020,537 B | 1,632,652,870 B |
| `linux/arm64` | `sha256:1a5e7c7a08b6a892b793cd5c596a3bde5184f9a7c1bd74efb846cf5550377f13` | 675,366,512 B | 1,979,324,372 B |

Both manifests have SLSA v1 provenance from the same GitHub Actions run and
source revision. The source merge commit is GitHub-verified; the release tag is
lightweight. No publisher OCI signature or publisher SBOM was present. Upstream
also built from `uv:latest`, version ranges, and no source lockfile. Independently
generated CycloneDX SBOMs contain 5,332 amd64 and 5,323 arm64 components; their
hashes are locked by the manifest. License review remains pending for missing,
unknown, digest-only and Chromium third-party records. These facts block
protected activation and require a locked, reviewed, Corbanu-signed rebuild in
PF-31-S01.

## Engine and failure contract

Explicit Podman/Docker choice is preserved. Without an explicit choice, an
equally eligible Podman installation wins over Docker. Every create/pull/start/
restart/test flow first acquires a Corbanu-owned lock, re-inspects state and
reuses exactly one eligible PF-31 worker. The contract never installs or elevates
an engine, touches unrelated resources, changes a user default, uses a mutable
tag as identity, or falls back after a verification failure.

Fourteen fixtures cover absent engine, stopped engine, stalled engine,
mismatched digest, tampering, offline verification, wrong architecture,
duplicate owned workers, concurrent clients, explicit selection, automatic
Podman preference, fallback to an eligible Docker engine, unrelated resources,
and a stalled owned worker. Each fixture is evaluated twice to prove
deterministic idempotency.

## Commands and actual counts

Formatting and local final-tree checks:

```text
ruff format scripts/security-retriever-artifact-check
ruff check scripts/security-retriever-artifact-check
scripts/security-retriever-artifact-check all
python3 -m py_compile scripts/security-retriever-artifact-check
jq empty qa/security-levels/retriever/artifact-manifest.json qa/security-levels/retriever/engine-fixtures/*.json
python3 docs/plans/check.py
python3 docs/sprints/check.py
git diff --check
```

Results: Ruff clean; one manifest validated; 14/14 fixtures and 14/14 repeated
idempotency checks passed; JSON parsing, byte compilation, plan governance,
sprint governance and whitespace checks passed on macOS. The same portable
bundle passed one manifest, 14/14 fixtures and 14/14 idempotency checks with
Python 3.12.3 on Ubuntu 24.04 Linux.

Resource probes launched the exact image by digest with network disabled, one
CPU, 1 GiB RAM and a 256-PID limit, opened a blank headless Chromium, recorded a
single idle sample, and exposed no port:

| Host | Engine | Image platform | Browser | Idle sample | Result |
| --- | --- | --- | --- | --- | --- |
| macOS 26.0 arm64 | Docker 27.5.1 | `linux/arm64` | Chromium 151.0.7922.34 | 170.9 MiB, 0.02% CPU, 68 PIDs | pass |
| Ubuntu 24.04 amd64 | Docker 29.1.3 | `linux/amd64` | Chromium 151.0.7922.34 | 152.8 MiB, 0.00% CPU, 72 PIDs | pass |
| Windows amd64 | pending | `linux/amd64` in Linux-container mode only | pending | supplied host unreachable | blocked |

The manifest reserves one CPU, 1 GiB RAM, 256 PIDs, 256 MiB temporary storage
and 2.5 GiB disk. These are preparation limits, not PF-31-S01 workload or
containment qualification.

## Blockers and handoff

- Windows engine/resource execution is incomplete because the supplied host was
  unreachable.
- Podman version selection and a three-host Podman matrix are unresolved.
- Publisher signature and SBOM are absent; the independently generated license
  inventory is not approved.
- A digest-locked Corbanu rebuild, signature identity and verification policy
  implementation belong to PF-31-S01 and need integration-owner approval.
- Claude Opus 5.0 Max review and finding disposition are pending.

Hand back the final candidate commit and this evidence to Jim Ricketts. The
integration owner audits scope and reruns the manifest, fixture and governance
checks on the combined tree. PF-31-S04 must remain blocked rather than archived,
and PF-33-S03 must not start, until the missing PF-31 evidence is resolved and
the integration owner completes the documented G2 transition.
