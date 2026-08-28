# PF-13-S05 credential-boundary qualification evidence

- Date: 2026-08-27 UTC; outside-review update 2026-08-28 UTC
- Status: in progress
- Product requirement: `docs/corbanu-product-spec.md`, **Required trust boundaries** — “Credentials are referenced by label and resolved only inside a trusted execution boundary.”
- Harness implementation commit: `27b738ab8d6289b2dc27fc45549fddc2622f6bc7`
- Linux report commit: `24ba535698e7b25cb8cf1c7d7e06689603731dcb`
- Original cross-platform workflow run: `32933578147` at `55025dd42a869221b023fd783d52038b4b7c092f`
- Windows follow-up workflow run: `33111412618` at `ea7d4bec720098f6e0994fcfcc59e272108f7e70`
- Windows candidate SHA-256: `f829e89081ead67b9d9fdabc1841759725983a8630dcb4affaf885dd7f0d9720`
- Windows downloaded-report SHA-256: `1636dc2a60be433139a0e4dcdc1c177c47df57634bc05522295528839c97bd21`
- Windows committed-report SHA-256: `23d6861b78552d363e422bf9712f1fd43c970c13bc3c95de810bf8e903b5376b`
- Windows uploaded-artifact SHA-256: `c9c17fd5c626df315512ce496f16acf00eb9024fc87ee45ad3b441f20917271b`
- Local macOS tested source commit: `55025dd42a869221b023fd783d52038b4b7c092f`
- Local macOS candidate SHA-256: `b4db4cf7b3f1f70465e25028e51f0ac7553427ad5be7dbc3b0a7e47dc68ed8f1`
- Local macOS report SHA-256: `6105fef2834e179ea955f9758f896005fb368a3228f87d3e3f840d75ef999ed5`
- Local macOS Core JUnit SHA-256 (gzip): `c2fdeb23c5e2c86dcc8cd636e25377c5ab20ce45ad83d96e25cb30b09a1476dd`
- Original TUI applicability: none. The later accepted panic-hook repair requires and now has final-candidate PTY proof in `repair-final-tui-macos.md`.

## Current result

Historical credential canary runs passed on Linux, macOS, and Windows, including
a separate local macOS reproduction and a Windows 2022 follow-up at the recorded
clean candidate. These are invoked component/test-seam results, not proof of a
native session using scoped credentials. Those historical records remain intact.
The accepted [repair cycle](repair-evidence.md) subsequently passed 47 canary
tests on both Mac and Linux at `f6ec1c75f`, final Mac PTY proof and Kimi review.
The fresh complete Core run passed 3,388/3,407 with 19 failures (down from the
historical 135); final-tree Windows evidence is unavailable. **Qualification
remains not ready**; see [Core triage](repair-core-triage.md).

Subsequent [outside-review attempt](fable-outside-review.md): all work through
the PF-30 platform repair and this Windows report was merged into the PF-13
branch at `044491b8b02b24a65a84e8da61619d3444e63fe0` before review. Fable High's
provider automatically switched models after a safeguard flag; the response
was stopped, and no substitute verdict was accepted. Travis then selected Kimi
3.0 High through Corbanu Terminal. Its [completed review and controller
dispositions](kimi-outside-review.md) preserve the raw response and corrections.
Earlier platform results below retain their original candidate identities and
are not relabeled as tests of the integrated merge.

The harness generated a
fresh credential canary inside the Rust test process, stored it through the
encrypted Vault path, consumed a complete Core credential capability, and
resolved it inside the trusted proxy injection callback invoked by the test.
An in-process `HeaderMap` observed the raw value once. A second invocation failed
as a replay. No live CONNECT/MITM socket or provider round trip was measured;
the historical `outgoing_request_count` field counts this test capture. Native
scoped-route installation is not wired into the session path at the reviewed
merge; PF-23 owns that profile integration. The S04 CLI raw-export denial is a
separate, exercised native entry point.

Only the canary SHA-256 digest crossed into the report. The test scanned its
constructed authority/model context, tool-shaped payload, virtualized environment,
audit metadata, errors, receipt logs, stable callback-panic error, and Vault
files. It did not capture a secret-bearing panic through the production panic
hook (review diagnostic C2). The harness scans retained subprocess captures and
the serialized report, but truncates stdout/stderr before scanning: diagnostic
C1 reproduces acceptance when a synthetic marker lies beyond the capture limit.
Thus the historical checks do not establish complete-output/crash-log secrecy.

The Linux machine-readable report is
`qa/security-levels/sprints/PF-13-S05/credential-canary-report.json`; the local
macOS report is
`qa/security-levels/sprints/PF-13-S05/credential-canary-report-macos.json`; and
the Windows report is
`qa/security-levels/sprints/PF-13-S05/credential-canary-report-windows.json`.
Each report binds the candidate identity, clean source commit, host identity,
exact commands, named expected tests, executed counts, source-file digests,
surface coverage, and canary digest.

## Adversarial matrix

