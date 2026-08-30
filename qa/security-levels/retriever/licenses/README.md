# PF-31-S04 license inventory

Artifact: Scrapling 0.4.15, source commit
`333fa22b7a5821194ce66b59b11f4b16a6484f02`, OCI index
`sha256:1bacbc8ec90b3090d462e12f6555e241daf0dfeb684ab326ffa09d52d8226e69`.

The upstream project declares BSD-3-Clause and includes its license in the
image. The per-architecture CycloneDX files beside this inventory are the exact
component/license inventories generated from the pinned image manifests with
Syft 1.51.1. The amd64 catalog has 5,332 records: 4,009 file records and 1,323
package/application/OS records representing about 795 unique identities. The
arm64 catalog has 5,323 records: 4,002 file records and 1,321 package/application/
OS records representing about 793 unique identities. The package records are:

| Architecture | Debian | Python | Cargo records / unique crates | npm | generic apps | OS | duplicated Simple Launcher apps |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amd64 | 199 | 66 | 1,046 / 523 | 2 | 3 | 1 | 6 / 1 identity |
| arm64 | 197 | 66 | 1,046 / 523 | 2 | 3 | 1 | 6 / 1 identity |

Cargo crates are represented once for each of the bundled `uv` and `uvx`
binaries. Record counts are therefore not unique-package counts or a
completeness measure.

The Python layer reports BSD, MIT, Apache-2.0, MPL, ISC, PSF and related
expressions. Two Python records require manual resolution: `prompt-toolkit`
3.0.53 has no license value in the generated metadata and `ptyprocess` 0.7.0
reports `UNKNOWN`. The amd64 catalog has 5,066 records without a `licenses`
array; arm64 has 5,059. On each architecture that set comprises every file
record, 1,046 cargo records, one additional library record, all nine application
records and the OS record. The catalogs explicitly identify Node 24.18.1,
Debian 13.6 and Chromium 151.0.7922.34; Chromium's file path also corroborates
Playwright revision 1234. Neither SBOM contains an `ffmpeg` record or string, so
the previously observed Playwright ffmpeg revision is deliberately excluded as
uncorroborated, not claimed absent from the root filesystem. Debian records
include GPL-2, GPL-3 and LGPL families as well as records that expose a
license-file digest instead of an SPDX expression. Those unresolved records,
all reciprocal-license obligations, and Chromium's third-party notices require
legal review before a Corbanu rebuild can be signed or enabled. The presence of
this inventory is not license approval.

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
