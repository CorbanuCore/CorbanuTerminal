# Test-profile / native-Keychain isolation repair — September 14, 2026

Class: bounded test-infrastructure fix; no production authentication, vault,
permissions, credential migration, or release behavior change. No new product
initiative or sprint. User authorized the repair and durable project guidance.
Product reference: **Non-negotiable controls**, “Default to no secret export,
arbitrary egress, clipboard exposure, or sensitive logging.”

Worktree: `worktrees/test-keychain-isolation-20260914`.
Branch: `fix/test-keychain-isolation-20260914`.
Base: `ff1d3a433` on `integrate/management-workstreams-20260911`.

## Evidence and cause

Read-only macOS securityd diagnostics showed 100 prompt requests from 100
development `codex-app-server` process IDs, 04:30:49–04:52:33 Arizona time.
The first overnight PF-83 verification log recorded native-store timeouts for
the account derived from the live Corbanu profile. The app-server fixture set
`CODEX_HOME` but inherited higher-priority `CORBANU_HOME`; the debug native-store
guard was not forced. Later clicks accepted some requests, while others remained
queued. This supersedes the earlier inference that unlocking the Mac was the
sufficient remedy. It does not establish credential exfiltration or that every
subsequent provider/git failure had the same cause.

The initiating test processes were no longer running during diagnosis. No
Keychain reads of secret values, ACL changes, password entry, process shutdown,
or application replacement were performed by this repair. Existing contaminated
results are not qualification evidence for PF-83.

## Scope and proof

Owner boundary: ordinary test launcher, app-server fixture process environment,
regression tests and operator/agent documentation. Production native-keyring
implementation is unchanged. Raw/release/native tests require the separate
isolated qualification lane, not a debug flag advertised as an OS sandbox.

- [x] Python runner regressions: 7 passed on final tree.
- [x] `just test -p app_test_support -p codex-keyring-store --locked --offline`:
  16 passed, 2 skipped. Nextest reported one leaky existing helper test on each
  support-suite run (different helpers); retained as a limitation, not silently
  counted as warning-free. The new profile regressions and native-denial guard
  passed. No native credential-value retrieval was performed.
- [x] Real app-server startup/configuration: 3 passed, repeated with all three
  inherited profile aliases pointing at a synthetic invalid-TOML canary and
  the inherited guard set to `0`: 3 passed again. Canary remained unchanged;
  no files were added to it. Final canary run ID:
  `027114d7-907f-4131-b29f-0eb3cf7a0648`.
- [x] Independent autoreview: Astra High, read-only, no test/credential execution.
  First pass found a P1 clustered `-r` bypass (`-vr`, `-rj4`); accepted and fixed
  with regressions. Second pass: no accepted/actionable findings, patch correct.
  Command: `python3 /Users/Neo/.codex/skills/autoreview/scripts/autoreview --mode local --engine codex --model gpt-6-astra --thinking high --codex-bin /Applications/ChatGPT.app/Contents/Resources/codex --no-web-search`.
  Raw review results retained privately in the workspace's
  `.codex-work/test-keychain-integration.MaYGV4/`.

`just fmt` ran; unrelated pre-existing formatter drift was excluded from this
bounded patch. `git diff --check` and sprint governance passed. Bounded securityd
inspection covering verification found zero new app-server Keychain prompt
requests. This is observed evidence for these runs, not a claim that environment
flags sandbox arbitrary programs. The operator's running Corbanu PID 86043 was
left running. macOS is the executed platform; Windows/Linux behavior is not
claimed as natively qualified by these results.

TUI/code-blind user-functional execution, live trading repositories and release
benchmarks: not applicable to this internal launcher repair; no user interaction
or production package changed. This does not close PF-83's separate functional
qualification gate. No production install/restart or human release sign-off.

Operational procedure: [safe automated tests](../../docs/development/test-isolation.md).
