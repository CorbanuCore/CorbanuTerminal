# PF-56 final qualification ledger

Date: 2026-09-02 UTC

## Candidate

- Branch: `feat/unified-provider-auth`
- Last signed scope checkpoint: `889cd68c99c22bf387c258c84ccbf23e5ceebac6`
- Product version: `0.1.35`
- Binary: `codex-rs/target/debug/codex`
- SHA-256: `c1a444f2f882cca9c2739dd27f2a692f284a1e498f299bdb1683a10e06d807a0`
- Mtime: `2026-09-02 22:20:58.565813213 +0000`
- Size: `1379560656` bytes
- Product edits after freeze: none. The later provider-management change is
  test-only navigation synchronization and does not alter the binary.

## Automated evidence

| Boundary | Command or filter | Result |
| --- | --- | --- |
| Provider contracts | `CARGO_INCREMENTAL=0 cargo test -p codex-provider-auth` | PASS: 62 passed, 0 failed |
| Login and custody | `CARGO_INCREMENTAL=0 cargo test -p codex-login` | PASS: 152 unit and 37 integration tests, 0 failed |
| Startup exact identity | exact custom-current versus OpenAI login-status regression | PASS: 1/1 |
| Startup resolution | `startup_provider` focused suite | PASS: 5/5 |
| Shared status host | `provider_status_host` focused suite | PASS: 9/9 |
| Model catalog | model-catalog focused suite | PASS: 16/16 |
| Provider manager | provider-manager focused suite | PASS: 8/8 |
| Formatting and diff | `just fmt-check`; `git diff --check` | PASS |
| Governance | `python3 docs/plans/check.py`; `python3 docs/sprints/check.py` | PASS: active plans 2/2, current sprints 59 |
| File caps | scoped production and TMUX files | PASS: status 278/198/219/294 lines; TMUX 942/700/910 |
| Accepted artifact hashes | stable `pf53-*`, `pf54-*`, and `pf55-*` bundles | PASS: 28 hash records all equal the candidate SHA-256 |
| Accepted artifact canaries | credential-canary patterns across the 26 accepted bundles | PASS: no match |

Known failed diagnostic bundles are preserved separately. Some diagnostic
command logs intentionally contain synthetic environment fixtures; they are not
accepted qualification evidence and were excluded from the accepted-bundle
redaction pass.

## Final true-TMUX matrix

All tests ran strictly serially with `CORBANU_TMUX_REQUIRED=1` against the
candidate above. Stable bundles are under
`codex-rs/tui/target/tmux-artifacts/`.

| # | Journey | Result |
| ---: | --- | --- |
| 1 | Configure many; first default; restart; request | PASS 210.73s |
| 2 | Deferred Plan cancel with fallback | PASS 123.62s |
| 3 | Fresh wallet Plan success preserves current | PASS 185.33s |
| 4 | Fresh wallet Plan without fallback selects in session | PASS 110.71s |
| 5 | Locked wallet failure, retry, cancel | PASS 128.90s |
| 6 | Only-Plan cancel returns to provider list | PASS 60.38s |
| 7 | Current deactivation cancel is inert | PASS 31.32s |
| 8 | Exact replacement persists before deactivation | PASS 39.41s |
| 9 | Environment copy/eligibility never deletes credentials | PASS 30.64s |
| 10 | Noncurrent deactivate/reactivate/restart/credential request | PASS 216.88s |
| 11 | PF-50 API-key setup and recovery | PASS 93.04s |
| 12 | PF-51 OpenAI cancel and retry correlation | PASS 29.24s |
| 13 | PF-52 Claude recovery/cancel/retry | PASS 145.25s |
| 14 | Shared/custom management status parity | PASS 36.47s |
| 15 | Command auth visible, validated, no invented enrollment | PASS 39.98s |
| 16 | Custom environment provider host/picker/request convergence | PASS 41.03s |
| 17 | Duplicate model slug preserves exact provider | PASS 37.33s |
| 18 | Exact replacement restart and request | PASS 41.12s |
| 19 | Existing-install upgrade preserves current and request | PASS 36.47s |
| 20 | Fresh A then B preserves first success across restart | PASS 189.46s |
| 21 | Inactive-current cancel blocks request without switching | PASS 39.51s |
| 22 | Inactive noncurrent hidden from picker, then reactivates | PASS 41.85s |
| 23 | Managed custom key survives restart and request | PASS 45.53s |
| 24 | Missing-profile current never silently switches | PASS 35.74s |
| 25 | Native spawn and parent use exact custom runtime | PASS 40.30s |
| 26 | Resumed main session retains exact runtime | PASS 45.46s |

