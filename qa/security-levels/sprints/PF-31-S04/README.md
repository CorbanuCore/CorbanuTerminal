# PF-31-S04 candidate evidence

Product citation: **Reconciled security scope — TO BUILD** — “Unknown or
unsupported protected paths fail visibly rather than falling back to raw secrets
or unscreened execution.”

## Candidate and scope

- Dispatch base: `ea23dfa38bc4f2cbfe0aceadd6777c3e436a53d4`
- Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-browser-retrieval`
- Branch: `feat/p0-security-browser-retrieval`
- Initial implementation candidate: `d93a8e4787aeacb4294a318243730ec35384bc39`
- Windows qualification candidate: `1cc3ad92e4de7cbeac40105c4e8d0e9d35ac30c0`
- Final code candidate: `cf05164e7d5c21a8e716a36c70455c5cff26f5ca`
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
generated CycloneDX SBOMs contain 5,332 amd64 and 5,323 arm64 catalog records;
the license inventory separates file records, package records and approximate
unique identities on both architectures, and its hash is locked by the manifest.
License review remains pending for missing, unknown, digest-only, reciprocal-
license and Chromium third-party records. These facts block
protected activation and require a locked, reviewed, Corbanu-signed rebuild in
PF-31-S01.

## Engine and failure contract

Explicit Podman/Docker choice is preserved. Without an explicit choice, an
equally eligible Podman installation wins over Docker. Every create/pull/start/
restart/test flow first acquires a Corbanu-owned lock, re-inspects state and
reuses exactly one eligible PF-31 worker. The contract never installs or elevates
an engine, touches unrelated resources, changes a user default, uses a mutable
tag as identity, or falls back after a verification failure.

Twenty-seven fixtures cover absent/stopped/stalled engines; index, platform,
manifest, config and tamper failures; offline verification; architecture;
same-engine and cross-engine duplicates; unavailable-engine ownership;
concurrent clients; explicit and automatic selection; ready-over-create
precedence; later-engine integrity failures; explicit and automatic prevention
of duplicate creation; unrelated resources; post-lock snapshot binding; and
both first-run lock encodings. Each fixture is evaluated
twice as a deterministic replay check. The replay check does not claim live
engine idempotence or TOCTOU qualification.

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

Results for final code candidate `cf05164e7`: Ruff clean; one manifest,
27/27 fixtures, 27/27 deterministic replays, five manifest-policy mutations and
ten path checks passed; JSON parsing, byte compilation, a read-only extracted
checkout, plan governance,
sprint governance and whitespace checks passed on macOS. Portable bundle
`5753260131b1c4fada152843a9ed792bbc5edb4b6d3e0634e90ee49d05661c0b`
passed the same counts with Python 3.12.3 on Ubuntu 24.04 Linux. Windows 11 Pro
10.0.26200 then passed the exact same hash-identified bundle with Python 3.13.15:
27 fixtures, 27 deterministic replays, five manifest-policy mutations and ten
evidence-path checks. Its pinned SSH fingerprint was verified before transfer;
the remote bundle SHA-256 matched before execution. The earlier Windows
engine/image probe remains the platform-specific launch qualification.

Resource probes launched the exact image by digest with network disabled, one
CPU, 1 GiB RAM and a 256-PID limit, opened a blank headless Chromium, recorded a
single idle sample, and exposed no port:

| Host | Engine | Image platform | Browser | Idle sample | Result |
| --- | --- | --- | --- | --- | --- |
| macOS 26.0 arm64 | Docker 27.5.1 | `linux/arm64` | Chromium 151.0.7922.34 | 170.9 MiB, 0.02% CPU, 68 PIDs | pass |
| Ubuntu 24.04 amd64 | Docker 29.1.3 | `linux/amd64` | Chromium 151.0.7922.34 | 152.8 MiB, 0.00% CPU, 72 PIDs | pass |
| Windows 11 Pro 10.0.26200 amd64 | Podman 5.8.3 client / 5.8.6 server | `linux/amd64` in Linux-container mode only | Chromium 151.0.7922.34 | 462.7 MiB cgroup memory, 0.09% cgroup CPU, 71 PIDs | pass |

The Windows probe additionally confirmed a read-only root, memory plus swap
capped at 1 GiB, no published ports and 1,647,867,740 bytes of Podman image
storage. The manifest reserves one CPU, 1 GiB RAM, 256 PIDs, 256 MiB temporary
storage and 2.5 GiB disk. These are preparation limits, not PF-31-S01 workload
or containment qualification. Full Windows commands and outputs are preserved
in `windows-podman.md`.

## Blockers and handoff

- Publisher signature and SBOM are absent; the independently generated license
  inventory is not approved.
- A digest-locked Corbanu rebuild, signature identity and verification policy
  implementation belong to PF-31-S01 and need integration-owner approval.
- Claude Opus 5 Max initial and corrected-candidate reviews returned actionable
  findings. Every finding was accepted or explicitly classified, fixed in the
  final candidate and regression-tested; the complete disposition and visible
  Opus 5/Max evidence are recorded under `review/claude-opus-5-max/`.

Hand back the final candidate commit and this evidence to Jim Ricketts. The
integration owner audits scope and reruns the manifest, fixture and governance
checks on the combined tree and completes the documented G2 transition.
PF-33-S03 must not start before that archive step.

## 2026-08-30 post-archive review remediation

A trace-backed independent review in tmux through Corbanu Terminal, requested as
`claude-opus-5-plan` at `max` effort and reported by the provider as
`claude-opus-5`, found that this evidence did not disclose the two identical
runtime pins in `codex-rs/browser-isolation/src/image.rs` and
`codex-rs/browser-isolation/worker/Dockerfile`. The first automatically pulls
the immutable `BASE_IMAGE` reference when it is absent; the second is the
`FROM` reference used by the `--pull=false` local worker build.

The standalone validator now treats both source literals as required runtime
copies of the PF-31 pin. Canonical repository-contained reads reject leaf or
ancestor symlinks; each file must expose exactly one canonical pin, and both
must equal `artifact.reference` byte for byte. The negative suite changes the
manifest reference and independently removes or duplicates each source literal.
The original five-mutation result above remains the historical candidate result;
the repaired tree reports 27 fixtures, 27 deterministic replays, eleven policy
mutations and ten evidence-path checks.

The initial review coordinates and disposition are preserved in
[`review/corbanu-tmux-claude-opus-5-max-20260830.md`](review/corbanu-tmux-claude-opus-5-max-20260830.md).
