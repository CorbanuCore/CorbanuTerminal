# PF Terminal 0.1.27 preflight test evidence

This file records release-worktree evidence before the RC commit is frozen. It is
not a waiver for the clean-RC, live-route, platform, parity, or complete-workspace
gates in the release specification.

## Passing suites

| Area | Command or suite | Result |
| --- | --- | ---: |
| Installer | `python3 -m unittest scripts.install.test_install_sh` | 7/7 |
| Package builder | `python3 -m unittest discover -s scripts/codex_package -p 'test_*.py' -v` | 18/18 |
| Provider metadata | `codex-model-provider-info` | 53/53 |
| Provider runtime | `codex-model-provider` | 67/67 |
| Model catalogue | `codex-models-manager` | 57/57 |
| API adapters | `codex-api` | 196/196 |
| Protocol | `codex-protocol` | 272/272 |
| App-server protocol | `codex-app-server-protocol` | 287/287 |
| App server (host-capability filter) | `just test -p codex-app-server -E 'not (test(command_exec) or test(executor_skills) or test(standalone_image_edit_uses_attached_model_visible_image) or test(turn_interrupt_aborts_running_turn) or test(turn_start_file_change_approval_accept_for_session_persists_v2))' --test-threads 1` | 1,020/1,020; 45 filtered |
| Rollout | `codex-rollout` | 112/112 |
| Thread store | `codex-thread-store` | 161/161 |
| Tools | `codex-tools` | 89/89 |
| GPU rental | `just test -p codex-gpu-market --test-threads 1` | 73/73 |
| GPU TUI | `just test -p codex-tui gpu_menu --test-threads 1` | 8/8 |
| TUI | `just test -p codex-tui --test-threads 1` | 3,402/3,402; 4 skipped |
| State | `just test -p codex-state` | 190/190 |
| CLI (host-capability filter) | `just test -p codex-cli` with two host-only cases skipped | 593/593; 2 skipped |
| Telegram | `just test -p codex-telegram` | 119/119 |
| Wallet | `just test -p codex-wallet` | 11/11 |
| Wallet daemon | `just test -p codex-wallet-daemon` | 8/8 |
| Vault | `just test -p codex-vault` | 14/14 |
| Login | `just test -p codex-login` | 178/178 |
| Secrets | `just test -p codex-secrets` | 16/16 |
| Analytics | `just test -p codex-analytics` | 95/95 |
| Exec | `just test -p codex-exec` | 128/128 |
| Core truncation | `just test -p codex-core truncation --test-threads 1` | 49/49 |
| Shell snapshots | `just test -p codex-core shell_snapshot --test-threads 1` | 21/21 |
| Resume fixtures | `just test -p codex-core resume_includes_initial_messages --test-threads 1` | 3/3 |
| Config schema | `just write-config-schema`, followed by a second generation and SHA-256 comparison | idempotent |
| App-server schemas | stable and experimental `just write-app-server-schema`, followed by full-directory SHA-256 comparison | idempotent |
| Bazel lock | `just bazel-lock-update && just bazel-lock-check` | pass |
| Patch hygiene | `git diff --check` | pass |
| Secret scan | `python3 scripts/release/scan_release_secrets.py --fail-on-finding` | 0 unreviewed findings; 8 exact-fingerprint test fixtures |
| TUI snapshots | `cargo insta pending-snapshots --manifest-path tui/Cargo.toml` | no pending snapshots |
| Repository argument comments | `just argument-comment-lint` | 852/852 Bazel targets pass |

Focused model switching, exact-route identity headers, remote environments,
legacy notification, nested v2 spawn, v1 fork behavior, cache, budget, search,
Responses Lite, and compact/replay regressions also pass in this worktree.

## Host-capability exclusions under investigation

This container cannot create the bubblewrap loopback namespace:

```text
bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted
```

The product sandbox was not weakened to accommodate the host. Serial isolation
produced these results:

- `codex-core` hooks: 68/70 pass; the two failures require the unavailable
  namespace capability.
- `codex-core` unified exec: 118/122 pass; the four failures are the glob-deny,
  sandbox-launch, and two network-denial tests, each failing at the same bwrap
  boundary.

The same groups must run on a namespace-capable Linux RC host before release.

The final app-server preflight excluded the complete `command_exec` and
`executor_skills` modules, plus the attached-image, active-turn interruption,
and file-change-approval persistence cases. The earlier unfiltered run and
focused reruns identify 18 cases that actually cross the unavailable bwrap
boundary. The broader 45-test filter prevents a host limitation from obscuring
unrelated app-server regressions; it is not a release waiver, and the complete
1,065-test suite remains required on the RC host.

## Broad-run diagnostic

An eight-thread `codex-core` run executed 3,194 tests: 3,107 passed, 86 failed,
one timed out, and eight skipped. Serial reruns showed that truncation,
shell-snapshot, and unified-exec failures were caused by resource contention or
the host namespace restriction. One independent defect was discovered: a fast
large-output command could overflow the live broadcast channel and lose bytes
from its terminal event. The producer now records an undrained cumulative
transcript, and `unified_exec_formats_large_output_summary` passes. The broad
run is diagnostic evidence only; it does not satisfy the complete-workspace
gate.

The post-fix serial TUI run supersedes the earlier 3,383-pass diagnostic:
all 3,402 executed tests pass and its four platform-gated tests remain skipped.

## Required remaining evidence

- clean RC commit and reproducible release build;
- namespace-capable Linux sandbox suites;
- approved complete workspace test;
- formatting and the frozen-RC/artifact repeat of the zero-hit secret scan;
- live route, switching, orchestration, Anthropic endurance, and capped GPU
  qualification;
- equal-count OpenAI upstream parity;
- Linux, macOS, and Windows install/upgrade/rollback matrix;
- artifact hashes, signatures, and public prerelease install proof.