Before the final run, case 10 exposed a test-only navigation race: the helper
treated overlay text retained in terminal scrollback as live state and did not
reliably close the manager. The accepted helper uses a bounded deadline and the
active Corbanu footer among the last non-empty terminal rows. Its discriminator
passed the complete case in 223.14s, then the matrix restarted from case 1 and
passed 26/26. Failed artifacts remain separate.

## Disposable live-repository workflows

### TensorCash

- Worktree: `/home/pfrpc/repos/worktrees/pf56-tensorcash-dd6e9202`
- Base: `dd6e92024254090de0f596b090bd5c74c4d97b90`
- Pre/post status: clean.
- `cargo test --workspace`: PASS, 586 passed, 0 failed, 1 ignored
  operator-calibration benchmark.
- Frozen Corbanu candidate ran twice as separate processes from the TensorCash
  root with one isolated home and loopback provider.
- Both requests reached `/v1/responses`, used exact model `fixture-model`,
  returned the asserted responses, and reported `authorized=true` without
  logging the synthetic credential.
- Temporary controller output: `/tmp/tmp.MXhcwiaQwz`.

### Isometric Game

- Worktree: `/home/pfrpc/repos/worktrees/pf56-isometricgame-59821b7a`
- Base: `59821b7a85524f186f946c4670480c7ee96483cb`
- Pre/post status: clean.
- `npm run check`: PASS for `worlds/sample/world.json` and
  `worlds/ash-ward/world.json`.
- `npm run test:math`: PASS (`math ok`).
- Frozen Corbanu candidate ran from the Isometric root, persisted a session,
  exited, and resumed it with `exec resume --last`.
- The same thread ID was retained; request input grew from 3 to 5 items; both
  requests used exact model `fixture-model`, returned asserted responses, and
  reported `authorized=true`.
- Temporary controller output: `/tmp/tmp.2V6tHyvzpR`.

An earlier TensorCash controller attempt incorrectly passed interactive-only
`-a` to `codex exec`; it made no request. Its unconditional trailing marker
is rejected. The accepted rerun used `set -euo pipefail` and typed
`approval_policy="never"`.

## Review evidence

- Sole completed external reviewer: Kimi 3.0 high,
  `moonshotai/kimi-k3`, through Vercel in TMUX, with no fallback.
- Runtime, controller, output, exit, and dispositions:
  `qa/provider-auth/pf-56/review/`.
- The attempted Claude Fable review failed OAuth before inference and is
  retained only as superseded evidence.
- No additional external or automatic review was launched. The four-pass
  maximum was not exceeded.
- Final disposition: one false positive rejected with code/test evidence, three
  findings confirmed and fixed, plus one adjacent exact-identity defect found
  during qualification and repaired.

## Documentation and remaining release gates

The verified multi-provider behavior is documented in
`docs/features/model-providers.md` and `docs/authentication.md`. The existing
Claude Plan guide remains accurate.

Implementation qualification is green, but shipment remains blocked on evidence
that cannot be fabricated:

- named human acceptance;
- live eligible production-account runs;
- required physical Linux/macOS/Windows confirmation;
- final upstream integration disposition;
- target version, integration commit/merge/tag, and release ledger;
- benchmark due-state decision and evidence when required.

Primary integration audit found no blocking issue. Signed implementation commit
`fd8a9c900e37bfb83d7cfb62d0b347d978b14065` was verified and pushed to
`origin/feat/unified-provider-auth`; the completed PF-56 sprint record was then
archived. Shipment remains gated by the open active-plan evidence above.
