# PF-31-S04 license inventory

Artifact: Scrapling 0.4.15, source commit
`333fa22b7a5821194ce66b59b11f4b16a6484f02`, OCI index
`sha256:1bacbc8ec90b3090d462e12f6555e241daf0dfeb684ab326ffa09d52d8226e69`.

The upstream project declares BSD-3-Clause and includes its license in the
image. The per-architecture CycloneDX files beside this inventory are the exact
component/license inventories generated from the pinned image manifests with
Syft 1.51.1. The amd64 catalog has 5,332 records: 4,009 file records and roughly
790 unique packages (199 Debian, 66 Python, 523 Rust crates and 2 npm packages).
The arm64 catalog has 5,323 records. Cargo crates are represented once for each
of the bundled `uv` and `uvx` binaries, so record counts are not unique-package
counts or a completeness measure.

The Python layer reports BSD, MIT, Apache-2.0, MPL, ISC, PSF and related
expressions. Two Python records require manual resolution: `prompt-toolkit`
3.0.53 has no license value in the generated metadata and `ptyprocess` 0.7.0
reports `UNKNOWN`. The amd64 catalog has 5,066 records without a `licenses`
array, including 1,046 duplicate cargo records representing 523 unique Rust
crates bundled in `uv` and `uvx`. Debian records include GPL-2, GPL-3 and LGPL
families as well as records that expose a license-file digest instead of an SPDX
expression. Those unresolved records, all reciprocal-license obligations, and
Chromium's third-party notices require legal review before a Corbanu rebuild can
be signed or enabled. The presence of this inventory is not license approval.

Upstream release facts observed 2026-08-29:

- `v0.4.15` is a lightweight tag at the commit above; the GitHub merge commit
  has a verified GitHub signature, but the tag itself is not signed.
- The GHCR image has per-platform SLSA provenance attestations pointing at that
  commit and the exact base-image inputs recorded in the artifact manifest.
- No publisher image signature or publisher SBOM was present. The SBOMs in this
  directory are independently generated evidence, not publisher attestations.
- The upstream Dockerfile uses `ghcr.io/astral-sh/uv:latest`, dependency ranges,
  and no source lockfile. A Corbanu-owned rebuild must lock every input and emit
  its own signed provenance and SBOM before protected activation.
