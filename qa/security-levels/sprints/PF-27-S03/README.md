# PF-27-S03 platform containment evidence

Candidate base: `1907d99aed9714f05a5f54fca1703658017d616c`

Contract: `corbanu.platform-containment/v1`

Fixture protocol: `corbanu.platform-probe/v1`

The probe uses random synthetic canaries only. It writes no credential, username,
hostname, source path, or exception text to result artifacts. Probe preparation
does not register a crate, enable a broker/profile, alter a host policy, or leave
an IPC/network listener running.

## Artifact identity

| Artifact | SHA-256 |
| --- | --- |
| `scripts/security-platform-probe` | `0045b29fc50c69c0282083d2c5da12d25a184e4d7742445ac5fa5515c4996a70` |
| `scripts/security_platform_probe.py` | `549c50a5fe8804a3ed6cf22b7de2a947dd8093a6a8bb18a0317e2cfecbcb0440` |
| `scripts/test_security_platform_probe.py` | `a26cb7874cb6753a3b403711c9c3bc6543e1d8ac2a6ac66ba10db5e6cf09244a` |
| `codex-rs/secret-broker/src/platform_contract.rs` | `9a0d07e9f7e2ce462f9956a33f0915607089aa529ddfdff969abd1a541d0bdcf` |
| `capability-result-v1.schema.json` | `da6cb78e37b2473713e652ecb15a871fa8dbc77683c246b3eb4da2ad15d82671` |
| `fixture-protocol-v1.md` | `27650019fe7bde431091c4309f4125ee0151bbf415664e3b6d03b68ccff4134a` |
| `containment-contract-v1.md` | `1e929b22e7429ae8f85f16265a2545676f961446237abe35f906efbe96bce2ae` |
| `platform-mechanisms-v1.md` | `a6fedef0c39661f9e9e74e6b8630a1ef1ba2ebfb30a7e64ba1201885b101dc74` |
| `results/macos.json` | `fa579d5586e35ce00a659562060a65fe508231acc366055b5f30a34a3655c62d` |
| `results/linux.json` | `999b1907db6c750a502efa5001dfc117ca2047a94799e823f19f19fc3a388088` |

The result files embed the exact linted implementation-module SHA-256. The
stable extensionless shim is separately bound above. The Markdown/schema hashes
bind the surrounding frozen contract; the G1 integration owner must recompute
them after any review repair.

## Platform matrix

| Target | Runtime | Supported | Unsupported | Untested | Eligibility |
| --- | --- | ---: | ---: | ---: | --- |
| macOS Apple M2 Ultra arm64 | Darwin 25.0.0; Python 3.14.4 | 3 | 6 | 1 | false |
| Ubuntu Linux AMD EPYC-Milan x86_64 | Linux 6.8.0-49; Ubuntu 24.04.3; Python 3.12.3 | 4 | 6 | 0 | false |
| Windows | Authorized endpoint absent from the connected Tailscale tailnet; both supplied IP routes timed out on port 22 | 0 | 0 | 10 | untested, never a pass |

Both executed targets prove that an ordinary same-user subprocess is not a
protected broker boundary: controller process/file/config and loopback access
remain available, and all protected-store attacks were possible. Descriptor
closure, non-interactive elevation denial, and process-memory denial are useful
inputs but do not compensate for another missing capability. Linux additionally
verified `SO_PEERCRED`; the selected macOS Python API exposed no peer-credential
mechanism, so that row is honestly untested.

Windows remains a hard completion gate. The local Tailscale client is connected,
but its peer list does not contain the authorized Windows host; direct SSH to the
two previously supplied Tailscale addresses timed out. No Windows result is
inferred from prior PF-13 evidence or from another OS.

## Commands and counts

- `scripts/security-platform-probe --self-test`: 8/8 semantic regressions pass,
  including wrong-target, wrong-probe, stale-result, future archival evidence,
  false-eligibility, duplicate/missing capability, and inconsistent
  status/observation rejection, plus an all-supported eligible report; each
  self-test also executes all 10 probes.
- `python3 -m unittest scripts/test_security_platform_probe.py`: 5/5 discovered
  tests pass, including the complete 8-case contract self-test, malformed-input
  stable-error handling, unknown-OS boot-identity denial, validation-mode
  eligibility enforcement, and JSON Schema target-metadata parity.
- `scripts/security-platform-probe --probe ...` and strict current-target
  `--validate ...`: 10/10 capability records generated and validated on macOS.
- The identical SHA-bound implementation and stable shim: 8/8 regressions and
  10/10 capability records generated/validated on Linux, then checked locally with
  `--validate-evidence` (which cannot authorize the local target).
- Standalone `rustc --test` activation-gate regressions pass 9/9 and library
  compilation passes
  without registering a Cargo/Bazel runtime route.
- `python3 -m py_compile`, `ruff check`, JSON parse, both governance checkers,
  and `git diff --check`: required before review handoff.

## Integration handoff

G1 is serialized to Jim Ricketts. The receiver audits the literal scope and
exclusively registers the future `codex-content-security` Cargo/Bazel surfaces.
PF-27-S04/S02 must select reviewed per-OS mechanisms and rerun these probes
against the real controller/broker/worker launch path. This fixture-only sprint
cannot activate protected mode.
