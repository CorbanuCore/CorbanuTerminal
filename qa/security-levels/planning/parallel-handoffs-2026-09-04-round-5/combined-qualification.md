# Combined round-five qualification handoff

Read-only testing handoff to integration owner. No new reviews invoked.

## Final integration qualification — supersedes the earlier candidate below

- Exact final tested commit: `dd2adb72b967602c6851bb4f4257eb34ad135459`.
- This includes provenance realtime guard remediation `e592cf75a`.
- Remote worktree remains `/home/travis/worktrees/security-round5-integration`.
- Fresh TMPDIR: `/home/travis/security-round5/integration-tmp/run.hnpBE0`.
- Core `just fix -p codex-core` and full `just fmt` passed; formatter patch is
  zero bytes and final worktree is clean. No lockfile drift.
- Focused Core provenance/realtime/broker/proxy: **94/94 pass**, 3381 filtered,
  run `681a2235-1fda-4d80-ad64-b507697e41d5`, 11.343 seconds.
- Filter: `test(pf_30_s01) | test(realtime_conversation) | test(broker_client) | test(network_proxy_credential)`.
- `cargo build --locked -p codex-cli --bin codex` passed.
- All three actual-key TMUX journeys passed, 71 filtered, run
  `876120a7-84c0-468b-9f92-f2fa84a55770`, 12.623 seconds.
- Plan/sprint governance and diff check passed; shared build lock released.

Final evidence root: `/home/travis/security-round5/evidence/integration/dd2adb72b/`.
Final immutable launch path (RTX Linux, **not** the Mac shortcut):
`/home/travis/security-round5/evidence/integration/dd2adb72b/candidate/codex`.
Version: `corbanu 0.1.38`.
SHA-256: `6af243a3a249ddb44f3344a40aeeb5e21dbbc5e0b156167577db4d848c806ab3`.
The copied candidate and actual tested shared debug binary have identical hashes
after the TMUX run. Seven final safe captures were inspected under `tmux/`.
Launch from the intended test directory using the absolute candidate path.

The unchanged protocol 285/285, broker/vault/proxy 338/338 and UI-unit 235/235
evidence below is reused explicitly; those full suites were not rerun after the
Core-only realtime guard change. No new review was run for this integration
delta. This does not assess the adjacent memory-worker gap being investigated
by the integration owner, nor claim human/live/platform/release acceptance.

## Earlier combined qualification (retained evidence)

## Exact tree

- Remote worktree: `/home/travis/worktrees/security-round5-integration`.
- Checked-out commit: `f60d15f16589bd634c52e717feb28b27158c4144`.
- Only content delta: alphabetical `broker_client` / `browser_isolation` module
  order in `codex-rs/core/src/security/mod.rs`. Coordinator already committed
  the identical change in `447c6b336`; this is the tested Rust source equivalent.
- Initial generated Cargo.lock delta preserved as stash `integration-precombined-lock`.
- Shared build lock held throughout fix/format and each test/build sequence.
- Fresh TMPDIRs: `integration-tmp/run.TJ1Mrz`, then `integration-tmp/run.s8xsWO`.

## Passing gates

| Gate | Result | Nextest run |
| --- | --- | --- |
| Fix six affected crates | Pass, existing warnings | `fix.log` |
| Complete `just fmt` | Pass, only module-order delta above | `fmt.log`, `formatter.patch` |
| Protocol full | 285/285, no skips | `0f1c3e65-c0e2-44c0-97b9-97cc9ef9bad6` |
| Core provenance + broker/proxy | 26/26, 3447 filtered | `fcb94756-d282-46d1-9d92-2ffd3b57e473` |
| Full broker / vault / proxy | 338/338, no skips | `dafed2b5-a0e8-4fd6-aa49-b07c84879fb8` |
| Focused security UI / status / slash | 235/235, 3737 filtered | `b97561ce-4bde-4ac2-b604-9aa7b37448c5` |
| Actual-key TMUX | 3/3, 71 filtered | `1000d61e-8049-40df-9baf-12867936fb50` |
| Plan/sprint governance, diff check | Pass | End of qualification session |

Core filter: `test(pf_30_s01) | test(broker_client) | test(network_proxy_credential)`.
UI filter: `test(security_view) | test(status::tests) | test(slash_command)`.
TMUX filter: `test(security_profiles) | test(tmux_smoke_single_enter_dispatches_slash_command_and_exits_cleanly)`.
All tests use `just test`, no retries; no local compilation.

## Candidate and interactive evidence

Evidence root: `/home/travis/security-round5/evidence/integration/f60d15f16/`.
Immutable candidate: `candidate/codex`, Corbanu 0.1.38.
SHA-256: `3318bfe3fd0b9ec0edf16078288a95f5ed4e2f835deb5d54e17c79e0cfa88aff`.
Copied while the build lock was held; subsequent lane builds cannot overwrite it.

Seven safe captures under `tmux/` were inspected: three profile views and their
matching status at 120/40/80 columns, plus unknown-configuration startup error.
Actual keys exercise navigation, inert Enter, Esc/reopen, unchanged config,
status and clean exit. Synthetic credentials and temporary homes only; trace
logging enabled. Captures show configuration-only requests, unverified effective
protection, blocked unqualified protected modes, and no activation claims.

## Environment recovery and limits

The first Core compile reported missing network-proxy exports even though all
five existed in source: known cross-worktree shared-target stale artifacts.
After authorized timestamp refresh of workspace Rust files, the same source
compiled and all 26 focused cases passed. Original failure preserved in
`core-focused.log`; successful rerun is `core-focused-cache-recovery.log`.
No source content was changed for cache recovery.

A completed broker Bazel server retained build.lock through inherited FD3.
Broker owner used normal `bazel shutdown`, exit 0; no process was force-killed.

This run predates the pending realtime guard remediation. Rebase and rerun its
affected paths after integration. Full Core baseline comparison belongs to the
provenance lane; this handoff does not claim a full Core or protected-mode pass.
No human, live-repository, cross-platform, benchmark or release sign-off claimed.