| Boundary case | Result | Probe |
| --- | --- | --- |
| One exact authorized OpenAI request binding | Passed; one raw observation in an in-process header capture, not a live request | `core-capability-and-unique-canary` |
| Malformed or ambiguous authority | Failed closed | `policy-authority-validation` |
| Forged bearer/public capability id | Failed closed without consuming valid authority | `core-capability-and-unique-canary` |
| Expired or revoked authority | Failed closed before reuse/decryption | policy, Vault, proxy, and Core probes |
| Sequential replay / redirect reuse | Repeated invocation failed before a second resolution; no live redirect measured | Core and proxy probes |
| Wrong actor, purpose, operation, method, host, path, or scope | Failed closed | Core and proxy probes |
| Concurrent duplicate use | Exactly one consumer succeeded | Core capability probe |
| Revocation racing an active use | Read/write locking linearized revocation after the active callback | `core-revocation-linearization` |
| Bounded store exhaustion and cleanup | Hard capacity enforced; expired entries reclaimed | Core capability probe |
| Callback error, cancellation, or panic | Stable returned outcomes; secret-bearing production panic-hook output untested (C2) | Vault and dynamic canary probes |
| Protected raw export and downgrade attempt | Moderate/Aggressive denial; persisted posture won | CLI raw-export probe |
| Child environment | Test environment virtualized to a dummy; no native child-spawn proof | proxy and dynamic canary probes |
| Logs, audit, receipts, errors, artifacts | Canary absent from tested surfaces; truncated-output and panic-hook gaps C1/C2 remain | dynamic canary probe |

## Harness result

```text
python3 scripts/security-credential-canary \
  --candidate codex-rs/target/debug/corbanu \
  --output qa/security-levels/sprints/PF-13-S05/
PASS — Linux x86_64, Python 3.12.3, kernel 6.8.0-49-generic.
PASS — source_dirty_paths was empty.
PASS — canary SHA-256 797eb9c51beb6f2ff92e94b828a165ba120718b09bbe19849dfa63b804aa4250.
PASS — outgoing_request_count=1 and raw_secret_observations=1.
PASS — 41 tests executed across six probe groups:
       policy 7, Vault 5, proxy 14, Core lifecycle 11,
       Core revocation linearization 1, CLI raw-export 3.
```

### Local macOS reproduction

```text
python3 scripts/security-credential-canary \
  --candidate codex-rs/target/debug/corbanu \
  --output /tmp/corbanu-pf13-report.1MMKeo
PASS — Darwin arm64, Python 3.12.5, Darwin 24.6.0.
PASS — source commit 55025dd42a869221b023fd783d52038b4b7c092f; source_dirty_paths was empty.
PASS — canary SHA-256 b0498cfb332b8f658a546451e88eac1153f2742f6e8d5330198872cfaa4f8041.
PASS — outgoing_request_count=1 and raw_secret_observations=1.
PASS — 41 tests executed across all six probe groups; every command returned 0.

python3 -m unittest scripts.test_security_credential_canary
PASS — 6 tests passed.
```

## Formatting and regression verification

```text
cd codex-rs && just fix -p codex-core && just fmt
PASS — only seven pre-existing Core dead-code warnings remained.

python3 -m unittest scripts.test_security_credential_canary
PASS — 6 tests passed.

cd codex-rs && cargo test -p codex-core --lib \
  credential_authority_unique_canary_is_confined_to_one_outgoing_request \
  -- --nocapture
PASS — 1 test passed; the intentionally panicking callback was contained.

cd codex-rs && just test -p codex-security-policy
PASS — 21 tests passed.

cd codex-rs && just test -p codex-vault
PASS — 29 tests passed.

cd codex-rs && just test -p codex-network-proxy
PASS — 208 tests passed.

python3 docs/plans/check.py && python3 docs/sprints/check.py
PASS — plans active 1/2; sprints current 18 and archived 84.

git diff --check
PASS — no whitespace errors.
```

No Cargo manifests, Cargo/Bazel locks, or crate dependency edges changed.

### Complete Core suite on macOS

The explicitly approved complete Core suite had one blocked build attempt and
two full executions from source commit
`55025dd42a869221b023fd783d52038b4b7c092f`. The initial attempt used the
repository command without linker overrides:

```text
cd codex-rs && just test -p codex-core
BLOCKED BEFORE EXECUTION — Apple ld 1053.12 could not link the monolithic
core/tests/all.rs binary: B/BL out of range (maximum +/-128 MiB).
```

The Rust 1.95 toolchain's bundled `ld64.lld` successfully linked the same
complete test selection. No test filter, feature exclusion, or source change
was applied:

```text
llvm_link_dir="$(rustc --print sysroot)/lib/rustlib/aarch64-apple-darwin/bin/gcc-ld"
PATH="$llvm_link_dir:$PATH" RUSTFLAGS="-C link-arg=-fuse-ld=lld" \
  just test -p codex-core
FAIL — 3,396 tests executed: 3,261 passed and 135 failed; 19 ignored tests were
outside execution. Nextest retried each failure once and returned exit 100.
```

