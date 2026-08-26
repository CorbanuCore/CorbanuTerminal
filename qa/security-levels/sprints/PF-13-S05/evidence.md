# PF-13-S05 credential-boundary qualification evidence

- Date: 2026-08-26 UTC
- Status: in progress
- Product requirement: `docs/corbanu-product-spec.md`, **Required trust boundaries** — “Permit agents to reference credentials only by label; resolve them solely inside the trusted execution boundary.”
- Harness implementation commit: `27b738ab8d6289b2dc27fc45549fddc2622f6bc7`
- Linux report commit: `24ba535698e7b25cb8cf1c7d7e06689603731dcb`
- Tested source commit: `27b738ab8d6289b2dc27fc45549fddc2622f6bc7`
- Candidate SHA-256: `24968f6e12106c37268f3193aa2495591b5000f2e819bdc47dcf71e361eb5f7f`
- Report SHA-256: `9dc2886639055d19fb0a015b805c3ca2d4d60e88ab162f35568e6845c2f83f56`
- TUI applicability: none; this sprint changes no interactive surface.

## Current result

The Linux qualification passed from a clean source tree. The harness generated a
fresh credential canary inside the Rust test process, stored it through the
encrypted Vault path, consumed a complete Core credential capability, and
resolved it only inside the trusted proxy injection callback. The exact
outgoing OpenAI request capture observed the raw value once. A second attempt
failed as a replay before another provider request.

Only the canary SHA-256 digest crossed into the report. The test scanned actual
secret-free authority/model context, tool-shaped payload, virtualized child
environment, audit metadata, errors, receipt logs, contained panic output, and
Vault files. The harness additionally scans every subprocess capture and the
serialized report for credential-shaped material.

The machine-readable report is
`qa/security-levels/sprints/PF-13-S05/credential-canary-report.json`. It binds
the candidate identity, clean source commit, host identity, exact commands,
named expected tests, executed counts, source-file digests, surface coverage,
and canary digest.

## Adversarial matrix

| Boundary case | Result | Probe |
| --- | --- | --- |
| One exact authorized OpenAI request | Passed; one raw observation in the outgoing capture | `core-capability-and-unique-canary` |
| Malformed or ambiguous authority | Failed closed | `policy-authority-validation` |
| Forged bearer/public capability id | Failed closed without consuming valid authority | `core-capability-and-unique-canary` |
| Expired or revoked authority | Failed closed before reuse/decryption | policy, Vault, proxy, and Core probes |
| Sequential replay / redirect reuse | Failed before a second resolver/provider request | Core and proxy probes |
| Wrong actor, purpose, operation, method, host, path, or scope | Failed closed | Core and proxy probes |
| Concurrent duplicate use | Exactly one consumer succeeded | Core capability probe |
| Revocation racing an active use | Read/write locking linearized revocation after the active callback | `core-revocation-linearization` |
| Bounded store exhaustion and cleanup | Hard capacity enforced; expired entries reclaimed | Core capability probe |
| Callback error, cancellation, or panic | Stable secret-free outcomes; panic contained | Vault and dynamic canary probes |
| Protected raw export and downgrade attempt | Moderate/Aggressive denial; persisted posture won | CLI raw-export probe |
| Child environment | Raw input replaced by a fresh opaque dummy | proxy and dynamic canary probes |
| Logs, audit, receipts, errors, artifacts | Canary absent | dynamic canary probe |

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

## Cross-platform enforcement

`.github/workflows/credential-boundary-canary.yml` runs the identical
fail-closed harness on Ubuntu 24.04, macOS 15, and Windows 2022. The harness
accepts only Linux, Darwin, or Windows and has no flag that skips host checks.
Each job uploads its complete report as a commit-bound artifact.

Linux evidence is present above. macOS and Windows workflow results have not yet
been produced and therefore remain an acceptance gate.

## Remaining acceptance gates

- Attach passing macOS and Windows reports from the credential-boundary canary workflow.
- Name an independent security reviewer and record the raw-secret reachability review and any corrections.
- Obtain explicit approval for, then run, the repository-policy-gated complete
  `just test -p codex-core` suite. The focused Core qualification tests passed,
  but this is not represented as the complete Core suite.
- After those gates pass, update the final candidate/evidence coordinates and
  archive PF-13-S05. The sprint remains `in_progress` until then.
