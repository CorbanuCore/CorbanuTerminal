# PF-27-S03 platform containment evidence

Candidate base: `1907d99aed9714f05a5f54fca1703658017d616c`

Contract: `corbanu.platform-containment/v1`

Fixture protocol: `corbanu.platform-probe/v1`

The probe uses random synthetic canaries only. It writes no credential, username,
hostname, source path, or exception text to result artifacts. G1 registers the
contract-only crate, but does not add a runtime consumer, enable a broker/profile,
alter a host policy, or leave an IPC/network listener running.

## Artifact identity

| Artifact | SHA-256 |
| --- | --- |
| `scripts/security-platform-probe` | `0045b29fc50c69c0282083d2c5da12d25a184e4d7742445ac5fa5515c4996a70` |
| `scripts/security_platform_probe.py` | `c1be49372faffe0d0ea1791290a1817ee0c2d9eaee510d2c6d830e876adb7acb` |
| `scripts/test_security_platform_probe.py` | `141eb77d7096ad56df7183743192ef5ae8918e97d30b608fce0fe7a4f38962c0` |
| `codex-rs/secret-broker/src/platform_contract.rs` | `7f6b597788010cc32b6156205e4a506cb293b528012883b6f69324dc62b825d5` |
| `codex-rs/secret-broker/src/platform_contract_fixture_tests.rs` | `18eebbeca85531e89b5c8859814f4a04e4c928b13a5e0bb3cb011c63fe488cd2` |
| `codex-rs/secret-broker/Cargo.toml` | `8dd9ba505ed575fd78f81c9b81a8069e908db0e864ab8aa5899ad1bb5466d67a` |
| `codex-rs/secret-broker/BUILD.bazel` | `665f804676a5acfccd12fb1b6095630ead6d13e05c8bf19aca1c665151faa0a7` |
| `codex-rs/secret-broker/src/lib.rs` | `03a2b98a134bb23cc9b993f537432770fe78d524b647d77f3dcf71f9789b4f79` |
| `codex-rs/Cargo.toml` | `82b971ddfafd10fdadbae414bab4c1a5119b0173fd9ac522c90efb0eabd8c2b3` |
| `codex-rs/Cargo.lock` | `e643a17d738119b18ae9af44a340bad283de306c2227060cbd922046cce43153` |
| `BUILD.bazel` | `226aeb958f0032a4efac71b9e9b1c6731bc2d26ee6d019c2af86b3b9787a8e79` |
| `.github/workflows/security-platform-contract.yml` | `e8228456a740dcb638fdffc2b00ea26ca8174886f3ef561dd01d74e9b3e7680e` |
| `capability-result-v1.schema.json` | `da6cb78e37b2473713e652ecb15a871fa8dbc77683c246b3eb4da2ad15d82671` |
| `fixture-protocol-v1.md` | `27650019fe7bde431091c4309f4125ee0151bbf415664e3b6d03b68ccff4134a` |
| `containment-contract-v1.md` | `1e929b22e7429ae8f85f16265a2545676f961446237abe35f906efbe96bce2ae` |
| `platform-mechanisms-v1.md` | `0e3e6bc3a0d2d6c91da0e83cc4d959d1dd041f523c2fbbf310b53726bb20529b` |
| `results/macos.json` | `456db8c727867b722d372b60ec339784b0ac8b4359f76334a09c7dd62ca7e4cf` |
| `results/linux.json` | `6306c9a6ad47e55b2311c59a9f3eb3384fe1aa0f79ba745d01fc7eea7ec3cda7` |
| `results/windows.json` | `c1d5c1ef8c75422ef18efdea3456c03d7e59f08c7dac7133655d9e5d49d67470` |

The result files embed the exact linted implementation-module SHA-256. The
stable extensionless shim is separately bound above. The Markdown/schema hashes
bind the surrounding frozen contract; the G1 integration owner must recompute
them after any review repair.