The result reproduced on a second full execution on macOS 15.6.1 arm64 with
Rust 1.95.0 and nextest 0.9.143 (run ID
`816bc19b-e502-4618-ab6c-b4fda8a30e6c`). The JUnit report records 12
failures in the `codex-core` unit binary and 123 in the `codex-core::all`
integration binary. Fifty-two failures cite unavailable companion binaries
such as `target/debug/codex` or `target/debug/test_stdio_server`; 66 failed test
names are code-mode cases, with at least one reproduced in isolation. These
groups overlap and are classifications, not an attribution of root cause.

All 13 credential-named Core tests passed in the complete run, including
`credential_authority_unique_canary_is_confined_to_one_outgoing_request`.
That scoped result supports the credential boundary, but the complete Core
gate remains failed until the broader failures are triaged and a clean full
rerun passes. The complete machine-readable report is
`qa/security-levels/sprints/PF-13-S05/core-nextest-macos-junit.xml.gz`.

## Cross-platform enforcement

`.github/workflows/credential-boundary-canary.yml` runs the identical
fail-closed harness on Ubuntu 24.04, macOS 15, and Windows 2022. The harness
accepts only Linux, Darwin, or Windows and has no flag that skips host checks.
Each job uploads its report with bounded captures as a commit-bound artifact. Workflow run
`32933578147` passed all three jobs at exact source commit
`55025dd42a869221b023fd783d52038b4b7c092f`:

| Host | Result | Artifact | Artifact SHA-256 |
| --- | --- | --- | --- |
| Linux | Passed | `credential-boundary-canary-Linux-55025dd42a869221b023fd783d52038b4b7c092f` | `354fe60b7cba53bf96f35e574e633d25702069d349ef34c9578dd2c7a0801c04` |
| macOS | Passed | `credential-boundary-canary-macOS-55025dd42a869221b023fd783d52038b4b7c092f` | `ed8a36d98ca7c049fc350e367dfc45579b964fe94d68c3650703df5c9b132272` |
| Windows | Passed | `credential-boundary-canary-Windows-55025dd42a869221b023fd783d52038b4b7c092f` | `33a9df2f03c00e6a5700583aa7a72c6462a4a6714e9e7ddd6878d08043326a18` |

The hosted macOS report itself has SHA-256
`3883577572583f7ece9ec3de7feede5caad2879579e3b6c4a4f2823d3b30474b`.

### Windows 2022 follow-up

The user requested the remaining Windows validation on 2026-08-27. Workflow
run [`33111412618`](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/33111412618)
executed the unmodified PF-13 qualification on a fresh GitHub-hosted Windows
2022 AMD64 machine at exact clean source commit
`ea7d4bec720098f6e0994fcfcc59e272108f7e70` with Python 3.12.10 and Rust
1.95.0 (`x86_64-pc-windows-msvc`).

```text
python -m unittest scripts.test_security_credential_canary
PASS — 6 tests passed.

python scripts/security-credential-canary \
  --candidate codex-rs/target/debug/corbanu.exe \
  --output credential-canary-artifacts/Windows
PASS — status=passed; source_dirty_paths was empty.
PASS — corbanu 0.1.35; candidate SHA-256 f829e89081ead67b9d9fdabc1841759725983a8630dcb4affaf885dd7f0d9720.
PASS — canary SHA-256 82844eed824c3a07cf29c8e3d2129f125adc25e7c60d3fae0d8da669a2e759c3.
PASS — outgoing_request_count=1 and raw_secret_observations=1.
PASS — 41 tests executed across all six probe groups; every command returned 0.
```

The committed report is
`qa/security-levels/sprints/PF-13-S05/credential-canary-report-windows.json`
(SHA-256 `23d6861b78552d363e422bf9712f1fd43c970c13bc3c95de810bf8e903b5376b`
after normalizing the downloaded report's CRLF line endings; the downloaded
report SHA-256 was `1636dc2a60be433139a0e4dcdc1c177c47df57634bc05522295528839c97bd21`).
The commit-bound uploaded artifact is
`credential-boundary-canary-Windows-ea7d4bec720098f6e0994fcfcc59e272108f7e70`
(artifact ID `9663966064`, uploaded-archive SHA-256
`c9c17fd5c626df315512ce496f16acf00eb9024fc87ee45ad3b441f20917271b`).
This closes that historical Windows follow-up; it does not establish integrated
merge qualification or a passing independent-review/complete-Core gate.

## Remaining acceptance gates

- Resolve the accepted [Kimi/controller findings](kimi-outside-review.md),
  especially complete-output scanning (C1) and panic-hook qualification (C2).
  Review execution is complete; its outcome is not a qualification pass.
- Triage the 135 complete-Core failures recorded above, correct or establish
  the required test prerequisites, and record a clean complete rerun.
- Repeat applicable affected tests and cross-platform canaries against the final
  integrated candidate. Preserve the PF-23/PF-26 native integration/TUI boundary;
  test-seam results alone cannot certify running Moderate/Aggressive profiles.
- After those gates pass, update the final candidate/evidence coordinates and
  archive PF-13-S05. The sprint remains `in_progress` until then.
