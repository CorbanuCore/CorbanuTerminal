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
| `scripts/security_platform_probe.py` | `30032a9359aca1672e14bd9571fb573aba2039bceb5576e526787786fd593de9` |
| `scripts/test_security_platform_probe.py` | `6a96c85eab3469fc68fea65bf2b6bc0aee84fc2be291cc36e28e621259f0acf8` |
| `codex-rs/secret-broker/src/platform_contract.rs` | `5ab71ddd09222fff9c6c7866eac71d4b50a964dd3f330a71c596ce41bd46bc31` |
| `codex-rs/secret-broker/Cargo.toml` | `9382ff053132b956cacbe806393101eeebb693b4ca5b2c4e225fb3dff5c0b78f` |
| `codex-rs/secret-broker/BUILD.bazel` | `b3644a641748ae9a6366ed69ec0c5512fecc5047f74aa870a9f664f77cef7a1d` |
| `codex-rs/secret-broker/src/lib.rs` | `256a9b1a0e9c771e3c7db8b014c00b517402314481b316c3c734f49d769d75c7` |
| `capability-result-v1.schema.json` | `da6cb78e37b2473713e652ecb15a871fa8dbc77683c246b3eb4da2ad15d82671` |
| `fixture-protocol-v1.md` | `27650019fe7bde431091c4309f4125ee0151bbf415664e3b6d03b68ccff4134a` |
| `containment-contract-v1.md` | `1e929b22e7429ae8f85f16265a2545676f961446237abe35f906efbe96bce2ae` |
| `platform-mechanisms-v1.md` | `0e3e6bc3a0d2d6c91da0e83cc4d959d1dd041f523c2fbbf310b53726bb20529b` |
| `results/macos.json` | `3964a310c9bbe31fe8c8a120e56f304d17ec4e436115e289ee325b30a44556a5` |
| `results/linux.json` | `a7089dd3e6117152e5cff617465707f56bda175685a89c2ddab3986b5ed02e6f` |
| `results/windows.json` | `fcdb7d7536c4e3d0ce9bdbaa15d1af46469ccd38d948a017695dcbd6cf356404` |

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
- `python3 -m unittest scripts/test_security_platform_probe.py`: 6/6 discovered
  tests pass, including the complete 8-case contract self-test, malformed-input
  stable-error handling, unknown-OS boot-identity denial, validation-mode
  eligibility enforcement, JSON Schema target-metadata parity, and fail-closed
  Windows token-elevation classification.
- `scripts/security-platform-probe --probe ...` and strict current-target
  `--validate ...`: 10/10 capability records generated and validated on macOS.
- The identical SHA-bound implementation and stable shim: 8/8 regressions and
  10/10 capability records generated/validated on Linux, then checked locally with
  `--validate-evidence` (which cannot authorize the local target).
- The same implementation and shim passed 8/8 regressions, generated and
  strictly validated 10/10 Windows capability records, returned exit 2 for
  `--require-eligible`, and passed local archival validation after retrieval.
- Standalone `rustc --test` activation-gate regressions pass 9/9. After the G1
  registration, `just test -p codex-secret-broker` passes 9/9 and Bazel's
  `//codex-rs/secret-broker:secret-broker-unit-tests` target passes 1/1.
- `python3 -m py_compile`, `ruff check`, JSON parse, both governance checkers,
  and `git diff --check`: required before review handoff.

## G1 integration

Jim Ricketts merged the completed PF-34-S04 tree first, audited the literal
PF-27 scope, and then exclusively registered `codex-secret-broker` on the Cargo
and Bazel workspace surfaces. `just bazel-lock-update` completed without
changing `MODULE.bazel.lock`. No workspace crate depends on the new crate, so
the registration exposes only the frozen contract and cannot activate protected
mode. PF-27-S04/S02 must select reviewed per-OS mechanisms and rerun these probes
against the real controller/broker/worker launch path.
