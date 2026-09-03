# PF-57 latest-main integration qualification

Date: 2026-09-03 UTC

## Candidate and lineage

- Integration branch: `integration/unified-provider-auth-final`
- PF-48–PF-56 feature tip: `06211dbfca61d3f36df3bf069a79ed53ad7a6fa2`
- Latest-main baseline: `81dcbef5dbd500326a14acf8584263d4d950009b`
- Merge commit preserving archived feature SHAs: `14959ccc0ce8fa8406a695d53956e2080236ca39`
- Credential-store liveness implementation: `458ac28b001ceb2f26b5aa5df41c7041cb6f462d`
- Provider compatibility implementation: `004556c527b69b42ab523a86822067ec86764edb`
- Startup/shared-recovery implementation: `247d5bbbcb5278f7f9901de8fd7101e6a56f3491`
- Final review remediation: `a935e507b0173f4ee9c1f0aa539eea6e24ed200f`
- Final local debug binary SHA-256: `23f37955e3e18f943e01e14dd3271db0e8811ec6924db78ea962218fdd8d33d2`

Both the latest-main baseline and the PF-48–PF-56 feature tip are ancestors of
the final candidate. The integration used a merge rather than a rebase so the
archived sprint commit identities remain valid.

## Product result

- Locked OS-keyring operations have a shared bound, per-operation execution
  deadline, stuck-worker circuit, and late-success recovery.
- Provider-vault mutations use one encrypted-file batch per logical operation,
  including managed Claude token enrollment, without changing the vault format
  or scrypt work factor.
- Development/test crypto runs no longer make the true-TUI harness exceed its
  deadline; the production cryptographic contract is unchanged.
- Command-auth, local, status-only, and credential-free custom providers remain
  lazy and selectable at their real runtime boundary.
- Fresh unconfigured profiles enter onboarding. Established inactive,
  unavailable, recovery-required, or removed-provider profiles enter chat with
  requests blocked until explicit `/providers` recovery; no provider is silently
  substituted.
- Startup and in-app provider setup both support explicit recovery selection.
  In-app selection obtains a compatible model before mutating completion state.
- Onboarding caches provider status and refreshes it only on state changes, so
  rendering and key navigation do not repeatedly decrypt the vault or query the
  keyring.

## Automated evidence

All commands used `CARGO_INCREMENTAL=0` and the external target directory
`/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/pf57`.

| Boundary | Result |
| --- | --- |
| Final build | PASS: `cargo build -p codex-cli` |
| Keyring store | PASS: 3/3 |
| Login | PASS: 153 unit + 37 integration |
| Model-provider information | PASS: 60/60 |
| Provider-auth contracts | PASS: 64/64 |
| Secrets | PASS: 18/18 |
| Vault | PASS: 55/55 |
| Core/provider total | PASS: 390 tests |
| Startup provider policy | PASS: 10/10 |
| Provider setup state machine | PASS: 14/14 |
| Onboarding auth | PASS: 27/27 |
| Onboarding screen | PASS: 9/9 |
| Provider status host | PASS: 10/10 |
| Shared selection model fallback | PASS: 1/1 |
| Shared noninteractive provider UI event | PASS: 1/1 |
| Formatting/diff | PASS: `cargo fmt`; `git diff --check` |

The combined login run that originally took 57.3 seconds fell to 1.736 seconds
after batching; provider bulk deletion fell from 48.2 seconds to 1.318 seconds.
Those measurements are development-profile liveness evidence, not a change to
the cryptographic work factor.

## True-TMUX evidence

- Final local exact runs on `a935e507b`: missing-current recovery PASS in
  25.01s; inactive-current cancellation PASS in 26.79s.
- Final remote Fable-controlled run on `a935e507b`: all 12 PF-55 convergence
  journeys plus PF-53 configure-many and only-Plan-cancel passed in 147.84s.
- Earlier combined-candidate checks also passed configure-many/default/restart
  (128.15s) and command-auth visible/lazy/no-enrollment (30.80s).

The matrix covers fresh install, established upgrade, environment and managed
credentials, inactive current/noncurrent providers, exact replacement,
missing-current blocking, restart/resume, native spawn, command auth, duplicate
model slugs, and deferred onboarding behavior.

## Review evidence

- Structured autoreview of the final remediation returned no actionable
  findings: `patch is correct` at confidence 0.82.
- Claude Fable 5.1 Plan at max effort ran through Corbanu Terminal in TMUX on
  the remote Linux host. Its first review produced three actionable findings;
  the final exact-commit review verified each remediation and ended
  `FINAL_REREVIEW: CLEAN`.
- Detailed disposition: [fable-review.md](fable-review.md).

## Remaining plan-level gates

PF-57 implementation and integration qualification are complete. The active
plan remains open for named human acceptance, live eligible production-account
evidence, applicable physical platform confirmation, target release/tag and
release-ledger decisions, upstream disposition, and benchmark due-state. These
release gates are not represented as passed by this sprint.