Each repository result is reproduced from the probe's raw `--output` file with
`jq -c . raw.json > results/<target>.json`; `cmp` then verifies exact byte
identity with that generated compact form before its hash is published.

## Platform matrix

| Target | Runtime | Supported | Unsupported | Untested | Eligibility |
| --- | --- | ---: | ---: | ---: | --- |
| macOS arm64 | Darwin 25.0.0; Python 3.14.4; Apple M2 Ultra | 3 | 6 | 1 | false |
| Linux x86_64 | Linux 6.8.0-49; glibc 2.39; Python 3.12.3; AMD EPYC-Milan | 4 | 6 | 0 | false |
| Windows 11 AMD64 | Windows 11 10.0.26200; Python 3.13.15; Intel64 Family 6 Model 106 | 0 | 8 | 2 | false |

All three executed targets prove that the measured worker context is not a
protected broker boundary: controller process/file/config and loopback access
remain available, and all protected-store attacks were possible. The macOS and
Linux workers were ordinary same-user subprocesses; the Windows token probe
detected that its SSH-launched worker was already elevated and classified that
as an explicit unsupported privilege bypass. Descriptor
closure, non-interactive elevation denial, and process-memory denial are useful
inputs but do not compensate for another missing capability. Linux additionally
verified `SO_PEERCRED`; the selected macOS Python API exposed no peer-credential
mechanism, so that row is honestly untested.

Windows was measured directly through the authorized Tailscale/SSH route. It
confirmed process, file, config, loopback, process-debug, elevated-token,
signature, and protected-store bypasses. Inherited-handle negative control and
AF_UNIX peer credentials remain explicitly untested; neither is inferred as a
pass. The probe and its synthetic temporary files were removed from the remote
host after evidence retrieval.

## Commands and counts

- `scripts/security-platform-probe --self-test`: 8/8 semantic regressions pass,
  including wrong-target, wrong-probe, stale-result, future archival evidence,
  false-eligibility, duplicate/missing capability, and inconsistent
  status/observation rejection, plus an all-supported eligible report; each
  self-test also executes all 10 probes.
- `python3 -m unittest scripts/test_security_platform_probe.py`: 7/7 discovered
  tests pass, including the complete 8-case contract self-test, malformed-input
  stable-error handling, unknown-OS boot-identity denial, validation-mode
  eligibility enforcement, JSON Schema target-metadata parity, and fail-closed
  Windows token-elevation classification, and long temporary-path independence.
- `scripts/security-platform-probe --probe ...` and strict current-target
  `--validate ...`: 10/10 capability records generated and validated on macOS.
- The identical SHA-bound implementation and stable shim: 8/8 regressions and
  10/10 capability records generated/validated on Linux, then checked locally with
  `--validate-evidence` (which cannot authorize the local target).
- The same implementation and shim passed 8/8 regressions, generated and
  strictly validated 10/10 Windows capability records, returned exit 2 for
  `--require-eligible`, and passed local archival validation after retrieval.
- Standalone `rustc --test` activation-gate regressions passed 9/9 before G1.
  After the G1 registration and fixture binding, `just test -p
  codex-secret-broker` passes 16/16 and Bazel's
  `//codex-rs/secret-broker:secret-broker-unit-tests` target passes 1/1.
- `python3 -m py_compile`, `ruff check`, JSON parse, both governance checkers,
  and `git diff --check`: required before review handoff.

## G1 integration

The Codex ingress/classifier integration lane merged the completed PF-34-S04
tree first, audited the literal PF-27 scope, and then exclusively registered
`codex-secret-broker` on the Cargo and Bazel workspace surfaces.
`just bazel-lock-update` completed without
changing `MODULE.bazel.lock`. No workspace crate depends on the new crate, so
the registration exposes only the frozen contract and cannot activate protected
mode. PF-27-S04/S02 must select reviewed per-OS mechanisms and rerun these probes
against the real controller/broker/worker launch path.
